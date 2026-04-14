# Step 03: Create the TypeScript ACP Connection Manager

## Objective

Create a singleton module that manages the lifecycle of the `GooseClient` connection to `goose serve`. This is the TypeScript equivalent of the Rust `GooseAcpManager::start()` singleton.

## Why

All ACP operations (send prompt, list sessions, export, etc.) need a shared, initialized `GooseClient` instance. This module:

1. Fetches the `goose serve` URL from the Rust backend (Step 01's command)
2. Creates a `GooseClient` with `createHttpStream`
3. Calls `client.initialize()` to complete the ACP handshake
4. Provides the initialized client to all other modules

## New File

### `src/shared/api/acpConnection.ts`

```typescript
/**
 * Singleton ACP connection manager.
 *
 * Manages the lifecycle of the GooseClient connection to goose serve.
 * All ACP operations go through the client returned by getClient().
 */
import { invoke } from "@tauri-apps/api/core";
import { GooseClient, createHttpStream } from "@aaif/goose-acp";
import type { Client, SessionNotification, RequestPermissionRequest, RequestPermissionResponse } from "@agentclientprotocol/sdk";

// Will be set by Step 04 — the notification handler
let notificationHandler: AcpNotificationHandler | null = null;

/**
 * Interface for the notification handler that processes ACP session events.
 * Implemented in Step 04 (acpNotificationHandler.ts).
 */
export interface AcpNotificationHandler {
  handleSessionNotification(notification: SessionNotification): Promise<void>;
}

/**
 * Register the notification handler. Called once during app initialization
 * after the handler is created in Step 04.
 */
export function setNotificationHandler(handler: AcpNotificationHandler): void {
  notificationHandler = handler;
}

// Singleton state
let clientPromise: Promise<GooseClient> | null = null;
let resolvedClient: GooseClient | null = null;

/**
 * Build the Client implementation that the ACP SDK calls back into.
 *
 * This handles two callback types:
 * - request_permission: auto-approve with the first option (same as Rust impl)
 * - session_notification: delegate to the registered notification handler
 */
function createClientCallbacks(): () => Client {
  return () => ({
    requestPermission: async (
      args: RequestPermissionRequest,
    ): Promise<RequestPermissionResponse> => {
      // Auto-approve with the first available option, matching the Rust behavior.
      const optionId = args.options?.[0]?.optionId ?? "approve";
      return {
        outcome: {
          type: "selected",
          optionId,
        },
      };
    },

    sessionNotification: async (
      notification: SessionNotification,
    ): Promise<void> => {
      if (notificationHandler) {
        await notificationHandler.handleSessionNotification(notification);
      }
    },
  });
}

/**
 * Initialize the ACP connection.
 *
 * 1. Calls the Rust backend to get the goose serve URL
 * 2. Creates a GooseClient with HTTP+SSE transport
 * 3. Sends the ACP initialize handshake
 *
 * This is idempotent — calling it multiple times returns the same client.
 */
async function initializeConnection(): Promise<GooseClient> {
  // Get the goose serve URL from the Rust backend.
  // This blocks until the server is confirmed ready.
  const serverUrl: string = await invoke("get_goose_serve_url");

  const client = new GooseClient(
    createClientCallbacks(),
    createHttpStream(serverUrl),
  );

  // Perform the ACP initialize handshake
  await client.initialize({
    protocolVersion: "2025-03-26",
    capabilities: {},
    clientInfo: {
      name: "goose2",
      version: "0.1.0",
    },
  });

  return client;
}

/**
 * Get the initialized GooseClient singleton.
 *
 * The first call triggers initialization (fetching the URL, creating the
 * connection, running the ACP handshake). Subsequent calls return the
 * same client immediately.
 *
 * Throws if initialization fails (e.g., goose serve is not running).
 */
export async function getClient(): Promise<GooseClient> {
  if (resolvedClient) {
    return resolvedClient;
  }

  if (!clientPromise) {
    clientPromise = initializeConnection().then((client) => {
      resolvedClient = client;
      return client;
    }).catch((error) => {
      // Reset so the next call retries
      clientPromise = null;
      throw error;
    });
  }

  return clientPromise;
}

/**
 * Check if the client has been initialized.
 * Useful for guards that need to know if ACP is ready without triggering init.
 */
export function isClientReady(): boolean {
  return resolvedClient !== null;
}

/**
 * Get the client synchronously, or null if not yet initialized.
 * Use getClient() for the async version that triggers initialization.
 */
export function getClientSync(): GooseClient | null {
  return resolvedClient;
}
```

## Architecture Notes

### Singleton Pattern

The module uses a promise-based singleton pattern:
- `clientPromise` ensures only one initialization runs at a time
- `resolvedClient` caches the result for synchronous access
- If initialization fails, `clientPromise` is reset so the next call retries

This mirrors the Rust `OnceCell<Arc<GooseAcpManager>>` pattern in `manager.rs`.

### Notification Handler Registration

The notification handler is registered separately (in Step 04) rather than being passed to `createClientCallbacks` at construction time. This avoids a circular dependency:
- `acpConnection.ts` creates the client
- `acpNotificationHandler.ts` needs the client (to know about sessions)
- `acpNotificationHandler.ts` needs to be registered with the connection

The `setNotificationHandler()` function breaks this cycle.

### Protocol Version

The `protocolVersion` in the initialize request should match what `goose serve` expects. The Rust code uses `ProtocolVersion::LATEST` from the `agent-client-protocol` crate. Check the `@agentclientprotocol/sdk` package for the equivalent constant — it may be exported as `LATEST_PROTOCOL_VERSION` or similar. If not, use the string `"2025-03-26"` (or whatever the current latest is).

### Error Handling

If `invoke("get_goose_serve_url")` fails (e.g., goose binary not found), the error propagates to the caller. The app startup code (Step 08) should handle this gracefully — showing an error state rather than crashing.

## Verification

1. `pnpm typecheck` passes.
2. `pnpm check` passes (Biome lint).
3. The module can be imported without side effects — initialization only happens when `getClient()` is called.

## Files Created

| File | Purpose |
|------|---------|
| `src/shared/api/acpConnection.ts` | Singleton ACP connection manager |

## Dependencies

- Step 01 (the `get_goose_serve_url` Tauri command must exist)
- Step 02 (`@aaif/goose-acp` and `@agentclientprotocol/sdk` must be installed)

## Notes

- The `createHttpStream` transport from `@aaif/goose-acp` handles the HTTP+SSE protocol details: POSTing JSON-RPC to `/acp`, reading SSE responses, managing the `Acp-Session-Id` header.
- The `GooseClient` constructor takes `() => Client` (a factory function that returns the callback object) and a `Stream` (the transport). It wraps `ClientSideConnection` from the SDK.
- The `requestPermission` callback shape may differ slightly between SDK versions. Check the `@agentclientprotocol/sdk` types for the exact interface. The Rust code returns `RequestPermissionResponse::new(RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(option_id)))` — the TS equivalent should match.
- The `initialize` request params shape may also vary. Check `InitializeRequest` in the SDK. The Rust code sends `InitializeRequest::new(ProtocolVersion::LATEST).client_info(Implementation::new("goose2", env!("CARGO_PKG_VERSION")))`.
