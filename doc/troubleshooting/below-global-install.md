# below 0.9.0 from crates.io differs from GitHub v0.11.0 during global installation

## Version selection at 2026-11-03T01:38:01-05:00

The requested installation uses the machine-global mpm toolchain.
After the crates.io attempt,
the user requested the GitHub version instead.
The final declaration selects GitHub release `v0.11.0`,
commit `6d62fc1f705385dfa71474e7c984b34c868ad089`.
Installation verification is pending.

The personal plan and generated lock stay in
`${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`.
No personal tool inventory is committed to this repository.

## Symptom

`mise run toolchain:update:global -- cargo:below`
resolved `cargo:below@latest` to `0.9.0`.
The crates.io sparse-index response also ended at `0.9.0`,
with checksum `38e9f87936bc2fa6beb9519855fe343fa6e002b457b5e8b6e6e82ca68bc0ea61`.
GitHub's latest release API returned `v0.11.0` and an empty assets array.

With `BINSTALL_DISABLE_STRATEGIES=compile`,
cargo-binstall `1.26.2` reported:

```text
# cargo-binstall diagnostic, propagated by mpm-shadow
For crate below: Fallback to cargo-install is disabled
```

Allowing compilation under the default nightly reached an older dependency:

```text
# rustc diagnostic while compiling the crates.io below 0.9.0 dependency tree
error[E0635]: unknown feature `stdsimd`
  --> ahash-0.8.0/src/lib.rs:48:42
```

The stable-compiler retry was stopped when the user selected GitHub's release.
Its outcome is unknown.

## Root cause and scope

The crates.io request selects the registry's release,
not GitHub's latest tag.
The separately fetched registry index and GitHub release responses demonstrate the version difference.
Neither queried GitHub release (`v0.9.0` or `v0.11.0`) had binary assets.
The disabled compilation fallback therefore left the original installation unsuccessful.
No cause beyond the quoted compiler diagnostic is assigned to the nightly dependency failure.

The existing Git route is owned by
`package/mpm-shadow/cli/src/toolchain/install_cargo.rs`.
Its `install_cargo_git` function passes the revision and package to Cargo:

```rust
// package/mpm-shadow/cli/src/toolchain/install_cargo.rs:117
let request: RunRequest = ctx.request("cargo", &["install", "--git", &url, "--rev", &revision, "--root", &install_text, &package]);
```

## Selected installation

The machine-global plan's `cargo:below` entry now uses:

```jsonc
// Machine-global toolchain.plan.jsonc, inside tools
"cargo:below": {
  "packageId": "below",
  "request": "git:https://github.com/facebookincubator/below@6d62fc1f705385dfa71474e7c984b34c868ad089",
  "route": "cargo-git",
  "bins": ["below"],
  "os": ["linux"]
}
```

The comment beside the declaration explains the release pin.
`mise run toolchain:update:global -- cargo:below` generated the lock.
The Git route compiles source because GitHub publishes no binary for this release;
it does not run cargo-binstall.
Future release upgrades require changing the selected commit.

## Verification

The initial `mise run toolchain:list:global` reported every existing tool in sync.
Both targeted updates retained all unrelated lock rows.
The GitHub build uses installed stable Rust `1.98.0`,
with `RUSTUP_TOOLCHAIN=stable` and `CARGO_BUILD_JOBS=2`.
Build limits are `CPUQuota=200%`,
`MemoryMax=8G`,
`TasksMax=256`,
and `IOWeight=10`.
Scratch files use a private directory under `${HOME}/temp/agent`.

The failing catalog consists of the disabled-fallback attempt and the nightly compilation diagnostic.
Successful installation and runtime checks remain pending.

## What does not work

- `cargo:below@latest` does not select GitHub `v0.11.0`.
- Disabling compilation does not install `below 0.9.0` on this machine.
- The stopped stable build proves no remedy for the nightly failure.

## Clang compatibility at 2026-11-03T01:45:58-05:00

The GitHub release's Rust dependencies compiled,
but its eBPF build failed with host clang `23.1.4`:

```text
# below v0.11.0 build-script diagnostic
exitstat.bpf.c:140:39: error: incompatible pointer types initializing 'const struct mm_struct___pre62 *'
exitstat.bpf.c:145:40: error: incompatible pointer types initializing 'const struct mm_struct___post62 *'
```

The complete diagnostics identify `-Wincompatible-pointer-types`.
Upstream issue `facebookincubator/below#8275` reports the same statements and diagnostic with clang `22`.
The issue comments point to pull request `8280`;
the repository's commit history contains the explicit-cast fix as
`1fa6d036eb8fb9acd3a79c2eb393f1e2f5bd9448`.
That fix is absent from release commit `6d62fc1f705385dfa71474e7c984b34c868ad089`.
No upstream source was edited locally.

A direct compilation of the release's existing `exitstat.bpf.c`,
using the installed libbpf headers and the build script's BPF arguments,
still failed with `CCC_OVERRIDE_OPTIONS=+-std=gnu17`.
The same compilation passed with
`CCC_OVERRIDE_OPTIONS=+-Wno-error=incompatible-pointer-types`.
The pointer diagnostics remained visible as warnings.
This is a compiler compatibility option scoped to the installation command;
the tradeoff is accepting these implicit conversions in the tagged release.

The resumed install uses:

```sh
# From the repository root, after preparing a private scratch directory under ${HOME}/temp/agent.
systemd-run --user --scope \
  --property=CPUQuota=200% --property=MemoryMax=8G \
  --property=TasksMax=256 --property=IOWeight=10 \
  env TMPDIR="${HOME}/temp/agent/below-install-jJBQNI" \
  CARGO_BUILD_JOBS=2 RUSTUP_TOOLCHAIN=stable \
  CCC_OVERRIDE_OPTIONS=+-Wno-error=incompatible-pointer-types \
  mise run toolchain:install:global -- --frozen-lockfile
```

## Failed-install directory at 2026-11-03T01:45:58-05:00

The first GitHub build left its destination with empty `.crates.toml` and `.crates2.json` files,
and no executable.
An immediate retry returned success with `62 unchanged`,
but also printed `toolchain install: unresolved bins: below`.
`command -v below` returned failure.

`package/mpm-shadow/cli/src/toolchain/observe.rs:220` sends the `CargoGit` route to directory observation.
The observation at lines `185` and `190` accepts a matching directory without checking the executable:

```rust
// package/mpm-shadow/cli/src/toolchain/observe.rs:185
if entry.path().is_dir() && !name.starts_with('.') {
    names.push(name);
}
```

```rust
// package/mpm-shadow/cli/src/toolchain/observe.rs:190
if names.contains(&tool.version) {
    return Some(tool.version.clone());
}
```

After listing the destination,
the incomplete directory was moved into the private scratch directory as `failed-git-root`.
The next installation launched Cargo again.
This recovery preserves the failed records;
it does not repair the toolchain's general incomplete-install detection.

## Upstream filing decision

- The clang incompatibility is already tracked and fixed in upstream source.
  The registry publication difference and the abandoned nightly build remain separate observations.
- No additional upstream fix is proposed.
- Source installation is the path documented by below's fetched README.
- Contribution-policy review is not needed for this local installation record.
- No claim about maintainer willingness is made.
- No upstream patch was prototyped because the matching explicit-cast fix already exists.

There is no upstream filing artifact or external message.
