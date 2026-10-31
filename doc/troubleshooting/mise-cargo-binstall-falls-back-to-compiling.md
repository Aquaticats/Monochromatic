# mise 2026.10.0 installs a `cargo:` tool by compiling it when its release tag and archive names differ from cargo-binstall's defaults

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

A crate that publishes prebuilt archives to GitHub releases is still compiled from source
by `mise install cargo:<crate>` unless the crate's `Cargo.toml` tells `cargo-binstall` where the archives are.
The compile takes minutes and prints almost nothing,
and while it runs every `mise` command in the same project waits on its install lock.
The case here is `monochromatic-lint` 0.1.0,
whose release is tagged `monochromatic-lint-v0.1.0` in a repository that releases several crates.

## Symptom

`mise install cargo:monochromatic-lint` (tool declared as `"cargo:monochromatic-lint" = "0.1.0"`)
ran for more than five minutes.
The recorded log,
with the progress bar removed:

```text
mise cargo:monochromatic-lint@0.1.0 cargo-binstall:  INFO resolve: Resolving package: 'monochromatic-lint@=0.1.0'
mise cargo:monochromatic-lint@0.1.0 cargo-binstall: ERROR Fatal error:
mise cargo:monochromatic-lint@0.1.0   × For crate monochromatic-lint: Fallback to cargo-install is disabled
mise cargo-binstall found no prebuilt binary; falling back to cargo install
mise cargo:monochromatic-lint@0.1.0   Installing monochromatic-lint v0.1.0
```

The `ERROR` line looks fatal and is not:
`mise` continues with `cargo install`.
At that time the release `monochromatic-lint-v0.1.0` existed with all eight archives,
so the user's objection was fair:
a prebuilt binary was one download away.

A second symptom followed.
After the first install was killed,
`mise exec -- true` in the same worktree printed
`waiting for install lock held by pid <pid>` every three seconds and never returned (25 s bound,
exit 124).
The holder was a `mise exec -- monochromatic-lint --version` that an earlier command line had started
and that had been stuck in uninterruptible sleep (`Dl`) for seven minutes.

## Root cause

`mise` runs `cargo-binstall` with only the `compile` strategy disabled,
and falls back to `cargo install` on any failure it recognizes
(`src/backend/cargo.rs` in `jdx/mise` at commit `bd4494ef07f27a8ea3f23bec8ddaf4ea3b0812a9`):

```rust
// src/backend/cargo.rs:37-38
const CARGO_BINSTALL_NO_FALLBACK_EXIT_CODE: i32 = 94;
const CARGO_BINSTALL_DEFAULT_DISABLED_STRATEGIES: &[&str] = &["compile"];
```

```rust
// src/backend/cargo.rs:246-251
BinstallStatus::Enabled(cargo_binstall) => {
    let mut cmd = CmdLineRunner::new(cargo_binstall).arg("-y");
    let disabled_strategies = CARGO_BINSTALL_DEFAULT_DISABLED_STRATEGIES
        .iter()
        .copied()
        .chain(
            (!Settings::get().cargo.binstall_quickinstall)
```

```rust
// src/backend/cargo.rs:268-276
if Settings::get().cargo.binstall_only
    || !result.as_ref().is_err_and(|err| {
        Error::get_exit_status(err)
            == Some(CARGO_BINSTALL_NO_FALLBACK_EXIT_CODE)
    })
{
    result?;
}
info!("cargo-binstall found no prebuilt binary; falling back to cargo install");
```

Exit status 94 from `cargo-binstall` therefore means "continue with `cargo install`" unless the setting
`cargo.binstall_only` is on.
The message "Fallback to cargo-install is disabled" is `cargo-binstall`'s own wording for the `compile` strategy that
`mise` turned off,
and it is the exit-94 case.

`cargo-binstall` then found no archive.
With `--log-level debug` it read the crate manifest from the registry
(`parse_manifest ... version="0.1.0"`),
found no `[package.metadata.binstall]`,
and rendered only the default URL set.
The default set is documented in `SUPPORT.md` of `cargo-bins/cargo-binstall`
at commit `cafc6eb3a0057e9d728f71b3d1eb4c85a8a6f9e7`,
lines 129 to 155:
every default file name appended to `{ repo }/releases/download/{ version }/`
and `{ repo }/releases/download/v{ version }/`.
The file name `{ name }-{ version }-{ target }{ archive-suffix }` is among the defaults,
so the archive names this repository uses
(`monochromatic-lint-0.1.0-x86_64-unknown-linux-gnu.tar.gz`) match.
The directory does not:
the repository tags each crate's release `monochromatic-lint-v<version>`
because one repository releases several crates,
and the default paths only know `0.1.0` and `v0.1.0`.
The debug log shows the probes,
for example
`https://github.com/Aquaticat/Monochromatic/releases/download/0.1.0/monochromatic-lint-0.1.0-x86_64-unknown-linux-gnu.tar.gz`.
The cutover delegate's note that the layout is "cargo-binstall's default layout" was wrong for the directory.

The published manifest is what `cargo-binstall` reads,
so a repository-side fix needs a new published version.
The 0.1.0 release is immutable (the repository has immutable releases),
so its assets cannot be renamed either.

The lock wait is `mise`'s per-tool install lock:
a `mise exec` in a project that declares a missing tool tries to install it first,
and a second one waits for the first.
The control below shows `mise exec` returning in 0.8 s with auto-install off.

## Which binstall mise runs

The `cargo-binstall` that ran is not bundled in the `mise` executable.
It is the tool this repository declares (`cargo-binstall = "latest"` in `mise.no-env.toml`),
installed like any other tool under `~/.local/share/mise/installs/cargo-binstall/<version>/`;
the failing log names `.../installs/cargo-binstall/1.23.0/cargo-binstall`.
The backend lists it as an optional dependency (`src/backend/cargo.rs:172-174`:
`Ok(vec!["cargo-binstall", "sccache"])`) and runs it when it is installed (`binstall_status`,
line 488).

`mise` does bundle a second installer.
When `cargo-binstall` is not installed,
the setting `cargo.binstall_native` decides whether `mise`'s own Rust
implementation (`src/backend/cargo/native_binstall.rs`) downloads the archive.
Its documentation in `settings.toml` says:
unset means not used in 2026.10.0,
a warning from 2027.1.0,
and `true` from 2027.7.0.
The repository now pins `binstall_native = false` (`[settings.cargo]` in `mise.no-env.toml`),
so a future `mise` default cannot start using the bundled installer on a machine that lacks `cargo-binstall`.
`mise settings get cargo.binstall_native` in the worktree prints `false`.

## Verification

Versions under test:
`mise` 2026.10.0 (linux-x64),
`cargo-binstall` 1.20.1 (run directly) and 1.23.0 (the copy `mise` used),
crate `monochromatic-lint` 0.1.0 (index checksum prefix `8b66bf24aac40`).

Failing probe,
`cargo-binstall` against the registry copy,
with the compile strategy disabled as `mise` does:

```bash
cargo-binstall binstall --dry-run --no-confirm --disable-strategies compile,quick-install \
  --log-level debug monochromatic-lint@0.1.0
```

Result:
exit 94,
`Fallback to cargo-install is disabled`,
18 s,
and every URL it probed was under `releases/download/0.1.0/` or `releases/download/v0.1.0/`.

Working probe,
the same tool pointed at the local manifest that carries the metadata from the next section
(the metadata is not on the registry for 0.1.0,
so this reads the checked-out manifest and downloads the real release):

```bash
cargo-binstall binstall --dry-run --no-confirm --disable-strategies compile,quick-install \
  --manifest-path package/linter/monochromatic-lint monochromatic-lint
```

Result:
exit 0,
`The package monochromatic-lint v0.1.0 (x86_64-unknown-linux-gnu) has been downloaded from github.com`,
`monochromatic-lint => /home/user/.cargo/bin/monochromatic-lint`,
6.6 s.
The failing probe is the positive control:
the same release,
a different manifest,
opposite outcomes.

Lock control,
in the worktree that declares the tool but has it uninstalled:

```bash
MISE_EXEC_AUTO_INSTALL=0 MISE_AUTO_INSTALL=0 mise exec -- true
```

Result:
`mise WARN missing: cargo:monochromatic-lint@0.1.0`,
exit 0,
0.8 s.
Without the two variables,
the same command waited on the lock until the 25 s bound.

## Verified workarounds

Add the table to the crate's `Cargo.toml` and publish a new version
(`package/linter/monochromatic-lint/Cargo.toml`):

```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/{ name }-v{ version }/{ name }-{ version }-{ target }{ archive-suffix }"
bin-dir = "{ name }-{ version }-{ target }/{ bin }{ binary-ext }"
pkg-fmt = "tgz"

[package.metadata.binstall.overrides.x86_64-pc-windows-msvc]
pkg-fmt = "zip"

[package.metadata.binstall.overrides.aarch64-pc-windows-msvc]
pkg-fmt = "zip"
```

The tradeoff is a version bump for a metadata-only change,
because the registry copy is what is read.
The tool pin in `mise.no-env.toml` moves with it.

Set `cargo.binstall_only` to make a missing archive fail in seconds instead of compiling for minutes:

```toml
[settings.cargo]
binstall_only = true
```

The tradeoff is that every `cargo:` tool in the project then needs a prebuilt archive for the host target,
including tools that are only ever built from source today.

To see what a hung `mise` command waits for,
read its output for `waiting for install lock held by pid`
and inspect that pid with `ps -o pid,stat,etime,cmd -p <pid>`.

## What does not work

- Retagging or renaming the 0.1.0 assets.
  Immutable releases refuse it.
- Passing `--disable-strategies` through `mise`.
  `mise` builds that argument itself (`src/backend/cargo.rs:248-251`) and exposes only `cargo.binstall_quickinstall`.
- Waiting it out.
  The install ran for more than five minutes without finishing before it was killed,
  and a second `mise exec` in that project queued behind its lock.
  The verifying build of the same crate (`cargo publish --dry-run`) took 2 m 02 s in the dev profile,
  so a release-profile install from source is slower than that.
- Using `pkill -f` with a pattern that also appears in the shell command that runs it.
  It matched the invoking shell and ended the command with exit 144.

## Upstream filing decision

Nothing is filed.

1. Is it upstream's fault?
   No.
   `cargo-binstall` documents its defaults and the override,
   and `mise` documents `cargo.binstall_only`.
   The one softness is cosmetic:
   `mise` shows an `ERROR` line for a case it handles by compiling,
   and the compile prints no progress beyond a timer.
   That is a wording preference,
   not a defect.
2. to 6.
   Not reached,
   because constraint 1 fails.
   No prototype was made.

`.out-of-scope/` has no entry for `mise` or `cargo-binstall`.

## Repository status

- `mise.no-env.toml` declares `"cargo:monochromatic-lint"` pinned to 0.1.1,
  the first version whose published manifest carries the metadata
  (release `monochromatic-lint-v0.1.1`,
  eight archives).
- Confirmed on 2026-10-10 with `MISE_CARGO_BINSTALL_ONLY=1 mise install cargo:monochromatic-lint`,
  which fails instead of compiling when no archive is found:
  `cargo-binstall` printed
  `The package monochromatic-lint v0.1.1 (x86_64-unknown-linux-gnu) has been downloaded from github.com`
  and `mise` finished in 11.5 s.
  `monochromatic-lint --version` printed `monochromatic-lint 0.1.1`.

## Forbidden-strings 0.5.0 has the same unresolved metadata gap

On October 15,
2026 local time,
or October 16 UTC,
`cargo-binstall` 1.25.3 reproduced the tag-layout problem for `forbidden-strings@0.5.0`.
The registry manifest checksum is
`6ddb894f80c0b4c0d40b6582c92ae69852fda3c6a3a88c401845f138cccd1759`.
Its normalized `Cargo.toml` has no package metadata table.
The current repository manifest also lacks `[package.metadata.binstall]`.

The release workflow creates the namespaced tag in `.github/workflows/cargo-publish.yml:321`:

```sh
# .github/workflows/cargo-publish.yml:321
tag="forbidden-strings-v${VERSION}"
```

The read-only upstream checkout at `0c2d7a00afc5de6d5d2d5fc44922d2f6f993dda2`
defines the default GitHub paths at `crates/binstalk-fetchers/src/gh_crate_meta/hosting.rs:34`:

```rust
// crates/binstalk-fetchers/src/gh_crate_meta/hosting.rs:34
const GITHUB_RELEASE_PATHS: &[Template<'_>] = &[
    template!("{ repo }/releases/download/{ version }"),
    template!("{ repo }/releases/download/v{ version }"),
    // %2F is escaped form of '/'
    template!("{ repo }/releases/download/{ subcrate }%2F{ version }"),
    template!("{ repo }/releases/download/{ subcrate }%2Fv{ version }"),
];
```

These paths do not include the repository's `forbidden-strings-v0.5.0` tag.
The actual registry probe's debug log used only `0.5.0` and `v0.5.0` tag directories.
It already resolved the repository redirect to `Aquaticats/Monochromatic`,
so the observed remaining failure is distinct from stale owner metadata.

The disposable probe disabled token discovery,
removed `GH_TOKEN` and `GITHUB_TOKEN` from its child environment,
selected only `crate-meta-data`,
and used `--dry-run` with a private install destination.
Its registry case ran this invocation:

```sh
# Run from a private disposable directory; no binary installation occurs.
cargo binstall --dry-run --no-confirm --disable-telemetry --no-discover-github-token \
  --strategies crate-meta-data --install-path "$PWD/scanner-template-dry-run" \
  --targets x86_64-unknown-linux-gnu --log-level debug --maximum-resolution-timeout 30 \
  forbidden-strings@0.5.0
```

It exited `94` with `Fallback to cargo-install is disabled`.
The positive control used the same command and version with these overrides:

```text
# Additional arguments supplied before forbidden-strings@0.5.0.
--pkg-url https://github.com/Aquaticats/Monochromatic/releases/download/forbidden-strings-v{version}/forbidden-strings-{version}-{target}{archive-suffix}
--bin-dir {name}-{version}-{target}/{bin}{binary-ext}
--pkg-fmt tgz
```

The control exited `0` and reported:

```text
# cargo-binstall 1.25.3, explicit-layout dry-run
The package forbidden-strings v0.5.0 (x86_64-unknown-linux-gnu) has been downloaded from github.com
Dry-run: Not proceeding to install fetched binaries
```

The [retained evidence](../audit/forbidden-strings-binstall-2026-10-15.evidence.json)
contains both statuses,
the manifest checksum,
the observed default tag directories,
and selected output.
No binary was installed,
no crate was published,
and this dry-run did not validate runtime behavior or provenance verification.

The explicit layout is a verified discovery workaround for the tested Linux target.
Other targets and archive formats need their own verification.
A durable manifest repair must use the existing file-enforcer owner,
then ship through an authorized new package publication;
changing source cannot rewrite the published 0.5.0 manifest.
This remains a repository metadata issue under the existing upstream filing decision,
not a demonstrated upstream installer defect.
The unresolved scanner repair is tracked in
[repository issue 1031](https://github.com/Aquaticats/Monochromatic/issues/1031).

The user-supplied forbidden-strings CI failure invoked `gh release`,
not `cargo-binstall`.
Its authenticated quota and transferred-provenance failures are documented separately in
[the release-transfer record](github-release-transfer-quota.md).

The new evidence was committed as `a361d83d11` before document verification.
Pandoc rendering of the new section and issue body was inspected.
Scoped `monochromatic-lint` initially found existing semantic line-break errors in the older sections;
its supported `--fix` normalized those prose breaks.
The subsequent check returned success with Rust-documentation and anonymous-function warnings on quoted upstream code.
The source excerpts remain verbatim.
