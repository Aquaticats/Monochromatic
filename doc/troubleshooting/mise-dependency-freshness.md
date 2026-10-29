# mise 2026.9.12 runs tasks on stale `node_modules` without `[deps]`; task `sources` miss installs

## Status

Investigated 2026-09-25 for
[issue #570](https://github.com/Aquaticat/Monochromatic/issues/570)
against installed mise `2026.9.12 linux-x64 (2026-09-20)`,
source tag `v2026.9.12` (commit `1698dd8`).
Every source citation in this document is at that tag.
Upstream `main` at `1249434` (2026-09-26) differs from the tag only in
`src/cli/run.rs`,
`src/task/task_executor.rs`,
and one line each in `src/deps/engine.rs` and `src/deps/mod.rs`
among the files cited here;
the excerpts from those files quoted here were read at `main` first and are unchanged there.

No repository file was changed by this investigation;
the remediation choice belongs to the #570 grilling session
(`doc/handover/config-oxlint-stale-plugin-bundle-issue-570.md`).

## Symptom

A merge changes workspace `package.json` files
(and `pnpm-lock.yaml`,
or `pnpm install --lockfile-only` rewrites it),
but nobody runs `pnpm install`.
The next `mise run` build or lint task starts immediately against the old `node_modules`.
mise prints nothing about dependency state.
The bundler turns the not-yet-linked workspace import into an external,
exits `0`,
and the broken bundle persists.
A later `pnpm install` does not repair it,
because no task freshness input changed.

This document answers which of those links belong to mise:
what mise offers for dependency freshness,
how task `sources` and `outputs` decide freshness,
and what in this repository's mise usage contributed.

## Root cause

### mise has a dependency-freshness feature, and this repository does not enable it

mise ships `mise deps`,
formerly `mise prepare`,
which keeps the old name as an alias:

```rust
// src/cli/deps/mod.rs:19
    alias = "prepare",
```

It is experimental
(`docs/dev-tools/deps.md:5`:
`# Deps <Badge type="warning" text="experimental" />`),
and every engine constructor checks the setting,
for example:

```rust
// src/deps/engine.rs:216
            Settings::get().ensure_experimental("deps")?;
```

This repository already sets `experimental = true`
(`mise.no-env.toml:225`,
generated into `mise.toml:226`),
so the feature is usable without further settings.
No repository config declares a `[deps]` table
(`rg '^\[deps' mise.no-env.toml mise.toml` finds nothing).

The built-in pnpm provider tracks the config root's lockfile and manifest,
and requires `node_modules` to exist:

```rust
// src/deps/providers/pnpm.rs:29-43
    fn sources(&self) -> Vec<PathBuf> {
        let root = self.base.config_root();
        self.base
            .sources(vec![root.join("pnpm-lock.yaml"), root.join("package.json")])
    }

    fn outputs(&self) -> Vec<PathBuf> {
        self.base
            .outputs(vec![self.base.config_root().join("node_modules")])
    }

    fn install_command(&self) -> Result<DepsCommand> {
        self.base
            .install_command("pnpm", &["install"], "pnpm install")
    }
```

With `auto = true`,
`mise run` checks the providers after resolving tasks and before running any task body:

```rust
// src/cli/run.rs:823-830
        if let Some(engine) = deps_engine {
            let (env, env_remove) = ts.env_with_path_and_removals(&config).await?;
            let result = otel::TaskRunTelemetry::phase(
                telemetry.as_ref(),
                "deps",
                engine.run(DepsOptions {
                    auto_only: true, // Only run providers with auto=true
```

`--no-deps` skips it (`src/cli/run.rs:249`,
`pub no_deps: bool`).
A failing provider command returns its error from `run_parallel`
(`src/deps/engine.rs:814-851`),
so `mise run` stops before the task body.

#### How the deps engine decides staleness

Freshness is content-hash based,
not mtime based.
`check_freshness` (`src/deps/engine.rs:1071`) walks these steps in order:
required outputs must exist;
outputs recorded after the last successful run must still exist;
a changed provider command is stale;
then blake3 hashes of the source files are compared with the stored ones:

```rust
// src/deps/engine.rs:1177-1184
        match st.get_hashes(provider_id) {
            Some(stored_hashes) => {
                // Check for changed files
                for (path, hash) in &current_hashes {
                    match stored_hashes.get(path.as_str()) {
                        Some(stored_hash) if stored_hash == hash => {}
                        Some(_) => {
                            return Ok(FreshnessResult::Stale(format!("{path} changed")));
                        }
```

Hashes are stored only after the provider command succeeds,
and they are recomputed after the install,
so a failed install is retried next time:

```rust
// src/deps/engine.rs:777-796
            // Save content hashes and existing outputs for each successfully
            // ran provider. [...]
            for step in &results {
                if let DepsStepResult::Ran(id) = step
                    && let Some(provider) = providers.iter().find(|p| p.id() == id)
                {
                    [...]
                    if let Ok(hashes) = state::hash_sources(&sources, project_root) {
                        [...]
                        st.set_hashes(provider_id, hashes);
```

State lives outside the project,
keyed by the project root path,
so each worktree has its own state:

```rust
// src/deps/state.rs:175-179
fn state_path(project_root: &Path) -> PathBuf {
    dirs::STATE
        .join("deps")
        .join(format!("{}.toml", hash_to_str(&project_root)))
}
```

#### Limits of the deps engine that matter here

- Default pnpm sources are only the root `pnpm-lock.yaml` and root `package.json`.
  A workspace-member `package.json` edit with no lockfile change stays "fresh"
  until `sources` is overridden;
  overriding replaces the defaults
  (`docs/dev-tools/deps.md:198-202`).
- The only installed-tree signal is that `node_modules` exists.
  mise states this directly:
  "Freshness checks do not inspect every installed package"
  (`docs/dev-tools/deps.md:268-271`).
  Deleting one link below `node_modules`,
  which is how the #570 reproduction sketch simulates the incident,
  is invisible to it.
- Only runs through mise record state.
  A manual `pnpm install` after a lockfile change leaves the stored hash old,
  so the next `mise run` runs `pnpm install` once more.
  That extra run is redundant,
  not incorrect.
- There is no cross-process lock.
  The only shared marker is an in-process set
  (`src/deps/mod.rs:323`,
  `static STALE_OUTPUTS: LazyLock<Mutex<HashSet<PathBuf>>>`).
  Two concurrent top-level `mise run` invocations that both see stale state
  can both start `pnpm install`.
  Inside one invocation the check happens before any task body,
  so child `mise run` processes spawned by a fanout task see the saved state
  and skip the install.

### Task `sources` and `outputs` share the `ensureOxlintConfig` flaw unless an install marker is a source

In the default metadata mode,
`sources_are_fresh` (`src/task/task_source_checker.rs:809`) first compares a
hash of each source's path,
size,
and mtime with the baseline stored after the last success;
a mismatch means stale:

```rust
// src/task/task_source_checker.rs:861-876
        let existing_hash = source_existing_hash(task, &root, use_content_hash);
        if existing_hash.as_deref().is_some_and(|h| h != source_hash) {
            [...]
            // Do not write the hash here — the task is about to run. If it
            // fails, the baseline must stay at the previous value so the next
            // invocation still detects the mismatch. save_checksum writes the
            // hash after a successful run.
            return Ok(false);
        }
```

When the baseline matches or does not exist,
it falls back to mtimes,
comparing the newest source against the newest output:

```rust
// src/task/task_source_checker.rs:907-918
        let sources = get_last_modified_from_metadatas(&source_metadatas);
        let outputs = get_last_modified(&root, &task.outputs.paths(task, &root))?;
        trace!("sources: {sources:?}, outputs: {outputs:?}");
        let fresh = match (sources, outputs) {
            (Some(sources), Some(outputs)) => {
                if equal_mtime_is_fresh {
                    sources <= outputs
                } else {
                    sources < outputs
                }
            }
```

```rust
// src/task/task_source_checker.rs:1464
    let last_mod = file_modified.into_iter().chain(directory_modified).max();
```

With `task.source_freshness_hash_contents = true`
(`MISE_TASK_SOURCE_FRESHNESS_HASH_CONTENTS`,
default `false`,
`settings.toml:3642-3646`),
blake3 content hashes replace the metadata hash,
the mtime comparison is skipped,
and a stored output hash must also match
(`src/task/task_source_checker.rs:878-904`).

Either mode answers "did a declared input change since the last successful run".
A build that ran against a stale `node_modules` and exited `0` is a successful run.
If `node_modules` is not a declared source,
the later `pnpm install` changes nothing mise looks at,
so the stale output stays fresh.
That is the same hole as `ensureOxlintConfig`
(`mise.toml:399-421`),
which also has no installed-dependency input.
The difference is in who owns it:
mise evaluates only what the task declares,
so the fix is declaring an install marker as a source.
pnpm rewrites `node_modules/.pnpm/lock.yaml` on each install
(`node_modules/.modules.yaml` in a workspace with no external packages),
and a metadata-mode source on that file makes the next check stale.

Dependencies do not carry this across processes.
`did_work` from a dependency skips the downstream freshness check
(`src/task/task_executor.rs:518-528`,
`&& !dependency_state.any_did_work`),
but only within one `mise run` invocation,
and only for dependencies that declare `sources`:

```rust
// src/cli/run.rs:1120-1122
                if outcome.did_work && !task.sources.is_empty() {
                    deps.mark_did_work(&task);
                }
```

An install run in an earlier invocation
(the #570 sequence:
`mise run prepare:pnpm:install` then a later build)
does not invalidate anything.

#### Success and failure tracking

`save_checksum` runs only on the success path
(`src/task/task_executor.rs:769`),
and its comment states the intent:

```rust
// src/task/task_source_checker.rs:982-985
    // Persist the source hash now that the task has succeeded. Doing this here
    // rather than in sources_are_fresh ensures a failed run never advances the
    // baseline — the next invocation will detect a mismatch and re-run.
    if let Some((hash, path)) = compute_source_hash(task, config).await? {
```

That holds when a baseline exists.
It does not hold for a task whose first run fails after writing its outputs:
there is no baseline,
the mtime fallback finds the outputs newer than the sources,
reports fresh,
and writes the baseline itself
(`src/task/task_source_checker.rs:919-925`).
Upstream PR
[jdx/mise#10953](https://github.com/jdx/mise/pull/10953)
(merged 2026-07-12)
fixed the same class of problem for `outputs = { auto = true }` only,
and says it "intentionally does not change behavior for explicit,
user-managed output files".
Content-hash mode is not affected,
because it treats a missing baseline as stale
(`src/task/task_source_checker.rs:885-888`).

For #570 this gap is not the cause:
the bundler exited `0`,
so mise correctly recorded a success.

#### Documentation does not match the mtime comparison

`docs/tasks/task-configuration.md:503-505` says mise
"skips the task when the modification time of the oldest output file is newer than
the modification time of the newest source file".
The code compares against the newest output
(`src/task/task_source_checker.rs:1464`,
`.max()`).
The experiment under "Verification" backdates one of two outputs to 2000
and mise still skips the task.
`ensureOxlintConfig` uses the oldest output,
which is the stricter reading.

### Task templates, `depends`, and hooks

Templates can carry `depends`,
`depends_post`,
and `wait_for`;
a task that sets its own value replaces the template value,
and an empty local list does not clear it
(`docs/tasks/templates.md:112`,
`docs/tasks/templates.md:124-127`).
So a template such as `[task_templates."build:js:node"]` could declare
`depends = ["//:prepare:pnpm:install"]`
and every package task extending it would inherit that.

Three things make that a poor fit here.
First,
`mise.no-env.toml` states a local policy against it
(`mise.toml:627`:
"never use depends or post depends,
use run only because it's the easiest to understand.").
Second,
this repository fans out through the `fanout` and `fanout_packages` Node snippets
(`mise.toml:293-349`),
which spawn a separate `mise run <child>` process per child task.
Each child resolves its own dependency graph,
so an install dependency would be evaluated once per leaf task,
possibly concurrently,
and mise does not deduplicate across processes.
Without `sources` and `outputs` on the install task,
each leaf would run `pnpm install`.
Third,
a dependency's `did_work` does not survive the process boundary,
so it cannot force downstream rebuilds after an install in another process.

`[deps]` with `auto = true` avoids all three:
the top-level `mise run` checks before spawning any child,
and each child's own check then finds the state fresh.

Hooks do not help for this incident.
`docs/hooks.md:7-8`:
"Except for the `preinstall` and `postinstall` hooks,
these require the `mise activate` shell hook to be installed in your shell."
Agents,
CI,
and scripted `mise run` calls do not go through `mise activate`,
so `enter`,
`cd`,
and `watch_files` never fire there.
`preinstall` and `postinstall` fire around tool installs by `mise install`
(`docs/hooks.md:15`),
not around `pnpm install`.
The commented-out hook
(`mise.toml:280`,
`# enter = { task = "bootstrap" }`)
would only run `mise install` and `mise upgrade`
(`[tasks.bootstrap]`,
`mise.toml:981-983`);
re-enabling it would not install workspace packages.

### Repository usage that contributed

- Nothing in the task graph ties builds or lint to install state:
  `prepare:pnpm:install` (`mise.toml:1015-1017`) is a plain `pnpm install`,
  and build templates (`mise.toml:513-545`) have no dependency or freshness input.
- `ensureOxlintConfig` reimplements freshness by hand with mtimes,
  omits installed dependencies,
  and omits transitively bundled workspace sources
  (see the #570 handover).
- The fanout design moves orchestration into Node subprocesses,
  which removes mise's per-invocation deduplication and `did_work` propagation.

### Full `mise run build` exiting 1 in a fresh worktree

A subagent reported that a full `mise run build` in a fresh worktree
exited 1 with no error line.
That was not reproduced.
In a throwaway worktree at `f34246851`
(outside `~/temp/agent`,
after `pnpm install --frozen-lockfile`),
`mise run build` exited 1 after 3874 log lines and ended with:

```text
Error: package fanout failed: //package/music-player/android-app:build, //package/ssg/aquati.cat:build, //package/doodle-widget/app:build, //package/terminal/app:build
    at [eval]:24:32
[//:build] ERROR task failed
```

Each named package printed its own error earlier in the log:
`No NDK under .../android-sdk/23.0/ndk; run mise run prepare:android`,
a failing `git log --follow` in the `aquati.cat` site build,
`[RESOLVE_ERROR] Could not resolve 'canvg'` from `jspdf` in `doodle-widget`,
and `missing release libghostty-vt build output; run build first` in `terminal/app`.
These are environment and build-order failures in those packages,
not mise behavior.
Correction (2026-10-20,
commit `1eee51ee2f`):
the "environment and build-order" diagnosis is wrong for the
`terminal/app` staging error,
and the staging error's own text names the same wrong assumption.
The build ran and succeeded;
`stage:runtime-libs` failed because it globbed only
`target/<profile>/build/libghostty-vt-sys-<hash>/out`.
This machine's `~/.cargo/config.toml` sets
`[build] build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"`
(relocated to a second disk on 2026-10-14),
and `cargo metadata` reports `build_directory`
`/var/mnt/encrypted/cargo-build/f8/4f84da14ad940c`
beside `target_directory` `/var/home/user/Monochromatic/package/terminal/app/target`,
so build-script output lands in
`<build_directory>/<profile>/build/libghostty-vt-sys/<hash>/out`
while final artifacts stay in `target/`.
Commit `1eee51ee2f` made `package/terminal/app/mise.toml` discover roots
from `cargo metadata` (`target_directory` and `build_directory`),
`CARGO_TARGET_DIR`,
and the package target dir,
and glob both directory layouts in every root.
This correction covers only the `terminal/app` staging error;
the other three diagnoses are unchanged,
and "not mise behavior" still holds for all four.
The `fanout_packages` snippet does name the failures;
the per-package error lines are interleaved far above the final line,
so a capped or filtered view of the output can show the exit code without them.
The same run printed `UNRESOLVED_IMPORT` warnings for `@monochromatic-dev/logger-core`
from two raw `src/main.ts` configs,
which the #570 handover already records.

### Durable lesson: consult `cargo metadata` `build_directory`

Any task staging cargo build-script output must consult the `cargo metadata` `build_directory`,
not just `target/`.
`cargo metadata` reports `target_directory` and `build_directory` separately,
and a machine-level `[build] build-dir` can move build-script output away from `target/`
while final artifacts stay in `target/`.
Build-script output directories have two layouts:
`<root>/<profile>/build/libghostty-vt-sys-<hash>/out` in the classic `target/` shape and
`<root>/<profile>/build/libghostty-vt-sys/<hash>/out` under a relocated build dir.
Discover every root from `cargo metadata`,
`CARGO_TARGET_DIR`,
and the package target dir,
and glob both layouts in each root
(the discovery `package/terminal/app/mise.toml` implements in commit `1eee51ee2f`).

## Verification

### Fixture

A throwaway pnpm workspace at `${HOME}/temp/agent-570-mise/ws`,
outside `~/temp/agent` so the stray `module-logger` symlink recorded in the #570
handover cannot affect resolution,
with isolated mise state and trust:

```sh
# every mise call in this section runs with these variables
export MISE_STATE_DIR="${HOME}/temp/agent-570-mise/state"
export MISE_TRUSTED_CONFIG_PATHS="${HOME}/temp/agent-570-mise/ws"
```

```json
// package.json
{ "name": "ws-root", "private": true }
```

```yaml
# pnpm-workspace.yaml
packages:
  - packages/*
```

`packages/b` is `@x/b` exporting one constant.
`packages/a/build.mjs` stands in for rolldown:
it resolves `@x/b` from `packages/a`,
writes `inlined @x/b` or `EXTERNAL @x/b (broken)` to the output path,
and exits `0` either way:

```javascript
// packages/a/build.mjs
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname } from 'node:path'
const out = process.argv[2]
let state
try { import.meta.resolve('@x/b'); state = 'inlined @x/b' } catch { state = 'EXTERNAL @x/b (broken)' }
mkdirSync(dirname(out), { recursive: true })
writeFileSync(out, state + '\n')
console.log(`build wrote ${out}: ${state}`)
```

```toml
# mise.toml
[settings]
experimental = true

[deps.pnpm]
auto = true

[tasks.build-plain]
run = "node packages/a/build.mjs packages/a/dist/plain.txt"
sources = ["packages/a/build.mjs", "packages/a/package.json", "pnpm-lock.yaml"]
outputs = ["packages/a/dist/plain.txt"]

[tasks.build-installaware]
run = "node packages/a/build.mjs packages/a/dist/aware.txt"
sources = ["packages/a/build.mjs", "packages/a/package.json", "pnpm-lock.yaml", "node_modules/.modules.yaml"]
outputs = ["packages/a/dist/aware.txt"]
```

The "merge" step writes
`"dependencies": { "@x/b": "workspace:*" }`
into `packages/a/package.json`
and runs `pnpm install --lockfile-only` (pnpm 12.5.1).

### Results that behave as intended

- First `mise run build-plain` with no deps state:
  `[deps.pnpm]` ran `pnpm install` before the task.
- Positive control for `[deps]`:
  after the merge step,
  `mise deps install pnpm --explain` printed
  `Status: stale (pnpm-lock.yaml changed)`
  and exited non-zero.
  `mise run build-plain` then ran `pnpm install`,
  `packages/a/node_modules/@x/b` appeared,
  and the build wrote `inlined @x/b`.
- Negative control:
  the next `mise run build-plain` printed no `[deps.pnpm]` lines
  and `sources up-to-date, skipping`.
- After a stale build and a manual `pnpm install`,
  `mise run --no-deps build-installaware` reran and wrote `inlined @x/b`,
  because `node_modules/.modules.yaml` changed.
- Overriding
  `sources = ["pnpm-lock.yaml", "pnpm-workspace.yaml", "package.json", "packages/*/package.json"]`
  on `[deps.pnpm]` made a member-manifest change report stale.
- A task whose earlier run succeeded,
  then whose source changed and whose run failed after writing its output,
  reran on the next invocation (baseline mismatch).
- In content-hash mode (`MISE_TASK_SOURCE_FRESHNESS_HASH_CONTENTS=1`),
  a task whose first run failed after writing its output reran.

### Results that reproduce the gaps

- The #570 sequence with `--no-deps`
  (merge step,
  `mise run --no-deps build-plain`,
  then manual `pnpm install`,
  then `mise run --no-deps build-plain`):
  the first build wrote `EXTERNAL @x/b (broken)`,
  and after the install mise printed `[build-plain] sources up-to-date, skipping`.
  `packages/a/dist/plain.txt` kept `EXTERNAL @x/b (broken)`.
- After that manual install,
  `mise deps install pnpm --explain` still printed
  `Status: stale (pnpm-lock.yaml changed)`
  (redundant install on the next `mise run`).
- With default provider sources,
  editing only `packages/a/package.json` left
  `Status: fresh (outputs are up to date)`.
- First-run failure in the default mode:

  ```toml
  # ${HOME}/temp/agent-570-mise/failfix/mise.toml
  [tasks.fail]
  run = "echo x > fail.out; test -z \"$FAIL\""
  sources = ["src.txt"]
  outputs = ["fail.out"]
  ```

  ```sh
  # fresh state dir, installed mise 2026.9.12
  env FAIL=1 MISE_STATE_DIR="${PWD}/.state" MISE_TRUSTED_CONFIG_PATHS="${PWD}" mise run fail
  # [fail] ERROR task failed
  env MISE_STATE_DIR="${PWD}/.state" MISE_TRUSTED_CONFIG_PATHS="${PWD}" mise run fail
  # [fail] sources up-to-date, skipping
  ```

- Oldest versus newest output:
  a task with `outputs = ["o1", "o2"]` ran once,
  then `touch --date=2000-01-01 o1`;
  the next run printed `sources up-to-date, skipping`
  although `o1` (2000-01-01) is older than the source.

## Verified workarounds

### Enable the pnpm deps provider

```toml
# mise.no-env.toml (then regenerate mise.toml through file-enforcer)
[deps.pnpm]
auto = true
sources = ["pnpm-lock.yaml", "pnpm-workspace.yaml", "package.json", "package/*/*/package.json"]
```

Tradeoffs:
the feature is experimental and its config surface has changed within 2026
(renamed from `prepare`,
state moved to `$MISE_STATE_DIR`,
see "Upstream context");
every `mise run` and `mise x` hashes the listed files,
including each fanout child process;
a manual `pnpm install` causes one redundant install;
partial damage inside `node_modules` is not detected;
`--no-deps` bypasses it.
Measured in a throwaway worktree of this repository with this block added
and a `run = "true"` task,
state fresh,
eight runs per series:
`--no-deps` took 539 to 648 ms per `mise run`;
two series with the check took 582 to 1669 ms and 586 to 1064 ms.
The spread with the check is wider than the difference between medians,
so these runs do not pin down a per-invocation cost;
they bound it below about one second on this machine,
multiplied by every child `mise run` a fanout spawns.
`pnpm install` also runs automatically before any task,
including tasks unrelated to Node,
whenever the lockfile moved.

### Declare an install marker as a task source

For tasks that must rebuild after an install,
add `node_modules/.pnpm/lock.yaml`
(repository root,
present because the workspace has external packages)
to their `sources`.
Tradeoffs:
it catches only installs that rewrite that file;
in content-hash mode it triggers only when the file content changes,
which is the lockfile-derived content,
not the mtime;
templates replace rather than merge `sources`,
so it has to be declared in each template or task that sets its own list.
It does not prevent the first stale build;
it only guarantees the rebuild after the install.

### Use content-hash freshness

`task.source_freshness_hash_contents = true` closes the first-run-failure gap.
Tradeoff:
every check hashes every source file's content
(a per-file size and mtime cache limits rehashing;
`src/task/task_source_checker.rs:1303-1325`).

## What does not work

- Re-enabling `enter = { task = "bootstrap" }`:
  requires `mise activate`,
  and `bootstrap` installs tools,
  not workspace packages.
- `preinstall` and `postinstall` hooks:
  tied to `mise install` of tools.
- Template `depends` on `prepare:pnpm:install`:
  evaluated separately in every fanout child process,
  with no cross-process deduplication or lock,
  and against the local no-`depends` policy.
- Relying on task failure tracking:
  the bundler exits `0`,
  so the stale build is a success to mise.
- pnpm `verifyDepsBeforeRun`:
  fires only for `pnpm run` and `pnpm exec`,
  which repository tasks do not use
  (from the #570 handover;
  not re-verified here).

## Upstream context

- [jdx/mise#8002](https://github.com/jdx/mise/discussions/8002):
  `mise prepare` improvement proposals;
  jdx acknowledged that prepare used mtime-only staleness,
  since replaced by blake3 hashing.
- [jdx/mise#9706](https://github.com/jdx/mise/discussions/9706):
  deps state moved to `$MISE_STATE_DIR`;
  jdx frames `mise deps` as dependency installation into untracked directories.
- [jdx/mise#10933](https://github.com/jdx/mise/discussions/10933):
  monorepo support,
  added as `mise deps --monorepo` in 2026.7.7.
- [jdx/mise#8733](https://github.com/jdx/mise/discussions/8733):
  "Possible to not skip task without output if previous run had non-zero exit?";
  [jdx/mise#10953](https://github.com/jdx/mise/pull/10953) fixed only automatic outputs.
- [jdx/mise#7656](https://github.com/jdx/mise/discussions/7656):
  a missing output among several now makes the task stale (fixed by PR #11786);
  it does not cover an old but present output.

Searches run:
`gh search issues --repo jdx/mise` for `deps auto stale`,
`prepare pnpm install`,
and `sources outputs`;
GitHub discussion search for `mise prepare`,
`deps provider`,
`sources failed task`,
`task failed up-to-date skipping`,
`failed task skipped next run`,
`task fails outputs sources up-to-date`,
and `oldest output mtime sources`;
`gh search prs` for `failed baseline sources` and `save_checksum failed`.
mise directs bug reports to Discussions
(`.github/ISSUE_TEMPLATE/config.yml`,
`blank_issues_enabled: false`).

## Upstream filing decision

`.out-of-scope/` was checked;
none of its entries
(`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`)
covers mise.

The #570 incident itself is not upstream's fault:
mise offers `[deps]` and this repository does not configure it,
and the stale build exited `0`.
Nothing is filed for it.
Two side findings were audited separately.

### Finding A: a failed first run with explicit outputs counts as fresh

1. **Upstream's fault?**
   Yes.
   `save_checksum`'s comment promises that "a failed run never advances the baseline"
   (`src/task/task_source_checker.rs:982-984`),
   but the mtime fallback writes the baseline on the next check
   when a failed run left outputs newer than sources.
2. **Can upstream fix it?**
   Yes;
   the prototype is below.
3. **Supported use case?**
   Yes:
   explicit `sources` and `outputs` are the documented freshness mechanism
   (`docs/tasks/task-configuration.md:499-525`),
   and `e2e/tasks/test_task_source_hash_not_written_on_failure` tests the neighboring case.
4. **Contribution welcome?**
   Yes with conditions.
   `docs/contributing.md:10-18` welcomes AI-assisted responses that the poster reviewed and verified,
   and bans drive-by AI answers in threads the poster has no connection to.
   This finding comes from our own use,
   so it is connected;
   the comment must disclose assistance.
   `docs/contributing.md:21-27` asks for a discussion before a non-obvious PR,
   which the comment serves.
5. **Likely fixed?**
   Soft yes.
   PR #10953 fixed the auto-output case and stated it
   "intentionally does not change behavior for explicit,
   user-managed output files";
   that limits that PR's scope,
   and no maintainer declined the explicit-output case.
   The prototype does not change or delete user-managed files,
   which respects that boundary.
6. **Prototyped?**
   Yes:
   [`mise-dependency-freshness.patch`](mise-dependency-freshness.patch),
   against tag `v2026.9.12`.
   It writes a `<task_state_key>-pending` marker in `$MISE_STATE_DIR/task-sources/`
   immediately before execution
   (next to the existing `remove_auto_output` calls in `src/task/task_executor.rs`),
   makes `sources_are_fresh` return stale while the marker exists,
   and removes it in `save_checksum` after success.
   The baseline file is untouched,
   so `task_source_files(only_changed=true)` keeps measuring from the last success.
   It adds `e2e/tasks/test_task_source_freshness_failed_first_run`.

Verification of the prototype:
`cargo build --bin mise` in `docker.io/library/rust:latest`
(with `cmake` installed;
2 GiB memory limit for the test runs,
`--network=none`),
then `e2e/run_test` for the new test and nine existing freshness tests.

```text
# patched 2026.9.12-DEBUG
PASS tasks/test_task_source_freshness_failed_first_run
PASS tasks/test_task_failed_freshness
PASS tasks/test_task_source_hash_not_written_on_failure
PASS tasks/test_task_source_freshness
PASS tasks/test_task_source_freshness_multi_output
PASS tasks/test_task_source_files_only_changed
PASS tasks/test_task_dep_invalidates_sources
PASS tasks/test_task_run_sources
PASS tasks/test_task_source_freshness_hash_contents_no_baseline
PASS tasks/test_task_source_comparison_no_exec

# released 2026.9.12 binary, same tests (Fedora 44 container)
FAIL tasks/test_task_source_freshness_failed_first_run
ERROR: E2E assertion failed: [mise run -q build] expected 'built' but got ''
(the other nine PASS)
```

The fixture `failfix/mise.toml` from "Verification" also ran against the patched binary:
fail,
rerun (succeeds),
skip;
after a source edit,
fail,
rerun,
skip.

Duplicate search found [jdx/mise#8733](https://github.com/jdx/mise/discussions/8733),
whose only reply links PR #10953 for the auto-output case.
The explicit-output case,
the reproduction,
and the prototype are absent from that thread,
so the artifact is an additive comment,
not a new discussion.
All six constraints hold;
the draft is fileable once the user approves posting it.

~~~md
PR #10953 fixed this for `outputs = { auto = true }`. The same thing still happens with explicit outputs in the default (mtime) freshness mode when the task's **first** run fails after writing its output (2026.9.12, also current `main`):

```toml
[tasks.build]
run = 'touch out.txt; if [ -f poison ]; then exit 1; fi; echo built'
sources = ['src.txt']
outputs = ['out.txt']
```

```sh
echo hello > src.txt; touch poison
mise run build        # ERROR task failed
rm poison
mise run build        # "sources up-to-date, skipping"; expected "built"
```

There is no stored baseline yet, so `sources_are_fresh` falls through to the mtime comparison (`src/task/task_source_checker.rs` around line 907), finds `out.txt` newer than `src.txt`, reports fresh, and writes the baseline itself. That contradicts the comment in `save_checksum` ("a failed run never advances the baseline"). Content-hash mode is not affected because it treats a missing baseline as stale.

A prototype that leaves user-managed outputs alone: write a `<task_state_key>-pending` marker under `task-sources/` right before execution (next to `remove_auto_output`), return stale from `sources_are_fresh` while it exists, and remove it in `save_checksum`. The baseline file is not touched, so `task_source_files(only_changed=true)` still measures from the last success. With a new e2e test (`test_task_source_freshness_failed_first_run`), the new test fails on 2026.9.12 and passes with the patch; `test_task_failed_freshness`, `test_task_source_hash_not_written_on_failure`, `test_task_source_freshness`, `test_task_source_freshness_multi_output`, `test_task_source_files_only_changed`, `test_task_dep_invalidates_sources`, `test_task_run_sources`, `test_task_source_freshness_hash_contents_no_baseline`, and `test_task_source_comparison_no_exec` pass either way.

This comment was drafted with an AI assistant; I ran the reproduction, the e2e tests before and after the patch, and checked the source trace.
~~~

### Finding B: documentation says "oldest output", code compares the newest

1. **Upstream's fault?**
   Yes:
   `docs/tasks/task-configuration.md:503-505` and
   `src/task/task_source_checker.rs:1464` disagree,
   and the "Verification" experiment follows the code.
2. **Can upstream fix it?**
   Yes,
   in either the docs or the code.
3. **Supported use case?**
   Yes,
   documented behavior.
4. **Contribution welcome?**
   Same evidence as finding A.
5. **Likely fixed?**
   No signal either way;
   [jdx/mise#7656](https://github.com/jdx/mise/discussions/7656) shows the maintainers
   accept output-completeness fixes.
6. **Prototyped?**
   No.
   Which side is intended is a maintainer decision
   (stricter code versus corrected docs),
   and the auto-prototype rule's "minimal fix" is undefined until that is known.
   The audit is incomplete here on purpose:
   a docs-only diff would assert the lax behavior is intended,
   and a code diff would change skip behavior for every multi-output task.

Do not file as-is.
Low impact for this repository (no repository task uses `outputs`),
and no thread exists.

~~~md
Title: tasks: docs say freshness uses the oldest output, code uses the newest

`docs/tasks/task-configuration.md` (`sources`): "mise skips the task when the modification time of the oldest output file is newer than the modification time of the newest source file."

`get_last_modified` in `src/task/task_source_checker.rs` returns `.max()` over output mtimes, so the newest output is compared. Reproduction (2026.9.12):

```toml
[tasks.twoout]
run = "touch o1 o2"
sources = ["s2.txt"]
outputs = ["o1", "o2"]
```

```sh
echo x > s2.txt; mise run twoout
touch --date=2000-01-01 o1
mise run twoout   # "sources up-to-date, skipping" although o1 is older than s2.txt
```

Which is intended? If the docs, `get_last_modified` would need `.min()` over output files; if the code, the docs sentence should say "newest output".
~~~
