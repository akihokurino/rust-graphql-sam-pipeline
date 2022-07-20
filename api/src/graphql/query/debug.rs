use async_graphql::Context;
use async_graphql::{Result, Object};

#[derive(Default)]
pub struct DebugQuery;

#[Object]
impl DebugQuery {
    async fn debug(&self, _ctx: &Context<'_>) -> Result<bool> {
        Ok(true)
    }
}