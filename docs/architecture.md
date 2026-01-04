# Architecture & Modularity Guide

`wtfpulse` is designed with modularity in mind, making it easy to extend with new commands and features. This document explains the architectural decisions and provides a guide for adding new functionality.

## Project Structure

The codebase is organized into clear domains:

```
src/
├── client.rs       # Centralized API client (authentication, request handling)
├── main.rs         # Entry point, CLI argument parsing
├── tui/            # Interactive Terminal User Interface modules
│   ├── app.rs      # State management
│   ├── components/ # Reusable UI widgets (tabs, dashboard, etc.)
│   └── ...
└── commands/       # Individual CLI commands
    ├── mod.rs      # Command registry and dispatch
    ├── user.rs     # 'user' command logic
    ├── pulses.rs   # 'pulses' command logic
    └── ...
```

## The Command System

The command system uses a "drag-and-drop" style architecture. Each command is a self-contained module.

### How to Add a New Command

1.  **Create the Module**:
    Create a new file in `src/commands/` (e.g., `src/commands/my_feature.rs`).

    ```rust
    // src/commands/my_feature.rs
    use anyhow::Result;
    use crate::client::WhatpulseClient;

    pub async fn execute(client: &WhatpulseClient) -> Result<()> {
        // Your logic here
        println!("My feature is running!");
        Ok(())
    }
    ```

2.  **Register the Module**:
    Open `src/commands/mod.rs` and add your module.

    ```rust
    // 1. Declare the module
    pub mod my_feature;

    // 2. Add to the Enum
    #[derive(Subcommand)]
    pub enum Commands {
        // ... existing commands
        /// Description for help text
        MyFeature,
    }

    // 3. Add to the Dispatcher
    impl Commands {
        pub async fn execute(self, client: &WhatpulseClient) -> Result<()> {
            match self {
                // ...
                Commands::MyFeature => my_feature::execute(client).await,
            }
        }
    }
    ```

3.  **Done!**: Recompile, and `wtfpulse my-feature` is now available.

## TUI Architecture

The TUI (Terminal User Interface) follows The Elm Architecture (Model-View-Update) pattern partially, adapted for Rust.

*   **App State (`src/tui/app.rs`)**: Holds all data, loading states, and configuration (including `DatePickerState` and `TimePeriod` selection).
*   **Events (`src/tui/event.rs`)**: Handles keyboard input and tick events.
*   **UI Components**: Rendering logic is distributed:
    *   **Common Widgets**: Located in `src/tui/` (e.g., `tabs.rs`).
    *   **Screen-Specific Views**: Located in `src/commands/<module>.rs` (via `render_tui`).

### Adding a TUI Tab

The TUI uses a decentralized plugin system powered by the `inventory` crate. This allows command modules to register their own TUI pages without modifying the core app logic.

1.  **Define the Page**: In your command module (e.g., `src/commands/my_feature.rs`), create a `render_tui` function and a `handle_key` function.
2.  **Register with Inventory**: Use the `inventory::submit!` macro to register your page.

```rust
// src/commands/my_feature.rs
use crate::commands::TuiPage;
use crate::tui::app::App;
use crossterm::event::KeyEvent;

inventory::submit! {
    TuiPage {
        title: "My Feature",
        render: render_tui,
        handle_key: handle_key,
        priority: 100, // Higher numbers appear later in the tab list
    }
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    // ... rendering logic
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // ... event handling logic
}
```

The application will automatically collect all registered `TuiPage` instances and display them in the tab bar.
