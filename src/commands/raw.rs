use crate::client::WhatpulseClient;
use anyhow::Result;
use serde_json::Value;

pub async fn execute(client: &WhatpulseClient, path: String) -> Result<()> {
    let json = client.get_json::<Value>(&path).await?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
