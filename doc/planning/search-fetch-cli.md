# `search-fetch` CLI design proposal

Status: converged design from the grilling interview of 2026-10-31, folded through two
independent design reviews, awaiting explicit user acceptance. Nothing here is implemented. Per
the recommendation rule, this proposal unlocks `doc/decision/` entries and implementation work
only after the user confirms it.

## Purpose

Give the shared web search and fetch capability a terminal front door: a `search-fetch` command
running `web_search` and `web_fetch` for humans in a shell and for agents invoking it through
bash. Provider chains, blocklist, and key loading stay exactly as they are today. The only
behavior changes are the two agreed shared-core additions, the whole-call deadline and temp-file
retention, and they apply to the shipped pi and MCP surfaces too, not only the new CLI.

## What to inspect

The decision record below, especially `Layout and rename ledger`, `CLI contract`, and
`Shared-core behavior changes`. Each decision names the interview answer it traces to or the
repo precedent it adopts.

## How to respond

Confirm the design as a whole, or veto any named item. Stream ownership under `CLI contract` is
pinned here for the first time and belongs to the output item a veto can move. Two delegated
picks were resolved by the author and confirmed in round 7: unknown flags are rejected, and
output is the rendered tool text with no pretty mode.

## Superseded wording

Interview messages before round 7, and the record snapshot given to the second reviewer, carry
three phrasings this document supersedes. They are listed so nobody reconciles them by hand:

- "60 s whole-call deadline" with the rationale "so fallback hops fit inside the budget":
  withdrawn in round 7 after review showed gh 30 s plus DeepSeek 30 s consumes 60 s before
  Linkup or Exa start. The value is 120 s.
- "Caller AbortSignal still wins": replaced by combined-timer semantics, where both timers run
  and no host extends past the default.
- "Multiple positionals joined with single spaces": replaced by the literal grammar, where only
  tokens after `--` join.

## Decision record

### Product shape

- A third surface over `@monochromatic-dev/agent-harness-shared-search-fetch`, alongside the pi
  adapter and the MCP adapter, so behavior has exactly one implementation (rounds 1 and 2).
- Command `search-fetch` with exactly two verbs, `search` and `fetch`, plus `--help` and
  `--version` (rounds 2 and 7). No diagnostics verb, no batch or stdin input, no timeout
  override flag (rounds 2 and 5).
- Serves both humans in a terminal and agent scripts (round 1).

### Layout and rename ledger

The search-fetch product moves into one category directory, `package/search-fetch/` (rounds 4
and 6). Package names follow directory paths under the naming rule, so every move is a rename
that lands with all consumers in one change.

- `package/agent-harness-shared/search-fetch` becomes `package/search-fetch/core`,
  `@monochromatic-dev/agent-harness-shared-search-fetch` becomes
  `@monochromatic-dev/search-fetch-core`.
- `package/mcp/search-fetch` becomes `package/search-fetch/mcp`,
  `@monochromatic-dev/mcp-search-fetch` becomes `@monochromatic-dev/search-fetch-mcp`, which now
  equals its bin name.
- `package/pi-plugin/search-fetch` becomes `package/search-fetch/pi-plugin`,
  `@monochromatic-dev/pi-plugin-search-fetch` becomes
  `@monochromatic-dev/search-fetch-pi-plugin`.
- `package/dsh-plugin/search-fetch` becomes `package/search-fetch/dsh-plugin`,
  `@monochromatic-dev/dsh-plugin-search-fetch` becomes
  `@monochromatic-dev/search-fetch-dsh-plugin`.
- New `package/search-fetch/cli`, `@monochromatic-dev/search-fetch-cli`, bin `search-fetch`.
- `@monochromatic-dev/agent-harness-shared-truncate` stays at `package/agent-harness-shared/truncate`
  (round 6): a general-purpose truncation library whose only runtime consumer today is the
  search-fetch core.

The rename ledger, all landed in the same change:

- `package.json` `name`, `repository.directory`, and workspace dependency specifiers of every
  moved package and every consumer. Root `package.json` is not file-enforcer managed, so the new
  root dependency is a direct edit (root workspace dependencies are permitted).
- Root `mise.toml` and `mise.no-env.toml` task paths, including `secrets:migrate` with its
  `//package/...:build` task and its literal `node package/.../migrate.mjs` invocation.
- Root `mise.toml`'s generated `_.path` block (the `node_modules/.bin` PATH entries) is produced
  by file-enforcer from its `package/*/*/node_modules/.bin` glob: re-run file-enforcer after the
  moves and commit the regenerated block as-is. Never hand-edit it.
- `.github/workflows/env-local-json.yml` holds trigger path filters as well as task invocations;
  both must move. A stale filter silently stops CI for the moved packages.
- `package/dsh-plugin/search-fetch/cordis.patch.yml` gets three coordinated edits: the
  `createRequire(...).resolve(...)` package-name argument, the relative artifact suffix
  (`../../mcp/search-fetch/dist/...` becomes `../mcp/dist/...`), and the header comments. Missing
  the name argument makes `resolve` throw at mount, and the bundle sets `failOnStartupError`.
- `package/config/pnpr/config.yaml` is generated: `mise run //package/config/pnpr:generate:config`
  then `mise run //package/config/pnpr:check:config`. Never hand-edit.
- `pnpm-lock.yaml` regenerated by the package manager, with unrelated drift reported separately.
  The `pnpm-workspace.yaml` `package/*/*` glob already covers the new layout.
- Each package `task.ts` self-reference (task bodies embed `//package/<old path>:*` strings),
  README, and any `HANDOVER.*` reference.
- Docs: about 45 documents name old paths, and more mention the product. Current docs
  (`doc/planning/`, `doc/decision/`, `doc/troubleshooting/`) get their paths updated. Historical
  records follow the repo's 2026-10-29 move precedent: top-of-record notes in narrative Markdown
  only, JSON and JSONL evidence files stay byte-stable, and the complete old-to-new mapping is
  recorded once in this ledger.
- External host configuration that invokes the old absolute artifact path (the Codex MCP
  registration recorded in `doc/troubleshooting/codex-search-fetch-mcp.md`) is migrated in the
  same landed change and reported.

### CLI contract

Grammar, with the token encoding sketched before the form (rounds 5 and 7):

- `search-fetch search <query>` and `search-fetch fetch <url>` take exactly one positional.
- Everything after `--` is joined with single spaces: `search-fetch search -- cachyos kernel 2026`
  builds one query string. Uniform for both verbs (round 6).
- Supplying a positional together with `--` words is explicitly undefined behavior and documented
  as such (round 5).
- Several positionals before `--`, zero positionals, an unknown verb, or a missing flag value are
  usage errors (exit 2). Tokens after `--` are always data, never options.
- A bare `search-fetch` with no verb prints usage on stderr and exits 2.

Options, long-form only (round 5):

- `--include-domains a.com,b.com`: comma list in one flag; entries are trimmed and empty entries
  dropped (round 5). A list whose entries are all empty after trimming (`--include-domains ",,"`)
  is the same as the flag being absent.
- `--from-date` and `--to-date`: ISO date strings passed through exactly as the tool schema
  accepts them, with no extra validation (the schema requires strings only).
- Repeating any single-valued flag is a usage error (exit 2), not last-wins.
- `--help`: global usage from `search-fetch --help`, verb usage from `search-fetch search --help`.
- `--version`: prints the `@monochromatic-dev/search-fetch-cli` package version (round 7).
- Unknown flags are rejected as usage errors (rounds 5 and 7): the CLI has a deliberately strict
  flag grammar, then feeds known values into the tolerant tool interface.

Output and stream ownership (rounds 5 and 7):

- A data run writes only the shared tools' rendered text to stdout: JSONL result lines for
  provider responses shaped like `{"results":[...]}`, JSON or raw page markdown otherwise, always
  ending with the truncation notice naming the full-response temp path when truncated. Not one
  uniform machine JSON schema.
- No pretty mode and no `--json` flag; both delegated picks were confirmed in round 7.
- Explicit `--help` and `--version` print to stdout and exit 0. Misuse prints usage to stderr and
  exits 2, and a failing run leaves stdout empty. Diagnostics and warnings go to stderr.

Exit codes (round 5):

- `0` success and explicit help or version output.
- `1` provider failure or deadline expiry.
- `2` usage error, configuration error, or blocklist refusal.

### Shared-core behavior changes

Two behavior changes to `@monochromatic-dev/search-fetch-core` (renamed as above), applying to all
three surfaces (rounds 3, 4, 7). This changes the shipped pi and MCP surfaces too: slow calls now
time out, and old temp dirs get pruned.

- Response temp-file retention: the core prunes its `pi-search-fetch-response-*` dirs on write,
  after a successful write, deleting dirs older than 7 days and anything beyond the newest 64,
  counting the dir just written and never removing the dir just returned. The cap is per scratch
  parent (nearest `node_modules/.monochromatic/search-fetch`, else the XDG cache dir), not
  machine-global. Best-effort: only the owned prefix is touched, concurrent writers are
  tolerated, and deletion failures are logged and never fail the search or fetch. MCP
  `resources/list` may then advertise an expired resource; that read already fails with
  `ResourceReadError`, and tests pin the expired-resource behavior.
- Whole-call deadline: a default of 120 s bounds one complete call, including every fallback
  hop. A caller `AbortSignal` combines with the default timer and whichever expires first
  aborts, so no call exceeds the default and no host extends past it. The `gh` CLI deadline drops
  from 60 s to 30 s, so gh plus DeepSeek (30 s) leave a real 60 s tail for Linkup and Exa in the
  worst case. Deadline expiry surfaces as an ordinary call failure with a diagnostic naming the
  budget.

### Construction (adopted from repo precedent)

- `node:util` `parseArgs` with hand-rolled verb dispatch; no third-party parser (round 4; the
  catalog strips `argparse`, CAC was vetted and rejected, and 15 packages mark Optique
  `// TODO: deprecate Optique`).
- The literal token grammar needs the `--` boundary preserved: `parseArgs` positionals cannot
  distinguish `search -- a b` from `search a b` (verified identical on node 26), so the
  implementation pre-splits `process.argv` at the first bare `--` before option parsing. The left
  side carries flags and at most one positional; words on the right join with single spaces and
  are always data. (`tokens: true` is the current spelling for token access; the older
  `includeTokens: true` is silently ignored.)
- Bespoke minimal help and exit handling in the new package; do not copy the `runCli` and
  usage-from-constants machinery of other CLIs (round 4).
- Validation reuses the shared valibot tool specs. Logging via tagged `logger-core` loggers with
  diagnostics kept off result stdout. Errors as custom classes with `@throws`, `nonNullishOrThrow`
  instead of non-null assertions, error text inside the thrown error.
- The top-level handler maps error classes to exit codes: usage, configuration, and
  `BlockedUrlError` failures set `process.exitCode = 2`; provider and deadline failures set 1.
- Build with the shared rolldown node config: `#!/usr/bin/env node` shebang injected at byte 0 of
  bin-targeted chunks, executable bit set, artifacts under `dist/final/node`.
- `task.ts` beside `mise.toml` (CI runs the mise-free task runner), with mise tasks mirroring
  sibling packages: `build`, `lint`, `lint:types`, `lint:oxlint`, `test:unit`, `buildAndTest`.
- Unit tests drive the built artifact. Package completeness: README, license files, zero lint
  errors, and tests covering every exported code path and implementation branch.

### Install and invocation (round 5)

- Root `package.json` gains a workspace dependency on `@monochromatic-dev/search-fetch-cli`, the
  verified mechanism that makes `mvm` resolve through `node_modules/.bin` today.
- Documentation also shows absolute-path invocation
  (`node package/search-fetch/cli/dist/final/node/index.mjs`), the mechanism the Codex MCP
  registration already uses.

### Config and degraded mode (adopted)

- Same configuration as the pi and MCP surfaces: blocklist from
  `~/.pi/agent/extensions/pi-search-fetch.json`, provider keys from `EXA_API_KEY`,
  `LINKUP_API_KEY`, `DEEPSEEK_API_KEY` or the sops/age `.env.local.json` chain via
  `env-local-json-core`. No data migration is needed: those key names and that path are
  unchanged. The `pi-` prefixes in the config filename and in the `pi-search-fetch-response-`
  temp-dir prefix are historical and intentionally kept.
- No `search-fetch://response/<id>` URIs or `resource_link` blocks in CLI output; those are MCP
  transport features. The truncation notice and temp path serve the same purpose.
- Identical degraded mode without keys: blocklist plus `gh`-routed fetch still work; calls
  needing a missing key fail with the value-free store diagnostic.

## Assumptions

- "Also be a CLI" covers both tools, `web_search` and `web_fetch`, with today's provider chains,
  blocklist, and key loading unchanged, except for the two shared-core behavior changes above
  (working reading from round 1, never contradicted).
- The 7 day and 64 dir retention numbers and the 30 s `gh` deadline are the values named in the
  interview options the user selected.

## Rejections considered in the interview

- CLI as a second bin inside `@monochromatic-dev/mcp-search-fetch`, or a bin inside the shared
  core (round 2).
- Third-party argument parsers and Optique (round 4).
- Warn-and-strip unknown flags; human pretty output; timeout override flag; diagnostics verb;
  batch or stdin input (rounds 2, 5, 7).
- Never truncating CLI output, or truncating only for humans (round 2).
- The broadened grammar rule joining arbitrary positionals that appear before `--`: withdrawn in
  round 7 in favor of the literal rule, where only tokens after `--` join.
- Count-only or CLI-only temp-file cleanup (rounds 3 and 4).
- A core-wide deadline below 120 s: the 60 s proposal was withdrawn after the review showed it
  starves the fallback chain (round 7).
- Short command aliases and the `search-fetch-cli` command spelling (round 2).

## Limits recorded

- Output is rendered tool text, so scripts must handle JSONL, JSON, or raw markdown depending on
  the provider response.
- Combined timers mean a caller cannot extend a call past the 120 s default.
- A positional plus `--` words is undefined behavior by design; the README must not present it as
  supported.
- The bare `search-fetch-mcp` registration snippet in the MCP package README cannot work in a
  checkout today (verified: the command resolves nowhere while a positive control resolves);
  fixing that snippet is accepted into scope.

## Verification plan

- Core tests: deadline expiry at each fallback stage, combined caller-signal semantics, the 30 s
  `gh` bound, cleanup age and count boundaries, nonmatching entries, deletion failures,
  concurrent writers, preservation of the just-returned path, expired MCP resource reads.
- Retention from two different working directories, pinning the per-scratch-parent cap.
- Built-artifact CLI tests: both verbs, every grammar form (single positional, `--` words, both
  together as undefined, several pre-`--` positionals, zero positionals, unknown verb, missing
  flag value, unknown flags), stdout and stderr separation, exit codes 0, 1, and 2, truncation
  with the temp file present and readable, absence of MCP resource URIs.
- Explicit furniture cases: `--help` global and verb, `--version`, and bare `search-fetch` with
  no args (usage on stderr, exit 2); repeated single-valued flags rejected; `--include-domains ",,"`
  equal to absent.
- Failure streams: stdout empty on both exit 1 and exit 2. Exit-code mapping cases: blocklist
  refusal to 2, configuration error to 2, deadline expiry to 1.
- Run the real command through both the root-linked bin and the documented absolute path.
- Existing MCP and pi package tests, plus host checks: dsh plugin mount and tool call, pi
  extension verification, MCP discovery and call, and the migrated Codex configuration.
- `lint:types` on every edited TypeScript package; generated pnpr config and lockfile diffs
  inspected; verification from a fresh worktree so stale `dist` trees at old paths cannot mask
  breakage.
- Post-move gates: `mise run //package/config/pnpr:check:config`, a clean file-enforcer re-run, a
  hidden-aware search for old names and paths (plain `rg` skips `.github`), and each moved
  `task.ts` self-reference.

## Documentation follow-ups in scope

- Fix the registration README bare-command gap described in Limits.
- Record the exit-code and stdout/stderr machine-output conventions once, in `AGENTS.md` or a
  `doc/decision/` entry, tightening existing rules rather than adding parallel ones.
- Rewrite `doc/decision/cli-arg-validation.md` off the deprecated Optique framework onto
  `parseArgs` plus the shared valibot specs.

## Next action

Both independent reviews are folded in. Request explicit user confirmation of this proposal.
Implementation starts only after that confirmation.
