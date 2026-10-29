# mpm-shadow

Shadows `mpm` ([Meta Package Manager](https://mpm.run)): runs the real `mpm`, then deletes the superseded cached versions
of this repository's own crates from cargo's registry.
Started as `mpm` (a link the toolchain plan creates), it finds the real `mpm` on PATH, runs it with the arguments
untouched and the standard streams inherited, waits for it, prunes, and exits with mpm's own status.
Started as `mpm-shadow`, it offers `prune` and `--help`.

## Why

mpm 8.2.0 has no cleanup for its `cargo` manager and removes no previous version on install or upgrade
(`doc/decision/mpm-toolchain-adoption.md`, section on pruning the repository's own crates).
The restore manifest routes the repository's crates through that manager, whose plain `cargo install` fills
`registry/cache/<index>/<crate>-<version>.crate` and `registry/src/<index>/<crate>-<version>/` with every version it
ever fetched.

## What it does

- Runs the real `mpm` as a child, not by replacing itself, because the prune has to run after mpm finishes.
- Passes arguments through byte for byte, including ones that are not valid UTF-8, and adds nothing to standard output.
- Exits with mpm's exit code, or `128` plus the signal number when mpm died from a signal.
- Prunes after every run that started, whatever the exit code and whatever the verb:
  the verb is hard to find when options come first, `mpm upgrade --all` exits 0 even when some managers fail,
  and the prune is idempotent.
  A run that could not start prunes nothing.
- Touches only crates whose name starts with `monochromatic-`.
  For each, it keeps the version cargo's install record (`<cargo home>/.crates.toml`) names as installed,
  or the newest cached version when the record does not name the crate (a library crate such as
  `monochromatic-jsonc-edit`), and deletes every other cached archive and extracted source.
  The record is `.crates.toml` because both `cargo install` and `cargo binstall` update it;
  `.crates2.json` is updated by cargo only, so after a binstall it still names the previous version.
  A crate whose name extends another's (`monochromatic-lint-closure`) is judged on its own versions.
- Reports each deletion on standard error as `mpm-shadow: deleted stale <path>`, and a deletion that fails as a warning;
  neither changes the exit code.
- Skips every candidate that resolves to its own executable when it looks for the real `mpm`,
  so a repository directory and a global directory can both be on PATH.

A cached version that a project's `Cargo.lock` still pins is deleted too when it is not the kept one;
cargo downloads it again on the next online build, but an offline build of that project fails until then.

## Use

The artifact is deliberately not named `mpm`, so a `target/*/mpm` or a tool `bin/mpm` never shadows mpm by accident.
The toolchain plan links it as `mpm` into a gitignored directory that comes first on PATH,
the way `cli-git-native` is linked as `git`.

```bash
# Prune by hand, for example to clear copies left before the shadow was installed.
mpm-shadow prune
```

## Exit codes

- `2`: a bad `mpm-shadow` command line.
- `126`: the real `mpm` was found but could not be started.
- `127`: no real `mpm` on PATH.
- Anything else: mpm's own exit code.

## Checks

```bash
mise run //package/mpm-shadow/cli:lint
mise run //package/mpm-shadow/cli:lint:clippy
mise run //package/mpm-shadow/cli:test
```
