# Step 07: Rewire `src/shared/api/acp.ts` to Use the TypeScript ACP Client

## Objective

Replace all `invoke()` calls in `src/shared/api/acp.ts` with calls to the new TypeScript ACP session manager (Step 05) and search module (Step 06). Keep the same public API signatures so consumers don't need to change.

## Why

`src/shared/api/acp.ts` is the single import point for all ACP operations in the frontend. Currently every function calls `invoke("acp_*")` which goes through Tauri IPC → Rust → WebSocket → goose serve. After this step, they call the TypeScript session manager which goes directly through HTTP+SSE → goose serve.

## Changes

### `src/shared/api/acp.ts`

Replace the entire file contents. The public API (function names, parameter types, return types) stays the same. Only the implementation changes.

**Before (current):**
```typescript
import { invoke } from "@tauri-apps/api/core";

export async function discoverAcpProviders(): Promise<AcpProvider[]> {
  return invoke("discover_acp_providers");
}

export async function acpSendMessage(sessionId, providerId, prompt, options): Promise<void> {
  return invoke("acp_send_message", { ... });
}
// ... etc
```

**After:**
```typescript
import {
  listProviders,
  sendPrompt,
  prepareSession,
  setModel,
  cancelSession,
  listSessions,
  loadSession,
  exportSession,
  importSession,
  forkSession,
  listRunning,
  cancelAll,
} from "./acpSessionManager";
import { searchSessionsViaExports } from "@/features/sessions/lib/sessionContentSearch";

// Re-export types (unchanged)
export type { AcpProvider, AcpSessionInfo, AcpRunningSession } from "./acpSessionManager";
export type { SessionSearchResult as AcpSessionSearchResult } from "@/features/sessions/lib/sessionContentSearch";
```

### Function-by-function rewiring

#### `discoverAcpProviders`

```typescript
export async function discoverAcpProviders(): Promise<AcpProvider[]> {
  return listProviders();
}
```

#### `acpSendMessage`

```typescript
export async function acpSendMessage(
  sessionId: string,
  providerId: string,
  prompt: string,
  options: AcpSendMessageOptions = {},
): Promise<void> {
  return sendPrompt(sessionId, providerId, prompt, {
    workingDir: options.workingDir,
    systemPrompt: options.systemPrompt,
    personaId: options.personaId,
    personaName: options.personaName,
    images: options.images,
  });
}
```

#### `acpPrepareSession`

```typescript
export async function acpPrepareSession(
  sessionId: string,
  providerId: string,
  options: AcpPrepareSessionOptions = {},
): Promise<void> {
  const { makeCompositeKey } = await import("./acpSessionManager");
  const compositeKey = makeCompositeKey(sessionId, options.personaId);
  const workingDir = options.workingDir ?? "~/.goose/artifacts";
  await prepareSession(compositeKey, sessionId, providerId, workingDir);
}
```

#### `acpSetModel`

```typescript
export async function acpSetModel(
  sessionId: string,
  modelId: string,
): Promise<void> {
  return setModel(sessionId, modelId);
}
```

#### `acpListSessions`

```typescript
export async function acpListSessions(): Promise<AcpSessionInfo[]> {
  return listSessions();
}
```

#### `acpSearchSessions`

```typescript
export async function acpSearchSessions(
  query: string,
  sessionIds: string[],
): Promise<AcpSessionSearchResult[]> {
  return searchSessionsViaExports(query, sessionIds);
}
```

#### `acpLoadSession`

```typescript
export async function acpLoadSession(
  sessionId: string,
  gooseSessionId: string,
  workingDir?: string,
): Promise<void> {
  return loadSession(sessionId, gooseSessionId, workingDir ?? "~/.goose/artifacts");
}
```

#### `acpExportSession`

```typescript
export async function acpExportSession(sessionId: string): Promise<string> {
  return exportSession(sessionId);
}
```

#### `acpImportSession`

```typescript
export async function acpImportSession(json: string): Promise<AcpSessionInfo> {
  return importSession(json);
}
```

#### `acpDuplicateSession`

```typescript
export async function acpDuplicateSession(sessionId: string): Promise<AcpSessionInfo> {
  return forkSession(sessionId);
}
```

#### `acpCancelSession`

```typescript
export async function acpCancelSession(
  sessionId: string,
  personaId?: string,
): Promise<boolean> {
  return cancelSession(sessionId, personaId);
}
```

### Keep the interface types

The `AcpSendMessageOptions`, `AcpPrepareSessionOptions`, `AcpSessionInfo`, `AcpSessionSearchResult`, `AcpProvider` interfaces should remain exported from this file (or re-exported from the session manager). Consumers import them from `@/shared/api/acp`.

If the types are now defined in `acpSessionManager.ts` or `sessionContentSearch.ts`, re-export them:

```typescript
export type { AcpProvider, AcpSessionInfo } from "./acpSessionManager";
export type { SessionSearchResult as AcpSessionSearchResult } from "@/features/sessions/lib/sessionContentSearch";

// Keep these interfaces here since they're specific to this API surface
export interface AcpSendMessageOptions {
  systemPrompt?: string;
  workingDir?: string;
  personaId?: string;
  personaName?: string;
  images?: [string, string][];
}

export interface AcpPrepareSessionOptions {
  workingDir?: string;
  personaId?: string;
}
```

### Update `src/shared/api/index.ts`

No changes needed — it already re-exports from `./acp`:

```typescript
export * from "./acp";
```

## Consumers That Import from `@/shared/api/acp`

Verify these files still work without changes (they should, since the public API is unchanged):

| File | Imports Used |
|------|-------------|
| `src/features/chat/hooks/useChat.ts` | `acpSendMessage`, `acpCancelSession`, `acpPrepareSession`, `acpSetModel` |
| `src/features/chat/stores/chatSessionStore.ts` | `acpListSessions`, `AcpSessionInfo` |
| `src/features/sessions/hooks/useSessionSearch.ts` | `acpSearchSessions` |
| `src/features/sessions/lib/buildSessionSearchResults.ts` | `AcpSessionSearchResult` |
| `src/app/AppShell.tsx` | `acpPrepareSession`, `acpLoadSession` |
| `src/app/hooks/useAppStartup.ts` | `discoverAcpProviders` |

## Remove `invoke` Import

The file should no longer import from `@tauri-apps/api/core` since all `invoke()` calls are replaced.

## Verification

1. `pnpm typecheck` passes — all consumers still type-check against the same API.
2. `pnpm check` passes.
3. `pnpm test` passes — existing tests that mock `invoke()` may need updating. Check `src/features/chat/hooks/__tests__/useAcpStream.test.ts` and `src/features/chat/hooks/__tests__/useChat.test.ts`.
4. Manual testing: start the app, verify sessions load, messages send, search works.

## Files Modified

| File | Change |
|------|--------|
| `src/shared/api/acp.ts` | Replace all `invoke()` calls with session manager / search calls |

## Dependencies

- Step 05 (`acpSessionManager.ts` — all session operations)
- Step 06 (`sessionContentSearch.ts` — search)

## Notes

- This is the "flip the switch" step. After this, the frontend no longer calls any `acp_*` Tauri commands (except `get_goose_serve_url` which is called by `acpConnection.ts`).
- The old Rust ACP commands still exist and are registered but are no longer called. They'll be removed in Step 09.
- If you want a gradual rollout, you could add a feature flag that switches between the old `invoke()` path and the new direct path. But since the public API is identical, it's simpler to just swap.
- The `@tauri-apps/api/core` import can be removed from this file entirely. Other files (agents, git, system, etc.) still use `invoke()` for non-ACP commands.
