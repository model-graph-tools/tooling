use anyhow::Result;
use wildfly_meta::update_all;

use crate::json::UpdateResult;

pub async fn update(json: bool) -> Result<()> {
    let result = tokio::task::spawn_blocking(update_all).await??;
    if json {
        let json_result = UpdateResult::from(&result);
        println!("{}", serde_json::to_string(&json_result)?);
    } else {
        println!("{}", result.summary());
    }
    Ok(())
}
