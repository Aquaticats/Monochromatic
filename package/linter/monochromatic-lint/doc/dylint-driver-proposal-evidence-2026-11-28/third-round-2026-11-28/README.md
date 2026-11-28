# The third round of measurements, 2026-11-28

What this folder is:
the containers of the third revision of
`doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`,
run after the third same-vendor gate read of that file.
It is unapproved and non-final;
the owner declares completion.

## Instrument

- Image:
  `localhost/dylint-probe-2026-11-28c`,
  digest `822e32a11951a20abd04720ca0811b6b6058c45a715b3c96c2449f4ea772677d`,
  built at 2026-11-28T07:54:59-05:00 from `Containerfile` beside this file.
  It holds today's floating `nightly` with `rustc-dev`,
  `llvm-tools` and `rust-src`,
  the lock's `nightly-2026-11-16` with the same components,
  `nightly-2026-11-17` with the minimal profile,
  and `cargo-dylint` and `dylint-link` 6.4.0.
- Linter:
  the released `monochromatic-lint` 0.5.2-alpha.25,
  sha256 `34ca8f26d618be2cbfce78bfd209aec0ae2d1c2faf7f00f4144374df37de485a`,
  copied from the machine-global install.
  Every `run.txt` prints it.
- Each container ran detached as rootless `podman` with
  `--cpus=4 --memory=8g --pids-limit=1024 --blkio-weight=10`,
  and the host script ran in `systemd-run --user --scope --slice=agent-work.slice`
  with `MemoryMax`,
  `CPUQuota`,
  `TasksMax`,
  `IOWeight=1` and `RuntimeMaxSec`.
- Nothing of this account's home was mounted.
  The library `package/linter/monochromatic-lint-closure`,
  `package/wg/cli-quicker-exempt`,
  and a `monochromatic-lint.config.jsonc` selecting only the closure rule were copied in with `podman cp`.
- Every container was removed at the end of its run;
  the image was removed after the round (`podman image rm`),
  so a rerun starts with `podman build`.
- Not kept:
  the two raw `strace` logs (their counts are kept in `*-counts.txt` and `*-execve` greps),
  the container image,
  the driver binaries,
  the cargo registry,
  and the build folders.

## The measurements

### `m12`: Answer I on the channel's current compiler

The floating channel is today's `nightly` with the components Answer I installs,
the library has no
toolchain file,
no driver exists,
and the three caches a warm run fills were filled first with the network,
so the linter's driver probe resolves the crate and reaches the library build.

- Arm A:
  the linter,
  no exported variable,
  no driver.
  Status **2**,
  one record at `pkg/Cargo.toml`,
  `core/processing-failure`,
  severity `error`,
  quoting `error[E0603]` against `dylint_linting`
  (`log/a-linter.jsonl`).
- Arm B:
  the warm command itself.
  Status 1,
  the `E0603` error lines,
  and no `~/.dylint_drivers` afterwards
  (`log/b-warm.err`,
  `log/b-error-lines.txt`,
  `log/b-driver-tree.txt`).

### `m13`: Answer K1's file, with today's compiler also on the channel

This run was meant to reduce the floating channel to the minimal profile first,
so that it would measure
K1 alone.
`rustup component remove` failed inside the container with
`Invalid cross-device link (os error 18)` while moving `rustc-src` (`log/component-remove.txt`),
so the channel kept the components and the run measures K1's file **with** the components on the channel.
It is kept because that state is a real one,
and because its arms say what happens in it:
arm A takes the driver-gap skip for the dated name,
arm B builds the driver in 89 s with 3 `git` executions,
arm C returns the six findings with the variable,
and arm D returns them **without** the variable,
because the linter's probe passed on the channel's components (`log/d-record.txt`).
`m15a` replaces it as the measurement of K1 alone.

### `m14`: rustup's behaviour with a toolchain file's components

- Arm A:
  a file naming an uninstalled toolchain,
  auto-install off,
  and the linter.
  `rustup show active-toolchain` fails (`log/a-active.err`);
  the linter prints **no record at all and exits `0`**,
  and its `--debug` notes say the driver check did
  not run and that the package produced "0 report(s) from the compiler"
  (`log/a-linter.jsonl` is empty,
  `log/a-linter-debug.err`).
- Arm B:
  the same file with auto-install on.
  rustup installs the toolchain and the file's components,
  with the deprecation warning,
  and the installed components are itemised in
  `log/b-installed-components.txt`.
- Arm C:
  a present toolchain that lacks a declared component.
  `rustup show active-toolchain` downloads and
  adds the two components (`log/c-components-before.txt`,
  `log/c-components-after-show.txt`);
  a later `cargo --version` adds nothing more.
- Arm D:
  `rustup toolchain install` after arm C.
  Nothing further is added,
  because arm C already did.

### `m15a`: Answer K1 alone, on the same image and the same day as `m12`

The floating channel was reinstalled with rustup's minimal profile (`log/floating-components.txt`),
the library holds `channel = "nightly-2026-11-16"` with its two components,
and no driver exists.

- Arm A:
  the linter without the variable.
  One rustc-dev skip per package,
  status `0`.
- Arm B:
  the warm command.
  Status `0` after 87 s,
  3 `git` executions,
  241 dated `rustc` executions,
  the driver's `-V` and its one RUNPATH entry.
- Arm C:
  the linter with `RUSTUP_TOOLCHAIN=nightly-2026-11-16`.
  Six findings at offsets 7146,
  1649,
  1962,
  2526,
  4978 and 2292,
  status `0`.
- Arm D:
  the linter without the variable,
  driver present.
  The rustc-dev skip again.

With `m12` this is the side-by-side the third read asked for:
one image,
one linter,
one day,
K1 returns the six findings with the variable and I returns status 2.

### `m15b`: the supported step for a file's components

A toolchain installed with the minimal profile,
and a file declaring `rustc-dev` and `llvm-tools-preview`.
`rustup toolchain install` with no argument in that directory synced the channel,
downloaded the two
components and reported the active toolchain installed (`log/a-install.txt`,
`log/components-before.txt`,
`log/components-after.txt`);
a bare `rustup show active-toolchain` then prints the name (`log/b-show.txt`).
This is the step that does not depend on the auto-install rustup calls deprecated.

### `m16`: Answer K1 once the lock reaches a compiler Dylint 6.4.0 cannot build on

`nightly-2026-11-28` is today's dated install and carries `c82ce1563 2026-11-27`
(`log/dated-rustc.txt`).
The library's file names it.

- Arm A:
  the warm command with the variable.
  Status 1,
  `error[E0603]` at `dylint_linting-6.4.0/src/lib.rs:794`,
  no driver (`log/a-warm.err`,
  `log/a-error-lines.txt`,
  `log/a-driver-tree.txt`).
- Arm B:
  the linter with the caches filled and no driver.
  Status **2**,
  one processing failure per package,
  the same message as `m12` (`log/b-linter.jsonl`).

The toolchain name a person writes is the **release** date,
not the compiler's:
`nightly-2026-11-27` does not exist,
and today's date is the install that carries `c82ce1563`.

### `m17`: a `cargo` whose own version probe fails

`RUSTUP_TOOLCHAIN=nightly-2026-11-30`,
which rustup does not have,
with auto-install off.
`cargo dylint --version` exits 1 (`log/b-version.err`),
and the linter returns one
`core/processing-failure` record at `monochromatic-lint.config.jsonc` and status **2**
(`log/c-record.txt`).
This is the `ProbeFailure::Broken` branch,
measured.
The first attempts of this measurement did not reach it and their logs are not kept:
`cargo dylint --version` alone ignores a toolchain file in a library directory,
and a cold registry
turns the run into a dependency skip before the probe matters.

## The probes

- `probes/repository-plan-lock-and-installer-lines.txt`:
  the `git grep` for `rustup update`,
  the workflow lines that install a floating `nightly`,
  the plan's `rust` row,
  the lock's `rust` entry,
  the two install functions of `mpm-shadow`,
  and `rustup_channel` and `expand_version`.
- `probes/todays-nightly-channel-manifest.txt`:
  the channel manifest and commit hash,
  with the clock.
- `probes/gh-dylint-releases-and-pull-request-2203.txt`:
  the releases,
  the pull request and the compare.
- `probes/gh-issue-6316-comments.txt`:
  the issue's comments as `gh` printed them,
  with their authors.
- `probes/m3-counts-recounted.txt`:
  the earlier round's counts,
  recounted from its kept files.
