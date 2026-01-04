# Extending `wtfpulse`

`wtfpulse` is designed to be modular and easily extensible. This guide explains how to add new commands to the CLI.

## Command Architecture

The project structure separates the API client logic from the command execution logic:

- **`src/client.rs`**: Contains the `WhatpulseClient` struct and response types (`UserResponse`, `PulseResponse`, etc.). This handles authentication and raw API requests.
- **`src/commands/`**: This directory contains the implementation of each subcommand.
- **`src/commands/mod.rs`**: This is the central registry for commands. It defines the `Commands` enum and dispatches execution.
- **`src/main.rs`**: The entry point. It parses arguments, initializes the client, and delegates to `Commands::execute`.

## Adding a New Command

To add a new command (e.g., `dashboard`), follow these steps:

### 1. Create the Command Module

Create a new file `src/commands/dashboard.rs`. This file should export an async `execute` function that takes a reference to the client.

```rust
// src/commands/dashboard.rs
use anyhow::Result;
use crate::client::WhatpulseClient;

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    println!("This is the dashboard command!");
    // Your logic here...
    Ok(())
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

### 5. Verify

Run `cargo check` to ensure everything compiles. Your new command is now available!

```bash
cargo run -- dashboard
```

## Best Practices

- **Keep it focused**: Each command file should handle one specific task or domain.
- **Use `anyhow`**: Return `anyhow::Result<()>` for simplified error handling.
- **Reuse Client Methods**: Use `client.get_resource::<T>("resource")` or `client.get_json::<T>("path")` to fetch data.
