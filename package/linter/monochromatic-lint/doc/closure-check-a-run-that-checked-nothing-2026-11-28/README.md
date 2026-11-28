# A run that checked nothing and printed nothing

Status:
unapproved and non-final.
Only the owner declares work complete.
This folder is the evidence of the round that answered issue 6667,
on `monochromatic-lint` `0.5.2-alpha.25`.

The round found that `rust/avoidable-closure` read a `cargo dylint` run that exited `0`
and printed none of cargo's records as a package without findings,
and changed the code so that a package the rule did not check never ends as a clean package.
The behaviour is stated in the package's `README.md`,
section "Machine requirements and skips".

## The state the issue was measured in

A copy of `package/linter/monochromatic-lint-closure` was given a `rust-toolchain.toml`
naming `nightly-2026-11-21`,
which the image does not hold,
and `RUSTUP_AUTO_INSTALL=0` was exported,
so `rustup` never installs a toolchain a file names on its own.
The fixture package beside it holds one closure the rule reports and one it leaves alone.

`scripts/run-arms.sh` and `scripts/arm.sh` ran one container per arm,
from the image that `Containerfile` builds,
with the fixture and the linter copied in and nothing of the account mounted.
`arm-a-before/` is the release's run and `arm-a-after/` is the fixed binary's run of the same arm,
each with `run.txt` (the whole arm),
the exact `cargo dylint` command's status,
standard output and standard error,
the linter's record,
standard error and status,
and the `--debug` notes.

What `arm-a-before/` shows,
which is the issue's cause:

- `rustup show active-toolchain` in the library directory exits `1`
  (`error: override toolchain 'nightly-2026-11-21-x86_64-unknown-linux-gnu' is not installed: the toolchain file at '/work/repo/lint-closure/rust-toolchain.toml' specifies an uninstalled toolchain`).
- `cargo dylint --path …/lint-closure --manifest-path …/pkg/Cargo.toml -- --locked --message-format json`
  exits `0`,
  prints nothing on standard output,
  and prints two warnings on standard error:
  ``Warning: `cargo metadata` exited with an error: error: toolchain 'nightly-2026-11-21-x86_64-unknown-linux-gnu' is not installed``
  and `Warning: No libraries were found.`
  Dylint gathers the library packages of `--path` by running `cargo metadata` in each one
  (`dylint-6.4.0/src/library_packages/mod.rs`,
  `from_opts`:
  `package_with_root(&path)` warns and continues),
  so a library whose own directory cannot resolve a toolchain contributes no library,
  and `resolve` then has nothing to check (`dylint-6.4.0/src/lib.rs`,
  `run_with_name_toolchain_map`).
- the linter exits `0` and prints no record at all,
  and only `--debug` says ``the Dylint driver check for the library lint-closure did not run: `rustup show active-toolchain` failed (exit status: 1)``
  and `rust/avoidable-closure checked pkg: 0 report(s) from the compiler`.

`arm-a-after/` shows the same state read as what it is:
one warning finding at `pkg/Cargo.toml`,
naming the library,
quoting `rustup` and giving `rustup toolchain install` in the library's directory,
with exit status `0` and one record in the JSONL.

## The other three states of the driver check

- A driver directory that cannot be named (`no-driver-directory/`):
  measured in the container as a user with no passwd entry and no `HOME`,
  which is the only way this machine can make the home directory unknown.
  `cargo dylint` then fails on its own,
  and the run exits `2` with the compiler's words,
  before and after the change alike.
- A cache directory that cannot be named:
  built only against the package's own `DriverSite`,
  in `closure_step::tests::a_missing_cache_directory_and_a_silent_run_is_a_failure`,
  because the home lookup of this machine answers a path in every process of this account.
- A resolve probe that fails (`stub-cargo/probe-fails-*`):
  `XDG_CACHE_HOME` points at a directory that cannot be written,
  and the package's stub `cargo` (`src/closure_stub_cargo.rs`) answers the way a `cargo dylint` does in the state above:
  exit `0`,
  no record.
  The release prints nothing and exits `0`;
  the fixed binary exits `2` with one `core/processing-failure`.
  `stub-cargo/checked-*` is the control:
  the same machine and one record on standard output,
  which is what a run that reached the compiler leaves,
  and both binaries exit `0` there.

`cargo-check/` holds what a real `cargo check --message-format json` prints,
from the same fixture package on the host:
`cold-first-run.jsonl` compiles and `fresh-second-run.jsonl` finds every unit built,
and both hold a `compiler-artifact` record and `{"reason":"build-finished","success":true}`.
That is why a run that printed no record at all cannot have reached the compiler,
and the stand-in that the package's tests use for a checked run is copied from the second file.

## What this evidence does not show

- The library was never built in the container:
  the image's floating nightly is newer than `dylint_linting` 6.4.0 compiles against
  (`error[E0603]: enum `ErrorOutputType` is private`),
  and a driver for the dated toolchain the repository locks would have to clone the Clippy repository,
  which this machine's rootless containers cannot do
  (`git clone` answers `fatal: could not create work tree dir '…': Permission denied`
  while the same shell's `mkdir` succeeds).
  So no arm here or above shows a package that a real `cargo dylint` did check with the library built.
- The paths inside a capture are the record's own text.
  The fixture they name existed only while the round ran,
  and nothing here depends on it still existing.

## Running it again

```sh
# build the image (needs the network) and remove it afterwards by the name it prints
podman build --tag localhost/monochromatic-lint-closure-issue-6667-20261128 --file Containerfile .

# one arm per container, with the linter given as the second word
bash scripts/run-arms.sh <label> <linter binary> arm-a arm-no-driver-dir

# the stub-cargo states, which need no container
bash scripts/stub-arms.sh <label> <linter binary> probe-fails
bash scripts/stub-arms.sh <label> <linter binary> checked

podman rmi localhost/monochromatic-lint-closure-issue-6667-20261128
```

The two binaries these captures name:

- the release:
  `0.5.2-alpha.25`,
  sha256 `34ca8f26d618be2cbfce78bfd209aec0ae2d1c2faf7f00f4144374df37de485a`.
- the build of this round:
  a debug build of the branch,
  sha256 `f26769c5891edee27af0e54d1747ac30cb1dfdd2accf4ecc48006a669d1c8f63`.

The image built here was `localhost/monochromatic-lint-closure-issue-6667-20261128`,
`8a6a24e8b45bf2b3eb8d0dd921e410e74cecdd098744266bbedb9c9b6d1f141e`,
and the round removed it when it ended.

The image installs the floating `nightly` channel and not the dated one the repository locks,
for the reason in `Containerfile`:
`dylint_driver`'s build script clones the Clippy repository to read `clippy_utils/src/sym.rs` for a dated channel,
and that clone cannot create a directory here.
