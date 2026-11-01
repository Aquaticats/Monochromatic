# Codex context and subagent settings

Status:
 accepted.
Date:
 2026-09-06.

## Scope

This decision governs user-level Codex sessions through `~/.codex/config.toml`.
It applies across repositories opened by this Codex installation.
It does not change the Claude Code policy in
[`general-purpose-subagent-ban.md`](./general-purpose-subagent-ban.md),
which concerns a different harness and tool surface.
The policies intentionally diverge:
the Claude Code decision relies on human-visible subagent monitoring,
while the user reports that Codex subagent monitorability is broken.

## Evidence gathered

The installed CLI identified itself as `codex-cli 0.153.4`.
`codex features list` reported two feature-registry entries with the exact
`experimental` maturity label:

- `network_proxy`,
  disabled;
- `prevent_idle_sleep`,
  disabled.

The same command reported 52 entries as `under development`.
OpenAI's
[feature-maturity reference](https://learn.chatgpt.com/docs/feature-maturity)
distinguishes those categories:
experimental features are unstable,
while under-development features are not ready for use.

The context feature has inconsistent maturity wording across the current surfaces.
The
[configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
calls it experimental context management,
but CLI `0.153.4` classifies the `context_management` feature as under development.
The documented behavior replaces repeated single-summary compression with notes and
searchable history.

Codex exposes two configuration-level ways to gate subagent tooling:

- `agents.enabled = false` is documented to disable multi-agent tools;
- `features.multi_agent = false` disables the multi-agent collaboration feature.

The CLI provides a persistent feature command for the second gate.
`codex features disable multi_agent` wrote the user config,
and a fresh `codex features list` readback reported `multi_agent` as `false`.

## Decision: evaluate context management

Keep `context_management = true` in the user-level Codex configuration as an
explicit evaluation.
It may help the long sessions this repository supports,
but it does not replace durable repository records.
The `DCK` rule in [`AGENTS.md`](../../AGENTS.md) remains authoritative:
requirements,
evidence,
rejected ideas,
open questions,
commits,
and the next action belong in durable documents before compaction.

This is separate from the Pi context mechanism recorded in
[`pi-context-management.md`](./pi-context-management.md).
The two settings run in different harnesses;
the Pi decision is precedent for auditing context ownership,
not evidence that the Codex feature conflicts with Pi.

## Decision: remove Codex subagent tools

Keep `multi_agent = false` under `[features]` in the user-level Codex configuration.
New Codex sessions must omit the collaboration tools used to spawn,
message,
interrupt,
or wait for subagents.

The hard feature gate is intentional.
Models had repeatedly launched unnecessary subagents,
making work harder to follow and often producing the wrong decomposition.
The user also found Codex subagents insufficiently monitorable,
so supervised delegation is not a dependable fallback.
That monitorability finding is user-observed workflow evidence;
it was not independently reproduced during this settings change.
A prompt telling models not to use an available tool is insufficient for this failure mode.
No compensating `AGENTS.md` instruction was added because the unavailable tool is the policy boundary.

The configured automatic approval reviewer is outside this decision.
This decision removes model-callable collaboration tools;
it does not change approval routing.

## Decision: leave other experimental features disabled

Keep `prevent_idle_sleep` disabled.
The user does not trust Codex's current implementation quality enough to let an
experimental feature inhibit system sleep.
Long-running-work conventions do not override that risk judgment.

Keep `network_proxy` disabled until it has a separately designed and tested domain policy.
With no domain rules,
the proxy blocks external destinations for sandboxed commands.
This repository uses GitHub,
package registries,
tool downloads,
and other external services,
so enabling the proxy without a complete allowlist would obstruct required workflows.
The proxy also does not govern hosted web search,
Apps,
or MCP traffic,
so enabling it alone would not create one complete network boundary.

Do not set `experimental_compact_prompt_file`.
Repository instructions already define the durable compaction boundary,
and a separate global prompt override would create another instruction owner.

Do not use experimental remote MCP placement without a concrete server and an audited need.

## Rejected alternatives

### Tell models not to launch subagents

Rejected because the tool remains callable and observed model behavior did not respect the intended restraint.
Tool absence is deterministic;
prompt compliance is not.

### Limit subagent concurrency

Rejected because a lower concurrency limit still permits unnecessary delegation.
It also does not repair the monitorability problem.
The requirement is no model-callable subagent tool,
not fewer simultaneous subagents.

### Enable sleep prevention for long-running work

Rejected because the convenience does not justify running experimental system-lifecycle code that the user distrusts.

### Enable the network proxy without a policy

Rejected because its default-deny behavior would block command-line network workflows,
while leaving other Codex network surfaces outside the proxy.

## Resulting configuration and verification

The relevant effective state after the changes was:

```toml
[features]
context_management = true
multi_agent = false
```

The verification readback was:

```text
context_management  under development  true
multi_agent         stable             false
network_proxy       experimental       false
prevent_idle_sleep  experimental       false
```

Existing sessions may retain the tool surface created when they started.
The user-level feature gate applies when a new Codex session initializes.

## Conflicting session observation on 2026-10-15

During research into Claude Code model restrictions,
the Codex session exposed working collaboration tools and launched research subagents.
A fresh read of `~/.codex/config.toml` still found `multi_agent = false` under `[features]`.
The user reiterated that Codex subagents are disabled.
The agent had followed delegation instructions in skills without first checking this decision.
The research agents finished and further delegation stopped.

The cause of the exposed tools is unresolved.
The configuration readback alone does not establish that this session honored the feature gate.
No Codex configuration was changed during this observation.
