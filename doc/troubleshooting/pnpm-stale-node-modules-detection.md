# pnpm 12.5.1: `--lockfile-only` drift goes unreported, and stale optional links re-import on every install

Investigated for issue [#570](https://github.com/Aquaticat/Monochromatic/issues/570)
(handover:
 [config-oxlint stale plugin bundle](../handover/config-oxlint-stale-plugin-bundle-issue-570.md)).
The question was how much of the #570 failure chain belongs to pnpm.

Short answer:
pnpm owns link 1 of the chain (install state drifts from manifests),
and its behavior there is mostly defensible design.
`--lockfile-only` is documented to skip `node_modules`,
and pnpm does ship an accurate drift detector (`verifyDepsBeforeRun`),
but it only runs inside `pnpm run` and `pnpm exec`,
which this repo's tasks never call.
Two genuine gaps surfaced along the way:
the install summary prints `Already up to date` while it creates or removes workspace links in non-root projects,
and,
 combined with this repo's `modulesCacheMaxAge: 0`,
leftover links to skipped optional dependencies make every frozen install re-import six packages.

All source citations are pnpm at tag `v12.5.1`
(commit `859a9cfe39f2afd0c4198f14ca7ef628ee1de70e`,
 release commit title "chore(release):
 pacquet 12.5.1").
pnpm 12 is the Rust rewrite under `pnpm/crates/`;
the TypeScript v11 code lives under `pnpm11/` in the same repository.
The installed binary is `~/.local/share/mise/installs/pnpm/12.5.1/pnpm` (ELF,
 `pnpm --version` prints `12.5.1`).

## Symptom

### Incident sequence

1.  A merge changed several `package.json` files.
2.  `pnpm install --lockfile-only` ran and exited 0.
3.  `node_modules` still lacked the newly declared workspace link.
4.  Builds bundled against the incomplete tree and succeeded;
    the broken bundle then broke every lint run (issue #570).

Nothing between steps 2 and 4 printed a warning.
`pnpm list --filter @monochromatic-dev/config-oxlint --depth 0` in that state lists the old dependencies,
omits the new one,
prints no warning,
and exits 0.

### Repair installs claim nothing happened

After `pnpm install --lockfile-only`,
the repairing `pnpm install --offline` (or `pnpm install --frozen-lockfile --offline`) creates the missing link
but prints:

```text
Lockfile is up to date, resolution step is skipped
Already up to date
Done in 271ms using pnpm v12.5.1
```

Removing a workspace dependency and reinstalling also prints `Already up to date` while it deletes the link.

### "added 6" on every no-op frozen install in the main checkout

In the main checkout,
every `pnpm install --frozen-lockfile --offline` prints:

```text
Packages: +6
++++++
Progress: resolved 6, reused 6, downloaded 0, added 6, done
```

The six packages (from `--reporter=ndjson` `imported` events) are
`oxlint@1.85.0`,
`oxlint-tsgolint@7.0.2002`,
`rolldown@1.2.9`,
`satteri@0.10.5`,
`typescript@7.0.2`,
and `yuku-parser@0.10.2`.
A fresh worktree of the same commit prints `Already up to date` instead.

## Root cause

### What `--lockfile-only` writes

The docs say it only updates `pnpm-lock.yaml` and `package.json`
(pnpm.io `docs/cli/install.md` lines 105 to 110):

```markdown
### --lockfile-only

When used, only updates `pnpm-lock.yaml` and `package.json`. Nothing gets written to the `node_modules` directory.
```

The source matches.
The CLI maps the flag at `pnpm/crates/cli/src/cli_args/install.rs:329`:

```rust
base_install.execution.lockfile_only = link.lockfile.only;
```

The fresh-lockfile path ends in `finish_lockfile_only`
(`pnpm/crates/package-manager/src/install_with_fresh_lockfile/persist.rs:213` to `230`),
which saves only the wanted lockfile:

```rust
} else if opts.write.config.lockfile {
    let can_record_lockfile_verification = save_wanted_lockfile(
        &opts.built_lockfile,
        &opts.write.dir.join(opts.write.config.wanted_lockfile_name()),
```

The apply stage then returns before linking and before any state file is written
(`pnpm/crates/package-manager/src/install/apply_materialization.rs:185` to `238`):

```rust
if complete_resolve_only::<Reporter>(&ResolveOnlyCompletionInputs { /* ... */ })? {
    return Ok(());
}
// ...
link_apply_projects::<Reporter>(&inputs, &state).await?;
commit_apply_state::<Reporter>(&inputs, &state, metadata)?;   // .modules.yaml + node_modules/.pnpm/lock.yaml
// ...
write_applied_workspace_state(&inputs)?;                       // node_modules/.pnpm-workspace-state-v1.json
```

So `--lockfile-only` rewrites `pnpm-lock.yaml` only.
It does not touch `node_modules/.pnpm/lock.yaml` (the "current lockfile"),
`node_modules/.modules.yaml`,
or `node_modules/.pnpm-workspace-state-v1.json`,
and it writes no marker saying `node_modules` is now behind.

It does not need one:
the mismatch between `pnpm-lock.yaml` and `node_modules/.pnpm/lock.yaml`
is itself the durable evidence,
and the state file's `lastValidatedTimestamp` stays older than the new lockfile mtime.

### The detector exists, but only `run` and `exec` consult it

`verifyDepsBeforeRun` is documented as running "on `pnpm run` and `pnpm exec` commands"
(pnpm.io `docs/settings/build.md` lines 175 to 186,
 default `install`).
The gate is `verify_deps_before_run` in `pnpm/crates/cli/src/cli_args/verify_deps.rs:37` to `73`,
called only from `run.rs:191`,
 `run.rs:283`,
 `exec.rs:107`,
 and `exec.rs:125` in the same directory.
There is no `status` or `deps status` command in `pnpm/crates/cli/src/cli_args/`,
none in v11 (`@pnpm/deps.status` under `pnpm11/deps/status` is a library,
 not a command),
and the Node-API crate (`pnpm/crates/napi/src`) exposes no status check.

The check itself is `check_deps_status_before_run_at`
(`pnpm/crates/package-manager/src/install/workspace_state/discovery.rs:29`),
which loads the workspace state (missing state reports "Cannot check whether dependencies are outdated")
and calls `check_deps_status_before_run`
(`pnpm/crates/package-manager/src/optimistic_repeat_install/deps_status.rs:43` to `81`).
It compares:

- settings,
   catalogs,
   config dependencies,
   patches and pnpmfiles against the recorded state;
- the workspace project list,
   and whether every project with dependencies has a `node_modules`;
- each `package.json` and `pnpm-lock.yaml` mtime against `lastValidatedTimestamp`;
- for a modified manifest,
   whether the wanted lockfile still satisfies it
  (`manifest_agreement.rs:229`,
   message `a modified manifest is no longer satisfied by the lockfile`);
- when `pnpm-lock.yaml` is newer than `lastValidatedTimestamp`,
  whether it equals the current lockfile
  (`pnpm/crates/package-manager/src/optimistic_repeat_install/manifest_agreement.rs:159` and `250` to `281`):

```rust
if modified_at_or_after(wanted.mtime, state.last_validated_timestamp) {
    assert_wanted_lockfile_equals_current(
// ...
            if materialized_shape_matches(wanted, &current, included) {
                Ok(())
            } else {
                Err("the installed dependencies are not up to date with the lockfile")
```

That last comparison is what catches the #570 state.
When the check passes after a modification,
`settle_content_check` rewrites the workspace state to refresh the timestamp (`deps_status.rs:192` to `236`),
so the check is not strictly read-only.

### `--frozen-lockfile` disables the fast no-op path

`install_is_already_up_to_date` (`pnpm/crates/package-manager/src/install/run/fast_path.rs:41` to `47`):

```rust
let eligible = check.mutation.is_full_install()
    && matches!(check.update_seed_policy, UpdateSeedPolicy::KeepAll)
    && !check.frozen_lockfile
    && !check.workspace.config.force
    && !check.disable_optimistic_repeat_install;
```

So `pnpm install --frozen-lockfile` always walks the full headless install,
and its `Already up to date` line comes from the reporter,
 not from the fast path.

### Why the install summary says "Already up to date" while linking

`pnpm/crates/default-reporter/src/state/progress.rs:132` to `160`
only counts stats events whose prefix is the current directory,
and prints `Already up to date` when that project added and removed zero packages:

```rust
if prefix != &self.rendering.cwd {
    return;
}
// ...
if added == 0 && removed == 0 {
    // ...
    self.display.frame.emit(&mut slot, "Already up to date".to_string(), false);
```

A workspace link added to `package/config/oxlint/node_modules` is a change in another importer,
and a `link:` dependency is not a package import,
so the root summary reports nothing.
Scoping the summary to the current project is long-standing pnpm behavior;
wording it as "up to date" when other importers changed is the misleading part.

### Why six packages re-import on every frozen install

The main checkout's virtual store holds 49 dangling symlinks,
dated 2026-09-07,
inside exactly those six slots,
each pointing at an optional platform binding that `.modules.yaml` lists under `skipped`,
for example `node_modules/.pnpm/typescript@7.0.2/node_modules/@typescript/typescript-aix-ppc64`
(16 in `typescript@7.0.2`,
 13 in `oxlint`,
 9 in `rolldown`,
 6 in `yuku-parser`,
 3 in `satteri`,
 2 in `oxlint-tsgolint`).

The warm-slot probe requires such links to be absent
(`pnpm/crates/deps-restorer/src/create_virtual_store/snapshot_plan/children.rs:151` to `167`,
called from `snapshot_plan.rs:330`):

```rust
pub(super) fn optional_child_matches(child_path: &Path, should_exist: bool) -> std::io::Result<bool> {
    if should_exist { /* ... */ }
    match std::fs::symlink_metadata(child_path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
```

A mismatch sends the slot to re-materialization,
but re-materialization never removes those links.
`create_symlink_layout` just skips skipped targets
(`pnpm/crates/deps-restorer/src/create_symlink_layout.rs:69` to `71`),
and optional children are only removed when optional dependencies are excluded altogether
(`pnpm/crates/deps-restorer/src/create_virtual_dir_by_snapshot.rs:129` to `131`):

```rust
if !self.dependencies.include_optional {
    self.remove_optional_children(&slot.node_modules)?;
}
```

So the slot never converges.

The probe only runs because this repo sets `modulesCacheMaxAge: 0`.
Without it,
 a frozen install whose current lockfile matches returns early and never probes warm slots;
with it,
 a prune is always due,
which declines that early return
(`pnpm/crates/package-manager/src/install/prepare_modules_state/up_to_date.rs:80` to `81`):

```rust
let tree_intact = context.repeat.rebuild.is_none()
    && !modules_cache_prune_due(config, context.modules_manifest)
```

and `should_prune_virtual_store` returns `true` whenever the max age is 0
(`pnpm/crates/package-manager/src/prune_virtual_store.rs:42` to `45`).
A fixture without the setting kept the stale link and imported nothing on every run;
the same fixture with it re-imported on every run (see "Verification").

The regular-dependency probe in the same file explicitly avoids this trap
(`children.rs:11` to `14`):
"re-importing would not remove them,
 so requiring absence would re-materialize the slot on every install."
The optional probe does the opposite.
The same code is unchanged on upstream `main` at `7c1b501` (2026-09-26).

Which install created the links is unknown.
A fresh fixture installed with pnpm 11.25.0 (the version current on 2026-09-07) created none.

### Other pnpm settings in this repo that affect detection

From `pnpm-workspace.yaml`:

- `dedupeDirectDeps: true` skips linking a dependency into a project's `node_modules`
  when the root `node_modules` already has it
  (pnpm.io `docs/settings/other.md` lines 312 to 317;
   observed:
   `package/config/oxlint/node_modules/@monochromatic-dev/` lacks the declared `config-typescript`).
  Resolution then walks up to the root,
  so a root-level dependency can mask a missing per-project link.
  In #570 `module-logger` is not a root dependency,
   so it did not mask anything.
- `hoist: false`,
   `hoistWorkspacePackages: false`,
   and an empty `publicHoistPattern`
  (recorded in `.modules.yaml`) remove the other masking source:
  undeclared packages are not hoisted into reach.
- No `injected` dependencies or `dependenciesMeta` exist,
   so no install-time copies can go stale.
- Workspace dependencies use `workspace:*`,
   so `linkWorkspacePackages` does not matter.
  Workspace links exist only after an install,
   never after a lockfile-only run.

## Verification

Environment:
pnpm 12.5.1 from mise,
Fedora (Bazzite) on Btrfs,
`packageImportMethod: clone-or-copy`,
a throwaway worktree at `~/temp/agent-570-pnpm/wt` (outside `~/temp/agent`,
 so no ancestor `node_modules`),
and a reflink copy of the main checkout without `.git` and `target`.
All installs ran with `--offline` against the warm store.

### Harness: lockfile-only drift

```sh
# throwaway worktree, after one full `pnpm install --frozen-lockfile --offline`
node --eval 'const f="package/config/oxlint/package.json";const fs=require("fs");
  const j=JSON.parse(fs.readFileSync(f,"utf8"));
  j.dependencies={"@monochromatic-dev/logger-core":"workspace:*",...j.dependencies};
  fs.writeFileSync(f,JSON.stringify(j,null,2)+"\n")'
pnpm --config.verify-deps-before-run=error exec true   # exit 1
pnpm install --lockfile-only --offline                 # exit 0
pnpm --config.verify-deps-before-run=error exec true   # exit 1
pnpm install --offline                                 # prints "Already up to date", creates the link
```

State-file hashes around `pnpm install --lockfile-only`:
only `pnpm-lock.yaml` changed (sha256 prefix `052f17012d7f` to `e85612613737`);
`node_modules/.pnpm/lock.yaml`,
 `.modules.yaml`,
 and `.pnpm-workspace-state-v1.json` kept their hashes and mtimes,
and `package/config/oxlint/node_modules/@monochromatic-dev/logger-core` stayed absent.

### Check outcomes

Detected (exit 1 with `--config.verify-deps-before-run=error`,
 warning line with `=warn`):

- manifest edited,
   no install:
  `× a modified manifest is no longer satisfied by the lockfile`
- manifest edited,
   then `pnpm install --lockfile-only`:
  `× the installed dependencies are not up to date with the lockfile`

Passes (exit 0):

- after a full install;
- after `touch`-ing `package.json`,
   `pnpm-lock.yaml`,
   or `.pnpmfile.mjs` without content changes.

Silent (exit 0,
 no warning):

- `pnpm install --lockfile-only` itself;
- `pnpm list --filter <project> --depth 0`.

### Timings

`hyperfine --warmup 3 --runs 30` on a quiet host
(an earlier round taken while a 16 GB reflink copy ran had 20 to 70 ms spreads and is discarded):

- `pnpm --config.verify-deps-before-run=error exec true`,
   up to date:
   85.0 ms ± 2.8 ms
- same,
   manifest drift:
   84.2 ms ± 2.7 ms
- same,
   after `--lockfile-only`:
   103.9 ms ± 2.2 ms
- `pnpm --config.verify-deps-before-run=false exec true` (exec overhead without the check):
   38.1 ms ± 1.2 ms
- `pnpm install --offline`,
   up to date (fast path):
   96.9 ms ± 7.2 ms
- `pnpm install --frozen-lockfile --offline`,
   up to date (full path):
   215.3 ms ± 10.9 ms

### Harness: stale optional links

```sh
# positive control in the clean throwaway worktree
cd node_modules/.pnpm/typescript@7.0.2/node_modules/@typescript
ln --symbolic ../../../@typescript+typescript-aix-ppc64@7.0.2/node_modules/@typescript/typescript-aix-ppc64 \
  typescript-aix-ppc64
cd -
pnpm install --frozen-lockfile --offline   # Packages: +1
pnpm install --frozen-lockfile --offline   # Packages: +1 again; the link survives
```

In the reflink copy of the main checkout,
two consecutive runs each printed `Packages: +6`;
after deleting the 49 dangling links,
two consecutive runs each printed `Already up to date`.

Minimal standalone harness,
run in `podman run --rm --memory=2g --cpus=2 docker.io/library/rust:latest` against the public registry
(the released binary is the host's `~/.local/share/mise/installs/pnpm/12.5.1/pnpm`):

```sh
# repro.sh <pnpm binary> workspace   (EXTRA holds extra pnpm-workspace.yaml lines)
set -eu
bin="$1"
layout="$2"
dir="$(mktemp -d)"
cd "$dir"
if [ "$layout" = workspace ]; then
  printf 'packages:\n  - a\n%b' "${EXTRA:-}" > pnpm-workspace.yaml
  mkdir a
  printf '{ "name": "a", "private": true }\n' > a/package.json
fi
printf '{ "name": "repro", "private": true, "dependencies": { "typescript": "7.0.2" } }\n' > package.json
"$bin" install --store-dir /tmp/store > install0.log 2>&1
link=node_modules/.pnpm/typescript@7.0.2/node_modules/@typescript/typescript-aix-ppc64
ln -s ../../../@typescript+typescript-aix-ppc64@7.0.2/node_modules/@typescript/typescript-aix-ppc64 "$link"
run() {
  "$bin" install --frozen-lockfile --store-dir /tmp/store --reporter=ndjson > run.log 2>&1
  echo "$layout $1: imported=$(grep -c '"status":"imported"' run.log || true) stats=$(grep -o '"added":[0-9]*' run.log | tr '\n' ' ') link=$(test -L "$link" && echo present || echo absent)"
}
run run1
run run2
run run3
```

Released 12.5.1:

- single project,
   or workspace without `modulesCacheMaxAge`:
  `imported=0` on all three runs,
   link present (the frozen early return never probes the slot);
- workspace with `EXTRA='modulesCacheMaxAge: 0\n'`:
  `imported=1 stats="added":1` on all three runs,
   link present.

## Verified workarounds

### Check install freshness before building

Run the `verifyDepsBeforeRun` gate as a standalone check:

```sh
pnpm --config.verify-deps-before-run=error exec true
```

It catches both manifest drift and lockfile-only drift,
costs about 47 ms over bare `pnpm exec` (85.0 ms against 38.1 ms),
and exits 1 with pnpm's own message.

Tradeoffs:
it borrows `exec` as a vehicle;
a passing check after a manifest `touch` rewrites `.pnpm-workspace-state-v1.json`;
a missing state file reports "Cannot check whether dependencies are outdated" (treat it as outdated);
and it says nothing about build outputs such as `package/config/oxlint/dist`,
so #570's link 3 still needs its own freshness tracking.

### Run a plain install instead of checking

`pnpm install --offline` reaches the fast path in 96.9 ms when nothing changed and repairs drift when something did.

Tradeoffs:
it may rewrite `pnpm-lock.yaml` when manifests changed,
which a CI-style `--frozen-lockfile` run would reject instead;
and its summary line cannot be used as evidence of what changed (`Already up to date` hides link changes).

### Avoid `--lockfile-only` after merges

Use `pnpm install` after any merge that touches manifests.
`--lockfile-only` remains fine for lockfile-only commits where nobody builds from the tree.

Tradeoff:
 slower than a lockfile-only run when many packages change.

### Remove leftover optional links

```sh
find node_modules/.pnpm -mindepth 3 -maxdepth 4 -path '*/node_modules/*' -xtype l -delete
```

Verified on the reflink copy:
 `+6` on every run before,
 `Already up to date` on every run after.

Tradeoffs:
it deletes every dangling link in the virtual store,
including any that point at a slot a concurrent install is still writing;
run it only when no install is running.
A dangling link for a dependency that should exist is recreated by the next install,
so the deletion cannot lose a required link.

### Drop `modulesCacheMaxAge: 0` (not recommended)

Removing the setting restores the frozen early return,
so the stale links are no longer probed and the re-imports stop (fixture evidence only:
`imported=0` on three runs without the setting).

Tradeoffs:
the stale links stay on disk,
and the setting exists to prune orphaned virtual-store entries
(rationale in the `pnpm-workspace.yaml` comment and [pnpm-modules-cache](pnpm-modules-cache.md)).
Deleting the dangling links is the better fix.

## What does not work

- Reading install output as evidence:
  `Already up to date` is printed while workspace links in other projects are created or removed.
- `pnpm list` as a check:
   it reports installed state silently and exits 0.
- `pnpm install --frozen-lockfile --offline` as a check:
   it repairs instead of reporting,
  and never takes the fast path,
   so it costs 215 ms even when nothing changed.
- `--config.optimistic-repeat-install=false` to force the full path:
  irrelevant under `--frozen-lockfile`,
   which already disables the fast path.
- My first reading of `check_deps_status_before_run`,
  that it only compared manifests with `pnpm-lock.yaml` and so would miss lockfile-only drift,
  was wrong:
  `manifest_agreement.rs:159` compares the wanted and current lockfiles whenever the wanted one is newer,
  and the harness produced `the installed dependencies are not up to date with the lockfile`.
- Reproducing the `added 6` in a fresh worktree failed repeatedly
  (plain,
   after copying all 118 `dist` directories from main,
   after `touch`-ing manifests and the pnpmfile):
  the cause lives only in the main checkout's `node_modules`.
- Attributing the dangling links to pnpm 11.25.0:
  a fresh fixture depending on `typescript@7.0.2` with the repo's `supportedArchitectures` got no dangling links.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` has no pnpm entry
(files:
 `bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
 `codex-harness.md`,
 `jsr.md`,
`lightningcss.md`,
 `low-impact-typescript-formatting.md`,
 `module-es-monolith.md`,
 `pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
 `typescript-project-references.md`).

Duplicate search (`gh search issues --repo pnpm/pnpm`,
 open and closed):
`lockfile-only`,
 `lockfile-only node_modules`,
 `verifyDepsBeforeRun`,
 `verify-deps-before-run command`,
`"Already up to date"`,
 `Already up to date workspace link added`,
 `deps status command`,
`check node_modules up to date`,
 `dangling symlink optional`,
 `skipped optional dependency stale symlink`,
`optional dependency symlink every install Packages`,
 `frozen-lockfile reimports every run`,
`supportedArchitectures reinstall every time`;
`gh search prs --repo pnpm/pnpm 'optional children'`.
Closest hits:
[#14891](https://github.com/pnpm/pnpm/issues/14891) (frozen install and `verifyDepsBeforeRun` never converge
when a lockfile snapshot is unreachable from every importer,
 closed 2026-09-14)
has the same `Packages: +N` every run symptom but a different cause (unreachable snapshots),
and its fix (`materialized_shape_matches`) is in 12.5.1;
[#12090](https://github.com/pnpm/pnpm/issues/12090) (`Already up to date` with missing `node_modules`,
 closed)
is a different defect.
No duplicate was found for any candidate.

Candidates and outcome:

- Stale optional links re-import slots forever:
   file (walked below).
- `--lockfile-only` leaves no marker:
   do not file.
  The documented contract is met,
   and the wanted-versus-current comparison already detects the state.
- No standalone status command:
   do not file now.
  It is a feature request;
  the existing `pnpm exec true` vehicle works,
  and v12 feature requests compete with the backlog in [#15277](https://github.com/pnpm/pnpm/issues/15277).
- `Already up to date` hides non-root link changes:
   do not file yet.
  Only v12 was observed;
   v11 behavior is unverified,
  and the prefix-scoped summary is long-standing design.

Constraints for the stale optional link bug:

1.  Upstream's fault:
     yes.
    The probe requires absence of links that no code path removes;
    the sibling regular-dependency probe documents exactly this hazard and avoids it.
2.  Upstream can fix it:
     yes;
     the prototype below is a single-function change.
3.  Supported use case:
     yes.
    `supportedArchitectures` and platform-skipped optional dependencies are documented features,
    and repeat installs are meant to converge (#14891's expected behavior,
     which maintainers fixed).
4.  Contributions welcome:
     yes.
    `CONTRIBUTING.md` "AI-assisted contributions" welcomes agent-made contributions
    and requires a footer naming the agent and the model;
    the draft carries that footer.
5.  Likely to fix:
     yes;
     the neighbouring non-convergence bug #14891 was fixed within weeks.
6.  Prototype:
     see "Prototype".

### Prototype

Disposable clone:
`~/temp/agent/upstream-prototype.r5FSDsyK/pnpm`,
 since deleted
(origin `https://github.com/pnpm/pnpm.git`,
 HEAD `859a9cfe39f2afd0c4198f14ca7ef628ee1de70e` = `v12.5.1`,
push URL disabled).
Diff:
 [pnpm-stale-node-modules-detection.patch](pnpm-stale-node-modules-detection.patch).
It widens the post-link cleanup in `CreateVirtualDirBySnapshot::run`
from "all optional children when optional dependencies are excluded"
to "every optional child the symlink layout did not link",
the same predicate `optional_children_match` checks,
so the first re-materialization removes the stale links and the next probe matches.

Build,
 inside `podman run --rm --memory=8g --cpus=6 docker.io/library/rust:latest`
with only the disposable clone mounted:
`cargo build --locked -p pnpm-cli --bin pnpm`,
 exit 0.
To build without running upstream's `pnpm install` (which vendors crates through scripts),
the container copy dropped the `# >>> pnpm-managed cargo sources >>>` block from `.cargo/config.toml`
so Cargo fetched the `Cargo.lock` versions from crates.io;
that edit is build plumbing and is not part of the patch.

`repro.sh` with `EXTRA='modulesCacheMaxAge: 0\n'`:

```text
released
workspace run1: imported=1 stats="added":1  link=present
workspace run2: imported=1 stats="added":1  link=present
workspace run3: imported=1 stats="added":1  link=present
patched
workspace run1: imported=1 stats="added":1  link=absent
workspace run2: imported=0 stats="added":0  link=absent
workspace run3: imported=0 stats="added":0  link=absent
```

After the patched runs,
`node_modules/.pnpm/typescript@7.0.2/node_modules/@typescript/` still holds `typescript-linux-x64`
(the installable optional child stays linked)
and `node_modules/typescript/package.json` resolves.

Crate tests,
same container:
`cargo test --locked -p pnpm-deps-restorer --lib`
gives `496 passed; 7 failed; 5 ignored` both with and without the patch,
and the same 7 tests fail in both runs
(pnpmfile tests need Node.js,
 which `rust:latest` lacks,
and permission-denied tests cannot fail as root).
The patch adds no failures.
No new unit test was written;
an upstream PR would add one beside `snapshot_plan/tests.rs` `invalid_optional_child_entries_do_not_match`.

Constraint 6 therefore holds,
and all six constraints read "yes":
the draft is fileable after a human reruns `repro.sh` and reads the patch
(the `CONTRIBUTING.md` policy makes the submitter answerable for it).
The disposable clone was deleted after verification;
the patch file and `repro.sh` in this doc reproduce it.

### Draft issue

~~~md
Title: Frozen install re-imports packages on every run when a slot holds a link to a skipped optional dependency

Labels: bug, area: pnpm 12

### pnpm version

12.5.1 (Linux x64). The code is unchanged on main at 7c1b501.

### Steps to reproduce

```sh
mkdir repro && cd repro
printf 'packages:\n  - a\nmodulesCacheMaxAge: 0\n' > pnpm-workspace.yaml
mkdir a && printf '{ "name": "a", "private": true }\n' > a/package.json
printf '{ "name": "repro", "private": true, "dependencies": { "typescript": "7.0.2" } }\n' > package.json
pnpm install
# Leave behind a link to an optional platform package that is skipped on this host
# (our real workspace had 49 of these; which install wrote them is unknown):
ln -s ../../../@typescript+typescript-aix-ppc64@7.0.2/node_modules/@typescript/typescript-aix-ppc64 \
  node_modules/.pnpm/typescript@7.0.2/node_modules/@typescript/typescript-aix-ppc64
pnpm install --frozen-lockfile   # Packages: +1
pnpm install --frozen-lockfile   # Packages: +1 again, on every run
```

`modulesCacheMaxAge: 0` matters: it makes every install prune, which declines the frozen early return
(`install/prepare_modules_state/up_to_date.rs:80-81`) and sends every warm slot through the probe.
Without it the stale link is never probed.

### Expected

The second frozen install is a no-op: either the stale link is removed during the first re-import,
or the probe tolerates it the way the regular-dependency probe does.

### Actual

Every run re-imports the slot and prints `Packages: +1`; the link survives.
In our real workspace this re-imports six packages (`typescript`, `rolldown`, `oxlint`, `oxlint-tsgolint`,
and two others with platform bindings) on every `pnpm install --frozen-lockfile`.

### Cause

`optional_child_matches` (crates/deps-restorer/src/create_virtual_store/snapshot_plan/children.rs:151-167)
reports a mismatch when a link exists for an optional child that should not exist.
Re-materialization does not remove it: `create_symlink_layout` skips skipped targets
(create_symlink_layout.rs:69-71), and `CreateVirtualDirBySnapshot::run` only removes optional children when
`include_optional` is false (create_virtual_dir_by_snapshot.rs:129-131).
The regular-dependency probe in children.rs:11-14 documents this exact hazard
("re-importing would not remove them, so requiring absence would re-materialize the slot on every install").

### Suggested fix

In `CreateVirtualDirBySnapshot::run`, after linking, unlink every optional child the symlink layout did not
link (all of them when optional dependencies are excluded, otherwise those whose resolved target is in
`skipped`). Patch attached. With it, the reproduction above re-imports once, removes the stale link,
and the next two frozen installs import nothing; the host-platform optional link
(`@typescript/typescript-linux-x64`) stays in place.

Written by an agent (Claude Code, claude-opus-5-5).
~~~
