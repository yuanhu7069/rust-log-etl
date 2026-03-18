//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_workers")]
    pub workers : usize,
    #[serde(default = "default_batch_size")]
    pub batch_size : usize,
}

fn default_workers() -> usize {
    4
}

fn default_batch_size() -> usize {
    1000
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InputConfig {
    File {
        path: PathBuf,
        #[serde(default)]
        checkpoink_file: Option<PathBuf>
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ParserConfig {
    #[serde(rename = "nginx")]
    Nginx {
        #[serde(default)]
        custom_regex: Option<String>
    },

    #[serde(rename = "json")]
    Json {
        #[serde(default)]
        field_map : Option<HashMap<String, String>>
    },

    #[serde(rename = "regex")]
    Regex {
        pattern: String,
        fields: Vec<String>
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum TransformerConfig {
    #[serde(rename = "filter")]
    Filter {
        field: String,
        operator: FilterOperator,
        value: FilterValue
    },

    #[serde(rename = "enrich")]
    Enrich {
        field: String,
        source_field: String,
        enrich_type: EnrichType
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    In,
    NotIn,
    Contains,
    Regex,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum FilterValue {
    Single(String),
    List(Vec<String>),
    Number(f64),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum EnrichType {
    #[serde(rename = "time_format")]
    TimeFormat { format: String },
    #[serde(rename = "url_parse")]
    UrlParse { part: UrlPart },
    #[serde(rename = "ip_geo")]
    IpGeo, // 阶段3：IP 转地域
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UrlPart {
    Domain,
    Path,
    Query,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SinkConfig {
    File {
        path: PathBuf,
        #[serde(default = "default_format")]
        format: String,
        #[serde(default)]
        rotate_size: Option<String>,
        #[serde(default)]
        rotate_time: Option<String>,
    },

    Kafka {
        brokers: Vec<String>,
        topic: String,
        #[serde(default = "default_batch_size")]
        batch_size: usize,
    }
}

fn default_format() -> String {
    "json".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub input: InputConfig,
    pub parser: ParserConfig,
    #[serde(default)]
    pub transformers: Vec<TransformerConfig>,
    pub sink: SinkConfig,
}


impl Config {

    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;

        let config :Config = toml::from_str(&content)
            .context("配置文件格式错误")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        match &self.input {
            InputConfig::File { path, .. } => {
                if !path.exists() {
                    anyhow::bail!("输入文件不存在: {}", path.display());
                }
            }
        }

        // 检查输出目录可写
        match &self.sink {
            SinkConfig::File { path, .. } => {
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        anyhow::bail!("输出目录不存在: {}", parent.display());
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_parse_log_config() {
        let config_str = r#"
        [global]
workers = 4
batch_size = 1000

[input]
type = "file"
path = "/var/log/nginx/access.log"

[parser]
type = "nginx"

[[transformers]]
type = "filter"
field = "status"
operator = "not_in"
value = [404, 500]

[sink]
type = "file"
path = "/var/log/output/processed.log"
format = "json" "#;
        let config = toml::from_str::<Config>(config_str).unwrap();
        assert_eq!(config.global.workers, 4);
        assert!(matches!(config.parser, ParserConfig::Nginx { .. }))
    }
}




