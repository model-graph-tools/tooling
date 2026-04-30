use anyhow::Result;
use wildfly_meta::update_all;

pub async fn update() -> Result<()> {
    let result = tokio::task::spawn_blocking(update_all).await??;
    println!("{}", result.summary());
    Ok(())
}
