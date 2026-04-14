# Step 06: Port Session Content Search to TypeScript

## Objective

Port the session content search logic from `src-tauri/src/services/acp/search.rs` to TypeScript. This is pure text processing on exported JSON — no native access needed.

## Why

The Rust search code:
1. Exports each session as JSON via the ACP `goose/session/export` extension method
2. Parses the JSON to extract user/assistant/system messages
3. Performs case-insensitive substring matching
4. Builds snippets around the first match

All of this is string processing that runs fine in JavaScript. Moving it to TypeScript eliminates the Rust→TS round-trip for each session export during search.

## New File

### `src/features/sessions/lib/sessionContentSearch.ts`

```typescript
/**
 * Search session message content via exported Goose sessions.
 *
 * Ported from src-tauri/src/services/acp/search.rs
 */
import { exportSession } from "@/shared/api/acpSessionManager"; // from Step 05

const SNIPPET_PREFIX_BYTES = 40;
const SNIPPET_SUFFIX_BYTES = 60;

export interface SessionSearchResult {
  sessionId: string;
  snippet: string;
  messageId: string;
  messageRole?: "user" | "assistant" | "system";
  matchCount: number;
}
```

## Functions to Port

### 1. `searchSessionsViaExports`

The top-level function that iterates over session IDs, exports each, and searches:

```typescript
export async function searchSessionsViaExports(
  query: string,
  sessionIds: string[],
): Promise<SessionSearchResult[]> {
  const trimmed = query.trim();
  if (!trimmed) return [];

  const seen = new Set<string>();
  const results: SessionSearchResult[] = [];

  for (const sessionId of sessionIds) {
    if (seen.has(sessionId)) continue;
    seen.add(sessionId);

    try {
      const exported = await exportSession(sessionId);
      const result = searchExportedSession(sessionId, exported, trimmed);
      if (result) results.push(result);
    } catch {
      // Skip sessions that fail to export
    }
  }

  return results;
}
```

### 2. `searchExportedSession`

```typescript
function searchExportedSession(
  sessionId: string,
  exportedJson: string,
  query: string,
): SessionSearchResult | null {
  let root: unknown;
  try {
    root = JSON.parse(exportedJson);
  } catch {
    return null;
  }

  const obj = root as Record<string, unknown>;
  const conversation = obj.conversation ?? obj.messages;
  if (!conversation) return null;

  const messages = extractMessages(conversation);
  if (messages.length === 0) return null;

  let firstMatch: { messageId: string; role: string | null; snippet: string } | null = null;
  let matchCount = 0;

  for (const message of messages) {
    for (const text of message.searchableTexts) {
      const occurrences = countOccurrences(text, query);
      if (occurrences === 0) continue;

      matchCount += occurrences;

      if (!firstMatch) {
        firstMatch = {
          messageId: message.id,
          role: message.role,
          snippet: buildSnippet(text, query),
        };
      }
    }
  }

  if (!firstMatch) return null;

  return {
    sessionId,
    snippet: firstMatch.snippet,
    messageId: firstMatch.messageId,
    messageRole: firstMatch.role as SessionSearchResult["messageRole"],
    matchCount,
  };
}
```

### 3. `extractMessages`

Recursively walks the JSON structure to find message objects:

```typescript
interface ExportedMessage {
  id: string;
  role: string | null;
  searchableTexts: string[];
}

function extractMessages(value: unknown): ExportedMessage[] {
  const messages: ExportedMessage[] = [];
  collectMessages(value, messages);
  return messages;
}

function collectMessages(value: unknown, messages: ExportedMessage[]): void {
  if (Array.isArray(value)) {
    for (const item of value) {
      collectMessages(item, messages);
    }
    return;
  }

  if (typeof value !== "object" || value === null) return;
  const obj = value as Record<string, unknown>;

  if (obj.message !== undefined) {
    collectMessages(obj.message, messages);
    return;
  }

  if (obj.messages !== undefined) {
    collectMessages(obj.messages, messages);
    return;
  }

  if (looksLikeMessage(obj)) {
    const fallbackId = `message-${messages.length}`;
    const message = extractMessage(obj, fallbackId);
    if (message) messages.push(message);
  }
}

function looksLikeMessage(obj: Record<string, unknown>): boolean {
  return "role" in obj && ("content" in obj || "text" in obj);
}
```

### 4. `extractMessage`

```typescript
function extractMessage(
  obj: Record<string, unknown>,
  fallbackId: string,
): ExportedMessage | null {
  const role = normalizeRole(obj.role as string | undefined);
  const searchableTexts: string[] = [];

  if (obj.content !== undefined) {
    searchableTexts.push(...extractSearchableTexts(obj.content, role));
  } else if (typeof obj.text === "string") {
    if (role && obj.text.trim().length > 0) {
      searchableTexts.push(obj.text.trim());
    }
  }

  if (searchableTexts.length === 0) return null;

  return {
    id: typeof obj.id === "string" ? obj.id : fallbackId,
    role,
    searchableTexts,
  };
}
```

### 5. `extractSearchableTexts`

```typescript
function extractSearchableTexts(value: unknown, role: string | null): string[] {
  if (typeof value === "string") {
    if (role && isSearchableRole(role)) {
      const trimmed = value.trim();
      return trimmed.length > 0 ? [trimmed] : [];
    }
    return [];
  }

  if (Array.isArray(value)) {
    return value.flatMap((item) => extractSearchableBlockText(item, role));
  }

  if (typeof value === "object" && value !== null) {
    return extractSearchableBlockText(value, role);
  }

  return [];
}

function extractSearchableBlockText(value: unknown, role: string | null): string[] {
  if (typeof value !== "object" || value === null) return [];
  const obj = value as Record<string, unknown>;

  const blockType = obj.type as string | undefined;
  const text = obj.text as string | undefined;

  switch (blockType) {
    case "text":
    case "input_text":
    case "output_text":
    case "systemNotification":
    case "system_notification": {
      const trimmed = text?.trim();
      return trimmed && trimmed.length > 0 ? [trimmed] : [];
    }
    case "toolRequest":
    case "toolResponse":
    case "thinking":
    case "redactedThinking":
    case "reasoning":
    case "image":
      return [];
    default: {
      if (role && isSearchableRole(role)) {
        const trimmed = text?.trim();
        return trimmed && trimmed.length > 0 ? [trimmed] : [];
      }
      return [];
    }
  }
}
```

### 6. Helper functions

```typescript
function normalizeRole(role: string | undefined): string | null {
  if (!role) return null;
  const trimmed = role.trim().toLowerCase();
  if (trimmed === "user") return "user";
  if (trimmed === "assistant") return "assistant";
  if (trimmed === "system") return "system";
  return null;
}

function isSearchableRole(role: string): boolean {
  return role === "user" || role === "assistant" || role === "system";
}

function countOccurrences(text: string, query: string): number {
  const haystack = text.toLowerCase();
  const needle = query.toLowerCase();
  if (needle.length === 0) return 0;

  let count = 0;
  let searchStart = 0;

  while (true) {
    const index = haystack.indexOf(needle, searchStart);
    if (index === -1) break;
    count += 1;
    searchStart = index + needle.length;
  }

  return count;
}

function buildSnippet(text: string, query: string): string {
  const haystack = text.toLowerCase();
  const needle = query.toLowerCase();
  const matchIndex = haystack.indexOf(needle);
  const effectiveMatchIndex = matchIndex >= 0 ? matchIndex : 0;

  const start = Math.max(0, effectiveMatchIndex - SNIPPET_PREFIX_BYTES);
  const end = Math.min(
    text.length,
    effectiveMatchIndex + query.length + SNIPPET_SUFFIX_BYTES,
  );

  const prefix = start > 0 ? "..." : "";
  const suffix = end < text.length ? "..." : "";
  const body = text.substring(start, end).trim();

  return `${prefix}${body}${suffix}`;
}
```

## Tests to Port

Port the tests from `search.rs` `mod tests`:

```typescript
// src/features/sessions/lib/__tests__/sessionContentSearch.test.ts

import { describe, it, expect } from "vitest";
// Import the internal functions (may need to export them for testing)

describe("sessionContentSearch", () => {
  it("finds user and assistant text matches", () => { /* ... */ });
  it("includes system notifications", () => { /* ... */ });
  it("skips tool and reasoning content", () => { /* ... */ });
  it("counts multiple matches in one session", () => { /* ... */ });
  it("builds trimmed snippets around first match", () => { /* ... */ });
  // ... port all tests from search.rs
});
```

## Integration with `useSessionSearch`

The existing `useSessionSearch` hook calls `acpSearchSessions()` from `@/shared/api/acp`. In Step 07, that function will be rewired to call `searchSessionsViaExports` from this module instead of `invoke("acp_search_sessions")`.

## Verification

1. `pnpm typecheck` passes.
2. `pnpm check` passes.
3. `pnpm test` — all ported search tests pass.

## Files Created

| File | Purpose |
|------|---------|
| `src/features/sessions/lib/sessionContentSearch.ts` | Session content search logic |
| `src/features/sessions/lib/__tests__/sessionContentSearch.test.ts` | Tests |

## Dependencies

- Step 05 (`acpSessionManager.ts` — provides `exportSession()`)

## Notes

- The Rust `build_snippet` uses byte-level `floor_char_boundary` / `ceil_char_boundary` for UTF-8 safety. In JavaScript, `String.substring()` operates on UTF-16 code units and is already safe for slicing. The snippet boundaries may differ slightly from the Rust version for multi-byte characters, but this is cosmetic.
- The Rust search is sequential (one session at a time). The TS version could parallelize with `Promise.all` for better performance, but sequential is simpler and matches the Rust behavior. Consider parallelizing later if search is slow.
- The `exportSession` call goes through the ACP client to `goose serve`, which reads from its database. This is the same path as the Rust code — no change in data source.
