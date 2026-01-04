# API Modes

`wtfpulse` is designed to work with both the official WhatPulse Web API and the local Client API. This dual-mode approach allows you to access both your historical cloud data and your real-time local statistics.

## 1. Web API (Cloud Mode)

**Enabled when:** `WHATPULSE_API_KEY` is set.

The Web API connects to `api.whatpulse.org` to fetch your global profile data. This is the same data you see on your profile on the WhatPulse website.

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
