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

## Restart check, closed at 2026-11-07T22:01:08-05:00

The change needed a server start and a new session,
and the check was that the injected context holds no `Instructions from: CLAUDE.md` block
while `Instructions from: AGENTS.md` is still present.
It passed.

A session created through the web profile's HTTP API with this repository as its working directory
(session `trial-real-home-3` on the real `~/.dsh`)
has one `Instructions from: AGENTS.md` header in its decompressed log,
no `Instructions from: CLAUDE.md` header,
and none of the preamble markers `question tool is unreliable`,
`spawn-claude`,
or `claude-code-bash`.
The working-directory line and the `AGENTS.md` block are the positive controls,
and both were present.

Session `trial-real-home-2`,
whose working directory was a scratch folder,
showed no instruction block at all because that folder holds neither file.
That run proved nothing about the override,
so the check needs a working directory that contains the instruction files.
How to start such a session is in `doc/agent/drive-dsh-as-a-subagent.md`.

## Upgrade to 0.2.2-rc.1, recorded at 2026-11-07T22:01:08-05:00

The user-level `~/.dsh/cordis.patch.yml` referenced a plugin subpath that 0.2.2-rc.1 no longer exports,
which stopped every session creation.
The cause,
the six-line fix,
and the backup are in `doc/troubleshooting/dsh-list-agents-subpath-removed-in-0-2-2-rc-1.md`.

In this repository,
the `@deepseek-ai/dsh-*` catalog pins in `pnpm-workspace.yaml` and the `engines.dsh` field of the
`advisor` and `neuralwatt` plugin manifests moved from 0.2.1-rc.5 to 0.2.2-rc.1,
and the `@deepseek-ai/cordis` pin moved from 4.0.5 to 4.1.0 because the install refused an unmet peer.
The type-check and unit tests of both plugins passed,
and the web profile then booted with no `skipping profile bundle` line.

## Open queue

1.  Make the `engines.dsh` field stop pinning one exact version.
    Both plugin manifests declare a single version,
    so every DSH upgrade makes DSH skip the bundles again,
    as `doc/troubleshooting/dsh-rc5-profile-startup.md` records.
    Whether a range is safe for plugins that call host APIs is the owner's call.
    Also open:
    the comments at `pnpm-workspace.yaml` about `@deepseek-ai/dsh-llm-pi-ai` and
    `@deepseek-ai/dsh-config-editor` still describe their 0.2.1-rc.2 behavior,
    and were not re-checked against 0.2.2-rc.1.
    Test sessions `trial-real-home-2` and `trial-real-home-3` remain in the real session list,
    along with the backup `~/.dsh/cordis.patch.yml.bak-20261107T214942-0500`,
    until the owner says to remove them.
2.  Collapse both search provider keys to a single home.
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
3.  Priority 7 in the audit:
    guard the `preset-cordis` override against upgrade drift.
    The override shadows all 35 shipped ids in identical order,
    so any upstream change to `presets/cordis.patch.yml` is lost silently at the
    next DSH upgrade,
    with a `MAINTENANCE` comment as the only defence.
    A repo task that diffs the shipped preset against the override and fails on
    drift would make that loud.
    The predicted failure happened on 2026-11-07:
    the user-level patch restated three presets with a `list-agents` row that 0.2.2-rc.1 removed,
    and no session could be created
    (`doc/troubleshooting/dsh-list-agents-subpath-removed-in-0-2-2-rc-1.md`).
    The row is fixed.
    The drift guard is still proposed and not taken,
    and is the owner's call.
4.  The 14 hidden workflow skills.
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
