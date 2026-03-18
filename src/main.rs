//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use std::path::PathBuf;



use clap::Parser as ClapParser;
use rust_log_etl::{Config};
use rust_log_etl::builder::PipelineBuilder;

#[derive(Debug, ClapParser)]
#[command(name = "rustlogetl")]
#[command(about = "高性能日志ETL工具")]
struct Args {
    #[arg[short, long]]
   config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let args = Args::parse();

    let config = Config::from_file(&args.config)?;
    println!("配置加载成功：{:?}", config.global);

    let pipeline = PipelineBuilder::build(&config).await?;
    println!("pipeline 构建完成");

    let reader = PipelineBuilder::open_input(&config.input).await?;
    println!("开始处理...");

    let count = pipeline.run(reader).await?;
    println!("处理完成，共处理{}行数据", count);

    Ok(())
}
