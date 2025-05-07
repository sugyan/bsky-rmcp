use anyhow::{Context, Result};
use bsky_sdk::BskyAgent;
use rmcp::ServiceExt;
use std::{env, io};
use tokio::io::{stdin, stdout};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use bsky_rmcp::BskyService;

#[tokio::main]
async fn main() -> Result<()> {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .with_ansi(false)
        .init();

    let agent = BskyAgent::builder().build().await?;
    let identifier = env::var("BLUESKY_IDENTIFIER")
        .context("failed to get environment variable BLUESKY_IDENTIFIER")?;
    let password = env::var("BLUESKY_APP_PASSWORD")
        .context("failed to get environment variable BLUESKY_APP_PASSWORD")?;
    let session = agent.login(identifier, password).await?;
    tracing::info!(
        "logged in as {} ({})",
        session.handle.as_str(),
        session.did.as_str()
    );

    let transport = (stdin(), stdout());
    let service = BskyService::new(agent)
        .serve(transport)
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;
    service.waiting().await?;
    Ok(())
}
