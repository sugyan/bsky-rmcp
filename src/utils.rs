use anyhow::{Result, anyhow};
use bsky_sdk::{BskyAgent, api::com::atproto};

pub async fn get_post(
    agent: &BskyAgent,
    at_uri: &str,
) -> Result<atproto::repo::get_record::Output> {
    let parts = at_uri
        .strip_prefix("at://")
        .ok_or(anyhow!("invalid AT URI"))?
        .splitn(3, '/')
        .collect::<Vec<_>>();
    let repo = parts[0].parse().map_err(|e| anyhow!("invalid repo: {e}"))?;
    let collection = parts[1]
        .parse()
        .map_err(|e| anyhow!("invalid collection: {e}"))?;
    let rkey = parts[2]
        .parse()
        .map_err(|e| anyhow!("invalid record key: {e}"))?;
    Ok(agent
        .api
        .com
        .atproto
        .repo
        .get_record(
            atproto::repo::get_record::ParametersData {
                cid: None,
                collection,
                repo,
                rkey,
            }
            .into(),
        )
        .await?)
}
