use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::PathBuf;

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

    pub fn get_heatmap_stats(&self, period: &str) -> Result<HashMap<String, u64>> {
        let conn = self.get_connection()?;

        let where_clause = match period {
            "today" => "WHERE day = date('now', 'localtime')",
            "yesterday" => "WHERE day = date('now', 'localtime', '-1 day')",
            "week" => "WHERE day >= date('now', 'localtime', '-7 days')",
            "month" => "WHERE day >= date('now', 'localtime', '-1 month')",
            "year" => "WHERE day >= date('now', 'localtime', '-1 year')",
            "all" => "WHERE 1=1",
            _ => "WHERE 1=1",
        };

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
        // Inspect keypress_frequency data
        let conn = db.get_connection().unwrap();
        // println!("--- Sample Data from keypress_frequency ---");
        let mut stmt = conn
            .prepare("SELECT day, hour, key, count, profile_id FROM keypress_frequency LIMIT 1")
            .unwrap();
        let _rows = stmt
            .query_map([], |row| {
                let day: String = row.get(0)?;
                let hour: i32 = row.get(1)?;
                let key: i32 = row.get(2)?;
                let count: i64 = row.get(3)?;
                Ok((day, hour, key, count))
            })
            .unwrap();
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
}
