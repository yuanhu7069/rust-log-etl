//! sink处理核心逻辑
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0


use crate::core::{Parser, TransformerChain, Sink, Transformer};
use anyhow:: Result;
use tokio::io::AsyncBufReadExt;

pub struct Pipeline {
    parser: Box<dyn Parser>,
    transformer: TransformerChain,
    sink: Box<dyn Sink>,
}

impl Pipeline {
    pub fn new(
        parser: Box<dyn Parser>,
        transformers: Vec<Box<dyn Transformer>>,
        sink: Box<dyn Sink>,
    ) -> Self {
        Self {
            parser,
            transformer: TransformerChain::new(transformers),
            sink,
        }
    }

    pub async fn run<R: AsyncBufReadExt + Unpin> (&self, reader: R) -> Result<u64> {
        let mut lines = reader.lines();
        let mut count = 0u64;

        while let Ok(Some(line)) = lines.next_line().await {
            // 1 解析
            let record = match self.parser.parse(&line) {
                Ok(r) => r,
                Err(e) => {
                    println!("Error parsing line: {}", e);
                    continue;
                }
            };

            // 2 转换
            let record = match self.transformer.transform(record)? {
                Some(r) => r,
                None => continue,
            };

            // 3 写入
            self.sink.write(&record).await?;
            count += 1;
        }
        self.sink.flush().await?;
        Ok(count)
    }
}

