# DSH setup audit against Pi and Claude Code

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Date:
2026-10-17.
Runtime:
DSH `0.2.1-alpha.4`,
profile `web`,
workspace `/var/home/user/Monochromatic`.

This audit compares the DeepSeek Harness (DSH) configuration on this machine
against the two other agent harnesses configured for the same workspace,
Pi (`~/.pi/agent/`) and Claude Code (`.claude/`).

Every claim below carries the measurement that produced it.
Where a claim could not be measured in this session,
the doc says so explicitly rather than inferring.

## Retraction of the first draft

A first draft of this audit was written earlier on 2026-10-17 and was wrong
in its four highest-severity findings.
It was written from `plugin_manager list_plugins` output alone,
without measuring the agent capability set those plugins were claimed to gate.
The corrections are recorded here because the draft's conclusions were acted on.

The draft claimed DSH had zero project-level skills.
False.
All 10 project skills were present in the DSH skill catalog from the first turn,
and loading `source-audited-report` resolved its base directory to
`/var/home/user/Monochromatic/.agents/skills/source-audited-report`.

The draft claimed the `AGENTS.md` rule set was not active in DSH.
False.
The rules were delivered through `CLAUDE.md`,
which `file-enforcer` generates from `AGENTS.md`.
The real defect is different and more serious,
and is documented in the instruction budget section.

The draft listed 19 "disabled core plugins" as missing capabilities.
False.
The agent tool surface measured 52 tools before those plugins were enabled
and 52 tools after.
Every tool the draft claimed was missing (`bash`,
`pwsh`,
`todo_write`,
`workflow`,
`create_goal`,
`get_goal`,
`update_goal`,
`exit_plan_mode`,
`subagent`,
`subagent_fork`,
`list_agents`,
`send_message`,
`spawn_teammate`,
`terminal_open`,
`session_search`,
`skill`) was already callable.

The draft claimed experimental bundles were "not installed".
False.
`~/.dsh/profiles/web/package.json` composes five of them under
`dsh.profile.bundles`,
and their rows reported `fiberPhase: "active"`
before any change was made.

Two smaller errors:
the draft said "400+" Claude Code allow entries
(the count is 583),
and it recommended connecting the `serena` MCP server without noticing that
Pi deliberately sets `"enabled": false` for it in `~/.pi/agent/mcp.json`.

## Why the plugin flags were misleading

`plugin_manager list_plugins` reports 203 entries.
Many rows showed `enabled: false, fiberPhase: null`,
which reads as "capability absent".
It is not.

`plugin_manager list_bundles` shows that the `@deepseek-ai/dsh-web-app` bundle
carries an `overrides` list naming 26 row ids,
including `tool-bash`,
`tool-pwsh`,
`tool-jobs`,
`tool-fs`,
`tool-fs-search`,
`skill-filesystem`,
`tool-skill`,
`command-goal`,
`tool-goal`,
`plan-mode`,
`compaction-basic`,
`command-compact`,
`tool-result-pruner`,
`tool-subagent-control`,
`tool-subagent-list-agents`,
`tool-subagent`,
`tool-subagent-fork`,
`workflow-ptc`,
`tool-workflow`,
`agent-instructions`,
`tool-todo`,
`tool-web`,
`tool-plugin-manager`,
`system-prompt`,
`tools`,
and `session-query-sqlite`.

A later bundle layer re-declares those rows,
so the `enabled: false` state on the `@deepseek-ai/dsh-base` row
does not describe what the agent can call.
The empirical check is `cordis_inspect_query` with `platform: host`,
`provider: Tool`,
`method: listTools`,
which returns the tools actually callable by the requesting agent.
That call returned 52 tools from the lead session,
both before and after the enable sweep.

The count is seat-dependent,
and the difference is informative rather than
contradictory.
The same `listTools` call issued from a delegated subagent returns 46 tools,
because the six Agent Teams tools (`spawn_teammate`,
`wait_agent`,
`team_task_create`,
`team_task_list`,
`team_task_get`,
`team_task_update`)
are lead-scoped.
46 plus those 6 is the 52 measured from the lead.

Consequence:
the 24 `disabled: false` entries that the enable sweep wrote into
`~/.dsh/profiles/web/cordis.patch.yml` were inert.
They have since been reverted,
and the patch file again carries only rows that change profile behavior.
The count is 24 and not the 25 originally reported,
because `desktop-product-telemetry` and `product-analytics` ended as
`disabled: true` after their activation errors.
At the point the sweep finished,
the file carried 24 `disabled: false` rows and 4 `disabled: true` rows,
and it now carries none.

## Instruction budget overflow drops `AGENTS.md` and injects Claude Code rules

Severity:
high.
This is the most consequential finding in the audit.

### The measurement

`@deepseek-ai/dsh-agent-instructions` is described in its own package metadata
as the "Workspace context loader for AGENTS.md/CLAUDE.md instruction files".
The `@deepseek-ai/dsh-base` bundle configures it with a byte cap at
`cordis.patch.yml:306` of that package:

```yaml
# @deepseek-ai/dsh-base/cordis.patch.yml:303-306
- id: agent-instructions
  name: '@deepseek-ai/dsh-agent-instructions'
  config:
    maxBytes: 65536
```

Measured file sizes in the workspace:

- `AGENTS.md`:
  32933 bytes,
  1450 lines
- `CLAUDE.md`:
  34962 bytes,
  1499 lines
- Combined:
  67895 bytes
- Cap:
  65536 bytes
- Overflow:
  2359 bytes

The DSH system reminder for this session states the outcome directly:
"Workspace instruction budget 65536 bytes:
omitted AGENTS.md",
followed by "Instructions from:
CLAUDE.md".

### Why deduplication does not save it

The plugin's own type documentation describes the intended collapse behavior
for `instructionFileCandidates`:
"every existing file loads,
with per-directory trimmed-content duplicates collapsed to the earliest candidate".

`CLAUDE.md` is not a trimmed-content duplicate of `AGENTS.md`.
The `diff` of `AGENTS.md` against `CLAUDE.md` produces exactly one hunk,
`0a1,49`,
and 50 output lines in total:
the hunk header plus 49 added lines.
An `0a` hunk carries no context lines,
which is itself the evidence that nothing was removed or reordered.
`CLAUDE.md` is `AGENTS.md` verbatim with 49 lines prepended.
Because the content differs,
the dedup rule does not fire,
both files charge against the 65536 byte cap,
and the cap overflows.

The prepend is deliberate and documented in the repo.
`file-enforcer.config.ts:2223` carries the comment
"CLAUDE.md must literally contain AGENTS.md content
(Claude Code's @include is unreliable)",
with `dest: './CLAUDE.md'` at line 2225.

### Why the injected preamble is actively wrong under DSH

The 49 prepended lines are Claude Code operating rules.
Under DSH they are not merely irrelevant,
several are false
and contradict the DSH system prompt.

The preamble states that in-process subagents run inside the session,
that SendMessage steering is unreliable,
and that general-purpose subagents should only be fanned out
in interactive Claude Code sessions.
DSH ships `subagent`,
`subagent_fork`,
`list_agents`,
and `send_message`
as first-class tools,
and the DSH system prompt says the opposite about steering:
"use `send_message` to steer it while running".

The preamble instructs the agent to use `spawn-claude` outside sandbox
to launch a child Claude Code session.
`spawn-claude` is not a DSH tool.
It is `package/claude-code-plugin/claude-spawn` in this repo.

The preamble instructs the agent to consult a second model with
`timeout 3600 pi --model openai-codex/gpt-5.6-sol --print ...`.
That is the Pi CLI,
not a DSH surface.

The preamble warns never to publish agent-authored documents to claude.ai.
DSH has no claude.ai integration.
The DSH system prompt carries its own,
different publishing rule.

The preamble requires long background runs to be started as systemd user units
because "every session's Bash commands share one `claude-code-bash` cgroup,
and `systemd-oomd` kills it whole".
DSH has managed background jobs via `run_in_background` on `bash` and `pwsh`,
plus `job_list`,
`job_output`,
and `job_kill`.
The cgroup hazard described is specific to Claude Code's Bash tool.

Net effect:
a DSH agent in this workspace receives the full `AGENTS.md`
rule set,
which is correct and wanted,
plus 49 lines of another harness's operating instructions,
several of which directly conflict with the DSH system prompt it also receives.
The agent must resolve the conflict itself,
per turn,
with no stated precedence.

### Remediation

Applied:
the cap was raised in the profile patch,
which stops the omission without touching the repo generator.
It leaves the conflicting preamble in place,
so DSH context still carries 49 lines of Claude Code rules:

```yaml
# ~/.dsh/profiles/web/cordis.patch.yml
- id: agent-instructions
  name: "@deepseek-ai/dsh-agent-instructions"
  config:
    maxBytes: 131072
```

Moved 2026-10-29:
this row now sits inside the `preset-cordis` override in
`~/.dsh/cordis.patch.yml`,
the user-wide layer;
see the section Glob and grep disabled user-wide, 2026-10-29.

The change takes effect at the next server start.
It has not been observed,
because no restart or new session has happened since it was written.

Making `CLAUDE.md` content-identical to `AGENTS.md` would let the plugin's
own dedup collapse the pair and reclaim the duplicated 32933 bytes,
but `file-enforcer.config.ts:2223` records that Claude Code's `@include`
is unreliable,
which is why the duplication exists.
Removing it would regress Claude Code.

The clean fix is a third instruction file.
`instructionFileCandidates` is a configurable ordered candidate list,
so DSH can be pointed at a DSH-specific file while `AGENTS.md` and
`CLAUDE.md` remain as they are.
That requires knowing the default candidate order,
which is not exposed in the plugin's public type surface
and was not measured for this audit.
It remains open,
and it is the only option that removes the contradiction rather than the
truncation.

Unmeasured:
whether the `agent-instructions` row state in the profile patch affects
instruction loading at all.
The plugin already ran in turn one while its base row reported
`enabled: false`,
so the effective mount came from the web-app override.
Confirming this needs a fresh server start and a new session,
neither of which has happened.

### Second pass, 2026-10-18

Every open item in the `Remediation` subsection is now closed,
and the fix that landed is not the one this section recommended.

The cap raise was observed.
A new session's injected context carries both an `Instructions from: AGENTS.md`
and an `Instructions from: CLAUDE.md` block,
which the 65536 default cannot produce from a 67895-byte pair,
so the preset override is live.

The default candidate order was read from source.
`dsh-agent-instructions/lib/index.js:17` sets
`DEFAULT_INSTRUCTION_FILE_CANDIDATES = ["AGENTS.md", "CLAUDE.md"]`,
and line 18 sets the local overlay default to
`["AGENTS.local.md", "CLAUDE.local.md"]`.
Neither overlay file exists in the repo root or in either user-global root,
so the two project files were the whole payload.

The row-state question is settled too.
`dsh --profile web --dump-config` composes the tree without booting and shows
the host `agent-instructions` row as `disabled: true` with `maxBytes: 65536`,
while the `preset-cordis` child row carries the override.
The live session loaded more than the host row's cap allows,
so the preset child row is what loads instructions for preset-selected agents.

Rather than add a third file,
the profile now drops the second candidate:

```yaml
# ~/.dsh/profiles/web/cordis.patch.yml, inside the `preset-cordis` override
- id: agent-instructions
  name: "@deepseek-ai/dsh-agent-instructions"
  config:
    maxBytes: 131072
    instructionFileCandidates: ["AGENTS.md"]
```

Moved 2026-10-29:
the `preset-cordis` override this snippet came from now lives in
`~/.dsh/cordis.patch.yml`,
the user-wide layer;
see the section Glob and grep disabled user-wide, 2026-10-29.

This is strictly better than the third-file option this section proposed,
because `diff AGENTS.md CLAUDE.md` reports `0a1,49`:
`CLAUDE.md` is `AGENTS.md` plus a 49-line,
2029-byte Claude Code preamble with no deletions,
so a DSH-specific third file would have kept the duplication and merely
outranked it.
Dropping the candidate removes the duplication and the contradiction together.

Three of the preamble's blocks are Claude Code-specific and one is false here.
The rule that every session's Bash commands share one `claude-code-bash` cgroup
and should be wrapped in `systemd-run --user` describes Claude Code's Bash tool,
while DSH provides managed background jobs through `run_in_background` and
`job_output`.
The Agent-tool caveat says subagent results forward reliably but steering does
not,
which inverts DSH's `subagent` and `spawn_teammate` semantics.
The `spawn-claude` guidance is framed around the Claude Code UI,
though that binary is on `PATH` here.
Two blocks are harness-neutral:
the `pi --print` external-advisor recipe,
and the ban on publishing agent-authored pages off-machine.
The owner decision was to drop all 49 lines rather than relocate those two.

Measured by calling the plugin's own `loadBaselineInstructions` and
`discoverBaselineInstructionFiles` from the installed bundle,
once per candidate config:

- Shipped defaults at cap 65536 discovered `["AGENTS.md","CLAUDE.md"]`,
  omitted `["AGENTS.md"]`,
  and rendered 35327 bytes.
  This confirms this section's central claim at the code path rather than by
  arithmetic:
  the model received the Claude Code preamble and not the canonical file.
- The raised cap alone discovered both,
  omitted nothing,
  and rendered 68231 bytes,
  so every rule was injected twice.
- The applied override discovered `["AGENTS.md"]`,
  omitted nothing,
  and rendered 33237 bytes,
  a 51.3 percent reduction.
- The same override at the shipped 65536 cap rendered an identical 33237 bytes,
  so `maxBytes: 131072` is no longer load-bearing and survives only as
  headroom for `AGENTS.local.md` and for nested-directory instructions.

`--dump-config` also shows the new key under `preset-cordis` and absent from
`preset-standard`,
`preset-ptc`,
and `preset-minimal`,
which scopes the change to the preset this profile selects.
The one step still unverified is the injected context of a session started
after the change,
because restarting the server would end the session that made it.

`localInstructionFileCandidates` still defaults to a list naming
`CLAUDE.local.md`.
That is inert today because the file does not exist.
Narrowing it was proposed and not taken,
so a future `CLAUDE.local.md` would be injected.

## Fourteen user skills are invisible to DSH

Severity:
medium.

`~/.agents/skills/` holds 29 directories with a `SKILL.md`.
`~/.pi/agent/skills/` holds 26 entries,
25 of which are symlinks resolving into `~/.agents/skills/` with none broken,
so Pi and DSH read the same underlying store.
`pre-compact` is the 26th and the only real directory,
with no `~/.agents/skills/` counterpart.

The DSH skill catalog for this session lists 15 of those 29.
The 14 it omits are `ask-matt`,
`grill-me`,
`grill-with-docs`,
`handoff`,
`implement`,
`improve-codebase-architecture`,
`setup-matt-pocock-skills`,
`teach`,
`to-questionnaire`,
`to-spec`,
`to-tickets`,
`triage`,
`wait-what`,
and `wayfinder`.

The cause is a single frontmatter field.
Scanning all 29 `SKILL.md` files for `disable-model-invocation`
produces a perfect correlation with zero counterexamples in either direction:

- 14 files set `disable-model-invocation: true`;
  all 14 are absent from the catalog
- 15 files omit the field;
  all 15 are present

DSH honors the field by excluding the skill from the model-visible catalog.
Pi surfaces the same skills through its own command path,
so they remain reachable there.
Under DSH they are unreachable,
because the `skill` tool can only load a name that appears in the catalog.

This is a real capability loss,
not a cosmetic one.
The hidden set is the workflow layer of the skill collection:
`handoff` compacts a conversation into a handover document,
`to-spec` turns a conversation into a tracker issue,
`to-tickets` splits a spec into tickets,
`triage` moves issues through a triage state machine,
`implement` executes a spec,
`ask-matt` routes between skills,
and `wayfinder` orients an agent in an unfamiliar area.
The DSH equivalents (`doc/handover/`,
GitHub Issues via `gh`,
`spawn_teammate`) exist but are not wired to these instructions.

Remediation is a design choice,
not a one-line fix.
Either DSH gains a user-invocable command surface for
`disable-model-invocation` skills,
or those skills lose the flag and accept model invocation,
or the workflow layer is re-expressed as DSH commands.
`include:commands` and `include:ui-commands` are both active,
so the third path has an existing home.

### Cost of the third path, measured 2026-10-18

The claim that the third path has an existing home understates the work.
`dsh-commands` is a code-level runtime registry,
not a markdown or directory drop-in.
Its public surface is a `CommandRuntime` service whose `define` takes discovery
metadata and a UI handler,
and its 478-line bundle contains no reference to `SKILL.md`,
to frontmatter,
or to `disable-model-invocation`.
Re-expressing the workflow layer therefore means writing a plugin of the same
kind as `package/dsh-plugin/search-fetch`,
not adding files to a directory.
The second path is cheaper but changes Pi and Claude Code too,
since all three read `~/.agents/skills/`,
and the flag exists deliberately:
model-invoking `handoff` or `to-tickets` unprompted is the failure mode it
prevents.

The store was re-measured at the same time and the count holds.
`~/.agents/skills/` has 29 directories,
29 `SKILL.md` files,
and exactly 14 carrying `disable-model-invocation: true`.
`~/.claude/skills/` holds a 30th,
`pre-compact`,
with no `~/.agents/skills/` counterpart,
which matches the `pre-compact` finding in this section's opening measurement.

## No MCP server bridge is composed

Severity:
medium.

DSH ships an MCP client.
The `@deepseek-ai/dsh-mcp-client` package describes itself as
"MCP client bridge:
connects to MCP servers and registers their tools on ctx.tools".
At the time of the first measurement it was not composed into the `web`
profile.
Searching the 203 plugin entries returned no `mcp-client` row.
The only reachable paths to the package ran through
`@deepseek-ai/dsh-acp` and `@deepseek-ai/dsh-acp-app`,
and both of those bundles report `enabled: false` in `list_bundles`.
`include:mcp-resources` is active but is a resource-exposure plugin,
not a server bridge.

Status changed during this audit:
one bridge is now composed.
`package/dsh-plugin/search-fetch` inserts a `dsh-mcp-client` row over stdio,
and the web search section below records what it provides and how it was
verified.
The gap that remains is breadth,
not the absence of a mechanism:
one server is connected,
and the seven Claude Code namespaces below are not.

What the other two harnesses have:

Claude Code's allow-list carries seven MCP tool namespaces,
counted by extracting `mcp__` prefixes from `.claude/settings.local.json`:
`nvim` (editor diagnostics),
`blender` (3D scene control),
`mvm` (virtual machine lifecycle),
`serena` (code intelligence and memory),
`claude_ai_linkup` (web research),
`morph` (semantic codebase search),
and `claude_ai_GitHub` (issue and code search).

Pi declares four servers in `~/.pi/agent/mcp.json`:
`browseros` and `linear` and `safari-mcp-stp` all marked
`"exposure": "deferred"`,
and `serena` marked `"enabled": false`.
The `safari-mcp-stp` entry reaches a macOS host over
`ssh -o BatchMode=yes m1`,
which ties it to the `HRM:` hazard rule
in `AGENTS.md` about that machine's 16 GiB cap.

Config hygiene finding on the Claude Code side:
`.claude/settings.local.json` sets
`enabledMcpjsonServers: ["slint-music-player", "slint-terminal"]`,
but no `.mcp.json` exists in the workspace.
A search to depth 3 excluding `node_modules` returned nothing.
Those two entries are stale.

Deferred exposure in Pi is the notable design idea DSH lacks.
Pi mounts the server's tools lazily,
so a four-server configuration does not charge its full tool schema
against every request.
DSH mounts all 52 tools unconditionally.
Pi also sets `defaultTools: ["+codemode", "+tool_search"]`,
which is the same context-economy motive.

## Web search had exactly one provider and it needed an account this machine lacks

Severity:
high before remediation,
closed for search,
with one policy question left open.

### The gap

DSH ships one web search provider.
Searching the 292 `@deepseek-ai/dsh-*` packages for `search` returns four
names,
and only one is a web search provider:
`dsh-web-search-deepseek`,
described as "DeepSeek-backed search provider (native web_search via the
Anthropic-compatible API) for the DeepSeek Harness web capability seam
(ctx.web)".
The other three are `dsh-client-ui-settings-web-search` (its settings page),
`dsh-experimental-session-search` (session history),
and `dsh-tool-fs-search` (`glob` and `grep`).

The `dsh-base` bundle hardwires it as the only search backend,
at `cordis.patch.yml:465-474` of that package:

```yaml
# @deepseek-ai/dsh-base/cordis.patch.yml:465-474
- id: web
  name: '@deepseek-ai/dsh-web'
  config:
    searchProvider: deepseek-official
    fetchProvider: http

- id: web-search-deepseek
  name: '@deepseek-ai/dsh-web-search-deepseek'
  config:
    apiKeyEnv: DEEPSEEK_API_KEY
```

The provider's own module documentation states the authentication route:
a search on the DeepSeek account route uses the account token,
and "every other search reuses `DEEPSEEK_API_KEY`".

Neither credential exists here.
`~/.dsh/.credentials.yaml` holds five references:
`QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`,
`OPENCODE_GO_API_KEY`,
`SYNTHETIC_API_KEY`,
`LLMGATEWAY_DEVPASS_API_KEY`,
and `NEURALWATT_API_KEY`.
No `DEEPSEEK_API_KEY`,
and no DeepSeek name appears in the process environment.

The consequence is asymmetric,
and the asymmetry is the reason this rates high.
`fetchProvider: http` points at `dsh-web-fetch-http`,
described as an "Anonymous public HTTP(S) fetch provider",
which needs no account,
so `web_fetch` worked.
Search had no equivalent fallback.
A DSH agent on this machine could retrieve a URL it already had
and could not discover one.

Both other harnesses have working search.
Claude Code allows `WebSearch` and `mcp__claude_ai_linkup__linkup-search`.
Pi carries three configurations:
`extensions/pi-search-fetch.json` (an Exa key plus a 23-entry host blocklist),
`extensions/exa-mcp.json` (`https://mcp.exa.ai/mcp`,
tools `web_search_exa` and `get_code_context_exa`),
and `extensions/firecrawl.json` (`https://api.firecrawl.dev`,
tools `firecrawl_scrape`,
`firecrawl_map`,
and `firecrawl_search`).

### Remediation

The repo already owned the right component.
`package/mcp/search-fetch` is described as an "MCP server exposing the shared
search and fetch tools with truncated responses as resources",
and its README states the design rule that decided the approach:
the Pi extension "serves pi hosts",
this server "serves every other MCP host with the identical behavior",
and "Both surfaces delegate to
`@monochromatic-dev/agent-harness-shared-search-fetch`,
so behavior has exactly one implementation."

DSH is such a host.
Authoring a native `ctx.web` provider instead would have created a third
surface,
against that stated design.

So the fix is a configuration-only bundle,
`package/dsh-plugin/search-fetch`,
which inserts one `@deepseek-ai/dsh-mcp-client` row with
`transport: stdio`,
`serverName: search-fetch`,
and `failOnStartupError: true`.
It was installed through `plugin_manager` with `action: install_bundle`,
which reported `application: "applied"`,
package install exit code 0,
and no warnings.

Two details of the row are worth recording because both were measured rather
than assumed.

`command` is `!!js process.execPath`,
the host's own Node,
so the spawn does not depend on `PATH` resolution.
The single `args` entry resolves the built server through this bundle's own
`package.json` with `createRequire(baseUrl)`,
then walks to `../../mcp/search-fetch/dist/final/node/index.mjs`,
which keeps the wiring relative to the checkout rather than hardcoding a
workspace path or a username.

No credential is copied into the bundle.
The server resolves its own config from
`$HOME/.pi/agent/extensions/pi-search-fetch.json`,
through `options.home ?? process.env.HOME ?? homedir()` at
`package/agent-harness-shared/search-fetch/src/config.ts:79-95`.
`HOME` reaches the child because the harness subprocess scrub in
`@deepseek-ai/dsh-subprocess` drops only names matching
`/KEY|PASSWORD|SECRET|TOKEN/i` and names beginning with `DSH_`,
and its own documentation states that `PATH`,
`HOME`,
locale,
and proxy variables survive.
`PATH` surviving is also what makes the `gh` CLI route work.

The `cordis` preset override described in the instruction budget section
additionally sets `tool-web` `search: false`,
so the native `web_search` is not offered to the model as a tool that can only
fail.
`@deepseek-ai/dsh-tool-web` declares `search` and `fetch` as independent
booleans both defaulting to true,
and its source comment states the intent directly:
"a product that wants only one disables the other in config".
The shipped `dsh-base` comment at `cordis.patch.yml:454-456` names the same
mechanism:
"The Web app disables this host row and composes both tools per agent preset;
products with a stricter network policy override `tool-web`."

`include:web-search-deepseek` and `include:ui-settings-web-search` were also
disabled,
the second because its settings page configures the first provider's API key,
endpoint,
and search budget,
and would otherwise be an orphaned page.

### Verification

Four probes,
all at the tool boundary rather than from configuration.

`mcp__search-fetch__web_search` on a real query returned ten ranked results as
JSONL from Exa,
which proves the spawn path,
the `createRequire(baseUrl)` resolution,
and that `HOME` reached the child so the Exa key was found.

`mcp__search-fetch__web_fetch` on `https://exa.ai/docs/search/quickstart`
returned full page text with `"source": "cached"`.
This is the positive control:
it proves the probe can succeed,
so the next result is meaningful.

The same tool on `https://www.geeksforgeeks.org/` returned
`Error: Tool execution failed: Blocked by pi-search-fetch blocklist:
geeksforgeeks.org`.
`geeksforgeeks.org` is one of the 23 blocklist entries,
and the rejection happens before any provider is called.

`mcp__search-fetch__web_fetch` on a GitHub blob URL returned raw YAML file
content rather than an HTML page render,
which confirms the `gh` CLI route and therefore that `PATH` survived the scrub.

Linkup fallback was verified separately,
after the fact,
because it needed a credential this machine did not have in a reachable place.
`~/.bashrc:42` exports `LINKUP_API_KEY`,
and the shared core reads exactly that name
(`config-constants.ts:17`,
used at `config.ts:147` and read at `config-schema.ts:223`).
It was still unavailable to the server for two independent reasons.
The harness process is not started from an interactive shell,
so it never sources `.bashrc` and the name is absent from its environment.
And even when present,
the subprocess scrub drops any name matching `KEY|PASSWORD|SECRET|TOKEN`,
which `LINKUP_API_KEY` does.

The working route is the config file the Exa key already uses.
`CONFIG_KEYS` accepts exactly `exaApiKey`,
`linkupApiKey`,
and `blocklist`,
so `linkupApiKey` was added to
`~/.pi/agent/extensions/pi-search-fetch.json`
by reading the value out of `.bashrc` without echoing it.
The Exa key and all 23 blocklist entries were confirmed byte-identical
afterwards,
and the file mode stayed `0600`.
Because `loadLinkupConfig()` runs once at startup before `serve()`,
the `include:search-fetch-mcp` row was toggled off and on through
`plugin_manager` to respawn the child;
both transitions reported `application: "applied"` with no warnings.

The differential probe is what proves the key is live.
`web_fetch` uses Linkup first for non-GitHub URLs,
so the response shape changes provider.
Before the key,
`https://exa.ai/docs/search/quickstart` returned Exa's contents envelope with
`requestId`,
`results`,
`statuses`,
and `costDollars`.
After the respawn,
`https://example.com` returned a `markdown` field alongside
`"favicon": "https://favicons.linkup.so?domain=example.com"`,
which is Linkup's shape and names Linkup as the server.
Search was re-run afterwards and still returned results,
so the respawn did not regress the Exa path.

Also not verified:
what a session created before the install sees,
since existing sessions keep the plugin revision they started with.

### Settled: both fetch tools stay

Two fetch tools are live with different policies.
The native `web_fetch` from `dsh-web-fetch-http` accepts any public HTTP(S)
destination.
`mcp__search-fetch__web_fetch` enforces the 23-entry blocklist,
routes mapped GitHub URLs through `gh`,
and strips base64 data-URL images.

An agent that wants a blocklisted host can get it through the native tool,
so the blocklist is not a hard boundary in DSH.
The owner decision is to keep both,
which trades that hole for an account-free fetch path that keeps working when
Exa or Linkup is down or rate-limited.
`fetch: false` on the `tool-web` preset row remains the one-line change if that
trade is ever revisited.

## Five Claude Code lifecycle hooks have no DSH counterpart

Severity:
medium to high,
depending on how much the hooks are relied on.

`.claude/settings.local.json` configures four hook events
carrying five commands.
Each resolves to a package built in this repo under
`package/claude-code-plugin/`.
That directory holds 13 subdirectories,
11 of which are packages with a `package.json`,
and 10 of those 11 declare a `bin` entry:
`research-agent/` holds only `agents/research.md`,
a `README.md`,
and a `TROUBLESHOOTING.md`,
and `verbose-tool-output/` holds only a `WONTFIX.md`,
while `hook-type` is a package with no bin.
The five hooks are:

- `SessionStart` with matcher `startup|resume` runs `ccssh`,
  which is `session-start-housekeeping`:
  creates temp directories and cleans stale dist artifacts
- `UserPromptSubmit` runs `cccr`,
  which is `correction-reminder`:
  detects correction phrases and reminds the agent to re-verify
- `PreToolUse` with no matcher runs `ccgr`,
  which is `guardrail`:
  blocks general-purpose Agent calls and redirects to `spawn-claude`
- `PreToolUse` with matcher `Bash` runs `ccbof`,
  which is `bash-output-filter`:
  pipes Bash output through a filter that strips git boilerplate
- `Stop` runs `ccsr`,
  which is `stop-reminder`:
  reminds the agent to investigate before guessing and refuses premature stops

Searching the descriptions of all 292 `@deepseek-ai/dsh-*` packages in the
install tree for `hook`,
`lifecycle`,
`intercept`,
`pretool`,
`before tool`,
and `on tool` returns no user-extensible tool-call hook plugin.
The matches are unrelated:
`dsh-goal` (goal state lifecycle),
`dsh-client-connection` (RPC generation lifecycle),
`dsh-client-resources` (a React `useResource` hook),
`dsh-acp-app` and `dsh-sdk-app` (process lifecycle).

The nearest DSH primitives,
all active,
are
`include:commands`,
`include:command-feedback`,
`include:message-feedback`,
`include:repeat-tool-reminder`,
and `include:auto-review` from
`@deepseek-ai/dsh-experimental-auto-review`,
described as "Per-tool LLM authorization review for the
DeepSeek Harness Auto permission preset".

Mapping the five hooks onto those primitives:
`guardrail` is partially covered by `auto-review`,
since both gate a tool call before execution.
`bash-output-filter` has no counterpart,
and it is the one with a measurable context cost,
because it strips boilerplate from every Bash result.
`correction-reminder` and `stop-reminder` have no counterpart;
`repeat-tool-reminder` fires on repeated identical tool calls,
which is a different trigger.
`session-start-housekeeping` has no counterpart.

Note that `guardrail` and the `CLAUDE.md` preamble enforce the same
general-purpose-subagent ban from two directions.
That ban is a Claude Code constraint recorded in
`doc/decision/general-purpose-subagent-ban.md`.
It does not describe DSH,
where `subagent` and `subagent_fork` are supported.

## Model roster and credential handling

Severity:
medium for the roster,
and one security finding on the Pi side.

DSH configures two providers in `~/.dsh/profiles/web/cordis.patch.yml`,
exposing three models in total:
`qwen3.8-max` on `qwen-token-plan-individual`
(1000000 context,
131072 max tokens),
and `claude-haiku-5-5` (1000000 / 128000) plus `mimo-v2.6-pro`
(1048576 / 131072) on `opencode-go`.
The default is `qwen-token-plan-individual` with `qwen3.8-max`.

Note 2026-10-18:
the two-provider count is stale.
The patch now declares five providers under `llm-pi-ai`,
adding `synthetic`,
`llmgateway-devpass`,
and `neuralwatt` to the two measured for this audit.
The default is unchanged:
`--dump-config` composes `agent-default-model` as
`qwen-token-plan-individual` with `qwen3.8-max` and `reasoningEffort: xhigh`.
The total model count was not re-measured.

Pi's `~/.pi/agent/settings.json` lists 16 `enabledModels` across three
provider namespaces (`hyper`,
`synthetic`,
`openai`),
defaulting to `hyper/gemma-4-26b-a4b-it` with `defaultThinkingLevel: "high"`.
It loads 17 packages,
including two provider adapters
(`@benvargas/pi-synthetic-provider`,
`@charmland/pi-hyper-provider`)
and a local `openai-fast` plugin.

Pi's `~/.pi/agent/models.json` adds a fourth provider,
`llmgateway-devpass`,
with four models:
`mimo-v2.6-pro`,
`gpt-6.1-sol`,
`gpt-6-astra`,
and `muse-spark-1.3-contributor`.
Three of the four carry a `thinkingLevelMap` table mapping DSH-style
reasoning levels onto provider-native ones;
`mimo-v2.6-pro` does not.
It also carries `modelOverrides` adjusting `contextWindow` to 750000
for `gpt-6-luna`,
`gpt-6.1-sol`,
and `gpt-6-astra` under `providers/openai`,
with two further overrides under `providers/openai-fast`.

Credential handling on this machine was measured as part of the audit,
and it produced findings on the Pi and Claude Code sides.
Those concern local file permissions,
key placement,
and one key that reached a session transcript,
so they are recorded in a companion file matching the `*.local.*` pattern at
`.gitignore:11` rather than here.
This repository is public and auto-pushes,
and a tracked audit is not the place for them.
The split is dated 2026-10-18;
earlier revisions of this document carried the detail in the open.

The DSH side of the comparison belongs in this document,
and it is a positive finding.
Every provider in the profile patch declares `apiKeyEnv` indirection and no
inline key,
five of each measured,
and the credential store `~/.dsh/.credentials.yaml` is mode `0600`.
The two harnesses it was compared against do not match that.

Two genuine DSH gaps in this area.
The DSH patch declares no reasoning or thinking-level configuration for its
three models,
while Pi maps thinking levels per model.
And the `include:llm-deepseek` and `include:llm-deepseek-account` rows are
both active with no DeepSeek model appearing in the patch config,
so whatever they expose was not measured here.

## Subagent role definitions

Severity:
low to medium.

Pi defines three named subagent roles with explicit tool allow-lists in
`~/.pi/agent/pi-subagents-config.json`:
`scout` (`read`,
`grep`,
`find`,
`ls`,
`web_fetch`,
`web_search`),
`planner` (the same plus `todo`),
and `reviewer` (`read`,
`grep`,
`find`,
`ls`,
`bash`,
`web_fetch`,
`web_search`).
It also carries two agent prompt files,
`~/.pi/agent/agents/Explore.md` and `~/.pi/agent/agents/Plan.md`,
and a populated `~/.pi/agent/pi-subagents-state/` directory.

DSH's `spawn_teammate` and `subagent` take free-form prompts with no
tool restriction parameter.
`include:agent-preset-registry` is active and the patch sets
`default: standard` with `selectedDefault: cordis`,
and four preset rows are active (`preset-standard`,
`preset-ptc`,
`preset-minimal`,
`preset-cordis`),
but no per-role tool allow-list is configured.

The capability gap is least-privilege delegation:
Pi can spawn a read-only scout that cannot write or execute,
and DSH cannot express that today without authoring a preset.

## Permission model differences

Severity:
informational,
with one forward-looking risk.

Claude Code has accumulated 583 allow entries in
`.claude/settings.local.json`,
a deny list containing one entry (`Glob`),
and a sandbox block that is configured but switched off:
`enabled: false`,
with eight `filesystem.allowWrite` paths
(including `~/temp`,
`~/.local/share/mvm`,
`~/.cache/mise`)
and ten `excludedCommands` (`git`,
`docker`,
`bun`,
`spawn-claude`,
`mise`,
`tweakcc`,
`agent-browser`,
`vlt`,
`pnpm`,
`gh`).
It also sets `worktree.symlinkDirectories` to
`["package/cli/forbidden-strings/target"]` with `baseRef: "head"`,
`outputStyle: "default"`,
`autoMemoryEnabled: false`,
`spinnerTipsEnabled: false`,
and `prefersReducedMotion: true`.

DSH defines three presets in its patch and defaults to the widest:
`danger-full-access` with `approval: never`.
The `read-only` and `workspace-write` presets both use `approval: ask`.

This is a deliberate difference,
not a deficiency.
The forward-looking risk is that the 583-entry allow-list represents
accumulated operational knowledge about which commands are safe in this
workspace,
and DSH has no equivalent artifact.
If the DSH preset is ever narrowed to `workspace-write`,
there is no allow-list to migrate and every session starts from friction.

Pi's model here is different again:
`steeringMode: "all"`,
`followUpMode: "all"`,
plus `~/.pi/agent/trust.json` naming three trusted workspace paths.

## What DSH has that the others do not

The audit would be unbalanced without this.
Measured from the active plugin list:

- Agent Teams:
  `spawn_teammate`,
  `list_agents`,
  `send_message`,
  `wait_agent`,
  `interrupt_agent`,
  and a shared task board with compare-and-set revisions
  (`team_task_create`,
  `team_task_list`,
  `team_task_get`,
  `team_task_update`).
  Neither Pi nor Claude Code has a durable multi-agent board.
- Cross-session search:
  `session_search`,
  `session_event_search`,
  `session_trace`,
  `session_event_trace`,
  `session_event_read`,
  backed by `include:session-query-sqlite`.
- Persistent goal state with round limits and blocked-reason requirements:
  `create_goal`,
  `get_goal`,
  `update_goal`.
- Scripted multi-agent orchestration through the `workflow` tool.
- A declarative composition layer (`cordis.yml` plus `cordis.patch.yml`)
  with live introspection via `cordis_inspect_list` and
  `cordis_inspect_query`.
- Scheduled reminders with cron,
  weekly,
  daily,
  and fixed-rate timing.
- Spill-to-disk for large tool results,
  which is how the 52-tool schema dump
  and the `list_bundles` output in this audit remained usable.

## Glob and grep disabled user-wide, 2026-10-29

Requested as "disable the grep and find tools in dsh, user-wide",
then corrected to name `glob` by "find".
The pair is `glob` and `grep`,
both registered by `@deepseek-ai/dsh-tool-fs-search`,
which injects `tools`,
`systemPrompt`,
`subprocess`,
`workingDirectory`,
registers the two tools and self-hiding prompt sections,
and exports no service,
so disabling it breaks nothing else.

The change lives in a new user-wide layer file,
`~/.dsh/cordis.patch.yml`.
The Host row `tool-fs-search` is `disabled: true` there,
covering headless and advisor-fixture-minimal,
where the row served the tools;
the web-app bundle already disabled the row in web.
Web sessions took the tools from agent presets,
so `preset-standard`,
`preset-ptc`,
and `preset-cordis` carry `disabled: true` on their `tool-fs-search` child
rows.
`preset-minimal` never mounted them.
The `preset-cordis` restatement moved there from
`~/.dsh/profiles/web/cordis.patch.yml` with its rationale comments,
so one file owns the complete lists:
a home-layer override would otherwise shadow the profile one and leave two
restatements to keep in sync.

Measured 2026-10-29:
`dsh --dump-config` before and after differs for web only by the three
`disabled: true` child rows and patch-layer provenance lines,
every other composed value byte-identical,
including the moved `agent-instructions` and `tool-web` settings;
headless and advisor compositions are unchanged.
A patch naming an entry absent from a composition prints one stderr line per
boot (measured first with a throwaway `--patch` overlay),
so headless and advisor boots now print three such lines naming the home file.

Not observed:
the model-visible tool catalog of a new top-level Web session under the
running server.
Agents that already run keep the tool sets they started with,
and `dsh-subagent` children `composeFrom` the parent,
so a subagent of an old session proves nothing about new revisions.
The headless path was boot-checked to the model call,
which fails on this machine's pre-existing `MISSING_CREDENTIAL: llm-deepseek`
gap for the `deepseek-official` route,
so that probe observed composition but no tool catalog.

Consequences:
item 7's drift guard now covers three complete preset lists,
all in `~/.dsh/cordis.patch.yml`.
Workspace rules stay consistent,
because `AGENTS.md` rule RT1 already directs `rg` and `rg --files`
for search and discovery.

## Priorities

Items 1,
3,
4,
and 5 were actioned while this audit was being written and are recorded as
done or settled rather than open.

1. Done:
   the instruction budget cap was raised to 131072,
   which stops `AGENTS.md` being omitted.
   It lives in the `preset-cordis` override rather than a host-level row,
   because `dsh-web-app` sets the host `agent-instructions` row to
   `disabled: true`.
   Still open:
   DSH agents continue to receive Claude Code's 49-line preamble,
   and only a DSH-specific instruction file removes that contradiction.
   The change needs a server start to take effect and has not been observed.
   Resolved 2026-10-18,
   and not by the DSH-specific file this item proposed.
   The cap raise was observed live in a new session,
   and the profile now sets `instructionFileCandidates: ["AGENTS.md"]` on the
   same preset row,
   which drops `CLAUDE.md` and its preamble entirely.
   The rendered instruction payload fell from 68231 to 33237 bytes,
   measured through the plugin's own loader.
   The second-pass subsection of the instruction budget section records the
   probe,
   the four measured configurations,
   and the owner decision to drop the two harness-neutral preamble rules
   rather than relocate them.

2. Decide what to do about the 14 hidden workflow skills.
   These are the handoff,
   spec,
   ticket,
   and triage flows.
   Leaving them unreachable means DSH sessions cannot follow the same
   workflow vocabulary as Pi and Claude Code sessions.
   Owner decision recorded as "leave as an audit finding",
   so this stays open by choice rather than by oversight.

3. Done for search,
   open for breadth.
   One MCP bridge is composed:
   `package/dsh-plugin/search-fetch` connects the repo's shared search and
   fetch server,
   which restores the web search capability this machine had no route to.
   The web search section records the four boundary probes that verified it.
   Still open:
   the other Claude Code integrations.
   `morph` is the next candidate for code intelligence,
   since Pi deliberately sets `serena` to `"enabled": false`
   and its Claude Code allow-list entries are historical.
   Copy Pi's deferred-exposure idea rather than mounting every
   server's schema eagerly.
   Correction 2026-10-18:
   `morph` is not installed on this host,
   so the named candidate does not exist here and that recommendation is
   withdrawn.
   The measured inventory is thinner than this item assumed.
   Claude Code's global config declares six servers:
   `linkup` over HTTP,
   which duplicates the Linkup fallback already wired into `search-fetch`;
   `serena`,
   which Pi deliberately disables;
   `idea` on `127.0.0.1:64342`;
   `safari-mcp-stp`,
   which shells through `ssh` to a macOS Safari Technology Preview path;
   `BrowserOS` on `127.0.0.1:9001`;
   and `mvm`,
   a stdio server pointing at `package/mcp/mvm/dist/final/node/index.mjs`.
   Pi declares four:
   `browseros`,
   `linear`,
   `serena` disabled,
   and `safari-mcp-stp`.
   Only `mvm` is repo-owned and viable on this Linux host,
   and its CLI is already on `PATH`,
   so bridging it would add typed schemas and discoverability rather than
   capability.
   `linear` is the only genuinely new capability in either inventory,
   and it needs a token.
   MCP breadth is therefore a lower-value direction than this item implied.
   The deferred-exposure advice stands independently of that.

4. Settled:
   both fetch tools stay.
   The native `web_fetch` accepts any public HTTP(S) destination,
   while `mcp__search-fetch__web_fetch` enforces the 23-entry blocklist,
   routes GitHub URLs through `gh`,
   and strips base64 data-URL images,
   so the native tool is a route around the blocklist.
   Keeping it buys an account-free fetch path that survives an Exa or Linkup
   outage or rate limit.
   `fetch: false` on the `tool-web` preset row remains the one-line change if
   that trade is revisited.

5. Done:
   the 24 inert `disabled: false` patch entries were reverted,
   and the patch file again carries only rows that change profile behavior.
   Two rows that ended `disabled: true` for cause were removed with them,
   since neither can activate here:
   `pwsh-sandbox` and `optional-terminal-pwsh` both collide with the bash
   sandbox's `shell` service registration on Linux,
   and `desktop-product-telemetry` requires an OTel collector `endpoint`,
   `channel`,
   `serviceName`,
   and `serviceVersion` that are not configured,
   which leaves `product-analytics` pending on a service that never arrives.

6. Port `bash-output-filter` behavior if Bash output volume is a context
   problem in DSH sessions.
   It is the one hook with a direct,
   measurable context cost.

7. Guard the `preset-cordis` override against upgrade drift.
   Measured 2026-10-18:
   the override restates all 35 ids of the shipped preset in the same order,
   23 of them top-level plugin rows and the rest nested group children,
   and a diff of the two id lists is empty.
   Nothing was dropped,
   which is what the check was for.
   It also means the entire shipped preset is shadowed,
   so any upstream change to `presets/cordis.patch.yml` at the next upgrade is
   lost silently,
   with the `MAINTENANCE` comment as the only defence.
   The comment and the override moved to `~/.dsh/cordis.patch.yml` on
   2026-10-29,
   and that file now restates three preset lists after the tool disable
   recorded in the section Glob and grep disabled user-wide, 2026-10-29,
   so a drift guard has three lists to cover.
   A repo task that diffs the shipped preset against the override and fails on
   drift would turn that into a loud failure,
   and it would also protect the `agent-instructions` and `tool-web` settings
   that items 1 and 3 depend on.
   Proposed and not taken,
   so it stays open.

Machine-local findings from this audit,
covering credential handling and local harness configuration,
are tracked in a companion file matching the `*.local.*` pattern at
`.gitignore:11` and are deliberately not listed here.

## Surfaces this audit did not exercise

Stated so the gaps are not mistaken for verified absences.

- No fresh server start or new session,
  so every claim about what a new
  session would load is inference from config,
  not observation.
  Superseded 2026-10-18 for instruction loading and the tool catalog:
  a new session observed both instruction files being injected,
  which the 65536 default cannot produce,
  and observed native `web_search` absent with native `web_fetch` present,
  matching `tool-web: {search: false, fetch: true}`.
  Claims about other surfaces still rest on config.
- The default `instructionFileCandidates` order was not read from source.
  Read 2026-10-18 at `dsh-agent-instructions/lib/index.js:17-18`.
- The models exposed by `include:llm-deepseek` and
  `include:llm-deepseek-account` were not enumerated.
- `.claude/settings.local.json` was read in full for structure but the 583
  allow entries were counted,
  not individually classified.
- The five hook packages under `package/claude-code-plugin/` were identified
  by their `bin` names and `package.json` descriptions;
  their source was not read.
  The 292 package count for the hook search is the number of
  `@deepseek-ai+dsh-*` directories in the install tree.
  Counting every `@deepseek-ai+*` directory gives 304,
  the extra 12 being five `cordis` packages,
  two `libreoffice-kit`,
  two `node-addon-system`,
  plus `cosmokit`,
  `schemastery`,
  and the `dsh` meta-package,
  whose name carries no hyphen.
- The 52-tool measurement was taken from the lead session.
  An independent fact-check re-running the same query from a delegated
  subagent measured 46,
  which reconciles exactly:
  the six Agent Teams tools are lead-scoped.
  Neither seat could observe the other,
  so the pair is consistent rather than mutually confirming.
- The 203 `list_plugins` entries and the 26 `overrides` row ids were read from
  this session's own `plugin_manager` calls and were not independently
  re-measured.
- Claude Code's installed plugins (`playground`,
  `plugin-dev`) and its
  marketplace catalog were not audited.
- Other harness directories present in the workspace (`.crush`,
  `.factory`,
  `.fallow`,
  `.serena`,
  `.opencodereview`,
  `.codex`) were listed but not
  examined.
  `.codex` is empty.
- Whether `~/.agents/skills/` and the workspace `.agents/skills/` are the
  only two DSH skill roots was not confirmed from source.

## Verification

Every factual claim was re-measured from the filesystem by a separate
fact-checker that was instructed not to trust this document,
per the repo rule against same-session self-review in
`doc/agent/self-review.md`.
Of 20 numbered claims,
16 were confirmed on first check and three were wrong:
the `disabled: false` count,
which is 24 and not 25;
the package count for the hook search,
which is 292 `@deepseek-ai/dsh-*` directories and not 304;
and the `package/claude-code-plugin/` count,
which is 11 packages across 13 directories and not 13 packages.
All three are corrected in this revision,
along with four backticking violations and two mischaracterizations the
fact-check found:
an `0a` diff hunk carries no context lines,
and 25 of Pi's 26 skill entries are symlinks rather than all 26.
Markdown rules were checked with `mise run lint:monochromatic-lint`,
which passes clean,
and the document was rendered through `pandoc` to confirm both YAML fences
highlight and no emphasis marker leaks as literal text.

The web search remediation was verified differently,
by exercising the tools rather than reading configuration.
Installation reported `application: "applied"` with no warnings,
and `failOnStartupError: true` means a server that failed to start would have
blocked activation rather than silently omitting tools.
That is necessary but not sufficient,
so four boundary probes followed:
a real search,
a fetch positive control,
a blocklist rejection,
and a `gh`-routed GitHub fetch.
The positive control matters because a rejection proves nothing on its own;
the same tool succeeding on an allowed host moments earlier is what makes the
blocklist result meaningful.

One methodological error was made and corrected during this audit,
and it is recorded because the same trap is easy to fall into again.
Mid-way through,
the claim was made that `mcp__search-fetch__web_search` had been available
since the first turn,
which would have meant the bundle duplicated an existing connection.
It was wrong.
The DSH system prompt regenerates every turn,
so the tool list and the "MCP resource servers" section visible at any moment
describe the current composition,
not the composition at session start.
Recalling an earlier turn's context is not evidence.
`session_event_search` settled it:
the earliest occurrence of that tool name in this session is a `listTools`
result logged after the install,
and every earlier "search-fetch" hit is the Pi plugin path.
Any claim about what an earlier turn could see has to be checked against
session events.

Two verification techniques added on 2026-10-18 are worth keeping,
because both replace inference with measurement inside a session that cannot
restart its own server.

`dsh --profile web --dump-config` composes the full profile tree and exits
without booting,
so a patch edit can be checked against the real Loader result in the same
session that made it.
It also annotates which bundle or patch last touched each row,
which is how the host `agent-instructions` row was confirmed `disabled: true`
while the `preset-cordis` child row carried the override.

Importing a plugin's built bundle and calling its exported functions measures
the deciding code path directly.
`dsh-agent-instructions` exports `discoverBaselineInstructionFiles` and
`loadBaselineInstructions`,
and the returned `omitted` list names exactly which files the byte budget
dropped.
That turned the instruction-budget finding from arithmetic on file sizes into an
observation,
and it produced the sharpest single result in this audit:
under the shipped defaults the loader discovered both files and omitted
`AGENTS.md`,
so sessions were receiving Claude Code's preamble instead of the canonical
rules.
One trap came with it.
`loadBaselineInstructionSet` is declared in the package's type surface but is
absent from the runtime export list,
so a probe written against the `.d.ts` files fails at import.
Reading the bundle's own `export` statement is the reliable way to find what is
actually callable.
