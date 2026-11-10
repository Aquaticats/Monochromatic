# session-start-housekeeping

Claude Code `SessionStart` hook that performs housekeeping tasks when a session starts or resumes.

## What it does

Runs these cleanup steps in parallel on every session startup or resume:

- Creates `/tmp/claude` and the per-user `/tmp/claude-<uid>` root (see [Sandbox roots](#sandbox-roots))
- Removes stale git metadata (`HEAD`,
   `config`,
   `hooks`,
   `objects`,
   `refs`) and `.claude` directories leaked into `package/*/*/dist/final/`
  and committed-plugin `package/*/*/bundle/` directories
- Removes zero-byte Claude Code sandbox sentinels (`HEAD`,
   `config`,
   `hooks`,
   `objects`,
   `refs`) from the workspace root when they are regular files
- Removes the ephemeral `.mcp.json` file from the workspace root

## Sandbox roots

Claude Code lays its sandbox out as `/tmp/claude-<uid>/<workspace>/<session>/scratchpad`,
so the trailing number is the session user's UID and differs on every host.
The hook creates `/tmp/claude` and that per-user root,
derived from the process it runs as rather than a fixed number.

## Hook configuration

```jsonc
// In .claude/settings.json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume",
        "hooks": [
          {
            "type": "command",
            "command": "ccssh",
          },
        ],
      },
    ],
  },
}
```

## Replaces

This single hook replaces five separate command hooks:

```jsonc
{ "type": "command", "command": "mkdir -p /tmp/claude" }
{ "type": "command", "command": "mkdir -p /tmp/claude-<uid>" }
{ "type": "command", "command": "rm -rf package/*/*/dist/final/{HEAD,config,hooks,objects,refs,.claude}" }
{ "type": "command", "command": "find . -maxdepth 1 -type f -empty \\( -name HEAD -o -name config -o -name hooks -o -name objects -o -name refs \\) -delete" }
{ "type": "command", "command": "rm -f .mcp.json" }
```
