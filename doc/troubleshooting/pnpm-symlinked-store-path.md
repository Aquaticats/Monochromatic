# pnpm 12.16.1: symlinked shared store breaks DSH Advisor's proper-lockfile import

## Symptom

DSH `0.2.1-rc.5` cannot activate `@monochromatic-dev/dsh-plugin-advisor`:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'proper-lockfile'
imported from .../@earendil-works/pi-coding-agent/dist/core/settings-manager.js
```

The installed Pi package is `@earendil-works/pi-coding-agent@1.5.0`.
Directly importing its `dist/core/settings-manager.js` with Node `26.13.0`
reproduced the same diagnostic without DSH.

## Root cause

The workspace intentionally replaces `proper-lockfile` with
`package/pnpm-override/shim-proper-lockfile` through a `link:` override.
The [shim decision](../decision/proper-lockfile-removal.md) still applies.
The shim file exists; its installed dependency link is broken.

The shared store setting expands `$HOME` to `/home/user`,
while the physical directory is `/var/home/user`.
Reading the installed dependency link and resolving its relative contents
from its physical parent produced
`/var/var/home/user/Monochromatic/package/pnpm-override/shim-proper-lockfile`.
That path does not exist.

The source explanation is in
[`pnpm/crates/fs/src/symlink_dir.rs:118`][link-source],
at release commit `c7b46aedef7b81c07816cb551187449a3dd81792`:

```rust
// pnpm/crates/fs/src/symlink_dir.rs:118
fn relative_target_for(original: &Path, link: &Path) -> PathBuf {
    let parent = link.parent().unwrap_or_else(|| Path::new(""));
    crate::relative_path(parent, original)
}
```

The parent is taken from the supplied path spelling.
The filesystem follows the parent symlink before interpreting the stored relative target,
so the number of parent traversals can be wrong.
The source text was confirmed in `agent-browser` through its raw-source page.
The same calculation remains in upstream commit `b4a2abfcb7cd17692ea90cea15ce7457fa7b4260`.

The corresponding reuse check also joins stored relative contents to the supplied parent spelling:

```rust
// pnpm/crates/fs/src/symlink_dir/reuse.rs:43
link.parent()
    .unwrap_or_else(|| Path::new(""))
    .join(existing_link_string)
```

That can mark a broken link as reusable.
Its [source selection][reuse-source] was also confirmed in `agent-browser`.

[link-source]: https://raw.githubusercontent.com/pnpm/pnpm/c7b46aedef7b81c07816cb551187449a3dd81792/pnpm/crates/fs/src/symlink_dir.rs#:~:text=crate%3A%3Arelative_path(parent%2C%20original)
[reuse-source]: https://raw.githubusercontent.com/pnpm/pnpm/c7b46aedef7b81c07816cb551187449a3dd81792/pnpm/crates/fs/src/symlink_dir/reuse.rs#:~:text=existing_symlink_up_to_date

## Verification

Verification recorded at 2026-11-05T09:54:59-05:00.

A disposable project used `is-odd@3.0.1`,
a local `is-number` shim,
`virtualStoreType: global`,
and a store reached through a symlink to a deeper directory.
Both installs used pnpm `12.16.1` through a fixture Mise task.

- Symlinked store spelling:
  installation succeeded,
  the transitive link pointed into a nonexistent directory,
  and the consumer failed with `Cannot find module 'is-number'`.
- Canonical spelling of the same store with `install --force`:
  the transitive link resolved to the shim,
  and `require('is-odd')(3)` printed `true`.

The fixture directory was
`~/temp/agent/advisor-store-path-krq2xQ`.
The runner was `~/temp/agent/advisor-store-path-probe.ts`.
These scratch paths are evidence locations,
not durable test dependencies.

### Public reproduction and CI evidence at 2026-11-05T10:37:09-05:00

The initial completion claim was premature:
the required standalone reproduction repository and inspected CI evidence were missing.
The user explicitly requested a public repository.
The durable artifact is now
[`Aquaticat/pnpm-symlinked-store-repro`][public-repro].

It creates an ESM consumer and a local `proper-lockfile` shim from synthetic fixtures,
packs the consumer locally,
and installs offline into disposable stores.
It needs no DSH installation or Monochromatic checkout.
Node and pnpm downloads are pinned and SHA-256 verified.

The corrected harness was run twice at commit `8c84d48f8c36ba3729a3e2f1fc982cf950f7e268`:

- [Run 42055610522][repro-run].
- [Run 42056201476][repro-repeat].

Both runs' job steps and diagnostic logs were read.
The affected jobs fail only at `Assert the symlinked store resolves its dependency`;
provisioning,
the real-store control,
the aliased-store install,
and the canonical-path workaround all pass.
The failing log contains both `ERR_MODULE_NOT_FOUND` for `proper-lockfile`
and the emitted diagnostic `targetExists: false`.

Measured affected cases:

- Linux with pnpm `12.16.1`,
  `12.18.0`,
  and `11.28.11`.
- macOS and Windows with pnpm `12.18.0`.
- Node `26.13.0` in every case.

The `10.34.9` case imports successfully,
but its logged consumer path is under `project/node_modules/.pnpm`.
It does not exercise the shared virtual store with this configuration.
That result is not a claim that v10 has a corrected shared-store implementation.

The earlier latest-release claim was wrong:
the release-list API showed `12.18.0` and `11.28.11`,
although the latest-release endpoint had returned `12.16.1`.
The CI matrix uses the release-list evidence.

The initial Windows harness failed with GNU tar's
`Cannot connect to D: resolve failed` before pnpm ran.
Selecting Windows' native `System32/tar.exe` repaired the harness.
The linked evidence runs pass extraction and reproduce the actual import failure.

[public-repro]: https://github.com/Aquaticat/pnpm-symlinked-store-repro
[repro-run]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42055610522
[repro-repeat]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42056201476

## Verified local workaround

`.config/mise/conf.d/20-env.toml` supplies `PNPM_CONFIG_STORE_DIR`
using Mise's `canonicalize` filter on the existing home directory.
The rest of the shared store path remains unchanged.
The environment override takes precedence over the workspace YAML spelling.
Canonicalizing home allows a fresh installation where the store does not yet exist.

This retains the approved shared virtual store and the existing lockfile shim.
It requires the repository's Mise environment;
a caller outside that environment must supply a canonical `--store-dir` itself.
Additional symlinks inside `.local/share/pnpm/store` are not covered by canonicalizing home alone.

Commit `bfeeffe65a` records the automatic environment workaround.
The existing dependency tree was regenerated with its owner:

```sh
# Repository root
mise run prepare:pnpm:install -- --force --frozen-lockfile --ignore-scripts
```

Pnpm completed successfully with no lockfile changes.
The original `settings-manager.js` import then printed `SettingsManager: function`.
No store symlink was edited by hand.

The consumer probe is runnable from this repository:

```sh
# Repository root
node --input-type=module -e 'const m = await import("./package/dsh-plugin/advisor/node_modules/@earendil-works/pi-coding-agent/dist/core/settings-manager.js"); console.log("SettingsManager:", typeof m.SettingsManager);'
```

A disposable `DSH_HOME` copied the installed web profile and plugin files,
without the real session or credential stores.
The DSH web app started successfully.
Its Plugins page listed Advisor with `Enable Advisor` checked;
`agent-browser eval` read `advisorEnabled: "true"` from that switch.
The browser error check was empty.
This verifies import and activation;
no model inference request was made.

## Permanent fix status

Automatic canonicalization is a workaround,
not the permanent package-manager fix.
The user's follow-up explicitly challenged stopping at a real-path workaround.
Pnpm should interpret both new and existing relative links from their physical parent directory.
Its reuse check must also recognize and replace links made with the old calculation.

A disposable upstream clone at `b4a2abfcb7cd17692ea90cea15ce7457fa7b4260`
contains a source prototype and a filesystem regression test.
Before the source change,
`unix_links_under_symlinked_parent_remain_resolvable` failed with `NotFound`
when resolving the newly created link.
The [saved source patch](pnpm-symlinked-store-path.patch) fixes creation and reuse together.
It resolves existing parent directories through pnpm's `realpath_missing` helper,
while preserving the target symlink itself so later retargeting remains visible.

Post-change Nextest output:

```text
Starting 19 tests across 8 binaries (211 tests skipped)
PASS pnpm-fs symlink_dir::tests::unix_links_under_symlinked_parent_remain_resolvable
Summary: 19 tests run: 19 passed, 211 skipped
```

The new regression exercises direct creation,
creation with missing parent directories,
target-alias retargeting,
reuse of a correct link,
and replacement of a planted broken link.
The existing symlink tests cover stale occupants,
relative paths,
scoped aliases,
and retry boundaries.

The test ran through the upstream sanitizing runner in a disposable container:

```sh
# Disposable upstream checkout at b4a2abfcb7cd17692ea90cea15ce7457fa7b4260
node pnpm/scripts/run-rust-tests.mjs --package pnpm-fs --filter-expr 'test(symlink_dir::)'
```

The outer Mise task ran Podman with `--cpus 2 --memory 8g --pids-limit 512 --blkio-weight 10`.
The container received only the disposable checkout,
with no host credential directories or writable mount of this repository.
`TMPDIR` pointed inside that checkout.
The initial `pnpm run test:rust` attempt triggered its automatic dependency install;
the completed test used the same Node runner directly to avoid that unrelated installation.

Limits:
the full pnpm CLI was not rebuilt,
the source patch's tests ran on Linux,
and the added filesystem lookup cost was not benchmarked.
The stock CLI reproduction now covers Windows,
macOS,
and v11 as recorded in the public-reproduction section.
The saved source patch targets the Rust implementation;
it does not claim to fix every affected pnpm implementation.

The [public source-patch workflow][prototype-run] also passed.
Its inspected logs show the unchanged-source regression failing with `NotFound`,
then all 19 symlink tests passing after the production patch is applied.
It downloads a checksum-pinned Nextest,
checks out the exact upstream commit,
and runs the same upstream sanitizing test runner.

[prototype-run]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42055609601

The prototype is not installed as the host's pnpm.
The workaround must stay until a tested pnpm release or explicitly adopted build includes the fix.

## What does not work

Adding another shim dependency to Advisor would leave Pi's broken dependency link in place.
Removing the shim would violate the runtime contract recorded in the shim decision.
Changing DSH's loader cannot repair a failure reproduced by direct Node import.
Hand-editing the store symlink would leave the next install able to recreate it incorrectly.

## Upstream filing decision

### Completed assessment at 2026-11-05T16:34:41+00:00

Recommend filing a new issue with the public reproduction and an accompanying draft PR for the verified prototype.
The [filing assessment](../planning/pnpm-symlinked-store-filing.md) records the completed checks and ranking.
Filing remains the poster's decision;
overstating an agent-authored prototype risks maintainer time and the poster's credibility.
The candidate must remain a draft with its limits disclosed.

The earlier turn stopped at artifact delivery while a mandatory filing verdict remained.
That was a completion error,
not an external blocker.
The remaining assessment has now been carried through to a recommendation.

The audit also closed an actual test gap:
pnpm v10 disables the global virtual store when `CI=true`,
even when it was requested.
Its initial green job therefore did not exercise the reported condition.
The fixture now sets `CI=false` for its child processes and asserts the consumer's physical location in the shared store.
At commit `3f9f3ce8a5b2e56865fa5278c220e550375668ad`,
[run 42063180350][filing-run] and [repeat 42063626526][filing-repeat]
both reproduce the same missing-package failure on v10,
v11,
and v12 with successful install,
real-store control,
and workaround steps.
Their step conclusions and diagnostic logs were inspected.

The deciding v10 source was fetched and its text confirmed in `agent-browser`:

```ts
// config/config/src/index.ts:636 at v10.34.9
if (pnpmConfig.ci) {
  pnpmConfig.enableGlobalVirtualStore = false
}
```

[v10 source selection][v10-ci-source].
The earlier v10 scope limit is superseded by this verified result.

[filing-run]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42063180350
[filing-repeat]: https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42063626526
[v10-ci-source]: https://raw.githubusercontent.com/pnpm/pnpm/v10.34.9/config/config/src/index.ts#:~:text=pnpmConfig.enableGlobalVirtualStore%20%3D%20false

- Upstream fault:
  reproduced in pnpm `10.34.9`,
  `12.16.1`,
  `12.18.0`,
  and `11.28.11`,
  supported by the public CI runs and the failing source regression.
- Fixable upstream:
  yes;
  the saved prototype passes the targeted Linux symlink tests.
- Supported use case:
  the installed CLI accepts `link:` overrides with `virtualStoreType: global` in both fixture arms.
  The real-directory control succeeds with the same manifest;
  path spelling is the measured difference.
- Contribution policy:
  the checked-out `CONTRIBUTING.md` permits AI-assisted contributions,
  requires disclosure and contributor understanding,
  and warns against unreviewed agent output.
  No external contribution was submitted.
- Maintainer intent and duplicates:
  the earlier empty CLI searches were insufficient.
  Fresh searches found related issues,
  including 17448,
  14286,
  and 9618;
  their inputs and source boundaries were checked against this fixture.
  The planning assessment records why this store-path case warrants its own report.
  All 202 open PRs were examined for changes in the relevant paths;
  the only matching draft reformats imports.
- Prototype:
  verified at the filesystem-library boundary on Linux,
  both locally and in public CI.
  The prototype's scope is the Rust implementation;
  the release matrix separately establishes affected CLI versions and platforms.

The filing assessment and recommendation are now complete.
The earlier absence of a rebuild of the entire CLI was not a requirement for publishing a minimal reproduction;
it must not be used to stop that work.
No upstream issue or pull request has been sent.
Any later submission remains the poster's decision:
unreviewed agent output can consume maintainer time and harm the poster's reputation.

### Local report draft

Title:
`pnpm 12.18.0 creates broken dependency links when the global store traverses a directory symlink`.

The body follows the checked bug-report form's field order.
The repro commands are the literal commands used by the public workflow in a fresh checkout.
Every field is backed by the recorded commands or CI logs;
no human verification is claimed.

~~~md
### Verify latest release

- [x] I verified that the issue exists in the latest pnpm release

### pnpm version

_No response; the primary reproduction uses the latest major release._

### Which area(s) of pnpm are affected? (leave empty if unsure)

Store;
Operating System (Windows, MacOS, Linux).

### Link to the code that reproduces this issue or a replay of the bug

https://github.com/Aquaticat/pnpm-symlinked-store-repro

### Reproduction steps

Use a checkout of the reproduction repository with `PNPM_VERSION=12.18.0`.
The workflow provisions checksum-pinned Node and pnpm.
It then runs these commands from the repository root:

```sh
# Reproduction repository root
node provision.ts
node reproduce.ts prepare control
node reproduce.ts assert control
node reproduce.ts prepare symlinked
node reproduce.ts assert symlinked
```

Both installs succeed.
The real-directory control imports its dependency successfully.
The final assertion fails with `ERR_MODULE_NOT_FOUND` for `proper-lockfile`.
No DSH installation,
private repository,
registry package,
or credentials are needed.

### Describe the Bug

The synthetic consumer declares a dependency replaced by a local `link:` override.
With the global virtual store reached through a directory symlink,
pnpm writes a relative dependency link that resolves to a nonexistent directory.
The same install through the physical directory works.

The dependency-import assertion fails in both inspected CI runs:

- https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42063180350
- https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42063626526

Provisioning,
the real-directory control,
and the aliased-directory install pass in the affected jobs.
Reinstalling with the canonical store path makes the import pass.
Linux also reproduces on 10.34.9,
11.28.11,
and 12.16.1.
The fixture explicitly verifies that the consumer uses the global virtual store;
it disables pnpm's CI-mode shortcut for that test.

Related pnpm/pnpm#17448 and its fix concern symlinked project `node_modules`.
This fixture uses an ordinary project directory and an aliased store,
and still fails on the latest release after that fix.
It also differs from pnpm/pnpm#9618:
the dependency is declared and its installed link is dangling.

Verification was performed by Codex automation;
no separate human rerun is claimed.

The repository includes a Rust source prototype.
Its new regression fails on unchanged source,
then all 19 Linux symlink tests pass with the patch:
https://github.com/Aquaticat/pnpm-symlinked-store-repro/actions/runs/42055609601

This prototype is not a claim that every affected implementation is already fixed.

### Expected Behavior

The installed consumer can import its declared dependency with either spelling of the same store directory.

### Which Node.js version are you using?

26.13.0.

### Which operating systems have you used?

- [x] macOS
- [x] Windows
- [x] Linux

### If your OS is a Linux based, which one it is? (Include the version if relevant)

Ubuntu on GitHub's `ubuntu-latest` runner.
The other jobs use `macos-14` and `windows-latest`.

### Contributing a fix

- [x] I am working on a fix

Written by an agent (Codex, GPT-6).
~~~
