# Cargo 1.96.1 can reuse stale artifacts after BuildKit restores older source timestamps

## Symptom

The Depot runner sizing fixture changed the application CLI name,
but the second round exported the original executable.
Its runtime assertion rejected the result:

```text
# trial:github:sizing, run 38819692881
Error: Expected runtime version music-player-sizing 0.1.4, received music-player 0.1.4
```

This was a failure of the trial's cache composition.
It was not a successful subsecond source rebuild.
The first round rebuilt correctly;
the later round reused the same changed-source layer with a separately populated Cargo target cache.

## Root cause

The failing build log records cached `COPY` steps,
then executes Cargo against `sizing-16-16-2-target`.
Cargo reports a finished release build in 0.28 seconds without compiling the changed application.
The exported runtime output is unchanged.
This excludes a skipped Cargo command:
the command ran and accepted stale local artifacts.

The installed tool is Cargo 1.96.1,
commit `356927216a2d746168cf76e5e88cc3f4b58e029d`,
from Alpine's Rust 1.96.1-r0 package.
The matching source was fetched from `rust-lang/cargo` and inspected.
In `src/cargo/core/compiler/fingerprint/mod.rs:2063`,
`find_stale_file` reads the reference timestamp.
At line 2179,
the timestamp-based path accepts source timestamps no newer than that reference:

```rust
//! `src/cargo/core/compiler/fingerprint/mod.rs:2179`.
if path_mtime <= reference_mtime {
    continue;
}
```

The same file's `LocalFingerprint::CheckDepInfo` handling at line 944 calls this freshness machinery.
The source documentation at line 231 identifies source files and dep-info as the compared inputs.
An older source timestamp therefore does not establish unchanged content.
Our trial combined immutable source layers with a mutable target cache without accounting for this distinction.

The minimal reproduction uses no BuildKit or exported artifact directory.
It restores the changed file with its old timestamp and directly executes Cargo's binary.
That produces the same stale result,
excluding export-directory reuse and cross-runner cache collisions as necessary causes.

## Verification

The local reproduction used Rust 1.96.1,
commit `31fca3adb283cc9dfd56b49cdee9a96eb9c96ffd`,
inside the existing `localhost/music-player-alpine-build:latest` image.
The image is produced by the desktop package's `test:alpine` task.
The container had two CPUs,
2 GiB memory,
no network,
and no host mounts.

The executable output was:

```text
# cargo-cache-repro task output
first change: changed source
second change with old mtime: Hello, world!
package invalidation: changed source
```

The changed source timestamp was `1792105608`;
the second target's binary timestamp was `1792105610`.
The stale build reported 0.00 seconds.
Cleaning only the generated `cache-probe` package made it rebuild and return the changed string.

The runnable fixture is preserved in the trial branch under
`package/music-player/desktop-app/depot-trial/cache-repro/mise.toml`.
From that directory,
run `mise run reproduce` after the production Alpine build image exists.
The fixture checks the original program,
the first changed program,
the deliberately stale negative control,
and the corrected program.
It removes its disposable container in cleanup.
The committed fixture at trial revision `67f7340dd5` passed on rerun.
Its initial container startup failed with local task-allocation pressure;
that attempt provided no cache verification evidence.

Working patterns are a newly timestamped source change and a package clean followed by a rebuild.
The failing pattern is an older changed source snapshot paired with a newer artifact for different source content.
The full application rerun subsequently passed in both corrected workflows.
The local fixture establishes the stale-timestamp mechanism;
the repeated application builds verify the BuildKit workaround at the consumer boundary.

## Verified workaround and application rerun

Package-scoped `cargo clean` fixes the minimal reproduction.
Trial commit `4648e5ad3d` applies that mechanism conditionally:
hash every copied source file,
compare the aggregate hash with the target cache's recorded hash,
and clean `music-player` plus `truepeak-core` when they differ.
Record the new hash only after compilation succeeds.

```dockerfile
# package/music-player/desktop-app/depot-trial/Containerfile, trial branch
RUN find /src -type f -exec sha256sum {} + | sort | sha256sum > /out/source-checksum.txt
```

The compile step compares that file with `/target/.trial-source-checksum`
and invokes `cargo clean --release --package music-player --package truepeak-core` on a mismatch.
Both packages rebuild conservatively when any copied content changes,
including documentation.
Registry dependency artifacts survive.
Unchanged content retains the local package artifacts.
The package list must expand if another local path dependency is introduced.

Runs `38822116503` and `38822116510` tested this change with the original repeated source-change assertion.
They use the fixed trial source and separately populated caches for each round.
All ten configurations passed all ten outputs:
100 checked artifacts,
including two source changes per configuration and their unchanged repetitions.
Both baseline outputs and the deliberately changed CLI name matched in every round.
This verifies the full workaround for the tested application inputs and local package set.
[Main execution](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116503),
[large-runner execution](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116510),
[measurements and limitations](../audit/depot-runner-sizing-2026-10-15.md)

## What does not work

- Changing the trial label forces the Docker `RUN` step,
  but does not force Cargo to rebuild a package it considers current.
- Separate target-cache namespaces per round did not prevent the stale result.
  Each namespace can still encounter a source layer older than its own artifacts.
- Checking only `cargo build` exit status misses the failure.
  Running the artifact and asserting its changed behavior detected it.
- A successful first source change did not establish repeated-snapshot correctness.

The [Cargo checksum-freshness tracking issue](https://github.com/rust-lang/cargo/issues/14136)
documents an upstream content-based alternative still requiring a nightly toolchain in its current testing instructions.
Its build-script caveat prevents treating that feature as an unqualified replacement for this consumer workaround.
The trial keeps the production Alpine toolchain.

## Upstream filing decision

The `.out-of-scope/` inventory has no exemption for timestamp freshness;
`cargo-workspace.md` concerns a separate workspace request.
Tracker searches for checksum freshness found Cargo issue 14136,
whose discussion already covers stale artifacts under fixed timestamps and content-aware remedies.
There is nothing additive to file from this trial.

1.   Fault attribution:
     the demonstrated error is our cache composition relying on an invalid timestamp assumption.
     Cargo's documented timestamp behavior explains it.
2.   Fixability:
     upstream already has a content-based implementation under development.
3.   Supported scope:
     ordinary timestamp tracking does not promise content identity after arbitrary timestamp restoration.
4.   Contribution policy:
     no upstream submission is proposed,
     so contribution-policy eligibility was not established.
5.   Maintainer direction:
     issue 14136 requests testing and names unresolved build-script and performance concerns.
6.   Prototype:
     the consumer invalidation prototype was exercised locally;
     no new Cargo patch is necessary to demonstrate this integration failure.
