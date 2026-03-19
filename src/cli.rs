//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use std::path::PathBuf;
use clap::{Subcommand, Parser as ClapParser};
use anyhow::Result;

#[derive(ClapParser, Debug)]
#[command(name = "rust-log-etl")]
#[command(about = "高性能日志ETL工具")]
#[command(version = "0.1.0")]
pub struct Args {
    #[command(subcommand)]
    pub mode: RunMode
}

#[derive(Subcommand, Debug)]
pub enum RunMode {
    // 使用配置文件运行
    Config {
        #[arg[short, long]]
        config: PathBuf
    },

    // 直接使用命令运行
    Run {
        #[arg[short, long]]
        input: PathBuf,

        #[arg[short, long]]
        output: PathBuf,

        #[arg[short, long, default_value = "nginx"]]
        parser: String,

        #[arg[short, long, default_value = "json"]]
        format: String,

        #[arg[short, long, default_value = "4"]]
        workers: usize,
    }
}

#[derive(Debug)]
pub struct RuntimeConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub parser_type: String,
    pub output_format: String,
    pub workers: usize
}

impl RuntimeConfig {
    /// 从配置文件加载
    pub fn from_config_file(path: &PathBuf) -> Result<Self> {
        use crate::config::{Config, InputConfig, ParserConfig, SinkConfig};

        let config = Config::from_file(path)?;

        let (input_path, _) = match &config.input {
            InputConfig::File { path, checkpoint_file: _ } => (path.clone(), ()),
        };

        let parser_type = match &config.parser {
            ParserConfig::Nginx { .. } => "nginx".to_string(),
            ParserConfig::Json { .. } => "json".to_string(),
            ParserConfig::Regex { .. } => "regex".to_string(),
        };

        let (output_path, output_format) = match &config.sink {
            SinkConfig::File { path, format, .. } => (path.clone(), format.clone()),
            SinkConfig::Kafka { .. } => anyhow::bail!("Kafka输出请使用配置文件模式"),
        };

        Ok(Self {
            input_path,
            output_path,
            parser_type,
            output_format,
            workers: config.global.workers,
        })
    }

    /// 从命令行参数构建
    pub fn from_args(input: PathBuf, output: PathBuf, parser: String, format: String, workers: usize) -> Self {
        Self {
            input_path: input,
            output_path: output,
            parser_type: parser,
            output_format: format,
            workers,
        }
    }
}

