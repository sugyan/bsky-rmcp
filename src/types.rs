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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNotificationsParams {
    #[schemars(
        description = "Limit for the number of posts to fetch",
        default = "default_limit"
    )]
    pub limit: u8,
}

fn default_limit() -> u8 {
    10
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePostParams {
    #[schemars(description = "Text content of the post")]
    pub text: String,
    #[schemars(description = "Optional uri target for reply", default = "String::new")]
    pub reply: String,
}
