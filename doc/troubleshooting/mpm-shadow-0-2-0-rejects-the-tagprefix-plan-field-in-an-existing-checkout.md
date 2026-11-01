# `mpm-shadow` 0.2.0 exits 2 on `tools.mpm-shadow.options.tagPrefix is not a known field` in an existing checkout

## Symptom

Observed 2026-10-31 in a disposable repository root that reproduces an existing checkout:
`toolchain.plan.jsonc` has the `mpm-shadow` entry with `options.tagPrefix`,
and both `mpm-shadow` on `PATH`
and `.cache/toolchain-bin/mpm-shadow` (a link to `~/.cargo/bin/mpm-shadow`)
are the released 0.2.0.
Any command that reads the plan prints this and exits 2,
even after `toolchain.lock.json` names a newer `mpm-shadow`:

```text
mpm-shadow: <repository>/toolchain.plan.jsonc: tools.mpm-shadow.options.tagPrefix is not a known field; the known fields are rustComponents, rustTargets, cargoFeatures, asset, allowBuilds, androidPackages, postinstall, binAliases, bootstrap
```

The 0.2.0 binary reports itself with `mpm-shadow --version` as `mpm-shadow 0.2.0`.

## Cause

`tagPrefix` is a resolver option that the plan's `mpm-shadow` entry needs
(`"request": "github:Aquaticats/Monochromatic@latest"` floats over the repository's per-crate release tags,
and the prefix `monochromatic-mpm-shadow-v` selects the ones that belong to `mpm-shadow`).
The option was added after 0.2.0 was released (it lands with 0.2.1,
which was not released when this was written),
so 0.2.0 decodes the plan's options against a fixed list and refuses the field:

```rust
// package/mpm-shadow/cli/src/toolchain/model_decode_support.rs:81-89
pub(super) fn reject_unknown_keys(map: &Map<String, Value>, allowed: &[&str], path: &str) -> DecodeResult<()> {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{path}.{key} is not a known field; the known fields are {}", allowed.join(", ")));
        }
    }
    return Ok(());
}
```

The `mpm-shadow` that runs matters because an existing checkout keeps its old copy:
`.cache/toolchain-bin/mpm-shadow` was linked by an `install` made before the lock moved,
and `.config/mise/conf.d/20-env.toml` puts `.cache/toolchain-bin` first on `PATH`,
so a task that runs `mpm-shadow` by name ran 0.2.0 no matter which version the lock named.

## Verification

In the disposable root,
with the 0.2.0 copy first on `PATH` and behind the link:

```sh
mpm-shadow --version
mpm-shadow install --lockfile-only --frozen-lockfile
```

printed `mpm-shadow 0.2.0`,
then the `tagPrefix` message of the Symptom section,
exit 2 (also through the link).

With the fix in place,
`mise run toolchain:lock:check` in the same root printed
`[toolchain] toolchain.lock.json names mpm-shadow 0.2.1 and no installed copy is at that version or above (<root>/.cache/toolchain-bin/mpm-shadow is mpm-shadow 0.2.0, older than the lock's 0.2.1, and never runs); fetching its archive for linux-x64`,
then `fetched and verified mpm-shadow 0.2.1 (SHA-256 ...)`,
then the 0.2.1 program's `toolchain lock is current: 64 tools, digest verified (...)`,
exit 0.
The fetched copy sat in `<data home>/monochromatic-dev/toolchain/mpm-shadow/0.2.1/` with a `current` link to `0.2.1`,
and the 0.2.1 `toolchain:list` that followed reported 63 of 64 tools drifted in the empty home,
so it counted the fetched `mpm-shadow` as installed.
A second run fetched nothing.
The stand-in for the release archive was served from a local address,
with its real SHA-256 in the lock.

## Fix

The root `toolchain:*` tasks of `mise.toml` no longer run `mpm-shadow` from `PATH`.
They run `package/monorepo-task/core-exec/src/mpm-shadow.ts`,
which follows the owner's rule "At least the lock's version":
it runs the newest installed copy of this repository's `mpm-shadow`
(`.cache/toolchain-bin/mpm-shadow` or a finished version folder of the toolchain root)
whose version is at or above the version `toolchain.lock.json` records,
with a note naming `mise run toolchain:update mpm-shadow` when that copy is newer than the lock,
and a copy older than the lock never runs.
Only when no copy is at or above the lock's version does it download the lock's archive for the platform,
check its SHA-256 against the lock,
and install it where `mpm-shadow install` puts an archive-route tool,
`<toolchain root>/mpm-shadow/<version>/` with the completion marker and the `current` link.
`mise run toolchain:install` is therefore the whole recovery on an existing checkout that has Node 24 or newer.
Without the task,
fetch the archive the lock names (see the README section "Clone and bootstrap")
and run that binary by its path for the first `install`;
that `install` rewrites `.cache/toolchain-bin/mpm-shadow` to the installed copy.

## What does not work

- Running `mpm-shadow` by name,
  or through `.cache/toolchain-bin`:
  both reach the stale 0.2.0.
- Editing `toolchain.plan.jsonc` to drop `tagPrefix`:
  the plan entry would then float to the newest release of any crate in the repository.
- `mpm-shadow update` with 0.2.0:
  it reads the plan first and exits 2 the same way.

## Upstream filing decision

No upstream filing:
the program is this repository's own,
and the behavior is its strict plan decoder working as designed.
