# DSH setup hardening

## Purpose

Hands over the DSH profile hardening thread that grew out of
`doc/audit/dsh-setup-gaps-2026-10-17.md`.
The audit compared this machine's DSH setup against Pi and Claude Code,
and the work since then has been closing the items it raised.
Two of them are now closed and one is closed pending a restart.

This document is tracked and published.
Machine-local credential findings are in
`doc/audit/dsh-setup-gaps-2026-10-17.local.md`,
which matches the `*.local.*` pattern at `.gitignore:11` and is not tracked.

## What changed

### Instruction candidates

`~/.dsh/profiles/web/cordis.patch.yml` carries an override of the shipped
`cordis` agent preset.
Its `agent-instructions` child row now reads:

```yaml
# ~/.dsh/profiles/web/cordis.patch.yml, inside the `preset-cordis` override
- id: agent-instructions
  name: "@deepseek-ai/dsh-agent-instructions"
  config:
    maxBytes: 131072
    instructionFileCandidates: ["AGENTS.md"]
    localInstructionFileCandidates: ["AGENTS.local.md"]
```

The defaults are `["AGENTS.md", "CLAUDE.md"]` and
`["AGENTS.local.md", "CLAUDE.local.md"]`,
at `dsh-agent-instructions/lib/index.js:17-18`.
`diff AGENTS.md CLAUDE.md` reports `0a1,49`,
so `CLAUDE.md` is `AGENTS.md` plus a 49-line,
2029-byte Claude Code preamble with no deletions,
and the plugin's dedup cannot collapse the pair because it only matches
trimmed-content duplicates.
Three of those 49 lines' blocks are Claude Code-specific and one is false under
DSH:
the `claude-code-bash` cgroup and `systemd-run --user` rule describes Claude
Code's Bash tool rather than DSH's managed background jobs,
and the Agent-tool caveat inverts DSH's `subagent` and `spawn_teammate` result
forwarding.
Two blocks were harness-neutral,
the `pi --print` external-advisor recipe and the ban on publishing
agent-authored pages off-machine.
The owner chose to drop all 49 lines rather than relocate those two.

The override lives on the preset's child row and not on the host-level row
because `dsh-web-app` sets the host `agent-instructions` row to
`disabled: true`.

### Audit document

`doc/audit/dsh-setup-gaps-2026-10-17.md` gained a second-pass subsection
recording the measurements below,
withdrew the `morph` recommendation in its priority list because that binary is
not installed on this host,
recorded that `dsh-commands` is a code-level `CommandRuntime` registry with no
markdown drop-in,
and moved its credential findings to the local companion file.

## Evidence

`dsh --profile web --dump-config` composes the profile tree and exits without
booting,
so a patch edit can be checked in the same session that made it.
The before and after dumps differ by exactly the added keys,
under `preset-cordis` only,
with `preset-standard`,
`preset-ptc`,
and `preset-minimal` unchanged.

The loader itself was probed by importing the installed bundle and calling its
exported `discoverBaselineInstructionFiles` and `loadBaselineInstructions` once
per config.
The probe script is at `${HOME}/temp/agent/probe-instructions.mjs`,
which is scratch and not tracked.

- Shipped defaults at cap 65536:
  discovered `["AGENTS.md","CLAUDE.md"]`,
  omitted `["AGENTS.md"]`,
  rendered 35327 bytes.
- Raised cap alone:
  both discovered,
  nothing omitted,
  rendered 68231 bytes.
- Override applied:
  discovered `["AGENTS.md"]`,
  nothing omitted,
  rendered 33237 bytes,
  a 51.3 percent reduction.
- Override at the shipped 65536 cap:
  identical 33237 bytes.

The first line is the significant one.
It confirms at the code path,
not by arithmetic on file sizes,
that DSH sessions were receiving Claude Code's preamble instead of the
canonical `AGENTS.md`.
The fourth line means `maxBytes: 131072` is no longer load-bearing and survives
only as headroom.

`loadBaselineInstructionSet` is declared in the package's type surface but is
absent from the runtime export list,
so a probe written against the `.d.ts` files fails at import.
Reading the bundle's own `export` statement is the reliable way to find what is
callable.

## What to inspect after a restart

The change needs a server start and a new session.
The single check is that the injected context no longer contains an
`Instructions from: CLAUDE.md` block,
and that `Instructions from: AGENTS.md` is still present.
A session that cannot restart its own server cannot perform this check,
which is why it is still open.

## Open queue

1. Restart the DSH server,
   open a new session,
   and confirm the `CLAUDE.md` block is gone.
   Everything measurable short of that is measured.
2. Collapse both search provider keys to a single home.
   The full procedure is
   `doc/runbook/search-fetch-credential-single-home.local.md`,
   which is local-only and not tracked,
   and which also covers the key rotation that goes with it.
   The single home is
   `~/.pi/agent/extensions/pi-search-fetch.json`,
   and reaching it requires deleting the `LINKUP_API_KEY` and `EXA_API_KEY`
   exports from `~/.bashrc`,
   because `resolveApiKey` prefers the environment and would otherwise shadow
   the file.
   The same procedure swaps Claude Code's hosted `linkup` MCP server for the
   repo's stdio `search-fetch` server at global scope.
   Only the owner can perform it:
   it needs a new key from the Linkup dashboard and Claude Code stopped while
   `~/.claude.json` is edited.
3. Priority 7 in the audit:
   guard the `preset-cordis` override against upgrade drift.
   The override shadows all 35 shipped ids in identical order,
   so any upstream change to `presets/cordis.patch.yml` is lost silently at the
   next DSH upgrade,
   with a `MAINTENANCE` comment as the only defence.
   A repo task that diffs the shipped preset against the override and fails on
   drift would make that loud.
   Proposed and not taken.
4. The 14 hidden workflow skills.
   `~/.agents/skills/` has 29 `SKILL.md` files and exactly 14 carry
   `disable-model-invocation: true`,
   which DSH honours by excluding them from the model-visible catalog.
   Exposing them means writing a plugin,
   since `dsh-commands` registers commands in code.
   Settled as an audit finding rather than work.

## Owner preferences

Recorded so a later session does not re-ask them.

MCP server configuration always goes at global scope,
which is `--scope user` in Claude Code's vocabulary rather than its `local`
default,
and the user-scoped `mcpServers` map in `~/.claude.json` rather than a
project-scoped `.mcp.json`.
That also matches the repo's own policy:
the `.gitignore` comment above the `*.local.*` pattern states that `.mcp.json`
is intentionally tracked and holds only loopback endpoints with no auth,
and that secret-bearing servers belong in a genuinely local file.

Findings that could reflect poorly on the owner's practice go in a `*.local.*`
markdown file rather than a tracked one.
The split is enforced by `.gitignore:11`,
and `doc/audit/dsh-setup-gaps-2026-10-17.local.md` plus
`doc/runbook/search-fetch-credential-single-home.local.md` are the two files
this thread produced under it.
The repository is public,
so anything written to a tracked path is published.

## Rejected and deferred

- A DSH-specific third instruction file.
  Superseded by dropping the `CLAUDE.md` candidate,
  which removes the duplication and the contradiction together instead of
  outranking one with the other.
- Relocating the two harness-neutral preamble rules into `AGENTS.md`.
  Owner chose to drop them.
- Changing permissions on the credential files or moving the key out of its URL
  query parameter.
  Owner declined;
  the finding is recorded in the local file.
- Bridging more MCP servers.
  Only `mvm` is repo-owned and viable on this Linux host,
  and its CLI is already on `PATH`,
  so a bridge would add typed schemas rather than capability.
- Reverting `maxBytes` to the shipped 65536.
  The probe shows it would work,
  but the headroom costs nothing.

## Commits

`f9c5852da3` recorded the audit,
`cd43557a5e` corrected three measured errors an independent fact-check found,
`c639db54ce` added the `search-fetch` MCP bundle,
`54c0737491` recorded the web search gap and remediation,
`729c8de4d3` recorded Linkup fallback verification,
and `e80b40623b` closed the instruction budget finding and withdrew `morph`.
The commit that splits the credential findings into the local file follows this
document.

The profile patch itself is user configuration outside the repository and is not
committed anywhere.
This document and the audit are its only record.
