# Who provides the Dylint driver `rust/avoidable-closure` needs, and what a skipped rule returns

Purpose:
answer,
with file and line,
who puts the Dylint driver on a machine and how,
what the linter does when each piece is absent,
what a skipped rule actually returns to a reader or a script,
and what this machine has today;
then set out the options for closing the gap,
ranked by what each gives a person who needs the rule's findings.
Status:
a proposal,
unapproved and non-final,
written 2026-11-27T21:03:56-05:00 for issue 6643.
Nothing in it is built,
decided,
or accepted;
the owner picks.
Beyond this file the round changed nothing in `package/linter/monochromatic-lint`,
`toolchain.plan.jsonc`,
`toolchain.lock.json`,
`mise.toml`,
or the workflows.
Every claim about behaviour carries what it rests on:
a source path with a line,
a command run in this round with its date,
or a record already in the repository.

## The question the issue records

Issue 6643 says `rust/avoidable-closure` runs on no host run of the linter on the coordinating session's machine,
so every count of findings taken there leaves that rule out.
The line a run prints begins
"rust/avoidable-closure skipped package/mpm-shadow/cli:
the Dylint driver for toolchain
`nightly-x86_64-unknown-linux-gnu` at /home/user/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver
is absent (cargo-dylint is 6.4.0),
and `dylint_driver` 6.4.0 with its dependencies is not in the cargo cache,
so `cargo dylint` would have to download it,
which this linter does not do".
The issue also records the consequence that matters more than the absence:
"a run "without findings" was read as clean more than once before the line was noticed".
The issue asks two things:
who provides the driver,
from the linter's source and README,
the toolchain plan and lock,
and the workflows,
stated with file and line;
and the options,
each with what it costs,
as a proposal under `doc/planning/` and not built until the owner picks.

## How the linter runs the rule

`rust/avoidable-closure` is not a rule of the linter's own analysis.
`package/linter/monochromatic-lint/README.md:384-390` states it runs the type-aware Dylint lint `avoidable_closure`
of `package/linter/monochromatic-lint-closure` over each Rust package that has a selected file,
and names `doc/decision/rust-closure-lint-dylint.md` as the decision.
The root configuration selects it for every Rust file with the library path relative to the configuration file
(`monochromatic-lint.config.jsonc:33-39`,
`"rust/avoidable-closure": { "severity": "warn", "library": "package/linter/monochromatic-lint-closure" }`).

After the per-file rules,
`run_closure_step_at` (`package/linter/monochromatic-lint/src/closure_step.rs:169-244`) does this,
in order:

1. It groups the selected Rust files by the nearest `Cargo.toml` above them
   (`src/closure_packages.rs`,
   `group_packages`),
   so the unit of work is the package,
   not the file.
2. It returns a note and nothing else under `--fix`
   (`src/closure_step.rs:177-180`),
   because the lint has no machine fix and a rewrite would make the compiler's positions stale.
3. It reports a configuration error,
   at processing-failure severity,
   when the configured library directory holds no `Cargo.toml`
   (`src/closure_step.rs:183-189`);
   that is a fault of the repository,
   not of the machine.
4. It runs `cargo dylint --version` once
   (`src/closure_step.rs:191`,
   `dylint_probe` at `src/closure_cargo.rs:191`).
5. It decides on a machine-wide gap:
   a `cargo` that exists and cannot be started fails the run with status `2`;
   a `cargo` or `cargo-dylint` that is not there,
   and a compiler without the `rustc-dev` component,
   skip every selected package
   (`src/closure_step.rs:192-213`,
   `rustc_dev_missing` at `src/closure_cargo.rs:291-303`).
6. It checks the Dylint driver once per distinct library
   (`driver_gaps` at `src/closure_driver.rs:278`),
   and skips the packages of a library whose driver cannot be had
   (`src/closure_step.rs:214-230`).
7. For every remaining package it checks dependencies,
   prerequisites and then runs the lint
   (`check_package` at `src/closure_step_package.rs:203-274`).

The run itself is one process per package
(`run_dylint` at `src/closure_cargo.rs:385-421`):
`cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked --message-format json`,
with `CARGO_NET_OFFLINE=true` (`src/closure_cargo.rs:400`)
and `DYLINT_RUSTFLAGS="-W avoidable_closure -W unfulfilled_lint_expectations"`
(`src/closure_cargo.rs:62`).
Only the library and binaries are compiled,
so `#[cfg(test)]` modules,
`tests/`,
examples and benchmarks are not
checked (`README.md:430-466`).

## What each absence produces

Every absence below is a distinct branch in the source,
and they do not behave the same.

- No `cargo`,
  or a `cargo` without the `dylint` subcommand:
  `ProbeFailure::Absent` (`src/closure_cargo.rs:191-215`),
  whose only accepted evidence is cargo's own `no such command` for `dylint`
  (`src/closure_cargo.rs:135`,
  `README.md:475-486`).
  The run skips every selected package.
- A `cargo` that exits 126 or 127,
  or fails without that evidence:
  `ProbeFailure::Broken`
  (`src/closure_cargo.rs:216-246`),
  which is a `core/processing-failure` finding and exit status `2`,
  because a skip that exits `0` would hide a stale install (`README.md:529-546`).
- A compiler without `rustc-dev`:
  a machine-wide skip
  (`rustc_dev_missing` at `src/closure_cargo.rs:291-316`,
  `src/closure_step.rs:202-213`).
- A driver that is absent or older than `cargo-dylint`,
  with the pinned `dylint_driver` crate unresolvable offline:
  one skip per package of that library
  (`driver_state` at `src/closure_driver.rs:178-230`,
  `driver_gaps` at `src/closure_driver.rs:278-330`).
- A package whose locked dependencies are not in the cargo cache:
  a skip naming `cargo fetch --locked`
  (`dependencies_missing` at `src/closure_cargo.rs:320-353`).
- A prerequisite named in `monochromatic-lint.prerequisites.jsonc` that the machine lacks:
  a skip naming the task that makes it (`src/closure_probe.rs`,
  `README.md:551-614`).
- A check that cannot be made at all,
  such as no readable `cargo dylint --version`,
  no `rustup`,
  or no known driver directory:
  a note that appears only under `--debug`,
  and the packages still run
  (`src/closure_driver.rs:185-187` and `:306-309`,
  `README.md:518-522`).

## What a skipped rule returns

This is the part the issue's title asks for,
and it is decided by three lines of code.

1. The skip is a diagnostic of the same rule code as a real finding:
   `Diagnostic::new(RULE, Severity::Warn, ...)` at `src/closure_step_package.rs:100-122`,
   where `RULE` is `rust/avoidable-closure`.
   Its file name is the package's `Cargo.toml` and its position is the first byte of it.
   Its message starts `rust/avoidable-closure skipped <package>: ` and ends
   "Nothing in this package was checked by this rule,
   so a run without findings says nothing about its closures."
   (`src/closure_step_package.rs:104-113`,
   the last sentence at `:109`).
2. Skips are merged into the same finding list as findings before output
   (`src/run_command.rs:204`,
   `findings.extend(closure.skips)`),
   so nothing downstream can tell them apart by code,
   file or position;
   only the word ` skipped ` in the message,
   and the fixed `warn` severity,
   mark them.
3. The exit status counts the skip as a warning and nothing else
   (`exit_status` at `src/run_output.rs:58-86`):
   any warning alone leaves `0`;
   `--max-warnings` counts it and exiting over the limit gives `1`;
   only a finding marked as a processing failure gives `2`.
   `--quiet` and `--silent` hide it with the other warnings
   (`src/run_output.rs:93-99`,
   `README.md:523-528`).

What that means for a reader or a script,
measured on this machine on 2026-11-27 at 21:00 -05:00 with the installed `0.5.2-alpha.23`,
whose plan row is `cargo:monochromatic-lint` at `toolchain.plan.jsonc:269-278`,
which is what `mise run lint:monochromatic-lint` runs:

- `monochromatic-lint package/wg/cli-quicker-exempt` printed one JSONL record and exited `0`.
  The record is
  `{"code":"rust/avoidable-closure","severity":"warn","filename":"package/wg/cli-quicker-exempt/Cargo.toml",...}`.
- `monochromatic-lint --max-warnings 0 package/wg/cli-quicker-exempt` exited `1`.
  A gate can therefore make a skip fail today,
  with no code change,
  at the price of failing every Rust package on a machine without a driver.
- `monochromatic-lint --quiet package/wg/cli-quicker-exempt` printed no finding and exited `0`,
  so the quiet form of a run looks exactly like a clean run.
- A script that counts findings by code counts a skip as a finding of the rule:
  the code field is identical.
  A script that reads `severity` sees a warning,
  which a repository with a warning backlog cannot tell from a real one
  (`doc/planning/repository-linter-stays-at-zero.md:19` records 516 real findings of this rule at `136a40fe69`,
  508 of them warnings).
- `--rules` reports the rule as not suppressible with one option,
  `library`
  (`src/run_modes_rule_table.rs:116-122`;
  the installed binary printed
  `{"id":"rust/avoidable-closure","fixable":false,"fixClass":null,"options":["library"],"suppressible":false}`
  on 2026-11-27 at 21:12 -05:00),
  so a project cannot turn a skip off without turning the whole rule off.

## Who provides the driver today

No component of this repository builds or installs a Dylint driver.
Each part below was read whole where the brief named it,
and each states what it does and does not do.

### The linter's own message and its warm command

The skip message names the command that would fill the caches
(`src/closure_driver.rs:218-231`),
built by `warm_command` (`src/closure_driver.rs:236-243`):

```sh
env --unset=CARGO_NET_OFFLINE cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked
```

That command is a normal `cargo dylint` run with the network:
it downloads the pinned `dylint_driver` crate and its dependencies,
builds the driver into the driver directory,
and checks the package on the way
(`src/closure_driver.rs:224-235`.
`README.md:495-517` describes the whole sequence,
including that the linter never fetches).
The linter's own check is offline by construction:
`dylint_drivers_directory` reads `DYLINT_DRIVER_PATH` or `<home>/.dylint_drivers`
(`src/closure_driver_probe.rs:69-104`,
`:110-141`),
`active_toolchain` asks `rustup show active-toolchain` in the library directory
(`src/closure_driver_probe.rs:162-204`),
`driver_version` runs the driver's `-V` (`src/closure_driver_probe.rs:205-230`),
and `driver_unresolved` resolves a throwaway crate that pins `dylint_driver = "=<version>"`
with `cargo metadata --offline`
(`src/closure_driver_probe.rs:232-334`).
So the linter reads the driver cache,
mirrors Dylint's staleness rule,
and refuses to write anything.

### The toolchain plan, the lock and the install task

`toolchain.plan.jsonc` holds three rows that matter here,
and none of them is a driver:

- `rust` (`toolchain.plan.jsonc:184-217`):
  the floating nightly,
  installed through rustup with the components `clippy`,
  `rust-src`,
  `llvm-tools`,
  `rustc-dev` and `rustfmt`
  (`toolchain.plan.jsonc:202-212`),
  and the targets `aarch64-linux-android` and `x86_64-linux-android`.
- `cargo:cargo-dylint` (`toolchain.plan.jsonc:235-241`):
  `cargo:cargo-dylint@latest` through `cargo-binstall`,
  exposing the bin `cargo-dylint`.
- `cargo:dylint-link` (`toolchain.plan.jsonc:242-248`):
  the same for `dylint-link`.

`toolchain.lock.json` records what those rows resolved to on this machine:
`nightly-2026-11-16` with the same five components (`toolchain.lock.json:493-528`),
`cargo-dylint` 6.4.0 (`toolchain.lock.json:568-577`)
and `dylint-link` 6.4.0 (`toolchain.lock.json:579-589`).
The single task that installs all of it is `toolchain:install` (`mise.toml:624`),
with `bootstrap` and `toolchain:install:global` as aliases of it (`mise.toml:56`,
`:1227-1235`).
`rg --line-number --ignore-case 'dylint|driver' toolchain.plan.jsonc toolchain.lock.json mise.toml` finds
the three rows above and no driver:
no plan row,
lock field,
or task names `.dylint_drivers`,
`dylint-driver`,
or `dylint_driver`.

So the install provides the two tools and the compiler component,
and stops one step short of the artifact the rule needs.
That gap is the whole of issue 6643.

### What the driver is, in Dylint's own terms

What Dylint documents is short.
Its README at tag `v6.4.0` (fetched 2026-11-28T02:0x UTC,
this round) opens with
`cargo install cargo-dylint dylint-link`
(`README.md:6` and `:31-42` of the tag),
and its "Caching in CI" section says,
"we recommend caching certain directories to avoid rebuilding libraries,
rebuilding drivers,
or redownloading toolchains",
then lists `~/.cargo/git/checkouts/`,
`~/.dylint_drivers/` and `~/.rustup/toolchains/`
(README at `v6.4.0`,
lines 203-222).
`docs/how_dylint_works.md` at the same tag describes how libraries are found and what they must export,
including that a library is named `LIBRARY_NAME@TOOLCHAIN` and that Dylint builds a package with the toolchain that
built the library (`docs/how_dylint_works.md:34-48`,
`:73-78` at `v6.4.0`);
it says nothing about drivers.
Neither document says how a driver is produced.

The build is described only by the source.
`dylint/src/driver_builder/mod.rs` at `v6.4.0` (fetched this round):

- `get` takes the driver directory,
  makes `<drivers>/<toolchain>` if it is missing,
  and builds when `<drivers>/<toolchain>/dylint-driver` does not exist or is older than `cargo-dylint`
  (lines 48-67).
- `dylint_drivers` returns `DYLINT_DRIVER_PATH` when it is set and a directory,
  and otherwise `<home>/.dylint_drivers`,
  which it creates with a `README.txt` that says deleting the directory
  "will cause Dylint to rebuild the drivers the next time it needs them,
  but will have no ill effects"
  (lines 18-24,
  69-91).
- `initialize` writes a throwaway crate whose manifest pins `dylint_driver = "=<version of cargo-dylint>"`,
  alongside `anyhow` and `env_logger`,
  and whose `rust-toolchain.toml` names the toolchain and the components `llvm-tools-preview` and `rustc-dev`
  (lines 192-235).
- `build` compiles that crate quietly,
  with an rpath into the toolchain's `lib`
  (lines 118-190).

One correction to a newer upstream document:
the README on Dylint's default branch (fetched the same time) now says drivers live in
`~/.dylint_drivers/`,
"or in `$DYLINT_HOME/drivers/` if `DYLINT_HOME` is set".
At tag `v6.4.0`,
`internal/src/env.rs:27-36` declares `DYLINT_DRIVER_PATH` and no `DYLINT_HOME`,
so the linter's mirror of the 6.4.0 rule is the right one for the pinned release,
and a `cargo-dylint` bump is the moment to re-read it.

### The workflows

`rg --line-number --ignore-case 'dylint' .github` returns no hit at all,
so no workflow names the tool,
the driver,
or the rule.
The one workflow that runs the linter is `.github/workflows/advisory-core.yml`:

- `.github/workflows/advisory-core.yml:152-159` installs the locked Rust toolchain with
  `rustup toolchain install "$toolchain" --profile minimal --no-self-update`,
  which carries neither `rustc-dev`,
  nor `llvm-tools`,
  nor `cargo-dylint`.
- `.github/workflows/advisory-core.yml:200-215` lints only the changed `*.md`,
  `*.mdx` and `*.rs` files of a
  pull request or push,
  through `task-run lint:monochromatic-lint -- "${files[@]}"`.

Because no `cargo-dylint` exists there,
a CI run that reaches a Rust package skips it at `dylint_probe`
(`src/closure_cargo.rs:207-215`),
and the message it would print names `cargo-dylint` as missing rather than a driver.
So the rule has never run in CI either,
and a green workflow never included it.

### The tasks that do name the rule's needs

Two tasks in `package/linter/monochromatic-lint/mise.toml` and one in
`package/linter/monochromatic-lint-closure/mise.toml` name what the rule needs,
and none of them is on any automatic path:

- `//package/linter/monochromatic-lint:test:host`
  (`package/linter/monochromatic-lint/mise.toml:78-80`)
  is the one ignored binary test,
  `host_real_run_over_wg_quicker_exempt`
  (`src/closure_binary_tests.rs:727-759`),
  which asserts a real run over `package/wg/cli-quicker-exempt` has no skip and at least one finding,
  and needs `cargo-dylint`,
  the `rustc-dev` nightly and a warm cargo cache.
  It fails today for the same reason a host run skips.
- `//package/linter/monochromatic-lint-closure:run` runs `cargo dylint --all-targets` over another package
  (`package/linter/monochromatic-lint-closure/mise.toml`,
  `[tasks.run]`),
  which requires a driver to exist already.
- The root `lint:monochromatic-lint` task (`mise.toml:1057-1062`) depends on `fetch:rust-dependencies`
  (`mise.toml:1059`),
  which runs `cargo fetch --locked` per package (`mise.toml:1031-1051`).
  That fills the package locks and not the driver crate,
  which belongs to no lockfile (`src/closure_driver.rs:4-27`).

## What this machine has

Read-only looks,
this round,
on 2026-11-27 at 20:59 to 21:03 -05:00.

- `~/.dylint_drivers` does not exist:
  `ls` answers "No such file or directory".
  The driver directory Dylint would use has never been created on this account.
- `rustup toolchain list`:
  `nightly-x86_64-unknown-linux-gnu (active, default)` and
  `nightly-2026-11-16-x86_64-unknown-linux-gnu`.
  The active one is the undated floating channel,
  which is the toolchain name in the skip message;
  the dated one is what the lock records.
- The toolchains differ in size on disk:
  `du -sh` gives 274M for `nightly-x86_64-unknown-linux-gnu` and 799M for `nightly-2026-11-16-x86_64-unknown-linux-gnu`,
  which is consistent with the dated one carrying `rustc-dev` and `llvm-tools` and the default carrying neither.
  That was not verified component by component,
  and it matters:
  the driver is built for the toolchain that is active in the library directory.
- `cargo` on `PATH` is this repository's shadow,
  `/home/user/Monochromatic/.cache/cargo-shadow-bin/cargo`,
  which reports `cargo-shadow 0.1.5-alpha.17, shadowing cargo at /home/user/Monochromatic/.cache/toolchain-bin/cargo`.
  `rustup` is `/usr/bin/rustup`.
  `~/.cargo/bin` holds `cargo-binstall`,
  `lsd`,
  `maki` and `tokei`,
  and no `cargo`,
  so this account has no rustup-proxy `cargo` on `PATH`.
- `cargo dylint --version` prints `cargo-dylint 6.4.0`,
  which matches `toolchain.lock.json:568-577`.
  `cargo-dylint` and `dylint-link` are on `PATH` through `.cache/toolchain-bin`.
- The cargo registry cache holds `dylint-6.4.0.crate`,
  `dylint_driver-6.4.0.crate`,
  `dylint_internal-6.4.0.crate`,
  `dylint_linting-6.4.0.crate` and `dylint_testing-6.4.0.crate`,
  and it holds `jiff-1.0.3` and not `jiff-static-1.0.3`.
  The throwaway driver crate reaches `jiff` through `env_logger` 0.11,
  whose manifest requires `jiff = "1.0.1"` with the `std` feature
  (`~/.cargo/registry/src/*/env_logger-0.11.12/Cargo.toml`,
  `[dependencies.jiff]`),
  so the offline resolve of the driver crate cannot complete even though `dylint_driver` 6.4.0 itself is cached.
  That is the second half of the message the issue quotes,
  and cargo's own words in it are "failed to download `jiff-static v1.0.3`".
- Dylint's own estimate of the cache this needs is 42 MB after a first run
  (`package/linter/monochromatic-lint-closure/README.md:428`).

## The rule run once where a driver exists

Every command below ran inside a rootless `podman` container in its own
`systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and
`RuntimeMaxSec`,
started with `podman run --rm --cpus=3 --memory=6g --pids-limit=256 --blkio-weight=10`.
The image was `docker.io/cachyos/cachyos:latest`,
already local.
Nothing of this account's home went into it except three read-only mounts:
the package `package/wg/cli-quicker-exempt`,
the library `package/linter/monochromatic-lint-closure`,
and the released linter binary
`~/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-lint/0.5.2-alpha.23/.../monochromatic-lint`.
The container built its own toolchain and its own driver,
and the network served only cargo,
rustup and one `git clone`.
Each container was removed at the end of its run,
and no container of this round is left.
The logs are in the round's scratch folder;
this section is their summary.

Six containers in all,
from the same local image:
five attempts at the rule and one probe of the image's `git`.
Every one of them was removed before this round ended.
The first four stopped before the rule ran,
and each stop is a fact about what provisioning a driver costs.
The fifth ran the rule and produced findings.
Every time below is the container clock (`date -u`) or the run's own `SECONDS` arithmetic.

### The first attempt: a nightly without `llvm-tools`

2026-11-28T01:58Z to 02:02Z,
256 s.
The container installed `nightly-2026-11-16` with `rustc-dev` and no `llvm-tools-preview`,
built `cargo-dylint` and `dylint-link` 6.4.0 from crates.io in 1 m 38 s,
and stopped in the lint library's link step with
`rust-lld: error: unable to find library -lLLVM-23-rust-1.102.0-nightly`
(`collect2: error: ld returned 1 exit status`).
The linter then reported the skip the issue quotes,
and `--max-warnings 0` exited `1`.

### The second attempt: the driver's build script cannot clone

2026-11-28T02:03Z to 02:08Z,
278 s.
With `llvm-tools-preview` added,
the library built,
and the driver's own build script stopped at
`fatal: destination path '/tmp/.tmpBYEnbX' already exists and is not an empty directory.`
That message comes from `dylint_driver`'s build script,
which clones `https://github.com/rust-lang/rust-clippy` into a temporary directory it has just created
(`dylint_internal/src/clippy_utils/repository.rs:55`,
`clone` at `dylint_internal/src/git.rs:16-41`).

### The third attempt: locating the refusal

A separate container,
02:10Z,
ended after 90 s.
`git clone --depth 1 https://github.com/trailofbits/dylint /probe-empty`,
where `/probe-empty` had just been created and `ls -la` showed it empty,
printed the same `already exists and is not an empty directory`,
and `git clone` into a path that did not exist printed
`fatal: could not create work tree dir '/probe-fresh': Permission denied`.
So the refusal was not about emptiness:
the container's `git` could not create a work tree under `/tmp` at all.

### The fourth attempt: the vendored-repository route, and a panic

2026-11-28T02:11Z to 02:18Z,
358 s.
A full clone of `rust-lang/rust-clippy` took 88 s and 111 MB,
and `DYLINT_CLIPPY_REPO` (`dylint_internal/src/env.rs:28`) pointed the build script at it.
The clone of the driver crate then built,
and its build script stopped at
`fatal: not a git repository (or any of the parent directories): .git`
followed by
`thread 'main' panicked at .../dylint_driver-6.4.0/build/extra_symbols.rs:94:5: assertion failed: status.success()`,
because the local repository libgit2 clones from a repository directory has no working tree of its own for the
`git checkout` that follows.
After that failure the linter's own probe still skipped the package,
this time with a different reason:
`Cargo's error: failed to download anstyle-wincon v3.0.11`,
a Windows-only crate that the throwaway driver crate resolves for every platform
(the README states this at `README.md:496-500`).
So a Linux-warm registry is not enough for the probe either.

### The fifth attempt: a writable temporary directory

2026-11-28T02:19Z to 02:26Z,
398 s,
and this one worked.
The only change from the second attempt was `TMPDIR` (and `TMP`,
`TEMP`) pointing at `/work/tmp`.
With that,
`git clone --depth 1 <url>` into an existing empty directory returned `0` at 02:19:41Z,
the same command that had failed 40 minutes earlier.
The cause of the refusal was not established:
this host enforces AppArmor,
and the container's profile was not inspected,
so the finding is the symptom and the setting that changes it,
not the mechanism.
No host path,
profile or mount was changed,
and the work was not moved elsewhere on this account.

What the fifth attempt measured,
all inside the container:

- Toolchain:
  `nightly-2026-11-16` with `rustc-dev` and `llvm-tools-preview`,
  88 s to install.
- `cargo-dylint` and `dylint-link` 6.4.0 compiled from crates.io:
  123 s.
- Warm run of `cargo dylint` (library,
  driver and check):
  178 s.
  The driver build is the part that needs the network:
  the pinned driver crate,
  its dependency tree,
  and a full
  clone of `rust-lang/rust-clippy` for the `clippy_utils` symbols its build script reads.
- The driver it left:
  `/root/.dylint_drivers/nightly-2026-11-16-x86_64-unknown-linux-gnu/dylint-driver`,
  43,970,800 bytes,
  `-V` printing
  `nightly-2026-11-16-x86_64-unknown-linux-gnu 6.4.0`,
  and 16 MB in the driver directory as `du` counts it.
- Disk after the run:
  `~/.rustup` 637 MB,
  `~/.cargo` 121 MB,
  the package's build tree 76 MB,
  `/work/tmp` 9.1 MB,
  plus the container image.

Then the linter,
with the driver present,
on the copied package and the same released binary:

- `monochromatic-lint --config ... pkg` printed `6` findings,
  all `rust/avoidable-closure` at `warn`,
  and exited `0`.
  Two seconds after the warm run,
  with no network use:
  the linter's own run sets `CARGO_NET_OFFLINE=true`
  (`src/closure_cargo.rs:400`).
  The `6`,
  as the package's own paths and the positions the compiler gave:
  `src/application_targets.rs:202:33`,
  `src/application_watch_mark_reconciliation.rs:40:14`,
  `:47:17` and `:62:14`,
  `src/keeper_process_identity.rs:142:19`,
  and `src/update_routing_engine_construction_and_marks.rs:72:39`.
  Three are closures that capture state for a callee that does not keep it,
  and three capture nothing at all.
- `monochromatic-lint --config ... --max-warnings 0 pkg` exited `1` with the same `6` findings.
- The direct `cargo dylint --path <library> --manifest-path <package> -- --locked --message-format json`
  of the warm step reported the same `6` `compiler-message` records.

## Options

Each option is designed far enough to show what it costs and where it breaks.
The four the issue names come first,
then three more the sources showed.
Every one of them is a repository change or an owner decision,
and this round builds none of them.

### Option A: the toolchain install provides the driver

Design:
give the driver an owner in `toolchain.plan.jsonc`,
so that the same step that installs `cargo-dylint` also produces
the driver for the toolchain it was just installed beside.
A new row,
say `dylint-driver`,
would carry a compile step rather than an archive:
after the `rust` row and the two Dylint tool rows are converged,
run Dylint's own build once with the full toolchain triple in `RUSTUP_TOOLCHAIN`
(the recorded workaround in `doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md`,
"Build the driver once and reuse it through `DYLINT_DRIVER_PATH`"),
place the binary at `<driver root>/<toolchain>/dylint-driver`,
and export `DYLINT_DRIVER_PATH` through the environment the installer already writes
(the launcher's `config/mise/conf.d/30-toolchain-env.local.toml`,
seen in the installed `mpm-shadow`'s own strings).
`toolchain.plan.jsonc` and `toolchain.lock.json` then need a record that ties the driver to the pair
(cargo-dylint version,
rust toolchain),
because either moving invalidates it.

Pros:
one owner for a version and its consequence,
so a bump of `cargo:cargo-dylint@latest` cannot leave a machine without
a driver;
no manual step for anyone who ran the installer;
the emitted `DYLINT_DRIVER_PATH` makes the linter's check succeed by the same route Dylint itself uses.

Cons:
it puts a compile into the installer,
which the owner has paused:
this round may not run `mise run toolchain:install`,
and by the owner's word the reason is "Because we need to make mpm-shadow prefer system package managers on
non-atomic distros first" (issue 6316).
It also needs network at install time for the driver crates,
and a repair path for the wrapper-`cargo` failure,
since the installer's `cargo` may be a shadow and not a rustup proxy.
It does nothing for a machine that never runs the installer,
and nothing for CI.

Leaves open:
where the driver root lives (per user,
or inside the repository's `.cache`),
what the lock records for a compiled artifact with no archive digest,
and whether the compiler step belongs in `toolchain:install` or in a task that depends on it.

### Option B: the linter's own task warms the driver once, outside the offline run

Design:
the root task that runs the linter gains a step after `fetch:rust-dependencies` that runs the warm command the skip
message already prints,
once,
per library directory,
with the network.
`fetch:rust-dependencies` (`mise.toml:1031-1051`) is the precedent:
it already fetches,
already needs the network,
and already exists to stop a cold-cache skip.
On this host the plain warm command does not work,
so the step would have to be the recorded workaround:
build Dylint's throwaway package with `RUSTUP_TOOLCHAIN=<dated toolchain>-<triple>` and install the binary at
`<drivers>/<toolchain>/dylint-driver`.

Pros:
the owner of the need pays for it,
and no installer change is needed;
it needs no paused component;
it is testable from a task alone,
and the linter keeps its never-fetch contract.

Cons:
`cargo:cargo-dylint@latest` moves under it,
and every move makes the existing driver outdated
Dylint's own rule at `dylint/src/driver_builder/mod.rs:93-116`,
so the warm step runs again on the next bump;
the recorded history is three releases in three days
(`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
"What the issue records" and "The intermittency"),
so a task that warms lazily still meets a stale driver often.
CI runs the same root task (`advisory-core.yml:213`) but installs neither `rustc-dev` nor `cargo-dylint`,
so the warm step would have to be skipped there by condition,
which is one more skip to explain.
And the step writes `~/.dylint_drivers`,
a per-user cache outside the repository,
which `mise run lint` has never done.

Leaves open:
whether a task may write a cache outside the checkout,
and whether the warm step fails the task when the network is not there,
which would turn a lint run into a network dependency.

### Option C: the rule moves to a check that runs where the driver is

Design:
stop pretending the rule runs in an ordinary local lint run,
and give it a gate that owns the driver:
a container image built with the nightly,
`rustc-dev`,
`llvm-tools`,
`cargo-dylint`,
`dylint-link` and a driver built
inside the image,
and one task that runs the lint over the packages the work owns
(the wrapper,
`mpm-shadow`,
`cargo-shadow`,
and the linter),
in the same shape as `package/linter/monochromatic-lint/test.Containerfile` and its tasks.
`//package/linter/monochromatic-lint:test:host` and `//package/linter/monochromatic-lint-closure:run` are the
existing fragments of exactly this.

Pros:
it produces findings mechanically,
over named packages,
without touching this account's caches;
the image is reproducible and the driver is built once per image;
it is where the repository already runs its heaviest checks.

Cons:
it gives no finding for a local edit,
and a local `mise run lint` still reports the rule as skipped
unless option D is taken as well;
the image build needs the same driver build and network;
a nightly or `cargo-dylint` bump rebuilds the image;
and the container path has measured traps for this package family,
see
"Three measured traps of a fresh environment" below.

Leaves open:
which packages the gate covers,
and whether the gate replaces the rule in local runs or duplicates it.

### Option D: a skip stops being a pass

Design,
in three separable pieces:

- D1,
  a status:
  a run that emitted any skip exits with a status of its own,
  neither `0` nor the `2` of an incomplete run,
  for example `3`,
  so a shell can tell "checked and clean" from "not checked".
- D2,
  a code:
  a skip is reported under a code of its own,
  for example `rust/avoidable-closure-skipped`,
  or under the rule's code plus a field such as `"checked": false`,
  so a count by code cannot quietly include or exclude it.
- D3,
  a summary:
  the run prints one line at the end,
  "N package(s) not checked by rust/avoidable-closure",
  which is what a reader of a long JSONL would have seen if it had been a single finding.

Pros:
cheap in code and immediate in effect,
and it is the only option that protects every future count without needing any
driver;
D2 alone fixes the "skip counted as a finding" ambiguity that makes a count by code wrong in both directions;
measurable today with `--max-warnings 0`,
which already turns a skip into exit `1`
(measured 2026-11-27T21:0x -05:00 on `package/wg/cli-quicker-exempt`).

Cons:
it contradicts a recorded owner answer.
`doc/decision/rust-closure-lint-dylint.md`,
"Amendment of 2026-11-07T22:19:43-05:00:
a skip is a warning",
records that a skip "counts against `--max-warnings` like any warning,
and `--quiet` and `--silent` hide it with the other warnings",
so D needs an amendment,
not only code.
D1 also breaks every host run of the lint until a driver exists:
that would be every Rust package on this machine,
and every Rust pull request in CI,
where the rule has never had a driver at all.
D2 changes a public code string in a JSONL contract.

Leaves open:
whether D1 is on by default or opt-in,
and whether CI may pass `--max-warnings 0` instead of a code change.

### Option E: publish the driver as a release asset and install it as a tool

Design:
the plan already installs tools from GitHub release assets with a sha256 per platform
(`toolchain.plan.jsonc`,
the `age` row at `:162-175` is the shape).
A workflow job on the locked nightly would build Dylint's driver package,
upload `dylint-driver` as an asset of a repository release,
and a `github:` row would install it into a directory the launcher already exposes,
with `DYLINT_DRIVER_PATH` pointing there.

Pros:
one build per pair,
no compiler in the installer,
no per-machine manual step,
and an artifact whose digest the lock can record;
the linter's check passes with no code change.

Cons:
it adds a release pipeline and a downloadable binary that is loaded into `rustc` on every developer machine,
which is a trust decision the owner has to make explicitly;
a `cargo-dylint` or nightly bump needs a fresh asset before anyone can lint,
so the pipeline becomes a prerequisite of
the bump;
the asset is only valid with the exact `rustc-dev` of the toolchain it was built against,
so a machine whose active toolchain differs still cannot use it.

Leaves open:
who builds and publishes,
and whether the toolchain lock may name a driver that does not exist yet.

### Option F: pre-fill the caches so the first run builds the driver itself

Design:
extend `fetch:rust-dependencies` to also fetch the throwaway driver crate
(`dylint_driver = "=<cargo-dylint version>"` with `anyhow` and `env_logger`),
so `driver_unresolved` succeeds offline,
and let the first `cargo dylint` build the driver.

Pros:
no new provider,
no new artifact,
and the linter's own skip check becomes true;
this is exactly "Fetch then warn" applied to the second cache the message names
(`doc/decision/rust-closure-lint-dylint.md`,
the 2026-11-07 amendment).

Cons:
its disqualifying problem is measured on this machine:
a driver build does not run under a `cargo` that is not the rustup proxy,
because Dylint removes `RUSTUP_TOOLCHAIN` before the build and the throwaway crate's `rust-toolchain.toml` is honoured
only by rustup
(`doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md`,
"Root cause" and "What does not work");
this account has no rustup-proxy `cargo` on `PATH` (measured above).
So F alone leaves the run failing at the driver build,
or skipping again,
and only helps machines that do have a proxy.
It also lacks `jiff-static` 1.0.3 today,
so its fetch would have to cover that too (measured above).

Leaves open:
whether a wrapper `cargo` could be made to pass the toolchain through,
which is a change to `cargo-shadow`.

### Option G: fold the criterion back into the linter

Design:
drop the Dylint library and compute the closure criterion in the linter's own type analysis.

Pros:
no driver,
no nightly,
no `cargo-dylint`,
no skip,
and the rule becomes as portable as the rest of the linter.

Cons:
this is the option the owner rejected on 2026-10-18,
from three,
in favour of a rustc-driver lint
(`doc/decision/rust-closure-lint-dylint.md`,
"Accepted decision"),
and the linter has no rustc driver of its own,
which is the machinery the driver provides;
`doc/planning/linter-selective-anonymous-functions.md:249-265` records why the alternatives lost.
It would also lose the `expect(.., reason)` suppression and the `unfulfilled_lint_expectations` signal that the rule
inherits from rustc.

Leaves open:
nothing this round;
it is not a candidate unless the owner reopens the 2026-10-18 decision.

### What every option that provisions a driver pays

These costs attach to A,
B,
C,
E and F,
and no option avoids them.

- The driver is pinned to the exact `cargo-dylint` version and compiled against the nightly's `rustc-dev`,
  so any move of either invalidates it
  (`dylint/src/driver_builder/mod.rs:93-116`,
  `:192-205` at `v6.4.0`;
  both requests are `@latest` in `toolchain.plan.jsonc:236`,
  `:243`).
- The build needs a toolchain holding `llvm-tools-preview` and `rustc-dev`
  (`dylint/src/driver_builder/mod.rs:237-247` at `v6.4.0`),
  and a `cargo` that lets the throwaway crate's `rust-toolchain.toml` decide the toolchain,
  or the full triple passed as `RUSTUP_TOOLCHAIN`.
- The build downloads crates the first time.
  A recorded run compiled the driver and the crate in 24.6 s
  (`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
  "Verification",
  arm C);
  another compiled a 6.3.1 driver in 58.3 s (same file,
  "Applied on this host").
  The driver cache was 42 MB after the first run
  (`package/linter/monochromatic-lint-closure/README.md:428`).
- The build also clones `rust-lang/rust-clippy` with `git`,
  and that clone is not optional:
  `dylint_driver`'s build script reads `clippy_utils/src/sym.rs` at a revision it picks from the channel
  (`dylint_internal/src/clippy_utils/repository.rs:13-59`,
  `dylint_driver-6.4.0/build/extra_symbols.rs:51-99`).
  Measured this round in the container:
  88 s and 111 MB for that clone,
  and 178 s for the whole warm step.
  A machine that builds a driver needs a writable temporary directory for `git`,
  not only network and a cargo cache.
  The driver binary itself was 43,970,800 bytes and the driver directory 16 MB.
- The check itself costs a `cargo dylint` per package,
  which compiles the library and the package:
  `doc/planning/repository-linter-stays-at-zero.md:105` records 89.65 s for a run restricted to one Rust file,
  because the whole package was checked,
  while `package/linter/monochromatic-lint-closure/README.md:412-414` records 3.12 s for a repeat run of one crate
  with the library already built.
- The registry cache must hold the driver crate's whole tree.
  Today it does not hold `jiff-static` 1.0.3 (measured above).

## Three measured traps of a fresh environment

An option that builds a toolchain or an image meets three traps,
each measured this round in a container and each reported above with the attempt that found it.

### The library needs `llvm-tools`, and the error names LLVM

A nightly installed with `rustc-dev` and without `llvm-tools-preview` builds the lint library up to the link step and
then fails:

```text
rust-lld: error: unable to find library -lLLVM-23-rust-1.102.0-nightly
collect2: error: ld returned 1 exit status
```

That error was measured this round in the container,
at 2026-11-28T02:02 UTC,
on `nightly-2026-11-16` where the toolchain had `rustc-dev` only.
The library links `librustc_driver` and needs the LLVM shared library beside it,
which is the component Dylint's throwaway crate asks rustup for,
`llvm-tools-preview` (`dylint/src/driver_builder/mod.rs:237-247` at `v6.4.0`).
This repository's plan already carries it as `llvm-tools` (`toolchain.plan.jsonc:205`),
so nothing in the repository is wrong;
the trap is for anyone provisioning a driver outside the plan,
and it is why an option that builds a toolchain must name both components.

### The driver build needs a writable temporary directory

`dylint_driver`'s build script clones `rust-lang/rust-clippy` into a directory `tempfile` has just created,
and the image's `git` could not use `/tmp` for that.
With `TMPDIR` under the container's own writable tree the same clone returned `0`,
and it was the only change between the attempt that failed and the one that worked,
both above.
The cause of the refusal was not established.

### Every platform's crates, not only this one's

After a failed warm run,
the linter's probe still skipped the package with
`failed to download anstyle-wincon v3.0.11`,
a Windows-only crate.
The throwaway driver crate is resolved for every platform,
so a registry filled by a Linux build is not enough for the probe
(`README.md:496-500`).

## Ranking

Ranking:
A > E > B > C > D > F > G.

- A above E:
  both give findings on any machine that runs their provider,
  and A has no pipeline to build first,
  so a bump of either version cannot leave the repository or a machine without a driver for a day,
  while E cannot install a driver that has not been built and published yet.
- E above B:
  E needs no per-machine manual step and no cache outside the repository,
  so a second machine and a fresh worktree get findings without anyone remembering anything,
  while B warms only the machine that ran the task and only until the next `@latest` move.
- B above C:
  B gives findings for every package a local run selects,
  which is what someone counting findings on their own machine needs,
  while C gives them only for the packages one gate names.
- C above D:
  C produces findings somewhere,
  and D produces none anywhere;
  D only stops the absence from reading as a pass.
- D above F:
  D costs an owner amendment and no build,
  and it protects every count immediately,
  while F spends a fetch that still leaves this machine unable to build the driver,
  because its `cargo` is not the rustup proxy.
- F above G:
  F keeps the accepted decision of 2026-10-18 and could be dropped without losing anything,
  while G reopens that decision and rebuilds the machinery it rejected.

Two things follow from the ranking that are not options of their own.
D2,
the skip's own code,
is worth taking whatever else is picked,
because it is what makes a count by code correct in both directions.
And D1 is the only part of D that conflicts with the 2026-11-07 answer,
so an owner who wants the honesty without the red runs can take D2 and D3 and leave D1.

## What the owner is asked to pick

1. Which provider of the driver:
   A (the toolchain install),
   E (a published asset),
   B (the linter's task),
   or C (a gate that owns its driver)?
   The ranking above prefers A,
   then E,
   then B,
   then C,
   and A is blocked only by the mpm-shadow work the owner has already named as its precondition.
2. Whatever is picked,
   does a skip keep exit status `0` (the 2026-11-07 answer),
   gain a code of its own (D2),
   print a closing count (D3),
   or end the run with a status that is not a pass (D1,
   which needs the 2026-11-07 amendment)?
3. If B is picked,
   may `mise run lint:monochromatic-lint` write `~/.dylint_drivers` and use the network,
   and may it fail when the network is not there?
4. If A or E is picked,
   who owns the driver's staleness record in `toolchain.lock.json`,
   given that no archive digest exists for a compiled binary?
5. Whichever is picked,
   is the follow-up run over the four packages the issue names
   (the wrapper,
   `mpm-shadow`,
   `cargo-shadow`,
   the linter) part of that work,
   and does its count go into `doc/planning/repository-linter-stays-at-zero.md`
   or a record of its own?

## What this round did not do and cannot say

- It built nothing and changed no source,
  task,
  plan,
  lock or workflow.
- It did not run `mise run toolchain:install`,
  install any tool on this account,
  or run any `cargo dylint` on this account that would download or build a driver.
  The one `cargo dylint` this round ran was inside the container,
  where it wanted its own toolchain anyway.
- It did not verify component by component which nightly holds `rustc-dev` here;
  it inferred that from the two toolchains' sizes on disk and says so above.
- It did not measure how long a warm run takes on this account,
  because that would have written
  `~/.dylint_drivers` and `~/.cargo`.
- It did not run the rule over all four packages the issue names.
  The container run covers one package,
  chosen small,
  and it found `6` findings there,
  so what it proves is that the path works,
  not what the rule would find in the wrapper,
  `mpm-shadow` or `cargo-shadow`.
  Those three counts remain open,
  and they are the follow-up the issue asks for after the owner picks.
- Open question it could not answer alone:
  whether the owner wants the driver under the per-user home,
  where Dylint puts it by default,
  or inside the repository cache,
  where several worktrees could share it.
