use anyhow::Result;
use crate::client::{WhatpulseClient, UserResponse};

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    let user = client.get_resource::<UserResponse>("user").await?;
    println!("User: {} (ID: {})", 
        user.account_name.as_deref().unwrap_or("unknown"), 
        user.id.as_deref().unwrap_or("unknown")
    );
    if let Some(keys) = &user.keys {
        println!("Keys: {}", keys);
    }
    if let Some(clicks) = &user.clicks {
        println!("Clicks: {}", clicks);
    }
    Ok(())
}
