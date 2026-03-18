use crate::config::EnrichType;
use crate::core::{ParsedRecord, Transformer};
use anyhow::Result;

pub struct EnrichTransformer {
    field: String,
    source_field: String,
    enrich_type: EnrichType,
}

impl EnrichTransformer {
    pub fn new(field: String, source_field: String, enrich_type: EnrichType) -> Result<Self> {
        Ok(Self {
            field,
            source_field,
            enrich_type,
        })
    }
}

impl Transformer for EnrichTransformer {
    fn transform(&self, record: ParsedRecord) -> Result<Option<ParsedRecord>>  {
        Ok(Some(record))
    }
}