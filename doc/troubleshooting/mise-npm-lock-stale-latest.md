# mise npm backend keeps a stale lockfile-pinned "latest" version
(npm:socket@2.1.0, a deprecated bad build) after the npm dist-tag rolls back

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

mise 2026.6.9,
 npm backend,
 `"npm:socket" = "latest"` in config.
`mise install` keeps installing `socket@2.1.0`,
 a version the author deprecated as a "bad build",
 even though
the npm `latest` dist-tag has since rolled back to the maintained `1.1.x` line.
 The installed binary crashes
at load.

## Symptom

Running the mise-installed `socket` crashes immediately:

```text
.../installs/npm-socket/2.1.0/.../node_modules/socket/dist/bootstrap.js:114
...minified bundle...

ReferenceError: __SOCKET_CLI_VERSION__ is not defined
    at Object.<anonymous> (.../node_modules/socket/dist/bootstrap.js:114:41)
    at Module._compile (node:internal/modules/cjs/loader:1873:14)
```

`__SOCKET_CLI_VERSION__` is a build-time define that the author's build step (`node scripts/build.mjs`) is
supposed to substitute before publish.
 In the `2.1.0` tarball it was never substituted,
 so every CLI entry
(`socket`,
 `socket-npm`,
 `socket-npx`,
 `socket-pnpm`,
 `socket-yarn`) throws at module load.
 The package has no
`postinstall`/`prepare`/`prepack` script,
 so no installer rebuilds it:
 the broken `dist/bootstrap.js` ships
as-is and fails identically under npm or pnpm.

## Root cause

Two facts combine.
 First,
 `socket@2.1.0` is a bad publish that the author then deprecated:

```bash
npm view socket@2.1.0 deprecated --json   # "bad build"
npm view socket dist-tags --json          # {"latest":"1.1.124"}
```

The `2.x` line (`2.0.8`,
 `2.0.9`,
 `2.0.10`,
 `2.1.0`) sits at a higher semver than the `latest` dist-tag,
 which
points back at the maintained `1.1.x` line.
 The author published `2.1.0` (npm sets `latest` to the newest
publish),
 found it broken,
 deprecated it,
 and moved `latest` back to `1.1.x`.

Second,
 mise captured `2.1.0` into `mise.lock` during the window when `2.1.0` was the `latest` dist-tag,
 and
`mise lock` never re-checks an existing concrete pin against the current dist-tag.
 From the mise source
(`jdx/mise`,
 commit `bf67479`,
 src tree version `2026.6.11`):

`src/cli/lock.rs:24`

```rust
/// Updates checksums and download URLs for all platforms already specified in the lockfile.
```

Pass one iterates the already-resolved toolset (which honours the locked version when a lockfile exists) and
keeps every concrete version verbatim,
 skipping only the symbolic string `"latest"`:

`src/cli/lock.rs:537`

```rust
// Skip unresolved symbolic versions (e.g., a lockfile poisoned with "latest"
// as the version). Pass 2's fallback will resolve these to a concrete version.
if tv.version == "latest" {
    continue;
}
```

Pass two is the only path that re-resolves against the registry,
 and it runs only for requests not already
matched by pass one,
 with `use_locked_version` forced off:

`src/cli/lock.rs:583`

```rust
let should_resolve_overridden =
    request.version() == "latest" || source.is_idiomatic_version_file();
if !matched_resolved && should_resolve_overridden {
    let mut resolve_options = match request.resolve_options(base_resolve_options) { ... };
    resolve_options.use_locked_version = false;
```

So once the concrete `2.1.0` is in the lock,
 the loop is self-perpetuating:
 resolution honours the lock
(`use_locked_version`) and yields `2.1.0`,
 pass one sees that concrete version and keeps it,
 and pass two's
re-resolution is skipped because the tool already matched.
 `mise install` then honours the lock and installs
the broken build.
 The pin only changes when something forces re-resolution (the entry is removed,
 the version
is edited,
 or `mise upgrade` finds a newer version,
 which never happens here because `2.1.0` already outranks
`1.1.124` by semver).

Note this is distinct from any "lockfile mutates on its own" churn:
 the `2.1.0` entry is a one-time stale
capture,
 not an ongoing rewrite.

### Earlier wrong reading

An earlier hypothesis was that `mise lock` resolves `"latest"` to the highest semver (`2.1.0`) rather than to
the dist-tag.
 That is wrong.
 `mise latest npm:socket` and a fresh `mise lock --dry-run` (no prior entry) both
resolve `"latest"` to the dist-tag `1.1.124`.
 The `2.1.0` came from `mise lock` writing the dist-tag that was
current at that time,
 then preserving it as a concrete pin afterward.

## Verification

Versions under test:
 mise `2026.6.9` (local),
 mise source `bf67479`;
 npm `socket` dist-tag `latest` =
`1.1.124`;
 broken version `2.1.0`.

Reproduce the crash in a throwaway directory (no repo state touched):

```bash
D=$(mktemp -d); cd "$D"; pnpm init >/dev/null 2>&1
pnpm add socket@2.1.0 >/dev/null 2>&1
node node_modules/socket/dist/bootstrap.js --version
# -> ReferenceError: __SOCKET_CLI_VERSION__ is not defined  at .../dist/bootstrap.js:114
cd - >/dev/null; rm -rf "$D"
```

Confirm mise would re-resolve correctly once the pin is symbolic or absent,
 but preserves a stale concrete pin:

```bash
mise latest npm:socket            # 1.1.124  (dist-tag, the correct resolution)
mise lock --dry-run npm:socket    # with 2.1.0 in mise.lock -> npm:socket@2.1.0 (stale, kept)
                                  # with 1.1.124 in mise.lock -> npm:socket@1.1.124
```

Works cleanly:

- `socket@1.1.124` (the maintained latest):
   `node node_modules/socket/dist/cli.js --version` prints `1.1.124`.
- After the workaround,
   mise-installed `socket --version` prints `1.1.124`.

Fails:

- `socket@2.1.0` (any CLI entry):
   `ReferenceError: __SOCKET_CLI_VERSION__ is not defined`.

## Verified workarounds

Correct the lockfile pin,
 then reinstall:

Current pin source:
 the recipe below edits `mise.lock`,
 which the toolchain migration retires.
 A stale repository tool version now fixes at the typed tool plan in
 `package/file-enforcer/core/src/toolchain/`,
 which regenerates `toolchain.lock.json`;
 rerun `mise run toolchain:install` to converge
 (`doc/troubleshooting/mpm-toolchain-migration.md`).

```bash
# 1. In mise.lock, change the socket entry version:
#      [[tools."npm:socket"]]
#      version = "1.1.124"   # was 2.1.0
# 2. Remove the broken install and reinstall via the configured package manager.
#    Run from a directory OUTSIDE the repo workspace so pnpm does not apply the
#    repo's pnpm-workspace.yaml overrides to a standalone global CLI tool.
mise uninstall --yes npm:socket@2.1.0
( cd /tmp && mise install --force npm:socket@latest )
# 3. If you deleted the install dir, clear the stale bin-path cache and reshim,
#    otherwise `mise exec`/shims may point at the removed content-hash dir:
rm -rf ~/.cache/mise/npm-socket && mise reshim
```

Tradeoffs:
 editing `mise.lock` by hand is normally discouraged,
 but it is the direct fix for a stale concrete
pin.
 This holds durably only because `1.1.124` is the current dist-tag:
 re-resolution now agrees with the lock.
The config stays `"latest"`,
 which is correct.

Removing the lock entry entirely and rerunning `mise install` also works now (re-resolution yields `1.1.124`),
but it is unsafe in general:
 during a window where a bad `2.x` is the `latest` dist-tag,
 fresh resolution would
re-pin the bad version.

## What does not work

- Switching package manager (npm versus pnpm):
   `socket` has no install scripts,
   so both unpack the same broken
  `dist`.
   The `__SOCKET_CLI_VERSION__` error is a publish defect,
   not an install-time difference.
- Bare `mise install --force` (no tool argument):
   the argless path installs only missing tools,
   so an
  already-installed `2.1.0` is never replaced.
- `mise lock` (refresh):
   preserves the existing concrete `2.1.0` and refreshes only its checksums and URLs,
   as
  the source above shows.
- `mise upgrade npm:socket`:
   `2.1.0` already outranks the `1.1.124` dist-tag by semver,
   so upgrade sees nothing
  newer and will not move (let alone downgrade) the pin.
   Reasoned from the semver ordering,
   not run here,
   since
  reproducing it would require re-pinning the bad version.

## Upstream filing decision

`.out-of-scope/` has no exemption matching mise,
 socket,
 or the npm backend.
 Walking the six constraints:

- Really upstream's fault?
   Only partly.
   `socket@2.1.0` is a genuine bad publish,
   but the author already
  deprecated it ("bad build"),
   which is the upstream remedy.
   mise preserving a concrete locked version is
  intended lockfile behavior,
   not a defect:
   pinning is the point of a lockfile.
- Can upstream fix it?
   socket already has (deprecation plus a maintained `1.1.x` `latest`).
   mise could
  optionally warn when a locked version is deprecated,
   but that is a feature request,
   not a bug.
- Supporting this use case?
   Not applicable;
   nothing is being misused.
- Would the repo welcome our contribution?
   Not assessed,
   because there is no defect to contribute a fix for.
- Will they likely fix it?
   socket already deprecated the bad version;
   there is no open mise defect here.
- Prototyped a minimal fix?
   Not applicable;
   no upstream code change is warranted.

Decision:
 do not file anything upstream.
 The bad version is already deprecated by its author,
 and the lockfile
staleness is expected behavior resolved on our side by correcting the pin.
 No duplicate search was needed
because there is nothing to file.
