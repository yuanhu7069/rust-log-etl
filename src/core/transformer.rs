//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use crate::core::parser::ParsedRecord;
use anyhow::Result;

pub trait Transformer: Send + Sync {
    fn transform(&self, record: ParsedRecord) -> Result<Option<ParsedRecord>>;
}

/// 链式组合多个转换器
pub struct TransformerChain {
    transformers: Vec<Box<dyn Transformer>>,
}

impl TransformerChain {
    pub fn new(transformers: Vec<Box<dyn Transformer>>) -> Self {
        Self {transformers}
    }
}

impl Transformer for TransformerChain {
    fn transform(&self, mut record: ParsedRecord) -> Result<Option<ParsedRecord>> {
        for t in &self.transformers {
            match t.transform(record) ? {
                Some(r) => record = r,
                None => return Ok(None),
            }
        }

        Ok(Some(record))
    }

}