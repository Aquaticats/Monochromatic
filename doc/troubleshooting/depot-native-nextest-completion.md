# Depot native tests with nextest 0.9.148 stop before complete coverage

## Symptom

Historical native replay `38842787230` reached macOS tests but reported:

```text
Summary [  24.953s] 55/1382 tests run: 54 passed, 1 failed, 1 skipped
```

Desktop replay `38843527788` similarly reported:

```text
Summary [  60.000s] 1020/1433 tests run: 1017 passed, 3 failed, 1 skipped
```

These are terminal failed commands with incomplete test execution.
They cannot establish every required test's outcome.
The distinct application failures remain in issues `#1062` through `#1065`.

## Root cause

The historical `package/music-player/desktop-app/mise.toml:166` task invokes:

```javascript
// package/music-player/desktop-app/mise.toml, historical source cb95cf6d78
runCargo(['nextest', 'run'])
```

Nextest release `cargo-nextest-0.9.148`,
source commit `cd1d6d5467322dbc0d39a80cb671d000ac302798`,
sets `fail-fast = true` in `nextest-runner/default-config.toml:58`.
The [current running-tests documentation](https://nexte.st/docs/running/#failing-fast)
confirms that the default stops scheduling new tests after failure.
The already running tests finish.

`cargo-nextest/src/dispatch/core/run.rs:261` applies the explicit override:

```rust
// nextest, cargo-nextest/src/dispatch/core/run.rs:261 at cargo-nextest-0.9.148
} else if self.no_fail_fast {
    builder.set_max_fail(MaxFail::from_fail_fast(false));
```

This is documented upstream behavior and a trial harness coverage omission.
The workflow matrix's `fail-fast: false` controls independent jobs;
it does not configure nextest inside a job.

## Verification

The retained failing catalog comprises the two macOS runs named in Symptom.
Windows native source `cb95cf6d78` failed test compilation before enumeration,
which is a different boundary tracked in issue `#1061`.

Trial commit `32a9250add` provides
`//package/monorepo-task/core-exec:trial:depot:native:nextest`
on branch `trial/depot-complete-scanner-20261016`.
It reads the selected historical task through `mise tasks info --json`,
reads its dispatcher through `mise config get`,
asserts that both replacement anchors occur exactly once,
and adds `--no-fail-fast` in memory.
It retains hashes of the original and adapted bodies.
The source checkout stays unchanged.

A disposable package control passed through the actual adapter.
Its dispatcher asserted the exact nextest arguments,
the package's environment variable,
and `SLINT_EMIT_DEBUG_INFO=1`.
The terminal native recovery runs are `38847322477` and `38847322559`.
Their independently reconciled results are retained in
[the native recovery evidence](../audit/depot-native-recovery-2026-10-17.evidence.json).

- Historical source `cb95cf6d785c3073c192b95f0632cdae7b910894` on macOS Intel:
  all 1382 eligible identities ran,
  with 1356 passed and 26 failed.
  The nextest summary reports one additional ignored case.
- Later source `3434eaa455d569c18b52cf15100f77f9db251d21` on macOS Intel:
  all 1433 eligible identities ran,
  with 1406 passed and 27 failed.
  The nextest summary reports one additional ignored case.
- Historical source on Windows:
  nextest test compilation failed with four `E0433` diagnostics for missing `libc`.
  No test identity ran.
  This remains the known historical source boundary in issue `#1061`,
  whose later-source fix is independently verified.

The reconciliation checks every numbered execution ordinal,
the denominator,
unique identity,
pass/fail status,
and complete nextest summary.
Nextest repeats failing records after its summary;
the retained evidence deduplicates only identical identity/status/ordinal records.
It retains every original failure output block and the provider log hash.
Each native job passed release build,
the real release CLI invocation,
and matching source/lockfile hash checks before and after execution.
All required test outcomes are accounted for on the native hosts,
but source assertions failed.

Provider job intervals were 2884 seconds for historical macOS,
827 seconds for historical Windows,
and 2015 seconds for later macOS.
The retained GitHub allocation evidence identifies four vCPUs for each standard public runner.
These intervals include setup and gate execution;
they are not nextest durations or isolated speed comparisons.

Linux recovery `pbg4h8ww6k`,
workflow revision `af3ccd254d717efd068ad55372c6eeba18fd8938`,
failed before nextest with Node.js v26.11.1 diagnostic
`Error: spawnSync slint-viewer ENOENT` for `slint-viewer --version`.
It produced no test identities or artifacts.
Its Depot attempt lasted 163.843 seconds on the workflow's `16x64` allocation.
This is a tool-discovery prerequisite failure,
not a source compilation failure or evidence that all Linux cases ran.
Corrected Linux recovery `xf4hhngdqg`,
workflow revision `409e6ec10600958a5eba45200944ce25eefe7749`,
passed the source-selected Slint viewer gate and ran every eligible test:

```text
Summary [  29.563s] 1460 tests run: 1460 passed, 1 skipped
```

Artifact `01a14827-9611-7704-bf87-fad55d354312` retains complete stdout,
stderr,
and source identity records.
Its SHA-256 is `28d345887f4f8f4c1336da7b2b9ce91362eb0b57d018b270b6f7427e4ed89e2b`.
Independent reconciliation found 1460 unique passed identities,
every numbered ordinal from 1 to 1460,
the exact 1460 denominator,
and no unexecuted eligible case.
All recorded source and input hashes match both before/after records
and the independently read immutable Git objects.
The original/adapted task body hashes match the later macOS source-task adapter.
The provider interval was 269.099 seconds on 16 vCPUs and 64 GB RAM,
or 4305.584 vCPU-seconds.
The adapter interval was 120.842 seconds;
the nextest summary reports 29.563 seconds.
These are different boundaries and do not establish an isolated speed comparison.

The fail-fast coverage omission now has complete execution evidence on macOS and Linux,
plus the genuine historical Windows source compile blocker.
This satisfies issue `#1073`'s recovery boundary;
the issue is closed with the retained execution evidence.
It does not establish passing cross-platform CI:
macOS assertions,
native Clippy diagnostics,
and historical Windows compilation remain separately recorded.
The missing Slint viewer wave in the initial Linux attempt is tracked separately in issue `#1108`;
the corrected tool gate and complete Linux test run verify that harness repair,
and the issue is closed.

## Verified workaround and tradeoff

The adapter's command and environment transformation passed the disposable control
and both macOS recovery sources reached all eligible test identities.
The supported invocation is `cargo nextest run --no-fail-fast`.
This schedules remaining tests despite earlier failures and therefore can consume more runner time.
It preserves failing exit status and default concurrency.
It does not repair application assertions or make an uncompilable test binary runnable.

## What does not work

- Treating a failed command as proof that every test ran loses unscheduled cases.
- Disabling workflow matrix fail-fast leaves nextest's internal policy unchanged.
- Treating `NEXTEST_PROFILE=ci` as a universal built-in profile is unsupported;
  custom profiles require configuration.

## Separate Clippy prerequisite incident

The same native replay reported this rustup diagnostic before Clippy could inspect code:

```text
error: 'cargo-clippy' is not installed for the toolchain 'nightly-2026-10-13-x86_64-apple-darwin'.
```

Windows reported the corresponding `cargo-clippy.exe` and Windows toolchain name.
The harness had installed Rust and cargo-nextest but had no explicit Clippy component step.
This proves a missing prerequisite at the consumer boundary;
it does not establish which mise or rustup selection branch omitted the component.
The [rustup component documentation](https://rust-lang.github.io/rustup/concepts/components.html)
identifies Clippy as a toolchain component.

Harness `32a9250add` records installed components,
runs `rustup component add clippy --toolchain nightly-2026-10-13`,
records them again,
and requires `cargo clippy --version` before package lint.
Run `38847322477` passed component installation and `cargo clippy --version`
on both native hosts;
the prerequisite incident is closed in issue `#1072`.
Clippy then emitted actual `clippy::implicit_return` source diagnostics:
seven sites at historical source on each native host,
and two sites in `src/launcher.rs` at later source on macOS.
Exact blocks are retained in the native recovery evidence;
the existing migration is tracked in issue `#372`.
The cross-platform nextest execution accounting is complete for issue `#1073`,
including the historical Windows source compile blocker.
The first Linux attempt's tool-discovery failure remains retained as development usage.
Installing the component adds setup work;
it cannot turn source Clippy diagnostics into passing lint.

## Upstream filing decision

1.  Upstream fault is not established:
    nextest implements its documented default,
    and the trial omitted an explicit Clippy prerequisite check.
2.  A consumer-side configuration remedy exists.
3.  Nextest documents the required full-run option;
    rustup documents component installation.
4.  Contribution policy assessment is unnecessary because no upstream change is proposed.
5.  The observed impact is incomplete trial coverage and failed lint setup.
6.  The consumer adapter control passed;
    macOS cloud recovery executed every eligible case,
    historical Windows reached its independently tracked source compile blocker,
    and corrected Linux recovery executed and passed all 1460 eligible tests.

The upstream nextest title search for `fail-fast` returned issues `#1955` and `#3624`,
which concern beta override behavior and stress-run exit status respectively.
Neither is the observed default-policy omission.
No upstream filing or comment is warranted,
and there is no additive upstream draft.
Repository issue tracking remains required for the trial defects.

## Coherent replay controls

The [prepared route report](../audit/depot-coherent-native-routes-2026-10-17.md)
separates immutable application source from trial harness revisions and shared replay provenance.
Original Windows file-redirection failure remains failed.
Independent merged-stream and uncaptured-console continuations retain their own results.
An uncaptured wrong-version control exited successfully but failed the expected-line check;
collectors must inspect that probe's actual provider output.

A first local control invocation passed `-NoProfile` as the image command.
The `crun` diagnostic was:

```text
# Initial disposable container invocation
Error: crun: executable file `-NoProfile` not found in $PATH: No such file or directory
```

Image inspection showed no entrypoint and a `pwsh-preview` default command.
Explicit `--entrypoint pwsh` corrected invocation.
The copied Node fixture then emitted `error while loading shared libraries: libatomic.so.1` and exited `127`.
Host `ldd` identified `/usr/lib64/libatomic.so.1`.
The verified workaround from [the fixture packaging diagnosis](llmgateway-systemone-boundaries.md)
supplied that installed library read-only and set the fixture library path.
Corrected positive and negative controls passed in the immutable image recorded in the route evidence.
Neither fixture packaging error is a music-player source failure.

The first generated Windows YAML also failed dprint with `invalid indentation`.
JavaScript replacement-string expansion interpreted the literal PowerShell `$` followed by a quote.
A function replacer preserved the literal command text;
targeted formatting and command-preservation checks passed.
The [rule clarification proposal](../planning/literal-replacement-text.md)
records the suggested tightening of `AGENTS.md` rule `SYB`.

## Common provenance must gate independent consumers

The held native-host workflow at `2032975cbad924e63ac98bc69e14a56b20288275`
placed canonical provenance before the source consumers,
but its independent build,
Clippy,
and nextest predicates used
`!cancelled() && steps.identity.outcome == 'success'`.
Smoke depended on build success.
No job or step declared `continue-on-error`.
The [GitHub status-function reference][native-status-functions] states that including a status function
overrides implicit `success()`.
A failed common-before therefore was not a named prerequisite in those expressions.
The historical execution consequence is an inference from configuration and documentation;
no rejected-cohort source execution was observed at the original revision.

Trial repair `8c1eac43273b0b0d468a54affd15dc9612d7fe49` adds
`steps.provenance-before.outcome == 'success'` to the four consumer conditions.
Original command bodies and the independent source-after identity condition remain unchanged.
Accepted,
failed,
skipped,
canceled,
and failed-build condition controls passed.
Removing the new predicates in an in-memory workflow again admitted the failed-common-before fixture.

Development GitHub run `39025557306` reached the actual Node diagnostic
`AssertionError [ERR_ASSERTION]: Final route requires a frozen complete case manifest`.
Build,
Clippy,
nextest,
and smoke were skipped;
ordinary source identity before and after succeeded with equal recorded inputs.
The separate artifact action emitted
`No files were found with the provided path` because the guard rejected before writing provenance.
Both failures and the full provider interval remain retained.
This control used source `cb95cf6d785c3073c192b95f0632cdae7b910894` on macOS Intel;
it gives no actual Windows/source3434 or accepted canonical-cohort execution credit.
The pilot differs from the held workflow only in its 20-minute job bound.

Repository issue [#1165][native-common-prerequisite]
is closed after the measured consumer dependency repair.
It differs from exact-byte transport issue `#1119`,
the logger guard predicate in `#1141`,
and expression-field scope issue `#1164`.
The [guard audit](../audit/depot-common-manifest-route-guards-2026-10-17.md)
retains the complete controls,
actual provider metadata,
diagnostics,
and original log ZIP.
Final coherent dispatch remains held.

### Upstream filing decision for provenance dependencies

- Fault is in the authored trial conditions;
  no GitHub service defect is established.
- A consumer configuration remedy exists and was exercised.
- GitHub explicitly documents the status-function behavior.
- Upstream contribution acceptance is not evaluated because no upstream change is proposed.
- No upstream change in direction is needed;
  the repair uses the documented step-outcome context.
- Local condition controls and actual provider rejection establish the trial remedy within the stated scope.

No upstream filing or comment is warranted.

[native-status-functions]: https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions
[native-common-prerequisite]: https://github.com/Aquaticats/Monochromatic/issues/1165

## Nextest archive extraction requires an existing explicit destination

### Symptom and deciding source

The frozen-source diagnostic producer `7sm49f324d` successfully created a Nextest archive,
then its attempted local replay exited `96` before test enumeration:

```text
# Diagnostic artifact member diagnostic-archive-local-lock-cohort.log
error: error extracting archive `.../diagnostic-native-nextest.tar.zst`
Caused by:
  error canonicalizing destination directory `.../.research-reuse/diagnostic-extract`
Caused by:
  No such file or directory (os error 2)
```

Nextest `0.9.148`,
source `cd1d6d5467322dbc0d39a80cb671d000ac302798`,
canonicalizes an explicit `--extract-to` directory in
`nextest-runner/src/reuse_build/unarchiver.rs:68`
before extraction.
The automatic temporary-directory branch creates its destination;
the explicit destination branch does not.
The research harness omitted creating that directory.
This is a harness prerequisite failure,
not a failing application assertion or a corrupt archive.

### Verified recovery and transport limits

The exact diagnostic ZIP and every member passed the provider-upload digest and ZIP CRC checks.
The Nextest archive is 17,046,905 bytes,
SHA-256 `d6b7edef092cad3c81a88deb293777665e4f38e5b9caf27221a5225c4965d596`.
The recovery independently verified all 1,079 source-file hashes,
the bundled Nextest executable and the separately bundled Git `2.56.0` executable.
It used a fresh,
existing extraction directory and remapped the workspace to `/work/package/git-policy/cli`.
The actual archived lock/metadata cohort then reported:

```text
# diagnostic-local-as-owner.log, local read-only Podman transport control
Summary [   0.173s] 34 tests run: 34 passed, 1922 skipped
```

This local control had no network,
two CPUs,
2 GiB memory,
read-only source/tool mounts and tmpfs fixture/extraction paths.
The existing native-test image declares `USER 1000:1000`;
that user initially failed to traverse the host-owner-only `0700` fixture directory.
Image inspection and file-mode inspection preceded the corrected rootless-container `--user 0:0` invocation.
The verified archive and source bytes remained unchanged.
The generic user-namespace boundary is documented separately in
[the Podman identity note](depot-rootless-podman-group-identity.md).

The [Nextest archiving documentation](https://nexte.st/docs/ci-features/archiving/)
requires matching source and recommends matching Nextest versions.
`--workspace-remap` changes runtime workspace paths,
but cannot rewrite Rust `env!` values compiled into an executable.
The frozen Git-config tests contain such a value;
the filtered lock/metadata cohort is therefore the exercised portability boundary.
This does not establish general relocation of every native test,
a matched cloud-filesystem cause,
or a successful native target-cache publication.

### Upstream filing decision for extraction

The source explains the observed prerequisite and the corrected consumer executes the archive.
No upstream defect is established,
and no upstream change or contribution acceptance is being assumed.
Current open/closed issue searches for `"extract-to"` returned `#1036` and `#2463`,
concerning coverage and a Cargo-related hang respectively.
Their bodies do not establish this destination-directory incident.
No upstream filing was sent.
Exact artifact identities,
source/tool hashes and both failed and successful consumer receipts are retained in
[the reuse evidence](../audit/depot-reuse-strategy-2026-10-18.evidence.json).
