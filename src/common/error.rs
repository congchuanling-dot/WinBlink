use thiserror::Error;

#[derive(Error, Debug)]
pub enum WinBlinkError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("Everything 错误: {0}")]
    Everything(String),

    #[error("快捷方式解析错误: {0}")]
    Shortcut(String),

    #[error("{0}")]
    General(String),

    #[error("未知错误")]
    Unknown,
}
