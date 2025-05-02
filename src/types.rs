use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAuthorFeedParams {
    #[schemars(description = "Handle or DID of account to fetch author feed of")]
    pub actor: String,
    #[schemars(
        description = "Limit for the number of posts to fetch",
        default = "default_limit"
    )]
    pub limit: u8,
}

fn default_limit() -> u8 {
    10
}
