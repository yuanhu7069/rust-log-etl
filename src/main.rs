//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0




use clap::Parser as ClapParser;
use rust_log_etl::{Config};
use rust_log_etl::builder::PipelineBuilder;
use rust_log_etl::cli::{Args, RunMode, RuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let args = Args::parse();

    let runtime_config = match &args.mode {
        RunMode::Config {config} => {
            println!("使用配置文件：{}", config.display());
            RuntimeConfig::from_config_file(config)?
        }

        RunMode::Run {input, output, parser, format, workers} => {
            println!("快速模式: {} -> {}", input.display(), output.display());
            RuntimeConfig::from_args(
                input.clone(),
                output.clone(),
                parser.clone(),
                format.clone(),
                *workers,
            )
        }
    };

    println!("运行配置: {:?}", runtime_config);

    // 根据模式构建 Pipeline
    let pipeline = match &args.mode {
        RunMode::Config { config } => {
            // 配置文件模式：完整功能
            let config = Config::from_file(config)?;
            PipelineBuilder::build(&config).await?
        }
        RunMode::Run { .. } => {
            // 快速模式：简化功能，无 transformers
            PipelineBuilder::build_from_runtime(&runtime_config).await?
        }
    };

    // 打开输入源
    let reader = match &args.mode {
        RunMode::Config { config } => {
            let config = Config::from_file(config)?;
            PipelineBuilder::open_input(&config.input).await?
        }
        RunMode::Run { input, .. } => {
            let file = tokio::fs::File::open(input)
                .await
                .map_err(|e| anyhow::anyhow!("无法打开输入文件: {}", e))?;
            tokio::io::BufReader::new(file)
        }
    };

    println!("开始处理...");
    let count = pipeline.run(reader).await?;
    println!("处理完成: {} 条记录", count);

    Ok(())
}
