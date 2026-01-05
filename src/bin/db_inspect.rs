use anyhow::Result;
use directories::BaseDirs;
use rusqlite::Connection;
use std::path::PathBuf;

fn main() -> Result<()> {
    let path = find_db_path()?;
    println!("Database path: {:?}", path);

    let conn = Connection::open(&path)?;

    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for table in tables {
        println!("\nTable: {}", table);
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let columns = stmt.query_map([], |row| {
            let name: String = row.get(1)?;
            let type_: String = row.get(2)?;
            Ok(format!("  {}: {}", name, type_))
        })?;

        for col in columns {
            println!("{}", col?);
        }
    }

    Ok(())
}

fn find_db_path() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA")?;
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

    Err(anyhow::anyhow!("Could not find whatpulse.db"))
}
