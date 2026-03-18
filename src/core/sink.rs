//! sink处理核心逻辑
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0


use async_trait::async_trait;
use crate::core::parser::ParsedRecord;
use anyhow::Result;

/// 输入端统计信息
#[derive(Debug, Default)]
pub struct SinkStats {
    pub records_written: u64,
    pub bytes_written: u64,
    pub errors: u64,
}

#[async_trait]
pub trait Sink: Send + Sync {
    /// 单条记录写入
    async fn write(&self, record: &ParsedRecord) -> Result<()>;

    /// 批量写入
    async fn write_batch(&self, records: &[ParsedRecord]) -> Result<SinkStats> {
        let mut stats = SinkStats::default();
        for r in records {
            match self.write(r).await {
                Ok(_) => stats.records_written += 1,
                Err(_) => stats.errors += 1,
            }
        }
        Ok(stats)
    }

    async fn flush(&self) -> Result<()>;

    /// 关闭连接
    async fn close(&self) -> Result<()> {
        Ok(())
    }

}