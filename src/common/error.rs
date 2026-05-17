use thiserror::Error;

#[derive(Error, Debug)]
pub enum WinBlinkError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    General(String),

    #[error("未知错误")]
    Unknown,
}
