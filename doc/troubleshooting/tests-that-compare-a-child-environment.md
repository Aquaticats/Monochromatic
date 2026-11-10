# A test that pins a child's whole environment depends on the harness that started it

## Symptom

`mise run //package/linter/monochromatic-lint:test` failed one control and skipped the rest of the run:

```text
FAIL [   0.110s] ( 19/949) monochromatic-lint closure_cargo::tests::the_dylint_run_has_the_documented_arguments_and_environment
    thread 'closure_cargo::tests::the_dylint_run_has_the_documented_arguments_and_environment' (2836134) panicked at src/closure_cargo_tests.rs:210:5:
    assertion `left == right` failed
      left: [ ... CARGO_NET_OFFLINE=true CARGO_BUILD_BUILD_DIR=/var/mnt/encrypted/cargo-build/toolchain/nightly-2026-11-08/83/dbc73fab259949 STUB_MARKER=a value with spaces]
     right: [ ... CARGO_NET_OFFLINE=true STUB_MARKER=a value with spaces]
    Summary [   0.944s] 21/949 tests run: 20 passed, 1 failed, 3 skipped
warning: 928/949 tests were not run due to test failure
```

The same control passes when its binary is started with that one variable removed,
and a shell outside mise has neither `RUSTUP_TOOLCHAIN` nor `CARGO_BUILD_BUILD_DIR`
(`printenv RUSTUP_TOOLCHAIN` and `printenv CARGO_BUILD_BUILD_DIR` both exit 1 there).
The variable is neither a fixture nor a constant of the package:
it is set by the harness,
above cargo.

## Root cause

Three configuration entries,
none of them in the test or in the linter,
put the variable in the test process:

1.  `toolchain.lock.json` line 485 gives the rust tool the environment entry
    `"RUSTUP_TOOLCHAIN": "nightly-2026-11-08"`,
    and mise exports it for every task.
    `mise exec -- printenv RUSTUP_TOOLCHAIN` answers `nightly-2026-11-08`;
    a shell outside mise has no value.
2.  `$CARGO_HOME/config.toml` sets `[build]` `build-dir` to
    `"/var/mnt/encrypted/cargo-build/{workspace-path-hash}"`,
    the shared build folder of [the host cache decision][build-dir-host-cache].
3.  Every `cargo` mise runs is the launcher `cargo-shadow`.
    When its caller set no `CARGO_BUILD_BUILD_DIR`,
    when that template ends in `{workspace-path-hash}`,
    and when a toolchain is known from a leading `+name` or `RUSTUP_TOOLCHAIN`,
    it sets `CARGO_BUILD_BUILD_DIR` to the toolchain-keyed folder
    (`package/cargo-shadow/cli/src/lean_keyed_folder.rs` line 209,
    called from `package/cargo-shadow/cli/src/shim.rs` line 169;
    mechanism A of `package/cargo-shadow/cli/README.md`).

The launcher sets the variable for the `cargo` child it starts.
Cargo then starts `cargo nextest`,
nextest starts the test binary,
and `std::process::Command` passes the environment on to every further child,
so the stub `cargo` of the control logs the variable with the value the launcher chose.

A control that writes the expected environment as one fixed string can only pass
when the program scrubs the environment it inherits,
which no program can do here:
`PATH`,
`HOME`,
`CARGO_HOME` and the toolchain variables must reach the child,
and the linter deliberately adds `CARGO_BUILD_BUILD_DIR` to it for the IDE package
(`package/linter/monochromatic-lint/src/closure_probe_platform.rs`).
The expectation was the wrong side,
not the program.

## Verified resolution

The control now states the contract it can hold:
the run sets `DYLINT_RUSTFLAGS` and `CARGO_NET_OFFLINE`,
appends the setup's variables,
and hands every other variable on unchanged.
It reads its own process's values for the two inherited variables the stub observes,
in the stub's order,
and keeps every other part of the expected string exact
(`package/linter/monochromatic-lint/src/closure_cargo_tests.rs`).

Both arms were measured on 2026-11-09:

- `mise run //package/linter/monochromatic-lint:test` with `CARGO_BUILD_BUILD_DIR` set by the launcher:
  the control passes,
  and the full task runs `949 tests run: 949 passed (1 slow), 3 skipped`.
- The test binary started with `env -u RUSTUP_TOOLCHAIN --unset=CARGO_BUILD_BUILD_DIR`:
  the control passes.
- Control of the control:
  adding `.env_remove("CARGO_BUILD_BUILD_DIR")` to `run_dylint` fails it,
  because the expected line then carries a variable the child no longer receives.
  The line was removed again.

## Related

- `doc/planning/lean-build-dirs.md`:
  mechanism A and its measurements.
- `doc/decision/cargo-build-dir-host-cache.md`:
  the shared build folder.
- `package/linter/monochromatic-lint/src/closure_stub_cargo.rs`:
  the names the stub logs,
  and why only the real run is asked about its environment.

[build-dir-host-cache]: ../decision/cargo-build-dir-host-cache.md
