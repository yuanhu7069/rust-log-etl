//! 死信队列模块
//!
//! @author yuanhu
//! @created 2026/3/19 15:21
//! @version 1.0.0

use tokio::sync::mpsc;
use anyhow::Result;
use log::info;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub struct DeadLetterQueue {
    receiver: mpsc::Receiver<String>,
    path: std::path::PathBuf,
}

impl DeadLetterQueue {
    pub fn new(path: std::path::PathBuf, buffer_size: usize) -> (Self, mpsc::Sender<String>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (Self { receiver, path },  sender)
    }

    pub async fn run(mut self) -> Result<u64> {
        let mut file = OpenOptions::new()
            .create(true).append(true).open(&self.path).await?;

        let mut count = 0u64;
        while let Some(line) = self.receiver.recv().await {
            file.write_all(line.as_bytes()).await?;
            count+=1;
        }
        file.flush().await?;
        info!("死信队列写入完成：{}条", count);
        Ok(count)
    }
}