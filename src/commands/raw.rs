use crate::client::WhatpulseClient;
use anyhow::Result;

pub async fn execute(client: &WhatpulseClient, path: String) -> Result<()> {
    let text = client.get_text(&path).await?;
    println!("{}", text);
    Ok(())
}
