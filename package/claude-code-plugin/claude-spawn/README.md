# claude-spawn

Claude Code plugin that spawns steerable child Claude Code sessions in terminal windows
with automatic result forwarding via hooks.

## Prerequisites

- [terminal-exec](../../terminal-exec/cli/) must be installed and on `PATH`
  (resolves the preferred terminal emulator on Linux/Windows)
- [Node.js](https://nodejs.org/) runtime (hooks and CLI use Node APIs)

## Installing

### From a marketplace

If this plugin is published to a marketplace:

```bash
claude plugin install claude-spawn@Monochromatic
```

Or from within a Claude Code session:

```text
/plugin install claude-spawn@Monochromatic
```

Choose a scope:

- `--scope user`:
   personal,
   all projects (`~/.claude/settings.json`)
- `--scope project`:
   team-shared via version control (`.claude/settings.json`)
- `--scope local`:
   gitignored,
   this machine only (`.claude/settings.local.json`)

### From a local directory

For development or when running from this monorepo:

```bash
claude --plugin-dir ./package/claude-code-plugin/claude-spawn
```

Or install permanently by adding to settings:

```bash
claude plugin install ./package/claude-code-plugin/claude-spawn --scope user
```

### From the Monochromatic GitHub marketplace

First add the marketplace,
 then install:

```bash
claude plugin marketplace add Aquaticat/Monochromatic
claude plugin install claude-spawn@Monochromatic
```

## Building

The hooks require a build step before first use:

```bash
mise run //package/claude-code-plugin/claude-spawn:build:js:node
```

The CLI (`src/cli.ts`) runs from raw TypeScript source via Bun and does not need building.

## Usage

The `spawn-claude` CLI launches a child Claude session in a visible terminal window:

```bash
spawn-claude "implement feature X"
spawn-claude --cwd /some/path "fix the bug"
spawn-claude --extra-arguments "--model sonnet" "refactor module Y"
```

Prints `{"spawnId":"<uuid>"}` on success.
Completed child results are injected into the parent's context automatically via hooks.

To check on a child manually,
 read `~/.claude/spawn-results/spawns/{spawnId}.json`.

## Verifying installation

After installing,
 start a new Claude Code session.
The plugin registers hooks on 6 events (SessionStart,
 Stop,
 SessionEnd,
 PreToolUse,
 PostToolUse,
 PostToolUseFailure).

Test the CLI:

```bash
spawn-claude "say hello and exit"
```

A new terminal window opens with a Claude session.
When that session ends,
 its result appears in the parent's context.

## Publishing

### To the Monochromatic marketplace

This plugin is already registered in `.claude-plugin/marketplace.json` at the repo root
under the `Monochromatic` marketplace.
 After pushing,
 users install with:

```bash
claude plugin marketplace add Aquaticat/Monochromatic
claude plugin install claude-spawn@Monochromatic
```

### To npm

1. Remove `"private": true` from `package.json` (or publish under a scope).
2. Ensure `dist/` is built:
    `mise run //package/claude-code-plugin/claude-spawn:build:js:node`
3. Publish:
    `npm publish`
4. Reference in `marketplace.json`:

   ```json
   {
     "name": "claude-spawn",
     "source": {
       "source": "npm",
       "package": "@monochromatic-dev/claude-code-plugin-claude-spawn"
     },
     "description": "Spawn steerable child Claude sessions with automatic result forwarding.",
     "version": "0.0.1"
   }
   ```

### Validating before publishing

```bash
claude plugin validate ./package/claude-code-plugin/claude-spawn
```

## How it works

The plugin has two components:
 a CLI tool and a set of hooks.

### CLI tool (`spawn-claude`)

Launches a child Claude session in a new terminal window via `terminal-exec`.
Sets `CLAUDE_SPAWN_ID` on the child to link it to a pre-created spawn state file.
Resolves the calling Claude session by walking the process tree upward,
matching ancestor PIDs against `.by-pid/` coordination files.
Falls back to the most recently modified `.by-pid/` file when the walk
fails (inside the Bash tool sandbox,
 which uses a separate PID namespace).
Returns a `spawnId` immediately without waiting for the child to finish.

### Hooks (automatic result forwarding)

Since all Claude sessions in the workspace share the same hooks,
 the child session
self-reports its state through the shared hook infrastructure:

- **SessionStart**:
   writes a PID-to-session mapping (for CLI coordination);
  claims spawn ownership by filling in `sessionId` on the pre-created spawn file
  (only if `sessionId` is empty,
   preventing stale `CLAUDE_SPAWN_ID` env vars
  from hijacking unrelated sessions);
   auto-symlinks `spawn-claude` CLI
- **Stop**:
   updates the child's `lastMessage` and sets `status: "stopped"` (child sessions,
  guarded by `sessionId` match);
   **consumes** completed children by blocking with
  `decision: "block"` + `reason` text (parent sessions)
- **SessionEnd**:
   no-op pass-through (kept for future use)
- **PreToolUse/PostToolUse/PostToolUseFailure**:
   **consumes** completed children
  and injects results via `additionalContext` (confirmed working in Claude Code v2.1.76)

Results appear in the parent at the earliest possible moment:
 the first tool-use
hook to fire after child completion delivers the result.
If the parent is stopping (no more tool calls),
 the Stop hook's blocking mechanism
delivers instead.

## Nesting

Arbitrary nesting depth is supported.
 Each session acts as both a potential parent
(checks for completed children) and a potential child (reports to its parent).
The `parentSessionId` field in spawn state files links each child to exactly
one parent using Claude Code's `session_id` from hook event data.

## File layout

All coordination files live under `~/.claude/spawn-results/`:

- `.by-pid/{claude_pid}`:
   maps Claude process PID to `{sessionId, transcriptPath}`;
  written by SessionStart hook,
   read by CLI via process tree walk
- `spawns/{spawnId}.json`:
   spawn state for each child session
- `spawns/{spawnId}.reported`:
   atomically renamed after result injection to
  prevent duplicate delivery

## Limitations

- **Sandbox incompatible**:
   `spawn-claude` must run outside the Bash tool sandbox.
  The sandbox kills detached child processes when the parent command exits,
  so the terminal window never opens.
   Claude will prompt for permission to
  bypass the sandbox on first use;
   approve it once and future invocations
  are allowed automatically
- The process tree walk reads `/proc/{pid}/status` (Linux-only);
   inside the Bash
  tool sandbox (separate PID namespace),
   it falls back to the most recently modified
  `.by-pid/` file,
   which is correct when running a single Claude session
- macOS and Windows would need platform-specific alternatives for the process tree walk
  (the most-recent fallback still works on all platforms)
- Results are delivered on the next hook event that supports `additionalContext`
  (PreToolUse,
   PostToolUse,
   PostToolUseFailure) or via Stop hook blocking;
  if the parent is completely idle with no hook activity,
   delivery waits until
  the parent's next tool call or stop attempt
