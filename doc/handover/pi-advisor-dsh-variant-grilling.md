# Pi advisor DSH variant grilling

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/terminal` is now `package/terminal/app`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/async-time` is now `package/async-time/core`;
> `package/module/logger` is now `package/logger/core`;
> `package/module/test` is now `package/test-harness/core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/unused-export` is now `package/mutation-test/cli-unused-export`.

> Moved 2026-10-31 by the search-fetch product move into `package/search-fetch`
> (`doc/planning/search-fetch-cli.md`);
> paths and names below this note are historical:
> `package/agent-harness-shared/search-fetch` is now `package/search-fetch/core`;
> `package/pi-plugin/search-fetch` is now `package/search-fetch/pi-plugin`;
> `package/dsh-plugin/search-fetch` is now `package/search-fetch/dsh-plugin`.

Purpose: record the design interview that gives `package/pi-plugin/advisor` a DSH variant
on a shared package,
and the decisions settled so far.
Status: interview complete.
Shared understanding confirmed on 2026-10-20 (Q13);
implementation is authorized and underway.
This file is also the working handover:
 commits, decisions since the interview, measured failures, and the queue
 live in Implementation progress.

## Request

Make the pi advisor package also have a DSH variant,
and use a shared package.

## Settled decisions

- Q1 Deliverable shape: the full triple mirroring `search-fetch`:
  `package/agent-harness-shared/advisor` (shared core),
  new `package/dsh-plugin/advisor` (DSH variant),
  `package/pi-plugin/advisor` rewired onto the core.
  Rejected: dsh consuming pi build output.
- Q2 Pi rewiring happens in the same series,
  deleting pi's private copies of moved modules.
- Q3 Fixes are allowed inside the move,
  not frozen to behavior identity.
  Attribution mitigation: every behavior pick lands with an updated test
  in the same commit.
- Q4 Day-one DSH surface: `advisor` tool plus `/advisor` command parity
  (all five subcommands: bare, `<slug>`, `status`, `off`, `on`).
  Anything DSH cannot express becomes a documented deliberate gap.
- Q5 Behavior picks when merged copies disagree: default to richer,
  no approval round,
  reported after the series in an end-of-series report with before/after per pick.
- Q6 Tie-break when richer is ambiguous: incumbent wins
  (repo's existing shared implementation when one exists,
  otherwise the information-preserving variant).
- Q7 Shared-core depth: middle.
  Pure modules plus the pi-ai-typed pipeline move behind narrow ports
  (model resolution, transcript source, theme tokens).
  Adapters keep registration, commands, pi-tui rendering, session plumbing.
- Q8 Config semantics: unified at today's paths,
  `~/.pi/agent/extensions/pi-advisor.json` global plus
  `.pi/extensions/pi-advisor.json` project.
  Both variants read the same files; pi-visible behavior stays untouched.

## Adopted without asking

- Package names follow SGD:
  `@monochromatic-dev/agent-harness-shared-advisor`,
  `@monochromatic-dev/dsh-plugin-advisor`.
- All new packages are `private: true`.
  Evidence: every precedent package measured private
  (`pi-plugin/advisor`, `pi-plugin/search-fetch`,
  `agent-harness-shared/search-fetch`, `pi-shared/model-selection`).
- Full behavior depth including hedging, overlap, deadline, and ledger.
  Forced by Q4 parity plus the report showing that machinery is host-neutral pure code.
- Extraction and pi rewiring land before the DSH adapter.
- Slopo issue 621 (shared fake pi extension host) stays out of this series,
  the narrow reading; proposed follow-up.
- Issue tracking follows the `pi-advisor-413` precedent:
  a planning doc plus a GitHub issue opened `needs-triage`.
- Verification runs on a disposable fixture profile first (THR),
  never the live profile.
- New packages get `README.md`, sibling-mirroring `mise.toml` tasks
  (`verify:extension` analogues),
  and `/ts` subpath exports (ST3).

## Facts the recommendations rest on

- `package/agent-harness-shared/search-fetch` plus `package/pi-plugin/search-fetch`
  plus `package/dsh-plugin/search-fetch` is the proven triple.
  `dsh-plugin/search-fetch` is a configuration-only DSH bundle
  (`cordis.patch.yml` mounting `@deepseek-ai/dsh-mcp-client`)
  with no `src/`.
- Advisor inventory: 67 `.ts` files, 13,040 lines.
  Pure (no pi imports) about 2,840 lines;
  pi-ai-typed pipeline about 1,600;
  host-coupled about 4,700
  (`pi-tui` rendering, `ModelRegistry` selection, `SessionEntry` context,
  `ExtensionAPI` registration).
- Advisor already consumes `@monochromatic-dev/pi-shared-model-selection`
  from 10 source files (scope resolution and default model selection).
- Host APIs used by the pi variant:
  `pi.registerTool`, `pi.registerCommand`, `pi.registerMessageRenderer`,
  `pi.on`, `pi.sendMessage`,
  `ctx.modelRegistry`, `ctx.waitForIdle`, `ctx.ui.notify`, `ctx.signal`.
- No in-repo DSH plugin authoring guide exists;
  the only DSH package is the config-only search-fetch bundle.
  `doc/audit/dsh-setup-gaps-2026-10-17.md:485-487` records that
  `dsh-commands` is a code-level `CommandRuntime` registry,
  so a workflow layer means writing a plugin,
  not dropping files in a directory.
- `plugin_manager` responds in the session that ran this interview
  (list returned 204 entries),
  so an end-of-work install is executable from the agent side.

## Settled decisions, later rounds

- Q9 `package/pi-shared/model-selection` moves and renames to
  `package/agent-harness-shared/model-selection`
  in this series, dir plus name plus consumers together
  (`pi-plugin/advisor`, `pi-plugin/goal`, the new core),
  keeping tiered exports and pi optional peers.
  Rejected: splitting tiers, leaving it in place,
  absorbing selection into the advisor core.
  `doc/todo/pi-shared-model-selection.md` gets rewritten to the new home.
- Q10 After disposable-profile verification passes,
  the advisor DSH bundle installs into the live DSH profile
  with `plugin_manager install_bundle`,
  reversible with `remove_bundle`,
  reported together with the behavior-pick list.
- Q11 Config convention collision resolved for the unified JSON file:
  the DSH plugin reads the same `~/.pi/agent/extensions/pi-advisor.json`
  and `.pi/extensions/pi-advisor.json` through the shared loader,
  its Cordis `Config` holds only structural fields,
  and its README records the deliberate deviation
  from the DSH "tunables in `cordis.patch.yml`" practice.
  Rejected: Cordis-config tunables (second source of truth),
  split across `dsh-settings` and files.
- Q12 Default reviewer choice without cost metadata:
  a `defaultReviewerModel` key in the unified config wins when set,
  otherwise the DSH variant ranks by largest `contextWindow` then name.
  The pi variant keeps its highest-expected-cost ranking when the key is unset.
  When the key is set it wins in both variants.
  Rejected: the pi-ai catalog cost side-table (the recommended option),
  fail-loudly without a model.
- Q12 amendment (user, 2026-10-20):
  `defaultReviewerModel` is set in the user's own global config
  `~/.pi/agent/extensions/pi-advisor.json` to `opencode-go/kimi-k3`
  (provider `opencode-go`, model `kimi-k3`, "Kimi K3",
  verified in `~/.pi/agent/models-store.json`),
  because the DSH LLM seam exposes no model cost metadata.
  Correction: an earlier draft of this record claimed the key
  ships as a schema default value;
  that claim is retracted.
  The schema key is optional with no baked-in default.
  Selection order: explicit tool `model` parameter,
  then `defaultReviewerModel` when resolvable in the current scope,
  then the variant fallback (pi highest expected cost,
  dsh largest `contextWindow` then name).
  Explicitly clearing the key, or naming a model absent from the scope,
  falls through to the variant fallback.
  Untouched by this request: pi `settings.json` `defaultProvider`
  and `defaultModel`, and any DSH profile default model.
  The config write is safe before the schema change lands:
  valibot `v.object` strips unknown keys,
  probed on the installed valibot where a parse of
  `{ a: 1, defaultReviewerModel: ... }` through `v.object({ a: v.number() })`
  succeeds and drops the extra key.
- Q13 Shared understanding confirmed by the user on 2026-10-20.

## DSH API probe findings (forced adoptions)

- Form: in-process Cordis plugin.
  `ctx.tools.register(defineTool(...))` and `ctx.commands.register(...)`
  both exist;
  the MCP-bundle form cannot express commands at all.
- Model calls: `ctx.llm.stream` plus `listProviders()`,
  the only supported path into provider adapters
  (dsh-llm README).
  Direct pi-ai import is rejected as a bypass
  of routes, credentials, retry, and session-log vocabulary.
- Command mapping: one `/advisor` handler parses its unstructured raw input
  (bare, `<slug>`, `status`, `off`, `on`);
  toggling maps to `ctx.tools.restrict`.
- Rendering: tool `output.render` produces `text` ContentBlocks
  with markdown riding inside text;
  no progress channel exists, so a tool call settles once
  and live progress text is a documented deliberate gap (Q4 clause).
- Transcript source for the DSH port: derived session message history
  through the session surface and `SessionProjection` state.
  Fallback if binding proves insufficient: the tool `question` parameter
  carries the content.
- Verification: fixture profile via `dsh --profile <fixture>`
  (`DSH_PROFILE`, `DSH_PROFILE_DIR`),
  checked with `cordis_inspect_query`;
  then the Q10 live install.
- Cost metadata: the LLM seam exposes none for text
  (`LlmModelInfo` is id, name, description, modalities);
  pi-ai catalogs do carry `cost: ModelCost`,
  which Q12 declined to use.

## Rejections on record

- Behavior freeze during extraction (Q3 recommendation),
  rejected in favor of fixes-in-move.
- Enumerated approval of behavior picks (Q5 recommendation),
  rejected in favor of default-to-richer with an after-report.
- Dsh variant consuming pi package build output (Q1),
  rejected as harness coupling.
- pi-ai catalog cost side-table (Q12 recommendation),
  rejected in favor of `defaultReviewerModel` plus context heuristic.

## Implementation progress

Authoritative task state,
 updated as work lands.

### Commits

- `b710410e81` `refactor(*): move model-selection to agent-harness-shared`:
  the Q9 move,
  dir plus name plus consumers (advisor, goal, thinking-default, model-retirement),
  the `mise.toml` PATH entry,
  the pnpr OIDC allowlist entry,
  a regenerated lockfile (rename-only,
  measured: no dependency version changed),
  and the WR6 note in `doc/todo/pi-shared-model-selection.md`.
- `c8d9af87d5` `fix(*): stub new pi-coding-agent 1.3.0 ExtensionAPI members in verify fakes`:
  TS2740 stubs in advisor,
  guardrail,
  and thinking-default `src/mise.verify-extension.ts`,
  mirroring the search-fetch fake.
- `3b9bb3ad6b` `fix(agent-harness-shared-shell-command-analyzer): parse unbash 6 TestLogical operands`:
  `visit-expression.ts` maps `operands`,
  dead `Time`/`Negation` branch removed in `visit-node.ts`,
  2-way and 3-way TestLogical cases added.
- `fce8efd40e` `fix(pi-plugin-current-time-context): fill hiddenTools and messageType in the test harness`:
  `pi-test-harness.ts` fixture updates for pi-coding-agent 1.3.0.
- `ad7a5a67b4` `fix(pi-plugin-terminal-title): narrow before_agent_start to its user variant`:
  handler reads `prompt` only on `messageType: 'user'`,
  custom runs skip the prompt title (new test case).
- `22f130ed5b` `fix(pi-plugin-model-retirement): regenerate the retirement table fixture for the 2026-10-15 catalog`:
  regenerated with the documented task after reviewing the retirement diff.
- `ea2812babc` `fix(cli-unused-export): adopt the yuku-analyzer 0.19.0 API`:
  `addFile` to `setFile`,
  `referencesOf` to `Binding.findReferences`,
  `Diagnostic.module` to `Diagnostic.path`.

### Decisions since the interview

- 2026-10-20 user directive: fix pre-existing failures too,
  not only this series' regressions.
- 2026-10-20 user directive: keep this handover current,
  updating it as work lands.
- 2026-10-20 user directive: use subagents aggressively,
  delegating independent workstreams in parallel.
- `defaultReviewerModel` stays user config only
  (`~/.pi/agent/extensions/pi-advisor.json`,
  value `opencode-go/kimi-k3`),
  never a schema default.
- 2026-10-20 user decision (flagged adoption approved):
  `defaultReviewerModel` selects the initial reviewer of a default call
  and keeps the serial fallback/overlap recovery semantics;
  only an explicit `model` argument or `<slug>` is exact.

### Pre-existing failures: measured, swept, fixed

Measured 2026-10-20 through package `lint:types` runs.
None of these files are in this series' diff,
and no dependency version moved in the move commit.

Fixed with the commits recorded in `### Commits`:

- `agent-harness-shared/shell-command-analyzer` (unbash 6.0.0 AST reshape),
  `pi-plugin/current-time-context`,
  `pi-plugin/terminal-title`,
  `pi-plugin/model-retirement` (catalog drift fixture),
  `cli/unused-export` (yuku-analyzer 0.19.0 API).

Full sweep 2026-10-20: 213 packages with a `lint:types` task,
170 pass,
43 fail.
Of the 43:
1 real API drift (`cli/unused-export`,
now fixed),
38 build-class (`translation-repair/*`,
`module/async-time`,
`module/logger`,
`module/test`;
missing or stale `dist`),
and 4 false positives (`config/typescript`,
`test-fixture/oxlint-*`,
whose `lint:types` tasks are commented out on purpose;
the sweep matched the comment text).

Delegated 2026-10-20 (subagents,
disjoint write scopes):

- extraction fork `40eef920`: DONE 2026-10-20,
  shared core `package/agent-harness-shared/advisor` (new)
  plus `package/pi-plugin/advisor` rewired onto it.
  Commits:
  `093f32832a` move the 53 core files,
  `38aeccbdbd` extract behind the three ports
  (`AdvisorModelResolution`,
  `AdvisorTranscript`/`AdvisorTranscriptSerialization`,
  `AdvisorTheme`),
  `ee89203126` retype the test fakes,
  `77e28de319` rewire the pi adapter,
  `a46ffb6b79` record install outputs,
  `09be402fe6` chain-per-line lint fix,
  `44f7ee2e56` Q5 pick applied
  (`formatDuration` DECIMAL_BELOW_MS precision,
  3 discriminating tests),
  `47699d3123` README with ports,
  adapter guide,
  `defaultReviewerModel` user-config-only policy,
  Q5/Q6 picks,
  LN3 suppression record.
  Q6 reviewed-not-applied (incumbent wins):
  `truncateContext`,
  `buildAdvisorUserMessageText`,
  config `readJsonFile` copies keep incumbent behavior.
  Verified by the fork:
  both packages `lint`,
  `lint:types`,
  `build`,
  `test:unit` green
  (shared 15 PASS suites,
  pi 4 PASS suites,
  0 FAIL).
  Lockfile and `config/pnpr` changes belonged to this scope alone.
- docs subagent `94e9414e`: `doc/troubleshooting/*` only,
  the 2026-10 dependency API drift write-up.
- sweep subagent `25c55b61`: the 38 build-class packages only;
  parked until the repo-wide `mise run build` ends
  (no concurrent builds),
  lead wakes it on the build's completion.
  NOTE 2026-10-20: became unresumable after parking
  (`send_message` target not found);
  scope handed to replacement `bd3dc9b4`,
  which also takes its findings (the three `module/*` TS2305s were
  stale-dist artifacts,
  `translation-repair/source-scan` and `translation-repair/test-fixture`
  ship no `dist` by design).
  `bd3dc9b4` DONE 2026-10-20: all 39 packages pass `lint:types` after the
  rebuild,
  zero source drift,
  zero commits.
  Green validated with positive controls (deliberate TS2322 probes that
  made `lint:types` fail,
  then removed).
  Count correction: 35 `translation-repair` packages were red at sweep
  time (38 build-class total with the three `module/*`),
  not 36;
  `translation-repair/test-fixture` was green even then.
  `source-scan`'s red was missing dependency dists (TS2307 on
  `@monochromatic-dev/translation-repair-markup` and siblings).
- design subagent `1d3fe543`: DONE 2026-10-20,
  commit `f382400eab`: `doc/planning/pi-advisor-dsh-variant.md`
  (probed host API signatures,
  capability gaps each with a designed fallback).
  Probe corrections to the briefing:
  `LlmModelInfo` carries no `contextWindow`
  (only `LlmResolvedModelInfo.context.contextWindow` and
  `LlmDiscoveredModel.contextWindow?` do),
  and `package/dsh-plugin/search-fetch` is a configuration-only bundle;
  the in-process `apply(ctx)` precedent is `@deepseek-ai/dsh-tool-cordis`.
  Two shared-core prerequisites it specifies
  (queued as follow-up below,
  because running subagents cannot be steered in this harness:
  `send_message` rejects subagent ids with `active teammate not found`):
  optional `defaultReviewerModel` in `AdvisorConfigFileSchema`/`AdvisorConfig`
  with no schema default,
  and a ranking seam in `selectAdvisorModel`
  (pi expected cost versus DSH largest `contextWindow` then `name`,
  both supplied externally).
- docs subagent `94e9414e`: DONE,
  commit `601c69399e`:
  `doc/troubleshooting/deps-update-api-drift.md` (new class doc,
  unbash 6.0.0 / yuku-analyzer 0.19.0 / pi-ai catalog incidents,
  symptoms reproduced from a pre-fix worktree at `b710410e81`)
  plus two pi-1.3.0 sections in
  `doc/troubleshooting/pi-0-87-extension-api-breakage.md`.
- build-fixes subagent `63d2bd1d`: DONE 2026-10-20,
  three pre-existing build failures fixed
  (separate incidents,
  separate commits):
  `ba5bd66c30` `pi-plugin/auto-mode` placeholder `src/index.ts`
  (a ground-up rebuild commit `7ac15b19b4` deleted sources
  but left the entry and `./ts` export pointing at it),
  `ef4a659b18` `config/rolldown` canvg alias path
  (`src/stubs/` versus the real `src/stub/`;
  no canvg install needed,
  the app rasterizes to PNG and never uses jspdf's SVG path),
  `d21582c951` `doodle-widget` `src/build.ts` asset path
  (stale plural `src/assets/` after singularization),
  `1eee51ee2f` `desktop-app/terminal` libghostty-vt staging
  (root cause is output discovery,
  not build ordering:
  machine `~/.cargo/config.toml` relocates `[build] build-dir`,
  so `cargo metadata` `build_directory` must be searched
  alongside `target/`,
  in both dir layouts).
- follow-up subagent `9f07ca3d`: DONE 2026-10-20,
  `32308cbf48` corrects the libghostty-vt staging diagnosis in
  `doc/troubleshooting/mise-dependency-freshness.md`
  (WR6 correction note plus a durable `cargo metadata`
  `build_directory` lesson),
  `bd5e135a03` repoints `ZIG_GLOBAL_CACHE_DIR`
  in root `.cargo/config.toml`
  (measured live-but-wrong with a build-script env probe;
  the singular `package/` cache holds 437M).
  Separate commits per incident.

The lead holds `doc/handover/` edits and cross-agent integration.

### 2026-10-20 uncommitted-work loss incident (extraction)

Around 17:52-17:53 EDT,
extraction fork `40eef920`'s UNCOMMITTED work vanished.
The first attribution
(a concurrent session's `git merge`/`reset` on `main`)
is retracted 2026-10-20:
`cli-git`'s `linked-worktree-only` policy rejects `reset`,
`stash`,
and deleting `clean` in the main worktree
(`GuardedCommand` is `stash` | `clean` | `reset`),
and `git reflog` shows no reset in the window,
only ordinary commits
and the landing machinery's fast-forward merges.
Lost:

- destroyed: the staged `git mv` batch
  (36 core files plus 17 tests into `package/agent-harness-shared/advisor/src`)
  and the unstaged pi-adapter edits
  (`index.ts`,
  `tool.ts`,
  `types.ts`,
  `commands.ts`,
  `rendering.ts`,
  `status.ts`,
  `command-message.ts`,
  `verify-host.ts`,
  `operation-failure-accounting.ts`,
  `package.json`,
  `config/pnpr/config.yaml`,
  `pnpm-lock.yaml`);
- survived: `package/agent-harness-shared/advisor` root files
  (`package.json`,
  `mise.toml`,
  `tsconfig.json`,
  rolldown config),
  `src/{index,ports,transcript-test-fixture}.ts`
  (latest versions,
  lint-clean,
  property-style port signatures,
  closed `AdvisorThemeColor` token union),
  its `dist/` build and `node_modules` link,
  and `package/pi-plugin/advisor/src/run-context.ts`;
- the only stash (`3f1dfd8c8a`) is an unrelated 2026-08-05
  translation-repair autostash,
  not this work.

Recovery plan (fork `40eef920` is executing it):
redo the `git mv` batches,
re-apply the recorded edits verbatim from its event log,
and commit in small units immediately after each coherent batch
(GCE),
never at the end.
Then `pnpm install` for the lockfile,
file-enforcer,
verification,
README.
First recovery commit landed 18:02:14 as `093f32832a`
(`refactor(agent-harness-shared-advisor): move the advisor core files`).

Cause candidates (open,
undetermined):

1. The landing machinery rewrites the index itself
   (`git reset --quiet <landed_tree> --`,
   `package/git-policy/cli/src/native/landing_index.rs:304`),
   and two subagent commits landed at 17:52:18 and 17:53:06,
   the exact claimed loss window;
   its e2e invariants `index-no-revert` and `worktree-preserved`
   name this failure mode.
2. `git checkout` and `git restore` are not guarded
   (see `GuardedCommand` above),
   so any session or stray agent can discard uncommitted state
   without a wrapper rejection
   (another session's handover at 18:05:52 warns of stray agent loops).
3. The fork's own action is not excluded.

The candidates were filed as GitHub issue `#1791`
for Claude,
the git wrapper's owner;
wrapper fixes are out of scope for this series.

### 2026-10-20 monochromatic-lint startup failure

Separate incident:
`monochromatic-lint` fails at startup for every file
with the config error `Unknown built-in rule rust/avoidable-closure`
in `monochromatic-lint.config.jsonc`
(reported by extraction fork `40eef920`,
which therefore could not run the markdown gate
and verified README rendering with `marked` instead).
The rule is documented and registered
(`package/linter/monochromatic-lint/README.md:274`,
`doc/decision/rust-closure-lint-dylint.md:111`),
so this is rule-registry or config-validation drift.
Correction 2026-10-20 (WR6):
that hypothesis was wrong.
Measured cause (subagent `5b28eefd`):
a stale resolved binary.
Bare-name lookup found published `monochromatic-lint` 0.2.0
via a stale command-context PATH entry
(`~/.local/share/mise/installs/cargo-monochromatic-lint/0.2.0/bin`),
while `mise.lock` pins 0.3.0,
whose rule registry registers `rust/avoidable-closure`
(commit `4b786ab4ad`).
0.2.0's compiled-in registry lacks both that rule
and the removed `rust/no-anonymous-functions`.
`mise run lint:monochromatic-lint` resolves 0.3.0
and lints the same files clean.
This is not the `deps-update-api-drift` class
(the consumer config was ahead of the resolved binary).
RESOLVED 2026-10-20 by subagent `5b28eefd`
(commits on `origin/main`):
`407481dedc` regression guard
(the repository configuration names only registered rules,
GFP-proven to fail on registry removal),
`14edcbf712` semantic line breaks in
`package/agent-harness-shared/advisor/README.md`
(gate `--fix`,
word sequence unchanged),
`725640eb57` troubleshooting record in
`doc/troubleshooting/mise-lock-pins-repo-tool-version.md`.
593 nextest tests pass;
markdown gate green on both advisor READMEs.
Caveat:
long-lived agent command contexts keep the stale PATH,
so bare `monochromatic-lint` there still runs 0.2.0
until the environment rebuilds;
invoke through `mise run`.
Follow-up DONE 2026-10-20 (subagent `8bb5ee46`;
`send_message` cannot resume finished subagents here,
so it ran as a fresh agent with a self-contained brief):
config-rejection diagnostics now self-identify the running binary
(`(monochromatic-lint <version>)` suffix on the whole
`ConfigError` class)
and hint at a stale resolved binary only when
the nearest `mise.lock` measures a strictly newer pin
(commits `d34026bb22`,
`d659a87347`,
`07cd829ea2`,
`6631927759`,
`e258a33c4a`;
603 nextest tests with 10 new guards;
four GFP removal proofs;
the honesty constraint proven by the equal-pin boundary).
Separate pre-existing findings it reported:
10 error-severity `markdown/semantic-line-breaks`
(fixed 2026-10-20 in `71352ac59d`,
gate green on `package/linter/monochromatic-lint`)
and a warn-level `rust/avoidable-closure` backlog
across 10 source files,
dispatched 2026-10-20 to subagent `720c4379`
for paydown under the config's ratchet policy
("Warnings until no finding is left").
Paydown DONE 2026-10-20 (subagent `720c4379`):
30 findings to zero across 10 source files
(commits `230883a259`,
`2deb5bfa79`,
`c7adda99a5`,
`fd9df3030c`,
`dc9ef661e0`,
`e62cbcce80`;
603 tests preserved;
9 suppression attributes cover 21 unrefactorable closures
(`catch_unwind`,
`spawn_scoped`,
`thread::scope`,
clap-derive),
each with a reason and a README suppression record).
Ratchet flipped 2026-10-20 (`773a6d0aa1`):
the new `closure-check-monochromatic-lint` config block
holds the package at error severity,
mirroring the `closure-check-ide` precedent.
GFP proof:
removing the `catch_unwind` suppression in
`markdown_source.rs` made the gate fail at the new severity
(`rust/avoidable-closure` reported at `error`),
restore returned it to green.
`file-enforcer` now owns the package's `check-cfg` block
(`9b0374ca95`,
`CARGO_DYLINT_DIRS`).
Generated `mise.toml` and `package/config/pnpr/config.yaml`
edits in the worktree encode `572a1354`'s in-flight
`package/dsh-plugin/advisor`;
they land with that package's commits.

### 2026-10-20 transient auto-push rejection

Commit `9830f476ed` saved locally,
auto-push rejected once by GitHub
(`GH006: Protected branch update failed ... protected branch hook
declined`).
A manual `git push` then succeeded
(`1dde8c5cd6..9830f476ed`),
and the branch ruleset holds only `copilot_code_review`
with `review_on_push`.
Treat a single GH006 rejection as transient and retry once;
if it repeats,
ask Claude
(who owns the git wrapper and remote settings).

### Unknowns

- Resolved:
  the shell-command-analyzer TS2339s were the unbash 6.0.0 AST reshape,
  not the TypeScript toolchain or the ownership branding.
- Resolved:
  no source drift hid behind the 38 build-class `lint:types` failures;
  subagent `bd3dc9b4` verified all 39 packages green after rebuild
  (2026-10-20).

## Next action

Implementation,
 in this order (user directives 2026-10-20: fix pre-existing failures too;
 use subagents aggressively):

1. DONE,
   commit `b710410e81`: move `package/pi-shared/model-selection` to
   `package/agent-harness-shared/model-selection`
   (dir plus name plus consumers).
2. DONE: fix the measured pre-existing failures
   (`3b9bb3ad6b`,
   `fce8efd40e`,
   `ad7a5a67b4`,
   `22f130ed5b`,
   `ea2812babc`,
   `c8d9af87d5`)
   and sweep every package `lint:types`
   (213 packages,
   170 pass,
   43 red triaged as above).
3. DONE 2026-10-20 (subagents,
   disjoint write scopes):
   shared core extraction and pi rewiring (`40eef920`,
   commits in the delegation list),
   the dependency API drift record (`94e9414e`,
   `601c69399e`),
   and the 38 build-class reds (`bd3dc9b4`).
4. DONE 2026-10-20:
   `git merge --ff-only origin/main` ran on a clean worktree and adopted
   `1dde8c5cd6`
   (the concurrent translation-repair NeuralWatt batch;
   the queued auto-push merges are absorbed).
   DONE 2026-10-20 (subagent `572a1354`):
   fold-ins and the DSH plugin landed
   (`16a22cd7de` `defaultReviewerModel` initial-reviewer behavior,
   `39d452035a` default-ranking seam in
   `src/default-ranking.ts`,
   `effcff526e` `package/dsh-plugin/advisor`,
   `3583eec279` install outputs,
   `ff2de2b5a3` synced `main`).
   Lead-verified:
   shared advisor and `dsh-plugin/advisor` `lint:types` and
   `test:unit` green
   (31 dsh test groups;
   shared suites cover the Q12 order,
   the ranking seam permutation guards,
   and the `defaultReviewerModel` config merge).
   Capability-gap deviations recorded in the dsh README
   (runtime pi imports kept external as peer deps,
   fabricated zero usage on derived history,
   unknown window 128_000,
   empty Schemastery `Config` row,
   no `verify:extension` analogue).
   ACTIVE (lead):
   fixture-profile verification,
   then the Q10 live install
   (`plugin_manager install_bundle`).
   Fixture 1 `advisor-fixture-minimal` created from the headless
   template and the bundle linked via `dsh plugin --profile ... add`;
   `--dump-config` composes `id: advisor`, `config: {}`.
   Measured fixture-surface gaps (diagnosed 2026-10-20):
   headless sessions mount no inspect tools,
   needing inserted rows `cordis-host-runner`,
   `cordis-inspect-providers`, and `tool-cordis`;
   fixtures do not share the live profile credential store;
   the `MISSING_CREDENTIAL` diagnostic sanctions exporting keys
   in the launching environment,
   and tool shells lack the keys the harness PID env carries.
   Workaround:
   disposable overlay `~/temp/agent/advisor-fixture-llm.patch.yml`
   (`llmgateway-devpass`/`mimo-v2.6-pro`,
   exported `LLMGATEWAY_DEVPASS_API_KEY`,
   inserted inspect rows).
   DONE 2026-10-20:
   troubleshooting docs
   `doc/troubleshooting/dsh-fixture-profile-plugin-verification.md`
   (`0a56240d1a`)
   and `doc/troubleshooting/dsh-cordis-inspect-query-input-coerced-to-string.md`
   (`c94dc5ed69`),
   both indexed in `doc/troubleshooting/README.md`.
   Measured defect (fixture 1, 2026-10-20):
   every `advisor` tool call failed with
   `prompt variable "{{model}}" has no value for this assembly`
   (section `deployment:persona-prefix`).
   Root cause:
   `buildDshAdvisorProjectContext` called `systemPrompt.assemble()`
   without the agent assembly context that the agent-loop's
   `model` and `provider` prompt variables read
   (`context.agent.options`).
   Fixed (`85434d0c40`):
   `DshAdvisorAgentSurface.assembleContext` forwards
   `assembleContextFor(agent, signal)` from `@deepseek-ai/dsh-agent`,
   matching the agent-loop's own assembly call shape.
   Measured defect 2 (fixture 1, 2026-10-20):
   after the first fix, calls failed with
   `advisor: no scoped models with configured auth`.
   Root cause:
   `createDshAdvisorRunContext` exposed no `getScopedModels` seam,
   so scope resolution fell into pi-only argv and settings branches
   and resolved zero entries in DSH.
   Fixed (`07ad8133c5`):
   the run context exposes `getScopedModels` backed by the catalog
   `getAvailable()` routes, the DSH equivalent of pi's live scope.
   Boundary re-probe (2026-10-20):
   the unknown-slug parity error now fires exactly
   (`advisor: model slug "no-such-model-slug" was not found in scoped
   models`) listing the ten scoped slugs,
   the eight `llmgateway-devpass` overlay models plus
   `deepseek-official/deepseek-flash` and `deepseek-official/deepseek-v4-pro`.
   Measured (fixture 1):
   agent sessions expose no command-invocation surface
   (no command tool or commands service),
   so `/advisor` parity moves to the user boundary after Q10,
   the plan's known gap.
   Fixture 2 PASS (2026-10-20, disposable home and project configs):
   default call served `llmgateway-devpass/deepseek-v4.1-flash-1017`
   (project `defaultReviewerModel: opencode-go/kimi-k3` unresolvable,
   falling through to the ranking pick, not global `claude-haiku-5-5`),
   and the reported deadline (`222222` minus elapsed) proves the project
   scalar `timeoutMs` won the merge;
   explicit `llmgateway-devpass/claude-haiku-5-5` served exactly.
   The result line `deadline remaining <timeoutMs - elapsed>ms`
   is the merged-config observable at the tool boundary.
   Fixture 1b PASS (2026-10-20, empty disposable home, non-project cwd):
   default call reported `deadline remaining 594995ms` after `5005ms`
   elapsed (`600000` equals `DEFAULT_TIMEOUT_MS`: config paths absent)
   and served the ranking pick `llmgateway-devpass/deepseek-v4.1-flash-1017`;
   `llmgateway-devpass/claude` raised the exact not-found parity error
   (slug matching is exact,
   so the ambiguous branch needs duplicate match keys and stays unit-covered);
   `model: ""` served the default by design:
   shared `normalizeModelArgument` drops blank model values
   (`agent-harness-shared/advisor/src/tool-params.ts`),
   so `model slug must not be empty` is a command-form error only.
   Q10 live install: DONE 2026-10-20
   (`plugin_manager install_bundle`: `application: applied`,
   `warnings: []`; `--dump-config` shows `id: advisor`, `config: {}`;
   the web app has no CLI task mode,
   so the live tool catalog is confirmed in a fresh GUI chat).
   QUEUED at the user boundary (tracked in issue `#1792`):
   new GUI chat `/advisor status` (expect the transcribed config lines),
   `/advisor off` (tool masked) and `/advisor on` (restored).
   Measured (2026-10-20):
   a leading-slash task line (`/advisor status`) reaches headless
   agents as an ordinary user message with no command dispatch,
   closing the dispatch probe.
   Parallel (subagent `5b28eefd`): DONE
   (see the monochromatic-lint incident section).
5. Tests, lint, and `mise` tasks for every new and moved package;
   behavior picks recorded per Q5 and Q6
   plus the 2026-10-20 reviewer-fallback decision
   (initial reviewer,
   serial fallback kept,
   explicit is exact).
6. DONE 2026-10-20:
   fixture-profile verification (fixtures 1, 2, and the empty-home 1b;
   fixture 3 toggle parity deferred to the user boundary)
   and the Q10 live install (`application: applied`, `warnings: []`).
7. DONE 2026-10-20:
   READMEs (dsh capability gaps now record the scope mapping
   and the command-parity boundary),
   planning doc (fixture findings, Q10 deviation, completion status),
   GitHub issue `#1792` (`needs-triage`),
   and this handover's completion record.
8. DONE 2026-10-20:
   the git-wrapper defects and UX problems are filed as
   GitHub issue `#1791`
   (`cli-git: uncommitted work can vanish with no guard and no log`),
   `needs-triage`,
   for Claude,
   who owns the git wrapper.
   No wrapper fixes in this series.
   Filed findings:
   unguarded `checkout`/`restore` can discard uncommitted state,
   the landing `reset --quiet` index rewrite can revert staged work,
   read-only `git stash` subcommands are over-blocked,
   no audit trail exists for state-discarding operations,
   and the auto-push ff-only sync advice cannot be followed
   in dirty agent worktrees.

## Completion record (2026-10-20)

The series landed as commits `16a22cd7de`, `39d452035a`, `effcff526e`,
`3583eec279`, `85434d0c40`, and `07ad8133c5`, with doc commits alongside.
The Q5 end-of-series picks report:
merged-copy disagreements defaulted to richer with no approval round,
and the recorded picks (initial-reviewer `defaultReviewerModel`,
explicit exact selection, serial fallback kept, the Q12 ranking order,
derived-history usage fabrication, unknown window `128_000`)
are documented in `package/dsh-plugin/advisor/README.md`
and the Q5/Q6 sections of this handover,
each pick landed with its test in the same commit.
Verification: fixtures 1, 2, and 1b green with the measured evidence above;
fixture 3 toggle parity and `/advisor` command parity wait on the user
boundary (issue `#1792`).
Rollback for the live install: `plugin_manager remove_bundle` with the
absolute `package/dsh-plugin/advisor` target,
then re-check `dsh --profile web --dump-config`.
Residue to clean: fixture profile `advisor-fixture-minimal`
and `~/temp/agent/advisor-fixture-*` inputs.
