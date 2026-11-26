# dylint_testing 6.4.0 cannot build its driver under a non-rustup `cargo`: `environment variable not found: RUSTUP_TOOLCHAIN`

Evidence date 2026-11-25,
host with `cargo-dylint` and `dylint-link` 6.4.0 (`toolchain.lock.json`,
the `cargo:cargo-dylint` and `cargo:dylint-link` rows),
`dylint_testing` and `dylint_linting` 6.4.0,
and `rustc 1.102.0-nightly (5f4effd57 2026-11-13)` from `nightly-2026-11-16`,
running the UI tests of `package/linter/monochromatic-lint-closure`.
Upstream tag `v6.4.0` is commit `703f588db3e50ca3519bc9abea2eac2a9308a70d`;
every citation of upstream source in this file is pinned to that commit.

## Symptom

The first `cargo test` of a Dylint library on this host fails in the test
`ui_tests::ui`,
which builds Dylint's driver before it compiles any fixture.
The unit tests around it pass.
Two different error strings appear,
depending on whether the toolchain name reaches the driver build at all.

With no `RUSTUP_TOOLCHAIN` in the child environment:

```text
error: failed to run custom build command for `dylint_driver v6.4.0`
Caused by:
  process didn't exit successfully: `.../build/dylint_driver/.../out/build_script_main` (exit status: 1)
  --- stderr
  Error: environment variable not found: RUSTUP_TOOLCHAIN
```

With the short toolchain name `nightly-2026-11-16` in the child environment:

```text
Error: Could not parse toolchain: nightly-2026-11-16
```

## Root cause

`dylint_testing::ui::Test::run` calls `initialize`,
which asks Dylint's driver builder for a driver and builds one when none is current.
The build writes a throwaway package whose `rust-toolchain.toml` names the toolchain,
then runs `cargo build` in it with the environment sanitized:
[dylint/src/driver_builder/mod.rs:140-168](https://github.com/trailofbits/dylint/blob/703f588db3e50ca3519bc9abea2eac2a9308a70d/dylint/src/driver_builder/mod.rs#L140-L168).

```rust
// dylint/src/driver_builder/mod.rs (tag v6.4.0)
    dylint_internal::cargo::build(&format!("driver for toolchain `{toolchain}`"))
        .quiet(opts.quiet)
        .build()
        .sanitize_environment()
        .envs([(env::RUSTFLAGS, rustflags)])
        .current_dir(package)
        .success()?;
```

Sanitizing means removing `CARGO`,
`RUSTC` and `RUSTUP_TOOLCHAIN`:
[internal/src/rustup.rs:15-21](https://github.com/trailofbits/dylint/blob/703f588db3e50ca3519bc9abea2eac2a9308a70d/internal/src/rustup.rs#L15-L21).

```rust
// internal/src/rustup.rs (tag v6.4.0)
impl SanitizeEnvironment for Command {
    fn sanitize_environment(&mut self) -> &mut Self {
        self.env_remove(env::CARGO);
        self.env_remove(env::RUSTC);
        self.env_remove(env::RUSTUP_TOOLCHAIN);
        self
    }
}
```

The throwaway package's `rust-toolchain.toml` is honored by rustup,
not by cargo,
so the design assumes the `cargo` in `PATH` is a rustup proxy:
the proxy reads that file and sets `RUSTUP_TOOLCHAIN` again for the build.

`dylint_driver`'s build script needs that variable:
[driver/build/extra_symbols.rs:24-53](https://github.com/trailofbits/dylint/blob/703f588db3e50ca3519bc9abea2eac2a9308a70d/driver/build/extra_symbols.rs#L24-L53).

```rust
// driver/build/extra_symbols.rs (tag v6.4.0)
    let rustup_toolchain = env::var(env::RUSTUP_TOOLCHAIN)?;
    ...
fn clippy_utils_has_extra_symbols(rustup_toolchain: &str) -> Result<Option<String>> {
    let (channel, _) = parse_toolchain(rustup_toolchain)
        .ok_or_else(|| anyhow!("Could not parse toolchain: {rustup_toolchain}"))?;
```

`parse_toolchain` splits the name on `-` and finds the architecture by looking for a known target component:
[internal/src/rustup.rs:81-92](https://github.com/trailofbits/dylint/blob/703f588db3e50ca3519bc9abea2eac2a9308a70d/internal/src/rustup.rs#L81-L92).

```rust
// internal/src/rustup.rs (tag v6.4.0)
pub fn parse_toolchain(toolchain: &str) -> Option<(String, String)> {
    let split_toolchain: Vec<_> = toolchain.split('-').collect();
    split_toolchain
        .iter()
        .rposition(|s| ARCHITECTURES.binary_search(s).is_ok())
        .map(|i| {
            (
                split_toolchain[..i].join("-"),
                split_toolchain[i..].join("-"),
            )
        })
}
```

A name without a target component,
such as the short `nightly-2026-11-16`,
returns `None`,
which is the second error string.

This repository's session `cargo` is a shadow wrapper built by the repository,
not a rustup proxy,
and this host has no rustup proxy in `PATH` at all.
The child therefore runs with no `RUSTUP_TOOLCHAIN`,
and when the variable is supplied from outside,
Dylint removes it before the child starts.

## Verification

Versions under test:
`dylint_testing` 6.4.0,
`dylint_linting` 6.4.0,
`cargo-dylint` 6.4.0 and `dylint-link` 6.4.0;
toolchain `nightly-2026-11-16`;
upstream commit `703f588db3e50ca3519bc9abea2eac2a9308a70d`.

Failing catalog (the library and its unit tests build,
the UI test does not):

- Plain `cargo test --locked --offline --all-targets` in the library directory with
  `RUSTUP_TOOLCHAIN=nightly-2026-11-16` and no driver:
  the child build stops with
  `Error: environment variable not found: RUSTUP_TOOLCHAIN`.
- The same with the driver built by a `cargo build` that was given
  `RUSTUP_TOOLCHAIN=nightly-2026-11-16` instead of the full triple:
  it stops with
  `Error: Could not parse toolchain: nightly-2026-11-16`.

Working catalog (the driver is built once outside Dylint,
then reused):

- A throwaway package holding Dylint's own driver manifest and `main.rs`
  (`anyhow`,
  `env_logger`,
  `dylint_driver = "=6.4.0"`,
  and the `MAIN_RS` constant of
  [dylint/src/driver_builder/mod.rs:29-42](https://github.com/trailofbits/dylint/blob/703f588db3e50ca3519bc9abea2eac2a9308a70d/dylint/src/driver_builder/mod.rs#L29-L42))
  is built with `RUSTUP_TOOLCHAIN=nightly-2026-11-16-x86_64-unknown-linux-gnu`,
  so the build script of `dylint_driver` sees a parseable full name.
- The resulting binary reports its version,
  which is what Dylint reads to decide whether a driver is current:

```text
$ dylint-driver -V
nightly-2026-11-16-x86_64-unknown-linux-gnu 6.4.0
```

- `DYLINT_DRIVER_PATH=<driver-root>` pointing at a directory that holds
  `<driver-root>/nightly-2026-11-16/dylint-driver`
  makes Dylint reuse it,
  because the printed version is not lower than `cargo-dylint`'s.
  The directory segment is the string that `dylint_testing` was compiled with
  in `RUSTUP_TOOLCHAIN`,
  which is the short `nightly-2026-11-16` when the test
  was compiled with that value.
- With that driver in place,
  the same `cargo test --locked --offline --all-targets` passes:
  the UI suite reports `test result: ok. 9 passed; 0 failed`,
  and the library suite reports
  `test result: ok. 24 passed; 0 failed`.

## Verified workarounds

### Build the driver once and reuse it through `DYLINT_DRIVER_PATH`

Run one build of Dylint's driver package with the full toolchain triple,
then point the test run at it:

```sh
# in a throwaway package that holds Dylint's driver Cargo.toml and src/main.rs
RUSTUP_TOOLCHAIN=nightly-2026-11-16-x86_64-unknown-linux-gnu cargo build
# then place the binary where Dylint looks for it
install -m 755 target/debug/dylint_driver-nightly-2026-11-16 <driver-root>/nightly-2026-11-16/dylint-driver
DYLINT_DRIVER_PATH=<driver-root> RUSTUP_TOOLCHAIN=nightly-2026-11-16 cargo test --locked --offline --all-targets
```

Tradeoffs:
the driver is now owned by whoever built it,
so a `cargo-dylint` upgrade makes it stale and Dylint rebuilds it (and fails the same way);
the two `RUSTUP_TOOLCHAIN` values differ for a reason
(the full triple satisfies the driver's build script,
the short name is what the test binary baked in),
and a test compiled with a different string needs the driver under that segment;
and the extra package is scratch,
so a fresh machine repeats the step.

### Run the tests through a rustup-proxy `cargo`

Put a rustup proxy first in `PATH` for the test run.
The proxy reads the throwaway package's `rust-toolchain.toml` and restores `RUSTUP_TOOLCHAIN` for the driver build.

Not run here:
this host installs no rustup proxies at all,
so the arm could not be exercised.

## What does not work

- Exporting `RUSTUP_TOOLCHAIN` for the outer test command only:
  Dylint sanitizes it away before the driver build starts.
- Passing the short toolchain name to the driver build:
  `parse_toolchain` needs a target component,
  and the build stops with
  `Could not parse toolchain: nightly-2026-11-16`.
- Relying on the throwaway package's `rust-toolchain.toml` alone:
  only rustup honors that file,
  and the repository's `cargo` is a wrapper.
- Pointing `DYLINT_DRIVER_PATH` at an empty directory:
  Dylint builds the driver when none is current,
  which fails as described.

## Upstream filing decision

Nothing to file;
the failure needs this repository's non-rustup `cargo`,
and Dylint's behavior is deliberate.

- Constraint 1 (really upstream's fault):
  no.
  Removing the variables is how Dylint lets the throwaway package's `rust-toolchain.toml` decide the toolchain,
  and a rustup proxy restores what the build script needs.
  The trigger is this host's `cargo` wrapper.
- Constraint 2 (can upstream fix it):
  it could,
  by resolving the toolchain itself instead of trusting the child `cargo`,
  so this constraint is not the blocker.
- Constraint 3 (do they support the use case):
  their docs assume rustup,
  so a wrapper `cargo` is not a documented case.
- Constraint 4 (would they welcome a contribution):
  `CONTRIBUTING.md` at the tag mentions no ban on AI-assisted work,
  no `VOUCHED` file exists,
  and `.github/` holds `dependabot.yml` and workflows only.
- Constraint 5 (will they fix it):
  no signal either way;
  the newest commit touching `internal/src/rustup.rs` before the release is
  `80b1bc6c0` (2026-11-06).
- Constraint 6 (prototyped fix):
  not applicable,
  because constraint 1 fails.

Duplicate and work-in-progress searches on 2026-11-25,
all with no matching result:
`gh search issues --repo trailofbits/dylint "RUSTUP_TOOLCHAIN driver"`,
`gh search issues --repo trailofbits/dylint "driver build RUSTUP_TOOLCHAIN"`,
`gh search issues --repo trailofbits/dylint "rustup proxy cargo"`,
`gh search prs --repo trailofbits/dylint "driver build environment"`.
`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md` covers the neighbouring
offline driver-pin failure and does not cover this one.

## Related

- `doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`
  is the other way the driver step fails:
  an offline registry without the pinned `dylint_driver` version.
- `doc/troubleshooting/dylint-library-tests-and-shared-build-dir.md`
  records the library and fixture-side setup of the same package's tests.
