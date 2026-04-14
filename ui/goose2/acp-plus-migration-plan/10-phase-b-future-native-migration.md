# Step 10: Phase B — Migrate Config, Personas, Skills, Projects, Git, Doctor to `goose serve`

## Objective

This step is a roadmap for the long-term migration. Each subsystem currently handled by Rust Tauri commands will move behind `goose serve` ACP extension methods, callable from TypeScript via `client.extMethod("goose/<domain>/<action>", params)` or via generated `GooseExtClient` methods.

**This step requires backend changes to the goose crate** — adding new ACP extension methods to `goose serve`. It cannot be done purely in the Tauri app.

## Current State After Phase A (Steps 01–09)

After Phase A, the Rust Tauri backend still handles:

| Module | Rust File(s) | Lines | Native Dependency |
|--------|-------------|-------|-------------------|
| Config (config.yaml, secrets, keyring) | `services/goose_config.rs`, `services/provider_defs.rs` | ~590 | Keyring, file system |
| Credentials commands | `commands/credentials.rs` | ~50 | GooseConfig |
| Personas | `services/personas.rs`, `types/agents.rs`, `types/builtin_personas.rs` | ~920 | File system |
| Persona commands | `commands/agents.rs` | ~210 | PersonaStore |
| Skills | `commands/skills.rs` | ~320 | File system |
| Projects | `commands/projects.rs` | ~495 | File system |
| Git operations | `commands/git.rs`, `commands/git_changes.rs` | ~570 | Shell commands |
| Doctor | `commands/doctor.rs` | ~15 | `doctor` crate |
| Agent setup | `commands/agent_setup.rs` | ~310 | Shell commands, streaming output |
| Model setup | `commands/model_setup.rs` | ~220 | Shell commands, streaming output |
| System utilities | `commands/system.rs` | ~360 | File system, dialog |
| **Total** | | **~4,060** | |

## Migration Pattern

For each subsystem, the pattern is:

1. **Backend**: Add ACP extension methods to `goose serve` (in the `goose-acp` or `goose` crate)
2. **Schema**: Regenerate the ACP schema (`npm run build:schema` in `ui/acp/`)
3. **Client**: The `GooseExtClient` auto-generates typed methods from the schema
4. **Frontend**: Replace `invoke("rust_command")` calls with `client.goose.<method>()` calls
5. **Cleanup**: Delete the Rust Tauri command and service code

## Subsystem Migration Details

### B1: Config Management

**Priority: High** — Config is needed for provider setup, which is part of the core onboarding flow.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/config/get` | `{ key: string }` | `{ value: string \| null }` |
| `goose/config/set` | `{ key: string, value: string }` | `{}` |
| `goose/config/delete` | `{ key: string }` | `{ removed: boolean }` |
| `goose/secret/getMasked` | `{ key: string }` | `{ value: string \| null }` |
| `goose/secret/set` | `{ key: string, value: string }` | `{}` |
| `goose/secret/delete` | `{ key: string }` | `{ removed: boolean }` |
| `goose/provider/status` | `{ providerId: string }` | `{ providerId: string, isConfigured: boolean }` |
| `goose/provider/statusAll` | `{}` | `{ providers: [{ providerId: string, isConfigured: boolean }] }` |
| `goose/provider/fields` | `{ providerId: string }` | `{ fields: [{ key: string, value: string \| null, isSet: boolean, isSecret: boolean, required: boolean }] }` |
| `goose/provider/deleteConfig` | `{ providerId: string }` | `{}` |

#### Backend Implementation Notes

- The goose binary already has config management internally (`goose configure` command). The extension methods expose the same logic over ACP.
- Keyring access happens in the `goose serve` process (which runs natively), so there's no loss of capability.
- The `provider_defs.rs` static definitions should move to the goose crate (or already exist there).

#### Frontend Changes

- Replace `invoke("get_provider_config")` → `client.goose.gooseProviderFields({ providerId })`
- Replace `invoke("save_provider_field")` → `client.goose.gooseSecretSet({ key, value })` or `client.goose.gooseConfigSet({ key, value })`
- Replace `invoke("delete_provider_config")` → `client.goose.gooseProviderDeleteConfig({ providerId })`
- Replace `invoke("check_all_provider_status")` → `client.goose.gooseProviderStatusAll({})`
- Replace `invoke("restart_app")` → keep in Rust (native window management)

#### Files Deleted After B1

- `src-tauri/src/services/goose_config.rs`
- `src-tauri/src/services/provider_defs.rs`
- `src-tauri/src/commands/credentials.rs` (except `restart_app` which stays)
- Remove `keyring` dependency from `Cargo.toml` (all 3 platform variants)
- Remove `etcetera` dependency

---

### B2: Personas

**Priority: Medium** — Personas are used in the chat flow but not on the critical path.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/personas/list` | `{}` | `{ personas: Persona[] }` |
| `goose/personas/create` | `CreatePersonaRequest` | `{ persona: Persona }` |
| `goose/personas/update` | `{ id: string, ...UpdatePersonaRequest }` | `{ persona: Persona }` |
| `goose/personas/delete` | `{ id: string }` | `{}` |
| `goose/personas/refresh` | `{}` | `{ personas: Persona[] }` |
| `goose/personas/export` | `{ id: string }` | `{ json: string, suggestedFilename: string }` |
| `goose/personas/import` | `{ fileBytes: number[], fileName: string }` | `{ personas: Persona[] }` |
| `goose/personas/saveAvatar` | `{ personaId: string, bytes: number[], extension: string }` | `{ filename: string }` |
| `goose/personas/avatarsDir` | `{}` | `{ path: string }` |

#### Backend Implementation Notes

- Persona storage (`~/.goose/personas.json`, `~/.goose/agents/*.md`) is file-based. The goose binary can read/write these.
- Avatar handling (`~/.goose/avatars/`) is also file-based.
- Builtin personas are currently defined in `types/builtin_personas.rs` — these should move to the goose crate.

#### Files Deleted After B2

- `src-tauri/src/services/personas.rs`
- `src-tauri/src/types/agents.rs`
- `src-tauri/src/types/builtin_personas.rs`
- `src-tauri/src/types/messages.rs`
- `src-tauri/src/types/mod.rs`
- `src-tauri/src/commands/agents.rs`

---

### B3: Skills

**Priority: Low** — Skills are a secondary feature.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/skills/list` | `{}` | `{ skills: SkillInfo[] }` |
| `goose/skills/create` | `{ name, description, instructions }` | `{}` |
| `goose/skills/update` | `{ name, description, instructions }` | `{ skill: SkillInfo }` |
| `goose/skills/delete` | `{ name: string }` | `{}` |
| `goose/skills/export` | `{ name: string }` | `{ json: string, filename: string }` |
| `goose/skills/import` | `{ fileBytes: number[], fileName: string }` | `{ skills: SkillInfo[] }` |

#### Files Deleted After B3

- `src-tauri/src/commands/skills.rs`

---

### B4: Projects

**Priority: Low** — Projects are a secondary feature.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/projects/list` | `{}` | `{ projects: ProjectInfo[] }` |
| `goose/projects/create` | `{ name, description, prompt, icon, color, ... }` | `{ project: ProjectInfo }` |
| `goose/projects/update` | `{ id, name, description, prompt, icon, color, ... }` | `{ project: ProjectInfo }` |
| `goose/projects/delete` | `{ id: string }` | `{}` |
| `goose/projects/get` | `{ id: string }` | `{ project: ProjectInfo }` |
| `goose/projects/listArchived` | `{}` | `{ projects: ProjectInfo[] }` |
| `goose/projects/archive` | `{ id: string }` | `{}` |
| `goose/projects/restore` | `{ id: string }` | `{}` |

#### Files Deleted After B4

- `src-tauri/src/commands/projects.rs`

---

### B5: Git Operations

**Priority: Medium** — Git state is shown in the workspace widget and context panel.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/git/state` | `{ path: string }` | `GitState` |
| `goose/git/changedFiles` | `{ path: string }` | `{ files: ChangedFile[] }` |
| `goose/git/switchBranch` | `{ path, branch }` | `{}` |
| `goose/git/stash` | `{ path }` | `{}` |
| `goose/git/init` | `{ path }` | `{}` |
| `goose/git/fetch` | `{ path }` | `{}` |
| `goose/git/pull` | `{ path }` | `{}` |
| `goose/git/createBranch` | `{ path, name, baseBranch }` | `{}` |
| `goose/git/createWorktree` | `{ path, name, branch, createBranch, baseBranch? }` | `CreatedWorktree` |

#### Backend Implementation Notes

- Git operations run shell commands (`git status`, `git switch`, etc.). The goose binary can run these the same way.
- The `ignore` crate is used for `.gitignore`-aware file scanning in `list_files_for_mentions`. This could also move to goose serve.

#### Files Deleted After B5

- `src-tauri/src/commands/git.rs`
- `src-tauri/src/commands/git_changes.rs`

---

### B6: Doctor

**Priority: Low** — Doctor is a diagnostic tool, not on the critical path.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/doctor/run` | `{}` | `DoctorReport` |
| `goose/doctor/fix` | `{ checkId: string, fixType: string }` | `{}` |

#### Backend Implementation Notes

- The `doctor` crate already exists in the goose ecosystem. The extension methods just expose it over ACP.

#### Files Deleted After B6

- `src-tauri/src/commands/doctor.rs`
- Remove `doctor` dependency from `Cargo.toml`

---

### B7: Agent & Model Setup

**Priority: Medium** — Needed for onboarding third-party agents and OAuth flows.

This is the trickiest subsystem because it involves **interactive shell commands with streaming output**. The current Rust code spawns a child process and streams stdout/stderr lines as Tauri events (`agent-setup:output`, `model-setup:output`).

#### Options

**Option A: ACP extension methods with streaming notifications**

Add extension methods that return immediately but stream progress via `SessionNotification` events (or a new notification type). The frontend listens for these notifications the same way it listens for chat streaming.

**Option B: Keep in Rust as the last native commands**

These commands are inherently interactive (they open browsers for OAuth, wait for user input). They may be better suited to native handling. Keep them as the last remaining Tauri commands.

**Option C: Move the logic into `goose serve` as a long-running task**

The goose binary already handles `goose configure` (which is what `model_setup.rs` wraps). Expose a `goose/setup/authenticateProvider` extension method that runs the configure flow and streams output.

#### Recommendation

Start with **Option B** (keep in Rust) and migrate to Option A or C later. These commands are rarely called (only during onboarding) and the streaming output pattern would need a new ACP notification type.

---

### B8: System Utilities

**Priority: Low** — These are helper functions, not core features.

#### New `goose serve` Extension Methods

| Method | Request | Response |
|--------|---------|----------|
| `goose/system/homeDir` | `{}` | `{ path: string }` |
| `goose/system/pathExists` | `{ path: string }` | `{ exists: boolean }` |
| `goose/system/listDir` | `{ path: string }` | `{ entries: FileTreeEntry[] }` |
| `goose/system/listFilesForMentions` | `{ roots: string[], maxResults?: number }` | `{ files: string[] }` |

#### Special Case: `saveExportedSessionFile`

This command uses `tauri_plugin_dialog` to show a native save dialog. This is inherently a Tauri/native operation — it cannot move to `goose serve`. **Keep this in Rust.**

#### Files Deleted After B8

- `src-tauri/src/commands/system.rs` (except `save_exported_session_file` which stays)
- Remove `ignore` dependency from `Cargo.toml`

---

## End State After All Phase B Steps

**Rust Tauri backend:**

```
src-tauri/src/
  lib.rs                    — ~40 lines: spawn goose serve, register ~3 commands
  main.rs                   — 6 lines (unchanged)
  commands/
    mod.rs                  — 3 modules
    acp.rs                  — get_goose_serve_url (~15 lines)
    system.rs               — save_exported_session_file (~40 lines)
    agent_setup.rs          — install/auth agents (~310 lines, if kept)
    model_setup.rs          — model provider auth (~220 lines, if kept)
  services/
    mod.rs                  — 1 module
    acp/
      mod.rs                — 1 module
      goose_serve.rs        — GooseServeProcess (~150 lines)
```

**Total: ~780 lines** if keeping agent/model setup, or **~250 lines** if those also move.

**Cargo.toml dependencies (minimal):**
```toml
tauri = "2"
tauri-plugin-opener = "2"
tauri-plugin-dialog = ">=2,<2.7"
tauri-plugin-window-state = "2"
tauri-plugin-log = "2"
serde = "1"
serde_json = "1"
tokio = "1"
dirs = "6"
log = "0.4"
```

## Migration Priority Summary

| Step | Effort | Blocking On | Value | Recommended Order |
|------|--------|-------------|-------|-------------------|
| B1 (Config) | Medium | Backend ACP methods | High (removes keyring dep) | 1st |
| B5 (Git) | Medium | Backend ACP methods | Medium | 2nd |
| B2 (Personas) | Medium | Backend ACP methods | Medium | 3rd |
| B3 (Skills) | Small | Backend ACP methods | Small | 4th |
| B4 (Projects) | Small | Backend ACP methods | Small | 5th |
| B6 (Doctor) | Small | Backend ACP methods | Small | 6th |
| B8 (System utils) | Small | Backend ACP methods | Small | 7th |
| B7 (Agent/Model setup) | Large | Streaming notification design | Medium | Last (or keep in Rust) |

## How to Propose Backend Changes

For each subsystem, the process is:

1. **Design the ACP extension method schemas** in `crates/goose-acp/`
2. **Implement the handlers** in the goose serve server
3. **Regenerate the schema**: `cd ui/acp && npm run build:schema`
4. **Rebuild the TS client**: `cd ui/acp && npm run build`
5. **Update goose2**: use the new `client.goose.<method>()` calls
6. **Delete the Rust Tauri code**

Each subsystem can be migrated independently. The frontend can use a mix of `invoke()` (for not-yet-migrated subsystems) and `client.goose.*()` (for migrated ones) during the transition.
