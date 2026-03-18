//! 处理 filter
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use crate::config::{FilterOperator, FilterValue};
use anyhow::Result;
use crate::core::{ParsedRecord, Transformer};

pub struct FilterTransformer {
    field: String,
    operator: FilterOperator,
    value: FilterValue,
}

impl FilterTransformer {
    pub fn new(field: String, operator: FilterOperator, value: FilterValue) -> Result<Self> {
        Ok(Self {field, operator, value})
    }
}

impl Transformer for FilterTransformer {
    fn transform(&self, record: ParsedRecord) -> Result<Option<ParsedRecord>> {
        Ok(Some(record))
    }
}
