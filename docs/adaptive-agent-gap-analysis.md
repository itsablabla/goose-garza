# Adaptive Agent: Gap Analysis & Plan
## Bringing Hermes-style Evolving Intelligence to Goose (and other agents)

_Generated from a comparative analysis of Hermes Agent (Python) and Goose (Rust)_

---

## Executive Summary

Hermes has several systems that make it "learn" and adapt as users interact with it — persistent memory with automatic extraction, skills that get created/patched from experience, trust-scored fact retrieval, dynamic reasoning depth, and background knowledge review loops. It also has aggressive context efficiency mechanisms that keep context usage remarkably low (6% after hours of heavy tool use). Goose has the foundational pieces (memory MCP server, skills discovery, session persistence, context compaction) but they're mostly **passive** — the user must explicitly ask the agent to remember things, the system doesn't evolve its own behavior based on interaction patterns, and tool results flow directly into the main context without isolation or aggressive truncation.

This document maps the gaps and proposes a phased plan to close them.

---

## Feature Comparison Matrix

| Capability | Hermes | Goose | Gap |
|---|---|---|---|
| **Explicit memory save** | memory() tool → MEMORY.md/USER.md | remember_memory MCP tool → .txt files | ✅ Parity (different format) |
| **Memory retrieval** | Injected into system prompt every turn | Global memories loaded at startup | ✅ Parity |
| **Automatic memory extraction** | Background review every N turns; flush before compression/exit | ❌ None | 🔴 Major gap |
| **User profile** | Separate USER.md with preferences, name, role, style | ❌ None (memories are untyped) | 🔴 Major gap |
| **Memory budget/limits** | Hard char limits (2200/1375) force curation | No limits (unbounded .txt files) | 🟡 Design gap |
| **Skills discovery** | Progressive: list → view → load linked files | Similar: list in prompt → load_skill on demand | ✅ Parity |
| **Skill creation from experience** | Agent creates SKILL.md after hard tasks | ❌ Agent can only load, not create | 🔴 Major gap |
| **Skill patching when wrong** | Agent patches outdated skills mid-session | ❌ Read-only | 🔴 Major gap |
| **Background skill review** | Every N tool iterations, background thread reviews | ❌ None | 🔴 Major gap |
| **Session search/recall** | FTS5 search + LLM summarization of past sessions | chatrecall tool (SQL LIKE search, disabled by default) | 🟡 Partial (exists but weak and off) |
| **Context compression** | 4-phase: tool pruning → boundary calc → LLM summary → assembly; iterative updates | LLM summarization + progressive tool removal | 🟡 Goose is simpler but functional |
| **Pre-compression memory flush** | Dedicated flush_memories() before compaction | ❌ None | 🔴 Major gap |
| **Smart model routing** | Per-turn simple/complex classification → cheap/strong model | complete_fast() for internal ops only | 🟡 Partial |
| **Iteration budget pressure** | 70%/90% warnings injected into tool results | Max 1000 turns, no pressure signals | 🟡 Minor gap |
| **Provider fallback chain** | Ordered fallback_providers list, auto-swap on failure | ❌ Single provider, no failover | 🟡 Nice-to-have |
| **Dynamic tool mutation (MCP)** | Listens for tools/list_changed, hot-reloads | Extension manager can enable/disable at runtime | ✅ Different approach, both work |
| **Plugin lifecycle hooks** | pre/post tool_call, pre/post llm_call, session start/end | ❌ No plugin system (extensions are the model) | 🟡 Architectural difference |
| **Trust-scored memory** | Holographic provider: helpful/unhelpful feedback loop | ❌ None | 🔴 Novel, high-value |
| **Dynamic reasoning depth** | Honcho provider adjusts reasoning level by message length | ❌ None | 🟡 Nice-to-have |
| **External memory providers** | Pluggable backends (honcho, holographic, mem0, etc.) | ❌ Single MCP memory server | 🟡 Extensibility gap |
| | | | |
| **Context Efficiency** | | | |
| **Subagent context isolation** | delegate_task runs children in separate context; parent sees only final summary JSON | ❌ All tool calls in main context | 🔴 Major gap |
| **Code execution collapsing** | execute_code runs up to 50 internal tool calls, returns only stdout (50KB cap) | ❌ Each tool call enters context individually | 🔴 Major gap |
| **Web content summarization** | Web pages LLM-summarized to 5K chars before entering context | Full content enters context (with compaction later) | 🔴 Major gap |
| **Tool result size caps** | Layered: per-tool caps (5K–50K) + global 100K hard cap | No per-tool caps; relies on compaction | 🔴 Major gap |
| **Head+tail truncation** | Terminal/code output keeps 40% head + 60% tail, drops middle | No truncation strategy | 🟡 Gap |
| **File read deduplication** | Re-reads of unchanged files return tiny stub | ❌ Full re-read every time | 🟡 Gap |
| **Large paste collapsing** | User pastes of 5+ lines collapsed to file reference | ❌ Full paste enters context | 🟡 Minor gap |
| **Compression threshold** | 50% of context window; target 10% after compression | 80% threshold; no explicit target | 🟡 Design difference |
| **Iterative compression** | Subsequent compressions UPDATE previous summary | Fresh summarization each time | 🟡 Gap |
| **Display vs context separation** | All CLI feedback (spinners, diffs, previews) is display-only, zero context tokens | Tool-pair summarization runs in background, but display separation unclear | 🟡 Minor |

---

## The Core Insight

The biggest difference isn't any single feature — it's two things working together:

**First, Hermes has autonomous knowledge management loops** running alongside the user's task:

1. **During the task**: The agent is instructed to proactively save memories when it notices preferences, corrections, or environment facts.
2. **Periodically**: A background thread reviews the conversation every N turns and extracts knowledge the inline agent missed.
3. **Before compression**: An emergency flush ensures nothing valuable is lost when context gets summarized.
4. **After complex work**: The agent reviews whether a reusable skill should be created or updated.

**Second, Hermes is obsessively efficient with context**, which is what makes everything else possible:

5. **Subagent isolation**: Heavy research/analysis is delegated to child agents whose full conversations never enter the parent's context. Only a short summary returns.
6. **Execution collapsing**: A script making 50 tool calls produces one 50KB result in context, not 50 separate tool results.
7. **Aggressive truncation**: Web pages are LLM-summarized to 5K chars. Terminal output is head+tail truncated. File re-reads return stubs. There's a global 100K char cap on any tool result.
8. **Early compression**: Triggers at 50% (not 80%), targets 10% after compression, and iteratively updates summaries rather than regenerating them.

These two systems are symbiotic — context efficiency creates headroom for knowledge management. If the agent is already at 80% context when a background review fires, it triggers compaction and potentially loses the very knowledge it's trying to extract.

Goose has none of the autonomous loops (memory is entirely user-initiated) and has minimal context efficiency measures (all tool results flow directly into the main conversation, compaction fires late at 80%).

---

## Phased Plan

### Phase 1: Autonomous Memory Extraction (Highest Impact)
_Bring Goose's existing memory system to life_

**1a. System prompt guidance for proactive memory use**
- Add memory guidance to the system.md template (similar to Hermes's MEMORY_GUIDANCE)
- Tell the agent to save when: user corrects it, shares preferences, reveals environment details
- Tell it NOT to save: task progress, session outcomes, temporary state
- **Where**: `crates/goose/src/prompts/system.md` + `prompt_manager.rs`
- **Effort**: Small (prompt-only change, no Rust code)

**1b. User profile as a memory category**
- Add a "user_profile" category to the memory MCP server with special semantics
- Profile entries always loaded into system prompt (already works for global memories)
- Encourage the agent to separate "who the user is" from "what I've learned"
- **Where**: `crates/goose-mcp/src/memory/mod.rs`
- **Effort**: Small (category convention + prompt guidance)

**1c. Memory budget and curation pressure**
- Add configurable size limits per memory category
- When near limit, the agent must curate (replace/remove low-value entries)
- This forces the memory to stay high-signal — unbounded memory becomes noise
- **Where**: `crates/goose-mcp/src/memory/mod.rs`
- **Effort**: Medium

**1d. Pre-compaction memory flush**
- Before context compaction runs, give the agent one turn with only memory tools
- Prompt: "Session is being compressed. Save anything worth remembering."
- **Where**: `crates/goose/src/context_mgmt/mod.rs` → call memory flush before `compact_messages()`
- **Effort**: Medium (needs a mini agent turn within the compaction flow)

### Phase 2: Background Knowledge Review Loops
_The agent reviews its own conversation and extracts knowledge autonomously_

**2a. Periodic memory review**
- Every N user turns, spawn a background task that:
  - Takes a snapshot of the conversation
  - Sends it to a cheap/fast model with a review prompt
  - Executes any memory tool calls the reviewer makes
- Runs AFTER the response is delivered (never competes with the user's task)
- **Where**: New module `crates/goose/src/agents/knowledge_review.rs`
- **Effort**: Medium-Large (needs async task spawning, conversation snapshot, tool dispatch)

**2b. Periodic skill review**
- After N tool-calling iterations, review whether:
  - A non-trivial approach was used that required iteration
  - The user corrected the agent's method
  - An existing skill was wrong/incomplete
- If so, create or patch a skill
- **Where**: Same module, needs skill write capability (see Phase 3)
- **Effort**: Depends on Phase 3

### Phase 3: Skill Creation & Evolution
_Skills become a living knowledge base, not just static instructions_

**3a. Skill write/patch capability**
- Add `create_skill` and `patch_skill` tools to the skills platform extension
- Validate YAML frontmatter, enforce naming conventions
- Security scan agent-generated content (no credential exfiltration, no prompt injection)
- Write to `~/.config/goose/skills/` (global) or `.goose/skills/` (local)
- **Where**: `crates/goose/src/agents/platform_extensions/skills.rs`
- **Effort**: Medium

**3b. Skill nudge in system prompt**
- After complex tasks (5+ tool calls, error recovery), the system prompt should remind the agent to consider saving a skill
- Track iteration count, inject nudge when threshold reached
- **Where**: `prompt_manager.rs` system_prompt_extras mechanism
- **Effort**: Small

### Phase 4: Smarter Retrieval & Trust
_Make memory retrieval adaptive, not just keyword-based_

**4a. Enable and improve chat recall**
- Turn chatrecall on by default
- Upgrade from SQL LIKE to FTS (SQLite FTS5 is available in Goose's SQLite)
- Add LLM-summarized results instead of raw message snippets
- **Where**: `crates/goose/src/session/chat_history_search.rs` + `chatrecall.rs`
- **Effort**: Medium

**4b. Trust-scored memories (novel)**
- Add a `trust_score` field to memories (default 1.0)
- Add a `rate_memory` tool: helpful (+0.05) or unhelpful (-0.10)
- Filter low-trust memories (<0.3) from retrieval
- The agent learns which of its stored facts are actually useful
- **Where**: `crates/goose-mcp/src/memory/mod.rs`
- **Effort**: Medium

**4c. Semantic memory retrieval**
- Current: exact keyword match or global load
- Upgrade: embed memories, retrieve by semantic similarity to current conversation
- Could use a local embedding model or external API
- **Where**: New retrieval layer in memory MCP server
- **Effort**: Large

### Phase 5: Context Efficiency (Sustaining Long Sessions)
_An agent that runs out of context can't learn anything_

The biggest context saver in Hermes is architectural: subagents and code execution sandboxes run in isolated context windows, and only short summaries flow back to the parent. This is what allows a multi-hour session to sit at 6% context usage while having read hundreds of files and made hundreds of tool calls.

**5a. Subagent context isolation**
- When delegating subtasks, spawn an independent agent with its own conversation and context window
- Return only the final response text + lightweight metadata (token counts, tool trace) to the parent
- The parent never sees intermediate tool calls, file contents, or reasoning from the child
- This is the single biggest context efficiency win — a delegation that involves 30 tool calls and 100K tokens of file reads returns maybe 500 tokens of summary
- **Where**: Goose already has subagent support via `summon` — but need to verify that intermediate tool results don't leak into the parent conversation. The key is: parent context should contain only the delegation request and the final summary, nothing in between
- **Effort**: Medium (verify current isolation, add summary-only return if not already isolated)

**5b. Code execution collapsing**
- When the agent runs a multi-step script (reading files, searching, processing), all internal tool calls should be invisible to the main context
- Only the script's final output (stdout) should enter the conversation, capped at 50KB
- Hermes's execute_code allows 50 internal tool calls that get collapsed into one result
- **Where**: `crates/goose/src/agents/platform_extensions/code_execution/`
- **Effort**: Medium (depends on current code execution architecture)

**5c. Tool result size caps (layered)**
- Add per-tool output caps with intelligent truncation:
  - **Web content**: LLM-summarize pages down to ~5K chars before entering context (this alone is huge — a 500K webpage becomes a few paragraphs)
  - **Terminal output**: Cap at 50K chars using head+tail strategy (40% head, 60% tail, truncation notice in middle) — the model sees the setup and the conclusion, not the noise
  - **File reads**: Reject reads above a configurable limit (~200K chars), tell the model to use offset/limit instead
  - **Search results**: Default limit of 50 results
  - **Global backstop**: Hard cap of 100K chars on ANY tool result
- **Where**: Each tool's result handler + a global cap in `agent.rs` tool dispatch
- **Effort**: Medium (per-tool changes, but each is small)

**5d. File read deduplication**
- Track (path, offset, limit, mtime) for recent file reads
- If the model re-reads a file that hasn't changed, return "File unchanged since last read" instead of the full content
- Reset the cache after context compaction (since the original read got summarized away)
- **Where**: `crates/goose/src/agents/platform_extensions/developer/`
- **Effort**: Small (HashMap cache + mtime check)

**5e. Smarter compression**
- Lower the compaction threshold from 80% to 50% (compress earlier, not as an emergency)
- Set an explicit target after compression (~10% of total window)
- Make compression iterative: UPDATE the previous summary rather than regenerating from scratch — this preserves accumulated context across multiple compressions
- Add pre-compaction tool result pruning: replace old tool results >200 chars with stubs BEFORE running the LLM summarizer (cheap, no API cost)
- **Where**: `crates/goose/src/context_mgmt/mod.rs`
- **Effort**: Medium

**5f. Large input collapsing**
- When users paste large blocks of text (5+ lines), save to a temp file and reference it instead of putting the full text in the user message
- **Where**: Input handling in CLI/server
- **Effort**: Small

**Why this matters for the whole plan**: Context efficiency isn't just about avoiding errors — it directly enables the knowledge management features in Phases 1-4. Background memory review needs spare context to work in. Pre-compaction flushes need room to add a flush message. If the agent is already at 80% context when these fire, they'll trigger MORE compaction and potentially lose the very information they're trying to save.

### Phase 6: Agent-Portable Standards (Cross-Agent Knowledge)
_Make this work across Goose, Hermes, and any other agent_

**5a. Shared skill format**
- Both Hermes and Goose already use SKILL.md with YAML frontmatter
- Standardize the schema: name, description, version, platform, tags
- Skills should be portable between agents
- **Where**: Spec document + validation in both codebases
- **Effort**: Small (mostly coordination)

**5b. Shared memory format**
- Define a portable memory format (structured entries with categories, trust scores, timestamps)
- Agents read/write to a common location (~/.agents/memories/)
- **Where**: Spec document + migration in both codebases
- **Effort**: Medium

**5c. AGENTS.md as the universal context protocol**
- Both already read AGENTS.md from the working directory
- Standardize what goes in it and encourage the ecosystem
- This is the "project-level memory" that any agent can read
- **Where**: Documentation/advocacy
- **Effort**: Small

---

## Priority Ranking (Impact vs Effort)

| Priority | Item | Impact | Effort | Why |
|---|---|---|---|---|
| **P0** | 1a. System prompt guidance | 🔥🔥🔥 | Small | Biggest bang for buck — just tell the agent to use memory proactively |
| **P0** | 1b. User profile category | 🔥🔥🔥 | Small | Separating "who" from "what" dramatically improves personalization |
| **P0** | 5c. Tool result size caps | 🔥🔥🔥 | Medium | Without this, every file read and web fetch bloats context — blocks everything else |
| **P1** | 5a. Subagent context isolation | 🔥🔥🔥 | Medium | The single biggest context efficiency win; verify summon already isolates |
| **P1** | 5e. Smarter compression | 🔥🔥🔥 | Medium | Lower threshold to 50%, add iterative summaries, pre-prune tool results |
| **P1** | 1d. Pre-compaction flush | 🔥🔥🔥 | Medium | Prevents knowledge loss during long sessions |
| **P1** | 3a. Skill write/patch | 🔥🔥🔥 | Medium | Skills that can't be updated from experience are static docs |
| **P1** | 4a. Enable chat recall | 🔥🔥 | Medium | Already built, just off and weak — low effort to activate |
| **P2** | 5b. Code execution collapsing | 🔥🔥🔥 | Medium | 50 tool calls → 1 result; massive context savings for scripted work |
| **P2** | 5d. File read deduplication | 🔥🔥 | Small | Prevents repeated file reads from bloating context |
| **P2** | 1c. Memory budget | 🔥🔥 | Medium | Prevents memory bloat, forces curation |
| **P2** | 2a. Background memory review | 🔥🔥🔥 | Large | Autonomous extraction is the killer feature, but complex to build |
| **P2** | 3b. Skill creation nudge | 🔥🔥 | Small | Pairs with 3a |
| **P3** | 4b. Trust-scored memories | 🔥🔥 | Medium | Novel differentiator, but needs volume to matter |
| **P3** | 2b. Periodic skill review | 🔥🔥 | Large | Depends on 3a, complex to get right |
| **P3** | 5f. Large input collapsing | 🔥 | Small | Minor but easy win |
| **P4** | 6a-c. Cross-agent standards | 🔥 | Medium | Long-term ecosystem play |
| **P4** | 4c. Semantic retrieval | 🔥🔥 | Large | Nice-to-have, depends on embedding infra |

---

## Architecture Note: MCP vs In-Process

Goose's memory is an MCP server. This is both a strength (portable, any agent can use it) and a constraint (the background review loop needs to make MCP tool calls, not just write files).

Two approaches for the background review:
1. **MCP-native**: Spawn a review task that calls the memory MCP server's tools through the normal extension manager. Clean but requires an async agent turn.
2. **Direct file write**: The review task writes directly to memory files, bypassing MCP. Simpler but breaks the abstraction.

Recommendation: **MCP-native** (option 1). It keeps the memory server as the single source of truth and means external memory servers get the same benefit automatically.

---

## What This Looks Like When Done

A user works with Goose for a week. By the end:

- Goose knows their name, role, preferred coding style, timezone
- Goose remembers they hate ORMs and prefer raw SQL
- Goose has skills for their deploy pipeline, their test framework quirks, their PR review checklist
- When Goose's memory says "user prefers Makefiles over shell scripts" and the user stops caring, the trust score decays and it stops being retrieved
- When context gets compressed in a long session, nothing important is lost because it was flushed to memory first
- Goose can search "how did we fix that Docker networking issue last month?" and find it in session history
- A complex investigation that reads 50 files and runs 30 commands uses only 5-10% of context because subagents handle the heavy lifting and only summaries return
- The agent can sustain multi-hour sessions without hitting context limits, because tool results are aggressively truncated and compression fires early

The agent feels like it *knows you* rather than starting fresh every time. And it can work all day without running out of room to think.
