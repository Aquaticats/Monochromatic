# File-enforcer Rust rewrite

## Status

Being grilled with the user, started 2026-10-29.
Nothing here is accepted yet;
decisions land in this file as the user answers them.

Superseded next action (2026-10-29; the current one is the section "Next action (2026-10-29, after confirmation)"):
before confirming shared understanding,
the user asked for reviews by `openai/gpt-6.1-sol`,
`openai/gpt-6-astra`,
and `opencode-go/kimi-k3.1-pro` through `pi`.
All three are running with this file,
the four `doc/research/file-enforcer-rust-rewrite-*.md` reports,
`doc/decision/monorepo-manager-all-rust.md`,
and `doc/decision/mpm-toolchain-adoption.md` attached;
output goes to `${HOME}/temp/agent/fe-rust/review-{sol,astra,kimi}.md`.
When they return,
each finding is checked against the repo,
the confirmed ones go to the user as questions,
and only after the user confirms does the decision record `doc/decision/file-enforcer-rust-rewrite.md` get written.
No code before the user accepts that record.

### Self-critique requested by the user (2026-10-29)

The user wrote:
"I am seeing a lot of things I don't like,
but I'd rather not say anything specific right now,"
and asked what Claude and its advisor think of the plan.
Both found:

- The plan ports the old tool along new seams.
  Every old output became a rule kind,
  about half of them repository-specific built-ins,
  and the toolchain concepts (routes,
  providers,
  exposure farm,
  postinstall links,
  `bin-paths`,
  global scope) moved into mpm-shadow intact.
  The old sync calls mpm once (`mpm --rustup restore`) and installs everything else itself;
  the plan ports that bypass instead of asking what mpm should do.
  No question asked which generated files should exist at all,
  or what the user wants that the old tool never did.
- Withdrawn:
  a point that the program's size and its one-switch cutover were the main risk.
  The user answered "Massiveness and single switchover are not problems." (2026-10-29)
- Three toolchain files (plan,
  resolution,
  lock) exist partly to repair a two-writer hazard the plan/lock split created.
- Claims that do not hold:
  a schemars schema tracks the Rust types,
  not the hand-written decoder,
  so it can drift;
  the existence-based `mise.toml` PATH makes `check` machine-dependent;
  the JSONC canonical-emit tradeoff may concern no target at all;
  "meow can embed" is weak support for the crate split while meow has no code.
- Process:
  questions used unexplained internal terms ("resolver families",
  "declaration fields"),
  one option rested on an untraced mechanism,
  one con was retracted,
  and the question tree followed the old package's module tree.
- The advisor also retracted its own claim that `package/dependency-catalog/core-rust` is a Rust crate;
  it is TypeScript.

Asked 2026-10-29 how to proceed after the critique,
the user chose "Reviews first,
then restart".
Next action:
when the sol,
astra,
and re-run kimi reviews return
(the first kimi run produced only a preamble;
its re-run writes `${HOME}/temp/agent/fe-rust/review-kimi-2.md`),
verify each finding against the repo,
then ask a root round with those findings folded in:
for each generated file keep,
drop,
or replace it (symlink,
build step,
`mpm restore`);
what mpm does versus mpm-shadow;
and what the user wants that the old tool never did.
The decisions in this file are then re-checked against those root answers.

### Outside review findings, checked against the repository (2026-10-29)

Reviews:
[`doc/research/file-enforcer-rust-rewrite-review-sol.md`](../research/file-enforcer-rust-rewrite-review-sol.md)
and
[`doc/research/file-enforcer-rust-rewrite-review-astra.md`](../research/file-enforcer-rust-rewrite-review-astra.md);
and
[`doc/research/file-enforcer-rust-rewrite-review-kimi.md`](../research/file-enforcer-rust-rewrite-review-kimi.md)
(second run).
All three worked from the attached text only.

Raised only by kimi,
for the root round:

- No initial resolution file:
  the conversion must split the existing repository and global locks into plan and resolution,
  keeping every version,
  URL,
  and SHA-256,
  and the new render must reproduce the old lock before the switch.
- A new npm or PyPI tool cannot be added:
  it has no resolved version for sync to install,
  so "sync first and retry" cannot make `mpm outdated` see it.
- Global lock and manifest rendering has no owner,
  since "committed inputs only" excludes `~/.config`.
- Global-only routes (`cargo-git`,
  `go-install`) escape a repository-only comparison.
- `update --check` has no recorded fate.
- Output file modes (0600 for both forbidden-strings files and the global plan) are not part of the write contract.
- The global plan's only backup would sit in agent scratch.
- A GitHub token is a bootstrap input for containers and CI.
- Catalog writes need whole-run validation before any file is replaced.

Confirmed by reading the repository:

- CI regenerates before it diffs:
  `.github/workflows/advisory-core.yml` runs `task-run prepare:pnpm`,
  which runs the sync,
  then `git diff --exit-code`;
  swapping the diff for `check` while keeping the prepare step would pass on drift.
- The IDE launches the old script by path:
  `package/ide/app/src/language/toolchain.rs:65` (`ENTRY_POINT`),
  its notices and tests,
  and `package/ide/app/bin/inspect-language-toolchain.ts:41,86,288`.
- The global toolchain folder also holds `plan.artifacts.data.ts` (28,691 bytes),
  which the conversion decision omits.
- Postinstall has a `run` form as well as `link`:
  `search-shadow link .cache/search-shadow-bin`
  (`package/file-enforcer/core/src/toolchain/plan.tools.data.ts:353-370`).
- Rust toolchain version lookup (rustup channel manifests,
  `static.rust-lang.org`) lives in `update.resolvers.rust.ts` and has no owner in the update decision.
- The old watch warns when a generated file is edited (`package/file-enforcer/core/src/watch/notify.ts:72`).
- Two consumers outside the deletion plan still depend on the old package:
  `package/test-fixture/file-enforcer-perf/package.json:9`
  and `package/oxlint-plugin/prefer-readonly-parameter-type/src/workspace-source-effect.unit.test.ts:22`.
- No symlinks exist in `.claude/skills` or `.factory/skills` today;
  the symlink-containment finding is about what the mirror rule must specify.

Weakened:

- Scanner cache staleness:
  `package/forbidden-strings/cli/README.md` keys the compiled artifact by the SHA-256 of the exact rules text,
  and "A later scan repairs a missing or rejected artifact automatically",
  so a rules change cannot reuse a stale artifact.
  Source-backed (`runtime_cache/envelope.rs:260`),
  not executed;
  a run on a throwaway rules file stays a gate before implementation.

Design gaps both reviews raise,
for the root round:
the update-then-render sequence,
an ordering model so rules read each other's planned output,
publishing before the switch,
the IDE launcher,
whether `mpm outdated` follows npm's `latest` tag (including downgrades),
writer coordination for `watch` against concurrent edits,
schema and hand-written decoder agreement,
clap's own help and error output under JSON-lines-only,
per-output coverage (license pruning,
create-if-absent seeds,
file modes),
and watch rediscovering new packages and matched paths.

## Accepted (2026-10-29)

The user accepted the record ("Accept");
it is [`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md),
and the amendments to `doc/decision/monorepo-manager-all-rust.md` and `doc/decision/mpm-toolchain-adoption.md` are applied.
Next:
the gates before implementation,
then one task per area,
tracked in issue #1818 (which points at the `prototype/fe-mpm-files` branch).
The prototype's images and worktrees were removed with the user's approval;
the branch stays locally and on `origin`.

Gates running (2026-10-29):
four choosing-technology vets (`toml_edit`,
`browserslist-rs`,
`schemars`,
an XML editor),
each writing `doc/audit/tech-file-enforcer-*-vet-2026-10-29.md` uncommitted for review,
and one agent running the three live probes in sequence,
writing `${HOME}/temp/agent/fe-rust/gate-probes.md`.
When they return:
review and commit the vets,
ask the user any question a vet ends with,
record the probe verdicts in issue #1818,
then start the first implementation area.

Update (2026-10-29):
the user halted the four vets for handoff,
then approved `toml_edit`,
`browserslist-rs`,
and `schemars` without vets
and chose an in-repo XML edit crate,
and wrote:
"Yes mark superseded,
but don't discard any work.
Even if work is superseded,
it should still become knowledge."
So each halted vet draft is committed whole under `doc/audit/`,
with only a superseded status line added,
and the XML draft is linked from the decision record as input to the XML crate.
The live probes keep running.

Reboot hold (2026-10-29):
the user plans a reboot and asked to start no new work and discard nothing,
including subagent state.
Done and committed:
the four vet drafts (`b34ed80`),
the `serde_json` approval (`365bd1ecae`).
In flight:
the live-probe agent,
checkpointing to `${HOME}/temp/agent/fe-rust/gate-probes.md` after each probe,
with containers under `systemd-run` units named `fe-gate-*`.
Probes finished 2026-10-29
([`doc/research/file-enforcer-rust-rewrite-gate-probes.md`](../research/file-enforcer-rust-rewrite-gate-probes.md)):

- forbidden-strings:
  pass.
  A changed rules text is a cache miss keyed by its SHA-256,
  never a stale hit;
  today a comma-joined `--rules` value is one filename and no optional entry exists,
  the baseline the `?` list changes.
- `browserslist-rs` 0.21.2:
  mechanics pass,
  data parity fails.
  It resolves the same queries in the same shape,
  but its data is older:
  14 targets against 18,
  `chrome151` against `chrome156`,
  `firefox140` (hard-coded ESR) against `firefox153`,
  and Node up to 26.8.0 against 26.12.0.
  The repo's `.browserslistrc` has no sections,
  so "production" means the defaults.
  This is the data drift the user accepted when choosing the crate,
  now measured.
- `mpm outdated` reading:
  the candidate rule fails as written.
  mpm 8.2.0 drops a failed or missing manager from its JSON with exit 0,
  reports an unreachable registry only in an `errors` array,
  and gives no row both when installed equals latest and when installed is newer than latest;
  the flag is `--table-format json`,
  not `--output-format json`.
  How to read it goes back to the user.
- Side finding,
  not investigated:
  tag `v8.2.0` resolves to `99eb596b`,
  while `doc/decision/mpm-toolchain-adoption.md` cites `f095931a`.

The user chose "Let it finish":
reboot is called safe only after all three probes finish,
their containers and units stop,
and the report is committed.
After the reboot:
read that checkpoint,
resume or rerun only the unfinished probes,
record the verdicts in issue #1818,
then start the first implementation area (tasks 20 to 26).

## Superseded next action (2026-10-29, after confirmation)

The user confirmed a shared understanding ("Shared understanding"),
with one addition adopted under the user's open veto:
mpm-shadow writes a digest of the lock's contents,
and `lock --check` recomputes it,
so a hand edit of a resolved version fails the gate.
The decision record draft is
[`file-enforcer-rust-rewrite-decision.md`](file-enforcer-rust-rewrite-decision.md);
it moves to `doc/decision/` only on the user's explicit acceptance (`DRR` in `AGENTS.md`).
No code before acceptance.

## Superseded next action (2026-10-29, second review round)

Asked how to close the grilling,
the user chose "Re-review,
then confirm".
A second round of sol,
astra,
and kimi reviews is running against this file,
the inventory,
toolchain-split,
replacements,
and mise conf.d research,
and the two amended decisions;
output goes to `${HOME}/temp/agent/fe-rust/review2-{sol,astra,kimi}.md`.
When they return:
verify each finding against the repository,
ask the user about those that hold,
then confirm and write `doc/decision/file-enforcer-rust-rewrite.md` for acceptance.
No code before acceptance.

Update:
the three second-round reviews returned and are recorded under "Second review round".
Still open:
who writes and checks `mpm.packages.toml` and `mpm.managers.toml`,
and file-enforcer's toolchain role.
A prototype agent is running real mpm in podman in a linked worktree,
on the throwaway branch `prototype/fe-mpm-files`,
testing four candidates (mpm-shadow owns and checks,
mpm-shadow writes and file-enforcer checks through a shared crate,
file-enforcer renders,
or the files should not exist);
its report goes to `${HOME}/temp/agent/fe-rust/proto-mpm-files.md`.
After it:
verify the report,
ask the user with its evidence,
then the shared-understanding confirmation and the decision record.

## Current design (consolidated 2026-10-29)

The live decisions,
with superseded ones left out;
the "Decisions" section holds each one's question,
answer,
and history.

- Tools:
  `package/file-enforcer/core-rust` (library) and `package/file-enforcer/cli` (clap binary `file-enforcer`),
  standalone,
  embeddable by meow later;
  config `file-enforcer.config.jsonc` read with `monochromatic-jsonc-edit`,
  decoded by hand;
  output is JSON lines only,
  including clap's help and errors.
- Config model:
  fixed Rust rule kinds with structured JSONC values,
  no string expressions;
  each run plans every rule against a virtual view of planned outputs,
  validates the whole run,
  then writes;
  file modes are part of a rule's contract.
- Server:
  one auto-started server per worktree serializes all writes over an owner-only Unix socket
  (newline-delimited JSON-RPC 2.0) and exits when idle;
  `write`,
  `check`,
  `watch`,
  and `explain` are clients;
  the server watches inputs,
  outputs,
  and glob roots while a `watch` client is attached,
  and warns on hand edits to generated files.
- Commands:
  `write`,
  `check` (with `--expected-dir <dir>` for standard diff tools),
  `watch`,
  `explain <path>`,
  `user write`,
  `user check`;
  exit codes 0,
  1,
  2.
- Outputs file-enforcer writes:
  `CLAUDE.md` (preamble plus `AGENTS.md`),
  root `LICENSE`,
  every package's `LICENSES/` copies,
  the `.claude/skills` and `.factory/skills` copies (targets owned outright),
  per-crate `Cargo.toml` keys,
  `[workspace.dependencies]` through the dependency-catalog Rust library,
  `.browserslistrc.resolved.local.json` through `browserslist-rs`,
  `.config/mise/conf.d/10-bin.local.toml`,
  and,
  through `user write`,
  the JetBrains Harper policy.
- file-enforcer has no toolchain role.
- Deleted outright:
  `mpm.packages.toml` and `mpm.managers.toml`,
  in both the repository and the global scope.
- Gone from file-enforcer:
  the generated root `mise.toml` (now hand-written plus committed `conf.d`),
  the rules file and its seed (the scanner reads a comma-separated list),
  the scanner step,
  task-header quoting,
  the `CONTEXT.md` guard,
  the OS package installer,
  the staleness cache,
  and `exec`.
- Schema:
  committed `file-enforcer.schema.json` with a cargo golden test against the decoder,
  mapped in `.idea/jsonSchemas.xml`.
- Toolchain (mpm-shadow):
  `sync`,
  `status`,
  `update [--check]`,
  `lock` (offline),
  `bin-paths`,
  each with `--global`;
  the only writer and checker of `toolchain.lock.json`
  (`mpm-shadow lock --check` is the toolchain drift gate in CI beside `file-enforcer check`);
  rustup channels and cargo-binstall tools install directly from the lock;
  first versions of new npm and PyPI tools from the registries,
  later ones from `mpm outdated`;
  native lookups for the rest,
  including Rust channels;
  postinstall links and commands;
  bootstrap by prebuilt archive with lock SHA-256 or `cargo install`.
- Elsewhere:
  pnpr generates its own `config.yaml`;
  the container runner moves to `package/test-harness/core-matrix`;
  vm-builder gets a private process runner.
- Cutover:
  one switch after everything is built,
  published,
  fetched by a clean container,
  and compared with the old output;
  the global plan is converted directly with a 0600 backup kept beside it.

## Request

"Migrate to Rust + clap + config with jsonc for file-enforcer.
Ground up rewrite.
No need to preserve anything." (user, 2026-10-29)

## Constraints stated by the user

- "Consider the original file-enforcer bad in every way and don't treat it as Jesus." (user, 2026-10-29)
  The TypeScript package at `package/file-enforcer/core` is evidence only of which files the repo needs managed.
  Its builder API,
  subsystem boundaries,
  CLI shape,
  staleness manifest,
  locking,
  and config-as-TypeScript model are not requirements and are not a design to port.

## Decisions

### Standalone tool that meow can embed

Asked 2026-10-29 how the rewrite relates to meow,
whose accepted decision (`doc/decision/monorepo-manager-all-rust.md`, 2026-09-16)
puts file enforcement inside the single meow binary with an HCL config and daemon-owned watch mode.
The user chose "Standalone, meow can embed".

- The engine is a library crate and the CLI is a clap binary over it;
  meow may depend on the library later instead of rewriting it.
- The tool has its own watch mode until a meow daemon exists.
- The config is JSONC,
  following the cli-git and linter rewrites (`doc/decision/cli-git-rust-rewrite.md`),
  which also moved off HCL.
- Consequence to record when this is accepted:
  amend `doc/decision/monorepo-manager-all-rust.md` so meow links this crate and its file-enforcement config is JSONC.

Evidence at the time:
meow is plan-only with no code
(`doc/handover/monorepo-manager.md`, "Design research; no product code"),
last designed 2026-09-21,
and issue #1799 prefers an existing tool over building a monorepo manager.

### JSONC through the in-repo jsonc-edit crate

Asked 2026-10-29 which JSONC stack reads `file-enforcer.config.jsonc` and edits JSONC targets;
the user chose "In-repo jsonc-edit" over `jsonc-parser` (dprint) and the `config` crate (config-rs).

- `monochromatic-jsonc-edit` (`package/jsonc-edit/core-rust`) is the accepted foundation
  (`doc/decision/jsonc-edit-rust-foundation.md`)
  and the loader for `cli-git.config.jsonc` and `monochromatic-lint.config.jsonc`.
- The config is decoded by hand per field,
  as `package/git-policy/cli/src/native/config_parse.rs` does;
  parse errors map the crate's byte offset to `path:line:col`.
- Emit is canonical,
  so editing a hand-formatted JSONC target re-formats it.
- Missing features (serde,
  value spans) get added to the crate,
  not worked around.
- Why not config-rs (`config` 0.15.27, researched 2026-10-29):
  no JSONC format (only JSON5 through `FileFormat::Json5`),
  no line or column on `ConfigError`,
  environment keys lowercased,
  and read-only.
  Evidence:
  [`doc/research/file-enforcer-rust-rewrite-crates.md`](../research/file-enforcer-rust-rewrite-crates.md),
  whose claims come from reading source and docs,
  not from running the crates.

## Research

- [`doc/research/file-enforcer-rust-rewrite-meow-status.md`](../research/file-enforcer-rust-rewrite-meow-status.md):
  meow's state and its decisions about file enforcement.
- [`doc/research/file-enforcer-rust-rewrite-inventory.md`](../research/file-enforcer-rust-rewrite-inventory.md):
  every file the repo needs managed,
  stated as outcomes,
  plus the old tool's flaws.
- [`doc/research/file-enforcer-rust-rewrite-crates.md`](../research/file-enforcer-rust-rewrite-crates.md):
  clap,
  config-rs,
  JSONC,
  schema,
  and structured-edit crates.

### Config model: rule kinds with structured values

Asked 2026-10-29 how much logic the JSONC config carries;
the user chose "Rule kinds + structured values"
over string expressions and over named Rust rules with JSONC toggles.

- Rust implements a fixed set of rule kinds
  (concatenate,
  mirror a directory,
  set TOML keys where a table exists,
  append a block if absent,
  fan out over packages,
  plus named built-ins).
- JSONC values are literals or tagged objects such as `{"file": ...}`,
  `{"var": "dir"}`,
  and `{"concat": [...]}`;
  no string expression language.
- Every input and output path is known from the config without running anything,
  so `check` is exact and a JSON Schema covers the whole file.

### Rust catalog sync: a Rust library in dependency-catalog

Asked 2026-10-29 who renders each crate's `[workspace.dependencies]` from `rust-catalog.toml`;
the user chose "Rust lib in dependency-catalog".

- A new Rust library crate in `package/dependency-catalog` owns parsing and rendering the catalog.
- file-enforcer links it behind a `rust-catalog` rule kind,
  so file-enforcer stays the one writer of every `Cargo.toml`.
- The TypeScript update,
  verify,
  and tighten tools keep their own parser until they move.

### pnpr owns its publishable-package list

Asked 2026-10-29 who produces `package/config/pnpr/config.yaml`;
the user chose "pnpr owns it, own check".

- `package/config/pnpr` generates the file with its own write and check tasks,
  reusing its `isTypeScriptSourcePath`;
  file-enforcer is not involved.
- CI runs pnpr's check beside file-enforcer's.

### Toolchain: split between file-enforcer and the mpm wrapper

Asked 2026-10-29 where toolchain sync,
status,
update,
and `bin-paths` go.
The user rejected all three options (own TypeScript package,
separate Rust tool,
inside file-enforcer) and wrote:
"Some needs to be folded into Rust file-enforcer,
some needs to be folded into our mpm wrapper."

- The mpm wrapper is `package/mpm-shadow/cli` (Rust,
  clap).
- Research:
  [`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`](../research/file-enforcer-rust-rewrite-toolchain-split.md).

Then asked 2026-10-29 how to split;
the user chose "Split by side effect":

- file-enforcer renders,
  from committed inputs only,
  `mpm.packages.toml` and `mpm.managers.toml` from the lock,
  and the lock's declaration fields from the plan.
- mpm-shadow gains `sync`,
  `status`,
  and `update` (network resolution writing lock versions and artifacts),
  each with `--global` for the machine scope,
  plus the `.cache/toolchain-bin` exposure farm and postinstall links.
  Under the `mpm` name it stays a pass-through plus prune.
- The bounded container runner moves to `package/test-harness/core-matrix`.
- Dropped as unused:
  `clearExposure`,
  `symlinkOrCopy`,
  `.cache/toolchain.env.json` (no reader),
  and the TypeScript plan loaders.

### Toolchain plan: JSONC plan and JSON lock

Asked 2026-10-29 where the hand-written plan lives;
the user chose "JSONC plan + JSON lock" over making the lock the only file.

- `toolchain.plan.jsonc` holds intent
  (id,
  request,
  route,
  bins,
  os,
  options)
  with rationale comments.
- `toolchain.lock.json` stays strict JSON for its `jq` readers
  (`Dockerfile:24-45`,
  `.github/actions/task-cli-setup/action.yml:42-46`,
  `.github/workflows/music-player-release.yml`).
- A file-enforcer rule projects the plan's fields into the lock,
  and `check` fails on drift.

### Lock writer: a separate resolution file

Asked 2026-10-29 who writes `toolchain.lock.json`;
the user chose "Separate resolution file".

- `mpm-shadow update` writes only `toolchain.resolution.json`
  (strict JSON:
  per tool,
  version and artifacts with URL and SHA-256).
- file-enforcer renders `toolchain.lock.json` from `toolchain.plan.jsonc` plus `toolchain.resolution.json`
  and is the lock's only writer,
  so `check` rebuilds the whole lock from committed inputs.

### Bootstrap: prebuilt mpm-shadow and cargo install

Asked 2026-10-29 how a fresh machine gets its first `mpm-shadow`;
the user answered "A+B":
both a prebuilt archive and `cargo install`.

- `mpm-shadow` becomes an archive-route plan entry with release URLs and SHA-256 in the lock;
  `Dockerfile` and CI fetch it with `jq`,
  `curl`,
  and `sha256sum`,
  as they already do for Node and pnpm (`Dockerfile:24-45`).
- `cargo install monochromatic-mpm-shadow` is the other documented path,
  and the only one on platforms without a prebuilt archive.
- A runbook step covers people's machines,
  since `AGENTS.md` (`SCR`) forbids bash and PowerShell scripts.

### bin-paths: an mpm-shadow subcommand

Asked 2026-10-29;
the user chose "mpm-shadow bin-paths".
`mpm-shadow bin-paths` prints the same JSON (`exposureDir`,
`pathDirs`,
`bins`) that `package/ide/app/src/language/toolchain.rs` parses today,
so mpm-shadow stays the only owner of the exposure layout.

### Update: mpm where it can, ported lookups elsewhere

Asked 2026-10-29 (re-asked in plainer words after the user wrote "What is going on? I don't understand.")
how `mpm-shadow update` finds new versions;
the user chose "Ask mpm where it can" over porting every lookup (my recommendation).

- npm and PyPI tools:
  `mpm outdated --output-format json`.
- Ported to Rust:
  GitHub releases,
  nodejs.org,
  crates.io (mpm's `cargo` manager has no `outdated`,
  `doc/research/mpm-run-capabilities.md`),
  go,
  zig,
  Adoptium,
  Android,
  conda.
- Accepted limits:
  `mpm outdated` sees only tools installed on the machine running it,
  and reports no download hashes.
- A planned npm or PyPI tool that is not installed:
  asked 2026-10-29,
  the user answered "Require sync first,
  and auto-sync on such failures and retry."
  `update` syncs the missing tools,
  then retries the lookup once;
  a second failure is a JSON-lines error naming each tool.
  Read narrowly as syncing only the missing tools and retrying once;
  a full sync or more retries would be a broader reading.
- Retracted from the first asking:
  my con that mpm "drops versions" described mpm's restore (install) behavior
  (`doc/decision/mpm-toolchain-adoption.md:389-399`),
  not version lookup.

### browserslist through browserslist-rs

Asked 2026-10-29 what happens to `.browserslistrc.resolved.local.json`;
the user chose "Rust rule kind (browserslist-rs)" over moving resolution into rolldown's config
(my recommendation) and over spawning Node.

- A `browserslist` rule kind resolves `.browserslistrc`'s production section with `browserslist-rs`
  (0.21.2,
  2026-09-16) and writes the same JSON array for `package/config/rolldown/src/browserslist-targets.ts`.
- Accepted tradeoff:
  the crate bundles its own browser data,
  versioned with the crate rather than the pnpm lock,
  so its targets can differ from what JavaScript tooling computes.
- `browserslist-rs` needs a choosing-technology vet before it enters `rust-catalog.toml`.

### Out-of-repo rules: per-user config, opt-in

Asked 2026-10-29 what happens to the Harper policy written into JetBrains state under `~/.config/JetBrains/`;
the user chose "Per-user config, opt-in".

- A per-user JSONC config under `~/.config` holds rules that write outside the repo,
  matching `doc/decision/monorepo-manager-per-user-config.md`.
- A separate subcommand applies it;
  the default `write` and `check` touch only repo files.

### No scanner step

Asked 2026-10-29 about the post-write `forbidden-strings compile-rules` run;
the user chose "Drop it".
file-enforcer writes `.cache/forbidden-strings.rules.txt` only,
and forbidden-strings compiles from that text when its cache is missing or stale
(`package/forbidden-strings/cli/src/runtime_cache/warning_tests.rs:23`).

### Crates: core-rust library and cli binary

Asked 2026-10-29 how to split crates so meow can embed the engine;
the user chose "core-rust lib + cli bin".

- `package/file-enforcer/core-rust`:
  library `monochromatic-file-enforcer`,
  the engine and rule kinds,
  no clap.
- `package/file-enforcer/cli`:
  binary `file-enforcer`,
  clap,
  JSON-lines output,
  exit codes.
- `package/file-enforcer/core` (TypeScript) is deleted at cutover.

### Config schema: generated and self-enforced

Asked 2026-10-29 whether the config gets a JSON Schema;
the user chose "Generated, self-enforced".

- The schema is derived from the Rust config types with `schemars`,
  which needs a choosing-technology vet.
- `file-enforcer.schema.json` is committed and kept current by a file-enforcer rule,
  so `check` fails when it is stale;
  the config's `"$schema"` key points at it.
- Unverified:
  JetBrains support for `$schema` in `.jsonc` files.

### mise task-header quoting: not normalized

Asked 2026-10-29;
the user chose "Drop it".
Both `[tasks.'x']` and `[tasks."x"]` stay as written,
since nothing reads the spelling and Mise is due to be removed.

### Root mise.toml PATH: existence-based, as today

Asked 2026-10-29 how `_.path` (`mise.toml:1262`) lists `package/*/*/node_modules/.bin` directories;
the user chose "Keep existence-based",
noting:
"The existence of option 1 means you don't understand how pnpm, mise, node_modules, path works."

- The rule globs the `node_modules/.bin` directories that exist and sorts them,
  as the TypeScript generator does (`file-enforcer.config.ts:722-745`),
  but skips empty ones:
  asked 2026-10-29,
  the user chose "Skip empty .bin" after the measurement found 18 of the 55 empty.
- The mechanism and counts are recorded in
  `doc/troubleshooting/pnpm-undeclared-workspace-links-survive-install.md`,
  "When a package gets node_modules/.bin",
  where the user chose to keep them instead of an `AGENTS.md` rule.
- Retracted:
  my option 1 derived the list from every package with a `package.json`.
  The user's correction:
  "You cannot meaningfully determine if a .bin will exist from just reading package.json."
  Whether a `.bin` exists depends on the resolved dependencies' bins and on how pnpm links them,
  which `package.json` does not determine,
  so the existing directories are the only reliable signal
  and option 1 would have named directories no install creates.
  Measured 2026-10-29 in the main checkout:
  223 `package/*/*/package.json`,
  205 `package/*/*/node_modules`,
  55 `package/*/*/node_modules/.bin`.
- Accepted tradeoff:
  issue #335 records the list changing between checkouts after filtered or stale installs,
  so `check` reports `mise.toml` drift on such a machine until a full `pnpm install` runs.

### Cutover: one switch

Asked 2026-10-29;
the user chose "One switch" over two stages (my recommendation).

- Everything is built first:
  the Rust file-enforcer,
  the dependency-catalog Rust library,
  the mpm-shadow subcommands,
  and the plan conversion.
- Old and new output are compared on this repo,
  then one commit switches tasks,
  CI,
  and `Dockerfile`,
  and deletes `package/file-enforcer/core`.
- Until that commit,
  the TypeScript tool keeps producing every managed file,
  including the new crates' own `Cargo.toml`.

### Global toolchain plan: converted directly

Asked 2026-10-29 about `~/.config/monochromatic-dev/toolchain/plan.tools.data.ts`
(mode 0600,
59 entries);
the user answered "Convert,
you just do it,
no need for my review."

- At the switch,
  a one-off converter writes `toolchain.plan.jsonc` beside it,
  keeping comments,
  at mode 0600,
  and replaces the `.data.ts`.
- The old file is backed up under `${HOME}/temp/agent` first.

### Skill mirror owns its targets

Asked 2026-10-29 how the mirror of `.agents/skills` into `.claude/skills` and `.factory/skills` decides deletions;
the user chose "Mirror owns the targets".

- Both targets are gitignored (`.gitignore:125-126`)
  and held only the 10 mirrored skills plus `.agents-mirror-manifest.json` when asked.
- Target contents equal `.agents/skills`;
  any other entry is drift,
  which `check` lists and `write` removes.
- The manifest files go away.

### Root round (2026-10-29)

Research:
[`doc/research/file-enforcer-rust-rewrite-replacements.md`](../research/file-enforcer-rust-rewrite-replacements.md).

#### CLAUDE.md stays generated

The user chose "Keep generating it",
noting:
"With closed source software,
you just can't assert anything."
The research's passing `@AGENTS.md` import probe on Claude Code 2.1.312 is not a contract
for a closed-source tool that updates itself,
so the generated preamble plus `AGENTS.md` stays.

#### Skill mirrors stay copies

The user chose "Keep the copy mirror",
noting:
"Once again,
closed source software."
Symlink loading in Claude Code and Factory's `.agents/skills` compatibility path are observed or documented behavior
of closed-source tools,
so the copies stay,
with the earlier "Mirror owns the targets" semantics.

#### Rules file: the scanner reads several files

The user answered:
"Teach it to read >1 files,
but make the syntax forbidden-strings.append.txt,forbidden-strings.append.local.txt to avoid repeating the --rules flag."

- `forbidden-strings --rules` takes one comma-separated list,
  and the `FORBIDDEN_STRINGS_RULES` variable takes the same form;
  cli-git's config names the files as a JSONC array.
- The local file is optional;
  the scanner joins the texts in memory before compiling.
- file-enforcer stops writing `.cache/forbidden-strings.rules.txt` and the seed.
- Tradeoff of the delimited form:
  a path containing a comma cannot be named.

#### JetBrains policy: keep user write

The user chose "Keep 'file-enforcer user write'",
noting:
"The problem is humans can't remember.
Especially with auto update."
The per-user config and its subcommand stay,
so IDE auto-updates get the policy reapplied by a command rather than a remembered manual import.

#### mise.toml: committed conf.d files (re-asked with full evidence)

Re-asked after the source-level study
[`doc/research/file-enforcer-rust-rewrite-mise-confd.md`](../research/file-enforcer-rust-rewrite-mise-confd.md);
the user chose "conf.d layout",
replacing the earlier decision to keep generating `mise.toml`.

- `mise.no-env.toml` is renamed to `mise.toml` and hand-written.
- `.config/mise/conf.d/20-env.toml` (committed) holds the sops `_.file`,
  the `.cache/*-bin` directories,
  `node_modules/.bin`,
  the Slint variables,
  and `FORBIDDEN_STRINGS_RULES`.
- `.config/mise/conf.d/10-bin.local.toml` (gitignored by `*.local.*`) is written by `file-enforcer write`
  with the existing non-empty `package/*/*/node_modules/.bin` directories.
- Accepted caveats:
  PATH order depends on the names sorting `10-` before `20-`;
  `_.path` stays out of `mise.toml`;
  a fresh clone lacks package bins until `file-enforcer write`;
  the file spelling of `MISE_TRUSTED_CONFIG_PATHS` in
  `doc/troubleshooting/mise-untrusted-temporary-worktree-shell.md` changes to the directory spelling;
  `package/monorepo-task/core-exec/src/build-and-test-fixture-config.ts`,
  its test,
  and the `mise.no-env.toml:238` fallback are fixed in the switch.
- `FORBIDDEN_STRINGS_RULES` becomes the comma-separated list of the two appendix files,
  following the rules-file decision.
- `doc/decision/monorepo-manager-all-rust.md` ("root `mise.toml` stays generated") is amended at acceptance.

#### mise.toml: first asking withdrawn until the evidence was complete

Asked whether `.config/mise/conf.d/*.toml` can replace the generated root `mise.toml`;
the user answered:
"Question asked too early.
Just a simple probe?
You sure you have enough evidence that this works?"
A source-level study of mise's `conf.d` loading is running
(how `{{config_root}}` and relative `_.path` entries resolve inside `conf.d` files,
merge order and precedence with `mise.toml` and `*.local.toml`,
trust prompts,
task discovery);
the question returns only with that evidence.

#### License copies: every package, as today

The user answered:
"Our disk space can handle a hundred extra 4kb txt files.
I don't know where this question is coming from."
Every package keeps its `LICENSES/` copies.
The question had no motivation beyond file count.

#### browserslist: the generated file stays

The user answered:
"But rolldown needs to go fast in the common case and we don't have 10ms to wait every build."
The `browserslist-rs` rule kind stays as decided,
because resolving live on every build costs too much time.

#### Schema: golden test and IDE mapping

The user chose "Golden test + IDE mapping".

- `file-enforcer.schema.json` is committed.
- A cargo test in `core-rust` asserts it equals the `schemars` output
  and that the hand-written decoder accepts and rejects the same sample configs.
- `.idea/jsonSchemas.xml` maps the config to it,
  and `"$schema"` stays for VS Code.
- This replaces the self-enforcing schema rule.

#### Cargo.toml keys: written, as decided (not asked)

No native mechanism shares these settings across independent Cargo roots
(nested workspaces are open as rust-lang/cargo#5042),
and `.cargo/config.toml` profiles both override manifest values and are found from the current directory,
with per-package overrides unable to set `panic` or `lto`
(`doc/research/file-enforcer-rust-rewrite-replacements.md`, "Per-crate Cargo.toml keys").
The TOML editor is needed for the catalog anyway,
so the metadata,
lint,
and profile keys stay written.

#### Toolchain lock: plan and lock, mpm-shadow writes the lock

The user chose "Plan + lock;
mpm-shadow writes",
replacing the three-file "Separate resolution file" decision.

- `toolchain.plan.jsonc` (people) and `toolchain.lock.json` (mpm-shadow only).
- `mpm-shadow update` resolves and writes the whole lock;
  an offline `mpm-shadow lock` re-projects plan edits without network.
- file-enforcer `check` verifies that the lock's plan fields equal the plan,
  and never writes the lock.
- Conversion cuts the plan out of today's lock,
  and the lock itself stays.

#### First version of a new npm or PyPI tool: a direct registry lookup

The user chose "Direct lookup for first version".
A planned npm or PyPI tool with no lock entry gets its first version from `registry.npmjs.org` or `pypi.org` once;
later updates use `mpm outdated`.

#### Concurrency: server-client architecture

Asked how concurrent runs avoid clobbering each other
(flock plus a target re-check was recommended);
the user answered "server client architecture".
One server process serializes all writes,
and CLI invocations are clients;
discovery,
startup,
and the relation to `watch` are follow-up questions.
Transport adopted from meow's design
(`doc/planning/monorepo-manager-from-scratch-design.md`, "RPC"):
a filesystem Unix socket under `$XDG_RUNTIME_DIR` with owner-only permissions,
newline-delimited JSON-RPC 2.0,
private plumbing behind the tool's own commands,
and a socket path length check (the design hit "path must be shorter than SUN_LEN").

#### New capabilities: diffs through standard tools, and explain

The user chose `check --diff` and `explain <path>`,
adding:
"We don't have to implement diff ourselves.
We can make our output outputs in a way that can be piped into standard unix tools to render nice diffs."
Follow-up answered:
the user chose "Write expected tree to a dir".
`file-enforcer check --expected-dir <dir>` writes each drifted file's rendered content under `<dir>` at its repo-relative path,
and each drift event names that path,
so `diff -ru`,
`git diff --no-index`,
or any diff viewer renders it;
stdout stays JSON lines.

#### Server follow-ups

- Lifetime:
  the user chose "Auto-start,
  idle exit".
  A client that finds no socket starts the server in the background and waits for its socket;
  the server exits after an idle period with no clients and no watch subscriber.
- Scope:
  the user chose "One per worktree".
  The socket name derives from a hash of the worktree root,
  within the socket path length limit.
- Watch:
  the user chose "Server watches;
  watch subscribes".
  The server watches inputs,
  outputs,
  and glob roots while a watch client is attached;
  `file-enforcer watch` prints the server's events,
  including a warning when a generated file is edited by hand.

#### Adopted from the outside reviews (each has one clear answer)

- Ordering:
  each run plans every rule first against a virtual view of planned outputs,
  so downstream rules read upstream rules' new content,
  and transformations of one file compose in a fixed order;
  cycles and conflicting writers fail.
- Whole-run validation:
  every catalog and manifest problem is collected before any file is replaced,
  so a failing run writes nothing.
- Discovery:
  rules declare selectors (globs,
  fan-out roots),
  and the server rediscovers matches when files appear or disappear.
- File modes are part of a rule's contract,
  checked and repaired independently of content.
- clap's help,
  version,
  and parse errors are emitted as JSON-lines events,
  keeping the JSON-only contract.
- mpm-shadow also owns Rust toolchain version lookup (rustup channel manifests)
  and postinstall `run` commands such as `search-shadow link`.
- `mpm-shadow update --check` keeps the report-only mode.
- The lock's frozen surface (tool ids,
  `version`,
  `artifacts.<platform>.{url,sha256}`,
  `options.androidPackages`) gets consumer tests running the existing `jq` selectors.
- Cutover gates:
  the new `mpm-shadow` and `file-enforcer` are published and fetched by a clean container before the switch;
  CI checks managed files before any step regenerates them;
  the IDE launcher (`package/ide/app/src/language/toolchain.rs:65`),
  its tests and notices,
  and `package/ide/app/bin/inspect-language-toolchain.ts` move to `mpm-shadow bin-paths`;
  the perf fixture retires and the `prefer-readonly-parameter-type` read moves,
  as `doc/decision/monorepo-manager-all-rust.md` already decides;
  a route parity matrix covers both the repository and global locks,
  including `cargo-git` and `go-install`.
- Global plan conversion keeps a 0600 backup beside the original until the converted global sync is verified.
- Bootstrap documentation names the GitHub token as an input for containers and CI.
- Gates before implementation:
  a live forbidden-strings run on a throwaway rules file,
  a `browserslist-rs` run on the real `.browserslistrc` feeding the rolldown consumer,
  and `mpm outdated` probes for rollback,
  unchanged,
  and pinned cases.

#### Second review round: fixes adopted (one clear answer each)

Reviews:
[`doc/research/file-enforcer-rust-rewrite-review2-sol.md`](../research/file-enforcer-rust-rewrite-review2-sol.md),
[`doc/research/file-enforcer-rust-rewrite-review2-astra.md`](../research/file-enforcer-rust-rewrite-review2-astra.md),
[`doc/research/file-enforcer-rust-rewrite-review2-kimi.md`](../research/file-enforcer-rust-rewrite-review2-kimi.md).
None found a contradiction between owner decisions.
Unlike the first round,
these findings were adopted without a repository verification pass,
because nearly all are design-level gaps in the plan's text rather than claims about repository facts.
The advisor was skipped when these were first adopted;
the user asked "Did you skip the advisor?",
and the advisor's corrections are folded in here.

- Filesystem containment:
  mirror roots,
  their ancestors,
  and entries are inspected with `symlink_metadata` and never followed;
  destination symlinks are replaced or removed as links,
  source symlinks are rejected,
  and containment is rechecked at mutation time.
- `check --expected-dir` requires an absent or empty directory disjoint from the worktree and every target,
  writes paired sparse actual and expected trees for the drift set,
  and reports removals and mode changes as events only.
- Server election:
  an owner-only startup lock separate from the socket;
  only its holder probes,
  removes a verified stale socket,
  and spawns;
  others wait with a deadline;
  readiness and draining are explicit;
  requests carry ids,
  and an ambiguous disconnect is never replayed blindly.
- Handshake:
  protocol version,
  build identity,
  and worktree identity (canonical path plus directory identity) are checked before any request;
  a mismatch drains and restarts the server;
  the server exits when its worktree root disappears or changes.
- Runtime directory:
  an unset or unusable `$XDG_RUNTIME_DIR` falls back to a verified owner-only directory;
  the server detaches its standard streams;
  CI and container steps get deterministic shutdown,
  tested in a clean container with the variable unset.
- Each request carries its canonical config path and options;
  the config is re-read every run.
- Snapshot revalidation:
  a plan records the content,
  modes,
  and directory membership it read,
  and is discarded and redone if any changed before publishing;
  the remaining race with non-participating writers is documented,
  not denied.
- Atomicity is per file:
  the no-write guarantee covers validation failures only;
  a partial run is reported,
  the next run reconverges,
  staging files use hidden names (mise skips hidden `conf.d` files) with final modes set before rename,
  and abandoned staging files are cleaned at startup.
- Composition:
  rule kinds declare whether they compose on one file (Cargo key rules with the catalog) or own it whole;
  only undeclared overlaps fail.
- Discovery watches the nearest existing ancestor of an absent fan-out root.
- The catalog library also writes `rust-catalog.toml` (adoption as `>=` floors,
  pruning,
  internal versions),
  planned before manifest rendering.
- License pruning touches only the tool-owned ids (CC-BY-SA-4.0,
  GPL-3.0-or-later,
  LGPL-3.0-or-later),
  leaving hand-kept texts.
- Offline `mpm-shadow lock` rejects additions and resolution-affecting edits (pins,
  package id,
  route,
  artifact selection,
  platforms) without changing the lock;
  one validator serves `lock`,
  `update`,
  `sync`,
  and file-enforcer's lock check.
- `sync` and `status` refuse to run when plan and lock disagree,
  naming `mpm-shadow lock` or `mpm-shadow update`.
- `update --check` never auto-syncs or installs.
- `mpm outdated` reading,
  pending the `mpm outdated` probe gate (not adopted):
  a candidate rule takes a tool's latest version from the outdated row when present,
  otherwise from the installed version mpm reports,
  then compares it with the lock;
  it assumes `mpm installed --output-format json` reports versions for the pnpm and uv managers
  and that an absent row means installed equals registry latest,
  neither verified.
  Settled either way:
  an absent or failed manager result is never read as "unchanged".
- mpm-shadow's own writes:
  the snapshot revalidation rule applies to its lock publish step
  (record the plan and lock revisions read,
  refuse to publish if either changed);
  an exclusive OS lock per scope in front of it is an implementation detail.
  Put to the user as a question,
  answered "?",
  so adopted with the user's veto open.
- `user` commands get their own auto-started server keyed by the canonical per-user config path,
  independent of any worktree,
  following the server-client decision.
  Put to the user as a question,
  answered "?",
  so adopted with the user's veto open.
- Rules list:
  each `FORBIDDEN_STRINGS_RULES` entry is anchored with `{{config_root}}`,
  cli-git array paths are relative to its config file,
  files join with an explicit newline boundary and a per-file source map,
  and duplicate sections across files fail.
- The forbidden-strings and git-policy changes join the pre-switch build list,
  and the switch lock must pin releases that understand the new rules list.
- The IDE launcher finds `mpm-shadow` by an explicit path (never the `mpm` link,
  whose name selects pass-through mode),
  tested in the real sandbox with the toolchain present and missing.
- The bootstrap runbook installs and verifies rustup before `cargo install`.

#### Rules list: a trailing '?' marks the optional file

Asked how the comma-separated list marks the file that may be missing;
the user chose "Trailing '?' marks optional"
over treating `.local.` names as optional (recommended after the advisor's note).
An entry ending in `?` may be absent,
every other entry must exist,
and any read error other than absence fails;
the environment variable uses the same syntax,
and cli-git's JSONC array uses `{"path": ..., "optional": true}`.

#### mpm files and file-enforcer's toolchain role: prototype first

Re-asked who writes and checks `mpm.packages.toml` and `mpm.managers.toml`,
with the consequence that file-enforcer could end up with no toolchain role;
the user answered:
"This is the 3rd time.
Please make sure we're actually right this time,
okay?
And you have worktrees.
You have podman.
You can prototype."
A throwaway prototype in a linked worktree and a podman container runs the real mpm and the candidate flows
before the question returns.

Prototype:
[`doc/research/file-enforcer-rust-rewrite-mpm-files-prototype.md`](../research/file-enforcer-rust-rewrite-mpm-files-prototype.md)
(branch `prototype/fe-mpm-files`,
head `fb874028f9`).
Asked again with its evidence,
the user chose "Drop both;
mpm-shadow checks".

- `mpm.packages.toml` and `mpm.managers.toml` are deleted in both scopes:
  mpm 8.2.0 ignored their versions for cargo and pnpm,
  pinned rustup only because the package name is the channel,
  and matched the direct `cargo binstall --version` byte for byte;
  the only code reader was the rustup call that already falls back to `rustup toolchain install`.
- mpm-shadow installs rustup channels and cargo-binstall tools directly from the lock,
  and `mpm-shadow lock --check` is the toolchain drift gate in CI beside `file-enforcer check`.
- file-enforcer has no toolchain role;
  the earlier "lock's plan fields equal the plan" check moves to `mpm-shadow lock --check`.
- `doc/decision/mpm-toolchain-adoption.md` (the restore manifest generated from the plan,
  `:92-112`,
  `:201-223`) is amended at acceptance.
- Follow-up recorded,
  not solved:
  a consistent hand edit of a resolved lock version passes every gate,
  so the lock needs its own integrity check (artifact URL against version,
  or a digest) or a scheduled `update --check`.
- Note:
  the prototype branch was auto-pushed to `origin`,
  as `APG` in `AGENTS.md` says auto-push does;
  it stays as the primary source.

#### Process lessons: deferred to issue #1817

Draft `AGENTS.md` rules for this round's slips
(closed-source evidence,
the evidence bar for tool-behavior options,
question motivation) were put to the user,
who answered:
"Open gh issue and I'll look at this in another session because I have a bad feeling about these rules as is."
Issue #1817 holds the slips,
the corrections,
and the unadopted drafts.

### Adopted defaults put to the user for confirmation

Listed 2026-10-29 in the confirmation question;
the user asked for an outside review before confirming.

- CLI:
  `file-enforcer write | check | watch`,
  plus `user write | user check` for the per-user config;
  global `--config` defaulting to `file-enforcer.config.jsonc` at the repo root;
  unknown flags rejected.
- Exit codes:
  0 clean,
  1 drift or rule failure,
  2 bad command line or config (`package/mpm-shadow/cli/src/constants.rs:31-46` precedent).
- Writes:
  temp file then rename,
  only when content differs;
  no lock and no cache.
- `watch`:
  `notify` with a debouncer (both in `rust-catalog.toml`),
  rerunning the rules whose declared inputs changed and reloading the config when it changes.
- Distribution:
  published to crates.io and installed by `mpm-shadow sync`,
  like `monochromatic-lint` and `mpm-shadow`.
- New dependencies each get a choosing-technology vet first:
  `toml_edit`,
  `browserslist-rs`,
  `schemars`,
  and an XML editor for the JetBrains rule.
- At acceptance,
  amend `doc/decision/monorepo-manager-all-rust.md` and `doc/decision/mpm-toolchain-adoption.md`.

### Adopted from the request and repository rules

- Rust is approved for this tool by the request itself;
  no existing approval scope (meow, cli-git, the linter) covers a standalone file-enforcer.
- The CLI uses clap,
  as requested;
  repository precedent is clap derive in `package/forbidden-strings/cli` and others.
- Every Rust dependency comes from `rust-catalog.toml` through `workspace = true`
  (`doc/decision/rust-dependency-catalog.md`).
- Rust tests run through nextest (`RNQ` in `AGENTS.md`).
- Every line the tool writes is one JSON object (`schemaVersion`, `sequence`, `type`, `code`, `message`, `hint`),
  matching meow's accepted "Emit every line as a json, error or warning or not." (user, 2026-09-17)
  and the live cli-git event stream (`package/git-policy/cli/src/native/diagnostics.rs:223`).
  Adopted because the library is meant for meow to embed,
  then confirmed 2026-10-29 when the user chose "JSON lines always" over a TTY-dependent text renderer
  and a text default with a `--json` flag.
- Dropped outright under "No need to preserve anything",
  because nothing outside the package uses them:
  the OS package installer (`ensurePackage`),
  its 22,607-line Repology-derived index,
  and the PostgreSQL `generator/` pipeline;
  the staleness manifest,
  its cross-process lock,
  and orphan-temp sweeping
  (`doc/troubleshooting/file-enforcer-staleness.md` shows the cache never recorded for the root config
  and could keep stale output with exit 0);
  the `CONTEXT.md` guard,
  which cli-git's `mono/forbidden-root-context` policy already enforces.
- No incremental cache ships without a measured need.
- `check` renders every output in memory and compares,
  needing no prior write,
  so CI's `git diff --exit-code` workaround in `.github/workflows/advisory-core.yml:92-105` can go.
- One entry point:
  the clap binary,
  with unknown flags rejected
  (the old `node file-enforcer.config.ts` path skipped `cli.ts` and silently dropped unknown flags).
- Output need not be byte-identical to the TypeScript tool's;
  the first Rust run may reformat managed files once.
- `exec` leaves with the rewrite:
  `package/vm-builder/cli` gets its private runner,
  as already designed in `doc/decision/monorepo-manager-all-rust.md` ("Consequences").
- The root `mise.toml` stays generated from `mise.no-env.toml` until Mise is removed,
  as `doc/decision/monorepo-manager-all-rust.md` already decides.
- Long fixed text (the `CLAUDE.md` preamble,
  the `mise.toml` `[env]` block) lives in content files the config names,
  never inline in the config.
- The TypeScript file-enforcer keeps running until the Rust tool replaces it;
  concurrent edits in `package/file-enforcer/core` belong to other sessions.

## Open questions

Tracked in the grilling rounds;
each answered question moves into a decisions section with the user's words and date.
