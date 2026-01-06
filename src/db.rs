use anyhow::{Context, Result};
use directories::BaseDirs;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct MouseStats {
    pub clicks: u64,
    pub scrolls: u64,
    pub distance_meters: f64,
    pub clicks_by_button: HashMap<i64, u64>,
}

#[derive(Debug, Clone)]
pub struct AppStats {
    pub name: String,
    pub keys: u64,
    pub clicks: u64,
    pub scrolls: u64,
    pub download_mb: f64,
    pub upload_mb: f64,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub interface: String,
    pub download_mb: f64,
    pub upload_mb: f64,
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
        // Allow override via environment variable
        if let Ok(path_str) = std::env::var("WTFPULSE_DB_PATH") {
            // println!("DEBUG: Found WTFPULSE_DB_PATH: {}", path_str);
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Ok(path);
            }
            return Err(anyhow::anyhow!(
                "WTFPULSE_DB_PATH specified but file does not exist"
            ));
        }
        // println!("DEBUG: No WTFPULSE_DB_PATH set");

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

        #[cfg(target_os = "linux")]
        {
            if let Some(base_dirs) = BaseDirs::new() {
                let path = base_dirs
                    .data_local_dir()
                    .join("WhatPulse")
                    .join("whatpulse.db");
                if path.exists() {
                    return Ok(path);
                }
                let path = base_dirs
                    .data_local_dir()
                    .join("whatpulse")
                    .join("whatpulse.db");
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // Fallback for other OSs (e.g. macOS) or if OS-specific paths failed
        if let Some(base_dirs) = BaseDirs::new() {
            let path = base_dirs
                .data_local_dir()
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

    pub fn get_mouse_heatmap_grid(
        &self,
        period: &str,
        grid_w: usize,
        grid_h: usize,
    ) -> Result<Vec<Vec<u64>>> {
        let conn = self.get_connection()?;
        let where_clause = self.get_where_clause(period);

        // 1. Get Bounds
        let sql_bounds = format!(
            "SELECT MIN(x), MAX(x), MIN(y), MAX(y) FROM mousepoints {}",
            where_clause
        );

        let bounds: Option<(f64, f64, f64, f64)> = conn
            .query_row(&sql_bounds, [], |row| {
                Ok((
                    row.get::<_, Option<f64>>(0)?.unwrap_or(0.0),
                    row.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
                    row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                    row.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
                ))
            })
            .optional()?;

        let (min_x, max_x, min_y, max_y) = match bounds {
            Some(b) => b,
            None => return Ok(vec![vec![0; grid_w]; grid_h]),
        };

        // Normalize if needed (like the original code did, though logic was slightly weird)
        let is_normalized = min_x >= 0.0 && max_x <= 1.0 && min_y >= 0.0 && max_y <= 1.0;
        let (use_min_x, use_max_x, use_min_y, use_max_y) = if is_normalized {
            (0.0, 1.0, 0.0, 1.0)
        } else {
            (min_x, max_x, min_y, max_y)
        };

        let width = use_max_x - use_min_x;
        let height = use_max_y - use_min_y;

        if width <= 0.0 || height <= 0.0 {
            return Ok(vec![vec![0; grid_w]; grid_h]);
        }

        // 2. Aggregate in SQL
        // We cast to int to get bin indices.
        // bin_x = cast((x - min_x) / width * (grid_w - 1))

        let sql_agg = format!(
            "SELECT 
                CAST((x - ?) / ? * ? AS INTEGER) as bin_x,
                CAST((y - ?) / ? * ? AS INTEGER) as bin_y,
                COUNT(*) as count
             FROM mousepoints 
             {}
             GROUP BY 1, 2",
            where_clause
        );

        let mut stmt = conn.prepare(&sql_agg)?;
        let grid_w_f = (grid_w as f64) - 1.0; // Use grid_w - 1 to map 1.0 inclusive to last index? Or just grid_w and clamp?
        // Original code: (norm_x * (grid_w - 1)).round()
        // Let's match original logic: (x - min) / width * (grid_w - 1)

        // Note: SQLite might return indices out of bounds if floating point errors occur or max_x is exactly hit?
        // We should clamp in Rust or handle carefully.

        let rows = stmt.query_map(
            [
                use_min_x,
                width,
                grid_w_f,
                use_min_y,
                height,
                (grid_h as f64) - 1.0,
            ],
            |row| {
                let bx: i64 = row.get(0)?;
                let by: i64 = row.get(1)?;
                let c: i64 = row.get(2)?;
                Ok((bx, by, c))
            },
        )?;

        let mut grid = vec![vec![0u64; grid_w]; grid_h];

        for row in rows {
            let (bx, by, count) = row?;
            // Clamp just in case
            let x_idx = (bx as usize).clamp(0, grid_w - 1);
            let y_idx = (by as usize).clamp(0, grid_h - 1);
            grid[y_idx][x_idx] += count as u64;
        }

        Ok(grid)
    }

    pub fn get_app_stats(&self, period: &str) -> Result<Vec<AppStats>> {
        let conn = self.get_connection()?;
        let where_clause = self.get_where_clause(period);

        // 1. Input Stats
        let sql_input = format!(
            "SELECT 
                COALESCE(a.product_name, i.path) as name,
                SUM(i.keys) as keys,
                SUM(i.clicks) as clicks,
                SUM(i.scrolls) as scrolls
            FROM input_per_application i
            LEFT JOIN applications a ON i.path = a.path
            {}
            GROUP BY name",
            where_clause
        );

        let mut map: HashMap<String, AppStats> = HashMap::new();

        let mut stmt = conn.prepare(&sql_input)?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let keys: i64 = row.get(1)?;
            let clicks: i64 = row.get(2)?;
            let scrolls: i64 = row.get(3)?;
            Ok((name, keys, clicks, scrolls))
        })?;

        for row in rows {
            let (name, k, c, s) = row?;
            map.insert(
                name.clone(),
                AppStats {
                    name,
                    keys: k as u64,
                    clicks: c as u64,
                    scrolls: s as u64,
                    download_mb: 0.0,
                    upload_mb: 0.0,
                },
            );
        }

        // 2. Bandwidth Stats
        let sql_bandwidth = format!(
            "SELECT 
                COALESCE(a.product_name, b.path) as name,
                SUM(b.download) as download,
                SUM(b.upload) as upload
            FROM application_bandwidth b
            LEFT JOIN applications a ON b.path = a.path
            {}
            GROUP BY name",
            where_clause
        );

        // Check if table exists first? Or just try-catch?
        // Assuming table exists as per schema dump
        if let Ok(mut stmt) = conn.prepare(&sql_bandwidth) {
            let rows = stmt.query_map([], |row| {
                let name: String = row.get(0)?;
                let down: i64 = row.get(1)?;
                let up: i64 = row.get(2)?;
                Ok((name, down, up))
            });

            if let Ok(rows) = rows {
                for (name, d, u) in rows.flatten() {
                    let entry = map.entry(name.clone()).or_insert(AppStats {
                        name: name.clone(),
                        keys: 0,
                        clicks: 0,
                        scrolls: 0,
                        download_mb: 0.0,
                        upload_mb: 0.0,
                    });
                    entry.download_mb += (d as f64) / 1024.0 / 1024.0;
                    entry.upload_mb += (u as f64) / 1024.0 / 1024.0;
                }
            }
        }

        let mut result: Vec<AppStats> = map.into_values().collect();
        result.sort_by(|a, b| b.keys.cmp(&a.keys));
        Ok(result)
    }

    pub fn get_network_stats(&self, period: &str) -> Result<Vec<NetworkStats>> {
        let conn = self.get_connection()?;
        let where_clause = self.get_where_clause(period);

        let sql = format!(
            "SELECT 
                COALESCE(n.description, b.mac_address) as interface,
                SUM(b.download) as download,
                SUM(b.upload) as upload
            FROM network_interface_bandwidth b
            LEFT JOIN network_interfaces n ON b.mac_address = n.mac_address
            {}
            GROUP BY interface",
            where_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let interface: String = row.get(0)?;
            let down: i64 = row.get(1)?;
            let up: i64 = row.get(2)?;
            Ok(NetworkStats {
                interface,
                download_mb: (down as f64) / 1024.0 / 1024.0,
                upload_mb: (up as f64) / 1024.0 / 1024.0,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        result.sort_by(|a, b| {
            b.download_mb
                .partial_cmp(&a.download_mb)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(result)
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
    fn test_inspect_tables() {
        // This test requires a real DB, so it might fail on CI without one.
        // We skip it if DB is not found.
        if let Ok(db) = Database::new() {
            let tables = db.debug_tables();
            assert!(tables.is_ok());
        }
    }
}
