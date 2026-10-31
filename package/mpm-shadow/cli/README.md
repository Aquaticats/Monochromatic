# mpm-shadow

Owns this repository's toolchain plan and lock,
and shadows `mpm` ([Meta Package Manager](https://mpm.run)).

- As `mpm-shadow` it runs the toolchain commands:
  `lock`,
  `lock --check`,
  `update`,
  `sync`,
  `status`,
  `verify-archive`,
  `bin-paths`,
  and `prune`.
- As `mpm` (a link the toolchain plan creates) it finds the real `mpm` on `PATH`,
  runs it with the arguments untouched and the standard streams inherited,
  waits for it,
  deletes the superseded cached versions of this repository's own crates from cargo's registry,
  and exits with mpm's own status.

## Toolchain plan and lock

People edit `toolchain.plan.jsonc` (JSONC:
comments and trailing commas are allowed).
mpm-shadow is the only writer and the only checker of `toolchain.lock.json`,
which stays strict JSON because `jq` readers in CI and Dockerfiles select
`.tools.<id>.version`,
`.tools.<id>.artifacts.<platform>.{url,sha256}`
and `.tools.<id>.options.androidPackages`.

The machine-global toolchain has the same two files in `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`;
every command takes `--global` to act on them.
The repository scope is found from the working directory by the nearest `toolchain.plan.jsonc`,
or from `MONOCHROMATIC_WORKTREE_ROOT`.

### The plan

```jsonc
// toolchain.plan.jsonc
{
  "schemaVersion": 1,
  "tools": {
    // Why this pin exists belongs here, beside it.
    "bin": { "request": { "pinned": "0.29.4" }, "route": "archive", "bins": ["bin"] },
    "cargo:fd-find": { "request": { "track": "latest" }, "route": "cargo-binstall", "packageId": "fd-find", "bins": ["fd"] },
    // `{version}` stands for the resolved version in `packageId` and in `env` values.
    "rust": {
      "request": { "channel": "nightly" }, "route": "rustup-toolchain", "packageId": "{version}",
      "bins": ["rustc"], "env": { "RUSTUP_TOOLCHAIN": "{version}" }
    }
  }
}
```

- A tool is `request` (`{"track": "latest"}`,
  `{"pinned": "<version>"}`
  or `{"channel": "<name>"}`),
  `route`,
  optional `manager`,
  `packageId`
  and `os`,
  `bins`,
  `env`
  and `options`.
- The routes are
  `bin-sourced`,
  `archive`,
  `cargo-binstall`,
  `cargo-git`,
  `mpm-pnpm`,
  `mpm-uvx`,
  `go-install`,
  `rustup-toolchain`,
  `rustup-components`
  and `android-sdk`.
- An unknown field is an error naming its path.
  `version` and `artifacts` are not plan fields:
  only `update` writes them.
- Tools install in the order written,
  except that `bin`,
  `uv`,
  `mpm`,
  `rustup`,
  `rust`,
  `cargo-binstall`,
  `go`,
  `node`
  and `pnpm` come first.

### The lock

- Tool entries keep the key order `request, version, route, manager, packageId, bins, os, env, options, artifacts`.
- A top-level `digest` (`sha256:` and the hash of the compact JSON of everything else) covers the whole content.
  `lock --check` recomputes it,
  so a hand edit of a resolved version,
  a URL or a hash fails the gate.
  A reformatted file keeps its digest.

## Commands

- `mpm-shadow lock`:
  rewrites the lock's declaration fields from the plan without the network,
  keeping every resolved version and artifact.
  It refuses a new tool or a resolution-affecting edit
  (`request`,
  `route`,
  `manager`,
  `packageId`,
  `os`,
  `assetPattern`,
  `cargoGit`,
  `bootstrap`)
  and names `mpm-shadow update`.
- `mpm-shadow lock --check`:
  the toolchain drift gate for CI,
  beside `file-enforcer check`.
  Writes nothing.
  Exits 1 when the plan and lock disagree or the digest does not match.
- `mpm-shadow lock --adopt`:
  the one-time conversion of a lock that has no digest yet.
  Versions and artifacts are taken as they are and the digest is added.
- `mpm-shadow update [--check]`:
  resolves every tool and writes the whole lock.
  `--check` reports what would move,
  writes nothing,
  installs nothing,
  and exits 1 when something would move.
  A tool without a resolver yet keeps its lock entry and is reported as `skipped`;
  a new tool without one fails the update.
- `mpm-shadow sync`:
  installs every locked tool that is not at its locked version,
  then makes the postinstall links,
  rebuilds the exposure directory and runs the postinstall commands.
- `mpm-shadow status`:
  compares the installed tools with the lock.
  Exits 1 on drift.
- `mpm-shadow verify-archive <tool> <file> [--platform <id>]`:
  hashes a downloaded file and compares it with the lock's SHA-256 for that tool and platform.
- `mpm-shadow bin-paths`:
  prints `exposureDir`,
  `pathDirs`
  and `bins` as JSON for the IDE launcher.
  It reads only the plan and the file system.

`sync` and `status` refuse when the plan and lock disagree,
and name `mpm-shadow lock` or `mpm-shadow update`.

## How it installs

- `rustup toolchain install --no-self-update <channel>`,
  then `rustup component add` and `rustup target add`,
  straight from the lock.
- `cargo binstall --no-confirm --version <locked> <crate>`.
- `pnpm add --global <package>@<locked>`
  and `uv tool install --force <package>==<locked>`,
  or a wheel whose SHA-256 the lock records.
- `archive` tools are downloaded,
  checked against the lock's SHA-256,
  and only then unpacked with `tar` or `unzip`.
  A file that does not match is deleted and nothing is installed from it.
- `bin-sourced` tools go through the `bin` CLI.
- mpm is never asked to install or look up a version:
  mpm 8.2.0 ignores a pinned version for cargo and pnpm,
  and reports registry failures only in an `errors` array while exiting 0.

## Network

Every npm and PyPI version,
first or later,
is looked up directly at `registry.npmjs.org` and `pypi.org` over HTTPS.
crates.io,
`static.rust-lang.org`,
`nodejs.org`
and the GitHub API are asked the same way.
The client is `hyper` with `hyper-rustls` (native root certificates),
HTTPS only,
with bounded timeouts and sizes,
and every failure names the URL.
`curl` is never started.

When an npm or PyPI lookup fails for a tool that is in the lock but not installed,
plain `update` installs just the missing tools (and `node`,
`pnpm`
or `uv` when they need them),
then looks each one up again once.
A second failure is final and says the lookup was made twice.
`update --check` never installs.

## Concurrency

`lock` and `update` read the plan and lock,
remember their content hashes,
take an exclusive OS lock for the scope,
read the hashes again,
and publish the new lock (a temporary file renamed over the old one) only if neither file changed.
A changed file refuses the publish and names the file.

## Bootstrap

A fresh machine needs one prebuilt `mpm-shadow` archive whose SHA-256 is in the lock,
or `cargo install monochromatic-mpm-shadow` after a verified `rustup` install.
`mpm-shadow verify-archive` checks a downloaded archive against the lock before it is run.
Containers and CI can use the same `jq`,
`curl`
and `sha256sum` pattern they use for Node and pnpm.

## mpm shadow mode

mpm 8.2.0 has no cleanup for its `cargo` manager and removes no previous version on install or upgrade
(`doc/decision/mpm-toolchain-adoption.md`,
section on pruning the repository's own crates).
Plain `cargo install` fills `registry/cache/<index>/<crate>-<version>.crate`
and `registry/src/<index>/<crate>-<version>/` with every version it ever fetched.

- Runs the real `mpm` as a child,
  not by replacing itself,
  because the prune has to run after mpm finishes.
- Passes arguments through byte for byte,
  including ones that are not valid UTF-8,
  and adds nothing to standard output.
- Exits with mpm's exit code,
  or `128` plus the signal number when mpm died from a signal.
- Prunes after every run that started,
  whatever the exit code and whatever the verb.
  A run that could not start prunes nothing.
- Touches only crates whose name starts with `monochromatic-`.
  For each,
  it keeps the version cargo's install record (`<cargo home>/.crates.toml`) names as installed,
  or the newest cached version when the record does not name the crate,
  and deletes every other cached archive and extracted source.
- Reports each deletion on standard error as `mpm-shadow: deleted stale <path>`.
- Skips every candidate that resolves to its own executable when it looks for the real `mpm`.

A cached version that a project's `Cargo.lock` still pins is deleted too when it is not the kept one;
cargo downloads it again on the next online build,
but an offline build of that project fails until then.

The artifact is deliberately not named `mpm`,
so a `target/*/mpm` or a tool `bin/mpm` never shadows mpm by accident.
The toolchain plan links it as `mpm` into a gitignored directory that comes first on `PATH`.

## Exit codes

- `0`:
  success.
- `1`:
  the plan and lock disagree,
  the lock's digest does not match,
  an install,
  download
  or lookup failed,
  or `status` found drift.
- `2`:
  a bad command line,
  or a plan or lock file that cannot be read.
- `126`:
  (shadow mode) the real `mpm` was found but could not be started.
- `127`:
  (shadow mode) no real `mpm` on `PATH`.
- Anything else in shadow mode:
  mpm's own exit code.

## Checks

```bash
mise run //package/mpm-shadow/cli:lint
mise run //package/mpm-shadow/cli:lint:clippy
mise run //package/mpm-shadow/cli:test
```
