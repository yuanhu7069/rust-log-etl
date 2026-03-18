//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use crate::core::{ParsedRecord, Parser};
use anyhow::Result;
use serde_json::Value;

pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for JsonParser {
    fn parse(&self, line: &str) -> Result<ParsedRecord>{
        let json: Value = serde_json::from_str(line)?;

        let mut fields = Vec::new();
        if let Value::Object(map) = json {
            for (k, v) in map {
                let v_str = match v {
                    Value::String(s) => s,
                    other => other.to_string()
                };
                fields.push((k, v_str));
             }
        }

        let timestamp = fields.iter()
            .find(|(k, _)| k == "timestamp" || k == "time")
            .and_then(|(_, v)| chrono::DateTime::parse_from_rfc3339(v).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        Ok(ParsedRecord {fields, timestamp})
    }
}