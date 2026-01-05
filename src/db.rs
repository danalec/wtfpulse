use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::PathBuf;
#[derive(Debug, Default, Clone)]
pub struct MouseStats {
    pub clicks: u64,
    pub scrolls: u64,
    pub distance_meters: f64,
    pub clicks_by_button: HashMap<i64, u64>,
}

pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn new() -> Result<Self> {
        let path = Self::find_db_path()?;
        Ok(Self { path })
    }

    fn find_db_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").context("LOCALAPPDATA not set")?;
            let path = PathBuf::from(local_app_data)
                .join("WhatPulse")
                .join("whatpulse.db");
            if path.exists() {
                return Ok(path);
            }
        }

        // Fallback or other OS
        Err(anyhow::anyhow!("Could not find whatpulse.db"))
    }

    pub fn get_connection(&self) -> Result<Connection> {
        Connection::open_with_flags(&self.path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .context("Failed to open database")
    }

    pub fn get_mouse_stats(&self, period: &str) -> Result<MouseStats> {
        let conn = self.get_connection()?;
        let where_clause = self.get_where_clause(period);

        // Total Clicks
        let sql_clicks = format!("SELECT SUM(count) FROM mouseclicks {}", where_clause);
        let clicks: i64 = conn
            .query_row(&sql_clicks, [], |row| row.get(0))
            .unwrap_or(0);

        // Total Scrolls
        let sql_scrolls = format!("SELECT SUM(count) FROM mousescrolls {}", where_clause);
        let scrolls: i64 = conn
            .query_row(&sql_scrolls, [], |row| row.get(0))
            .unwrap_or(0);

        // Total Distance
        let sql_distance = format!(
            "SELECT SUM(distance_inches) FROM mousedistance {}",
            where_clause
        );
        let distance_inches: f64 = conn
            .query_row(&sql_distance, [], |row| row.get(0))
            .unwrap_or(0.0);

        // Clicks by Button
        let sql_buttons = format!(
            "SELECT button, SUM(count) FROM mouseclicks_frequency {} GROUP BY button",
            where_clause
        );
        let mut stmt = conn.prepare(&sql_buttons)?;
        let rows = stmt.query_map([], |row| {
            let button: i64 = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((button, count))
        })?;

        let mut clicks_by_button = HashMap::new();
        for row in rows {
            let (button, count) = row?;
            clicks_by_button.insert(button, count as u64);
        }

        Ok(MouseStats {
            clicks: clicks as u64,
            scrolls: scrolls as u64,
            distance_meters: distance_inches * 0.0254,
            clicks_by_button,
        })
    }

    fn get_where_clause(&self, period: &str) -> String {
        match period {
            "today" => "WHERE day = date('now', 'localtime')".to_string(),
            "yesterday" => "WHERE day = date('now', 'localtime', '-1 day')".to_string(),
            "week" => "WHERE day >= date('now', 'localtime', '-7 days')".to_string(),
            "month" => "WHERE day >= date('now', 'localtime', '-1 month')".to_string(),
            "year" => "WHERE day >= date('now', 'localtime', '-1 year')".to_string(),
            "all" => "WHERE 1=1".to_string(),
            p if p.starts_with("custom:") => {
                let parts: Vec<&str> = p.split(':').collect();
                if parts.len() == 3 {
                    format!("WHERE day >= '{}' AND day <= '{}'", parts[1], parts[2])
                } else {
                    "WHERE 1=1".to_string()
                }
            }
            _ => "WHERE 1=1".to_string(),
        }
    }

    pub fn get_heatmap_stats(&self, period: &str) -> Result<HashMap<String, u64>> {
        let conn = self.get_connection()?;
        let where_clause = self.get_where_clause(period);

        let sql = format!(
            "SELECT key, SUM(count) as total_count FROM keypress_frequency {} GROUP BY key",
            where_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let key_id: i64 = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((key_id, count))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (key_id, count) = row?;
            let key_name = crate::key_mapping::map_key_id_to_name(key_id);
            // Some keys might be duplicates in mapping (e.g. left/right shift?), so we sum them up
            *map.entry(key_name).or_insert(0) += count as u64;
        }

        Ok(map)
    }

    pub fn get_mouse_points(&self, period: &str) -> Result<Vec<(f64, f64)>> {
        let conn = self.get_connection()?;
        let where_clause = self.get_where_clause(period);

        let sql = format!("SELECT x, y FROM mousepoints {}", where_clause);

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let x: f64 = row.get(0)?;
            let y: f64 = row.get(1)?;
            Ok((x, y))
        })?;

        let mut points = Vec::new();
        for row in rows {
            points.push(row?);
        }
        Ok(points)
    }

    pub fn debug_tables(&self) -> Result<Vec<String>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut tables = Vec::new();
        for table in rows {
            tables.push(table?);
        }
        Ok(tables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tables() {
        let db = Database::new().unwrap();
        let conn = db.get_connection().unwrap();

        // Check tables
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let tables = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();

        println!("Tables:");
        for table in tables {
            println!("- {}", table.unwrap());
        }

        // Check keypress frequency
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM keypress_frequency", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        println!("Total rows in keypress_frequency: {}", count);

        // Check mousepoints using new function
        match db.get_mouse_points("all") {
            Ok(points) => {
                println!("Total mouse points fetched: {}", points.len());
                if !points.is_empty() {
                    println!("First 5 points: {:?}", &points[0..5.min(points.len())]);
                }
            }
            Err(e) => {
                println!("Error fetching mouse points: {}", e);
                // Don't fail the test if table doesn't exist or is empty, just log
            }
        }
    }

    #[test]
    fn test_get_heatmap_stats() {
        let db = Database::new().unwrap();
        let stats = db.get_heatmap_stats("all").unwrap();
        // println!("Heatmap Stats (All): {:?}", stats);
        assert!(
            !stats.is_empty(),
            "Heatmap stats should not be empty for 'all' period"
        );
    }

    #[test]
    fn test_inspect_tables() {
        let db = Database::new().unwrap();
        let conn = db.get_connection().unwrap();

        let tables = vec![
            "mouseclicks",
            "mouseclicks_frequency",
            "mousescrolls",
            "mousedistance",
            "pulses",
            "account_pulses",
        ];

        for table in tables {
            println!("--- Schema for {} ---", table);
            let mut stmt = conn
                .prepare(&format!("PRAGMA table_info({})", table))
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    let _cid: i64 = row.get(0)?;
                    let name: String = row.get(1)?;
                    let type_: String = row.get(2)?;
                    Ok(format!("{}: {}", name, type_))
                })
                .unwrap();

            for row in rows {
                println!("{}", row.unwrap());
            }
        }
    }
}
