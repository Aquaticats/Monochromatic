# pnpm 12.16.1: pnpmfile cannot enforce prune before install, and prune is not install plus flags

## Symptom

The request was to make `pnpm prune` run before every `pnpm install`, always, and to enforce that from
`.pnpmfile.mjs`.
Two failure modes surfaced while answering it:

1. There is no pnpmfile hook that runs before the install pipeline starts, so a pnpmfile cannot
   sequence any command ahead of `pnpm install`.
2. The summary "`pnpm prune` is just install with `modulesCacheMaxAge: 0`,
   `pruneDirectDependencies: true`, `pruneStore: true`" is wrong for the binary in use.
   It paraphrases `doc/handover/pnpm-modules-cache-benchmark.md` (a `pnpm11` TypeScript note) and even
   there omits a fourth option.
   `pnpm` here is the pacquet ELF (Rust) CLI, whose prune is a separate entry point.

## Root cause

### Step 1: pnpmfile hooks run inside a command that has already started

The documented hook surface is `readPackage`, `updateConfig`, `preResolution`, `afterAllResolved`,
`beforePacking`, plus resolvers, fetchers, and finders ([pnpmfile hooks][pnpmfile-hooks]).
The `pnpm11` TypeScript interface at `pnpm11/hooks/pnpmfile/src/Hooks.ts:10-21` still declares two
more, `filterLog` and `importPackage`, which the v12 docs mark deprecated and removed.
Timing is intra-command: `updateConfig` is "Called after pnpm reads its configuration"
([updateConfig timing][pnpmfile-config-timing]) and `preResolution` is "Called before the
dependencies are resolved" ([preResolution timing][pnpmfile-preresolution-timing]).
No hook precedes the command, and no hook schedules another command.
A hook can only refuse to proceed: "A hook that throws fails the install"
([throw behavior][pnpmfile-throws]).

One nuance: `updateConfig` runs at config-load time, before the command handler
(`pnpm11/pnpm/src/getConfig.ts:179-181`), which is the only reason a spawn from it lands ahead of
the install work.
It is still a pure config transform: `requireHooks.ts:249-254` gives it the config object and
expects one back, and no hook context exposes a command runner
(`pnpm11/hooks/pnpmfile/src/Hooks.ts:6-8`, `pnpm11/hooks/types/src/index.ts:11-31`).

Fixture probe on pnpm 12.16.1, `pnpm install` with logging hooks:

```txt
PNPMFILE-PROBE updateConfig ran        <- hook 1, first
. pnpm:devPreinstall: RAN devPreinstall <- root script 1
. preinstall: RAN preinstall            <- root script 2
PNPMFILE-PROBE preResolution ran        <- hook 2
```

### Step 2: `pnpm prune` in pnpm 12 (pacquet) is its own run entry point

The clone under test is `pnpm/pnpm` at `c7b46ae` (`chore(release): pacquet 12.16.1`),
matching the installed `pnpm v12.16.1` ELF at the toolchain bin path.

`pnpm/crates/cli/src/cli_args/prune.rs:66-71`:

```rust
base_install.lockfile_policy.trust = state.config.trust_lockfile;
base_install.lockfile_policy.disable_optimistic_repeat = true;
base_install.context.lockfile_path = Some(&lockfile_path);
// ...
.run_prune::<Reporter>(selection)
```

`pnpm/crates/package-manager/src/install/entry_points.rs:71-84`:

```rust
/// Execute `pacquet prune`, enabling extraneous direct-dependency
/// removal and optional workspace project selection.
pub async fn run_prune<Reporter: self::Reporter + 'static>(
    self,
    selection: Option<WorkspaceInstallSelection<'_>>,
) -> Result<(), InstallError> {
    Box::pin(self.run_inner::<Reporter>(InstallRunOptions {
        selection,
        manifests: crate::install::InstallManifestOptions {
            prune_direct_dependencies: true,
            ..Default::default()
        },
        ..Default::default()
    }))
    .await
}
```

So pacquet prune = the install pipeline with `prune_direct_dependencies: true`,
the optimistic-repeat-install fast path disabled, a lockfile path attached, and
`--prod` / `--dev` / `--no-optional` group filtering (`prune.rs` `dependency_groups`).
It does not zero `modules_cache_max_age` and it does not prune the store
(`pnpm store prune` is a separate command, `pnpm/crates/store-dir/src/prune.rs`).

### Step 3: the pnpm11 TypeScript handler has four options, not three

`pnpm11/installing/commands/src/prune.ts:60-63`:

```ts
    modulesCacheMaxAge: 0,
    optimisticRepeatInstall: false,
    pruneDirectDependencies: true,
    pruneStore: true,
```

`optimisticRepeatInstall: false` is the option the handover note dropped.
`doc/handover/pnpm-modules-cache-benchmark.md` records only the other three and labels them as the
whole of prune; that note has been amended in place.

### Step 4: orphan virtual-store sweeping is age-throttled config, not a prune flag

`pnpm/crates/package-manager/src/prune_virtual_store.rs:26-39`:

```rust
pub fn should_prune_virtual_store(
    is_global_virtual_store: bool,
    prior_pruned_at: Option<&str>,
    modules_cache_max_age: u64,
    now: SystemTime,
) -> bool {
    if is_global_virtual_store {
        return false;
    }
    match prior_pruned_at {
        Some(pruned_at) if !pruned_at.is_empty() && modules_cache_max_age > 0 => {
            cache_expired(pruned_at, modules_cache_max_age, now)
        }
        _ => true,
    }
}
```

The sweep of `node_modules/.pnpm/<dir>` entries the wanted lockfile no longer references runs when
`modulesCacheMaxAge` is `0`, when no `prunedAt` is recorded, or when the recorded age expired.
This repo sets `modulesCacheMaxAge: 0` in `pnpm-workspace.yaml:13` (commit `7ae77227d`),
so every install already sweeps orphans immediately.

### Step 5: anything that calls `pnpm prune` from a script or hook recurses

`pnpm prune` loads the pnpmfile and runs the root `pnpm:devPreinstall` and `preinstall` scripts
(fixture probe, pnpm 12.16.1):

```txt
PNPMFILE-PROBE updateConfig ran
. pnpm:devPreinstall: RAN devPreinstall
. preinstall: RAN preinstall
PNPMFILE-PROBE preResolution ran
```

A lifecycle script or hook that spawns `pnpm prune` therefore re-enters itself.
An environment guard is mandatory.

## Verification

Version under test: `pnpm v12.16.1`, the pacquet ELF at the toolchain bin path
(`file` reports ELF; `pnpm --version` reports `12.16.1`).
Source clone: `gh repo clone pnpm/pnpm "${HOME}/temp/agent/pnpm-src-2026-10-29" -- --depth 1`,
HEAD `c7b46ae` (`chore(release): pacquet 12.16.1 (#17433)`).

Harness: a disposable single-project fixture (`${HOME}/temp/agent/pnpm-preinstall-probe`) with
`ms@2.1.3`, `is-odd@3.0.1`, and `escape-string-regexp@1.0.5` as dependencies and four scripts
(`pnpm:devPreinstall`, `preinstall`, `install`, `postinstall`) each printing `RAN <name>`.
Replace the fixture scripts with `node -e "console.log('RAN <name>')"` and rerun the commands below
to reproduce.

### Catalog: commands that behave cleanly

Root project scripts run, per full-output captures on pnpm 12.16.1:

- `pnpm install`: `pnpm:devPreinstall`, then `preinstall`, both before packages link;
  `install`, `postinstall` after.
- `pnpm prune`: `pnpm:devPreinstall`, `preinstall`, `install`, `postinstall` all run.
- `pnpm add <pkg>` (new package actually added): only `pnpm:devPreinstall` runs.
- The pnpmfile loads for both `pnpm install` and `pnpm prune`, `updateConfig` first.

The `pnpm add` row is by design in the `pnpm11` source, which pacquet mirrors:
`pnpm:devPreinstall` runs for every mutation kind passing `installRunsDevPreinstall`
(`pnpm11/installing/deps-installer/src/install/installPredicates.ts:36-40`, so `--ignore-scripts`,
`ignorePackageManifest`, and any install excluding devDependencies skip it),
while early root `preinstall` requires the root project's mutation to be `'install'`
(`installPredicates.ts:48-63`).
`add` is `'installSome'`, so it gets no early `preinstall`; its post-link stages are path-dependent
(the isolated full-include linker excludes `installSome` entirely,
`linkResolvedProjects.ts:424-426`; the hoisted or group-filtered path runs
`preinstall`/`install`/`postinstall`, `deps-restorer/src/projectLifecycle.ts:101-104`).
`prune --prod` drops the prepare family and `--ignore-scripts` runs nothing
(`pnpm11/installing/commands/test/prune.ts:116-182`).

Clean behavior used by the workarounds:

- `modulesCacheMaxAge: 0` sweeps orphan virtual-store dirs on every install
  (`should_prune_virtual_store` returns `true` when the age is `0`).
- A guarded `updateConfig` hook can spawn `pnpm prune` at the head of an install
  (verified below).

### Catalog: patterns that fail

- Sequencing any command from a pnpmfile hook before install begins: no such hook exists
  (hook list above).
- Calling `pnpm prune` from `pnpm:devPreinstall`, `preinstall`, or a hook without an
  environment guard: `pnpm prune` re-runs those scripts and reloads the pnpmfile (probe above),
  so the call recurses.
- Treating prune as install plus three flags: wrong in both implementations
  (four options in `pnpm11/installing/commands/src/prune.ts:60-63`;
  a separate `run_prune` entry point in pacquet).

## Verified workarounds

### A. `modulesCacheMaxAge: 0` (config-level, already in this repo)

`pnpm-workspace.yaml:13` sets `modulesCacheMaxAge: 0`, so every `pnpm install` sweeps orphan
virtual-store entries instead of caching them for seven days.
Tradeoffs: the sweep cost is paid on every install; this removes only virtual-store orphans,
not extraneous direct-dependency links (that is `prune_direct_dependencies`) and not unreferenced
store files (`pnpm store prune` is separate).
Bypassable only by overriding the setting.

### B. Guarded `updateConfig` spawn of `pnpm prune` (pnpmfile-enforced, verified)

`.pnpmfile.mjs`:

```js
import { execFileSync } from 'node:child_process';

export const hooks = {
  updateConfig(config) {
    if (process.env.PRUNE_RAN) return config;
    execFileSync('pnpm', ['prune'], {
      stdio: 'inherit',
      env: { ...process.env, PRUNE_RAN: '1' },
    });
    return config;
  },
};
```

Verified run on pnpm 12.16.1 (`pnpm install`, exit 0):

```txt
PNPMFILE-PROBE outer before spawn
PNPMFILE-PROBE updateConfig skip (guarded)
. pnpm:devPreinstall: RAN devPreinstall
. preinstall: RAN preinstall
Already up to date
. install: RAN install
. postinstall: RAN postinstall
Done in 403ms using pnpm v12.16.1      <- nested pnpm prune
PNPMFILE-PROBE outer after spawn
Already up to date
Done in 551ms using pnpm v12.16.1      <- outer pnpm install
```

Tradeoffs: a nested package-manager run inside an install (store and modules-dir state are shared,
so this is unsupported territory); the nested run executes the project lifecycle scripts
(the outer run then printed only "Already up to date" and no script lines, mechanism untraced);
`ignorePnpmfile: true` disables the enforcement entirely;
the environment guard is load-bearing against recursion.

### C. A mise task wrapper (repo-idiomatic, recursion-free)

Run `pnpm prune` and `pnpm install` as two sequential tasks (`mise.toml` `run = [...]`),
never from inside a hook or lifecycle script.
Two separate pnpm processes cannot recurse or nest.
Tradeoffs: only installs routed through `mise run` are covered (this repo's task policy routes
them there); a raw `pnpm install` typed anywhere else skips the wrapper.

## What does not work

- A pnpmfile "preInstall" hook: none exists; `updateConfig` is the earliest hook and it is
  already inside the command.
- Unguarded `pnpm prune` from `pnpm:devPreinstall` or `preinstall`: prune runs both scripts,
  so the recipe recurses.
- Unguarded `pnpm prune` from `updateConfig`: prune loads the pnpmfile, so the hook re-enters.
  The first probe attempt without a guard produced only two probe lines and one `Done in` line
  for two pnpm processes (recursion stopped by process exit, output interleaved),
  which is why the verified recipe keeps the `PRUNE_RAN` guard.
- The three-flag paraphrase of prune: it drops `optimisticRepeatInstall: false` (pnpm11) and
  does not describe pacquet's `run_prune` at all.

## Upstream filing decision

`.out-of-scope/` checked first: `bun-install.md`, `cargo-workspace.md`,
`claude-code-upstream-bugs.md`, `codex-harness.md`, `jsr.md`, `lightningcss.md`,
`low-impact-typescript-formatting.md`, `module-es-monolith.md`, `pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`, `typescript-project-references.md`.
No pnpm entry, so no exemption applies.

Duplicate search (`gh search issues` over pnpm/pnpm, terms: preinstall order, devPreinstall pacquet,
prune semantics): [#3836](https://github.com/pnpm/pnpm/issues/3836) (preinstall ordering, closed as
duplicate of #3760) and [#13313](https://github.com/pnpm/pnpm/issues/13313)
(`pnpm:devPreinstall` missing under pacquet, closed 2026-07-26) cover the lifecycle topics.

Six-constraint walk:

1. Is it really upstream's fault?
   No.
   The v12 behavior matches the v12 docs: prune is documented as removing packages no longer in the
   lockfile, and the pnpmfile hook list documents no pre-install hook.
   The inexactness was our paraphrase of `pnpm11` source and a handover note, not a defect.
2. Can upstream fix it?
   Not applicable given 1; nothing is broken.
3. Are they supporting this use case?
   The supported primitives are the lifecycle scripts (`pnpm:devPreinstall` runs "before any
   dependency is installed", [scripts docs][scripts-devpreinstall]) and settings
   (`modulesCacheMaxAge`).
4. Would the repo welcome our contribution?
   Not reached.
5. Will they likely fix it?
   Not reached.
6. Have we prototyped a minimal fix?
   Not reached; there is no fix to prototype.

Verdict: nothing to add.
No issue or comment draft is kept.
The asymmetries worth knowing are recorded here instead: pacquet prune does not zero
`modulesCacheMaxAge` and does not prune the store the way the `pnpm11` handler did,
and `pnpm add` runs only `pnpm:devPreinstall` under the isolated full-include linker while the
scripts page's "when it installs specific packages" sentence describes the hoisted or
group-filtered path.

## Unknowns

- After the nested prune in workaround B ran the lifecycle scripts, the outer install printed
  "Already up to date" and no script lines of its own.
  The skip mechanism (likely a scripts-run record in the modules manifest) is untraced.
- The `pnpm add` script set matches the `pnpm11` mutation gate cited in Verification;
  the same gate in the pacquet CLI is inferred from the matching probe result, not read directly.
- `prune_direct_dependencies` is a config field (`pnpm/crates/config/src/settings.rs:264`) and
  materialization honors `config.prune_direct_dependencies`
  (`pnpm/crates/package-manager/src/install/run/materialization_inputs.rs:131-133`),
  but no `pnpm/docs` page documents the key.
  Whether `updateConfig` can set it effectively is untested.

[pnpmfile-hooks]: https://pnpm.io/pnpmfile
[pnpmfile-config-timing]: https://pnpm.io/pnpmfile#:~:text=Called%20after%20pnpm%20reads%20its%20configuration
[pnpmfile-preresolution-timing]: https://pnpm.io/pnpmfile#:~:text=Called%20before%20the%20dependencies%20are%20resolved
[pnpmfile-throws]: https://pnpm.io/pnpmfile#:~:text=A%20hook%20that%20throws%20fails%20the%20install
[scripts-devpreinstall]: https://pnpm.io/scripts#:~:text=Runs%20before%20any%20dependency%20is%20installed
