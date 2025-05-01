use atrium_api::types::string::Datetime;
use bsky_sdk::{BskyAgent, rich_text::RichText};
use rmcp::{
    Error, ServerHandler,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars,
    serde_json::Value,
    tool,
};

#[derive(Clone)]
pub struct BskyService {
    agent: BskyAgent,
}

#[tool(tool_box)]
impl BskyService {
    pub fn new(agent: BskyAgent) -> Self {
        BskyService { agent }
    }
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
        let profile = self
            .agent
            .api
            .app
            .bsky
            .actor
            .get_profile(
                atrium_api::app::bsky::actor::get_profile::ParametersData {
                    actor: actor.parse().map_err(|e: &str| {
                        Error::internal_error(
                            "failed to parse actor",
                            Some(Value::String(e.into())),
                        )
                    })?,
                }
                .into(),
            )
            .await
            .map_err(|e| {
                Error::internal_error("failed to get profile", Some(Value::String(e.to_string())))
            })?;
        Ok(CallToolResult::success(vec![Content::json(profile)?]))
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
            .create_record(atrium_api::app::bsky::feed::post::RecordData {
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
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
