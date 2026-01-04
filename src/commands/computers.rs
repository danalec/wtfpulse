use anyhow::Result;
use crate::client::{WhatpulseClient, UserResponse};

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    // Computer stats are nested inside the User response
    let user = client.get_resource::<UserResponse>("user").await?;
    if let Some(computers) = user.computers {
        println!("Found {} computers:", computers.len());
        for (_, comp) in computers {
            println!("{} ({}): {} keys, {} clicks", 
                comp.name.as_deref().unwrap_or("unknown"),
                comp.id.as_deref().unwrap_or("unknown"),
                comp.keys.as_deref().unwrap_or("0"),
                comp.clicks.as_deref().unwrap_or("0")
            );
        }
    } else {
        println!("No computers found in user profile.");
    }
    Ok(())
}
