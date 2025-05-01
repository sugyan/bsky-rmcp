use anyhow::{Context, Result};
use bsky_sdk::BskyAgent;
use rmcp::ServiceExt;
use std::env;
use tokio::io::{stdin, stdout};

use bsky_rmcp::BskyService;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = BskyAgent::builder().build().await?;
    let identifier = env::var("BLUESKY_IDENTIFIER")
        .context("failed to get environment variable BLUESKY_IDENTIFIER")?;
    let password = env::var("BLUESKY_APP_PASSWORD")
        .context("failed to get environment variable BLUESKY_APP_PASSWORD")?;
    let session = agent.login(identifier, password).await?;
    eprintln!(
        "logged in as {} ({})",
        session.handle.as_str(),
        session.did.as_str()
    );

    let transport = (stdin(), stdout());
    BskyService::new(agent)
        .serve(transport)
        .await?
        .waiting()
        .await?;
    Ok(())
}
