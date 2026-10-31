# mise 2026.10.9 refuses to install `cargo:cargo-ndk`
until `rust@nightly` is installed through mise,
failing a CI job that has Rust only from rustup

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

## Symptom

The Android job of `.github/workflows/music-player-release.yml` installed its tools with a single call:

```bash
mise install cargo:cargo-ndk node java android-sdk
```

On a GitHub-hosted `ubuntu-24.04` runner where mise had installed no Rust,
the job failed in 3 ms with this output (dry run 38657771731,
mise `2026.10.9 linux-x64 (2026-10-13)`):

```text
mise ✗ cargo:cargo-ndk@4.1.2  2ms · failed: tool 'cargo:cargo-ndk@latest' requires configured install dependency 'rust@nightly', but its selected version is not installed
mise ERROR Failed to install cargo:cargo-ndk@latest: tool 'cargo:cargo-ndk@latest' requires configured install dependency 'rust@nightly', but its selected version is not installed
hint: Run `mise install rust@nightly` before installing 'cargo:cargo-ndk@latest'. Remove the dependency from configuration to allow the install hook to rely on system PATH instead.
##[error]Process completed with exit code 1.
```

The root `mise.toml` declares `rust = { version = "nightly", ... }` and `mise.lock` pins it to a dated nightly,
so the request is satisfied on paper but the toolchain is not on disk in a fresh runner.

## Root cause

The cargo backend declares Rust as an install dependency of every `cargo:` tool.
`src/backend/cargo.rs:168` of the mise `v2026.10.9` tag (commit `ac91d06fa6100935b7a07b08bebf03dd6dd9a0dd`):

```rust
fn get_dependencies(&self) -> eyre::Result<Vec<&str>> {
    Ok(vec!["rust"])
}
```

`InstallDependencyContext::resolve` (`src/install_context.rs:119-165`) matches that dependency against the tools
configured in the repository,
resolves them offline,
and refuses to continue when the selected version is not installed:

```rust
for (backend, dependency) in toolset.list_current_versions() {
    if !backend.is_version_installed(config, &dependency, true) {
        // ... unresolved-version branch elided
        bail!(
            "tool '{}' requires configured install dependency '{}', but its selected version is not installed\n\
             hint: Run `mise install {}` before installing '{}'. Remove the dependency from configuration to allow the install hook to rely on system PATH instead.",
```

So the failure is a deliberate precondition,
with a hint that names the fix.
It matches the dependency against the tools the repository configures,
and this repository configures `rust`.
The check asks whether that configured tool's selected version is installed by mise.
The message's last sentence says the system `PATH` is used only after the dependency is removed from configuration,
so a rustup toolchain already on the runner is not what the check looks for.
`is_version_installed` itself was not read.

The workflow also installs `cargo-binstall` in the first command because `cargo:` tools resolve through it
(see the comment on `cargo-binstall` in the root `mise.toml`).
The October 16 trial separately exercised the missing `cargo-binstall` precondition,
as recorded in the native trial section.

## Verification

Version under test:
mise `2026.10.9` (`linux-x64`,
built 2026-10-13),
through `jdx/mise-action` with `install: false`.
The tag `v2026.10.9` was cloned shallowly to read the two files above.

Failing:
run 38657771731,
job `build-android`,
the call shown in the symptom.

Working,
run 38658350643 and every later dry run (job `build-android` green,
APK built):

```bash
mise install rust cargo-binstall
mise install cargo:cargo-ndk node java android-sdk
mise exec rust -- rustc -Vv
```

The log of run 38669432843 prints `rustc 1.101.0-nightly (e9bbadf68 2026-10-12)`,
which is the `nightly-2026-10-13` of `mise.lock`,
so the toolchain mise installed is the repository's pinned one.

Not tested:
a version of mise other than `2026.10.9`,
and removing `rust` from the configuration to use the system
Rust (the hint's second suggestion),
which would change the repository's toolchain pin.

## Verified workarounds

Current toolchain:
 `mise run toolchain:install` stages `bin`,
 `uv`,
 `mpm`,
 `rust`,
 and `cargo-binstall` before the tools that depend on them
 (`doc/planning/mpm-toolchain-migration.md`,
 sections `Sync algorithm` and `Implementation record (toolchain-core, 2026-10-26)`),
 so the ordering this record establishes is now built into the sync.

Install Rust (and `cargo-binstall`) through mise before any `cargo:` tool,
as shown above.
Tradeoff:
the job downloads the pinned nightly through mise (about 16 s in run 38669432843),
in addition to any rustup toolchain the runner already has,
and the pinned date must stay available from Rust's servers.

## What does not work

- Installing the `cargo:` tool with no Rust installed through mise:
  the failure above,
  with the runner's own tools
  as they were.
- Not tested:
  naming `rust` in the same `mise install` command as the `cargo:` tool.

## Upstream filing decision

`.out-of-scope/` was checked:
no exemption names mise's install dependencies.
A search of `jdx/mise` issues and pull requests for `requires configured install dependency`,
`cargo backend install dependency rust not installed` and `install dependency selected version is not installed`
returned no matches.

1.  Is it upstream's fault:
    no. The message states the precondition and the fix.
2.  Can upstream fix it:
    not applicable,
    there is no defect.
3.  Supported use case:
    installing a `cargo:` tool needs Rust,
    which the hint says.
4.  Contribution welcome:
    not assessed,
    nothing to contribute.
5.  Likely to be fixed:
    not applicable.
6.  Prototype:
    not applicable.

Nothing is filed and no draft is kept:
the behavior is intended and documented by its own diagnostic.

## Native trial also needs the configured optional Cargo installer

GitHub Actions run [38842242564][native-prerequisite-failure] used mise `2026.10.9`
on `windows-2025` and `macos-15-intel`.
Both jobs successfully installed Node `26.11.1` and Rust `nightly-2026-10-13`,
then failed before package checks when installing `cargo:cargo-nextest@0.9.148`:

```text
tool 'cargo:cargo-nextest@0.9.148' requires configured install dependency 'cargo-binstall@latest', but its selected version is not installed
```

This is a distinct prerequisite from Rust.
The trial adapter had omitted `cargo-binstall` from the documented preparation command.
The source revision `cb95cf6d785c3073c192b95f0632cdae7b910894` declares `cargo-binstall = "latest"`
and locks its resolved version to `1.25.3`.

In the same mise source commit,
`src/backend/cargo.rs:172` declares the additional dependency:

```rust
// src/backend/cargo.rs:172
fn get_optional_dependencies(&self) -> eyre::Result<Vec<&str>> {
    Ok(vec!["cargo-binstall", "sccache"])
}
```

Optional backend dependencies still participate when the repository configures them.
`src/install_context.rs:61` selects configured requests matching those declarations:

```rust
// src/install_context.rs:61
fn matching_requests(&self, requests: &ToolRequestSet) -> ToolRequestSet {
    requests
        .iter()
        .filter(|(ba, ..)| self.matches(ba))
        .map(|(ba, requests, source)| (ba.clone(), requests.clone(), source.clone()))
        .collect()
}
```

Those requests reach the installed-version guard at `src/install_context.rs:136`,
which emits the diagnostic quoted in the root cause section.
Installing Rust alone therefore satisfies only the Rust prerequisite.

Trial commit `87f5bf2c1f` corrects the preparation sequence in the disposable harness checkout:

```bash
# Trial harness checkout, before immutable source checks.
mise install --yes node@26.11.1 rust@nightly-2026-10-13 cargo-binstall@1.25.3
mise install --yes cargo:cargo-nextest@0.9.148 cmake@4.4.4
```

[Retry 38842787230][native-prerequisite-recovery] at `87f5bf2c1f7f4e132bd6ad12b26cd6f45f06de22`
verified the explicit-version prerequisite sequence on both hosts.
macOS job `116786450062` and Windows job `116786450596` completed
`Install the source-declared Node and Cargo tool prerequisites`,
`Install source-declared native tools after Rust`,
and `Verify immutable source and record tools` successfully.
Both then reached native package checks.
The [retained accounting evidence](../audit/depot-complete-ci-accounting-2026-10-16.evidence.json)
preserves the successful setup steps and the original failed attempts.
[Issue #1059](https://github.com/Aquaticats/Monochromatic/issues/1059) is closed for this prerequisite omission.

This verifies setup only.
The overall retry was still running at that observation boundary,
and its Windows Clippy step had a separate failure.
Neither the setup repair nor the issue closure establishes complete native validation.
Its tradeoff is an additional installer download,
with all versions kept equal to the historical source lockfile.

## Expanded Linux graph repeats the configured installer omission

GitHub Actions run [39135886118][expanded-prerequisite-failure] at harness
`815591bf307b1237d31b1c4e644463d7c13654fa`
used unchanged application cohort `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Both standard GitHub jobs and both Depot GitHub runner jobs passed checkout,
resource recording and mise installation,
then failed this setup command:

```bash
# Disposable expanded-graph research workflow.
mise install --locked node pnpm rust cargo:cargo-nextest
```

The exact emitting tool was mise `2026.10.9 linux-x64 (2026-10-13)`.
All four failures named the same missing configured dependency:

```text
tool 'cargo:cargo-nextest@latest' requires configured install dependency 'cargo-binstall@latest', but its selected version is not installed
```

The log resolves Nextest to `0.9.148`.
The omitted installer matches the optional-dependency source path and historical verified prerequisite
documented in the native trial section.
No application build,
test,
browser owner or graph campaign ran.
These failures establish a bootstrap omission on both providers;
they establish no workload timing comparison.

Correction `fdca95cc72` stages the configured prerequisites without changing application source or owning tasks:

```bash
# Disposable expanded-graph research workflow, within the existing setup time limit.
mise install --locked node pnpm rust cargo-binstall
mise install --locked cargo:cargo-nextest
```

The additional installer download is part of measured setup.
Independent local preflight used the exact frozen configuration,
fresh tool/home/cache state and anonymous public downloads in a two-CPU,
two-GiB container.
The official mise executable's SHA-256 is
`6ab6646b52b83f6bbd51c9d0d3ae54264ffa90efd1ac6d7c0ec5a86bbc6a0030`.
The prerequisite command passed in 35,568 ms;
the separate Nextest install passed in 6,487 ms.
Actual binary probes reported Node `26.12.0`,
pnpm `12.14.0` and Nextest `0.9.148`.
These are single local setup observations,
not a matched cloud speed comparison or application-test execution.
The corrected immutable Linux attempts subsequently passed bootstrap and reached the actual graph.
GitHub Actions run `39140975786` uses harness `234ccc92dabb81e998e0a0bb5e838713924dd83f`;
the [original repeated job evidence](../audit/depot-expanded-graph-github-repetition-2026-10-18.evidence.json)
retains all four completed jobs and their application commands.
Native run `fnvwqfg0bb` uses harness `72af18d5b130dabef7ff5b656ffdc1b58df4fcde`;
the [native correction evidence](../audit/depot-expanded-graph-native-correction-2026-10-18.evidence.json)
retains both corrected complete 222-command observations.
These later runs verify bootstrap recovery;
their separate application failures do not establish a passing whole CI graph.
Historical Mac/Windows setup is not substituted for this Linux cohort's result.
The upstream filing decision remains unchanged:
this fixture repeated an already documented repository prerequisite,
so there is no new upstream defect or additive filing.

[expanded-prerequisite-failure]: https://github.com/Aquaticats/Monochromatic/actions/runs/39135886118

[native-prerequisite-failure]: https://github.com/Aquaticats/Monochromatic/actions/runs/38842242564
[native-prerequisite-recovery]: https://github.com/Aquaticats/Monochromatic/actions/runs/38842787230
