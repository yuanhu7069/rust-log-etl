//! 错误处理模块
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0


use thiserror::Error;
use crate::core::ParsedRecord;

#[derive(Error, Debug)]
pub enum EtlError {
    #[error("配置错误： {0}")]
    Config(String),
    
    #[error("IO错误： {0}")]
    Io(#[from]std::io::Error),
    
    #[error("解析错误： line={line}， 原因： {reason}")]
    ParseError { line: String, reason: String},
    
    #[error("转换错误： {0}")]
    Transform(String),
    
    #[error("Sink错误： {0}")]
    Sink(String),
    
    #[error("未知错误： {0}")]
    Unknown(String)
}

pub enum LineResult {
    // 成功（继续执行）
    Successs(ParsedRecord),
    
    // 被过滤掉（正常流程，不计入错误）
    Filtered,
    
    // 失败
    Failed{line: String, error: EtlError},
}