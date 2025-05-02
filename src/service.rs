use crate::types::GetAuthorFeedParams;
use bsky_sdk::{
    BskyAgent,
    api::{
        app::bsky,
        types::{LimitedNonZeroU8, string::Datetime},
    },
    rich_text::RichText,
};
use rmcp::{
    Error, ServerHandler,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars,
    serde_json::Value,
    tool,
};

#[derive(Clone)]
pub struct BskyService {
    pub(crate) agent: BskyAgent,
}

impl BskyService {
    pub fn new(agent: BskyAgent) -> Self {
        BskyService { agent }
    }
}

#[tool(tool_box)]
impl BskyService {
    #[tool(description = "Get the current user DID")]
    async fn get_did(&self) -> Result<CallToolResult, Error> {
        Ok(if let Some(did) = self.agent.did().await {
            CallToolResult::success(vec![Content::text(did.as_ref())])
        } else {
            CallToolResult::error(vec![Content::text("failed to get did")])
        })
    }
    #[tool(description = "Get detailed profile view of an actor")]
    async fn get_profile(
        &self,
        #[tool(param)]
        #[schemars(description = "Handle or DID of account to fetch profile of")]
        actor: String,
    ) -> Result<CallToolResult, Error> {
        let actor = actor.parse().map_err(|e: &str| {
            Error::internal_error("failed to parse actor", Some(Value::String(e.into())))
        })?;
        let profile = self
            .agent
            .api
            .app
            .bsky
            .actor
            .get_profile(bsky::actor::get_profile::ParametersData { actor }.into())
            .await
            .map_err(|e| {
                Error::internal_error("failed to get profile", Some(Value::String(e.to_string())))
            })?;
        Ok(CallToolResult::success(vec![Content::json(profile)?]))
    }
    #[tool(description = "Get a view of an actor's 'author feed' (post and reposts by the author)")]
    async fn get_author_feed(
        &self,
        #[tool(aggr)] GetAuthorFeedParams { actor, limit }: GetAuthorFeedParams,
    ) -> Result<CallToolResult, Error> {
        let actor = actor.parse().map_err(|e: &str| {
            Error::internal_error("failed to parse actor", Some(Value::String(e.into())))
        })?;
        let limit = Some(LimitedNonZeroU8::<100u8>::try_from(limit).map_err(|e| {
            Error::internal_error("failed to parse limit", Some(Value::String(e.to_string())))
        })?);
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(
                bsky::feed::get_author_feed::ParametersData {
                    actor,
                    cursor: None,
                    filter: None,
                    include_pins: None,
                    limit,
                }
                .into(),
            )
            .await
            .map_err(|e| {
                Error::internal_error(
                    "failed to get author feed",
                    Some(Value::String(e.to_string())),
                )
            })?;
        Ok(CallToolResult::success(vec![Content::json(
            output.data.feed,
        )?]))
    }
    #[tool(description = "Post a new message")]
    async fn create_post(
        &self,
        #[tool(param)]
        #[schemars(description = "Text content of the post")]
        text: String,
    ) -> Result<CallToolResult, Error> {
        let rt = RichText::new_with_detect_facets(text).await.map_err(|e| {
            Error::internal_error(
                "failed to create rich text",
                Some(Value::String(e.to_string())),
            )
        })?;
        let post = self
            .agent
            .create_record(bsky::feed::post::RecordData {
                created_at: Datetime::now(),
                embed: None,
                entities: None,
                facets: rt.facets,
                labels: None,
                langs: None,
                reply: None,
                tags: None,
                text: rt.text,
            })
            .await
            .map_err(|e| {
                Error::internal_error(
                    "failed to create record",
                    Some(Value::String(e.to_string())),
                )
            })?;
        Ok(CallToolResult::success(vec![Content::json(post)?]))
    }
}

#[tool(tool_box)]
impl ServerHandler for BskyService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("bsky service".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
