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
Windows and macOS were not exercised,
and the added filesystem lookup cost was not benchmarked.
PnPM v11 behavior was not validated.
These are limits on upstream adoption of the prototype,
not on the verified local Advisor recovery.

The prototype is not installed as the host's pnpm.
The workaround must stay until a tested pnpm release or explicitly adopted build includes the fix.

## What does not work

Adding another shim dependency to Advisor would leave Pi's broken dependency link in place.
Removing the shim would violate the runtime contract recorded in the shim decision.
Changing DSH's loader cannot repair a failure reproduced by direct Node import.
Hand-editing the store symlink would leave the next install able to recreate it incorrectly.

## Upstream filing decision

- Upstream fault:
  yes for pnpm `12.16.1`,
  supported by the disposable install reproduction and the failing source regression.
- Fixable upstream:
  yes;
  the saved prototype passes the targeted Linux symlink tests.
- Supported use case:
  pnpm's `link:` dependency handling explicitly installs these links into shared virtual-store slots
  in `pnpm/crates/deps-restorer/src/create_symlink_layout.rs`.
  The real-directory control succeeds with the same manifest.
- Contribution policy:
  the checked-out `CONTRIBUTING.md` permits AI-assisted contributions,
  requires disclosure and contributor understanding,
  and warns against unreviewed agent output.
  No external contribution was submitted.
- Maintainer intent and duplicates:
  searches on 2026-11-05 for issues matching `symlink store canonical` and `"symlinked" "store"`
  returned no results.
  The broader PR search `symlink canonical` found related path work,
  including closed PR `13603` about mountpoint detection,
  whose code and failure differ from this incident.
  Current `symlink_dir.rs` history and source contain no matching repair.
- Prototype:
  verified at the filesystem-library boundary on Linux;
  full-CLI,
  other-platform,
  v11,
  and performance validation remain outside the completed prototype checks.

No upstream message has been sent,
and this record does not recommend filing yet.
Any later submission remains the poster's decision:
unreviewed agent output can consume maintainer time and harm the poster's reputation.

### Local report draft

Title:
`pnpm 12.16.1 creates broken workspace links when the shared store traverses a directory symlink`.

Pnpm reports a successful install,
but a transitive `link:` override in the global virtual store points at a nonexistent directory.
The link's relative contents were calculated from a symlinked parent spelling,
while the filesystem interprets them from the physical parent.
On this host,
`/home` points to `/var/home`,
and the dependency link incorrectly reaches `/var/var/home`.

Using the canonical store spelling and forcing a reinstall restores the consumer import.
The attached source prototype resolves the link parent consistently for creation and reuse,
and preserves target aliases.
Its new filesystem regression fails on the unchanged source and passes with the patch;
the Linux symlink test group reports 19 passing tests.
The prototype has not been verified as a rebuilt pnpm CLI or on other platforms.

Written by an agent (Codex, GPT-6).
