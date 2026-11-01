# mise 2026.10.0 treats ordinary Rust installs as user links and ignores the lock

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

## Symptom

On 2026-10-16,
the historical native trial source locked Rust to `nightly-2026-10-13`,
but host mise `2026.10.0` selected `nightly-2026-10-15`.
Both `mise ls --current --json rust` and an actual `mise exec -- rustc -Vv` reproduced the mismatch.
The commands exited successfully;
there was no mise diagnostic explaining that lock resolution had been bypassed.

The source was the immutable native trial checkout at
`$HOME/temp/agent/depot-ci-source-truepeak-20261016`.
Its configuration requested the floating `nightly` channel with components and Android targets.
Ordinary mise Rust install entries were absolute symlinks into `$HOME/.cargo/bin`,
which contained the Rustup proxies.
No manually linked toolchain was needed to trigger the behavior.

This is the same defect as [mise issue 13912][issue].
[Pull request 13915][fix] fixed it in [mise 2026.10.1][release].
The isolated downloaded `2026.10.9` binary selected the locked nightly and invoked the expected compiler.
The host installation was not upgraded during diagnosis.
This host result does not establish the cause of a separate cloud failure running a newer mise version.

## Root cause

The failing source is mise tag `v2026.10.0`,
commit `bc11f90c74eba23bf0d7350efb540e62fb7d9ffd`.
The inspected fixed source is tag `v2026.10.9`,
commit `ac91d06fa6100935b7a07b08bebf03dd6dd9a0dd`.
Source paths in this section belong to the corresponding mise checkout.

In `v2026.10.0`,
`src/plugins/core/rust.rs:592` creates the normal installation symlink after Rustup installs the toolchain:

```rust
// mise v2026.10.0: src/plugins/core/rust.rs:592
file::remove_all(tv.install_path())?;
file::make_symlink(&runtime.bin_dir, &tv.install_path())?;
```

The version resolver in `src/toolset/tool_version.rs:192` consults the lock only when no linked version exists:

```rust
// mise v2026.10.0: src/toolset/tool_version.rs:192
if opts.use_locked_version
    && !has_linked_version(request.ba())
    && let Some(lt) = request.lockfile_resolve(config)?
{
    return Ok(Self::from_lockfile(request.clone(), lt).with_before_date(opts.before_date));
}
```

`has_linked_version` scans installed entries.
Its condition at `src/toolset/tool_version.rs:1248` misclassifies a Rustup proxy directory outside mise's directories:

```rust
// mise v2026.10.0: src/toolset/tool_version.rs:1248
if target.is_absolute() && !is_mise_managed_symlink_target(&target) {
    return true;
}
```

Consequently,
the resolver skips the lock lookup and resolves the floating channel.
Component ordering and lockfile serialization cannot repair a lookup that never runs.

The fixed condition in `v2026.10.9`,
`src/toolset/tool_version.rs:1401`,
exempts ordinary Rustup installs:

```rust
// mise v2026.10.9: src/toolset/tool_version.rs:1401
if target.is_absolute()
    && !is_mise_managed_symlink_target(&target)
    && !is_rustup_install(ba, &path, &target)
{
    return true;
}
```

The helper at `src/toolset/tool_version.rs:1416` requires `core:rust`,
a target containing the platform's `rustup` executable,
and a standard Rust toolchain name:

```rust
// mise v2026.10.9: src/toolset/tool_version.rs:1422
ba.full_without_opts() == "core:rust"
    && target.join(rustup).exists()
    && link
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_rustup_toolchain_spec)
```

## Verification

The downloaded release archive was `mise-v2026.10.9-linux-x64.tar.gz`.
Its measured SHA-256 was
`7af2e3d798e9ddec142f5a968a18c3f33aa68d19aa8a4111302d4c7d04feb6fa`.
The probe used already installed Rust toolchains and disabled automatic installation.
It did not regenerate the source lockfile.

The reduced fixture had these files:

```toml
# Disposable fixture: mise.toml
[tools]
rust = { version = "nightly", components = "clippy,rust-src,llvm-tools", targets = "aarch64-linux-android,x86_64-linux-android" }

[settings]
lockfile = true
```

```toml
# Disposable fixture: mise.lock
lockfile_version = 3

[[tools.rust]]
version = "nightly-2026-10-13"
backend = "core:rust"
specifiers = ["nightly"]

[tools.rust.options]
components = "clippy,llvm-tools,rust-src"
targets = "aarch64-linux-android,x86_64-linux-android"
```

With both dated toolchains already installed through mise,
the reproducing commands run from the trusted fixture or historical checkout are:

```sh
# Run inside the disposable fixture or immutable historical checkout.
MISE_AUTO_INSTALL=false mise ls --current --json rust
MISE_AUTO_INSTALL=false mise exec -- rustc -Vv
MISE_AUTO_INSTALL=false MISE_RUST_VERSION=nightly-2026-10-13 mise ls --current --json rust
MISE_AUTO_INSTALL=false MISE_RUST_VERSION=nightly-2026-10-13 mise exec -- rustc -Vv
```

The verification harness stored both selection metadata and actual compiler output before asserting equality
between the baseline selected version and the lock.
Its baseline assertion failed on `2026.10.0` and passed on `2026.10.9`.
The retained artifacts are `rust-selection-repro.json`,
`rust-selection-fixed.json`,
`rust-minimal-result.json`,
and `rust-selection-controls.json` under `$HOME/temp/agent/depot-complete-ci-20261016`.

### Failing catalog

- mise `2026.10.0`,
  original historical source:
  selected `nightly-2026-10-15`;
  actual compiler commit `7df81381d027710708209f36b98d11b1c88f8d2e`,
  dated `2026-10-14`.
- mise `2026.10.0`,
  reduced fixture:
  selected `nightly-2026-10-15`.
- The reduced fixture still selected the wrong nightly after matching component order,
  removing component options from both files,
  or changing the fixture lock format to version `2`.

### Working catalog

- mise `2026.10.0` with `MISE_RUST_VERSION=nightly-2026-10-13`:
  selected the requested dated nightly and invoked compiler commit
  `e9bbadf6824e65b3583c2dda3a0926ec76df9143`,
  dated `2026-10-12`.
- Downloaded mise `2026.10.9` without an override:
  selected the lock's dated nightly and invoked that same compiler commit.
- Downloaded mise `2026.10.9` with the explicit dated override:
  retained the same selected toolchain and actual compiler.

## Verified workarounds

For an immutable historical execution,
set the dated override from that source's lock and verify the actual compiler:

```sh
# Run inside the immutable historical checkout whose lock names this nightly.
MISE_AUTO_INSTALL=false MISE_RUST_VERSION=nightly-2026-10-13 mise exec -- rustc -Vv
```

The tradeoff is duplicated selection state:
an environment override takes precedence over future lock changes.
Read the immutable source lock for each execution;
do not carry this date into another source revision.
This probe verified compiler selection,
not the availability of every requested cross-compilation target.

Alternatively,
invoke a verified fixed mise binary without replacing the host installation:

```sh
# Run inside the same historical checkout after extracting the tested release archive.
MISE_AUTO_INSTALL=false "$HOME/temp/agent/depot-complete-ci-20261016/mise/bin/mise" ls --current --json rust
MISE_AUTO_INSTALL=false "$HOME/temp/agent/depot-complete-ci-20261016/mise/bin/mise" exec -- rustc -Vv
```

The tradeoff is a different mise release with changes beyond this fix.
Record its version and preserve before/after tracked-source identity checks.
The first fixed release was established from the upstream release record;
the locally exercised fixed binary was `2026.10.9`.

## What does not work

- Disabling automatic installation alone does not make `2026.10.0` honor this Rust lock.
  All failing probes disabled it.
- Sorting component names does not repair the selection.
  The matching-order control still selected the newer nightly.
- Removing component options does not repair the selection.
  The no-components control still selected the newer nightly.
- Downgrading the disposable fixture's lock format does not repair the selection.
  The version `2` control still selected the newer nightly.
- Treating this as evidence against newer cloud mise is unsupported.
  The directly tested `2026.10.9` binary selected the lock correctly;
  a cloud incident needs its own actual binary and source evidence.

## Upstream filing decision

The `.out-of-scope/` catalog had no mise-specific exemption.
Tracker searches covered open and closed issues and pull requests using Rust,
lockfile,
symlink,
and Rustup terms.
Issue [13912][issue] and pull request [13915][fix],
including their comments,
already contain the same root cause,
isolated reproduction,
and fix.

1.  Upstream fault:
    yes for mise `2026.10.0`;
    its normal Rust installation layout triggers its own user-link classifier.
2.  Fixable upstream:
    yes;
    the merged implementation distinguishes normal Rustup installs from custom links.
3.  Supported use case:
    yes;
    mise's Rust documentation describes Rustup-backed toolchain selection,
    and the upstream issue and regression tests explicitly cover honoring a Rust lock.
4.  Contribution policy:
    `CONTRIBUTING.md` and `docs/contributing.md` allow disclosed AI assistance for one's own filings,
    but restrict AI replies to existing threads based on contribution history.
    No eligibility claim or reply is needed here.
5.  Will upstream fix it:
    already fixed by pull request `13915`,
    released in `2026.10.1`.
6.  Minimal fix and verification:
    the existing merged fix is present in the inspected `2026.10.9` source,
    and the same source-selection/actual-compiler reproduction passes with its release binary.
    Creating another patch for a fixed defect would add no implementation evidence.

There is nothing additive to file or comment.
No upstream mutation was made.

[issue]: https://github.com/jdx/mise/issues/13912
[fix]: https://github.com/jdx/mise/pull/13915
[release]: https://github.com/jdx/mise/releases/tag/v2026.10.1
