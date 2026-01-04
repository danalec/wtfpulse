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
*   **Fetching**: Typed methods like `get_user()`, `get_pulses()`, etc., handle authentication and API requests.

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
| `heatmap` | CLI & TUI | Keyboard heatmap visualization | "Heatmap" |
| `monitor` | CLI & TUI | Real-time kinetic stats | "Kinetic" |
| `raw` | CLI | Raw JSON output | *N/A* |
| `tui` | CLI Entry | Launches the TUI mode | *N/A* |

# API Modes

`wtfpulse` is designed to work with both the official WhatPulse Web API and the local Client API. This dual-mode approach allows you to access both your historical cloud data and your real-time local statistics.

## 1. Web API (Cloud Mode)

**Enabled when:** `WHATPULSE_API_KEY` is set.

The Web API connects to `https://whatpulse.org/api/v1` to fetch your global profile data. This is the same data you see on your profile on the WhatPulse website.

### Capabilities
*   **Historical Data:** Access to your lifetime statistics.
*   **Computers:** List all computers associated with your account.
*   **Pulses:** View history of your recent pulses.
*   **Ranks:** See your global rankings.

### Configuration
To use this mode, you must obtain a Client API Key from the [WhatPulse website](https://whatpulse.org/) (usually under your profile settings or developer section) and set it as an environment variable:

```bash
# Windows (PowerShell)
$env:WHATPULSE_API_KEY = "your-api-key-here"

# Linux/macOS
export WHATPULSE_API_KEY="your-api-key-here"
```

---

## 2. Client API (Local Mode)

**Enabled when:** `WHATPULSE_API_KEY` is **NOT** set.

The Client API is a lightweight JSON HTTP and WebSocket API that is available on the WhatPulse client to retrieve real-time statistics from your local computer. It is a small web server that is embedded in the client and is disabled by default. You will need to enable it before attempting to make a connection.

**Base URL:** `http://localhost:3490/` (REST) / `ws://localhost:3489/` (WebSocket)

### Capabilities
*   **Real-time Stats:** Keys per second, current bandwidth usage.
*   **Unpulsed Stats:** Keys/clicks accumulated since the last pulse.
*   **Privacy:** Data never leaves your local network; no API key required.

### Enabling the Client API
1.  Open your WhatPulse Client.
2.  Go to **Settings** -> **Client API**.
3.  Check **Enable Client API**.
4.  Ensure the port is set to `3490` (REST) and `3489` (WebSocket).
5.  Allow your local IP (usually `127.0.0.1` is allowed by default).

See [this article](https://whatpulse.org/help/docs/software/settings/enabling-the-client-api) for more information.

---

# Client API Reference

Once enabled, you can visit `http://localhost:3490/` to get a Swagger UI with a list of all available API calls. Below is a detailed reference of the endpoints used by `wtfpulse` in Local Mode.

| Endpoint | HTTP Method | Description |
| :-- | :-- | :-- |
| / | GET | Index of all possible calls |
| /v1/account-totals | GET | Get all total account stats |
| /v1/unpulsed | GET | Get all unpulsed stats |
| /v1/all-stats | GET | Get all stats, a combination of all 3 calls above |
| /v1/profiles | GET | Get a list of all profiles. Includes which one is active. |
| /v1/pulse | POST | Ask the client to pulse |
| /v1/open-window | POST | Ask the client to open its window |
| /v1/profiles/activate | POST | Activate a Profile |

### HTTP REST API

If you need real-time statistics flowing from the client to your application, we suggest [using the WebSocket](https://whatpulse.org/help/api/client-api#using-the-websocket).

#### Status Codes

| HTTP Status Code | Description |
| :-- | :-- |
| 200 | Call success, the result is in the body |
| 401 | Connecting IP address not allowed in the client settings |
| 404 | Invalid URL |
| 405 | Invalid HTTP Method (only GET and POST are allowed) |

#### Formatting

When retrieving stats, there will be fields that are suffixed with `_formatted`. These values are formatted in the locale of the client. If you've set it to metric, distances will be formatted in meters. It'll also format the numbers and their separators (dot or comma) like how the computer is configured.

### Account Totals

In the **Account** tab, the client displays the total statistics of your account. Each time the client pulses, these stats are updated. Get your total clicks, keys, download (in MB), upload (in MB), uptime (in seconds), and the online rankings on all those stats.

**HTTP Request:** `GET http://localhost:3490/v1/account-totals`

**Example Response:**
```json
{
  "clicks": "21534236",
  "clicks_formatted": "21.534.236",
  "distance_formatted": "320km, 320m",
  "distance_miles": "199.038",
  "download": "19425048",
  "download_formatted": "4,71TB",
  "keys": "90808209",
  "keys_formatted": "90.808.209",
  "ranks": {
    "rank_clicks": "2605",
    "rank_keys": "448",
    "rank_uptime": "1617"
  },
  "scrolls": "3256578",
  "upload": "4935646",
  "uptime": "195392798"
}
```

### Pulsing

You can remotely execute a pulse via the Client API.

**HTTP Request:** `POST http://localhost:3490/v1/pulse`

**Response:**
```json
{ "msg": "Pulse executed." }
```

### Real-time Stats

The client keeps real-time statistics, like keys and clicks per second, and the current download and upload. This API call gives you access to that real-time data.

**HTTP Request:** `POST http://localhost:3490/v1/realtime`

**Response:**
```json
{
  "clicks": "0,10",
  "download": "21KB/s",
  "keys": "2,17",
  "upload": "2KB/s"
}
```

### Unpulsed Stats

The stats that are accumulated between pulses are accessible through this call. Get the unpulsed clicks, keys, download (in bytes), upload (in bytes), and uptime (in seconds).

**HTTP Request:** `GET http://localhost:3490/v1/unpulsed`

**Response:**
```json
{
  "clicks": 3474,
  "download": 3444883866,
  "keys": 22061,
  "upload": 230877183,
  "uptime": 22346
}
```

### Profiles

You can find a list of all created profiles and activate them.

**Get List:** `GET http://localhost:3490/v1/profiles`
**Activate:** `POST http://localhost:3490/v1/profiles/activate` with body `{ "profile_id": 1 }`

---

## Using the WebSocket

[WebSockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API) are connections that stay open and generally have a web-based message structure. If you're looking for (near) real-time statistics from WhatPulse to your application, this is your best option.

**Default Port:** `3489`

### Identifying

Before WhatPulse will start sending statistics to a WebSocket client, it needs to identify itself.

```json
{
  "source": "plugin",
  "action": "identify"
}
```

### Receiving Stats

After you've identified, WhatPulse will start sending stats every 2 seconds.

```json
{
  "action": "update-status",
  "data": {
    "account-totals": { ... },
    "realtime": { ... },
    "unpulsed": { ... }
  }
}
```

### Actions

You can ask WhatPulse to pulse and open the main window via the websocket:

```json
{ "source": "plugin", "action": "pulse" }
{ "source": "plugin", "action": "open-window" }
```
