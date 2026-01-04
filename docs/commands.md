# CLI Commands Reference

This document provides a detailed reference for the `wtfpulse` command-line interface.

> **Want to add a new command?**  
> Check out the [Contribution Guide](contributing.md) to learn how to create your own module.

## Available Commands

### `calorimetry`
**Source:** [`src/commands/calorimetry.rs`](../src/commands/calorimetry.rs)

**Description:**
Calculates the estimated energy expenditure (in calories) based on your total keystrokes. This module applies physics principles to estimate the work performed by your fingers.

**How it works:**
The calculation assumes the usage of a standard mechanical keyboard switch. In the TUI mode, you can interactively switch between profiles (e.g., Cherry MX Red, Blue, Brown, Membrane) to see how different force/distance parameters affect the result.

**Formula:**
$$ Work (J) = Force (N) \times Distance (m) \times Keystrokes $$

The result is then converted from Joules to:
*   **Calories (cal)**: Small calories.
*   **Kilocalories (kcal)**: Food calories.

**Fun Comparisons:**
The command also provides fun comparisons to put the energy into perspective:
*   **M&Ms**: How many M&Ms worth of energy you've burned.
*   **Running**: Equivalent time spent running (at a pace burning ~10 kcal/min).

**Usage:**
```bash
wtfpulse calorimetry
```
*Note: For the interactive version with profile switching, use `wtfpulse tui`.*

**Example Output:**
```text
Fetching latest pulse data...

Energy Expenditure Report:
──────────────────────────
Total Keystrokes: 15,432,109
Work Performed:   27,777.80 J
Calories Burned:  6,639.03 cal
                  6.6390 kcal
──────────────────────────
Fun Comparisons:
• Equivalent to 0.6639 M&Ms
• Like running for 39.8 seconds
```

---

### `tui`
**Source:** [`src/tui/`](../src/tui/)

**Description:**
Launches the interactive Terminal User Interface (dashboard). This mode provides a tabbed interface to view user stats, computer lists, and calorimetry data in a more visual way.

**Modes:**
*   **Web Mode:** Displays historical data with time period selection (Today, Yesterday, Week, etc.). Requires `WHATPULSE_API_KEY`.
*   **Local Mode:** Displays real-time statistics directly from the local WhatPulse client. Shows Total Stats, Real-time Keys/sec, and Unpulsed Stats. No API Key required.

**Controls:**
*   **Global Navigation**:
    *   **Tab / Left / Right**: Navigate between main tabs.
    *   **q / Esc**: Quit.
    *   **r**: Refresh data.

*   **Dashboard Tab (Web Mode)**:
    *   **h / l** or **[ / ]**: Cycle through time periods (Today, Yesterday, Week, etc.).
    *   **/**: Switch to "Custom" period and open Date Picker.
    *   **Enter**: Open Date Picker (if "Custom" period is already selected).
    *   **Date Picker**:
        *   **Arrow Keys**: Navigate calendar.
        *   **Enter**: Select Start/End date.
        *   **Esc**: Cancel.

*   **Dashboard Tab (Local Mode)**:
    *   Time period selection is disabled (real-time data only).

*   **Calorimetry Tab**:
    *   **p**: Cycle through keyboard switch profiles.

**Usage:**
```bash
wtfpulse tui
```

---

### `user`
**Source:** [`src/commands/user.rs`](../src/commands/user.rs)

**Description:**
Fetches and displays the current user's global statistics.
*   **Web Mode:** Fetches from `api.whatpulse.org`. Includes Account Name, ID, Total Keys, Clicks, and Ranks.
*   **Local Mode:** Fetches from `localhost:3490/v1/account-totals`. Includes Total Keys, Clicks, Download, Upload, and Uptime.

**Usage:**
```bash
wtfpulse user
```

---

### `pulses`
**Source:** [`src/commands/pulses.rs`](../src/commands/pulses.rs)

**Description:**
Retrieves the most recent pulses (updates) sent to the WhatPulse API.
*   **Requirement:** Only available in **Web Mode** (requires `WHATPULSE_API_KEY`).
*   **Local Mode:** Displays a message explaining that pulse history is not available locally.

**Usage:**
```bash
wtfpulse pulses
```

---

### `computers`
**Source:** [`src/commands/computers.rs`](../src/commands/computers.rs)

**Description:**
Lists all computers associated with your WhatPulse account.
*   **Requirement:** Only available in **Web Mode** (requires `WHATPULSE_API_KEY`).
*   **Local Mode:** Displays a message explaining that an API key is required to view per-computer stats.

**Usage:**
```bash
wtfpulse computers
```

---

### `monitor` (Kinetic)
**Source:** [`src/commands/monitor.rs`](../src/commands/monitor.rs)

**Description:**
A real-time, physics-based dashboard for visualizing your typing mechanics. This command works best in the TUI (as the "Kinetic" tab) but can also be run in CLI mode to stream raw WebSocket data.

**TUI Features (Kinetic Tab):**
*   **Real-time Gauges**: Displays instantaneous Power (Watts) and Keys/sec.
*   **Physics Telemetry**:
    *   **Peak Velocity**: Maximum speed of finger travel (m/s or cm/s).
    *   **Burst Acceleration**: Maximum acceleration during typing bursts.
    *   **Work**: Accumulated energy in Joules.
*   **Sparkline**: A scrolling graph of power output over time.

**Controls:**
*   **`u`**: Toggle units between **Metric** (m/s) and **Centimeters** (cm/s).
*   **`p`**: Switch keyboard profiles (e.g., Cherry MX Red vs Blue) to adjust force/distance constants.
*   **`Space`**: Manually trigger a pulse (Local Mode only).

**Usage (CLI Mode):**
Streams raw JSON events from the local WhatPulse client WebSocket.
```bash
wtfpulse monitor
```

---

### `raw`
**Source:** [`src/commands/raw.rs`](../src/commands/raw.rs)

**Description:**
Allows you to query any arbitrary API endpoint path directly. Useful for debugging or accessing data not yet covered by specific commands.

**Usage:**
```bash
wtfpulse raw <PATH>
```

**Example:**
```bash
wtfpulse raw user.php
```
