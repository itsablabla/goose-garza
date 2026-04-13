# Adaptive Agent P0: Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Make Goose learn and adapt like Hermes — user profile memory, enforced memory budgets, tool result size caps with intelligent truncation, and skill write/patch capability.

**Architecture:** Three workstreams that build on existing infrastructure: (1) Upgrade the memory MCP server with user_profile category and size budgets, (2) Add a layered tool result truncation system at the global chokepoint in `large_response_handler.rs` plus per-tool improvements, (3) Add skill creation/patching tools to the skills platform extension.

**Tech Stack:** Rust, MCP (rmcp crate), existing goose extension system.

---

## Workstream A: Memory Categories & Budgets

### Task A1: Add user_profile category with priority loading

**Objective:** Separate "who the user is" from "what the agent has learned" in the memory system.

**Files:**
- Modify: `crates/goose-mcp/src/memory/mod.rs`
- Test: `crates/goose-mcp/src/memory/mod.rs` (existing test module at bottom)

**What to do:**

The memory server currently treats all categories equally. Add special handling for `user_profile`:

1. Add constants at the top of the file (near line 30):

```rust
/// Reserved category for user identity — name, role, preferences, style.
/// Always loaded first and separately in system prompt.
const USER_PROFILE_CATEGORY: &str = "user_profile";

/// Maximum total chars for all global memories in system prompt.
const GLOBAL_MEMORY_BUDGET: usize = 2200;

/// Maximum total chars for user profile in system prompt.
const USER_PROFILE_BUDGET: usize = 1375;
```

2. In `MemoryServer::new()` (line 90-145), restructure the instructions building:

Currently (lines 130-140):
```rust
instructions.push_str("Global Memories:\n");
for (category, memories) in &global_memories {
    instructions.push_str(&format!("Category: {}\n", category));
    for memory in memories {
        instructions.push_str(&format!("- {}\n", memory));
    }
}
```

Replace with logic that:
- Extracts `user_profile` category first, renders it under a `# User Profile` header
- Renders remaining categories under `# Memories` header  
- Enforces character budgets on each section (truncate oldest entries if over budget)

New approach:
```rust
// Separate user_profile from other memories
let user_profile = global_memories.remove(USER_PROFILE_CATEGORY);

// Render user profile section (budget: USER_PROFILE_BUDGET)
if let Some(profile_entries) = user_profile {
    instructions.push_str("\n# User Profile\n");
    let mut profile_chars = 0;
    for memory in &profile_entries {
        let entry = format!("- {}\n", memory);
        if profile_chars + entry.len() > USER_PROFILE_BUDGET {
            instructions.push_str(&format!(
                "\n[Profile at capacity ({}/{} chars). Remove old entries before adding new ones.]\n",
                profile_chars, USER_PROFILE_BUDGET
            ));
            break;
        }
        instructions.push_str(&entry);
        profile_chars += entry.len();
    }
}

// Render other memories (budget: GLOBAL_MEMORY_BUDGET)
if !global_memories.is_empty() {
    instructions.push_str("\n# Memories\n");
    let mut memory_chars = 0;
    for (category, memories) in &global_memories {
        let header = format!("## {}\n", category);
        if memory_chars + header.len() > GLOBAL_MEMORY_BUDGET {
            instructions.push_str(&format!(
                "\n[Memory at capacity ({}/{} chars). Curate: replace or remove low-value entries.]\n",
                memory_chars, GLOBAL_MEMORY_BUDGET
            ));
            break;
        }
        instructions.push_str(&header);
        memory_chars += header.len();
        for memory in memories {
            let entry = format!("- {}\n", memory);
            if memory_chars + entry.len() > GLOBAL_MEMORY_BUDGET {
                instructions.push_str(&format!(
                    "\n[Memory at capacity ({}/{} chars). Curate: replace or remove low-value entries.]\n",
                    memory_chars, GLOBAL_MEMORY_BUDGET
                ));
                break;
            }
            instructions.push_str(&entry);
            memory_chars += entry.len();
        }
    }
}
```

3. Update the instruction text (lines 91-103 and 119-128) to tell the agent about user_profile:

Add to the base instructions:
```
Use the 'user_profile' category for WHO the user is: name, role, timezone, communication style, preferences, pet peeves.
Use other categories for WHAT you've learned: environment facts, project conventions, tool quirks.

Save proactively when:
- User corrects you or says 'remember this'
- User shares a preference, habit, or personal detail
- You discover something about the environment
Do NOT save: task progress, session outcomes, temporary state.

Memory has size limits. When at capacity, curate: replace outdated entries, remove low-value ones.
```

Remove the line "Always confirm with the user before saving" — this kills proactive memory. The agent should save silently when it notices important facts.

**Verification:**
```bash
cargo test -p goose-mcp -- memory
cargo clippy -p goose-mcp -- -D warnings
```

**Commit:**
```bash
git add crates/goose-mcp/src/memory/mod.rs
git commit -s -m "feat(memory): add user_profile category with priority loading and size budgets"
```

---

### Task A2: Add replace_memory tool

**Objective:** Let the agent update an existing memory entry without needing to remove+re-add (which requires knowing the exact old content).

**Files:**
- Modify: `crates/goose-mcp/src/memory/mod.rs`

**What to do:**

Currently the agent can only `remove_specific_memory` (substring match) + `remember_memory` (append). This is clunky for curation. Add a `replace_memory` tool that does an atomic find-and-replace.

1. Add a new params struct:
```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaceMemoryParams {
    /// The memory category
    category: String,
    /// Substring that uniquely identifies the memory entry to replace
    old_content: String,
    /// The new content for this entry (replaces the entire entry)
    new_content: String,
    /// Optional tags for the new entry
    #[serde(default)]
    tags: Vec<String>,
    /// Whether to operate on global or local memory
    #[serde(default = "default_true")]
    is_global: bool,
}
```

2. Add a `replace_memory` method on MemoryServer that:
   - Reads the category file
   - Finds the entry containing `old_content` (substring match, same as remove_specific_memory)
   - Replaces it with `new_content` (preserving tags)
   - Writes back
   - Returns error if old_content matches zero or multiple entries

3. Register the tool in the `tool_router!` macro and `call_tool` match.

**Verification:**
```bash
cargo test -p goose-mcp -- memory
cargo clippy -p goose-mcp -- -D warnings
```

**Commit:**
```bash
git add crates/goose-mcp/src/memory/mod.rs
git commit -s -m "feat(memory): add replace_memory tool for atomic entry updates"
```

---

### Task A3: Enforce budgets at write time

**Objective:** When the agent tries to save a memory that would exceed the budget, return a warning instead of silently growing.

**Files:**
- Modify: `crates/goose-mcp/src/memory/mod.rs`

**What to do:**

In the `remember` method (line 205-230), after writing the new entry, check the total size of the category file:

```rust
// After writing the entry to file...
let file_content = std::fs::read_to_string(&file_path).unwrap_or_default();
let budget = if category == USER_PROFILE_CATEGORY {
    USER_PROFILE_BUDGET
} else {
    GLOBAL_MEMORY_BUDGET
};

if file_content.len() > budget {
    let over_by = file_content.len() - budget;
    return Ok(CallToolResult::success(vec![Content::text(format!(
        "Memory saved, but category '{}' is now over budget by {} chars ({}/{} chars). \
         Use replace_memory or remove_specific_memory to curate — oldest/least-valuable entries should go.",
        category, over_by, file_content.len(), budget
    ))]));
}
```

This doesn't prevent the write (the agent needs to see what it saved to curate intelligently), but it creates curation pressure on every subsequent save.

**Verification:**
```bash
cargo test -p goose-mcp -- memory
```

**Commit:**
```bash
git add crates/goose-mcp/src/memory/mod.rs
git commit -s -m "feat(memory): warn when category exceeds size budget after write"
```

---

### Task A4: Tests for memory budgets and user_profile

**Objective:** Test the budget enforcement, user_profile rendering, and replace_memory tool.

**Files:**
- Modify: `crates/goose-mcp/src/memory/mod.rs` (test module)
- Or create: `crates/goose-mcp/tests/memory_test.rs`

**Tests to write:**

1. `test_user_profile_renders_first_in_instructions` — Create a MemoryServer with both user_profile and general memories, verify instructions string has "# User Profile" before "# Memories"

2. `test_user_profile_budget_truncates` — Write entries exceeding USER_PROFILE_BUDGET to user_profile, verify instructions string contains the "at capacity" message and stops rendering entries

3. `test_global_memory_budget_truncates` — Same for general memories with GLOBAL_MEMORY_BUDGET

4. `test_replace_memory_updates_entry` — Write a memory, replace it, verify old gone and new present

5. `test_replace_memory_fails_on_ambiguous_match` — Write two entries containing same substring, verify replace returns error

6. `test_remember_warns_on_over_budget` — Write enough memories to exceed budget, verify the tool result contains the over-budget warning

**Verification:**
```bash
cargo test -p goose-mcp -- memory
```

**Commit:**
```bash
git add crates/goose-mcp/
git commit -s -m "test(memory): add tests for user_profile, budgets, and replace_memory"
```

---

## Workstream B: Tool Result Size Caps

### Task B1: Add head+tail truncation utility

**Objective:** Create a reusable truncation function that keeps the beginning and end of large text, dropping the middle.

**Files:**
- Modify: `crates/goose/src/utils.rs`

**What to do:**

Add a `head_tail_truncate` function:

```rust
/// Truncate large text keeping the head and tail, dropping the middle.
/// head_ratio controls the split (0.4 = 40% head, 60% tail).
/// Returns the original string if under max_chars.
pub fn head_tail_truncate(s: &str, max_chars: usize, head_ratio: f64) -> String {
    let total_chars = s.chars().count();
    if total_chars <= max_chars {
        return s.to_string();
    }

    let head_chars = ((max_chars as f64) * head_ratio) as usize;
    let tail_chars = max_chars - head_chars;
    let dropped = total_chars - max_chars;

    let head: String = s.chars().take(head_chars).collect();
    let tail: String = s.chars().skip(total_chars - tail_chars).collect();

    format!(
        "{}\n\n... [{} characters truncated] ...\n\n{}",
        head, dropped, tail
    )
}
```

Add tests:
```rust
#[test]
fn test_head_tail_truncate_short() {
    assert_eq!(head_tail_truncate("hello", 100, 0.4), "hello");
}

#[test]
fn test_head_tail_truncate_long() {
    let input = "a".repeat(1000);
    let result = head_tail_truncate(&input, 100, 0.4);
    assert!(result.contains("truncated"));
    assert!(result.starts_with(&"a".repeat(40)));
    assert!(result.ends_with(&"a".repeat(60)));
}
```

**Verification:**
```bash
cargo test -p goose -- utils
```

**Commit:**
```bash
git add crates/goose/src/utils.rs
git commit -s -m "feat(utils): add head+tail truncation for large text"
```

---

### Task B2: Lower the large_response_handler threshold and add head+tail truncation

**Objective:** Currently, tool results under 200K chars pass through completely. Add a tiered system: above 100K chars gets head+tail truncated (not file-offloaded), above 200K gets file-offloaded as before.

**Files:**
- Modify: `crates/goose/src/agents/large_response_handler.rs`

**What to do:**

Replace the current single-threshold logic with a two-tier system:

```rust
use crate::utils::head_tail_truncate;

/// Results above this get head+tail truncated in-place
const TRUNCATION_THRESHOLD: usize = 100_000;

/// Results above this get written to file (existing behavior)
const FILE_OFFLOAD_THRESHOLD: usize = 200_000;

/// Head ratio for truncation (40% head, 60% tail)
const HEAD_RATIO: f64 = 0.4;
```

In `process_tool_response`, modify the text handling:

```rust
Some(text_content) => {
    let char_count = text_content.text.chars().count();
    if char_count > FILE_OFFLOAD_THRESHOLD {
        // Existing file offload behavior (unchanged)
        // ...
    } else if char_count > TRUNCATION_THRESHOLD {
        // New: in-place head+tail truncation
        let truncated = head_tail_truncate(
            &text_content.text,
            TRUNCATION_THRESHOLD,
            HEAD_RATIO,
        );
        processed_contents.push(Content::text(truncated));
    } else {
        processed_contents.push(content);
    }
}
```

Update existing tests and add:
- `test_medium_text_gets_head_tail_truncated` — 150K chars → truncated to ~100K with head+tail
- `test_truncated_text_has_head_and_tail` — verify the result starts with the original head and ends with the original tail

**Verification:**
```bash
cargo test -p goose -- large_response
```

**Commit:**
```bash
git add crates/goose/src/agents/large_response_handler.rs
git commit -s -m "feat(agent): add head+tail truncation tier to large_response_handler"
```

---

### Task B3: Improve shell tool truncation to use head+tail

**Objective:** The shell tool currently keeps only the LAST 50 lines when truncating. Switch to head+tail so the model sees both the start (command setup, headers) and end (final output, errors).

**Files:**
- Modify: `crates/goose/src/agents/platform_extensions/developer/shell.rs`

**What to do:**

In `truncate_output()` (line 625), change the preview logic from tail-only to head+tail:

Currently:
```rust
let preview_start = total_lines.saturating_sub(OUTPUT_PREVIEW_LINES);
let preview = lines[preview_start..].join("\n");
```

Replace with:
```rust
let head_lines = OUTPUT_PREVIEW_LINES * 2 / 5;  // 20 lines = 40%
let tail_lines = OUTPUT_PREVIEW_LINES - head_lines;  // 30 lines = 60%
let head: Vec<&str> = lines[..head_lines.min(total_lines)].to_vec();
let tail: Vec<&str> = lines[total_lines.saturating_sub(tail_lines)..].to_vec();
let dropped_lines = total_lines - head.len() - tail.len();
let preview = format!(
    "{}\n\n... [{} lines truncated, full output saved to {}] ...\n\n{}",
    head.join("\n"),
    dropped_lines,
    saved_file_path,
    tail.join("\n"),
);
```

The `saved_file_path` variable already exists from the file-save logic earlier in the function. This needs slight restructuring — save the file first, then build the preview using the path.

**Verification:**
```bash
cargo test -p goose -- shell
cargo test -p goose -- truncat
```

**Commit:**
```bash
git add crates/goose/src/agents/platform_extensions/developer/shell.rs
git commit -s -m "feat(shell): switch truncation from tail-only to head+tail strategy"
```

---

### Task B4: Add size cap to ACP file read

**Objective:** The `read` tool in `goose-acp/src/fs.rs` has no size limit. Add a configurable max that tells the agent to use line ranges for large files.

**Files:**
- Modify: `crates/goose-acp/src/fs.rs`

**What to do:**

Add a constant:
```rust
/// Maximum characters returned from a file read. Larger files must use line/limit params.
const MAX_FILE_READ_CHARS: usize = 200_000;
```

In `acp_read()` (line 125), after reading the file content, check size:

```rust
let content = std::fs::read_to_string(&path)?;
let content = apply_line_limit(&content, params.line, params.limit);

if content.chars().count() > MAX_FILE_READ_CHARS {
    let total_lines = content.lines().count();
    return Ok(CallToolResult::success(vec![Content::text(format!(
        "File too large to read in full ({} chars, {} lines). Use the 'line' and 'limit' parameters to read specific sections.\n\nFirst 50 lines:\n{}",
        content.chars().count(),
        total_lines,
        content.lines().take(50).collect::<Vec<_>>().join("\n")
    ))]));
}
```

**Verification:**
```bash
cargo test -p goose-acp
cargo clippy -p goose-acp -- -D warnings
```

**Commit:**
```bash
git add crates/goose-acp/src/fs.rs
git commit -s -m "feat(acp): add size cap to file read with guidance to use line ranges"
```

---

### Task B5: Add size cap to automation_script output

**Objective:** The `automation_script` tool in computercontroller returns full stdout/stderr with no limit.

**Files:**
- Modify: `crates/goose-mcp/src/computercontroller/mod.rs`

**What to do:**

Find `automation_script_impl()` (line 762). Where it builds the result content from stdout/stderr (around line 911), apply truncation before returning:

```rust
// Use a shared truncation helper or inline:
const AUTOMATION_OUTPUT_CAP: usize = 50_000;

fn truncate_output_text(text: &str, max_chars: usize) -> String {
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }
    let head_chars = (max_chars as f64 * 0.4) as usize;
    let tail_chars = max_chars - head_chars;
    let head: String = text.chars().take(head_chars).collect();
    let tail: String = text.chars().skip(total - tail_chars).collect();
    format!(
        "{}\n\n... [{} characters truncated] ...\n\n{}",
        head, total - max_chars, tail
    )
}
```

Apply to both stdout and stderr before they enter the Content::text() result.

**Verification:**
```bash
cargo test -p goose-mcp -- automation
cargo clippy -p goose-mcp -- -D warnings
```

**Commit:**
```bash
git add crates/goose-mcp/src/computercontroller/mod.rs
git commit -s -m "feat(computercontroller): cap automation_script output at 50K chars with head+tail"
```

---

### Task B6: Tests for tool result size caps

**Objective:** Verify the layered truncation system works end-to-end.

**Files:**
- Modify test modules in each affected file

**Tests:**

1. In `large_response_handler.rs`:
   - `test_medium_text_truncated_with_head_tail` — 150K chars → ~100K with head+tail marker
   - `test_very_large_text_still_offloaded` — 250K chars → file offload (existing behavior preserved)
   - `test_exactly_at_threshold_passes_through` — 100K chars exact → no truncation

2. In `shell.rs`:
   - `test_head_tail_truncation_has_both_ends` — generate 5000-line output, verify result contains first 20 lines AND last 30 lines
   - Update existing truncation tests for new behavior

3. In `goose-acp/src/fs.rs`:
   - `test_large_file_read_returns_guidance` — read a file >200K, verify response mentions line/limit params and includes first 50 lines

**Verification:**
```bash
cargo test -p goose -- large_response
cargo test -p goose -- shell
cargo test -p goose-acp
```

**Commit:**
```bash
git add crates/
git commit -s -m "test: add tests for layered tool result size caps"
```

---

## Workstream C: Skill Creation & Patching

### Task C1: Add create_skill tool to skills extension

**Objective:** Let the agent create new skills from experience — the most impactful learning capability.

**Files:**
- Modify: `crates/goose/src/agents/platform_extensions/skills.rs`

**What to do:**

Currently the extension only has `load_skill`. Add `create_skill`:

1. Add a params struct:
```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct CreateSkillParams {
    /// Skill name (lowercase, hyphens allowed, no slashes). e.g. "docker-networking"
    name: String,
    /// Full SKILL.md content including YAML frontmatter (---\nname: ...\ndescription: ...\n---\n)
    content: String,
}
```

2. Add the tool implementation:
```rust
async fn create_skill(&self, params: CreateSkillParams) -> Result<CallToolResult> {
    // Validate name: lowercase, hyphens/underscores only, no slashes, max 64 chars
    if !params.name.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c == '_')
        || params.name.contains('/')
        || params.name.len() > 64
        || params.name.is_empty()
    {
        return Ok(CallToolResult::error(vec![Content::text(
            "Invalid skill name. Use lowercase letters, hyphens, underscores. Max 64 chars."
        )]));
    }

    // Validate YAML frontmatter exists
    if !params.content.starts_with("---") {
        return Ok(CallToolResult::error(vec![Content::text(
            "Skill content must start with YAML frontmatter (---\\nname: ...\\ndescription: ...\\n---)"
        )]));
    }

    // Write to global skills directory: ~/.config/goose/skills/{name}/SKILL.md
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("goose")
        .join("skills")
        .join(&params.name);
    
    std::fs::create_dir_all(&config_dir)?;
    let skill_path = config_dir.join("SKILL.md");
    
    if skill_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Skill '{}' already exists. Use patch_skill to update it.",
            params.name
        ))]));
    }
    
    std::fs::write(&skill_path, &params.content)?;
    
    Ok(CallToolResult::success(vec![Content::text(format!(
        "Created skill '{}' at {}",
        params.name,
        skill_path.display()
    ))]))
}
```

3. Register in the tool router. The skills extension uses `tool_router!` macro — check the pattern used for `load_skill` and add `create_skill` the same way.

**Verification:**
```bash
cargo test -p goose -- skills
cargo clippy -p goose -- -D warnings
```

**Commit:**
```bash
git add crates/goose/src/agents/platform_extensions/skills.rs
git commit -s -m "feat(skills): add create_skill tool for agent-generated skills"
```

---

### Task C2: Add patch_skill tool to skills extension

**Objective:** Let the agent fix/update existing skills when it discovers they're wrong or incomplete.

**Files:**
- Modify: `crates/goose/src/agents/platform_extensions/skills.rs`

**What to do:**

1. Add params struct:
```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct PatchSkillParams {
    /// Name of the skill to patch
    name: String,
    /// Text to find in the skill (must be unique)
    old_text: String,
    /// Replacement text
    new_text: String,
}
```

2. Add the tool implementation:
```rust
async fn patch_skill(&self, params: PatchSkillParams) -> Result<CallToolResult> {
    // Find the skill across all discovery paths
    let working_dir = /* get from context */;
    let skills = discover_skills(working_dir);
    
    let skill = skills.iter().find(|s| s.name == params.name);
    let Some(skill) = skill else {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Skill '{}' not found", params.name
        ))]));
    };
    
    // Only allow patching user-owned skills (not builtins)
    if matches!(skill.kind, SourceKind::BuiltinSkill) {
        return Ok(CallToolResult::error(vec![Content::text(
            "Cannot patch builtin skills. Create a new skill instead."
        )]));
    }
    
    let skill_path = PathBuf::from(&skill.path).join("SKILL.md");
    let content = std::fs::read_to_string(&skill_path)?;
    
    // Verify old_text exists and is unique
    let matches: Vec<_> = content.match_indices(&params.old_text).collect();
    if matches.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "old_text not found in skill. Load the skill first to see current content."
        )]));
    }
    if matches.len() > 1 {
        return Ok(CallToolResult::error(vec![Content::text(
            "old_text matches multiple locations. Use a more specific string."
        )]));
    }
    
    let new_content = content.replacen(&params.old_text, &params.new_text, 1);
    std::fs::write(&skill_path, &new_content)?;
    
    Ok(CallToolResult::success(vec![Content::text(format!(
        "Patched skill '{}' — replaced {} chars with {} chars",
        params.name,
        params.old_text.len(),
        params.new_text.len()
    ))]))
}
```

3. Register in the tool router.

**Verification:**
```bash
cargo test -p goose -- skills
cargo clippy -p goose -- -D warnings
```

**Commit:**
```bash
git add crates/goose/src/agents/platform_extensions/skills.rs
git commit -s -m "feat(skills): add patch_skill tool for updating existing skills"
```

---

### Task C3: Add skill creation nudge to system prompt

**Objective:** After complex tool-heavy work, remind the agent to consider saving a skill.

**Files:**
- Modify: `crates/goose/src/agents/agent.rs`
- Modify: `crates/goose/src/prompts/prompt_manager.rs`

**What to do:**

The agent already tracks tool call counts in the main loop. Add a nudge mechanism:

1. In `prompt_manager.rs`, add a method:
```rust
/// Add a skill creation nudge to the system prompt extras
pub fn add_skill_nudge(&mut self) {
    self.add_system_prompt_extra(
        "After this complex task, consider: should this approach be saved as a skill? \
         Use create_skill if you discovered a non-trivial workflow (5+ tool calls, error recovery, \
         or user corrections). Use patch_skill if an existing skill was wrong or incomplete.".to_string()
    );
}
```

2. In `agent.rs`, in the main loop of `reply_internal()`, track the tool call count. When it crosses a threshold (e.g., 8 tool calls in this reply), set a flag. After the loop exits (when the agent gives its final response), if the flag is set, inject the nudge for the NEXT turn.

Actually, simpler: use the existing iteration counter in the loop. After the loop exits:
```rust
if iteration_count >= 8 {
    self.prompt_manager.lock().await.add_skill_nudge();
}
```

This way the nudge appears in the next user turn's system prompt, reminding the agent to reflect on the completed work.

**Verification:**
```bash
cargo build -p goose
cargo clippy -p goose -- -D warnings
```

**Commit:**
```bash
git add crates/goose/src/agents/agent.rs crates/goose/src/prompts/prompt_manager.rs
git commit -s -m "feat(agent): nudge skill creation after complex tasks (8+ tool calls)"
```

---

### Task C4: Tests for skill creation and patching

**Objective:** Test the skill write/patch tools and the nudge mechanism.

**Files:**
- Create: `crates/goose/tests/skills_write_test.rs`

**Tests:**

1. `test_create_skill_writes_to_disk` — call create_skill, verify SKILL.md exists with correct content
2. `test_create_skill_rejects_invalid_names` — uppercase, slashes, too long, empty
3. `test_create_skill_rejects_missing_frontmatter` — content without `---`
4. `test_create_skill_rejects_duplicate` — create twice, verify error on second
5. `test_patch_skill_replaces_text` — create then patch, verify content updated
6. `test_patch_skill_rejects_ambiguous_match` — patch with text that appears twice
7. `test_patch_skill_rejects_missing_text` — patch with text not in skill
8. `test_patch_skill_rejects_builtin` — attempt to patch a builtin skill

Use temp directories for the skills path to avoid polluting the real filesystem.

**Verification:**
```bash
cargo test -p goose -- skills_write
```

**Commit:**
```bash
git add crates/goose/tests/skills_write_test.rs
git commit -s -m "test(skills): add tests for create_skill and patch_skill"
```

---

## Workstream D: System Prompt Memory Guidance

### Task D1: Add memory and skill guidance to system prompt

**Objective:** Tell the agent HOW to use memory and skills proactively. This pairs with the tools from Workstreams A and C.

**Files:**
- Modify: `crates/goose/src/prompts/system.md`

**What to do:**

After the extensions section and before "# Response Guidelines", add:

```
{% if has_memory_extension %}
# Memory

You have persistent memory across sessions. Save proactively — don't wait to be asked.

**Save when:**
- User corrects you or says "remember this"
- User shares a preference, habit, or personal detail (use category: user_profile)
- You discover something about the environment (use category: environment)
- You learn a project convention or tool quirk (use relevant category name)

**Don't save:** task progress, session outcomes, temporary TODO state.

Use replace_memory to update stale entries. When memory is at capacity, curate: remove low-value entries to make room.
{% endif %}

{% if has_skills_extension %}
# Skills

After completing complex work (many tool calls, error recovery, non-obvious workflow), consider saving a reusable skill with create_skill. If you loaded a skill and found it wrong or incomplete, patch it immediately with patch_skill.
{% endif %}
```

In `prompt_manager.rs`, add the `has_memory_extension` and `has_skills_extension` booleans to `SystemPromptContext` by checking the extension list for extensions named "memory" and "skills".

**Verification:**
```bash
cargo build -p goose
```

**Commit:**
```bash
git add crates/goose/src/prompts/system.md crates/goose/src/prompts/prompt_manager.rs
git commit -s -m "feat(prompts): add memory and skill guidance to system prompt"
```

---

## Implementation Order

The tasks have these dependencies:

```
B1 (head+tail utility)
├── B2 (large_response_handler upgrade) 
├── B3 (shell truncation improvement)
├── B5 (automation_script cap)
└── B6 (tests)

B4 (file read cap) — independent

A1 (user_profile + budgets)
├── A2 (replace_memory tool)
├── A3 (write-time budget enforcement)
└── A4 (tests)

C1 (create_skill)
├── C2 (patch_skill)
├── C3 (skill nudge)
└── C4 (tests)

D1 (system prompt guidance) — depends on A1, C1
```

**Recommended execution order:**

1. B1 → B2 → B3 → B4 → B5 → B6 (context efficiency first — creates headroom)
2. A1 → A2 → A3 → A4 (memory upgrade)
3. C1 → C2 → C3 → C4 (skills upgrade)
4. D1 (ties it together)

Total: 14 tasks, each 5-15 minutes of focused implementation.

---

## Final PR

After all tasks:
```bash
cargo fmt
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

Single PR with title: `feat: adaptive agent P0 — memory categories, tool result caps, skill creation`
