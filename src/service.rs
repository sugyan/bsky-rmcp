use crate::{
    types::{CreatePostParams, GetAuthorFeedParams, GetPostThreadParams, ListNotificationsParams},
    utils::get_post,
};
use bsky_sdk::{
    BskyAgent,
    api::{
        app::bsky,
        com::atproto,
        types::{LimitedNonZeroU8, LimitedU16, TryFromUnknown, string::Datetime},
    },
    rich_text::RichText,
};
use rmcp::{
    Error, RoleServer, ServerHandler,
    model::{
        CallToolResult, Content, GetPromptRequestParam, GetPromptResult, ListPromptsResult,
        PaginatedRequestParam, Prompt, PromptMessage, PromptMessageRole, ServerCapabilities,
        ServerInfo,
    },
    schemars,
    serde_json::Value,
    service::RequestContext,
    tool,
};

#[derive(Clone)]
pub struct BskyService {
    agent: BskyAgent,
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
    #[tool(description = "Get posts in a thread")]
    async fn get_post_thread(
        &self,
        #[tool(aggr)] GetPostThreadParams {
            uri,
            depth,
            parent_height,
        }: GetPostThreadParams,
    ) -> Result<CallToolResult, Error> {
        let depth = Some(LimitedU16::<1000u16>::try_from(depth).map_err(|e| {
            Error::internal_error("failed to parse depth", Some(Value::String(e.to_string())))
        })?);
        let parent_height = Some(LimitedU16::<1000u16>::try_from(parent_height).map_err(|e| {
            Error::internal_error(
                "failed to parse parent height",
                Some(Value::String(e.to_string())),
            )
        })?);
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(
                bsky::feed::get_post_thread::ParametersData {
                    depth,
                    parent_height,
                    uri,
                }
                .into(),
            )
            .await
            .map_err(|e| {
                Error::internal_error(
                    "failed to get post thread",
                    Some(Value::String(e.to_string())),
                )
            })?;
        Ok(CallToolResult::success(vec![Content::json(output.data)?]))
    }
    #[tool(description = "Enumerate notifications for the requesting account")]
    async fn list_notifications(
        &self,
        #[tool(aggr)] ListNotificationsParams { limit, reasons }: ListNotificationsParams,
    ) -> Result<CallToolResult, Error> {
        let limit = Some(LimitedNonZeroU8::<100u8>::try_from(limit).map_err(|e| {
            Error::internal_error("failed to parse limit", Some(Value::String(e.to_string())))
        })?);
        let output = self
            .agent
            .api
            .app
            .bsky
            .notification
            .list_notifications(
                bsky::notification::list_notifications::ParametersData {
                    cursor: None,
                    limit,
                    priority: None,
                    reasons: Some(reasons.iter().map(|r| r.to_string()).collect()),
                    seen_at: None,
                }
                .into(),
            )
            .await
            .map_err(|e| {
                Error::internal_error(
                    "failed to list notifications",
                    Some(Value::String(e.to_string())),
                )
            })?;
        Ok(CallToolResult::success(vec![Content::json(
            output.data.notifications,
        )?]))
    }
    #[tool(description = "Post a new message")]
    async fn create_post(
        &self,
        #[tool(aggr)] CreatePostParams { text, reply }: CreatePostParams,
    ) -> Result<CallToolResult, Error> {
        let rt = RichText::new_with_detect_facets(text).await.map_err(|e| {
            Error::internal_error(
                "failed to create rich text",
                Some(Value::String(e.to_string())),
            )
        })?;
        let reply = if reply.is_empty() {
            None
        } else {
            let output = get_post(&self.agent, &reply).await.map_err(|e| {
                Error::internal_error("failed to get post", Some(Value::String(e.to_string())))
            })?;
            let strong_ref =
                atproto::repo::strong_ref::Main::from(atproto::repo::strong_ref::MainData {
                    cid: output
                        .data
                        .cid
                        .ok_or(Error::internal_error("failed to get cid", None))?,
                    uri: output.data.uri,
                });
            let record =
                bsky::feed::post::Record::try_from_unknown(output.data.value).map_err(|e| {
                    Error::internal_error(
                        "failed to convert record",
                        Some(Value::String(e.to_string())),
                    )
                })?;
            let root = if let Some(reply) = &record.reply {
                reply.root.clone()
            } else {
                strong_ref.clone()
            };
            Some(
                bsky::feed::post::ReplyRefData {
                    parent: strong_ref,
                    root,
                }
                .into(),
            )
        };
        let post = self
            .agent
            .create_record(bsky::feed::post::RecordData {
                created_at: Datetime::now(),
                embed: None,
                entities: None,
                facets: rt.facets,
                labels: None,
                langs: None,
                reply,
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
    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, Error> {
        match request.name.as_ref() {
            "get_self_feed" => Ok(GetPromptResult {
                description: None,
                messages: vec![
                    PromptMessage::new_text(
                        PromptMessageRole::User,
                        "I want to view my own recent posts.",
                    ),
                    PromptMessage::new_text(
                        PromptMessageRole::Assistant,
                        "First, call the `get_did` tool to retrieve the current user's DID. Then, call the `get_author_feed` tool using that DID as the `actor` parameter. If the user specifies a number of posts to retrieve (e.g., 'last 3 posts'), include that as the `limit` parameter. Otherwise, omit `limit`.",
                    ),
                ],
            }),
            "get_unreplied_replies" => Ok(GetPromptResult {
                description: None,
                messages: vec![
                    PromptMessage::new_text(
                        PromptMessageRole::User,
                        r#"Please show me replies that I haven't responded to yet."#,
                    ),
                    PromptMessage::new_text(
                        PromptMessageRole::Assistant,
                        r#"To find replies the user hasn't responded to:

1. Call `get_did` to retrieve the current user's DID.
2. Call `list_notifications` with the `reason` parameter set to `["reply"]`. If a `limit` is provided, include it as a parameter. Otherwise, omit it.
3. For each returned reply notification, call `get_post_thread` with `depth: 1` and `parent_height: 0`.
4. In each thread, examine the `replies` array. If none of the replies are authored by the user's DID, consider it as not responded.
5. Return the reply notifications where the user has not responded."#,
                    ),
                ],
            }),
            _ => Err(Error::invalid_params("prompt not found", None)),
        }
    }
    async fn list_prompts(
        &self,
        _: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, Error> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![
                Prompt::new(
                    "get_self_feed",
                    Some("Get the self feed of the current user"),
                    None,
                ),
                Prompt::new(
                    "get_unreplied_replies",
                    Some("Retrieve recent replies that the user has not yet responded to"),
                    None,
                ),
            ],
        })
    }
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("bsky service".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            ..Default::default()
        }
    }
}
