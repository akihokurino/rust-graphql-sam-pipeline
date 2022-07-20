use thiserror::Error;

#[derive(Error, Debug, PartialOrd, PartialEq, Clone)]
pub enum AppError {
    #[error("不正なパラメーターです: {0}")]
    BadRequest(String),
    #[error("認証エラーです")]
    UnAuthenticate,
    #[error("禁止された行為です")]
    Forbidden,
    #[error("指定されたリソースが見つかりません")]
    NotFound,
    #[error("サーバーエラーです: {0}")]
    Internal(String),
}