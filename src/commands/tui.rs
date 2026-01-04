use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::sync::mpsc;

use crate::client::WhatpulseClient;
use crate::commands::monitor::spawn_monitor_task;
use crate::tui::{
    app::{App, spawn_fetch},
    event::start_event_listener,
    ui::draw,
};

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Setup App & Channels
    let (tx, mut rx) = mpsc::channel(10);
    let mut app = App::new(client.clone(), tx.clone());

    // 3. Spawn Event Listener
    start_event_listener(tx.clone());

    // 4. Initial Data Fetch
    spawn_fetch(client.clone(), tx.clone());

    // 5. Spawn Monitor Task (Real-time)
    let tx_monitor = tx.clone();
    let (monitor_tx, monitor_rx) = mpsc::channel(10);
    app.set_monitor_tx(monitor_tx);

    tokio::spawn(async move {
        spawn_monitor_task(tx_monitor, monitor_rx).await;
    });

    // 6. Main Loop
    loop {
        terminal.draw(|f| draw(f, &app))?;

        if let Some(action) = rx.recv().await {
            if app.update(action).await {
                break;
            }
        } else {
            break;
        }
    }

    // 6. Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
