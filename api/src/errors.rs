use async_graphql::{ErrorExtensions, FieldError, Value};
use strum_macros::Display as StrumDisplay;
use convert_case::{Case, Casing};
use app::errors::AppError;

#[derive(StrumDisplay, Debug)]
pub enum FieldErrorCode {
    BadRequest,
    UnAuthenticate,
    NotFound,
    Forbidden,
    Internal,
}

impl Into<Value> for FieldErrorCode {
    fn into(self) -> Value {
        Value::String(self.to_string().to_case(Case::UpperSnake))
    }
}

pub struct FieldErrorWithCode {
    err: AppError,
    code: FieldErrorCode,
}

impl From<AppError> for FieldErrorWithCode {
    fn from(err: AppError) -> Self {
        FieldErrorWithCode {
            err: err.clone(),
            code: match err {
                AppError::BadRequest(_) => FieldErrorCode::BadRequest,
                AppError::UnAuthenticate => FieldErrorCode::UnAuthenticate,
                AppError::Forbidden => FieldErrorCode::Forbidden,
                AppError::NotFound => FieldErrorCode::NotFound,
                AppError::Internal(_) => FieldErrorCode::Internal,
            },
        }
    }
}

impl From<FieldErrorWithCode> for FieldError {
    fn from(v: FieldErrorWithCode) -> Self {
        let code = v.code.to_string().to_case(Case::UpperSnake);
        let err = FieldError::new(format!("{}", v.err));
        err.extend_with(|_, e| e.set("code", code.clone()))
    }
}