mod debug;

use async_graphql::{MergedObject};
use crate::graphql::query::debug::DebugQuery;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    DebugQuery
);