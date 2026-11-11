# LPPWeb

Batteries-included web server package for LPPO. Requires [LPPNet](https://github.com/AlexMatzen01/LPPNet).

## Features

- **Hosting**: TCP/UDP server primitives via LPPNet (bind, listen, accept, send, receive)
- **Routing**: Pattern-based route matching with parameter extraction (`/user/:id`)
- **Local Storage**: File-backed key-value storage (`put`, `get`, `delete`, `exists`, `list`)
- **Database**: File-backed SQL-like database with 15+ functions (`open`, `close`, `exec`, `query`, `insert`, `select`, `update`, `delete`, `count`, `create_table`, `drop_table`, `list_tables`, and more)
- **Encryption**: XOR symmetric cipher (`encrypt`, `decrypt`)
- **Hashing**: SHA-256 (cryptographic) and DJB2 (non-cryptographic)
- **Security Headers**: `security_headers()`, `cors()`, `hsts()`, `csp()`, `x_frame_options()`, `x_content_type_options()`, `header_set()`, `header_clear()`
- **Response Building**: `response_status_line()`, `response_build()`

## Installation

```bash
lppo pkg install lppnet
lppo pkg install LPPWeb
```

## Usage

```lppo
use LPPNet;
use LPPWeb;

// Hashing
let own buf: Int64 = buf_new();
buf_set(buf, "hello world");
let own sha: Int64 = web_sha256_buf(buf);
out "SHA-256:", buf_get(sha);

// Local storage
let own key: Int64 = buf_new();
buf_set(key, "user:1");
let own val: Int64 = buf_new();
buf_set(val, "{\"name\":\"Alice\"}");
web_storage_put(key, val);
let own stored: Int64 = web_storage_get(key);
out "stored:", buf_get(stored);

// Encryption (XOR)
let own key2: Int64 = buf_new();
buf_set(key2, "secret");
let own enc: Int64 = web_xor_encrypt(key2, buf);
out "encrypted:", buf_get(enc);

// Security headers
let own sec: Int64 = web_security_headers();
out "headers:", buf_get(sec);

// Routing
let own match: Bool = web_route_match(buf_new(), "/user/:id");
buf_set(match, "/user/42");
out "route_match:", web_route_match(buf_new(), "/user/42");
```

## Native Functions

All functions accept `Int64`/`Bool` arguments and return `Int64`/`Bool`. String data is exchanged through LPPO's shared buffer registry (`buf_new`, `buf_set`, `buf_get`).

| Function | Params | Return | Description |
|---|---|---|---|
| `web_sha256_buf` | `Int64` | `Int64` | SHA-256 hash of buffer, returns new buffer id |
| `web_djb2_buf` | `Int64` | `Int64` | DJB2 hash of buffer |
| `web_xor_encrypt` | `Int64, Int64` | `Int64` | XOR encrypt (key_buf, data_buf) |
| `web_xor_decrypt` | `Int64, Int64` | `Int64` | XOR decrypt (symmetric) |
| `web_storage_put` | `Int64, Int64` | `Bool` | Store key-value |
| `web_storage_get` | `Int64` | `Int64` | Get value (new buffer id) |
| `web_storage_delete` | `Int64` | `Bool` | Delete key |
| `web_storage_exists` | `Int64` | `Bool` | Check key exists |
| `web_storage_list` | — | `Int64` | List all keys (JSON array) |
| `web_db_open` | `Int64` | `Int64` | Open database (new handle id) |
| `web_db_close` | `Int64` | `Bool` | Close database |
| `web_db_exec` | `Int64, Int64` | `Bool` | Execute SQL |
| `web_db_query` | `Int64, Int64` | `Int64` | Query (new buffer id, JSON) |
| `web_db_insert` | `Int64, Int64, Int64` | `Bool` | Insert row |
| `web_db_select` | `Int64, Int64, Int64` | `Int64` | Select rows |
| `web_db_update` | `Int64, Int64, Int64, Int64` | `Bool` | Update rows |
| `web_db_delete` | `Int64, Int64, Int64` | `Bool` | Delete rows |
| `web_db_count` | `Int64, Int64` | `Int64` | Count rows |
| `web_db_create_table` | `Int64, Int64` | `Bool` | Create table |
| `web_db_drop_table` | `Int64, Int64` | `Bool` | Drop table |
| `web_db_list_tables` | `Int64` | `Int64` | List tables (JSON array) |
| `web_header_set` | `Int64, Int64` | `Bool` | Set response header |
| `web_header_clear` | — | `Bool` | Clear all headers |
| `web_security_headers` | — | `Int64` | Standard security headers |
| `web_cors` | `Int64` | `Int64` | CORS headers |
| `web_hsts` | — | `Int64` | HSTS header |
| `web_csp` | — | `Int64` | CSP header |
| `web_x_frame_options` | — | `Int64` | X-Frame-Options |
| `web_x_content_type_options` | — | `Int64` | X-Content-Type-Options |
| `web_route_match` | `Int64, Int64` | `Bool` | Match path against pattern |
| `web_route_extract` | `Int64, Int64` | `Int64` | Extract route params (JSON) |
| `web_response_status_line` | `Int64` | `Int64` | HTTP status line |
| `web_response_build` | `Int64, Int64, Int64` | `Int64` | Build HTTP response |
| `web_last_error` | — | `Int64` | Last error code |

## Architecture

LPPWeb shares string data with LPPO through a shared buffer registry exposed by the LPPO interpreter (`lppo_buf_*` symbols). Native packages resolve these symbols from the main executable at runtime, enabling lossless string exchange between LPPO and native code.

Requires LPPO >= 5.3.0 with buffer builtins (`buf_new`, `buf_set`, `buf_get`, `buf_append`, `buf_len`, `buf_clear`, `buf_byte`).

## License

MIT
