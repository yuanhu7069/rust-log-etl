//! sink处理核心逻辑
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0


use crate::core::{Parser, TransformerChain, Sink, Transformer};
use anyhow:: Result;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;
use log::{error, info};
use crate::error::{EtlError, LineResult};

#[derive(Default, Debug)]
pub struct ProcessStats {
    pub total_lilnes: u64,
    pub parsed_ok: u64,
    pub parsed_failed: u64,
    pub filtered: u64,
    pub written: u64,
}

pub struct Pipeline {
    parser: Box<dyn Parser>,
    transformer: TransformerChain,
    sink: Box<dyn Sink>,
    dlq_sender: Option<mpsc::Sender< String>>
}

impl Pipeline {
    pub fn new(
        parser: Box<dyn Parser>,
        transformers: Vec<Box<dyn Transformer>>,
        sink: Box<dyn Sink>,
        dlq_sender: Option<mpsc::Sender< String>>
    ) -> Self {
        Self {
            parser,
            transformer: TransformerChain::new(transformers),
            sink,
            dlq_sender,
        }
    }

    async fn process_line(&self, line: String) -> LineResult {
        // 解析
        let record = match self.parser.parse(&line) {
            Ok(r) => r,
            Err(e) => {
                return LineResult::Failed {
                    line: line.clone(),
                    error:EtlError::ParseError {
                        line: line.clone(),
                        reason: e.to_string()
                    }
                }
            }
        };
        // 转换
        match self.transformer.transform(record) {
            Ok(Some(r)) => LineResult::Successs(r),
            Ok(None) => LineResult::Filtered,
            Err(e) => LineResult::Failed {
                line: line.clone(),
                error: EtlError::Transform(e.to_string()),
            }
        }
    }

    /// 发送line到死信队列
    async fn send_to_dlq(&self, line: &str, error: &EtlError) {
        if let Some(sender) = &self.dlq_sender {
            let dlq_line = format!("{}|{}\n", line, error);
            let _ = sender.try_send(dlq_line);
        }
    }

    pub async fn run<R: AsyncBufReadExt + Unpin> (&self, reader: R) -> Result<ProcessStats> {
        let mut lines = reader.lines();
        let mut stats = ProcessStats::default();

        while let Ok(Some(line)) = lines.next_line().await {
            stats.total_lilnes += 1;
            match self.process_line(line.clone()).await {
                LineResult::Successs(record) => {
                    stats.parsed_ok += 1;
                    if let Err(e) = self.sink.write(&record).await {
                        self.send_to_dlq(&line, &EtlError::Sink(e.to_string())).await;
                        stats.parsed_failed += 1;
                    } else {
                        stats.written += 1;
                    }
                },
                LineResult::Filtered => {
                    stats.filtered += 1;
                },
                LineResult::Failed { line: failed_line, error } => {
                    stats.parsed_failed += 1;
                    self.send_to_dlq(&failed_line, &error).await;
                }
            }
        }
        if let Err(e) = self.sink.flush().await {
            error!("flush sink error: {}", e)
        }
        Ok(stats)
    }
}

