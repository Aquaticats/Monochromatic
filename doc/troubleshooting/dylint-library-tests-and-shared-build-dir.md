# Dylint 6.2: library tests, the shared build directory and the commit policy

Evidence from building `package/linter/monochromatic-lint-closure` on `nightly-2026-10-17` with `dylint_testing` 6.2.0 and `cargo-dylint` 6.2.0.

## Fixture directory name corrupts words in the golden output

`dylint_testing::ui_test(name, "ui")` runs `compiletest_rs` 0.11.2,
which replaces the fixture directory path with `$DIR` in the normalised output.
With the relative directory `ui`,
the substring `ui` inside any word is replaced too,
so `required` becomes `req$DIRred` in the golden file and in the comparison.
The comparison still passes,
which hides the damage.
Workaround:
use a directory name that cannot occur inside words,
such as `tests/ui`.

## Golden files and the commit policy disagree about the last line

The harness compares the output exactly;
a warning run ends with the summary line and one empty line,
an error run ends without the empty line.
The commit wrapper's `final-newline` policy rewrites every non-empty text file to end in exactly one LF,
so a committed golden file no longer matches.
Workaround:
stage the fixtures into a scratch directory before the run
and restore the empty line there for files whose last line is a warning summary
(`src/ui_tests.rs` of the package).
The harness never reads `--bless` from the environment of `dylint_testing`;
the package's `bless` task copies the saved actual output instead.

## The scratch files go to the temporary directory

`compiletest_rs` writes `<fixture>.stage-id.stderr` and its build files to `std::env::temp_dir()`,
so `TMPDIR` decides where a failing run leaves its actual output.

## A separate cargo build directory moves the toolchain-named library copy

`dylint-link` writes `lib<name>@<toolchain>.so` next to the build output,
not next to the uplifted `target/debug/lib<name>.so`.
With the shared `build.build-dir` in `~/.cargo/config.toml`,
the copy lands in that directory,
and `cargo dylint --lib-path target/debug/lib<name>.so` fails with `the filename does not have the required form: "lib" LIBRARY_NAME "@" TOOLCHAIN ".so"`.
Workaround:
copy the library under the required name (the package's `run` task),
or set `CARGO_BUILD_BUILD_DIR=target` for the build.
The same split moves a build script's `OUT_DIR`,
which the IDE's `build.rs` climbs with `ancestors().nth(3)`:
its runtime stub then belongs in `target/debug/build/runtime`,
not `target/debug/runtime`.

## The library macro trips the repository's `unwrap` ban

`dylint_linting::declare_late_lint!` expands to a call to `Result::unwrap`,
which the root `clippy.toml` bans (`disallowed-methods`).
An `allow` attribute on the macro call is ignored (`unused_attributes`),
so the macro is placed alone in a module with `#![allow(clippy::disallowed_methods)]`.

## Changing the component list of the floating nightly moves the locked nightly

Evidence from stage 2a on 2026-10-18 with Mise 2026.10.0.
`mise.lock` stores the `components` and `targets` of the `rust` entry under `[tools.rust.options]`.
After `rustc-dev` was added to the `components` of `mise.no-env.toml`,
the locked entry no longer matched,
`mise install --dry-run rust` reported `rust@nightly-2026-10-18 would install`,
and `mise lock rust` pruned the entry for `nightly-2026-10-17` and wrote the one for `nightly-2026-10-18`.
The lock edit through Mise is therefore also a nightly bump.
`mise lock cargo:cargo-dylint cargo:dylint-link` adds only the two tools and leaves the `rust` entry alone,
but the stale options remain,
so every session would try to install the newer nightly until the `rust` entry is refreshed.

## Nightly cargo cannot reach crates.io on this host, stable cargo can

On the stage-2a host,
`cargo +nightly-2026-10-18 fetch` failed with
`[7] Could not connect to server (Failed to connect to index.crates.io:443 after 15 ms)`,
while `curl` and `cargo +stable` reached the same host.
The same failure appeared for `nightly-2026-10-13` and `nightly-2026-10-17`,
so it is not tied to the newest nightly.
`strace` showed every TCP `connect` of the nightly cargo ending in `SO_ERROR = ECONNABORTED` right after `EINPROGRESS`;
the cause was not found at that time.
It was found later: an OpenSnitch `reject` rule for the toolchain's `cargo` path destroys the sockets
(`doc/troubleshooting/cargo-nightly-1-101-cannot-reach-crates-io.md`).
Workaround used:
fill the default registry cache with the stable cargo (`cargo +stable fetch --locked` in the package,
and in a scratch crate that depends on `dylint_driver = "=6.2.0"`,
`anyhow` and `env_logger`,
which are the dependencies of the driver that Dylint builds),
then run with `CARGO_NET_OFFLINE=true`.
Without the driver prefetch the first `ui_tests::ui` run fails with `no matching package named dylint_driver found`.

## A Dylint run over the IDE needs two files in the build directory

The IDE's `build.rs` reads `debug/build/runtime/manifest.json` and `debug/crate-licenses.json` from the cargo build directory.
With `CARGO_BUILD_BUILD_DIR=target`,
relative to the working directory of the `run` task,
that directory is the lint package's `target`,
not the IDE's.
Copy both from the IDE's own `target` before the run.
