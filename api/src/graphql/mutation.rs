mod debug;

use async_graphql::{MergedObject};
use crate::graphql::mutation::debug::DebugMutation;

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    DebugMutation
);