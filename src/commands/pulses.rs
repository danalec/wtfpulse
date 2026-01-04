use anyhow::Result;
use crate::client::{WhatpulseClient, PulseResponse};
use std::collections::HashMap;

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    let pulses_map = client.get_resource::<HashMap<String, PulseResponse>>("pulses").await?;
    println!("Found {} pulses:", pulses_map.len());
    
    // Convert to vector and sort by key (Pulse ID) descending to show newest first
    let mut pulses: Vec<_> = pulses_map.into_iter().collect();
    // Pulse IDs are strings like "Pulse-123", so string sort works reasonably well for ordering
    pulses.sort_by(|a, b| b.0.cmp(&a.0));

    for (id, pulse) in pulses.iter().take(5) {
        println!("{}: {} keys on {}", 
            id, 
            pulse.keys.as_deref().unwrap_or("0"),
            pulse.date.as_deref().unwrap_or("unknown date")
        );
    }
    Ok(())
}
