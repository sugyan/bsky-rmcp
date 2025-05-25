use crate::{
    types::{
        CreatePostParams, DEFAULT_DEPTH, DEFAULT_LIMIT, DEFAULT_PARENT_HEIGHT, GetAuthorFeedParams,
        GetPostThreadParams, ListNotificationsParams, ReasonEnum, SearchPostsParams,
    },
    utils::{convert_datetime, get_post},
};
use bsky_sdk::{
    BskyAgent,
    api::{
        app::bsky,
        com::atproto,
        types::{LimitedU16, TryFromUnknown, Union, string::Datetime},
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
use std::collections::HashSet;

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
    #[tool(description = "Get the current user DID.")]
    async fn get_did(&self) -> Result<CallToolResult, Error> {
        Ok(if let Some(did) = self.agent.did().await {
            CallToolResult::success(vec![Content::text(did.as_ref())])
        } else {
            CallToolResult::error(vec![Content::text("failed to get did")])
        })
    }
    #[tool(description = "Get detailed profile view of an actor.")]
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
        Ok(CallToolResult::success(vec![Content::json(
            convert_datetime(profile).map_err(|e| {
                Error::internal_error(
                    "failed to convert datetime",
                    Some(Value::String(e.to_string())),
                )
            })?,
        )?]))
    }
    #[tool(
        description = "Get a view of an actor's 'author feed' (post and reposts by the author)."
    )]
    async fn get_author_feed(
        &self,
        #[tool(aggr)] params: GetAuthorFeedParams,
    ) -> Result<CallToolResult, Error> {
        let actor = params.actor.parse().map_err(|e: &str| {
            Error::internal_error("failed to parse actor", Some(Value::String(e.into())))
        })?;
        let filter = if params.with_replies.unwrap_or_default() {
            None
        } else {
            Some("posts_no_replies".into())
        };
        let limit = Some(
            params
                .limit
                .unwrap_or(DEFAULT_LIMIT)
                .try_into()
                .map_err(|e| {
                    Error::internal_error("failed to parse limit", Some(Value::String(e)))
                })?,
        );
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
                    filter,
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
            convert_datetime(output.data.feed).map_err(|e| {
                Error::internal_error(
                    "failed to convert datetime",
                    Some(Value::String(e.to_string())),
                )
            })?,
        )?]))
    }
    #[tool(description = "Get posts in a thread.")]
    async fn get_post_thread(
        &self,
        #[tool(aggr)] params: GetPostThreadParams,
    ) -> Result<CallToolResult, Error> {
        let depth = Some(
            params
                .depth
                .unwrap_or(DEFAULT_DEPTH)
                .try_into()
                .map_err(|e| {
                    Error::internal_error("failed to parse depth", Some(Value::String(e)))
                })?,
        );
        let parent_height = Some(
            params
                .parent_height
                .unwrap_or(DEFAULT_PARENT_HEIGHT)
                .try_into()
                .map_err(|e| {
                    Error::internal_error("failed to parse parent height", Some(Value::String(e)))
                })?,
        );
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
                    uri: params.uri,
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
        Ok(CallToolResult::success(vec![Content::json(
            convert_datetime(output.data).map_err(|e| {
                Error::internal_error(
                    "failed to convert datetime",
                    Some(Value::String(e.to_string())),
                )
            })?,
        )?]))
    }
    #[tool(description = "Find posts matching search criteria, returning views of those posts.")]
    async fn search_posts(
        &self,
        #[tool(aggr)] params: SearchPostsParams,
    ) -> Result<CallToolResult, Error> {
        let limit = Some(
            params
                .limit
                .unwrap_or(DEFAULT_LIMIT)
                .try_into()
                .map_err(|e| {
                    Error::internal_error("failed to parse limit", Some(Value::String(e)))
                })?,
        );
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .search_posts(
                bsky::feed::search_posts::ParametersData {
                    author: None,
                    cursor: None,
                    domain: None,
                    lang: None,
                    limit,
                    mentions: None,
                    q: params.q,
                    since: None,
                    sort: None,
                    tag: None,
                    until: None,
                    url: None,
                }
                .into(),
            )
            .await
            .map_err(|e| {
                Error::internal_error("failed to search posts", Some(Value::String(e.to_string())))
            })?;
        Ok(CallToolResult::success(vec![Content::json(
            convert_datetime(output.data.posts).map_err(|e| {
                Error::internal_error(
                    "failed to convert datetime",
                    Some(Value::String(e.to_string())),
                )
            })?,
        )?]))
    }
    #[tool(description = "Enumerate notifications for the requesting account.")]
    async fn list_notifications(
        &self,
        #[tool(aggr)] params: ListNotificationsParams,
    ) -> Result<CallToolResult, Error> {
        Ok(CallToolResult::success(vec![Content::json(
            convert_datetime(self._list_notifications(params).await?).map_err(|e| {
                Error::internal_error(
                    "failed to convert datetime",
                    Some(Value::String(e.to_string())),
                )
            })?,
        )?]))
    }
    #[tool(
        description = "Get the reply or mention notifications that have not been responded to by the user."
    )]
    async fn get_unreplied_mentions(
        &self,
        #[tool(param)]
        #[schemars(description = "Maximum number of notifications to retrieve.")]
        max_num: Option<u8>,
    ) -> Result<CallToolResult, Error> {
        // Get the recent notifications that are replies or mentions
        let notifications = self
            ._list_notifications(ListNotificationsParams {
                limit: max_num,
                reasons: vec![ReasonEnum::Mention, ReasonEnum::Reply],
            })
            .await?;
        // Get the post thread for each notification concurrently
        let mut handles = Vec::with_capacity(notifications.len());
        for notification in notifications.iter() {
            let agent = self.agent.clone();
            let uri = notification.uri.clone();
            handles.push(tokio::spawn(async move {
                agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_post_thread(
                        bsky::feed::get_post_thread::ParametersData {
                            depth: 1.try_into().ok(),
                            parent_height: Some(LimitedU16::MIN),
                            uri,
                        }
                        .into(),
                    )
                    .await
            }));
        }
        let did = self
            .agent
            .did()
            .await
            .ok_or(Error::internal_error("failed to get did", None))?;
        // Collect the uris of posts that have been replied from the current user
        let mut replied = HashSet::new();
        for handle in handles {
            // Wait for the task to finish and get the result
            let output = handle
                .await
                .map_err(|e| {
                    Error::internal_error(
                        "failed to await task",
                        Some(Value::String(e.to_string())),
                    )
                })?
                .map_err(|e| {
                    Error::internal_error(
                        "failed to get post thread",
                        Some(Value::String(e.to_string())),
                    )
                })?;
            // Check if the thread contains a reply from the user
            if let Union::Refs(
                bsky::feed::get_post_thread::OutputThreadRefs::AppBskyFeedDefsThreadViewPost(
                    thread_view_post,
                ),
            ) = &output.thread
            {
                if let Some(replies) = &thread_view_post.replies {
                    if replies.iter().any(|reply| {
                        if let Union::Refs(
                            bsky::feed::defs::ThreadViewPostRepliesItem::ThreadViewPost(view_post),
                        ) = reply
                        {
                            view_post.post.author.did == did
                        } else {
                            false
                        }
                    }) {
                        replied.insert(thread_view_post.post.uri.clone());
                    }
                }
            }
        }
        // Filter the notifications to only include those that have not been replied to
        Ok(CallToolResult::success(vec![Content::json(
            convert_datetime(
                notifications
                    .iter()
                    .filter(|notification| !replied.contains(&notification.uri))
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| {
                Error::internal_error(
                    "failed to convert datetime",
                    Some(Value::String(e.to_string())),
                )
            })?,
        )?]))
    }
    async fn _list_notifications(
        &self,
        params: ListNotificationsParams,
    ) -> Result<Vec<bsky::notification::list_notifications::Notification>, Error> {
        let limit = Some(
            params
                .limit
                .unwrap_or(DEFAULT_LIMIT)
                .try_into()
                .map_err(|e| {
                    Error::internal_error("failed to parse limit", Some(Value::String(e)))
                })?,
        );
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
                    reasons: Some(params.reasons.iter().map(|r| r.to_string()).collect()),
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
        Ok(output.data.notifications)
    }
    #[tool(
        description = "Create a regular or reply post. Use `text` for content. Set `reply` to a post URI if replying."
    )]
    async fn create_post(
        &self,
        #[tool(aggr)] params: CreatePostParams,
    ) -> Result<CallToolResult, Error> {
        let rt = RichText::new_with_detect_facets(params.text)
            .await
            .map_err(|e| {
                Error::internal_error(
                    "failed to create rich text",
                    Some(Value::String(e.to_string())),
                )
            })?;
        let reply = if let Some(reply) = &params.reply {
            let output = get_post(&self.agent, reply).await.map_err(|e| {
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
        } else {
            None
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
            prompts: vec![Prompt::new(
                "get_self_feed",
                Some("Get the self feed of the current user"),
                None,
            )],
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
