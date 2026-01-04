# wtfpulse

**wtfpulse** is a blazing fast, asynchronous CLI client for the WhatPulse Web API, built with Rust. It allows developers and power users to programmatically access their WhatPulse statistics, including user details, computer lists, and recent pulses, directly from the terminal.

## Introduction

### Purpose and Scope
The goal of `wtfpulse` is to provide a type-safe, efficient, and easy-to-use command-line interface for the WhatPulse Web API. Unlike the local Client API (which requires the WhatPulse client to be running locally), this tool connects directly to `api.whatpulse.org`, making it suitable for server-side scripts, CI/CD pipelines, or standalone monitoring tools.

### Target Audience
- **Developers** integrating WhatPulse stats into their dashboards.
- **Power Users** who prefer CLI tools over web interfaces.
- **Data Enthusiasts** wanting to export or analyze their input history programmatically.

---

## Installation Guide

### System Requirements
- **OS**: Windows, macOS, or Linux
- **Rust Toolchain**: 1.70.0 or later (includes `cargo`)
- **Network**: Internet connection to reach `api.whatpulse.org`

### Step-by-Step Installation

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/yourusername/wtfpulse.git
    cd wtfpulse
    ```

2.  **Build the Project**
    Use Cargo to build the release binary:
    ```bash
    cargo build --release
    ```

3.  **Verify Installation**
    Run the binary to see the help message:
    ```bash
    ./target/release/wtfpulse --help
    ```

### Configuration
The tool relies on an environment variable for authentication. You must generate a **Web API Token** from your WhatPulse account settings.

**PowerShell:**
```powershell
$env:WHATPULSE_API_KEY = "your-long-bearer-token"
```

**Bash/Zsh:**
```bash
export WHATPULSE_API_KEY="your-long-bearer-token"
```

---

## Usage Documentation

For a complete list of commands and detailed explanations, please refer to the [Commands Reference](docs/commands.md).

If you want to contribute or add new commands, check out the [Contribution Guide](docs/contributing.md).

`wtfpulse` uses a subcommand structure. The general syntax is:
```bash
wtfpulse <SUBCOMMAND>
```

### Core Features
- **User Stats**: View global keys, clicks, and rank.
- **Pulses**: List recent pulse history.
- **Computers**: Enumerate all computers associated with the account.
- **Calorimetry**: Calculate energy burned by typing (physics-based estimation).
- **Raw Access**: Query any API endpoint manually for debugging or new features.

### Practical Examples

#### Example 1: Fetching User Statistics
**Description**: Retrieve your global account statistics, including total keys and clicks.

**Code**:
```bash
# Assuming binary is in path or run via cargo
cargo run -- user
```

**Expected Output**:
```text
User: UserResponse { id: Some("12345"), username: Some("JaneDoe"), keys: Some(15000000), clicks: Some(5000000), ... }
Username: JaneDoe
```

#### Example 2: Listing Recent Pulses
**Description**: View the last 5 pulses to track your recent activity.

**Code**:
```bash
cargo run -- pulses
```

**Expected Output**:
```text
Found 5 pulses:
Pulse 987654: 5200 keys on My-Laptop
Pulse 987653: 1200 keys on Work-PC
Pulse 987652: 8500 keys on My-Laptop
Pulse 987651: 300 keys on Work-PC
Pulse 987650: 15000 keys on Gaming-Rig
```

#### Example 3: Debugging with Raw JSON
**Description**: If a new endpoint isn't supported yet, or you want to see the raw JSON structure, use the `raw` command.

**Code**:
```bash
# Fetch raw data for the user endpoint
cargo run -- raw /api/v1/user
```

**Expected Output**:
```json
{
  "AccountName": "JaneDoe",
  "Country": "United States",
  "Keys": "15000000",
  "Ranks": {
    "Keys": 42
  }
}
```

---

## API Reference

This tool maps to the standard WhatPulse Web API.

| Command | Target Endpoint | Description |
| :--- | :--- | :--- |
| `user` | `/api/v1/user` | Fetches `UserResponse` object. |
| `pulses` | `/api/v1/pulses` | Fetches array of `PulseResponse` objects. |
| `computers` | `/api/v1/computers` | Fetches array of `ComputerResponse` objects. |

**Note**: The actual endpoints (`/api/v1/...`) in the code are placeholders based on the task description. In a real-world scenario, these would match the official `api.whatpulse.org` PHP endpoints (e.g., `user.php`).

### Error Codes
- **401 Unauthorized**: Your `WHATPULSE_API_KEY` is missing or invalid.
- **404 Not Found**: The endpoint path is incorrect.
- **500 Internal Server Error**: The WhatPulse API is experiencing issues.

---

## Troubleshooting

### Common Issues

1.  **"set WHATPULSE_API_KEY environment variable..."**
    - **Cause**: The tool cannot find the API key in your environment.
    - **Solution**: Export the variable as shown in the Configuration section.

2.  **"request failed: GET ..."**
    - **Cause**: Network connectivity issue or DNS failure.
    - **Solution**: Check your internet connection and ensure `api.whatpulse.org` is reachable.

3.  **JSON Parse Error**
    - **Cause**: The API response format has changed or is unexpected.
    - **Solution**: Use the `raw` command to inspect the actual JSON response and report an issue.

### Debugging
To see more details, you can run the tool with Rust backtraces enabled if it crashes:
```bash
RUST_BACKTRACE=1 cargo run -- user
```

---

## FAQ

**Q: Can I use this with the local client API?**
A: No, this tool is designed for the Web API. The local client API runs on `localhost:3490` and has a different authentication mechanism.

**Q: Is this official?**
A: No, this is a community-driven open-source project.

**Q: How often can I pull data?**
A: Respect WhatPulse's API rate limits. Generally, do not poll more than once every few minutes.

---

## Contributing Guidelines

We welcome contributions!

1.  **Reporting Issues**: Open an issue on GitHub describing the bug or feature request.
2.  **Pull Requests**:
    - Fork the repo.
    - Create a feature branch (`git checkout -b feature/amazing-feature`).
    - Commit your changes.
    - Push to the branch.
    - Open a Pull Request.
3.  **Code Style**: Run `cargo fmt` before committing to ensure standard Rust formatting.

---

## License Information

Copyright (c) 2026.

This project is licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

**Usage Restrictions**:
- You may freely use, modify, and distribute this software.
- Attribution is required.
- No warranty is provided.
