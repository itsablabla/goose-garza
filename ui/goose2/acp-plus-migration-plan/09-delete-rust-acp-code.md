# Step 09: Delete the Rust ACP Middleware and Unused Dependencies

## Objective

Remove all Rust ACP protocol handling code that is no longer called by the frontend. This is the cleanup step — only do this after Steps 01–08 are working and tested.

## Why

After Steps 01–08, the frontend communicates directly with `goose serve` via HTTP+SSE. The Rust ACP middleware (WebSocket bridge, session dispatcher, message writer, session registry, search) is dead code. Removing it:

- Eliminates ~3,500 lines of Rust
- Removes 5–6 heavy crate dependencies
- Reduces compile times
- Simplifies the codebase

## Changes

### 1. Delete the ACP manager subtree

**Delete these files entirely:**

```
src-tauri/src/services/acp/manager/
  command_dispatch.rs
  dispatcher.rs
  dispatcher_tests.rs
  session_ops.rs
  session_ops/
    prompt_ops.rs
    tests.rs
  thread.rs
```

**Delete these files:**

```
src-tauri/src/services/acp/manager.rs
src-tauri/src/services/acp/writer.rs
src-tauri/src/services/acp/payloads.rs
src-tauri/src/services/acp/registry.rs
src-tauri/src/services/acp/search.rs
```

### 2. Simplify `services/acp/mod.rs`

**File:** `src-tauri/src/services/acp/mod.rs`

Replace the entire file with:

```rust
pub(crate) mod goose_serve;

pub(crate) use goose_serve::GooseServeProcess;
```

All the old re-exports (`GooseAcpManager`, `AcpSessionRegistry`, `TauriMessageWriter`, `search_sessions_via_exports`, `make_composite_key`, `split_composite_key`, `AcpService`, `AcpRunningSession`, `AcpSessionInfo`, `SessionSearchResult`) are removed.

### 3. Simplify `commands/acp.rs`

**File:** `src-tauri/src/commands/acp.rs`

Replace the entire file with just the URL command:

```rust
use crate::services::acp::GooseServeProcess;

/// Return the HTTP base URL of the running goose serve process.
///
/// This command blocks until the server is confirmed ready. The frontend
/// uses this URL to establish a direct HTTP+SSE ACP connection.
#[tauri::command]
pub async fn get_goose_serve_url() -> Result<String, String> {
    GooseServeProcess::start().await?;
    let process = GooseServeProcess::get()?;
    Ok(process.http_url())
}
```

All other ACP commands are deleted:
- `discover_acp_providers`
- `acp_prepare_session`
- `acp_set_model`
- `acp_send_message`
- `acp_cancel_session`
- `acp_list_sessions`
- `acp_search_sessions`
- `acp_load_session`
- `acp_list_running`
- `acp_cancel_all`
- `acp_export_session`
- `acp_import_session`
- `acp_duplicate_session`

Also delete the helper functions that were only used by those commands:
- `AcpProviderResponse` struct
- `should_include_provider`
- `default_artifacts_working_dir`
- `expand_home_dir`
- `resolve_working_dir`
- The `#[cfg(test)] mod tests` block (the tests tested those helpers)

### 4. Update `lib.rs`

**File:** `src-tauri/src/lib.rs`

Remove the `AcpSessionRegistry` from managed state and remove all old ACP command registrations.

**Before:**
```rust
use std::sync::Arc;
use services::acp::AcpSessionRegistry;

// ...
let acp_registry = Arc::new(AcpSessionRegistry::new());
let acp_registry_for_exit = Arc::clone(&acp_registry);

let builder = tauri::Builder::default()
    // ...
    .manage(acp_registry);

// In invoke_handler:
commands::acp::discover_acp_providers,
commands::acp::acp_prepare_session,
commands::acp::acp_set_model,
commands::acp::acp_send_message,
commands::acp::acp_cancel_session,
commands::acp::acp_list_sessions,
commands::acp::acp_search_sessions,
commands::acp::acp_load_session,
commands::acp::acp_list_running,
commands::acp::acp_cancel_all,
commands::acp::acp_export_session,
commands::acp::acp_import_session,
commands::acp::acp_duplicate_session,

// In run closure:
.run(move |_app, event| {
    if let tauri::RunEvent::Exit = event {
        acp_registry_for_exit.cancel_all();
    }
});
```

**After:**
```rust
// Remove: use std::sync::Arc;
// Remove: use services::acp::AcpSessionRegistry;

// Remove: let acp_registry = ...
// Remove: let acp_registry_for_exit = ...

let builder = tauri::Builder::default()
    // ...
    // Remove: .manage(acp_registry)
    ;

// In invoke_handler, replace all old ACP commands with just:
commands::acp::get_goose_serve_url,

// Simplify the run closure:
.run(|_app, _event| {});
```

The `Arc` import can be removed if nothing else uses it. Check if `PersonaStore` or `GooseConfig` need it — they don't (they use `tauri::State` which handles the wrapping).

### 5. Clean up `goose_serve.rs`

**File:** `src-tauri/src/services/acp/goose_serve.rs`

The file is mostly kept as-is. Make these changes:

1. Add the `http_url()` method (from Step 01):
```rust
impl GooseServeProcess {
    pub fn http_url(&self) -> String {
        format!("http://{LOCALHOST}:{}", self.port)
    }
}
```

2. Remove the `WS_BRIDGE_BUFFER_BYTES` constant — it was only used by the WebSocket bridge in `thread.rs`:
```rust
// DELETE:
pub(crate) const WS_BRIDGE_BUFFER_BYTES: usize = 64 * 1024;
```

3. The `resolve_goose_binary` function is still needed by `model_setup.rs` (which runs `goose configure`). Keep it exported as `pub(crate)`.

### 6. Remove unused Cargo dependencies

**File:** `src-tauri/Cargo.toml`

Remove these dependencies that were only used by the deleted ACP code:

```toml
# DELETE these lines:
agent-client-protocol = { version = "0.10.4", features = ["unstable_session_fork"] }
acp-client = { git = "https://github.com/block/builderbot", rev = "db184d20cb48e0c90bbd3fea4a4a871fc9d8a6ad" }
tokio-tungstenite = "0.21.0"
async-trait = "0.1"
```

**Check before removing — these may still be used by remaining code:**

- `futures = "0.3"` — check if any remaining code uses it. `goose_serve.rs` uses `futures::{SinkExt, StreamExt}` for the WebSocket readiness probe. After removing the WS probe... wait, the readiness probe in `wait_for_server_ready` uses `tokio-tungstenite` and `futures`. We need to replace the readiness check.

**Replace the WebSocket readiness probe with an HTTP health check:**

The `wait_for_server_ready` function currently connects via WebSocket to check if the server is up. Replace it with a simple HTTP GET to `/health`:

```rust
async fn wait_for_server_ready(base_url: &str, child: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + GOOSE_SERVE_CONNECT_TIMEOUT;
    let health_url = format!("{base_url}/health");

    loop {
        match reqwest::get(&health_url).await {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(_) | Err(_) => {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|e| format!("Failed to poll goose serve process: {e}"))?
                {
                    return Err(format!(
                        "Goose serve exited before becoming ready: {status}"
                    ));
                }

                if Instant::now() >= deadline {
                    return Err(format!(
                        "Timed out waiting for goose serve at {health_url}"
                    ));
                }

                tokio::time::sleep(GOOSE_SERVE_CONNECT_RETRY_DELAY).await;
            }
        }
    }
}
```

Wait — adding `reqwest` is a new dependency. A simpler approach: use `tokio::net::TcpStream::connect` to check if the port is open:

```rust
async fn wait_for_server_ready(port: u16, child: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + GOOSE_SERVE_CONNECT_TIMEOUT;
    let addr = format!("{LOCALHOST}:{port}");

    loop {
        match tokio::net::TcpStream::connect(&addr).await {
            Ok(_) => return Ok(()),
            Err(_) => {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|e| format!("Failed to poll goose serve process: {e}"))?
                {
                    return Err(format!(
                        "Goose serve exited before becoming ready: {status}"
                    ));
                }

                if Instant::now() >= deadline {
                    return Err(format!(
                        "Timed out waiting for goose serve on port {port}"
                    ));
                }

                tokio::time::sleep(GOOSE_SERVE_CONNECT_RETRY_DELAY).await;
            }
        }
    }
}
```

This only needs `tokio` (already a dependency). Update the `spawn` method to call `wait_for_server_ready(port, &mut child)` instead of `wait_for_server_ready(&ws_url, &mut child)`.

With this change, `futures`, `tokio-tungstenite` can be removed.

- `tokio-util = { version = "0.7", features = ["compat", "rt"] }` — check if anything else uses it. It was used by `thread.rs` for `TokioAsyncReadCompatExt`/`TokioAsyncWriteCompatExt`. If nothing else uses it, remove it.

After all removals, the remaining dependencies should be:

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-app-test-driver = { path = "plugins/app-test-driver" }
tauri-plugin-opener = "2"
tauri-plugin-dialog = ">=2,<2.7"
tauri-plugin-window-state = "2"
tauri-plugin-log = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "6.0.0"
log = "0.4.29"
tokio = { version = "1.50.0", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde_yaml = "0.9"
etcetera = "0.8"
ignore = "0.4.25"
doctor = { git = "https://github.com/block/builderbot", rev = "8e1c3ec145edc0df5f04b4427cfd758378036862" }
keyring = { ... } # platform-specific
```

Use `cargo remove` or edit `Cargo.toml` directly, then run `cargo check` to verify nothing breaks.

### 7. Run full verification

```bash
cd ui/goose2/src-tauri

# Format
cargo fmt

# Check compilation
cargo check

# Clippy
cargo clippy --all-targets -- -D warnings

# Run any remaining Rust tests
cargo test
```

Then from the `ui/goose2` directory:

```bash
source ./bin/activate-hermit
just check
just test
just tauri-check
```

## Summary of Deletions

| Path | Lines | Purpose (was) |
|------|-------|---------------|
| `services/acp/manager/command_dispatch.rs` | ~258 | Command dispatch loop |
| `services/acp/manager/dispatcher.rs` | ~532 | Session event dispatcher + Client trait impl |
| `services/acp/manager/dispatcher_tests.rs` | ~28 | Dispatcher tests |
| `services/acp/manager/session_ops.rs` | ~611 | Session prepare/load/cancel/set-model |
| `services/acp/manager/session_ops/prompt_ops.rs` | ~(inline) | Send prompt logic |
| `services/acp/manager/session_ops/tests.rs` | ~(inline) | Session ops tests |
| `services/acp/manager/thread.rs` | ~169 | Manager thread + WebSocket bridge |
| `services/acp/manager.rs` | ~308 | GooseAcpManager struct + ManagerCommand enum |
| `services/acp/writer.rs` | ~156 | TauriMessageWriter |
| `services/acp/payloads.rs` | ~106 | Tauri event payload structs |
| `services/acp/registry.rs` | ~114 | AcpSessionRegistry |
| `services/acp/search.rs` | ~467 | Session content search |
| **Total** | **~2,749** | |

Plus significant simplification of `commands/acp.rs` (~330 → ~15 lines) and `services/acp/mod.rs` (~147 → ~4 lines) and `lib.rs` (~114 → ~80 lines).

## Cargo Dependencies Removed

| Crate | Why it was needed |
|-------|-------------------|
| `agent-client-protocol` | Rust ACP client types (Agent, ClientSideConnection, etc.) |
| `acp-client` | Agent discovery, MessageWriter trait |
| `tokio-tungstenite` | WebSocket connection to goose serve |
| `async-trait` | MessageWriter + Client trait impls |
| `futures` | WebSocket stream splitting (SinkExt, StreamExt) |
| `tokio-util` | Compat adapters for async read/write |

## Files Modified

| File | Change |
|------|--------|
| `src-tauri/src/services/acp/mod.rs` | Simplified to just goose_serve re-export |
| `src-tauri/src/services/acp/goose_serve.rs` | Add `http_url()`, remove WS constants, replace readiness probe |
| `src-tauri/src/commands/acp.rs` | Replaced with single `get_goose_serve_url` command |
| `src-tauri/src/lib.rs` | Remove AcpSessionRegistry, old ACP commands, simplify run closure |
| `src-tauri/Cargo.toml` | Remove 6 dependencies |
| `src-tauri/Cargo.lock` | Auto-updated |

## Files Deleted

All files listed in the "Summary of Deletions" table above.

## Dependencies

- Steps 01–08 must be working and tested before this cleanup step.

## Notes

- Run `cargo check` after each deletion batch to catch any remaining references.
- The `doctor` crate dependency stays — it's used by `commands/doctor.rs` which is not part of this migration.
- The `acp-client` crate was also used by `goose_serve.rs` for `acp_client::find_acp_agent_by_id("goose")` in binary resolution. Check if this import still exists. If so, you need an alternative way to find the goose binary. Options:
  - Keep `acp-client` just for binary discovery (lightweight — only uses the `find_acp_agent_by_id` function)
  - Inline the binary discovery logic (look for `goose` on PATH, check `GOOSE_BIN` env var)
  - The `GOOSE_BIN` env var path is already handled. The `acp_client::find_acp_agent_by_id` fallback scans login shell PATH. You could replace it with a simple `which goose` equivalent.
  
  If keeping `acp-client` just for binary discovery is acceptable, that's the simplest path. Otherwise, port the discovery logic.
