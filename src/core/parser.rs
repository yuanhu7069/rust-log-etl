
use anyhow::Result;
pub trait Parser: Send + Sync {
    fn parse(&self, line: &str) -> Result<ParsedRecord>;
}

pub struct ParsedRecord {
    pub timestamp : Option<chrono::DateTime<chrono::Utc>>,
    pub fields: Vec<(String, String)>,
}

