# Editing a codex hook declaration untrusts it

## Symptom

After editing a hook entry in `~/.codex/hooks.json`,
the hook stops running entirely.
 No error card appears;
 the hook summary simply omits that handler,
 shadow records stop arriving,
 and the `/hooks` view shows the hook as modified.

Adding a harmless flag such as `"async": true` is enough to trigger it.

## Root cause

Codex approves a hook's normalized declaration,
not its file text and
not the script it runs.
 `hook_hash` hashes one handler's
`event_name`,
group `matcher`,
and normalized handler config
(`type`,
`command`,
`timeout`,
`async`,
`statusMessage`,
`additionalContextLimit`) as recursively key-sorted compact JSON,
then SHA-256 with a `sha256:` prefix.
 Any declaration change moves that hash,
 `hook_trust_status` reports `Modified`,
 and dispatch skips the handler until it is approved again.

Evidence read from the codex source at commit `83086188ae`
(openai/codex,
`codex-rs`):

- `hooks/src/engine/discovery.rs`,
  `hook_hash`,
  `NormalizedHookIdentity`,
  and `hook_trust_status`.
- `config/src/fingerprint.rs`,
  `version_for_toml`.
- `hooks/src/engine/discovery.rs`,
  the `handlers.push` gate that admits only `Trusted` and `Managed`.

## Workarounds

1.  Approve the hook again in the `/hooks` view,
    which writes the new `trusted_hash` into `~/.codex/config.toml`.
2.  Recompute the fingerprint yourself and write it into
    `[hooks.state."<key>"]` of `~/.codex/config.toml`.
    Validate the method first:
    reproduce the `trusted_hash` of an unchanged hook entry exactly,
    and only then write the new value.

## Reproducing the fingerprint

2026-11-01,
this recipe matched all five stored hashes of
`~/.codex/config.toml` before any edit,
and the recomputed values kept the hooks running after a flag change.

- Build the identity object for one handler:
  `event_name` is the snake_case event key,
  `matcher` is the group matcher when present,
  and `hooks` holds one normalized handler table.
- The normalized handler table carries `type: "command"`,
  the command string,
  `timeout` filled with `600` (`normalize_command_hook`'s default),
  and `async` as declared.
  `None` fields are omitted,
  never `null`.
- Serialize with every object key sorted recursively,
  compact separators,
  SHA-256 the bytes,
  hex-encode,
  prefix `sha256:`.
- The state key is
  `"<hooks.json path>:<event_key>:<group_index>:<handler_index>"`,
  matching `/hooks`.

Relevant for hook authors:
 codex's `"async": true` is a command-handler field like Claude Code's.
 Async handlers run in the background,
 their `decision` and control fields are dropped,
 and session shutdown aborts handlers still running
 (`hooks/src/engine/command_runner.rs`,
`schedule_async_hook`;
 `hooks/src/registry.rs`,
`shutdown`).
 A hook that records a final observation at `Stop` can therefore lose it
 to teardown;
 this repository's cost-dismissal `Stop` leg runs async anyway,
 by the owner's 2026-11-01 decision,
 accepting that risk for the turn-end latency;
 see `doc/research/claude-code-hooks-sync-vs-async.md` for the
 Claude Code side and its measured delivery timing.
