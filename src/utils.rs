use anyhow::anyhow;
use bsky_sdk::{
    BskyAgent,
    api::{com::atproto, types::string::Datetime},
};
use chrono::Local;
use rmcp::serde_json::{self, Map, Value};
use serde::Serialize;

pub async fn get_post(
    agent: &BskyAgent,
    at_uri: &str,
) -> anyhow::Result<atproto::repo::get_record::Output> {
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

pub fn convert_datetime<S>(data: S) -> Result<Value, serde_json::Error>
where
    S: Serialize,
{
    let value = serde_json::to_value(data)?;
    fn recursive(value: Value) -> Result<Value, serde_json::Error> {
        Ok(match value {
            Value::Object(map) => {
                let mut new_map = Map::new();
                for (key, value) in map {
                    if key.ends_with("At") {
                        if let Value::String(datetime_str) = &value {
                            if let Ok(datetime) = datetime_str.parse::<Datetime>() {
                                let local_dt = datetime.as_ref().with_timezone(&Local);
                                let converted =
                                    Datetime::new(local_dt.with_timezone(local_dt.offset()));
                                new_map.insert(key, serde_json::to_value(converted)?);
                                continue;
                            }
                        }
                    }
                    new_map.insert(key, recursive(value)?);
                }
                Value::Object(new_map)
            }
            Value::Array(array) => Value::Array(
                array
                    .into_iter()
                    .map(recursive)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            _ => value,
        })
    }
    recursive(value)
}
