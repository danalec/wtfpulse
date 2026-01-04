use anyhow::Result;
use crate::client::WhatpulseClient;

pub async fn execute(client: &WhatpulseClient, path: String) -> Result<()> {
    let text = client.get_text(&path).await?;
    println!("{}", text);
    Ok(())
}
