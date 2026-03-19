//! pipeline 构建
//!
//! @author yuanhu
//! @created 2026/3/17 17:04
//! @version 1.0.0

use anyhow::{Context, Result};
use crate::{Config, ParserConfig, Pipeline, NginxParser, Parser, TransformerConfig, Transformer, SinkConfig, JsonParser, FilterTransformer, EnrichTransformer, OutputFormat, FileSink};
use crate::cli::RuntimeConfig;
use crate::config::{parse_filter_value, InputConfig};
use crate::core::{Sink};

pub struct PipelineBuilder ;

impl PipelineBuilder {

    pub async fn build_from_runtime(config: &RuntimeConfig) -> Result<Pipeline> {
        let parser: Box<dyn Parser> = match config.parser_type.as_str() {
           "nginx" => Box::new(NginxParser::new()),
            "json" => Box::new(JsonParser::new()),
            _ => anyhow::bail!("不支持的解析器类型: {}", config.parser_type),
        };

        let output_format = match config.output_format.as_str() {
            "json" => OutputFormat::Json,
            "raw" => OutputFormat::Raw,
            "csv" => OutputFormat::Csv,
            _ => panic!("不支持的输出格式"),
        };

        // let transformers = Self::build_transformers(&config.transformers);
        let sink = Box::new(FileSink::new(config.output_path.clone(), output_format).await?);

        Ok(Pipeline::new(parser, vec![], sink))
    }

    pub async fn build(config: &Config) -> Result<Pipeline>{
        let parser = Self::build_parse(&config.parser)?;
        let transformers = Self::build_transformers(&config.transformers)?;
        let sink = Self::build_sink(&config.sink).await?;

        Ok(Pipeline::new(parser, transformers, sink))
    }

    fn build_parse(config : &ParserConfig) -> Result<Box<dyn Parser>> {
        match config {
            ParserConfig::Nginx { custom_regex} => {
                if let Some(_regex) = custom_regex {
                    todo!("自定义Nginx规则")
                }
                Ok(Box::new(NginxParser::new()))
            }

            ParserConfig::Json { field_map: _} => {
                Ok(Box::new(JsonParser::new()))
            }

            ParserConfig::Regex { pattern, fields } => {
                todo!("正则解析")
            }
        }
    }

    fn build_transformers(configs: &[TransformerConfig]) -> Result<Vec<Box<dyn Transformer>>> {
        let mut transformers: Vec<Box<dyn Transformer>> = Vec::new();
        
        for cfg in configs {
            let t: Box<dyn Transformer> = match cfg {
                TransformerConfig::Filter {
                    field, operator, value
                } => {
                    let filter_value = parse_filter_value(value)?;
                    Box::new(FilterTransformer::new(field.clone(), operator.clone(), filter_value)?)
                }
                TransformerConfig::Enrich { 
                    field, source_field, enrich_type
                } => Box::new(EnrichTransformer::new(field.clone(), source_field.clone(), enrich_type.clone())?) ,
                
            };
            transformers.push(t);
        }

        Ok(transformers)
    }

    async fn build_sink(config: &SinkConfig) -> Result<Box<dyn Sink>> {
        match config {
            SinkConfig::File {
                path, format, rotate_size, rotate_time,
            } => {
                let output_format = match format.as_str() {
                    "json" => OutputFormat::Json,
                    "raw" => OutputFormat::Raw,
                    "csv" => OutputFormat::Csv,
                    _ => panic!("不支持的输出格式"),
                };
                Ok(Box::new(FileSink::new(path.clone(), output_format).await?))
            }
            SinkConfig::Kafka {
                brokers: _, topic: _, batch_size: _
            } => {
                todo!("构建kafka sink")
            }
        }
    }

    pub async fn open_input(config: &InputConfig) -> Result<tokio::io::BufReader<tokio::fs::File>> {
        match config {
            InputConfig::File {path, .. } => {
                let file = tokio::fs::File::open(path)
                    .await
                    .with_context(|| format!("无法打开文件：{}", path.display()))?;
                Ok(tokio::io::BufReader::new(file))
            }
        }
    }


}







