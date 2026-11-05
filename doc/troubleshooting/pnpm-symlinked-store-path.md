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

[link-source]: https://raw.githubusercontent.com/pnpm/pnpm/c7b46aedef7b81c07816cb551187449a3dd81792/pnpm/crates/fs/src/symlink_dir.rs#:~:text=crate%3A%3Arelative_path(parent%2C%20original)

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
Post-change verification is pending.

The prototype is not installed as the host's pnpm.
The workaround must stay until a tested pnpm release or explicitly adopted build includes the fix.

## What does not work

Adding another shim dependency to Advisor would leave Pi's broken dependency link in place.
Removing the shim would violate the runtime contract recorded in the shim decision.
Changing DSH's loader cannot repair a failure reproduced by direct Node import.
Hand-editing the store symlink would leave the next install able to recreate it incorrectly.

## Upstream filing decision

No upstream message has been sent.
Source references and the filing assessment will be recorded with final verification.
