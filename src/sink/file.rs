//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use tokio::fs::OpenOptions;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::io::{AsyncWriteExt, BufWriter};
use anyhow::{Result, Context};
use async_trait::async_trait;
use crate::core::{ParsedRecord, Sink};

pub struct FileSink {
    writer: Mutex<BufWriter<tokio::fs::File>>,
    format: OutputFormat,
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    /// 原始字符串
    Raw,
    /// json格式
    Json,
    /// csv 格式
    Csv,
    /// 自定义分隔符
    Delimited {delimiter: char, fields: Vec<String>}
}

impl FileSink {
    pub async fn new(path: PathBuf, format: OutputFormat) -> Result<Self>{
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .with_context(|| format!("无法打开文件: {}", path.display()))?;
        Ok(Self{
            writer: Mutex::new(BufWriter::new(file)),
            format
        })
    }

    /// 将记录格式化为字符串
    fn format_record(&self, record: &ParsedRecord) -> Result<String> {
        match &self.format {
            OutputFormat::Raw => {
                let raw = record.fields.iter()
                    .find(|(k, _)| k == "raw")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                Ok(raw)
            }

            OutputFormat::Json => {
                let mut map = serde_json::Map::new();

                if let Some(ts) = record.timestamp {
                    map.insert("timestamp".to_string(), serde_json::json!(ts.to_rfc3339()));
                }

                for (k, v) in &record.fields {
                    map.insert(k.clone(), serde_json::json!(v));
                }
                let json = serde_json::to_string(&map)?;

                Ok(json)
            }

            OutputFormat::Csv => {
                let values: Vec<&str> = record.fields.iter()
                    .map(|(_, v)| v.as_str())
                    .collect();

                Ok(values.join(", "))
            }

            OutputFormat::Delimited {delimiter, fields} => {
                let values: Vec<String> = fields.iter()
                    .map(|f| {
                        record.fields.iter()
                            .find(|(k, _)| k==f)
                            .map(|(_, v)| v.clone())
                            .unwrap_or_default()
                    }).collect();

                Ok(values.join(&delimiter.to_string()))
            }
        }
    }
}

#[async_trait]
impl Sink for FileSink {
    async fn write(&self, record: &ParsedRecord) -> Result<()> {
        let line = self.format_record(record)?;
        let mut writer = self.writer.lock().await;

        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.flush().await.context("无法刷新文件");
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.flush().await?;
        Ok(())
    }
}