use anyhow::{Context, Result};
use crate::client::{WhatpulseClient, UserResponse};
use uom::si::energy::{joule, calorie, kilocalorie};
use uom::si::f64::Energy;
use uom::si::force::newton;
use uom::si::length::meter;

// Constants based on Cherry MX Red switches (common mechanical switch)
const FORCE_NEWTONS: f64 = 0.45; // 45g actuation force ≈ 0.45N
const DISTANCE_METERS: f64 = 0.004; // 4.0mm travel distance

// Conversion constants
const CALORIES_PER_M_AND_M: f64 = 10.0; // ~10 kcal per M&M (standard size)
const CALORIES_PER_MINUTE_RUNNING: f64 = 10.0; // ~10 kcal/min for average runner

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    println!("Fetching latest pulse data...");
    
    // Fetch user stats to get total keys
    let user = client.get_resource::<UserResponse>("user")
        .await
        .context("Failed to fetch user data")?;

    let keys_str = user.keys.as_deref().unwrap_or("0");
    // Remove commas if present (API might return "15,234")
    let keys_clean = keys_str.replace(',', "");
    let keys: f64 = keys_clean.parse().context("Failed to parse keys count")?;

    // Calculate Work: W = F * d * keys
    // We calculate work for ONE keystroke first
    let force = uom::si::f64::Force::new::<newton>(FORCE_NEWTONS);
    let distance = uom::si::f64::Length::new::<meter>(DISTANCE_METERS);
    let work_per_keystroke: Energy = force * distance;
    
    // Total work
    let total_work = work_per_keystroke * keys;
    
    // Convert to calories (small calories)
    let total_calories = total_work.get::<calorie>();
    // Convert to kilocalories (food calories)
    let total_kcal = total_work.get::<kilocalorie>();

    // Comparisons
    let m_and_ms = total_kcal / CALORIES_PER_M_AND_M;
    let running_minutes = total_kcal / CALORIES_PER_MINUTE_RUNNING;
    let running_seconds = running_minutes * 60.0;

    // Formatting output
    println!("\nEnergy Expenditure Report:");
    println!("──────────────────────────");
    println!("Total Keystrokes: {}", keys_str); // Use original string with commas if available
    println!("Work Performed:   {:.2} J", total_work.get::<joule>());
    println!("Calories Burned:  {:.2} cal", total_calories);
    println!("                  {:.4} kcal", total_kcal);
    println!("──────────────────────────");
    println!("Fun Comparisons:");
    println!("• Equivalent to {:.4} M&Ms", m_and_ms);
    
    if running_minutes >= 1.0 {
        println!("• Like running for {:.1} minutes", running_minutes);
    } else {
        println!("• Like running for {:.0} seconds", running_seconds);
    }

    Ok(())
}
