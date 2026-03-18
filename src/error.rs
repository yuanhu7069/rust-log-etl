use thiserror::Error;

#[derive(Error, Debug)]
pub enum EtlError {
    #[error("解析失败：{line}")]
    ParseError{
        line: String,
        source: anyhow::Error
    },
    #[error("配置无效：{0}")]
    ConfigError(String),

    #[error("IO错误")]
    Io(#[from] std::io::Error),
}