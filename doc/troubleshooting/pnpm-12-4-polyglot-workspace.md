# pnpm 12.3.4 lacks the 12.4 polyglot install and pipeline commands

## Symptom

A Mise-replacement assessment treated pnpm as an npm-only package manager and demoted it as a workspace orchestrator.
That assessment missed pnpm 12.4.0,
which adds Cargo and Python dependency installation plus `pnpm pipeline`.

This repository currently resolves pnpm 12.3.4 in `mise.lock:958-959`.
Its help lists `runtime` but not `pipeline`.
Invoking the newer command against that installed version produced:

```text
Error: ERR_PNPM_RECURSIVE_EXEC_FIRST_FAIL

  × Command "pipeline" not found
```

The probe was mistakenly run in the real repository rather than a disposable fixture.
pnpm refreshed `node_modules/.pnpm/lock.yaml`,
`node_modules/.modules.yaml`,
and workspace-state metadata,
but `git status --short` remained clean.
No tracked file changed.

## Root cause

The capability was added in pnpm 12.4.0,
after the repository's installed 12.3.4.
The official 12.4 release notes call the multi-ecosystem support and `pnpm pipeline` early or experimental,
so these are candidates rather than established replacement contracts.

Source was inspected at tag `v12.4.1`,
commit `19eb39448649c926bc63b0e9fa16f0e340701460`,
from `https://github.com/pnpm/pnpm.git`.

### Cargo and Python join the install plan

`pnpm/crates/cli/src/ecosystem_install.rs:13-39` gates the additional install tasks on workspace configuration:

```rust
pub(crate) fn is_enabled(config: &Config) -> bool {
    config.cargo.enabled || config.python.enabled
}

if config.cargo.enabled {
    plan = plan.with_task(cargo_deps::plan::<Reporter>(context.clone(), &inventory).await?);
}
if config.python.enabled {
    // ...
    plan = plan.with_task(python::plan::<Reporter>(/* ... */).await?);
}
```

Cargo support installs dependencies rather than compiling the workspace.
`pnpm/crates/cli/src/cargo_deps.rs:56-93` owns `Cargo.lock` and a marked source-replacement block in
`.cargo/config.toml`:

```rust
const MANAGED_START: &str = "# >>> pnpm-managed cargo sources >>>";
const MANAGED_END: &str = "# <<< pnpm-managed cargo sources <<<";

pub(crate) fn metadata_paths(root: &Path) -> [PathBuf; 2] {
    [root.join("Cargo.lock"), root.join(".cargo/config.toml")]
}
```

`pnpm/crates/cli/src/cargo_deps.rs:147-164` links prepared sources,
writes the Cargo configuration,
and writes the lockfile when it changed:

```rust
link_workspace(&self.root, &CRATES_SOURCE_DIRECTORY, &slots.crates)?;
write_cargo_config(&self.root, &self.index_url, &slots.git_sources)?;
// ...
pnpm_fs::write_atomic(&path, self.lock.as_bytes())?;
```

Python support still requires an interpreter.
`pnpm/crates/cli/src/python.rs:121-145` discovers `pyproject.toml` projects,
probes `python.executable`,
and includes `pylock.toml` in install metadata:

```rust
let metadata = manifests.iter().map(|path| path.with_file_name("pylock.toml")).collect();
// ...
let interpreter: Interpreter =
    host::run(&config.python.executable, "probe", serde_json::json!({})).await?;
```

`pnpm/crates/cli/src/python.rs:422-437` publishes the lock and atomically replaces the pnpm-owned environment link:

```rust
let lock_path = self.root.join("pylock.toml");
// ...
Some(None) => match pnpm_fs::remove_symlink_dir(&self.root.join(".venv")) {
```

### Pipeline tasks still come from package scripts

`pnpm pipeline` does not directly execute TypeScript task declarations.
`pnpm/crates/cli/src/cli_args/pipeline.rs:204-244` discovers workspace projects,
builds their project graph,
and selects affected projects after the frozen install:

```rust
/// Run the pipeline.
/// The frozen install has already happened by the time
/// this is called;
/// this is selection,
/// graph,
/// cache,
/// and report.
pub fn run_pipeline(/* ... */) -> miette::Result<PipelineOutcome> {
    // ...
    let (projects, _) = discover_workspace_projects(run.workspace_root, config)?;
    let graph = build_full_graph(&projects, config);
    // ...
}
```

`pnpm/crates/cli/src/cli_args/pipeline.rs:395-412` selects task scripts from each project's manifest:

```rust
let select_scripts = |project: &Path, task_name: &str| -> Vec<String> {
    let manifest = plan.graph[project].package.project.manifest.value();
    match ScriptSelector::new(task_name) {
        Ok(selector) => selector.select(manifest),
        Err(_) => Vec::new(),
    }
};
```

`pnpm/crates/cli/src/cli_args/pipeline.rs:1000-1079` then resolves and runs those manifest scripts.
Therefore a TypeScript-canonical repository still needs file-enforcer to generate or update package script entries and
`pnpm-workspace.yaml`.

### The feature has a known cross-ecosystem lifecycle boundary

Merged pnpm PR [#14740](https://github.com/pnpm/pnpm/pull/14740) records that `pnpm run` deliberately walks past a
Cargo-only or Python-only subdirectory to the enclosing npm project,
because Cargo and Python manifests do not own pnpm scripts.

The same PR records an unresolved lifecycle ordering limitation:
a Python project's `.venv/bin` is available to `pnpm run` and `pnpm exec`,
but not to npm lifecycle scripts during `pnpm install`,
because Node lifecycle preparation can run before the Python environment is published.
This matters if a Node install hook tries to invoke a Python console script.

## Verification

### Installed repository version

```bash
pnpm --version
# 12.3.4
```

```bash
pnpm --help
# Lists `runtime`; does not list `pipeline`.
```

The failed `pnpm pipeline --help` probe is recorded in the Symptom section and should not be repeated in the real
repository.

### Upstream version and source

```bash
git -C ~/temp/agent/pnpm-2026-09-11 remote get-url origin
# https://github.com/pnpm/pnpm.git

git -C ~/temp/agent/pnpm-2026-09-11 rev-parse HEAD
# 19eb39448649c926bc63b0e9fa16f0e340701460

git -C ~/temp/agent/pnpm-2026-09-11 tag --points-at HEAD
# pnpr@0.1.0-alpha.11
# v12.4.1
```

Official documentation confirms:

- `cargo.enabled` installs locked crate sources and writes Cargo source replacement;
- `python.enabled` resolves `pyproject.toml`,
  writes `pylock.toml`,
  and manages `.venv`;
- `pnpm pipeline` runs a frozen install,
  affected-project selection,
  task graph execution,
  caching,
  and aggregate failure reporting;
- all three 12.4 surfaces are experimental or described as early.

No pnpm 12.4 executable was run against this repository.
Runtime suitability remains unverified and must be exercised in a disposable fixture before adoption.

## Verified workarounds

### Correct capability screening by version

Treat pnpm 12.3.4 and 12.4+ as different candidates when discussing replacement coverage.
The repository can continue using 12.3.4 while 12.4 behavior is evaluated separately.

Tradeoff:
this does not provide the new features locally.
It prevents a planning error without changing the working environment.

### Keep TypeScript as the canonical source through file-enforcer

The existing `file-enforcer.config.ts` can produce `pnpm-workspace.yaml` and package manifest task adapters from typed
TypeScript data while pnpm consumes its native formats.

Tradeoff:
generated YAML and JSON remain in the checkout,
and commands such as `pnpm add` must not become competing writers for fields owned by TypeScript generation.
Ownership must be assigned per field before enabling mutation commands.

## What does not work

### Treating the installed `latest` request as current 12.4 behavior

`mise.lock` resolves pnpm 12.3.4,
so the repository does not currently expose 12.4 commands despite `mise.toml` requesting `latest`.

### Invoking a new command in the real repository to test command presence

On pnpm 12.3.4,
`pnpm pipeline --help` entered pnpm's fallback command path and refreshed ignored dependency metadata before reporting
`ERR_PNPM_RECURSIVE_EXEC_FIRST_FAIL`.
Use a disposable fixture and inspect `pnpm --help` first.

### Treating Cargo support as a Rust toolchain manager

pnpm installs and vendors crate dependencies.
It does not replace rustup,
Rust components and targets,
Cargo itself,
or compilation.

### Treating Python support as a Python runtime manager

pnpm probes the configured Python executable and builds environments from it.
It does not provision that interpreter.

### Treating pipeline configuration as native TypeScript

Pipeline relationships live in `pnpm-workspace.yaml`,
and executable task bodies come from project manifest scripts.
TypeScript-only authoring therefore requires generated adapters.

## Upstream filing artifact

Nothing to file.
This is a consumer-side version and planning mismatch,
not an upstream defect.

### Upstream filing decision

1.  Upstream fault:
    no.
    pnpm 12.3.4 correctly lacks a feature added in 12.4.0.
2.  Upstream fixability:
    not applicable because no defect is proposed.
3.  Supported use case:
    yes for 12.4+;
    official Cargo,
    Python,
    pipeline,
    and release documentation covers it.
4.  Contribution policy:
    not investigated because there is no proposed issue or patch.
5.  Likely upstream action:
    none requested.
6.  Minimal prototype:
    not applicable because the upstream-fault gate fails.

`.out-of-scope/` contains no pnpm-specific exemption.
Open and closed issue and pull-request searches for `"pipeline" "command not found"` returned no duplicate.
A broader merged-PR search found the implementing pipeline PR #14233 and the cross-ecosystem script-boundary PR #14740;
those are implementation evidence rather than reports of this version mismatch.
