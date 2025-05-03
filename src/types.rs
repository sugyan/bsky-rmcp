use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use std::fmt;

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
pub struct GetPostThreadParams {
    #[schemars(description = "Reference (AT-URI) to post record.")]
    pub uri: String,
    #[schemars(
        description = "How many levels of reply depth should be included in response",
        default = "default_depth"
    )]
    pub depth: u16,
    #[schemars(
        description = "How many levels of parent (and grandparent, etc) post to include",
        default = "default_parent_height"
    )]
    pub parent_height: u16,
}

fn default_depth() -> u8 {
    1
}

fn default_parent_height() -> u8 {
    10
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReasonEnum {
    Like,
    Repost,
    Follow,
    Mention,
    Reply,
    Quote,
    StarterpackJoined,
    Verified,
    Unverified,
}

impl fmt::Display for ReasonEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            ReasonEnum::Like => "like",
            ReasonEnum::Repost => "repost",
            ReasonEnum::Follow => "follow",
            ReasonEnum::Mention => "mention",
            ReasonEnum::Reply => "reply",
            ReasonEnum::Quote => "quote",
            ReasonEnum::StarterpackJoined => "starterpack-joined",
            ReasonEnum::Verified => "verified",
            ReasonEnum::Unverified => "unverified",
        };
        write!(f, "{reason}")
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNotificationsParams {
    #[schemars(
        description = "Limit for the number of posts to fetch",
        default = "default_limit"
    )]
    pub limit: u8,
    #[schemars(description = "Notification reasons to include in response")]
    pub reasons: Vec<ReasonEnum>,
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
