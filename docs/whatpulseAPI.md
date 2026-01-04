# Client API

The Client API is a lightweight JSON HTTP and WebSocket API that is available on the WhatPulse client to retrieve real-time statistics from your local computer. It is a small web server that is embedded in the client and is disabled by default. You will need to enable it before attempting to make a connection, see [this article](https://whatpulse.org/help/docs/software/settings/enabling-the-client-api) for more information on the setting. The Client API does not need authentication, only enabling and the connecting IP address has to be allowed.[^1]

Once it is enabled, you can visit the main index webpage to get Swagger UI with a list of all available API calls and an easy way to try them. By default, this index can be found here: [http://localhost:3490/](http://localhost:3490/). At this moment, these are the possible API calls:[^1]


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

Let's dive into each one separately in the next chapters.[^1]

***

## HTTP REST API

First, let's cover the HTTP REST API for general purposes. If you need real-time statistics flowing from the client to your application, we suggest [using the WebSocket](https://whatpulse.org/help/api/client-api#using-the-websocket).[^1]

### Status Codes

Before we go into the available API calls, here's an overview of the possible status codes the client can return:[^1]


| HTTP Status Code | Description |
| :-- | :-- |
| 200 | Call success, the result is in the body |
| 401 | Connecting IP address not allowed in the client settings |
| 404 | Invalid URL |
| 405 | Invalid HTTP Method (only GET and POST are allowed) |

### Formatting

When retrieving stats, there will be fields that are suffixed with `_formatted`. These values are formatted in the locale of the client. If you've set it to metric, distances will be formatted in meters. It'll also format the numbers and their separators (dot or comma) like how the computer is configured.[^1]

### Account Totals

In the **Account** tab, the client displays the total statistics of your account. Each time the client pulses, these stats are updated. Get your total clicks, keys, download (in MB), upload (in MB), uptime (in seconds), and the online rankings on all those stats.[^1]

#### HTTP Request

`GET http://localhost:3490/v1/account-totals`[^1]

#### Examples (PHP)

```php
$json   = file_get_contents("http://localhost:3490/v1/account-totals");
$result = json_decode($json);
$keys   = $result->keys;
$clicks = $result->clicks;
$rank_uptime = $result->ranks->rank_uptime;
echo "Current keys: " . $keys . ", current clicks: " . $clicks . ", rank in uptime: " . $rank_uptime;
?>
```


#### Results

If successful, this call will return a JSON formatted array with the total keys, clicks, download, upload, uptime, and the ranks of your account. These values are updated from the website each time you pulse.[^1]

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
    "rank_clicks_formatted": "2.605th",
    "rank_distance": "2021",
    "rank_distance_formatted": "2.021st",
    "rank_download": "2407",
    "rank_download_formatted": "2.407th",
    "rank_keys": "448",
    "rank_keys_formatted": "448th",
    "rank_scrolls": "790",
    "rank_scrolls_formatted": "790th",
    "rank_upload": "2887",
    "rank_upload_formatted": "2.887th",
    "rank_uptime": "1617",
    "rank_uptime_formatted": "1.617th"
  },
  "scrolls": "3256578",
  "scrolls_formatted": "3.256.578",
  "upload": "4935646",
  "upload_formatted": "4,71TB",
  "uptime": "195392798",
  "uptime_formatted": "6y, 125d, 11h, 46m"
}
```


***

### Pulsing

You can remotely execute a pulse via the Client API.[^1]

#### HTTP Request

`POST http://localhost:3490/v1/pulse`[^1]

#### Examples (PHP)

```php
$url  = "http://localhost:3490/v1/pulse";
$curl = curl_init($url);
curl_setopt($curl, CURLOPT_POST, 1);
curl_setopt($curl, CURLOPT_RETURNTRANSFER, true);
$response = curl_exec($curl);
curl_close($curl);
$result = json_decode($response);
?>
```


#### Results

If successful, this call will return a JSON formatted array with the field `"msg"` which displays whether the action is successful.
Please note that at this time, it will always return `"Pulse executed."` as pulsing is an asynchronous process and the API call does not wait around for the result. Future versions will return the actual result of the pulse.[^1]

```json
{ "msg": "Pulse executed." }
```


***

### Real-time Stats

The client keeps real-time statistics, like keys and clicks per second, and the current download and upload. This API call gives you access to that real-time data.[^1]

Note: Available from client version 2.8 and up.[^1]

#### HTTP Request

`POST http://localhost:3490/v1/realtime`[^1]

#### Examples (PHP)

```php
$json   = file_get_contents("http://localhost:3490/v1/realtime");
$result = json_decode($json);
$keys   = $result->keys;
$download = $result->download;
echo "Keys per second: " . $keys . ", download rate: " . $download;
?>
```


#### Results

The results of the API call will be a JSON formatted array with the current keys and clicks per second, download, and upload rates (KB/s, MB/s, GB/s). These values are updated in real-time with an average of the last 5 seconds.[^1]

```json
{
  "clicks": "0,10",
  "download": "21KB/s",
  "keys": "2,17",
  "upload": "2KB/s"
}
```


***

### Unpulsed Stats

The stats that are accumulated between pulses are accessible through this call. Get the unpulsed clicks, keys, download (in bytes), upload (in bytes), and uptime (in seconds).[^1]

#### HTTP Request

`GET http://localhost:3490/v1/unpulsed`[^1]

#### Examples (PHP)

```php
$json   = file_get_contents("http://localhost:3490/v1/unpulsed");
$result = json_decode($json);
$keys   = $result->keys;
$clicks = $result->clicks;
echo "Current keys: " . $keys . ", current clicks: " . $clicks;
?>
```


#### Results

If successful, this call will return a JSON formatted array with the current unpulsed keys, clicks, download (in bytes), upload (in bytes), and uptime (in seconds). These values are updated in real-time.[^1]

```json
{
  "clicks": 3474,
  "download": 3444883866,
  "keys": 22061,
  "upload": 230877183,
  "uptime": 22346
}
```


***

### Profiles: Get a list

You can find a list of all created profiles using this GET request.[^1]

#### HTTP Request

`GET http://localhost:3490/v1/profiles`[^1]

#### Examples (PHP)

```php
$json   = file_get_contents("http://localhost:3490/v1/profiles");
$result = json_decode($json);
var_dump($result);
?>
```


#### Results

If successful, this call will return a JSON formatted array with the current list of profiles. These values are updated in real-time.[^1]

```json
{
  "profiles": [
    {
      "name": "gaming",
      "id": 1,
      "active": false,
      "created_at": "2024-05-08T10:47:09.000",
      "updated_at": "2024-05-10T09:40:22.000"
    }
  ]
}
```


***

### Profiles: Activate a Profile

After finding the correct profile id in the above GET request, you can activate a specific profile using this POST request.[^1]

#### HTTP Request

`POST http://localhost:3490/v1/profiles/activate`[^1]

With body:[^1]

```json
{ "profile_id": 1 }
```


#### Examples (PHP)

```php
$url = 'http://localhost:3490/v1/profiles/activate';
$options = [
  'http' => [
    'method'  => 'POST',
    'header'  => 'Content-Type: application/json',
    'content' => json_encode(['profile_id' => 1])
  ]
];
$context  = stream_context_create($options);
$response = file_get_contents($url, false, $context);
echo $response;
?>
```


#### Results

This call will return a JSON formatted array with the status of the request using a `msg` or `error` value.[^1]

```json
{ "msg": "Profile 'profileName' activated." }
```

or

```json
{ "error": "Profile id doesn't exist." }
```


***

## Using the WebSocket

[WebSockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API) are connections that stay open and generally have a web-based message structure. If you're looking for (near) real-time statistics from WhatPulse to your application, this is your best option. By default, the WebSocket server runs on **port 3489**, but you can change that to another port in the Client API settings.[^1]

Currently, the WebSocket supports a flow of stats, and actions to open the WhatPulse window, and pulsing. Messages are sent and received in JSON format.[^1]

Before you begin, make sure you're running **WhatPulse v5.6** or above.[^1]

### Identifying

Before WhatPulse will start sending statistics to a WebSocket client, it needs to identify itself. This is to prevent sending unwanted data. Once you've connected to the WebSocket server, this is the first message you should send:[^1]

```json
{
  "source": "plugin",
  "action": "identify"
}
```


### Receiving Stats

After you've identified, WhatPulse will start sending stats every 2 seconds. The incoming message will look like this:[^1]

```json
{
  "action": "update-status",
  "data": {
    "account-totals": {
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
        "rank_clicks_formatted": "2.605th",
        "rank_distance": "2021",
        "rank_distance_formatted": "2.021st",
        "rank_download": "2407",
        "rank_download_formatted": "2.407th",
        "rank_keys": "448",
        "rank_keys_formatted": "448th",
        "rank_scrolls": "790",
        "rank_scrolls_formatted": "790th",
        "rank_upload": "2887",
        "rank_upload_formatted": "2.887th",
        "rank_uptime": "1617",
        "rank_uptime_formatted": "1.617th"
      },
      "scrolls": "3256578",
      "scrolls_formatted": "3.256.578",
      "upload": "4935646",
      "upload_formatted": "4,71TB",
      "uptime": "195392798",
      "uptime_formatted": "6y, 125d, 11h, 46m"
    },
    "realtime": {
      "clicks": "0.00",
      "download": "368,44KB/s",
      "keys": "0.00",
      "upload": "4,07MB/s"
    },
    "unpulsed": {
      "clicks": 0,
      "clicks_formatted": "0",
      "distance_formatted": "None",
      "distance_inches": 0,
      "download": 128864445,
      "download_formatted": "122,89MB",
      "keys": 0,
      "keys_formatted": "0",
      "scrolls": 0,
      "scrolls_formatted": "0",
      "upload": 41165767,
      "upload_formatted": "39,26MB",
      "uptime": 4811,
      "uptime_formatted": "1h, 20m"
    }
  }
}
```


### Actions

You can ask WhatPulse to pulse and open the main window via the websocket by sending one of these messages:[^1]

```json
{ "source": "plugin", "action": "pulse" }
{ "source": "plugin", "action": "open-window" }
```

[^1]: https://whatpulse.org/help/api/client-api

