# claude-code-plugin-terminal-title

Claude Code hook that sets the terminal tab title to reflect current activity.

## What it shows

Tool titles use lifecycle verbs:
present-tense wording during `PreToolUse`,
past-tense wording after `PostToolUse`.

Examples:

- `Bash`:
   `✳ Running git status` while running,
  `✳ Ran git status` after completion.
- `Read`:
   `✳ Reading src/index.ts` while running,
  `✳ Read src/index.ts` after completion.
- `Edit`:
   `✳ Editing src/config.ts` while running,
  `✳ Edited src/config.ts` after completion.
- `Write`:
   `✳ Writing dist/output.ts` while running,
  `✳ Wrote dist/output.ts` after completion.
- `Grep`:
   `✳ Searching for TODO` while running,
  `✳ Searched for TODO` after completion.
- `Glob`:
   `✳ Finding *.ts` while running,
  `✳ Found *.ts` after completion.
- `Agent`:
   `✳ Running agent find endpoints` while running,
  `✳ Finished agent find endpoints` after completion.
- `WebSearch`:
   `✳ Searching web for react hooks` while running,
  `✳ Searched web for react hooks` after completion.
- `WebFetch`:
   `✳ Fetching example.com` while running,
  `✳ Fetched example.com` after completion.
- `AskUserQuestion`:
   `✳ Asking: Which library?` while running,
  `✳ Asked: Which library?` after completion.
- `TaskList`:
   `✳ Listing tasks` while running,
  `✳ Listed tasks` after completion.
- Unknown and MCP tools:
   `✳ Running mcp__weather` while running,
  `✳ Ran mcp__weather` after completion.

## Session and lifecycle examples

- `PermissionRequest`:
   `✳ Requesting permission: Bash`
- `PostToolUseFailure`:
   `✳ Failed tool: Bash`
- `SessionStart`:
   `✳ Started session: startup`
- `SessionEnd`:
   `✳ Ended session`
- `InstructionsLoaded`:
   `✳ Loaded instructions: CLAUDE.md`
- `UserPromptSubmit`:
   `✳ Received prompt: Fix the auth bug`
- `Stop`:
   `✳ Stopped agent`
- `SubagentStart`:
   `✳ Starting subagent: Explore`
- `SubagentStop`:
   `✳ Finished subagent: Explore`
- `TeammateIdle`:
   `✳ Marked idle: reviewer`
- `TaskCompleted`:
   `✳ Completed task: Fix login bug`
- `Notification`:
   `✳ Notified: Permission granted`
- `ConfigChange`:
   `✳ Updated config: project_settings`
- `WorktreeRemove`:
   `✳ Removed worktree`
- `PreCompact`:
   `✳ Compacting: auto`

## Terminal compatibility

Works with any terminal that supports OSC 0 title sequences:
Ptyxis,
Konsole,
Wezterm,
Ghostty,
iTerm2,
kitty,
and others.

## How it works

The hook receives JSON on stdin from Claude Code,
extracts a descriptive summary based on the event and its input,
replaces OSC-breaking controls with visible tokens,
byte-caps the final title payload below Ghostty's 256-byte UTF-8 title reject threshold,
and writes `\x1b]0;title\x07` directly to `/dev/tty`.

Hooks run outside the sandbox,
so `/dev/tty` is accessible.

## Setup

Install the package to your workspace root,
which makes the `cctt` bin available.
Then add the hook to `.claude/settings.local.json`:

```jsonc
// .claude/settings.local.json
{
  "hooks": {
    "SessionStart": [{ "type": "command", "command": "cctt" }],
    "InstructionsLoaded": [{ "type": "command", "command": "cctt" }],
    "UserPromptSubmit": [{ "type": "command", "command": "cctt" }],
    "PreToolUse": [{ "type": "command", "command": "cctt" }],
    "PermissionRequest": [{ "type": "command", "command": "cctt" }],
    "PostToolUse": [{ "type": "command", "command": "cctt" }],
    "PostToolUseFailure": [{ "type": "command", "command": "cctt" }],
    "Notification": [{ "type": "command", "command": "cctt" }],
    "SubagentStart": [{ "type": "command", "command": "cctt" }],
    "SubagentStop": [{ "type": "command", "command": "cctt" }],
    "Stop": [{ "type": "command", "command": "cctt" }],
    "TeammateIdle": [{ "type": "command", "command": "cctt" }],
    "TaskCompleted": [{ "type": "command", "command": "cctt" }],
    "ConfigChange": [{ "type": "command", "command": "cctt" }],
    "WorktreeRemove": [{ "type": "command", "command": "cctt" }],
    "PreCompact": [{ "type": "command", "command": "cctt" }],
    "SessionEnd": [{ "type": "command", "command": "cctt" }]
  }
}
```

Leave `WorktreeCreate` unregistered.
A command hook on that event replaces Claude Code's worktree creation and must print the new worktree's path;
`cctt` only sets a title,
so registering it there makes every worktree-isolated subagent fail to start.
Evidence:
`doc/troubleshooting/claude-code-worktree-create-hook-no-path.md`.
