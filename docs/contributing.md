# Extending `wtfpulse`

`wtfpulse` is designed to be modular and easily extensible. This guide explains how to add new commands to the CLI and components to the TUI.

## Command Architecture

The project structure separates the API client logic from the command execution logic:

- **`src/client.rs`**: Contains the `WhatpulseClient` struct and response types (`UserResponse`, `PulseResponse`, etc.). This handles authentication and raw API requests.
- **`src/commands/`**: This directory contains the implementation of each subcommand.
- **`src/commands/mod.rs`**: This is the central registry for commands. It defines the `Commands` enum and dispatches execution.
- **`src/tui/`**: Contains the TUI application logic.
- **`src/main.rs`**: The entry point. It parses arguments, initializes the client, and delegates to `Commands::execute`.

## Adding a New Command

To add a new command (e.g., `dashboard`), follow these steps:

### 1. Create the Command Module

Create a new file `src/commands/dashboard.rs`. This file should export an async `execute` function for the CLI and optionally register itself for the TUI.

```rust
// src/commands/dashboard.rs
use anyhow::Result;
use crate::client::WhatpulseClient;
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::tui::app::App;
use crate::commands::TuiPage;
use crossterm::event::KeyEvent;

// CLI Execution Logic
pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    println!("This is the dashboard command!");
    // Your logic here...
    Ok(())
}

// TUI Registration
inventory::submit! {
    TuiPage {
        title: "Dashboard",
        render: render_tui,
        handle_key: handle_key,
        priority: 0,
    }
}

// TUI Rendering Logic
pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    // Render your widgets here...
}

// TUI Event Handling
fn handle_key(app: &mut App, key: KeyEvent) {
    // Handle key presses...
}
```

### 2. Register the Module

Open `src/commands/mod.rs` and add your module to the list of public modules:

```rust
// src/commands/mod.rs
pub mod calorimetry;
pub mod user;
pub mod pulses;
pub mod computers;
pub mod raw;
pub mod dashboard; // <--- Add this line
```

### 3. Add to the CLI Enum

In `src/commands/mod.rs`, add a new variant to the `Commands` enum. Use doc comments to define the help text shown in the CLI.

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...
    
    /// Show the interactive dashboard
    Dashboard, 
}
```

### 4. Dispatch Execution

In `src/commands/mod.rs`, add a match arm to the `execute` method:

```rust
impl Commands {
    pub async fn execute(self, client: &WhatpulseClient) -> Result<()> {
        match self {
            Commands::User => user::execute(client).await,
            // ...
            Commands::Dashboard => dashboard::execute(client).await, // <--- Add this line
        }
    }
}
```

## Adding a TUI Component

If your command has a corresponding TUI view, you need to register it using the `inventory` system as shown above.

1.  Import `crate::commands::TuiPage` and `crossterm::event::KeyEvent`.
2.  Implement `render_tui` and `handle_key` functions.
3.  Use `inventory::submit!` to register the page.
4.  Set a `priority` to control the tab order (lower numbers appear first).

The application automatically discovers all registered pages at runtime.

### 3. Verify

Run `cargo check` to ensure everything compiles. Your new command is now available!

```bash
cargo run -- dashboard
# Or open the TUI
cargo run -- tui
```

## Best Practices

- **Keep it focused**: Each command file should handle one specific task or domain.
- **Use `anyhow`**: Return `anyhow::Result<()>` for simplified error handling.
- **Reuse Client Methods**: Use `client.get_resource::<T>("resource")` or `client.get_json::<T>("path")` to fetch data.
- **Unit Tests**: Add tests within your command module to verify logic and TUI rendering.
