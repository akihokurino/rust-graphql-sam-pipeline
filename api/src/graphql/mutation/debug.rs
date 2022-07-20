use async_graphql::Context;
use async_graphql::{Result, Object, InputObject, SimpleObject};
use app::errors::AppError;
use crate::errors::{FieldErrorWithCode};

#[derive(Default)]
pub struct DebugMutation;

#[Object]
impl DebugMutation {
    async fn debug(&self, _ctx: &Context<'_>, input: DebugInput) -> Result<DebugPayload> {
        if input.token.is_empty() {
            return Err(FieldErrorWithCode::from(AppError::BadRequest("token empty".to_string())).into())
        }

        Ok(DebugPayload {
            is_ok: true
        })
    }
}

#[derive(InputObject)]
struct DebugInput {
    pub token: String,
}
#[derive(SimpleObject)]
struct DebugPayload {
    pub is_ok: bool,
}