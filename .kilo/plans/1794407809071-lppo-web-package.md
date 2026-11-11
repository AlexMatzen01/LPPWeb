# LPPWeb — LPPO Web Server Package Plan

**Plan file:** `.kilo/plans/1794407809071-lppo-web-package.md`
**Timestamp:** 1794407809071
**Status:** Open — one gating decision unresolved (see Open questions / Question below)

## Goal

Create `LPPWeb`, a batteries-included LPPO package for hosting web servers, requiring `LPPNet`. It must provide:
- HTTP hosting (listen/accept/respond) built on `LPPNet`
- Local storage + variable declaration (LPPO `let` + file-backed storage)
- Encryption methods
- Many database functions
- Routing + path handling
- Operators (LPPO's built-in operators)
- Security measures including response headers
- Hashing (cryptographic and non-cryptographic)

`LPPWeb` is an **LPPO package** = a Rust/Cargo `cdylib` with `package.json` + `lppo_interface.json`, following `lppo/packages.md`. It lives at `~/Documents/LPPWeb` and depends on `LPPNet` (`~/Documents/LPPNet`).

## Constraint that shapes the whole design (verified against lppo source)

LPPO native FFI (`lppo/src/pkg/native.rs`) is the only way an installed package exposes functions to LPPO:
- Signature: `extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64) -> i64`, max 8 args.
- Accepted arg types: `Int64`, `Bool`, `Str`. A `Str` is **hashing-only** into a lossy `i64` — the native side cannot read the actual string characters.
- No `Arr<T>` in FFI args; no `Str` payload.
- Native code **cannot call back into LPPO procedures** (no callback/closure mechanism).
- When a package is installed, `use LPPWeb;` registers only its native functions; the package's on-disk `.lppo` source is **not** loaded (`lppo/src/lib.rs:309-322`).

Consequence: **native package functions cannot receive HTTP request bodies, header values, paths, or route handlers as strings.** A full web framework (routing with LPPO logic, request/response bodies, headers) cannot be implemented purely as native `cdylib` functions without editing `lppo` (real string/buffer FFI + native→LPPO callbacks), which you asked me not to do without telling you first.

## Decisions (resolved)

| Decision | Resolution |
|---|---|
| Package kind | Rust/Cargo `cdylib` LPPO package (`package.json` + `lppo_interface.json` + `src/lib.rs`) |
| Package identifier | `LPPWeb`, version `1.0.0`, author `Alexander Matzen`, MIT |
| Location | `~/Documents/LPPWeb` (uses the existing `package.json` stub there) |
| Dependency | `package.json` `dependencies: { "lppnet": "1.0.0" }` — requires LPPNet |
| Build | `cargo`, `crate-type = ["cdylib"]`, release |
| Native calling convention | `extern "C" fn(i64 x8) -> i64`; `Int64`/`Bool`/`Str(hashed)` only; max 8 args |
| String data across FFI | Not passed as payload; exchange via files or kept inside LPPO (see open question) |
| Database backend | File-based (Rust `std::fs` inside native code; exposed as Int64/Bool ops). No new Rust deps. |
| New LPPO operators | None — LPPO operators are language builtins; adding operators requires editing `lppo` compiler. LPPWeb uses LPPO's existing operators (`+ - * / % == != < > <= >= and or not` bitwise) in its LPPO-level logic. |
| Edit `lppo`? | No — not without your explicit approval (you asked to be told first). A full native-only web framework needs `lppo` edits; see open question. |

## LPPNet updates needed (if we build hosting on LPPNet)

`LPPNet` currently sends/receives **one byte per call** (`net_tcp_send`/`net_tcp_receive` take a single byte, return i64). Byte-by-byte HTTP through LPPO is possible but impractical.
- Add buffer-oriented primitives to `LPPNet` (e.g. send/receive a length-prefixed chunk, or a `net_tcp_receive_into_file(fd, path)` / `net_tcp_send_from_file(fd, path)` so string data travels via files, not FFI args).
- Keep `LPPNet` Int64/Bool-only per FFI convention; file paths can travel as... **also hashed**, so file-path exchange across FFI has the same string problem — buffer ops must be driven entirely from LPPO or via LPPNet-internal state keyed by fd.

This is why the architecture question below matters.

## Affected files

New (`~/Documents/LPPWeb/`):
1. `Cargo.toml` — `lppweb` cdylib, depends on `lppnet` (path dep) + optional crypto crate
2. `package.json` — edit existing stub: add `lppnet` dependency, real description
3. `lppo_interface.json` — declare all native functions (hash, crypto, storage, db, headers, security, routing primitives)
4. `src/lib.rs` — native function implementations (`#[no_mangle] extern "C"`)
5. `README.md` — package docs + LPPO usage example
6. `examples/` — example LPPO program(s) using `use LPPWeb;`

Edit (`~/Documents/LPPNet/`):
7. `src/lib.rs` — add buffer send/receive primitives (only if approach requires it)
8. `lppo_interface.json` — register the new `LPPNet` functions

Forbidden without your approval:
- `~/Documents/lppo/` — do not edit. Real string FFI / callbacks would require changes here.

## Native function signatures (LPPWeb `lppo_interface.json`)

All `Int64`/`Bool` in/out (strings are hashed on entry, so avoid string args):

Hashing:
- `web_hash_djb2(input: Str) -> Int64` — non-crypto hash (input is hashed twice; lossy — see constraint)
- `web_hash_sha256_hex(input: Str) -> Int64` — placeholder; real crypto hashing of arbitrary strings cannot receive the string via FFI. Operate on file-based input instead: `web_hash_file(path_hashed...)`. **Needs resolution.**

Encryption:
- `web_encrypt(...) -> Int64` / `web_decrypt(...) -> Int64` — symmetric crypto; same string-input limitation.

Storage / database (file-backed, no string args — operate on fd/id or file paths handled in LPPO):
- `web_storage_put(id: Int64, ...) -> Bool`, `web_storage_get(...) -> Int64`, `web_storage_delete(...) -> Bool`
- `web_db_open(...) -> Int64`, `web_db_put(...) -> Bool`, `web_db_get(...) -> Int64`, `web_db_delete(...) -> Bool`, `web_db_query(...) -> Int64`, ... (many DB functions, file-backed key/value + table files)

Security / headers:
- `web_header_set(...) -> Bool`, `web_security_headers(...) -> Bool` (HSTS, CSP, X-Frame-Options, etc.)
- `web_cors(...) -> Bool`

Hosting / routing primitives:
- `web_server_start(port: Int64) -> Int64` (server id)
- `web_server_stop(server_id: Int64) -> Bool`
- `web_route_register(...)` — routing table; **handler invocation cannot be native-to-LPPO** (see constraint)
- `web_response_send(...) -> Bool`

**Note:** Several signatures above are not implementable as-is under the FFI constraint (they need real string payloads or callbacks). Final signatures depend on the architecture decision below.

## Data flow (hybrid approach, within current lppo)

LPPO program:
1. `use LPPNet; use LPPWeb;`
2. `net_tcp_bind(port)` → listen fd (LPPNet)
3. Loop: `net_tcp_accept(listen_fd)` → client fd; receive request bytes (LPPNet) into LPPO string via `str_concat` byte-by-byte (or via LPPNet buffer primitive if added)
4. Parse request in LPPO (`str_split`, `str_trim`, `str_contains`-style logic built from LPPO string ops)
5. Route match in LPPO; call the matching LPPO handler procedure
6. Handler builds response string in LPPO; `write_file` to a temp file; LPPWeb reads the file and `net_tcp_send` bytes over client fd (LPPNet)
7. `net_tcp_close(client_fd)`

Native (LPPWeb) handles only: hashing of file contents, crypto on file-backed data, header/security constants, DB file ops — anything not requiring a live string argument.

## Failure modes & handling

- Native function receives a `Str` arg → only its hash is available; any logic expecting the actual characters silently fails/uses wrong data.
- `net_tcp_receive` returns -1 on EOF/timeout → LPPO must treat as end of request.
- Port already bound / socket errors → `net_last_error()` (LPPNet) or LPPWeb error fd/bool.
- Missing `lppnet` installed → `use LPPWeb;` type-checks but calls fail at runtime ("native function not registered").
- `lppo` edits required for full feature → plan stalls until you approve.

## Validation

- `cd ~/Documents/LPPWeb && cargo build` — package compiles as cdylib
- `cd ~/Documents/LPPNet && cargo build` — LPPNet compiles (with any added primitives)
- Install both via LPPO package manager (`lppo pkg install`) into a test project
- Write an LPPO program `use LPPNet; use LPPWeb;` that starts a server, responds to a request, sets a header, hashes a file, and runs a DB op; verify it runs under the LPPO interpreter
- `cargo test` in both workspaces; `cargo fmt --check` if available

## Open questions

1. **(Gating)** Architecture approach given the FFI string constraint:
   - **(A) Recommended:** Hybrid — routing/request/response logic in LPPO source (calling LPPNet primitives byte-by-byte or via new buffer primitives); LPPWeb native cdylib limited to crypto/hash/storage/security that operate on file-backed data or Int64 ids. No `lppo` edits.
   - **(B):** Authorize editing `lppo` to add real string/buffer FFI and native→LPPO callbacks, enabling a full native web framework. You asked to be told before any `lppo` edit — this is that request.

Which do you want? My recommendation: **(A)** — proceed within current `lppo` constraints, no `lppo` edits.

2. Database backend: file-based key/value (+ optional `rusqlite` SQLite)? Recommended: file-based, no new deps.
3. Should `LPPNet` gain buffer send/receive primitives, or stay 1-byte and let LPPO loop? Recommended: add buffer primitives to make HTTP practical.

## Implementation-ready?

No — blocked on open question #1 (and #2, #3). Once you answer, I'll finalize the plan and call `plan_exit` so an implementation agent can proceed. I will **not** edit `lppo` unless you explicitly approve approach (B).
