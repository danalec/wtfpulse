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

**Controls:**
*   **Global Navigation**:
    *   **Tab / Left / Right**: Navigate between main tabs.
    *   **q / Esc**: Quit.
    *   **r**: Refresh data.

*   **Dashboard Tab**:
    *   **h / l** or **[ / ]**: Cycle through time periods (Today, Yesterday, Week, etc.).
    *   **/**: Switch to "Custom" period and open Date Picker.
    *   **Enter**: Open Date Picker (if "Custom" period is already selected).
    *   **Date Picker**:
        *   **Arrow Keys**: Navigate calendar.
        *   **Enter**: Select Start/End date.
        *   **Esc**: Cancel.

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
Fetches and displays the current user's global statistics, including account name, total keys, clicks, and user ID.

**Usage:**
```bash
wtfpulse user
```

---

### `pulses`
**Source:** [`src/commands/pulses.rs`](../src/commands/pulses.rs)

**Description:**
Retrieves the most recent pulses (updates) sent to the WhatPulse API. By default, it lists the top 5 most recent pulses.

**Usage:**
```bash
wtfpulse pulses
```

---

### `computers`
**Source:** [`src/commands/computers.rs`](../src/commands/computers.rs)

**Description:**
Lists all computers associated with your WhatPulse account, along with their individual statistics (keys, clicks).

**Usage:**
```bash
wtfpulse computers
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
