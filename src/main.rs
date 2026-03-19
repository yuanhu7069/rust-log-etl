//! 主函数入口
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use std::path::PathBuf;
use clap::Parser;
use rust_log_etl::builder::PipelineBuilder;
use rust_log_etl::cli::{Args, RunMode, RuntimeConfig};
use rust_log_etl::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match &args.mode {
        RunMode::Config { config } => {
            run_with_config(config).await?;
        }
        RunMode::Run { input, output, parser, format, workers: _ } => {
            // 快速模式：默认死信队列为输出路径 + .dlq
            let dlq_path = Some(output.with_extension("dlq"));
            let runtime = RuntimeConfig::from_args(
                input.clone(),
                output.clone(),
                parser.clone(),
                format.clone(),
                4, // 默认 workers
            );
            run_with_runtime(&runtime, dlq_path).await?;
        }
    }

    Ok(())
}

async fn run_with_config(config_path: &PathBuf) -> anyhow::Result<()> {
    let config = Config::from_file(config_path)?;
    println!("配置加载成功");

    // 构建 pipeline 和死信队列
    let (pipeline, dlq) = PipelineBuilder::build(&config).await?;
    println!("Pipeline 构建完成");

    // 启动死信队列任务
    let dlq_handle = if let Some(dlq) = dlq {
        let handle = tokio::spawn(async move {
            dlq.run().await
        });
        Some(handle)
    } else {
        println!("未配置死信队列");
        None
    };

    // 打开输入
    let reader = PipelineBuilder::open_input(&config.input).await?;
    println!("开始处理...");

    // 运行
    let stats =  {
        let mut pipeline = pipeline;
        pipeline.run(reader).await?
    };

    println!("\n处理统计:");
    println!("  总行数:     {}", stats.total_lilnes);
    println!("  解析成功:   {}", stats.parsed_ok);
    println!("  解析失败:   {}", stats.parsed_failed);
    println!("  被过滤:     {}", stats.filtered);
    println!("  写入成功:   {}", stats.written);

    // 等待死信队列完成
    if let Some(handle) = dlq_handle {
        let dlq_count = handle.await??;
        println!("  死信队列:   {}", dlq_count);
    }

    Ok(())
}

async fn run_with_runtime(
    runtime: &RuntimeConfig,
    dlq_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!("快速模式: {} -> {}", runtime.input_path.display(), runtime.output_path.display());

    // 构建 pipeline 和死信队列
    let (pipeline, dlq) = PipelineBuilder::build_from_runtime(runtime, dlq_path).await?;

    // 启动死信队列
    let dlq_handle = if let Some(dlq) = dlq {
        let handle = tokio::spawn(async move {
            dlq.run().await
        });
        Some(handle)
    } else {
        None
    };

    // 打开输入
    let file = tokio::fs::File::open(&runtime.input_path)
        .await
        .map_err(|e| anyhow::anyhow!("无法打开输入文件: {}", e))?;
    let reader = tokio::io::BufReader::new(file);

    // 运行
    let stats = pipeline.run(reader).await?;

    println!("\n处理统计:");
    println!("  总行数:     {}", stats.total_lilnes);
    println!("  解析成功:   {}", stats.parsed_ok);
    println!("  解析失败:   {}", stats.parsed_failed);
    println!("  被过滤:     {}", stats.filtered);
    println!("  写入成功:   {}", stats.written);

    if let Some(handle) = dlq_handle {
        match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
            Ok(Ok(Ok(count))) => println!("  死信队列:   {}", count),
            Ok(Ok(Err(e))) => eprintln!("  死信队列错误: {}", e),
            Ok(Err(e)) => eprintln!("  死信队列任务 panic: {}", e),
            Err(_) => eprintln!("  死信队列超时"),
        }
    }


    Ok(())
}