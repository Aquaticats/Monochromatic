# pnpm 12.16.1: "Already up to date" with stale undeclared workspace links and dedupe-hidden declared ones

Investigated on 2026-10-29 after a root `pnpm i` printed `Already up to date` while
`package/file-enforcer/core/node_modules/@monochromatic-dev/` lacked a declared link
and still held links into the removed `package/module/` category.

Short answer: two unrelated observations, neither a pnpm defect.

- The missing `@monochromatic-dev/config-typescript` link is intended `dedupeDirectDeps` behavior.
  The root manifest declares the same package, so resolution walks up to the root link.
  A scan of all 225 importers found no declared link that is absent for any other reason.
- The stale `module-*` links are extraneous entries.
  pnpm's install only unlinks a direct dependency when the previous install recorded it in the current lockfile
  (`node_modules/.pnpm/lock.yaml`).
  Here the current lockfile no longer lists them, so no install path (fast, frozen, or forced) can see them.
  Only `pnpm prune` scans the directory.
  The scan found 181 such entries across 78 importers, 166 of them dangling.

All source citations are pnpm at tag `v12.16.1`
(commit `c7b46aedef7b81c07816cb551187449a3dd81792`, release commit title "chore(release): pacquet 12.16.1").
pnpm 12 is the Rust rewrite under `pnpm/crates/` of that repository.
The installed binary is `.cache/toolchain-bin/pnpm` in this repository (`pnpm --version` prints `12.16.1`).
Related earlier write-up for 12.5.1:
[pnpm-stale-node-modules-detection](pnpm-stale-node-modules-detection.md)
(it already records the `dedupeDirectDeps` absence and the prefix-scoped `Already up to date` summary).

## Symptom

User command at the repository root on 2026-10-29:

```text
Scope: all 225 workspace projects
Already up to date
Done in 2.5s using pnpm v12.16.1
```

State right after, for the importer `package/file-enforcer/core`:

- `package.json` declares `@monochromatic-dev/config-typescript` as `workspace:*` in `devDependencies`
  (the package exists at `package/config/typescript`),
  but `node_modules/@monochromatic-dev/` has no `config-typescript`.
- `node_modules/@monochromatic-dev/` holds seven `module-*` links
  (`module-caught-value`, `module-fs-path`, `module-logger`, `module-matrix`, `module-rust-catalog`, `module-test`,
  `module-toml-edit`), each pointing into `package/module/`, for example
  `module-test -> ../../../../module/test`.
  Commit `592d78b143` (2026-10-29 06:03 -0400) removed that category.
  No current `package.json` declares them.

Timeline from file times (all 2026-10-29 -0400 unless noted):

- `module-logger`, `module-matrix`, `module-test`, `module-toml-edit` links: Jun 22.
  `module-caught-value`: Jul 14.
  `module-fs-path`: Sep 9.
- `module-rust-catalog` link: 02:25.
- `package/file-enforcer/core/package.json` edited 05:26; the replacement links
  (`caught-value-core`, `fs-path-core`, `logger-core`, `test-harness-core`, and others) were created 05:28.
- `pnpm-lock.yaml` written 05:37; commit `592d78b143` at 06:03.
- `node_modules/.modules.yaml` and `node_modules/.pnpm/lock.yaml` written 07:22:26
  (`prunedAt: Thu, 29 Oct 2026 11:22:26 GMT`),
  so a full install ran after the commit and still kept every `module-*` link.
- `node_modules/.pnpm-workspace-state-v1.json` rewritten 08:45:34, consistent with the user's `pnpm i`.

## Root cause

### Question 1: the wanted lockfile lists `config-typescript`

The importer block `package/file-enforcer/core:` in `pnpm-lock.yaml` (61 lines, read in full) holds, among others:

```yaml
# pnpm-lock.yaml
    devDependencies:
      '@monochromatic-dev/config-typescript':
        specifier: workspace:*
        version: link:../../config/typescript
```

It lists no `module-*` entry.
The same importer block in `node_modules/.pnpm/lock.yaml` (the current lockfile) is byte-identical
(`diff` of the two extracted blocks printed nothing), so pnpm's own record says the link was installed or deduped.

### The absent link is `dedupeDirectDeps`, not drift

`pnpm-workspace.yaml:227` sets `dedupeDirectDeps: true`.
The root importer in `pnpm-lock.yaml` records the same dependency:

```yaml
# pnpm-lock.yaml, importer "."
      '@monochromatic-dev/config-typescript':
        specifier: workspace:*
        version: link:package/config/typescript
```

Both resolve to the directory `package/config/typescript`.
The link pass skips an importer's entry when the root provides the same target
(`pnpm/crates/deps-restorer/src/symlink_direct_dependencies.rs:486` to `497`):

```rust
fn dedupe_resolved_entries<'a>(
    entries: Vec<ResolvedEntry<'a>>,
    dedupe_against: Option<&BTreeMap<String, PathBuf>>,
) -> Vec<ResolvedEntry<'a>> {
    if let Some(root_targets) = dedupe_against {
        entries
            .into_iter()
            .filter(|entry| {
                root_targets
                    .get(&entry.name_str)
                    .is_none_or(|root_target| root_target != &entry.target)
```

Node resolution from the importer reaches the root link.
`require.resolve('@monochromatic-dev/config-typescript/package.json', { paths: ['<repo>/package/file-enforcer/core'] })`
fails only with `ERR_PACKAGE_PATH_NOT_EXPORTED` (the package has an `exports` map without that subpath),
and the error text names `/var/home/user/Monochromatic/node_modules/@monochromatic-dev/config-typescript/package.json`,
which is the root link.

A read-only scan of the lockfile against the disk (`extraneous.ts`, "Harness: the repository scan")
found `missing=0` over 225 importers: every declared link is present unless the root provides the same target.

### Question 2: why pnpm decided "Already up to date"

The user's output has no `Lockfile is up to date, resolution step is skipped` line.
In the fixture that line appears on every run that went through the install pipeline,
and never on the plain repeat install that took the pre-setup fast path (see "Verification").
So the message most likely came from the fast path in
`pnpm/crates/cli/src/cli_args/install/fast_path.rs`.
The same decision function (`check_freshness`) also runs inside the pipeline
(`pnpm/crates/package-manager/src/install/run/fast_path.rs:60` to `80`, `repeat_install_verdict`),
so the verdict does not depend on which of the two printed the line.

The fast path emits the message at `fast_path.rs:31` to `35`, and its decision is logged at `fast_path.rs:65` to `81`:

```rust
Ok(()) => {
    tracing::debug!(
        target: "pacquet::install",
        "repeat-install fast path taken before the install setup; nothing changed since the previous install",
    );
    true
}
```

The decision is `check_optimistic_repeat_install_in`
(`pnpm/crates/package-manager/src/optimistic_repeat_install.rs:171` to `207`), which builds a gate policy and calls
`check_freshness`:

```rust
let policy = GatePolicy {
    gate: Gate::Install,
    ignored_settings: &[],
    check_local_file_deps: true,
    check_dangling_direct_links: true,
    missing_wanted_lockfile: MissingWantedLockfile::Restore,
    newer_patch: NewerPatch::Refuse,
    recorded_groups: RecordedGroups::Current,
};
match check_freshness(check, &state, moved, &policy, scope) {
    Ok(()) => Decision::UpToDate,
```

`check_freshness` (`optimistic_repeat_install/freshness.rs:148` to `192`) compares recorded inputs, not the tree:

- settings, config dependencies, catalogs, the workspace project list, and a modules directory for each project
  with dependencies (`freshness.rs:222` to `267`);
- the SHA-256 prefix of the lockfile and a digest of each manifest against
  `validation` in `node_modules/.pnpm-workspace-state-v1.json`
  (`freshness/drift.rs:190` to `224`, `ProjectStatus::Proven` when `record.manifest == digest` and the lockfile digest
  matches);
- whether a direct dependency link that a manifest declares dangles (`freshness.rs:263` to `265`).

This repository's state file passes every one of those:

- The recorded `validation.lockfile` is `4d360381d9b6a180447e906585385cbf`.
  `sha256sum pnpm-lock.yaml | cut --characters=1-32` prints the same value, and the recorded size and mtime
  (`1793266646590436900` ns, 05:37:26) equal the file's.
- `validation.manifests` holds 225 entries and `projects` has 225 entries, so the project list matches.
- `.npmrc` does not exist, so `pnpm-workspace.yaml` is the only settings file of this repository.

The dangling-link probe is the only check that looks at links, and it only looks at declared names
(`optimistic_repeat_install/modules_dirs.rs:164` to `184`):

```rust
/// ... A missing entry is not a broken link: skipped optional and
/// excluded dependencies have none, and a healthy entry costs one `stat`.
pub(super) fn direct_dependency_link_dangling(check: &OptimisticRepeatInstallCheck<'_>) -> bool {
    let groups = included_groups(check.layout.included);
    check.project_manifests
        .par_iter()
        .any(|(root_dir, manifest)| {
            project_modules_dirs(check, root_dir, manifest)
                .iter()
                .any(|modules_dir| {
                    manifest
                        .dependencies(groups.iter().copied())
                        .any(|(alias, _)| is_dangling_link(&modules_dir.join(alias)))
```

The iteration is `manifest.dependencies(...)`,
so `module-test` (declared nowhere) is never stat'd even though it dangles,
and the absent `config-typescript` is "not a broken link" by the documented rule.
A project with dependencies and no modules directory is also tolerated
when `dedupeDirectDeps` would link nothing into it
(`modules_dirs.rs:117` to `120` and `144` to `159`).

The recorded state was last written at 08:45:34, so a `pnpm i` that finds every digest equal returns `Ok(())` and
prints the line.
This is the designed behavior: the fast path trusts recorded digests and never walks `node_modules`.

The CLI pre-setup path does not run for `--frozen-lockfile` or `--force`
(`fast_path.rs:129` and `132`: `"the lockfile is frozen"`, `"--force"`).
That is why those fixture runs printed `Lockfile is up to date, resolution step is skipped`,
and why they are no help here.
They still reach the same `check_freshness` inside the pipeline.

### Question 3: the stale links are a second, separate cause

Even the full install path would leave the `module-*` links.
`PruneStaleModules::run` (`pnpm/crates/deps-restorer/src/prune_stale_modules.rs:83` to `111`)
iterates the importers of the current lockfile,
and `unlink_stale_direct_deps` (`prune_stale_modules.rs:235` to `254`) removes only the direct dependencies
that the current lockfile recorded and the wanted lockfile no longer does:

```rust
for (alias, current_spec, group) in direct_deps_of(current_snapshot, &RECORDED_GROUPS) {
    let still_wanted = is_still_wanted(alias, current_spec, wanted_specs);
    // ...
    if still_wanted && !deduped_by_root {
        continue;
    }
    let alias_name = alias.to_string();
    remove_direct_dep_link(modules_dir, &alias_name)?;
```

So the removal set is (current lockfile) minus (wanted lockfile).
The current lockfile for `package/file-enforcer/core` has no `module-*` entry, so the set is empty.
The caller falls back to the wanted lockfile when the current one is absent
(`pnpm/crates/deps-restorer/src/linking.rs:285`):

```rust
let current_lockfile = inputs.graph.current_lockfile.unwrap_or(inputs.graph.lockfile);
```

The directory scan exists only behind `prune_direct_dependencies`, which `pnpm prune` sets
(`pnpm/crates/package-manager/src/install/entry_points.rs:71` to `82`, `linking.rs:299` to `311`) and which runs
`PruneExtraneousDirectDeps` (`deps-restorer/src/prune_extraneous_deps.rs:35` to `91`):
it lists every non-dot entry of the importer's `node_modules` (`pnpm/crates/fs/src/read_modules_dir.rs:6` to `8`)
and removes the ones that are not wanted direct dependencies.
Upstream states this split in the commit that added the scan, pull request
[#17154](https://github.com/pnpm/pnpm/pull/17154) (merged 2026-10-20, merge commit
`473cccecefa50b2d966b76ae46c4f6357d720719`):
"`prune_stale_modules` only unlinked direct dependencies that were present in `current_lockfile` and removed from
`wanted_lockfile`, never scanning `node_modules` on disk for extraneous entries when `prune` ran".

### How the current lockfile lost the links without the links going

Not determined for this repository.
One mechanism reproduces the exact state: an install that runs while `node_modules/.pnpm/lock.yaml` is missing
(deleted, or replaced by a racing session) prunes nothing and writes a fresh current lockfile
(see the fixture run `current-lockfile-missing`).
Install paths that do not reproduce it (all checked in the fixture): `--lockfile-only` then install, `--filter` on the
other project, `--filter` on the owner, and `pnpm remove`.
Shared `node_modules` between concurrent sessions makes the race plausible, but nothing recorded here proves it.

### Question 3 answer: two problems, not tied

- Absent declared links: `dedupeDirectDeps`, working as documented.
- Present undeclared links: extraneous entries that no install-time path removes
  once the current lockfile stops recording them.
- The only connection is that both are invisible to the fast path.
  Fixing the second does not change the first, and the first never needs a fix.

## Verification

Environment: pnpm 12.16.1 (`.cache/toolchain-bin/pnpm`) on Fedora (Bazzite), Linux 7.2.9.
The fixture workspace lives under `${HOME}/temp/agent/pnpm-fixture`, outside any ancestor `node_modules`.
All fixture installs ran with `--offline --store-dir <fixture store>`
and `XDG_CONFIG_HOME` pointing at an empty directory, so no user config applied.
No `pnpm` command was run in this repository.

### Harness: the minimal reproduction

Three workspace projects `a`, `b`, `c`, root `devDependencies: { "@x/b": "workspace:*" }`,
`a` depends on `@x/b` and `@x/c`, `dedupeDirectDeps: true`:

```sh
# fixture root: pnpm-workspace.yaml lists a, b, c and sets dedupeDirectDeps: true
pnpm install --offline          # a/node_modules/@x holds only c: b is deduped against the root
# edit a/package.json to drop @x/c
rm node_modules/.pnpm/lock.yaml # the race: the current lockfile is gone
pnpm install --offline          # prints "Lockfile is up to date" and "Already up to date"; a/node_modules/@x/c stays
pnpm install --offline          # fast path, 22 ms, link stays
pnpm install --frozen-lockfile --offline   # 150 ms, link stays
pnpm install --force --offline             # 92 ms, link stays
pnpm prune --offline            # removes it
```

Condensed results of that run (timings are from the run, durations vary):

```text
(first install)   + @x/b <- b      a/node_modules/@x: c
(after the race)  Lockfile is up to date, resolution step is skipped / Already up to date / Done in 50ms     a: c
(repeat)          Scope: all 4 workspace projects / Already up to date / Done in 22ms
(frozen)          Lockfile is up to date, resolution step is skipped / Already up to date / Done in 150ms
(force)           Lockfile is up to date, resolution step is skipped / Already up to date / Done in 92ms       a: c
(prune)           Lockfile is up to date, resolution step is skipped / Already up to date / Done in 55ms
                  a/node_modules empty; node_modules/@x holds b
```

The regenerated current lockfile lists `@x/b` only for `a`, matching the wanted one, which is the repository's state.

### Catalog of removal behavior (planted links, no current-lockfile record)

Fixture state: `a` no longer declares `@x/c`; the install was run; then `a/node_modules/@x/c` (live) and
`a/node_modules/@x/gone` (dangling) were created by hand.
The commands are the ones in the minimal reproduction, each run on a fresh copy of that fixture.

- `pnpm install`: fast path, 24 ms, `c` and `gone` remain.
- `pnpm install --frozen-lockfile`: 99 ms, both remain.
- `pnpm install --force`: 520 ms, both remain.
- `pnpm install --force --frozen-lockfile`: 33 ms, both remain.
- `pnpm prune`: 51 ms, both removed together with the emptied `@x` scope directory.

### Catalog of how links get removed when the install does know them

- Control: with the current lockfile intact, dropping `@x/c` from `a` and running `pnpm install`
  removed `a/node_modules/@x` entirely (`ls` failed: no such directory).
  So the prune pass works when the current lockfile records the link.
- `--lockfile-only` then `pnpm install`: removed.
- `pnpm install --filter @x/a`: removed.
- `pnpm remove @x/c --filter @x/a`: removed.
- `pnpm install --filter @x/b` (a manifest edited, the owner not selected): `c` stayed, but the wanted and current
  lockfiles for `a` also still listed `c`, so a following `pnpm install` removed it.
- Current lockfile deleted, then edit and install: `c` stayed (the repository's state).

### Fast-path decision log

`TRACE=pacquet::install=debug` (the variable read by `pnpm/crates/diagnostics/src/local_tracing.rs:6`):

```text
stale state, plain install:
  DEBUG pacquet::install: repeat-install fast path taken before the install setup; nothing changed since the previous install
after editing a/package.json (version bump), positive control:
  DEBUG pacquet::install: repeat-install fast path not taken before the install setup; the install decides, reason: "workspace project list changed"
  DEBUG pacquet::install: repeat-install fast path skipped; running the full install, reason: "workspace project list changed"
```

The positive control proves the log can report a skipped path.
This run was not repeated in the repository (it would rewrite the shared state file), so the repository conclusion rests
on the digest evidence in "Question 2".

### Harness: the repository scan

`extraneous.ts` (read-only; Node with the `yaml` package from the repository's root `node_modules`)
reads `pnpm-lock.yaml`,
applies the `dedupeDirectDeps` skip (root and importer resolve the same normalized `link:` target),
skips dot entries as `read_modules_dir` does, and compares with each importer's `node_modules`.
It printed:

```text
importers=225 extraneous=181 missing=0
```

Breakdown from its output:

- 181 extraneous entries across 78 importers; the root importer has none.
- 166 dangling, of which 161 are `@monochromatic-dev/module-*`; 162 entries in total have a `module-` alias
  (`module-test` 59, `module-logger` 22, `module-caught-value` 18, `module-async-time` 11, `module-fs-path` 8,
  `module-or-throw` 6, `module-toml-edit` 5, `module-const` 4, `module-rust-catalog` 3).
- 15 not dangling: 4 stale `config-typescript` links (`config/oxlint`, `dom/core`, `git-policy/cli`,
  `pnpm-override/stub-throwing`; the root now provides it), `config-oxlint-shared` and `config-oxlint-test-support` in
  four `oxlint-plugin/*` importers, `agent-harnesses-shared-text-scan`, `agent-harnesses-shared-shell-command-analyzer`,
  and `module-llm-type`.
- No source file imports any of these aliases
  (`rg` over `package/**/*.ts`, `*.mjs`, `*.json` found only comments and a test fixture string).

Full source of the scan:

```ts
// Read-only: list entries under each importer's node_modules that the wanted lockfile does not want,
// approximating pnpm 12.16.1 prune_extraneous_deps.rs (dot entries skipped, dedupeDirectDeps against root).
import { createRequire } from 'node:module';
import { lstatSync, readdirSync, readFileSync, readlinkSync, statSync } from 'node:fs';
import { join, normalize } from 'node:path';

const repo = '/var/home/user/Monochromatic';
const { parse } = createRequire(join(repo, 'package.json'))('yaml') as { parse: (text: string) => any };
const lock = parse(readFileSync(join(repo, 'pnpm-lock.yaml'), 'utf8'));
const groups = ['dependencies', 'devDependencies', 'optionalDependencies'];

function directDeps(importer: any, id: string): Map<string, string> {
  const map = new Map<string, string>();
  for (const group of groups) {
    for (const [alias, spec] of Object.entries<any>(importer?.[group] ?? {})) map.set(alias, String(spec.version).startsWith('link:') ? 'link:' + normalize(join(id, String(spec.version).slice(5))) : String(spec.version));
  }
  return map;
}

function installedNames(modulesDir: string): string[] {
  let entries: string[];
  try {
    entries = readdirSync(modulesDir);
  } catch {
    return [];
  }
  const names: string[] = [];
  for (const entry of entries) {
    if (entry.startsWith('.')) continue;
    if (entry.startsWith('@') && !lstatSync(join(modulesDir, entry)).isSymbolicLink()) {
      for (const inner of readdirSync(join(modulesDir, entry))) {
        if (!inner.startsWith('.')) names.push(`${entry}/${inner}`);
      }
    } else {
      names.push(entry);
    }
  }
  return names;
}

const rootDeps = directDeps(lock.importers['.'], '.');
let totalExtraneous = 0;
let totalMissingLinks = 0;
const lines: string[] = [];
for (const [id, importer] of Object.entries<any>(lock.importers)) {
  const dir = id === '.' ? repo : join(repo, id);
  const wanted = new Set<string>();
  for (const [alias, version] of directDeps(importer, id)) {
    if (id !== '.' && rootDeps.get(alias) === version) continue;
    wanted.add(alias);
  }
  const installed = installedNames(join(dir, 'node_modules'));
  const extraneous = installed.filter((name) => !wanted.has(name));
  const missing = [...wanted].filter((name) => !installed.includes(name));
  for (const name of extraneous) {
    const path = join(dir, 'node_modules', name);
    let target = '';
    let dangling = false;
    try {
      target = lstatSync(path).isSymbolicLink() ? readlinkSync(path) : '(real dir)';
      statSync(path);
    } catch {
      dangling = true;
    }
    lines.push(`EXTRANEOUS ${id} ${name} -> ${target}${dangling ? ' [dangling]' : ''}`);
  }
  for (const name of missing) lines.push(`MISSING ${id} ${name}`);
  totalExtraneous += extraneous.length;
  totalMissingLinks += missing.length;
}
console.log(lines.join('\n'));
console.log(`importers=${Object.keys(lock.importers).length} extraneous=${totalExtraneous} missing=${totalMissingLinks}`);
```

The 166 dangling count equals `find package -mindepth 5 -maxdepth 5 -path
'package/*/*/node_modules/@monochromatic-dev/*' -xtype l -print | wc --lines`, run as a dry run.

## When a package gets node_modules/.bin

A package's own `package.json` cannot predict whether its `node_modules/.bin` exists.
pnpm creates that directory only when it links a dependency that exposes a `bin` into that package,
and with `dedupeDirectDeps: true` (`pnpm-workspace.yaml:227`)
a dependency the root also declares is linked only at the root,
so its bin lands in the root `node_modules/.bin`.
Whether a directory exists therefore depends on the root manifest,
each linked dependency's own `bin` field,
and leftovers from earlier installs.

Measured 2026-10-29 in the main checkout after `pnpm prune && pnpm i`,
with the scratch probe described below:

- 223 `package/*/*/package.json`,
  205 `package/*/*/node_modules`,
  55 `package/*/*/node_modules/.bin`.
- 208 packages have at least one direct dependency not linked in their own `node_modules`.
- No package has a locally linked bin-exposing dependency without a `.bin`.
- 18 of the 55 `.bin` directories are empty,
  in packages with no direct dependency linked locally,
  for example `package/config/pnpr`,
  `package/mcp/mvm`,
  and every `package/claude-code-plugin/*` package.

The probe walks each `package/*/*/package.json`,
checks `node_modules/<dep>/package.json` for every direct dependency,
reads the linked manifest's `bin` field,
and compares that with whether `node_modules/.bin` exists and what it holds.

Consequence for generated PATH lists such as the root `mise.toml` `_.path`:
derive them from the directories that exist,
and skip empty ones;
deriving them from `package.json` names directories no install creates
(decided in `doc/planning/file-enforcer-rust-rewrite.md`,
"Root mise.toml PATH").

## Verified workarounds

### Run `pnpm prune` once, with no other install running

```sh
# repository root; needs user approval because node_modules is shared with live sessions
pnpm prune
```

Fixture-verified: it is the only command that removed the planted links (and the now-empty `@x` scope directory).
What it would change in this repository, from the scan and source:

- Removes the 181 entries listed in "Harness: the repository scan" (166 dangling, 15 live but undeclared).
  It leaves dot entries (`.bin`, `.cache`, `.monochromatic`, `.pnpm_patches`) alone.
- Runs the full install pipeline under `prune_direct_dependencies`, so it rewrites `node_modules/.modules.yaml`,
  `node_modules/.pnpm/lock.yaml`, and `node_modules/.pnpm-workspace-state-v1.json`.
- Runs project lifecycle scripts: the fixture's root `postinstall` echoed once under `pnpm prune`.
  No `package.json` in this repository declares `preinstall`, `install`, `postinstall`, or `prepare`
  (`rg` over `package.json` files outside `node_modules`), so none would run here.
- Keeps dev dependencies and every declared link (fixture: root `node_modules/@x/b` stayed).

Tradeoffs: a concurrent install in another session can race the state files;
it also deletes the 15 live undeclared links, which is intended,
but anything that still resolved through them would break (no source import of them was found);
and it takes a full pipeline run, not the 2.5 s fast path.

### Delete only the dangling links

```sh
# repository root; dry run first by replacing -delete with -print
find package -mindepth 5 -maxdepth 5 -path 'package/*/*/node_modules/@monochromatic-dev/*' -xtype l -delete
```

Removes exactly the 166 dangling entries (dry-run count verified), touches no pnpm state, runs no scripts, and cannot
affect resolution because the targets do not exist.
Tradeoffs: leaves the 15 live undeclared links; leaves the empty `@monochromatic-dev` scope directories; a concurrent
install that recreates a link is unaffected because dangling links of undeclared names are never recreated.

### Prevent recurrence with `pruneDirectDependencies: true` (not recommended)

`config_blocks_fast_path` returns early when the setting is on
(`optimistic_repeat_install.rs:242` to `244`, returning the reason `"prune_direct_dependencies enabled"`),
so every `pnpm i` would run the full pipeline instead of the fast path.
This was read, not run.
Tradeoff: loses the fast repeat install for a problem that appears only when the current lockfile goes missing.

## What does not work

- Deleting `.pnpm-workspace-state-v1.json` only forces the full install path.
  The fixture shows the full path (`--frozen-lockfile`, `--force`) leaves undeclared links alone.
- `pnpm install --force` rebuilds the virtual store and re-imports packages; it does not run the extraneous scan.
- `modulesCacheMaxAge: 0` (set in `pnpm-workspace.yaml`) sweeps orphan virtual-store directories, not importer links.
- Reading the install summary as evidence: it printed `Already up to date` on a fresh fixture install that created a
  link (`+ @x/b <- b`) and on every repeat.
- Expecting the missing `config-typescript` to be relinked: `dedupeDirectDeps` skips it on every install by design,
  so a forced install would not create it either.
- The 12.5.1 doc's reading that `--frozen-lockfile` always disables the fast path is out of date for 12.16.1
  (named in "Question 2").
- `pnpm install --filter` as a leak source: ruled out in the fixture (see the removal catalog).

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was listed on 2026-10-29: `bun-install.md`, `cargo-workspace.md`, `claude-code-upstream-bugs.md`,
`codex-harness.md`, `jsr.md`, `lightningcss.md`, `low-impact-typescript-formatting.md`, `module-es-monolith.md`,
`pi-gpt55-long-context.md`, `terminal-title-fork-parity-tests.md`, `typescript-project-references.md`.
None covers pnpm, so tracking is in scope.

Duplicate search (`gh search issues` and `gh search prs`, repo `pnpm/pnpm`, open and closed, 14 terms each:
`stale symlink`, `extraneous`, `dangling symlink`, `removed dependency symlink remains`,
`node_modules link not removed`,
`dedupeDirectDeps`, `dedupe-direct-deps`, `Already up to date`, `optimistic repeat install`, `repeat install stale`,
`current lockfile`, `pruneDirectDependencies`, `prune-direct-dependencies`, `workspace link remains after removing`).
Closest hits, read:

- [#13775](https://github.com/pnpm/pnpm/issues/13775) (closed): `dedupeDirectDeps` leaves a link that became redundant;
  fixed by [#13780](https://github.com/pnpm/pnpm/pull/13780).
  Same family (a link the install does not retire), different trigger, and fixed through the same current-versus-wanted
  diff that this write-up shows is blind to links not in the current lockfile.
- [#10081](https://github.com/pnpm/pnpm/issues/10081) (closed 2026-09-19): renaming or moving workspace packages leaves
  dangling links and `ENOENT`.
  Same user-visible class on 10.18.2.
- [#17154](https://github.com/pnpm/pnpm/pull/17154) (merged 2026-10-20): adds the directory scan to `pnpm prune`,
  with the limitation of the install-time pass stated in its description.
- [#15569](https://github.com/pnpm/pnpm/pull/15569) (merged): "relink a dangling direct dependency on repeat install",
  the origin of the declared-name probe.
- No issue or pull request covers "an extraneous link survives because the current lockfile no longer records it".

Constraints:

1. Upstream's fault: no.
   The install pass is documented upstream as a current-minus-wanted diff, and the extraneous scan is the designated
   remedy (#17154).
   The absent `config-typescript` link is documented `dedupeDirectDeps` behavior.
   The remaining gap is wording: `Already up to date` is printed when stale links exist.
2. Upstream can fix it: yes (the install could run the scan, at the cost of the fast path).
3. Supported use case: yes, workspaces with `dedupeDirectDeps` are supported.
4. Contributions welcome: yes.
   `CONTRIBUTING.md` "AI-assisted contributions" (line 325) welcomes agent-made contributions and requires a footer
   naming the agent and the model, which the draft carries.
   No issue template or ban against agent-assisted reports was found.
5. Likely to fix: unknown.
   Nearby bugs were fixed within weeks (#13775, #14891),
   and maintainers just shipped #17154 as the answer to this class, which leans toward "use `pnpm prune`".
6. Prototype: not attempted.
   Constraint 1 is a no, and the skill's auto-prototype step applies only when constraints 1 to 5 hold or sorta-hold.

Verdict: do not file.
The draft is kept as an auditable record in case upstream signal changes
(for example, a maintainer asking for an install-time extraneous check).

### Draft issue (do not file as-is)

~~~md
Title: Repeat install and the install pipeline never remove a workspace link once the current lockfile stops recording it

Labels: bug, area: pnpm 12

### pnpm version

12.16.1 (Linux x64), tag v12.16.1 (c7b46aedef7b81c07816cb551187449a3dd81792).

### Steps to reproduce

```sh
mkdir repro && cd repro
mkdir a b c
printf 'packages:\n  - a\n  - b\n  - c\ndedupeDirectDeps: true\n' > pnpm-workspace.yaml
printf '{"name":"root","private":true,"devDependencies":{"@x/b":"workspace:*"}}\n' > package.json
printf '{"name":"@x/a","version":"1.0.0","devDependencies":{"@x/b":"workspace:*","@x/c":"workspace:*"}}\n' > a/package.json
printf '{"name":"@x/b","version":"1.0.0"}\n' > b/package.json
printf '{"name":"@x/c","version":"1.0.0"}\n' > c/package.json
pnpm install
printf '{"name":"@x/a","version":"1.0.0","devDependencies":{"@x/b":"workspace:*"}}\n' > a/package.json
rm node_modules/.pnpm/lock.yaml   # a racing install or a cleanup removed the current lockfile
pnpm install                      # Already up to date
ls a/node_modules/@x              # c is still linked
pnpm install --frozen-lockfile    # still linked
pnpm install --force              # still linked
pnpm prune                        # removed
```

### Expected

An install that makes `a/node_modules/@x/c` undeclared removes it, or reports that extraneous entries exist.

### Actual

`PruneStaleModules::run` (crates/deps-restorer/src/prune_stale_modules.rs:83-111) diffs the current lockfile's importers
against the wanted lockfile, so a link absent from the current lockfile is never unlinked
(linking.rs:285 substitutes the wanted lockfile when the current one is missing).
The repeat-install gate probes only declared aliases
(crates/package-manager/src/optimistic_repeat_install/modules_dirs.rs:171-184), so it reports `Already up to date`.
Only `pnpm prune` removes the entry (prune_extraneous_deps.rs).
In our workspace this left 181 undeclared entries across 78 importers, 166 of them dangling.

### Suggested fix

Either document that the install pass does not retire links missing from the current lockfile and that `pnpm prune`
does, or let the repeat-install gate stat the dangling targets of `read_modules_dir` entries of each importer so that
dangling undeclared links force the full pipeline (which would still need the scan to remove them).

Written by an agent (Claude Code, claude-sonnet-5-5).
~~~
