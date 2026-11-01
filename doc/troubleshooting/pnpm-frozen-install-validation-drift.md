# pnpm 12.14.0 `pnpm install --frozen-lockfile` rejects a lockfile whose recorded state is not committed (`ERR_PNPM_PACKAGE_MANAGER_NO_IMPORTER`, `ERR_PNPM_LOCKFILE_CONFIG_MISMATCH`)

## Symptom

A CI install (`mise exec node pnpm -- pnpm install --frozen-lockfile`, the pack step of
`.github/workflows/npm-release.yml` and `.github/workflows/npm-publish.yml`) stops before installing anything
with one of two diagnostics.

Variant 1, a workspace manifest changed without its lockfile regen (`ERR_PNPM_PACKAGE_MANAGER_NO_IMPORTER`):

```text
// ~/temp/agent/repro-noimporter.txt
  × installing dependencies
  ╰─▶ Cannot install with "frozen-lockfile" because pnpm-lock.yaml has no
      `importers["package/dsh-plugin/neuralwatt"]` entry. Regenerate the
      lockfile with `pnpm install --lockfile-only`.
```

Variant 2, a lockfile regen committed while the config it encoded stayed uncommitted
(`ERR_PNPM_LOCKFILE_CONFIG_MISMATCH`):

```text
// ~/temp/agent/repro-checksum.txt
  × installing dependencies
  ╰─▶ Cannot proceed with the frozen installation. The current
      "packageExtensionsChecksum" configuration doesn't match the value found
      in the lockfile
  help: Update your lockfile using "pnpm install --no-frozen-lockfile"
```

Surface patterns that trigger them:

- Committing an added, removed, or renamed workspace package, or changed package dependencies,
  without `pnpm install --lockfile-only`: variant 1, naming the missing importer id.
- Committing `pnpm-lock.yaml` after a regen that ran while `pnpm-workspace.yaml` (or the root `package.json`)
  held uncommitted `packageExtensions` edits: variant 2.
- Both were observed 2026-10-22 in this repository's release workflows on runs 39747926280,
  39748789348, 39749506245 (variant 1 then variant 2, sequentially).

## Root cause

`--frozen-lockfile` validates the lockfile against the manifests and config in the checkout and refuses to
install on any drift; pnpm's own docs state the rule
("In a CI environment, installation fails if a lockfile is present but needs an update",
https://pnpm.io/cli/install#--frozen-lockfile).
The two diagnostics are two validators on that path.

### Step 1: each workspace project's manifest must have an importer entry

`check_lockfile_freshness` walks the workspace projects and calls `check_importer_satisfies`
(pnpm/pnpm `pnpm/crates/package-manager/src/install/lockfile_freshness/manifest.rs:52-59`,
source read at commit `f0982d4f11`, Rust rewrite tree):

```rust
// pnpm/crates/package-manager/src/install/lockfile_freshness/manifest.rs:52-59
pub(crate) fn check_importer_satisfies(
    check: &ImporterSatisfactionCheck<'_>,
) -> Result<Vec<(String, String)>, FreshnessCheckError> {
    let importer = check.lockfile.importers
        .get(check.importer_id)
        .ok_or_else(|| FreshnessCheckError::NoImporter {
            importer_id: check.importer_id.to_string(),
        })?;
```

The error is built at `pnpm/crates/package-manager/src/install/errors.rs:345-351`:

```rust
// pnpm/crates/package-manager/src/install/errors.rs:345-351
#[display(
    r#"Cannot install with "frozen-lockfile" because pnpm-lock.yaml has no `importers["{importer_id}"]` entry. Regenerate the lockfile with `pnpm install --lockfile-only`."#
)]
#[diagnostic(code(ERR_PNPM_PACKAGE_MANAGER_NO_IMPORTER))]
NoImporter { importer_id: String },
```

### Step 2: the recorded `packageExtensionsChecksum` must match the current config

The lockfile records a top-level `packageExtensionsChecksum` key (an earlier reading in this investigation
placed it under `settings:`; that was wrong. Evidence: `pnpm11/lockfile/types/src/index.ts:19-27` puts it in
`LockfileBase` beside `settings`, and `pnpm11/lockfile/fs/src/sortLockfileKeys.ts:31-41` emits it top-level).

The value is a sha256 over the `packageExtensions` map:
`pnpm/crates/package-manager/src/install_with_fresh_lockfile/lockfile_build.rs:416-422` and
`pnpm/crates/graph-hasher/src/object_hasher.rs:39-49`:

```rust
// pnpm/crates/package-manager/src/install_with_fresh_lockfile/lockfile_build.rs:416-422
pub(crate) fn compute_package_extensions_checksum(config: &Config) -> Option<String> {
    let extensions = config.package_extensions
        .as_ref()
        .filter(|extensions| !extensions.is_empty())?;
    let value = serde_json::to_value(extensions).ok()?;
    pnpm_graph_hasher::hash_object_nullable_with_prefix(&value)
}
```

The frozen path recomputes that value from the checkout's config
(`pnpm/crates/package-manager/src/install/lockfile_freshness/settings.rs:124-125`) and compares it with the
recorded one (`pnpm/crates/lockfile/src/freshness.rs:424-428`):

```rust
// pnpm/crates/lockfile/src/freshness.rs:424-428
if lockfile.package_extensions_checksum.as_deref() != check.package_extensions_checksum {
    return Err(StalenessReason::PackageExtensionsChecksumChanged {
        lockfile: lockfile.package_extensions_checksum.clone(),
        config: check.package_extensions_checksum.map(str::to_string),
    });
}
```

Mismatch becomes the error at `pnpm/crates/package-manager/src/install/errors.rs:302-309`:

```rust
// pnpm/crates/package-manager/src/install/errors.rs:302-309
#[display(
    r#"Cannot proceed with the frozen installation. The current "{setting}" configuration doesn't match the value found in the lockfile"#
)]
#[diagnostic(
    code(ERR_PNPM_LOCKFILE_CONFIG_MISMATCH),
    help(r#"Update your lockfile using "pnpm install --no-frozen-lockfile""#)
)]
LockfileConfigMismatch { setting: &'static str },
```

The explicit `--frozen-lockfile` path runs these checks at
`pnpm/crates/package-manager/src/install/run/dispatch.rs:333`; without the flag the same drift falls through
to a fresh resolve (`dispatch.rs:404-410`), which is why only CI-style runs see these errors.

### Why the two variants surface one after the other

Both validators run over the same drift pair (manifests plus config versus what the lockfile recorded).
When a regen and the state it encodes land in separate commits, or only one of them lands,
each validator reports its own half: first the missing importer, then the checksum.
Fixing one half alone reproduces the other error on the next run (verified: run 39749506245 failed with
variant 2 right after variant 1 was fixed by commit `774d1dbd67`).

pnpm 11 (TypeScript tree in the same source checkout) keeps the checksum comparison
(`pnpm11/lockfile/settings-checker/src/getOutdatedLockfileSetting.ts:85-86`,
`pnpm11/installing/deps-installer/src/install/frozenInstallErrors.ts:3-9`) but reports a missing importer as
`ERR_PNPM_OUTDATED_LOCKFILE` with `Failure reason: no importer`
(`pnpm11/lockfile/verification/src/satisfiesPackageManifest.ts:39`,
`pnpm11/installing/deps-installer/src/install/verifyFrozenLockfile.ts:85-101`).

## Verification

Version under test: pnpm 12.14.0 (`mise exec node pnpm -- pnpm --version`, the `mise.toml` pin in this
repository). Source read from `pnpm/pnpm` commit `f0982d4f11`
(`gh repo clone pnpm/pnpm "${HOME}/temp/agent/pnpm-src-20261022" -- --depth 1`), whose Rust rewrite tree
reports 12.15.0 and whose `pnpm11/` TypeScript tree reports 11.28.7. The clone was read-only scratch; nothing
in it was modified.

Harness: two throwaway worktrees of this repository, one per drift half. Both reproduce as written.

```sh
// variant 1: manifest committed without its lockfile importer
git worktree add "${HOME}/temp/agent/repro-noimporter" c05a869682
cd -- "${HOME}/temp/agent/repro-noimporter" && mise exec node pnpm -- pnpm install --frozen-lockfile
```

```sh
// variant 2: lockfile regen committed, the packageExtensions it encoded not
git worktree add "${HOME}/temp/agent/repro-checksum" 774d1dbd67
cd -- "${HOME}/temp/agent/repro-checksum" && mise exec node pnpm -- pnpm install --frozen-lockfile
```

Catalog, fails:

- `c05a869682` (workspace package `package/dsh-plugin/neuralwatt` committed,
  `pnpm-lock.yaml` without its `importers` entry): variant 1, importer id quoted above.
- `774d1dbd67` (lockfile regen committed; the `packageExtensions` entry it hashed left uncommitted):
  variant 2, `packageExtensionsChecksum` quoted above.

Catalog, passes cleanly:

- `451c369859` (regen and its config committed together): `mise exec node pnpm -- pnpm install --frozen-lockfile`
  exits 0 (`Done in 622ms using pnpm v12.14.0`, recorded in `~/temp/agent/pnpm-frozen-check.txt`).
- `774d1dbd67` with the uncommitted `packageExtensions` present in the working tree (the state where the regen
  was produced): passes locally and fails on a clean checkout, showing the drift is about what is committed.

## Verified workarounds

### Commit the regen and the state it encodes together

```sh
// the repair that unblocked release 0.4.5 (commits 774d1dbd67 + 451c369859)
git add pnpm-workspace.yaml pnpm-lock.yaml
git commit -m "chore(*): ..."
```

Verified: the frozen install then exits 0 locally and the CI pack job of run 39749801676 succeeds.
Tradeoffs: none for semantics; the committer must know which uncommitted manifest or config the regen encoded.
When concurrent work owns one half, committing it on its behalf splits provenance across authors.

### Regenerate against exactly the committed state

```sh
// clean-tree regen, e.g. in a throwaway worktree at HEAD
git worktree add "${HOME}/temp/agent/regen" HEAD
cd -- "${HOME}/temp/agent/regen" && mise exec node pnpm -- pnpm install --lockfile-only
```

Verified: the produced `pnpm-lock.yaml` passes the frozen install on a clean checkout of the same commit.
Tradeoffs: it drops the influence of any uncommitted local config, so it must be re-run and recommitted when
that config lands later.

### Let CI update the lockfile (`pnpm install --no-frozen-lockfile`, the error's own hint)

Verified mechanically: the install proceeds and rewrites `pnpm-lock.yaml` in the runner's workspace.
Tradeoffs: the rewrite is discarded with the runner, CI stops detecting drift, and the repository can stay
inconsistent indefinitely; not suitable as a standing release-pipeline setting.

## What does not work

- Hand-editing `pnpm-lock.yaml` fields (importer entries, the checksum): the checksum is a sha256 over the
  parsed config, so hand-written values only accidentally match. The repository rule `LFW` in `AGENTS.md`
  (never hand-edit lockfiles; regenerate via the owning package manager) covers this.
- Fixing only one half of the drift pair: run 39749506245 still failed with variant 2 after the importer
  half was fixed.
- Re-running the failed workflow unchanged: same checkout, same diagnostics (runs 39747926280,
  39748789348).
- Regenerating locally while the config is still uncommitted, then committing only the lockfile: that is
  exactly the variant 2 trap, reproduced by `774d1dbd67`.

## Upstream filing artifact

### Upstream filing decision

1.  Is it really upstream's fault? No. Both validations enforce documented, intended behavior
    (pnpm.io `--frozen-lockfile` docs: "fails to install if the lockfile is out of sync with the manifest").
    The incidents came from this repository committing a drift pair in halves. Behavior, not wording.
2.  Can upstream fix it? Not applicable; there is nothing broken to fix. The guards do exactly what their
    messages say.
3.  Are they supporting this use case? Yes. Frozen CI installs of a committed lockfile are the documented CI
    mode, and both remedies the diagnostics name (`--lockfile-only`, `--no-frozen-lockfile`) are documented
    options.
4.  Would the repo welcome our contribution? Policy check run: `CONTRIBUTING.md` exists at pnpm/pnpm,
    issues are enabled (184 open), and no ban on external or assisted filings was found in the searched
    surfaces. Not applicable anyway, since nothing is being filed.
5.  Will they likely fix it? Not applicable; no request to make. No won't-fix signal is needed for behavior
    we are not asking to change.
6.  Have we prototyped a minimal fix? No, and none is required: constraint 1 fails, so the auto-prototype
    trigger (constraints 1-5 holding) does not fire.

### Duplicate search

- `ERR_PNPM_LOCKFILE_CONFIG_MISMATCH` in pnpm/pnpm: no hits.
- `packageExtensionsChecksum` in pnpm/pnpm: five closed issues, none about the frozen-install mismatch
  (they cover `verifyDepsBeforeRun` rewrites, `pnprServer` ignoring extensions, lockfile merging,
  pnpmfile dep removal, and settings-change resolution).
- `ERR_PNPM_PACKAGE_MANAGER_NO_IMPORTER` across GitHub: the closest is pnpm/pnpm#15875,
  "`--frozen-lockfile` fails after adding a workspace package with no dependencies" (closed).
  That is a different trigger (a regenerated lockfile still lacking a dependency-less package's importer).
  Here regeneration recorded the importer correctly; the error only appeared because the regen was not
  committed.

### Filing artifact

Nothing to add upstream: no new issue, no comment on pnpm/pnpm#15875. Default policy (do not file) holds.

Out-of-scope check: `.out-of-scope/` holds no exemption for pnpm lockfile or install behavior;
`.out-of-scope/bun-install.md` exempts Bun install bugs only, on the grounds that this workspace installs
with pnpm, which is the tool in scope here.
