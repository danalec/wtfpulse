# Developer & AI Guide

This document provides a comprehensive overview of the `wtfpulse` project structure, architecture, and development patterns. It is intended to guide developers in understanding the codebase and implementing new features.

## Project Overview

`wtfpulse` is a Rust-based CLI and TUI client for WhatPulse. It uses a modular architecture to separate concerns and allow for easy extension.

### Key Technologies
- **Runtime**: `tokio` (Async/Await)
- **CLI**: `clap` (Command parsing)
- **TUI**: `ratatui` (Rendering), `crossterm` (Events)
- **HTTP**: `reqwest` (API Client)
- **Serialization**: `serde` / `serde_json`
- **Plugin System**: `inventory` (Decentralized module registration)
- **Units**: `uom` (Units of measurement, e.g., in Calorimetry)

## Directory Structure

```
src/
├── client.rs       # Core API Client (WhatpulseClient)
├── main.rs         # Application Entry Point & CLI Setup
├── commands/       # Feature Modules (CLI & TUI Logic)
│   ├── mod.rs      # Command Registry & Dispatcher
│   ├── user.rs     # 'user' command & dashboard
│   ├── pulses.rs   # 'pulses' command
│   ├── calorimetry.rs # 'calorimetry' command & physics logic
│   ├── computers.rs   # 'computers' command
│   └── ...
└── tui/            # Shared TUI Infrastructure
    ├── app.rs      # Global Application State (App struct)
    ├── event.rs    # Event Loop & Input Handling
    ├── tabs.rs     # Tab Navigation Component
    └── components/ # Reusable UI widgets
```

## Architecture Patterns

### 1. The Command System ("Drag-and-Drop")

The project uses a decentralized command pattern where each feature is a self-contained module in `src/commands/`.

*   **Definition**: Each module (e.g., `src/commands/my_feature.rs`) exposes a `pub async fn execute(client: &WhatpulseClient) -> Result<()>` function.
*   **Registration**:
    1.  The module is declared in `src/commands/mod.rs`.
    2.  An enum variant is added to `Commands` in `src/commands/mod.rs`.
    3.  The `execute` method in `Commands` matches the variant to the module's function.

### 2. The TUI System

The TUI architecture combines **The Elm Architecture (Model-View-Update)** pattern with a decentralized plugin system.

#### Elm Architecture Adaptation
*   **App State (`src/tui/app.rs`)**: Holds all data, loading states, and configuration (including `DatePickerState` and `TimePeriod` selection).
*   **Events (`src/tui/event.rs`)**: Handles keyboard input and tick events.
*   **UI Components**: Rendering logic is distributed between common widgets (`src/tui/`) and screen-specific views (`src/commands/`).

#### Decentralized Registration (Inventory)
The project uses the `inventory` crate to allow modules to self-register their UI pages without modifying the core TUI loop.

*   **Registry**: `src/commands/mod.rs` defines the `TuiPage` struct and collects them.
*   **Registration**: Modules use `inventory::submit!` to register a `TuiPage`.
*   **Rendering**: The main loop iterates over registered pages to render the active tab.

### 3. Data Access

*   **Client**: `src/client.rs` provides `WhatpulseClient`.
*   **Fetching**: Methods like `get_resource::<T>(path)` handle authentication and JSON deserialization.

## Implementation Guide

### How to Add a New Command (CLI Only)

1.  **Create Module**: `src/commands/hello.rs`
    ```rust
    use anyhow::Result;
    use crate::client::WhatpulseClient;

    pub async fn execute(client: &WhatpulseClient) -> Result<()> {
        println!("Hello World");
        Ok(())
    }
    ```

2.  **Register**: Update `src/commands/mod.rs`.
    ```rust
    // 1. Declare the module
    pub mod hello;

    // 2. Add to the Enum
    #[derive(Subcommand)]
    pub enum Commands {
        // ... existing commands
        /// Prints Hello World
        Hello,
    }

    // 3. Add to the Dispatcher
    impl Commands {
        pub async fn execute(self, client: &WhatpulseClient) -> Result<()> {
            match self {
                // ...
                Commands::Hello => hello::execute(client).await,
            }
        }
    }
    ```

3.  **Done!**: Recompile, and `wtfpulse hello` is now available.

### How to Add a New TUI Page

1.  **Create Module**: `src/commands/stats.rs`
2.  **Define Logic**:
    ```rust
    use crate::tui::app::App;
    use crate::commands::TuiPage;
    use ratatui::Frame;
    use ratatui::layout::Rect;
    use crossterm::event::KeyEvent;

    inventory::submit! {
        TuiPage {
            title: "Stats",
            render: render_tui,
            handle_key: handle_key,
            priority: 50, // Higher numbers appear later in the tab list
        }
    }

    pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
        // Render widgets
    }

    fn handle_key(app: &mut App, key: KeyEvent) -> bool {
        // Handle input. Return true if consumed.
        false
    }
    
    // Optional: Add CLI entry point if needed
    pub async fn execute(client: &WhatpulseClient) -> anyhow::Result<()> {
        Ok(())
    }
    ```
3.  **Register**: Update `src/commands/mod.rs` (declare module). Even if it has no CLI command, it must be compiled (mod declaration) for inventory to pick it up.

## Current Module Audit

| Module | Type | Description | TUI Page |
| :--- | :--- | :--- | :--- |
| `user` | CLI & TUI | User stats, keys, clicks | "Dashboard" |
| `calorimetry` | CLI & TUI | Physics calculations (Energy, Force) | "Calorimetry" |
| `pulses` | CLI & TUI | List recent pulses | "Pulses" |
| `computers` | CLI & TUI | List computers | "Computers" |
| `monitor` | CLI & TUI | Real-time kinetic stats | "Kinetic" |
| `raw` | CLI | Raw JSON output | *N/A* |
| `tui` | CLI Entry | Launches the TUI mode | *N/A* |