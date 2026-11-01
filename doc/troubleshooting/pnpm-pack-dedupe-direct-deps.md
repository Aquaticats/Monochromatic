# pnpm pack cannot resolve deduplicated workspace dependencies

## Symptom

Packing `@monochromatic-dev/git-policy-cli` with pnpm 11.10.0 failed:

```text
ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL
Cannot resolve workspace protocol of dependency "@monochromatic-dev/config-typescript"
because this dependency is not installed. Try running "pnpm install".
```

A normal filtered install and a forced filtered install both left that direct development dependency linked only at the
workspace root.
`package/git-policy/cli/node_modules/@monochromatic-dev` did not contain `config-typescript`.

Using `npm pack` instead created a tarball,
but its manifest retained `catalog:` dependency specifiers.
A disposable npm consumer then failed with:

```text
EUNSUPPORTEDPROTOCOL
Unsupported URL Type "catalog:": catalog:
```

## Root cause

This workspace sets `dedupeDirectDeps: true` in `pnpm-workspace.yaml:151`.
The setting can deduplicate a direct workspace dependency to the root installation,
while pnpm's pack-time `workspace:` rewrite still expects the dependency under the package being packed.

Open pnpm issue [#9566][pnpm-9566] reports the same
`ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL` during pack or publish when `dedupeDirectDeps` is enabled.
Its reproducer states that disabling direct-dependency deduplication and forcing installation restores publishing.

Pnpm's workspace documentation says `pnpm pack` replaces `workspace:` dependencies with package versions
([workspace publishing][workspace-publishing]).
Its catalog documentation likewise says `pnpm pack` removes `catalog:` and writes ordinary version ranges
([catalog publishing][catalog-publishing]).
`npm pack` has no corresponding knowledge of the pnpm workspace catalog,
so it is not a valid replacement for packaging this repository's source manifest directly.

## Verification

The following sequence was tested against the cli-git package:

```text
pnpm install --force --config.dedupe-direct-deps=false --filter @monochromatic-dev/git-policy-cli
pnpm pack --pack-destination dist/pack
```

After the first command,
`package/git-policy/cli/node_modules/@monochromatic-dev/config-typescript` existed.
The second command produced a tarball whose manifest contained ordinary registry versions instead of `catalog:` or
`workspace:` protocols.

A disposable project installed that tarball with npm,
imported the authoring API,
bundled a self-contained config through tsdown,
resolved the packaged shadow `git` bin before `/usr/bin/git`,
and forwarded `git --version` successfully.

## Verified workaround

`package/git-policy/cli/mise.toml` makes `pack:npm` perform a forced package-filtered install with
`dedupe-direct-deps=false` immediately before `pnpm pack`.
The override is command-local;
it does not weaken workspace-wide deduplication.

Tradeoffs:

- the pack task relinks the filtered dependency graph before every tarball;
- `--force` is required by the upstream issue's workaround so existing deduplicated links are replaced;
- the package remains unpublished until deferred issue #358 is explicitly resumed.

## pnpm 12.3.4 ignores the command-line override

Found 2026-09-15 while verifying `package/config/pnpr/src/publish-missing-versions.ts`.
On pnpm 12.3.4,
 `--config.dedupe-direct-deps=false` no longer changes the install layout,
 so the `pack:npm` workaround in the top-level "Verified workaround" section
 leaves `pnpm pack` failing with the same error.
`.github/workflows/npm-release.yml` moved to the environment variable in `c92279465`.
These places still pass the ignored flag and need the same change:

- `package/git-policy/cli/mise.toml:46`,
   the `pack:npm` task;
- `doc/runbook/publish-npm-package-first-time.md:93`;
- `doc/planning/private-npm-registry.md:295`,
   which says packing reuses the `npm-release.yml` flag.

### Root cause

pnpm 12 is the Rust CLI (formerly pacquet).
Every `--config.<key>=<value>` token goes through an allow-list that drops unported keys without a message,
 and `dedupe-direct-deps` is not on it.
The environment variable takes a separate path that covers every setting in the schema.
Cited source is tag `v12.3.4`,
 commit `666c35e95c17f1a36414adc28bc25a73a8f1f67f`,
 in a depth-1 clone.

The CLI strips the tokens from argv before clap parses it:

```rust
// pnpm/crates/cli/src/lib.rs:96
    let (config_overrides, argv) = ConfigOverrides::extract(argv_with_alias);
```

`classify` accepts any dotted key.
It rejects a token only when `setting_takes` says the value does not fit,
 and `setting_takes` answers `true` for every key outside `BARE_SETTING_FLAGS`:

```rust
// pnpm/crates/cli/src/config_overrides.rs:829
    if let Some(rest) = arg.strip_prefix("--config.") {
        let Some((key, value)) = rest.split_once('=') else {
            return ConfigToken::Malformed;
        };
```

```rust
// pnpm/crates/cli/src/config_overrides.rs:980
fn setting_takes(key: &str, value: &str) -> bool {
    match named_bare_setting_flag(key) {
        Some((_, SettingArity::Boolean)) => parse_bool(value).is_some(),
        // ...
        None => true,
    }
}
```

`extract` then removes the token from argv and hands it to `set`:

```rust
// pnpm/crates/cli/src/config_overrides.rs:221
                ConfigToken::WellFormed { key, value } => overrides.set(key, value),
```

`set` matches a fixed list of keys,
 and everything else reaches the empty fallback arm:

```rust
// pnpm/crates/cli/src/config_overrides.rs:240
    fn set(&mut self, key: &str, value: &str) {
        match key {
            "allow-unused-patches" => self.allow_unused_patches = parse_bool(value),
            // ...
            "virtual-store-only" => self.virtual_store_only = parse_bool(value),
            _ => {}
        }
```

The `_ => {}` arm is line 302.
`rg dedupe pnpm/crates/cli/src/config_overrides.rs` prints nothing:
 `ConfigOverrides` has no `dedupe_direct_deps` field,
 so `ConfigOverrides::apply` (called at `pnpm/crates/cli/src/cli_args/dispatch.rs:145`)
 has nothing to copy onto `Config`.
The type's own doc comment states the tradeoff:

```rust
// pnpm/crates/cli/src/config_overrides.rs:93
/// Unknown keys are accepted silently: pnpm exposes a long tail of config
/// keys, and erroring on an unrecognized one would break the moment pnpm
/// adds a new key that pacquet hasn't ported yet. Dropping one is only
/// harmless when pnpm parsed the token first and delegated, leaving the
/// pacquet leg to fall back to the yaml value. When the binary runs
/// standalone there is no other leg, so a setting that changes what gets
/// installed has to be ported here.
```

The mise-installed pnpm 12.3.4 is that standalone binary
 (`~/.local/share/mise/installs/pnpm/12.3.4/pnpm`),
 so the flag is consumed and lost.

The environment variable works because `pnpm/crates/config/src/lib.rs:3717`
 calls `WorkspaceSettings::from_pnpm_config_env`,
 which reads the key directly:

```rust
// pnpm/crates/config/src/env_overlay.rs:225
        json_field!(dedupe_direct_deps, "DEDUPE_DIRECT_DEPS");
```

The module header explains both the precedence and why `npm_config_dedupe_direct_deps` does nothing:

```rust
// pnpm/crates/config/src/env_overlay.rs:4
//! Reads `pnpm_config_<key>` (or its `PNPM_CONFIG_<KEY>` uppercase form)
//! for every key in the schema and applies it to the config *after*
//! `pnpm-workspace.yaml`. That ordering means env vars override yaml.
//!
//! Pacquet does NOT read `npm_config_*` / `NPM_CONFIG_*` env vars (with
//! the exception of `NPM_CONFIG_WORKSPACE_DIR`, which has its own narrow
//! handler in [`crate::Config::current`]).
```

pnpm 11.10.0 honored the flag (the top-level "Verification" section),
 so this is a pnpm 12 regression.
pnpm.io still documents the form on its CLI page
 ([pnpm CLI][pnpm-cli], `pnpm/pnpm.io` `docs/pnpm-cli.md:29` and `:42` at `3ee793826`),
 and the v12.0.0 release notes do not mention removing it.
pnpm/pnpm `main` at `8f20a3fd69748b1f6eb5c6a7c54d5917456a2013` (after 12.4.1)
 still has no `dedupe` match in `pnpm/crates/cli/src/config_overrides.rs`.

### Verification

A disposable worktree at `00bd2867b` deleted every `node_modules` directory before each case,
 ran `pnpm install --frozen-lockfile` with the listed change,
 and checked for `package/or-throw/core/node_modules/@monochromatic-dev/config-typescript`
 (script `dedupe-env-probe.ts` in that session's scratchpad):

```text
default: install exit 0, config-typescript linked in or-throw: false
cli --config.dedupe-direct-deps=false: install exit 0, config-typescript linked in or-throw: false
env pnpm_config_dedupe_direct_deps=false: install exit 0, config-typescript linked in or-throw: true
env npm_config_dedupe_direct_deps=false: install exit 0, config-typescript linked in or-throw: false
```

With the environment variable set for the whole run,
 `pnpm pack` succeeded for `module-or-throw` and `oxlint-plugin-tsdoc`,
 and both published to a local pnpr and installed in a disposable consumer.
Adding `--force` to the command-line form did not link the dependency either.

A layout installed with the variable does not survive a later plain `pnpm install`:
 a publish run whose build tasks invoked pnpm without the variable restored the root-only link,
 and pack failed again.

#### Other `--config.` keys

Any key missing from `ConfigOverrides::set` is dropped the same way.
`config-flag-probe.ts` (that session's scratchpad) recreated a single-package project depending on `is-number@7.0.0`
 for each case,
 ran the pnpm 12.3.4 binary's `install` with the listed change,
 and checked the result:

```text
default: exit 0, node_modules/.pnpm true, node_modules/is-number true, pnpm-lock.yaml true
--config.lockfile=false (ported key, positive control): exit 0, node_modules/.pnpm true, node_modules/is-number true, pnpm-lock.yaml false
--config.symlink=false (unported key): exit 0, node_modules/.pnpm true, node_modules/is-number true, pnpm-lock.yaml true
env pnpm_config_symlink=false: exit 0, node_modules/.pnpm true, node_modules/is-number false, pnpm-lock.yaml true
--config.dedupe-direct-deps=false (unported key, no observable effect in a single project): exit 0, node_modules/.pnpm true, node_modules/is-number true, pnpm-lock.yaml true
```

- `lockfile` is on the allow-list (`"lockfile" => self.lockfile = parse_bool(value),` at `config_overrides.rs:264`),
   and the flag suppresses `pnpm-lock.yaml`,
   so the probe can show a flag taking effect.
- `symlink` is not on it:
   the flag leaves the direct link in place,
   while `pnpm_config_symlink=false` (`json_field!(symlink, "SYMLINK");` at `env_overlay.rs:170`) removes it.
- The single-project `dedupe-direct-deps` row shows nothing by design;
   the workspace probe in "Verification" is the evidence for that key.

`--config.node-linker=hoisted` was tried first as the control and discarded:
 neither the flag nor `pnpm_config_node_linker=hoisted` changed whether `node_modules/.pnpm` existed,
 so that check could not show a difference.
`node-linker` is on the allow-list,
 so it says nothing about dropped keys.

The bare flag is not a way around the drop:
 `pnpm install --dedupe-direct-deps=false` on 12.3.4 exits with clap's
 `error: unexpected argument '--dedupe-direct-deps' found`,
 because the key is also missing from `BARE_SETTING_FLAGS`.

### Verified workaround

Set `pnpm_config_dedupe_direct_deps=false` in the environment of every pnpm command in the packing job,
 as `.github/workflows/pnpr-publish.yml` does at job level.

Tradeoffs:

- the variable reaches every pnpm command in the job,
   including unrelated installs,
   which is acceptable in a disposable CI checkout but would change a developer's layout if exported locally;
- the variable outranks `pnpm-workspace.yaml` for every command it reaches
   (`env_overlay.rs:4` to `:6`),
   so a later yaml change to `dedupeDirectDeps` has no effect in that job.

## What does not work

### Pass `--config.dedupe-direct-deps=false` on pnpm 12.3.4

See "pnpm 12.3.4 ignores the command-line override":
 the flag installs the deduplicated layout unchanged,
 with or without `--force`,
 because `ConfigOverrides::set` drops the key.

### Pass `--dedupe-direct-deps=false` or `npm_config_dedupe_direct_deps=false` on pnpm 12.3.4

The bare flag fails argument parsing with `error: unexpected argument '--dedupe-direct-deps' found`.
The `npm_config_` variable is not read at all (`env_overlay.rs:8`),
 so the workspace probe keeps the root-only link.

### Repeat pnpm install with repository defaults

Both ordinary and forced installs preserve the deduplicated root-only workspace dependency,
so pack still cannot rewrite its `workspace:` specifier.

### Use npm pack directly

Npm creates an archive but does not expand pnpm catalogs.
The resulting tarball is not installable by a standalone npm consumer.

### Hand-edit the packed manifest

Rejected because it bypasses the package manager's workspace and catalog transformations,
creates version drift,
and violates the repository rule against hand-maintained generated dependency state.

## Upstream filing decision

### `pnpm pack` with `dedupeDirectDeps` enabled

Do not open a duplicate.
Pnpm issue [#9566][pnpm-9566] is open,
labeled `type: bug`,
and matches the observed error plus workaround.
The repository has a verified mitigation
 (the `pnpm_config_dedupe_direct_deps` variable on pnpm 12);
no additional upstream report is needed unless pnpm maintainers request a new pnpm 11 reproduction.
Its workaround says to turn off `dedupe-direct-deps` and force an install,
 without naming a spelling,
 so the pnpm 12 flag drop is a separate bug:
 see "pnpm 12 drops `--config.dedupe-direct-deps`".

### pnpm 12 drops `--config.dedupe-direct-deps`

Decision:
 all six constraints hold,
 so the new-issue draft in "Draft issue" is fileable once a human completes "Before filing".
Nothing was filed or commented upstream from this session.

`.out-of-scope/` was checked:
 it has no entry for pnpm or for command-line configuration overrides.

#### Duplicate search

`gh search issues --repo pnpm/pnpm --include-prs`,
 open and closed,
 for `dedupe-direct-deps`,
 `dedupeDirectDeps`,
 `"--config." ignored`,
 and `config override unknown key pacquet` (no results).
No issue or pull request title in those results reports the `--config.dedupe-direct-deps` drop.
Closest hits
 (#14450 and #9566 read in full,
 #12009 and #14101 searched for `dedupe` and `--config.`,
 the rest by title and state):

- [#14450][pnpm-14450] (closed):
   `pnpm deploy --legacy` ignored `--config.allow-unused-patches` on pnpm 12.
   Same allow-list,
   different key;
   its triage comment names `ConfigOverrides::set` as the cause,
   and [#14459][pnpm-14459] fixed it by porting that one key.
- [#13986][pnpm-13986] (closed):
   `--config.ignore-scripts=true` ignored by `pnpm pack` on pnpm 12,
   closed by #13993,
   which ported that one key.
- [#12009][pnpm-12009] and [#14101][pnpm-14101] (closed):
   umbrella trackers for settings missing from the Rust CLI;
   neither thread mentions `dedupe-direct-deps` or the `--config.` form for it.
- [#9566][pnpm-9566] (open):
   the pack error itself,
   covered in the "`pnpm pack` with `dedupeDirectDeps` enabled" subsection.
- #13775 (closed):
   a `dedupeDirectDeps` link-removal regression,
   unrelated to command-line parsing.

#### Six constraints

1.   **Upstream's fault:**
      yes.
      This is behavior,
       not wording:
       the binary removes the token from argv and discards it.
      pnpm 11.10.0 honored the flag (top-level "Verification"),
       pnpm.io documents `--config.` as the way to pass any setting,
       and the `ConfigOverrides` doc comment (`config_overrides.rs:93`)
       says a setting that changes what gets installed "has to be ported here".
      The environment variable for the same key works,
       so no architectural restriction is involved.
2.   **Upstream can fix it:**
      yes;
      "Prototype fix" adds the key to the existing allow-list.
3.   **Supported use case:**
      yes.
      `dedupeDirectDeps` is documented (pnpm.io `docs/settings/other.md:291`),
       the Rust CLI has a `dedupe_direct_deps` integration suite (`pnpm/crates/cli/tests/suite/dedupe_direct_deps.rs`),
       and the `BARE_SETTING_FLAGS` doc comment (`pnpm/crates/cli/src/config_overrides/tokens.rs:151` on `main`)
       says pnpm makes every setting spellable on the command line.
4.   **Contribution welcome:**
      yes.
      `CONTRIBUTING.md` "AI-assisted contributions" (lines 318 to 338 on `main`) welcomes agent-made contributions,
       requires a checked duplicate search,
       understanding,
       passing tests,
       no unrelated changes,
       and a footer naming the agent and model.
      No ban was found in `CONTRIBUTING.md` or `.github/ISSUE_TEMPLATE/`;
       the draft carries the footer.
5.   **Likely to fix:**
      yes.
      Maintainers accepted and fixed the same class per key for `allow-unused-patches` (#14459,
       merged 2026-09-02) and `ignore-scripts` (#13993).
      No won't-fix or non-goal was found.
6.   **Prototype:**
      yes;
      see "Prototype fix".

#### Prototype fix

Patch:
 [`pnpm-pack-dedupe-direct-deps.patch`](pnpm-pack-dedupe-direct-deps.patch),
 against pnpm/pnpm `main` at `8f20a3fd69748b1f6eb5c6a7c54d5917456a2013`
 (`git apply --check` passes on a clean worktree of that commit).
It mirrors #14459:

- `pnpm/crates/cli/src/config_overrides.rs`:
   a `dedupe_direct_deps: Option<bool>` field,
   and a `"dedupe-direct-deps"` arm in `set_layout_option`
   (`set_boolean_install_option` already spans lines 251 to 291,
    and #14792 enforces a 40-line limit on production functions);
- `pnpm/crates/cli/src/config_overrides/apply.rs`:
   `dedupe_direct_deps => "dedupeDirectDeps"` in `apply_install_policy_overrides`,
   which also records it as explicitly set;
- `pnpm/crates/cli/src/config_overrides/tokens.rs`:
   `("dedupe-direct-deps", SettingArity::Boolean)` in `BARE_SETTING_FLAGS`,
   so `--dedupe-direct-deps` and `--no-dedupe-direct-deps` work too;
- `extract_applies_dedupe_direct_deps_override` in `config_overrides/tests.rs`,
   covering the bare,
   negated,
   and `--config.` spellings;
- `dedupe_direct_deps_cli_override_keeps_per_project_symlinks` in `tests/suite/dedupe_direct_deps.rs`:
   yaml `dedupeDirectDeps: true` plus `--config.dedupe-direct-deps=false install` must keep the sibling's link;
- a `.changeset/dedupe-direct-deps-cli-override.md` patch note for `pacquet`.

Build bounds:
 `podman run --rm --memory=2g --cpus=2` with `docker.io/library/rust:1.97-bookworm`,
 only a fresh `mktemp` prototype directory mounted (clone,
 a throwaway `CARGO_HOME` holding a copy of the `upstream-prototype.hkarDR0l` crate cache,
 which has no credentials file,
 and target directory),
 no credentials in the environment,
 `RUSTUP_TOOLCHAIN=1.97.1` (the image's toolchain;
 upstream pins `1.97.0`),
 `CARGO_PROFILE_DEV_DEBUG=0`,
 and `CARGO_INCREMENTAL=0`.
The clone's `.cargo/config.toml` block
 that points crates.io at the pnpm-managed `.pnpm/crates` vendor directory
 was deleted locally,
 because a `pnpm install` the container did not run populates that directory;
 the patch does not include that edit.
With `--jobs 2`,
 rustc compiling `pnpm-cli` was killed with `signal: 9, SIGKILL` at the 2 GiB limit,
 so the recorded runs use `--jobs 1`.

Test command,
 run once with only the two tests added and again with the fix applied:

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${PROTOTYPE}:/proto:Z" \
  --env CARGO_HOME=/proto/cargo-home --env CARGO_TARGET_DIR=/proto/target \
  --env CARGO_PROFILE_DEV_DEBUG=0 --env CARGO_INCREMENTAL=0 --env RUSTUP_TOOLCHAIN=1.97.1 \
  --workdir /proto/pnpm docker.io/library/rust:1.97-bookworm \
  cargo test --package pnpm-cli --lib --test suite --no-fail-fast --jobs 1 -- dedupe_direct_deps config_overrides
```

The unpatched run used only the `dedupe_direct_deps` filter.
Unpatched `main` (tests only):

```text
test config_overrides::tests::extract_applies_dedupe_direct_deps_override ... FAILED
assertion failed: `(left == right)`: --dedupe-direct-deps
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1487 filtered out; finished in 0.04s
test dedupe_direct_deps::dedupe_direct_deps_cli_override_keeps_per_project_symlinks ... FAILED
sibling_dep="/tmp/pacquet-test-DnapWJ/workspace/packages/dup/node_modules/@pnpm.e2e/hello-world-js-bin" linked=false
--config.dedupe-direct-deps=false should keep the sibling direct-dep symlink
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 2119 filtered out; finished in 4.24s
```

Patched `main`:

```text
test config_overrides::tests::extract_applies_dedupe_direct_deps_override ... ok
test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 1426 filtered out; finished in 0.02s
test dedupe_direct_deps::dedupe_direct_deps_cli_override_keeps_per_project_symlinks ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 2119 filtered out; finished in 1.82s
```

The 62 unit tests include every `config_overrides` test,
 among them `no_bare_setting_flag_shadows_a_global_option`,
 which proves the new bare flag collides with no global clap option.
The ten existing `dedupe_direct_deps` integration tests pass before and after.

User-boundary check:
 the draft's reproduction script ran in `docker.io/library/node:24-slim` (same 2 GiB,
 2 CPU bounds) with each binary mounted as `pnpm`.
pnpm 12.3.4 ended with
 `ls: cannot access 'packages/dup/node_modules/is-number': No such file or directory`;
the patched debug binary (reporting `12.4.1`) listed `LICENSE`,
 `README.md`,
 `index.js`,
 and `package.json` from the linked package.
`dedupe-minimal-repro.ts` (that session's scratchpad) gave the same split with the environment variable as a control:

```text
pnpm 12.3.4
pnpm install: exit 0, packages/dup/node_modules/is-number false
pnpm --config.dedupe-direct-deps=false install: exit 0, packages/dup/node_modules/is-number false
pnpm_config_dedupe_direct_deps=false pnpm install: exit 0, packages/dup/node_modules/is-number true
```

```text
pnpm 12.4.1
pnpm install: exit 0, packages/dup/node_modules/is-number false
pnpm --config.dedupe-direct-deps=false install: exit 0, packages/dup/node_modules/is-number true
pnpm_config_dedupe_direct_deps=false pnpm install: exit 0, packages/dup/node_modules/is-number true
```

Not run:
 upstream's full test suite,
 `just lint`,
 `just dylint`,
 and formatting
 (the image has no `rustfmt`,
 and upstream's `rustfmt.toml` sets `chain_complexity_layout`);
 `CONTRIBUTING.md` asks for the full suite before a pull request.

#### Before filing

A human must personally:

- rerun "Code to reproduce the issue" from the draft on pnpm 12.3.4 or later;
- open the cited lines in `pnpm/crates/cli/src/config_overrides.rs` on current `main`
   and confirm the key is still missing;
- apply the patch and run the test command;
- confirm the `pnpm_config_dedupe_direct_deps=false` workaround;
- remove any clause of the draft's footer describing a check they did not do.

#### Draft issue

~~~md
Title: pnpm 12 ignores `--config.dedupe-direct-deps=<value>` without a message
Template: Regression Report
Labels: regression

### Last pnpm version that worked

11.10.0

### pnpm version

12.3.4 (also reproduced on `main` at 8f20a3fd69748b1f6eb5c6a7c54d5917456a2013)

### Code to reproduce the issue

```sh
mkdir -p repro/packages/dup && cd repro
echo '{"name":"ws-root","private":true,"dependencies":{"is-number":"7.0.0"}}' > package.json
echo '{"name":"dup","version":"1.0.0","dependencies":{"is-number":"7.0.0"}}' > packages/dup/package.json
printf "packages:\n  - 'packages/*'\ndedupeDirectDeps: true\n" > pnpm-workspace.yaml
pnpm --config.dedupe-direct-deps=false install
ls packages/dup/node_modules/is-number
```

### Expected behavior

`--config.dedupe-direct-deps=false` overrides `dedupeDirectDeps: true` for this run,
so `packages/dup/node_modules/is-number` is linked.
That is what pnpm 11 does,
and what `pnpm_config_dedupe_direct_deps=false pnpm install` does on pnpm 12.

### Actual behavior

The install succeeds with the deduplicated layout:

```text
ls: cannot access 'packages/dup/node_modules/is-number': No such file or directory
```

Nothing reports that the setting was ignored.
`pnpm install --dedupe-direct-deps=false` fails with `error: unexpected argument '--dedupe-direct-deps' found`.

### Additional information

Impact: the #9566 workaround for `ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL` during `pnpm pack`
(install with `dedupe-direct-deps=false` before packing) silently stops working on pnpm 12 when written as `--config.`.

Cause (v12.3.4 line numbers):
`ConfigOverrides::extract` strips every well-formed `--config.<key>=<value>` token from argv
(`pnpm/crates/cli/src/lib.rs:96`, `pnpm/crates/cli/src/config_overrides.rs:221`),
`setting_takes` accepts any key outside `BARE_SETTING_FLAGS` (`config_overrides.rs:988`, `None => true`),
and `ConfigOverrides::set` has no `dedupe-direct-deps` arm,
so the token falls into `_ => {}` (`config_overrides.rs:302`).
The environment variable works because `pnpm/crates/config/src/env_overlay.rs:225` reads `DEDUPE_DIRECT_DEPS`.
This is the same class as #14450 (fixed per key in #14459) and #13986.

Suggested fix, verified against `main` at 8f20a3fd6:

```diff
--- a/pnpm/crates/cli/src/config_overrides.rs
+++ b/pnpm/crates/cli/src/config_overrides.rs
@@ -73,2 +73,3 @@ pub struct ConfigOverrides {
     dangerously_allow_all_builds: Option<bool>,
+    dedupe_direct_deps: Option<bool>,
     deploy_all_files: Option<bool>,
@@ -389,2 +390,5 @@ impl ConfigOverrides {
         match key {
+            "dedupe-direct-deps" => {
+                self.dedupe_direct_deps = parse_bool(value);
+            }
             "global-dir" => {
--- a/pnpm/crates/cli/src/config_overrides/apply.rs
+++ b/pnpm/crates/cli/src/config_overrides/apply.rs
@@ -194,2 +194,3 @@ impl ConfigOverrides {
             dangerously_allow_all_builds => "dangerouslyAllowAllBuilds",
+            dedupe_direct_deps => "dedupeDirectDeps",
             engine_strict => "engineStrict",
--- a/pnpm/crates/cli/src/config_overrides/tokens.rs
+++ b/pnpm/crates/cli/src/config_overrides/tokens.rs
@@ -158,3 +158,3 @@ pub(super) enum SettingArity {
 /// on every command line and so must not appear here at all.
-pub(super) const BARE_SETTING_FLAGS: [(&str, SettingArity); 39] = [
+pub(super) const BARE_SETTING_FLAGS: [(&str, SettingArity); 40] = [
     ("allow-unused-patches", SettingArity::Boolean),
@@ -162,2 +162,3 @@ pub(super) const BARE_SETTING_FLAGS: [(&str, SettingArity); 39] = [
     ("dangerously-allow-all-builds", SettingArity::Boolean),
+    ("dedupe-direct-deps", SettingArity::Boolean),
     ("engine-strict", SettingArity::Boolean),
```

With a unit test over the bare, negated, and `--config.` spellings and an integration test in
`pnpm/crates/cli/tests/suite/dedupe_direct_deps.rs`, both fail before this change and pass after it;
the existing `config_overrides` unit tests and `dedupe_direct_deps` integration tests still pass.
I can open this as a pull request.

A broader question for maintainers: `pnpm-workspace.yaml` now reports unrecognized settings
(`ERR_PNPM_UNRECOGNIZED_WORKSPACE_SETTINGS`), while an unported `--config.` key is still dropped in silence,
which also affects keys such as `symlink`.

### Node.js version

v26.8.2

### Operating System

Linux

---

Written by an agent (Claude Code, claude-opus-5).
A human reran the reproduction, checked the cited source lines, and ran the prototype's tests before filing.
~~~

[pnpm-9566]: https://github.com/pnpm/pnpm/issues/9566
[pnpm-12009]: https://github.com/pnpm/pnpm/issues/12009
[pnpm-13986]: https://github.com/pnpm/pnpm/issues/13986
[pnpm-14101]: https://github.com/pnpm/pnpm/issues/14101
[pnpm-14450]: https://github.com/pnpm/pnpm/issues/14450
[pnpm-14459]: https://github.com/pnpm/pnpm/pull/14459
[pnpm-cli]: https://pnpm.io/pnpm-cli
[workspace-publishing]: https://pnpm.io/workspaces#publishing-workspace-packages
[catalog-publishing]: https://pnpm.io/catalogs#publishing
