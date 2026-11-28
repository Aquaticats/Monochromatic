# Who provides the Dylint driver `rust/avoidable-closure` needs, and what a skipped rule returns

Purpose:
answer,
with file and line and with measurements,
who puts the Dylint driver on a machine and how,
what the linter does when each piece is absent,
what a skipped rule returns to a reader or a script,
and what this machine has today;
then set out the decisions the owner has to make,
each option designed to its disqualifying problems,
ranked by what it delivers for a person who edits Rust here and for a pull request.
Status:
a proposal,
unapproved and non-final,
for issue 6643.
This revision was written after the clock read of 2026-11-28T04:00:48-05:00.
It answers a same-vendor gate read of 2026-11-27,
which is a second pass by another Claude instance and not an independent review;
the section "What the gate read found,
and what happened to each finding" answers that read finding by finding.
This round measured and changed nothing that ships:
no source of `package/linter/monochromatic-lint`
or `package/linter/monochromatic-lint-closure`
that a release or a test compiles,
no `monochromatic-lint.config.jsonc`,
no `toolchain.plan.jsonc`,
no `toolchain.lock.json`,
no `mise.toml`,
and no workflow.
Besides this file,
the round adds only the evidence folder
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/`,
which holds every log the measurements below cite,
with the command that produced it.
Every claim about behaviour carries what it rests on:
a `path:line`,
a command run with its date,
or a log in that folder.
Where a claim is inference,
it says so.

## The question the issue records

Issue 6643 is titled
"monochromatic-lint:
rust/avoidable-closure is skipped on every host run
because its Dylint driver is absent,
and a skip reads as a pass".
It opens:

"`rust/avoidable-closure` runs on no host run of the linter on the coordinating session's machine,
so every count of findings taken there leaves that rule out.
Filed by the coordinating session of the Git wrapper work."

Its section "Why it matters" says:

"The rule exists and its findings are unknown for every Rust package here.
A skipped rule looks like a passing one unless the reader knows to look for the line,
and the repository's rule for a count asks that its scope be checked before a zero is trusted."

And its record of the harm:

"Rounds and second passes of 2026-11-27 each noted the skip and went on,
and a run "without findings" was read as clean more than once before the line was noticed."

The issue asks three things:
who puts the driver on a machine,
the options with what each costs,
and one run of the rule over the four packages the work owns with its findings counted.

## How the linter runs the rule

`rust/avoidable-closure` is not a rule of the linter's own analysis.
`package/linter/monochromatic-lint/README.md:384` states it runs the type-aware Dylint lint `avoidable_closure`
of `package/linter/monochromatic-lint-closure` over each Rust package that has a selected file,
and names `doc/decision/rust-closure-lint-dylint.md` as the decision.
The root configuration selects it for every Rust file,
with the library path relative to the configuration file
(`monochromatic-lint.config.jsonc:33-39`,

```jsonc
"rust/avoidable-closure": {
  "severity": "warn",
  "library": "package/linter/monochromatic-lint-closure",
},
```

After the per-file rules,
`run_closure_step_at` (`package/linter/monochromatic-lint/src/closure_step.rs:169`) does this,
in order:

1. It groups the selected Rust files by the nearest `Cargo.toml` above them
   (`src/closure_packages.rs:108`,
   `group_packages`),
   so the unit of work is the package and not the file,
   and a directory whose name ends in `.fuzz` or `.bench` is exempt
   (`src/closure_packages.rs:90`).
2. It returns a note and nothing else under `--fix`
   (`src/closure_step.rs:177-180`),
   because the lint has no machine fix and a rewrite would make the compiler's positions stale.
3. It reports a configuration error,
   at processing-failure severity,
   when the configured library directory holds no `Cargo.toml`
   (`missing_libraries` at `src/closure_step.rs:102`,
   called at `:186-189`);
   that is a fault of the repository and not of the machine.
4. It runs `cargo dylint --version` once
   (`src/closure_step.rs:191`,
   `dylint_probe` at `src/closure_cargo.rs:191`).
5. It decides on a machine-wide gap:
   a `cargo` program that cannot be started at all,
   or `cargo dylint --version` answering with cargo's own `no such command` for `dylint`,
   skips every selected package
   (`ProbeFailure::Absent`,
   `dylint_probe` at `src/closure_cargo.rs:191`,
   the spawn failure at `:198-203` and the unknown-subcommand answer at `:205-215`);
   a `cargo` that starts and exits 126 or 127,
   or fails without that evidence,
   fails the run with status `2`
   (`ProbeFailure::Broken`,
   `src/closure_cargo.rs:216-246`);
   a compiler without the `rustc-dev` component skips every selected package
   (`rustc_dev_missing` at `src/closure_cargo.rs:291`,
   the machine gap at `src/closure_step.rs:202-213`).
   While `MONOCHROMATIC_LINT_CARGO` names a replacement program,
   the component check does not run
   (`src/closure_cargo.rs:291-296`),
   because the replacement stands for the whole toolchain.
6. It checks the Dylint driver once per distinct library
   (`driver_gaps` at `src/closure_driver.rs:278`,
   called at `src/closure_step.rs:214`),
   and skips the packages of a library whose driver cannot be had
   (`src/closure_step.rs:214-230`).
7. For every remaining package it reads the prerequisites,
   checks the locked dependencies offline,
   and then runs the lint
   (`check_package` at `src/closure_step_package.rs:203`):
   a missing prerequisite or a cold cargo cache skips that package
   (`src/closure_step_package.rs:227-246`).

The run itself is one process per package
(`run_dylint` at `src/closure_cargo.rs:385`):
`cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked --message-format json`,
with `CARGO_NET_OFFLINE=true` (`src/closure_cargo.rs:400`)
and `DYLINT_RUSTFLAGS="-W avoidable_closure -W unfulfilled_lint_expectations"`
(`src/closure_cargo.rs:62`).
Only the library and the binaries are compiled,
so `#[cfg(test)]` modules,
`tests/`,
examples and benchmarks are not checked
(`README.md:430-466`).

### What `cargo dylint` itself does, and in which order

The linter calls one program;
`cargo dylint` then does three things,
and the order matters for every option below.
At upstream tag `v6.4.0`
(the version `toolchain.lock.json:570` records):

1. It computes the toolchain name once,
   as the first word of `rustup show active-toolchain` run in the library directory
   with `RUSTUP_TOOLCHAIN`,
   `CARGO` and `RUSTC` removed
   (`internal/src/rustup.rs:35-45`,
   called for a `--path` library at `dylint/src/library_packages/mod.rs:409`).
2. It builds the lint library before it looks for a driver.
   The library is built while the `--path` library is resolved
   (`dylint/src/name_toolchain_map/maybe_library.rs:63` calls
   `library_packages::build_library`),
   and only afterwards does `check_or_fix` ask the driver builder for a driver
   (`dylint/src/lib.rs:382`).
   The measured order in the container of this round was the library first
   ("Building workspace metadata entry `monochromatic-lint-closure`",
   then the library's link step),
   and `~/.dylint_drivers` was still absent when the library build failed
   (`.../m1-f1-nonproxy-cargo/log/warm.err.dylint-link-failure.txt`,
   `.../m1-f1-nonproxy-cargo/run.txt`).
   The library is built with `cargo build --release` into the package's own
   `target/dylint/libraries/<toolchain>` directory,
   with `RUSTUP_TOOLCHAIN`,
   `CARGO` and `RUSTC` removed and `RUSTFLAGS` cleared
   (`dylint/src/library_packages/mod.rs:501-520`;
   `internal/src/rustup.rs:15-21`).
3. It builds the driver when `<drivers>/<toolchain>/dylint-driver` is absent or reports a version below
   `cargo-dylint`'s
   (`dylint/src/driver_builder/mod.rs:48-67`,
   `:93-116`),
   from a throwaway crate that pins `dylint_driver = "=<version of cargo-dylint>"`
   and whose `rust-toolchain.toml` names the toolchain and the components
   `llvm-tools-preview` and `rustc-dev`
   (`:192-238`),
   with the same three variables removed (`:143-150`) and an absolute rpath into
   `<the toolchain's rustup path>/lib` (`:132-136`).
4. It checks the package with `RUSTUP_TOOLCHAIN` set to the computed name
   and `RUSTC_WORKSPACE_WRAPPER` set to the driver
   (`dylint/src/lib.rs:420-434`).

Two consequences of that order and those sanitised environments are what this round measured,
and they are the reason the machine has no findings from the rule:

- The library build runs with `RUSTUP_TOOLCHAIN` removed,
  and the library's own `.cargo/config.toml` sets `rustflags = ["-C", "linker=dylint-link"]`
  (`package/linter/monochromatic-lint-closure/.cargo/config.toml:1-2`).
  `dylint-link` reads `RUSTUP_TOOLCHAIN` itself
  (`dylint-link/src/main.rs:50`,
  `:65`),
  so without a `cargo` that puts the variable back (a rustup proxy does),
  the library's every link fails with
  `Error: environment variable not found: RUSTUP_TOOLCHAIN`
  (measured,
  `.../m1-f1-nonproxy-cargo/log/warm.err.dylint-link-failure.txt`;
  `.../m2-f2-proxy-cargo/log/lib-nonproxy.err.dylint-link.txt`).
- The compiler that builds the library and runs the check is the one the `cargo` in `PATH`
  resolves to,
  not the toolchain the linter's caller exported as `RUSTUP_TOOLCHAIN`
  (`RUSTUP_TOOLCHAIN` is removed for the build and the check's own value is set by Dylint):
  a direct toolchain link selected the dated compiler,
  and a rustup proxy selected rustup's default
  (measured:
  `.../m2-f2-proxy-cargo/log/execve-counts.txt` shows 132 executions of the floating
  toolchain's `rustc`,
  and `.../m1-f1-nonproxy-cargo/log/execve-counts.txt` shows the dated one).

## What each absence produces

Every absence below is a distinct branch in the source,
and they do not behave the same.

- No `cargo`,
  or no `cargo-dylint` behind it:
  `ProbeFailure::Absent` (`src/closure_cargo.rs:192-215`),
  whose only accepted evidence is cargo's own `no such command` for `dylint`
  (`src/closure_cargo.rs:135`,
  `README.md:475-486`).
  The run skips every selected package.
- A `cargo` that starts and exits 126 or 127,
  or fails without that evidence:
  `ProbeFailure::Broken` (`src/closure_cargo.rs:216-246`),
  a `core/processing-failure` finding and exit status `2`,
  because a skip that exits `0` would hide a stale install (`README.md:529-546`).
- A compiler without `rustc-dev`:
  a machine-wide skip
  (`rustc_dev_missing` at `src/closure_cargo.rs:291-316`,
  `src/closure_step.rs:202-221`).
- A driver that is absent or older than `cargo-dylint`,
  with the pinned `dylint_driver` crate unresolvable offline:
  one skip per package of that library
  (`driver_state` at `src/closure_driver.rs:178`,
  `driver_gaps` at `:278`).
- A package whose locked dependencies are not in the cargo cache:
  a skip naming `cargo fetch --locked`
  (`dependencies_missing` at `src/closure_cargo.rs:320`).
- A prerequisite named in `monochromatic-lint.prerequisites.jsonc` that the machine lacks:
  a skip naming the task that makes it
  (`src/closure_probe.rs:247`,
  `README.md:551-614`).
- A check that cannot be made at all,
  such as no readable `cargo dylint --version`,
  no `rustup`,
  or no known driver directory:
  a note that appears only under `--debug`,
  and the packages still run
  (`src/closure_driver.rs:185-187`,
  `:306-309`,
  `README.md:518-522`).

Two more states exist that the list above does not cover,
and both were measured in this round:

- A `cargo dylint` run can fail after the probe passed,
  because the library build or the check failed for a reason no probe reads
  (the missing `RUSTUP_TOOLCHAIN` of `dylint-link`,
  or a library that the compiler cannot compile).
  That is a `core/processing-failure` finding and status `2`,
  not a skip (`src/closure_step_package.rs:266-271`).
- The linter's driver probe can answer `Ready` while the run still cannot build the library,
  because the probe reads the driver and the crate cache and never builds the library
  (measured,
  `.../m5-option-k-end-to-end/log/no-file-linter.jsonl`).

## What a skipped rule returns

This is the part the issue's title asks for,
and it is decided by three places in the code.

1. The skip is a diagnostic of the same rule code as a real finding:
   `Diagnostic::new(RULE, Severity::Warn, ...)` at `src/closure_step_package.rs:100-123`,
   where `RULE` is `rust/avoidable-closure`.
   Its file name is the package's `Cargo.toml` and its position is the first byte of it
   (`:111-112`).
   Its message starts `rust/avoidable-closure skipped <package>: ` and ends
   "Nothing in this package was checked by this rule,
   so a run without findings says nothing about its closures."
   (`:104-113`,
   the last sentence at `:109`).
2. Skips are merged into the same finding list as findings before output
   (`src/run_command.rs:204`,
   `findings.extend(closure.skips)`),
   so a consumer that reads only the code cannot tell them apart;
   the file can:
   a skip names the package's `Cargo.toml`,
   and a real finding of this rule names a `.rs` file
   (`findings_from` at `src/closure_findings.rs:178`).
3. The exit status counts the skip as a warning and nothing else
   (`exit_status` at `src/run_output.rs:58-86`):
   a warning alone leaves `0`;
   `--max-warnings` counts it,
   and exceeding the limit gives `1`;
   only a finding marked as a processing failure gives `2`.
   `--quiet` and `--silent` hide it with the other warnings
   (`src/run_output.rs:93-99`,
   `README.md:523-528`).

Measured in the container of this round,
with the released `monochromatic-lint` 0.5.2-alpha.24
(`toolchain.lock.json:612-616`)
over the copied `package/wg/cli-quicker-exempt`,
with no driver present and a cold registry
(`.../m5-option-k-end-to-end/log/skip-default.jsonl`,
`skip-max-warnings.jsonl`,
`skip-quiet.jsonl`):

- The default run printed one JSONL record,
  `{"code":"rust/avoidable-closure","severity":"warn","filename":"pkg/Cargo.toml",...}`,
  and exited `0`.
- `--max-warnings 0` printed the same record and exited `1`.
  A gate can therefore make a skip fail today with no code change,
  at the price of failing every package that skips.
- `--quiet` printed nothing and exited `0`,
  so the quiet form of a run looks exactly like a clean run.
- `--rules` reports the rule as not suppressible with one option,
  `library`
  (`src/run_modes_rule_table.rs:116-122`;
  the installed binary printed
  `{"id":"rust/avoidable-closure","fixable":false,"fixClass":null,"options":["library"],"suppressible":false}`
  on 2026-11-27 at 21:12 -05:00 in the previous round),
  so a project cannot turn a skip off without turning the whole rule off.
- A skip's message names the command that would fill the caches,
  built by `warm_command` (`src/closure_driver.rs:236`):

  ```sh
  env --unset=CARGO_NET_OFFLINE cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked
  ```

  and the record of 2026-10-30 shows that this command,
  run by hand with network access,
  is what filled the driver cache then
  (`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md:335-358`).

## Who provides the driver today

No component of this repository builds or installs a Dylint driver.
Each part below states what it does and does not do.

### The toolchain plan, the lock and the install task

`toolchain.plan.jsonc` holds three rows that matter here,
and none of them is a driver:

- `rust` (`toolchain.plan.jsonc:184-217`):
  the floating nightly,
  installed through rustup with the components `clippy`,
  `rust-src`,
  `llvm-tools`,
  `rustc-dev` and `rustfmt`
  (`:202-207`),
  the targets `aarch64-linux-android` and `x86_64-linux-android` (`:209-212`),
  and `RUSTUP_TOOLCHAIN` set to `{version}` in the row's environment (`:214-216`).
- `cargo:cargo-dylint` (`:235-241`):
  `cargo:cargo-dylint@latest` through `cargo-binstall`,
  exposing the bin `cargo-dylint`.
- `cargo:dylint-link` (`:242-248`):
  the same for `dylint-link`.

`toolchain.lock.json` records what those rows resolved to on this machine:
`nightly-2026-11-16` with the same five components (`toolchain.lock.json:493-528`),
`RUSTUP_TOOLCHAIN` as `nightly-2026-11-16` (`:511-513`),
`cargo-dylint` 6.4.0 (`:568-577`)
and `dylint-link` 6.4.0 (`:579-589`).
The task that installs all of it is `toolchain:install` (`mise.toml:624`);
`toolchain:install:global` (`mise.toml:684-694`) is a different task,
it converges the machine-global tools with `mpm-shadow install --global`.
The comment in `[hooks]` (`mise.toml:56`) calls `bootstrap` a thin alias of `toolchain:install`,
but no `bootstrap` task exists in `mise.toml`,
and the hook is commented out in favour of daily development (`mise.toml:60-61`).
`rg --line-number --ignore-case 'dylint|driver' toolchain.plan.jsonc toolchain.lock.json mise.toml`
finds the three rows above,
no plan row,
lock field or task that names `.dylint_drivers`,
`dylint-driver` or `dylint_driver`,
and `clippy-driver` in two `bins` lists.

So the install provides the two tools and the compiler component,
and stops one step short of the artifact the rule needs.

### The linter's own message and its warm command

The skip message names the command that would fill the caches
(`src/closure_driver.rs:218-235`),
and the linter's own check is offline by construction:
`dylint_drivers_directory` joins `.dylint_drivers` under the home directory
(`src/closure_driver_probe.rs:69-71`),
`driver_site_from_environment` reads `DYLINT_DRIVER_PATH` (`:110-125`),
`active_toolchain` asks `rustup show active-toolchain` in the library directory with `RUSTUP_TOOLCHAIN`,
`CARGO` and `RUSTC` removed (`:162-202`),
`driver_version` runs the driver's `-V` (`:205-228`),
and `driver_unresolved` resolves a throwaway crate that pins `dylint_driver = "=<version>"`
with `cargo metadata --offline` (`:249-267`,
`resolve_probe` at `:302-340`).
The linter reads the driver cache,
mirrors Dylint's staleness rule,
and refuses to write anything (`README.md:495-528`).

### The tasks that do name the rule's needs

- `//package/linter/monochromatic-lint:test:host`
  (`package/linter/monochromatic-lint/mise.toml:77-80`)
  is the one ignored binary test,
  `host_real_run_over_wg_quicker_exempt`
  (`src/closure_binary_tests.rs:728-781`),
  which asserts a real run over `package/wg/cli-quicker-exempt` has no skip and at least one finding,
  and whose positive control runs the same library with `--all-targets`
  to show that the absence of test findings can be told from a probe that could not find them
  (`:760-780`).
  It needs `cargo-dylint`,
  the `rustc-dev` nightly and a warm cargo cache.
- `//package/linter/monochromatic-lint-closure:run`
  (`package/linter/monochromatic-lint-closure/mise.toml:53-75`)
  runs `cargo dylint --path <this package> --manifest-path <directory>/Cargo.toml -- --all-targets`
  with no offline setting,
  so it is a warming command as well as a lint:
  it builds the driver when none is current
  (`dylint/src/driver_builder/mod.rs:61-64`).
- The root `lint:monochromatic-lint` task (`mise.toml:1057-1062`) depends on `fetch:rust-dependencies`
  (`mise.toml:1059`),
  which runs `cargo fetch --locked` per package (`mise.toml:1031-1051`).
  That fills the package locks and not the driver crate,
  which belongs to no lockfile (`src/closure_driver.rs:4-27`).

### The workflows

`rg --line-number --ignore-case 'dylint' .github` returns no hit at all,
so no workflow names the tool,
the driver or the rule.
The one workflow that runs the linter is `.github/workflows/advisory-core.yml`,
and it cannot start the linter today.

- The `advisory core` job installs the repository tools with the closed name list
  `tools: file-enforcer mpm-shadow cargo-shadow` (`:95`),
  and `package/monorepo-task/core-exec/src/repo-tools-setup-archives.ts:45-50`
  accepts exactly `mpm-shadow`,
  `file-enforcer`,
  `cargo-shadow` and `git-policy-cli`
  (`repo-tools-setup.ts:138-140` rejects any other name).
  `monochromatic-lint` is not one of them,
  and nothing in the job installs it.
- The "Static changed paths" step (`:190-214`) calls
  `task-run lint:monochromatic-lint -- "${files[@]}"` (`:213`).
  The last retrievable log of that step in which it reached the call,
  as the gate read of 2026-11-27 found it,
  is run `41274002049` of 2026-11-01,
  and it printed
  `.github/CODE_OF_CONDUCT.md`,
  then `task-run: line 1: monochromatic-lint: command not found`,
  then `Error: task-cli: step exited 127`
  (`.../ci/ci-run-41274002049-static-step-excerpt.txt`).
  The step reports `success` only because of `continue-on-error: true` (`:192`);
  the "Enforce advisory result" step fails the run on the step's outcome (`:247-270`).
  The linter never started,
  so no package was probe-checked in CI at all.
- The job installs the locked Rust toolchain with
  `rustup toolchain install "$toolchain" --profile minimal --no-self-update` (`:152-159`),
  which carries neither `rustc-dev`,
  nor `llvm-tools`,
  nor `cargo-dylint`.
  `task-run` does not see the toolchain environment either
  (`doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md:25-35`),
  so an exported `DYLINT_DRIVER_PATH` or `RUSTUP_TOOLCHAIN` in the lock would not reach the step.
- The recent history is silent:
  `gh run list --workflow advisory-core.yml --limit 200` returned 200 `push` runs in the window the gate read
  checked,
  199 of them with the `advisory core` job skipped by the workflow's own budget gate
  (`:44-69`,
  budgets at `:61`),
  as the gate read recorded on 2026-11-27.

The first thing missing in CI is therefore not a driver but the linter,
and after that the toolchain components,
the proxy `cargo` and the network for a first driver build.
Every provider option below pays that same CI cost.

## The machine's toolchain state

Read-only looks on 2026-11-28 between 03:07:05 and 03:11:10 -05:00,
all in `.../machine/machine-state.txt`:

- `rustup toolchain list`:
  `nightly-x86_64-unknown-linux-gnu (active, default)` and
  `nightly-2026-11-16-x86_64-unknown-linux-gnu`.
- The floating default's installed components are `cargo`,
  `clippy`,
  `rust-std`,
  `rustc` and `rustfmt`:
  it has neither `rustc-dev`,
  nor `llvm-tools`,
  nor `rust-src`.
  Its `rustc` is `1.102.0-nightly (5df8c02af 2026-11-16)`.
- The dated toolchain the lock records has `cargo`,
  `clippy`,
  `llvm-tools`,
  `rust-src`,
  the two Android `rust-std` targets,
  `rustc`,
  `rustc-dev` and `rustfmt`.
  Its `rustc` is `1.102.0-nightly (5f4effd57 2026-11-13)`.
- On disk the floating one is 274M and the dated one 799M.
- `type -a cargo` lists this repository's shadow
  (`/home/user/Monochromatic/.cache/cargo-shadow-bin/cargo`),
  then `/home/user/Monochromatic/.cache/toolchain-bin/cargo`,
  then `/usr/bin/cargo`.
  `readlink -f /usr/bin/cargo` is `/usr/bin/rustup`,
  so a rustup-proxy `cargo` is installed system-wide and is on `PATH`,
  behind the other two.
- `.cache/toolchain-bin/cargo` and `.cache/toolchain-bin/rustc` are links straight into the dated toolchain
  (`.../nightly-2026-11-16-x86_64-unknown-linux-gnu/bin/...`),
  so on this host the first `cargo` and the first `rustc` are both non-proxy programs of the dated toolchain.
- `ls -d /home/user/.dylint_drivers` answers
  "No such file or directory",
  so no driver exists now;
  the records below show one existed before.

The 2026-11-25 record said the opposite of the third point,
and the coordinating session corrected it on 2026-11-27:
that record now holds both the original sentence
("this host has no rustup proxy in `PATH` at all",
`doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md:113-115`)
and the correction that `/usr/bin/cargo` is a link to `/usr/bin/rustup`.
Its second workaround also says the proxy arm was never run
("so the arm could not be exercised",
`:204-206`).
This round ran it,
in a container,
and the results are below.

Three toolchain names are in play,
and the rest of this proposal keeps them apart:

- the floating channel,
  `nightly-x86_64-unknown-linux-gnu`,
  which is rustup's default here and has no `rustc-dev` or `llvm-tools`;
- the dated name the lock records,
  `nightly-2026-11-16`,
  whose full form is `nightly-2026-11-16-x86_64-unknown-linux-gnu`
  and which carries the components;
- the name `rustup show active-toolchain` prints for the library directory with `RUSTUP_TOOLCHAIN` removed,
  which is the floating one today and the one Dylint files a driver under.

The driver directory the linter's own message names is
`/home/user/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver`
(the issue's capture,
and the same message reproduced in the container of this round),
so a driver filed under `nightly-2026-11-16` (the directory the recorded workaround used)
is never read here.

## What this round measured

Every container ran as rootless `podman`,
each inside its own
`systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`,
from the image `docker.io/cachyos/cachyos-v4:latest`
with rustup,
`strace`,
`jq`,
`binutils` and the two Dylint tools the lock records installed.
Each container was removed at the end of its run,
and no container of this round is left.
Nothing of this account's home was mounted:
the library,
the test package,
the released linter binary and the registry cache were copied in with `podman cp`.
The scripts are in
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/scripts/`,
and every log is in the folder named under each measurement.

Each instrument was first shown able to show a difference:

- `strace -f -e trace=execve` recorded the `cargo`,
  `rustc`,
  `rustup` and `git` programs each run executed,
  and the tables below are counts from those logs.
- The clone counter showed 3 `git` executions (version,
  clone,
  checkout) for a dated-name driver build
  and 0 for the channel named `nightly` (`m2` and `m5` logs below),
  with the same flags.
- The driver-load probe ran the driver where its RUNPATH resolves and where it does not (M4).
- The linter's skip and its findings were both produced in the same container,
  before and after a driver existed (M5).
- `--max-warnings 0` and the default run gave different statuses in the same container (M5).

### M1: the machine's state with a non-proxy `cargo` first (F1)

Folder:
`.../m1-f1-nonproxy-cargo/`.
The container had the floating nightly installed minimal,
the dated toolchain with `rustc-dev` and `llvm-tools`,
`RUSTUP_TOOLCHAIN=nightly-2026-11-16` exported as the lock's environment sets it,
and `/opt/nonproxy/cargo` and `/opt/nonproxy/rustc` (links into the dated toolchain,
the shape this host's `.cache/toolchain-bin` has) first on `PATH`.
The released `monochromatic-lint` 0.5.2-alpha.24 and a copy of
`package/linter/monochromatic-lint-closure` and `package/wg/cli-quicker-exempt` were the inputs.

- Control:
  in the library directory with `RUSTUP_TOOLCHAIN` removed,
  `rustup show active-toolchain` printed
  `nightly-x86_64-unknown-linux-gnu (default)`;
  with it set it printed `nightly-2026-11-16-x86_64-unknown-linux-gnu (overridden by environment variable RUSTUP_TOOLCHAIN)`
  (`log/active-without-env.txt`,
  `log/active-with-env.txt`).
  The same probe can therefore show either name.
- The linter printed one skip whose message names
  ``the Dylint driver for toolchain `nightly-x86_64-unknown-linux-gnu` at
  /root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver``,
  and exited `0` (`log/linter-skip.jsonl`).
  That is the issue's line,
  reproduced.
- The warm command failed in the library build:
  `error: linking with `dylint-link` failed: exit status: 1` ... `Error: environment variable not found: RUSTUP_TOOLCHAIN`
  (`log/warm.err.dylint-link-failure.txt`),
  and `~/.dylint_drivers` did not exist afterwards (`m1-run.txt`),
  so no driver build was even attempted.
  The `execve` log shows 16 executions of the dated `rustc` and none of `git`.
- A driver build for the dated name with a proxy `cargo` was then attempted;
  it cloned into
  `/tmp/.tmpEjtkLF` and failed with
  `fatal: destination path '/tmp/.tmpEjtkLF' already exists and is not an empty directory`,
  the trap the 2026-11-27 round met (`log/dated-driver.err.last-12.txt`).

What it settles:
the toolchain name Dylint files a driver under here is the floating one;
the directory the linter reads is `<home>/.dylint_drivers/nightly-x86_64-unknown-linux-gnu`;
a non-proxy `cargo` cannot build the library at all,
so in this state no compiler builds the library
and no check runs,
whatever driver exists;
and the recorded workaround's dated-name directory is not the one read.

### M2a: a proxy `cargo` first, and the floating nightly still minimal

Folder:
`.../m2a-proxy-alone-fails-at-rustc-dev/`.
With the proxy first (`/usr/sbin/cargo`,
a link to `/usr/bin/rustup`) and nothing else changed,
the library build failed with
`error[E0463]: can't find crate for `rustc_driver``
on 132 floating-`rustc` executions
(`log/warm.err.e0463.txt`,
`log/execve-counts.txt`):
the toolchain rustup selects for the library directory has no `rustc-dev`.
The linter skipped and no driver existed (`log/linter-skip.jsonl`).

What it settles:
the proxy alone is not enough;
the toolchain the library build selects must carry the components.

### M2b: a proxy `cargo` first and the components on the floating nightly

Folder:
`.../m2-f2-proxy-cargo/`.
The floating nightly received `rustc-dev`,
`llvm-tools-preview` and `rust-src` in 24 s
(`log/component-add.txt`).
It is the nightly of 2026-11-27 (`rustc 1.102.0-nightly (c82ce1563 2026-11-27)`).

- The library build then failed with `error[E0603]` in `dylint_linting` 6.4.0
  (`log/lib-proxy.err.e0603.txt`):
  a nightly newer than the compiler `dylint` 6.4.0 was built against cannot compile the library.
- The same library build with the non-proxy `cargo` failed in 1 s at `dylint-link`
  (`log/lib-nonproxy.err.dylint-link.txt`),
  the M1 failure isolated.
- A driver built directly for the dated name with the proxy succeeded in 140 s,
  with 3 `git` executions (version,
  clone,
  checkout) (`log/dated-driver.err.last-4.txt`,
  `log/dated-driver-execve.git-lines.txt`).
- The same driver build with the non-proxy `cargo` failed in 46 s with
  `Error: environment variable not found: RUSTUP_TOOLCHAIN`
  (`log/nonproxy-driver.err.last-8.txt`).

What it settles:
adding the components to the floating channel is not enough while the channel is newer than the
`dylint` release supports;
the non-proxy failure is a property of `dylint-link` and the driver build script,
not of the library source;
and the clone belongs to the dated name.

### M5: the configuration that runs the rule end to end (K), and the skip's statuses

Folder:
`.../m5-option-k-end-to-end/`.
The container had the proxy first and a copy of the library carrying
`rust-toolchain.toml` with `channel = "nightly-2026-11-16"` and the components.
`rustup show active-toolchain` in that directory printed the dated full name
(`log/k-active-without-env.txt`).

- The skip's statuses,
  measured before any driver existed (the skip here is the machine-wide
  `rustc-dev` gap,
  because with the proxy in front `rustc --print sysroot` is the floating one):
  the default run printed one warning record and exited `0`;
  `--max-warnings 0` exited `1`;
  `--quiet` printed nothing and exited `0`
  (`log/skip-default.jsonl`,
  `log/skip-max-warnings.jsonl`,
  `log/skip-quiet.jsonl`,
  the last is 0 bytes).
- The driver build for the channel named `nightly` (the F5 arm) failed after 82 s with `error[E0603]`
  in `dylint_driver` itself,
  with 0 `git` executions (`log/nightly-driver.err.tail.txt`,
  `log/execve-counts.txt`):
  the build script ran (it fails the build only later,
  at compile time) and did not clone,
  because the channel is exactly `nightly`.
- `cargo dylint` with the library's toolchain file printed
  ``Building driver for toolchain `nightly-2026-11-16-x86_64-unknown-linux-gnu` ``
  (`log/k-warm.err.building-driver.txt`),
  built the library in 53.41 s (`log/k-warm.err.finished.txt`),
  then built the driver in 1 m 39 s with 3 `git` executions (`log/k-warm.err.finished.txt`,
  `log/execve-counts.txt`),
  and left `/root/.dylint_drivers/nightly-2026-11-16-x86_64-unknown-linux-gnu/dylint-driver`
  whose `-V` is `nightly-2026-11-16-x86_64-unknown-linux-gnu 6.4.0`
  (`log/k-driver-tree.txt`,
  `log/k-driver-version.txt`).
  The compiler that built the library and ran the check is the dated one:
  241 executions of the dated `rustc` and 4 of the floating one in the same log;
  the caller of those four was not traced and is not claimed.
  The driver's dynamic section says
  `RUNPATH [/root/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib:/root/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib]`
  and `NEEDED librustc_driver-5adf19dcd2f7d879.so`
  (`log/k-driver-readelf.txt`).
  The two RUNPATH entries are an artefact of this round's script,
  which had exported a floating
  `RUSTFLAGS` for the arm above;
  Dylint's own entry is the dated one.
- The linter,
  offline,
  over the same package with that library found 6 findings at the positions the
  2026-11-27 round found and exited `0` (`log/k-linter.jsonl`):
  `src/application_targets.rs:202:33`,
  `src/application_watch_mark_reconciliation.rs:40:14`,
  `:47:17`,
  `:62:14`,
  `src/keeper_process_identity.rs:142:19`,
  `src/update_routing_engine_construction_and_marks.rs:72:39`.
- The same linter with the original library (no toolchain file) still skipped:
  the probe reads the floating name,
  whose driver is absent,
  and its offline resolution failed on
  `failed to download `anstyle-wincon v3.0.11``
  (`log/no-file-linter.jsonl`).

What it settles:
K is a configuration in which the name,
the compiler and the driver agree,
and the rule runs;
the check is the dated compiler;
the first run pays a clone plus a driver build;
and the linter's probe can skip for a cache reason that is not the driver,
which is a defect of the
probe's message rather than of the driver.

### M6: the probe's answer, and a driver built without Dylint's rpath

Folder:
`.../m6-clean-driver-and-probe/`.

- A driver built by a bare `cargo build` for the dated name (75 s,
  3 `git` executions,
  `log/clean.err.tail.txt`) has no RUNPATH at all,
  and running it fails with
  `error while loading shared libraries: librustc_driver-5adf19dcd2f7d879.so: cannot open shared object file`
  (`m6-run.txt`).
  Dylint supplies the rpath through `RUSTFLAGS` (part 1's citation),
  so a provider that builds a
  driver outside Dylint must supply the same flag,
  and a hand-built driver without it cannot load.
- After `cargo fetch` of the probe's own manifest (4 s;
  it downloaded `windows-sys`,
  `r-efi` and `portable-atomic`,
  `log/probe-fetch.err`),
  the linter's offline resolution of the pinned driver
  crate succeeded (`cargo metadata --offline` exit 0,
  `log/probe-metadata.err` empty).
  With that,
  the linter over the original library (no toolchain file) stopped skipping:
  it reported a `core/processing-failure` and status `2` because the library build could not resolve
  `dylint_linting` offline in this container (`log/probe-vs-run.jsonl`).
  The driver probe had answered `Ready`,
  and the run could not build the library.
  The library's failure here was the cold registry,
  a different cause from M2b's `E0603`,
  but the same class:
  a probe that reads only the driver and the crate cache cannot promise that the
  run will compile the library.

What it settles:
a driver artifact must be built with Dylint's rpath flag or it does not load;
and the linter's `Ready` is not a promise that the run can compile the library.


### M4: the driver's RUNPATH, and a container whose toolchain sits elsewhere (F10)

Folder:
`.../m4-f10-rpath-and-other-home/`.
The driver M5 built (the one Dylint filed) was copied into a container of the base image,
whose `/root/.rustup` holds both toolchains.

- Control:
  run with `HOME=/home/other`,
  where the builder's paths exist,
  the driver exited `0`
  and printed `nightly-2026-11-16-x86_64-unknown-linux-gnu 6.4.0`
  (`log/control-rpath-present.txt`),
  and `readelf --dynamic` showed the RUNPATH and
  `NEEDED librustc_driver-5adf19dcd2f7d879.so` (`log/driver-rpath-and-needed.txt`).
- Then `/root/.rustup` was moved away and the same toolchain installed at `/opt/rustup`
  (install status `0`),
  where the same `librustc_driver-5adf19dcd2f7d879.so` exists (`log/other-path-lib.txt`).
  The driver run with `HOME=/home/other` and no `LD_LIBRARY_PATH` exited `127` with
  `error while loading shared libraries: librustc_driver-5adf19dcd2f7d879.so: cannot open shared object file: No such file or directory`
  (`log/other-path-run.txt`).
- Positive control:
  the same run with
  `LD_LIBRARY_PATH=/opt/rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib`
  exited `0` and printed the same version (`log/other-path-with-ld-library-path.txt`).

What it settles:
the RUNPATH is absolute to the builder's toolchain path;
the same toolchain at another path does not satisfy it,
and only an added library path does.
A published asset therefore loads only where the builder's path exists,
unless a wrapper supplies `LD_LIBRARY_PATH`.


### M3: which of the thirty selectable lock directories still skip with a driver present (F3)

Folder:
`.../m3-f3-skips-with-a-driver-1/` and `.../m3-f3-skips-with-a-driver-2/`.

The set is `git ls-files '*Cargo.lock'` (39 tracked locks)
minus the nine whose directory ends in `.fuzz` or `.bench`
(`src/closure_packages.rs:90`),
so 30 directories.
Two containers ran fifteen each:
the proxy first,
the library naming the dated toolchain (K),
the host's registry cache copied in,
network available for a crate the cache lacked,
and one current driver for the dated name,
built by the warm run at the start of each container.
For each directory the script ran `cargo metadata --offline --locked`
(fetching with network first when that failed)
and then one linter invocation over that directory with the root configuration.
`log/results.tsv` has one row per directory,
with its status,
wall time,
metadata result and closure
skip records;
`log/closure-skips.txt`,
`log/closure-failures.txt` and `log/closure-findings.txt` are the same records
filtered to the closure rule.

All thirty ran,
and the closure rule's records say:

- Six packages skipped,
  and none for the driver:
  - `gtk4` is missing (`pkg-config --exists gtk4` failed),
    a declared prerequisite:
    `package/file-manager/app` and `package/file-manager/variant-gtk-sticky`.
  - `qmake6` and `qmake` are missing from `PATH`,
    a declared prerequisite:
    `package/file-manager/variant-qt`.
  - `cargo ndk-env --target arm64-v8a` failed,
    the `android-ndk` prerequisite:
    `package/music-player/android-app/rust`.
  - the cargo cache check came first and failed:
    `package/ide/app` and `package/terminal/app`,
    each because a git dependency could not be fetched in the container
    (`git fetch` failed for `helix-editor/helix` and for `Uzaaft/libghostty-rs`),
    so the offline dependency check skipped them before their prerequisite check ran.
- Two packages failed the closure check with status `2`:
  `package/browser-selector/desktop-app` and `package/music-player/desktop-app`,
  both because `fontconfig` is not installed
  (`yeslogic-fontconfig-sys` panics in its build script),
  and neither package declares that system library,
  so they fail instead of skipping.
- Seven packages produced closure findings:
  6 in `package/cargo-shadow/cli`,
  13 in `package/cpu-temperature/cli`,
  1502 in `package/git-policy/cli`,
  31 in `package/linter/monochromatic-lint`,
  72 in `package/mpm-shadow/cli`,
  6 in `package/wg/cli-quicker-exempt`,
  and 44 in `package/zfs/request-to-openzfs-kernel-device`.
  The 1502 are distinct positions in that package.
- Fifteen packages produced no closure finding and no skip.
- No record of any of the thirty names `cargo-dylint`,
  the driver directory or `rustc-dev`.

The run's status is every rule's,
not only the closure rule's,
so a status of `1` or `2` in `results.tsv` is not by itself the closure check's verdict;
the filtered files are the source of the counts above.

What it settles:
with a current driver and the dated toolchain,
twenty-two of the thirty were checked (fifteen clean and seven with findings)
and eight were not (six skips and two failures);
the remaining skips are machine facts no provider removes,
four of them a declared prerequisite and two a cache gap a network fetch would close;
a package whose system library its manifest omits fails with status `2` rather than skipping;
and the six packages with prerequisite manifests cannot all be relied on to skip for their declared
prerequisite,
because the cache check runs first.

## What the measurements imply

1. The toolchain name,
   the compiler that builds,
   and the components are three separate things,
   and no option may assume they agree.
   Dylint removes `RUSTUP_TOOLCHAIN` for the library build,
   the driver build and the packaged check
   (`internal/src/rustup.rs:15-21`),
   so the name it uses is `rustup show active-toolchain` in the library directory,
   and the compiler is whatever the `cargo` in `PATH` resolves to.
   On this host those were the floating name and the dated compiler (M1);
   under K and a proxy they are both the dated toolchain (M5).
2. Every option needs a `cargo` that is the rustup proxy,
   or one that otherwise restores
   `RUSTUP_TOOLCHAIN` for Dylint's children.
   Without it the library's own link step fails before any driver is consulted (M1,
   M2b),
   so no provider,
   task,
   asset or cache can deliver anything.
   The linter's `MONOCHROMATIC_LINT_CARGO` does not solve it:
   it replaces only the outer program and turns off the component check
   (`src/closure_cargo.rs:36-49`,
   `:291-296`),
   while Dylint spawns `cargo` by name from `PATH` (`internal/src/cargo.rs:142-146`).
3. Every option needs a toolchain that `dylint` 6.4.0 can compile against.
   The dated `nightly-2026-11-16` does (M5);
   a `nightly` installed on 2026-11-28 does not (M2b,
   M5):
   the library and the driver both fail with
   `error[E0603]`,
   and the floating channel without components fails with `E0463` (M2a).
   The compatibility of this host's own floating build (`5df8c02af`,
   2026-11-16) was not measured
   and is not known.
4. A driver must be filed under the name rustup reports in the library directory.
   The recorded workaround's `nightly-2026-11-16` directory is short of the full name,
   and the linter's probe never reads it (M1's message,
   M5's dated directory).
5. The clone is a cost of naming a dated toolchain (3 `git` executions,
   1 m 39 s for the whole driver build in M5),
   and the channel named exactly `nightly` avoids it,
   at the price of the compiler being the channel
   (M5's `nightly` arm could not compile the driver at all on 2026-11-27).
6. A published driver carries an absolute RUNPATH into the builder's toolchain (M5,
   M4),
   so it loads only where that path exists.
7. Eight of the thirty selectable packages are not checked with a driver present (M3):
   six skip,
   four of them for a declared prerequisite and two for a cache gap a network fetch would
   close,
   and two fail because a system library their manifests omit is missing.
   A status keyed on "any skip" is red for the six.
8. The linter's driver probe can answer "the driver is absent" for a cache reason that has nothing
   to do with the driver (`.../m5-option-k-end-to-end/log/no-file-linter.jsonl`),
   and it can answer `Ready` while the run still
   cannot build the library (M6).
9. CI cannot start the linter at all today,
   so every option's CI delivery waits on a workflow change
   that no provider option includes.

## The decisions, and the options under each

Three decisions are separable,
and this proposal keeps them apart.
A provider and a skip return are complements,
not alternatives:
a provider changes whether the rule runs,
and a skip return changes what an absent provider says.
Nothing in one ranking is a reason against an option in the other.

1. which toolchain builds the library and the driver,
   and how the machine makes rustup report it
   (options K,
   I,
   and the unmeasured M);
2. who provides the driver (options B,
   Z,
   A,
   C,
   H,
   E,
   F,
   with G excluded and L a complement);
3. what a skip returns (the record's contract,
   and options J,
   D2,
   D1,
   D3).

Every provider option below assumes the two measured prerequisites:
the proxy first for the whole `cargo dylint` tree,
and a toolchain the `dylint` release can compile against.
Neither is a provider,
and neither is optional.

### Decision 1: which toolchain builds the library and the driver

#### Option K: name the dated toolchain in the library directory

Design:
a `rust-toolchain.toml` in `package/linter/monochromatic-lint-closure` naming
`nightly-2026-11-16` (the name the lock records) and the components `llvm-tools-preview` and `rustc-dev`,
generated from `toolchain.lock.json` by file-enforcer if the file is to have one owner.
With it,
`rustup show active-toolchain` in the library directory answers the dated full name
instead of the floating one,
and the driver Dylint builds is filed under that name and read by the linter's probe (M5).

What it delivers:
the rule runs end to end (measured:
6 findings,
offline,
from the linter),
with the name,
the compiler and the driver all the dated toolchain.
For a person editing Rust,
the first run pays the driver build (1 m 39 s including the clone in M5)
and later runs reuse it until `cargo-dylint` moves or the driver stops loading.
For a pull request,
nothing until CI can start the linter,
and then the same build (the clone needs
network and `git`;
the CI caches Dylint recommends cover `~/.cargo/git/checkouts/` and `~/.dylint_drivers/`).

Disqualifying problem:
none was measured.
What it conflicts with is a recorded decision:
the owner adopted,
open to veto,
"the floating nightly stays (no pin)"
(`doc/decision/rust-closure-lint-dylint.md:182`),
and the same record lists "a pin of the nightly" among the things not approved (`:66-68`).
K is a package-level pin,
and it names the toolchain a second time beside the lock unless it is
generated.
The owner has to decide that conflict;
the proposal does not decide it.

Leave open:
whether file-enforcer generates the file (and from which lock field),
and whether the components list belongs beside the channel in it.

#### Option I: put the components on the floating channel

Design:
install `rustc-dev`,
`llvm-tools-preview` and `rust-src` into the floating channel that rustup reports
for the library directory (the 2026-10-18 approved component list,
`doc/decision/rust-closure-lint-dylint.md:51`),
and make the proxy `cargo` the first one for the lint's whole tree.
Then the name Dylint files under and the toolchain it compiles with are the floating channel,
which now carries the components.

What it delivers:
on a machine whose floating channel is compatible with the installed `dylint`,
the same delivery as K
with no new file and no pin.
The component add took 24 s (M2b) and 63 s (M5) from a warm network.

Disqualifying problem:
its delivery depends on the floating channel staying inside what the installed `dylint` compiles
against,
and that is measured to fail today:
on a `nightly` of 2026-11-27 with the components,
the library fails with `error[E0603]` (M2b) and so
does `dylint_driver` (M5's `nightly` arm);
without the components it fails with `E0463` (M2a).
So a machine that installs `nightly` afresh today cannot run the rule at all,
and every nightly
update can break it,
which is the risk the record already accepts in its warning that
"A floating-nightly upgrade can break the lint source"
(`package/linter/monochromatic-lint-closure/README.md`).
The compatibility of this host's current floating build is untested.

Leave open:
whether the floating channel is the toolchain the project wants the lint compiled with at all.

#### Option M: give `dylint-link` the toolchain through the library's cargo configuration

Design:
a `[env]` entry in `package/linter/monochromatic-lint-closure/.cargo/config.toml` that sets
`RUSTUP_TOOLCHAIN` to the dated toolchain with `force = true`,
so the library build's linker (`dylint-link`) receives the variable although Dylint removed it from
the `cargo` process.
The driver build would still need a proxy,
or a pre-built driver (a provider option).

What it would deliver:
the library build without a proxy `cargo`,
which is the smaller machine change.
Cost:
it names the toolchain in a second place (the same pin question as K),
and it depends on cargo's
configuration environment reaching the linker,
which was not measured.

What would settle it:
one container run of the non-proxy library build with that `[env]` entry,
and one of the whole
`cargo dylint` with a pre-built driver filed under the reported name.
It is listed and not ranked because it is not measured.

### What each answer to the pin question makes the provider options do

- If the nightly is not pinned (the record's adopted default,
  `doc/decision/rust-closure-lint-dylint.md:182`):
  I is the only toolchain configuration on the table,
  and every provider option depends on the floating channel staying inside what the installed
  `dylint` compiles against.
  Measured today,
  a freshly installed `nightly` is not (M2b,
  M5),
  so on a new machine or a runner the first step is to install a nightly the installed `dylint`
  supports,
  which is either recorded somewhere (a pin in effect) or a best-effort update that can break the
  lint again.
- If the dated toolchain is named for the library directory (K):
  the driver's directory is the dated one,
  the clone returns at each driver build (M5),
  and every machine and runner must have that toolchain installed (the plan already installs it).
  Every provider option then needs the dated toolchain and the proxy first.

The owner's answer decides which of those two cost tables applies.
The proposal does not answer it.

### Decision 2: who provides the driver

Each of these assumes Decision 1 (K,
or I where it works) and the proxy.

#### Option B: the linter's task warms the driver before the offline run

Design:
the root `lint:monochromatic-lint` task gains a step after `fetch:rust-dependencies` that runs the
warm command the skip message prints,
once per library directory,
without `CARGO_NET_OFFLINE`.
`fetch:rust-dependencies` (`mise.toml:1031-1051`) is the precedent:
it already fetches,
already needs the network,
and exists to stop a cold-cache skip.

What it delivers:
every local run gets a current driver without anyone remembering,
which is exactly the harm the issue
records;
it needs no installer and no new lock record.
The warm is one `cargo dylint` run (M5 measured 1 m 39 s of driver build plus 53 s of library build
on a cold container).

Disqualifying problem:
none was measured.
Its costs:
it writes `~/.dylint_drivers` and the cargo registry from a lint task,
which `mise run lint` has
never done;
it makes a lint run need the network and `git` at least once per `cargo-dylint` move or nightly update;
and the run itself still needs the proxy and K (or a compatible floating channel).
For a pull request it does nothing until CI can start the linter.

Leave open:
whether the task fails a lint run when the network is not there,
and who owns the staleness record.

#### Option Z: the documented manual warm (today's provider)

Design:
keep the current shape:
the skip message names the warm command,
`README.md:513-517` tells the owner
to run it once with network access and again after each `cargo-dylint` upgrade,
and the 2026-10-30 record shows it done with the owner's go-ahead
(`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md:335-358`).

What it delivers:
clean findings on a machine where someone ran the command,
with no new parts.
It is the only provider that exists today,
and it is measured to work once Decision 1 and the proxy hold.

Disqualifying problem:
none in delivery,
but it is the design the issue shows failing in practice:
a person has to remember after every `cargo-dylint` move,
and the record's own history has three
releases in three days (`:118-121`,
`:380-381`).
For a pull request it does nothing.

#### Option A: the toolchain install provides the driver

Design:
a new row in `toolchain.plan.jsonc` (say `dylint-driver`) that runs Dylint's build once for the
toolchain the library directory reports,
places the binary at
`<driver root>/<toolchain>/dylint-driver`,
and records the pair (cargo-dylint version,
toolchain) so either move invalidates it.
It needs the installer's `cargo` to be the proxy (or the build to be run with one),
and it needs the
driver root it exports,
which the launcher can write into
`.config/mise/conf.d/30-toolchain-env.local.toml` in each worktree
(`package/mpm-shadow/cli/src/toolchain/scope.rs:42`,
`package/mpm-shadow/cli/README.md:797`).
That file is per worktree and `task-run` does not read it
(`doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md:25-35`),
so A's environment route is a worktree-local one,
not a CI one.

What it delivers:
every local run after an install has a current driver without a network call or a compile,
on any machine that installed.
Its costs:
a compile inside the installer,
which the owner has paused (issue 6316);
a lock record for a compiled artifact with no archive digest;
and the same K/component and proxy prerequisites as every other option.

Disqualifying problem:
while the installer is paused it delivers nothing,
and its `cargo` is not guaranteed to be the proxy.

#### Option C: a gate that owns its driver

Design:
a container image that bakes the dated toolchain with the components,
`cargo-dylint` and `dylint-link`,
a driver built inside the image,
and the released linter,
plus one task that runs the lint over the packages the work owns (the wrapper,
`mpm-shadow`,
`cargo-shadow`,
the linter).
The image is not the shape of `package/linter/monochromatic-lint/test.Containerfile`,
which is a pinned stable image with `COPY vendor`,
`CARGO_NET_OFFLINE=true` and no nightly,
`rustc-dev` or Dylint (`test.Containerfile:1-14`);
a driver image needs network at build and a nightly with the components.

What it delivers:
findings for the named packages without touching this account's caches,
and,
because the linter travels in the image,
a pull-request gate that does not need the workflow's
closed tool list opened.
The measured traps a fresh image meets are in the evidence:
the library needs `llvm-tools` as well as `rustc-dev` (M2a),
and the driver build needs a writable temporary directory for `git` (M1's `/tmp` refusal).

Disqualifying problem:
none for a gate,
but it gives no finding for a local edit,
and a local `mise run lint` still reports the rule as skipped;
a nightly or `cargo-dylint` move rebuilds the image,
and the image build carries the clone and the
component install.

#### Option H: a CI job that follows Dylint's caching guidance

Design:
a job on the hosted runner that installs the lock's nightly with `rustc-dev`,
`llvm-tools-preview` and
`rust-src`,
installs `cargo-dylint` and `dylint-link` at the lock's versions,
puts the rustup proxy
`cargo` first,
adds K's toolchain file to the library directory (or sets rustup's default to the dated
name),
restores and saves `~/.cargo/git/checkouts/`,
`~/.dylint_drivers/` and `~/.rustup/toolchains/`
keyed on the two versions,
and runs the linter over changed Rust files.
The caching guidance is Dylint's own README at `v6.4.0`:

"If you run Dylint in CI,
we recommend caching certain directories to avoid rebuilding libraries,
rebuilding drivers,
or redownloading toolchains."
(`README.md:201-203`),
listing `~/.cargo/git/checkouts/`,
`~/.dylint_drivers/` and `~/.rustup/toolchains/` (`:209-216`).

What it delivers:
pull-request findings,
the surface the issue says never ran at all.

Disqualifying problem:
none for its own surface,
but it needs the linter on the job's `PATH` first
(the workflow's closed tool list,
`advisory-core.yml:95`),
and the same components,
proxy and network
as every other option;
its cost is unmeasured (no job was run).

#### Option E: publish the driver as a release asset

Design:
a workflow job on the locked nightly builds the driver and uploads it as a release asset,
and a `github:` row installs it with a sha256,
the shape of the `age` row
(`toolchain.plan.jsonc:162-175`).

What it delivers:
no compile on the machine,
and a digest the lock can record.

Disqualifying problem,
measured:
the driver carries an absolute RUNPATH into the builder's toolchain (M5:
two absolute entries;
M4:
it does not load where that path is gone and does load with `LD_LIBRARY_PATH` at the other copy),
so an asset is usable only on a machine whose toolchain sits at the builder's absolute path.
A wrapper that sets `LD_LIBRARY_PATH` would be a second mechanism;
the asset is also only valid with the builder's `rustc-dev`,
and the pipeline has to publish before anyone can move `cargo-dylint` or the nightly.
It also needs the same K/component and proxy prerequisites for the library build.

#### Option F: pre-fill the caches so the first run builds the driver

Design:
extend `fetch:rust-dependencies` to fetch the throwaway driver crate
(`dylint_driver = "=<cargo-dylint version>"` with `anyhow` and `env_logger`) for every platform,
so `driver_unresolved` succeeds offline and the first `cargo dylint` builds the driver.

What it delivers:
a cheaper first run for a machine that has the proxy and a compatible toolchain,
and it makes the probe's cache check true.

Disqualifying problem,
measured:
it provides no driver and cannot make the driver build work:
the build fails without the proxy (M2b:
`environment variable not found: RUSTUP_TOOLCHAIN`),
and the probe's resolution needs the Windows-only crates
(`anstyle-wincon`,
which the probe's message names in M5's `no-file` log).
It also does nothing for the library build.
It is a complement to B,
Z,
A or C,
not a provider.

#### Option G: fold the criterion back into the linter

Design:
drop the Dylint library and compute the closure criterion in the linter's own type analysis.

What it delivers:
no driver,
no nightly,
no `cargo-dylint`,
no skip.

Disqualifying problem:
it is the option the owner rejected on 2026-10-18 from three,
in favour of a rustc-driver lint
(`doc/decision/rust-closure-lint-dylint.md:5-9`),
and the in-linter route has measured coverage and cost problems:
with the shipped load mode 41 of 67 IDE closures stay reported because the callee does not resolve,
and 11 sit in `#[cfg(test)]` sidecars the session does not load
(`doc/planning/linter-selective-anonymous-functions.md:150-168`);
`BuildGenerated` executes the build script of every crate in the graph,
unsandboxed,
and the mode is
not the one the executable requests (`:277-281`,
`:304-360`).
It also loses rustc's `expect` suppression and the `unfulfilled_lint_expectations` signal.
The old proposal's reason ("the linter has no rustc driver of its own") was wrong:
the linter carries a rust-analyzer-based semantic engine (`RustFileEngine`,
`src/run_command.rs:180`);
the conclusion stands on the measured coverage and sandboxing limits.

#### Option L: pin `cargo-dylint` to an exact version

Design:
replace `@latest` for the two Dylint tools (`toolchain.plan.jsonc:236`,
`:243`) with an exact request,
the 2026-10-30 record's option D
(`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md:409-417`).

What it delivers:
a bump becomes deliberate,
so whichever provider is picked rebuilds only when the owner moves the pin,
and the "three releases in three days" recurrence stops.

Disqualifying problem:
none on its own,
and it provides no driver.
It is a complement to any provider,
and the owner decides whether to take it.

### Decision 3: what a skip returns

The record's contract is the baseline:
a skip is a warning finding of `rust/avoidable-closure` at the package's `Cargo.toml`,
counted by `--max-warnings` and hidden by `--quiet` (`README.md:487-528`,
`doc/decision/rust-closure-lint-dylint.md:243-253`).
The owner's words,
from `doc/decision/rust-closure-lint-dylint.md:236-237`:

> in their own words and not as one of the offered options:
> "Fetch then warn."

The record's reading of them (`:239-253`),
whole:

> This reads as two rules:
>
> - The repository's own lint tasks fetch the Rust dependencies first (`mise run fetch:rust-dependencies`),
>   so the cold cache that caused the skip is filled before the closure check reads it.
> - A skip that remains is reported as a warning-level finding of `rust/avoidable-closure`,
>   at the package's `Cargo.toml`,
>   naming the package and the command that fixes it.
>   It is not a silent message with exit status 0,
>   and it is not an error:
>   it counts against `--max-warnings` like any warning,
>   and `--quiet` and `--silent` hide it with the other warnings.
>
> A machine-wide gap (no `cargo-dylint`,
> no `rustc-dev`) and a missing driver produce one such finding per package,
> so the report shows how many packages went unchecked.
> Failures that are not a missing prerequisite keep exiting with status 2.

"Fetch then warn."
is the owner's sentence.
Every sentence of the block above is the record's reading
(the record introduces it with "This reads as two rules"),
not a further statement by the owner.

Measured:
the three statuses above,
and a skip whose file is the `Cargo.toml` while a finding's is a
`.rs` file.
Options J and D1 touch the reading's "it is not an error" and its fixed severity;
D2's code variant touches "a warning-level finding of `rust/avoidable-closure`";
D3 touches neither.

#### Keep the contract

What it delivers:
no change;
`--max-warnings 0` can already fail a skip (measured),
and the driver-gap skip is one
finding per package (`doc/decision/rust-closure-lint-dylint.md:251-253`).

Where it fails:
the quiet form looks clean (measured:
`--quiet` prints nothing and exits 0),
and a reader who does not know to look at the message still reads a skip as a pass,
which is the issue's harm.

#### Option J: a configurable severity for a skip, by cause

Design:
a rule option (say `skipSeverity`) beside `library`,
so a configuration block can make a driver-gap
skip an `error` and leave prerequisite skips at `warn`.
`exit_status` already returns `1` for an error finding (`src/run_output.rs:75-77`),
and `--quiet` hides only warnings (`:97`),
so a quiet run no longer looks clean at error severity.
It needs the skip's cause to be carried:
it can be,
because `DriverState::Missing` is told from the
package's other skips in `driver_gaps` (`src/closure_driver.rs:278-330`).

What it delivers:
a gate can fail on the machine-wide gap and stay green on the six prerequisite packages (M3),
with no new exit status and no new code string.

Disqualifying problem:
none measured;
it needs a rust change,
tests and a README change,
and it amends the record's fixed severity (the config decides it).

#### Option D2: a code of its own, or a field

Design:
either a skip's code becomes `rust/avoidable-closure-skipped`,
or the record keeps the rule's code and
gains a field such as `"checked": false`.

What it delivers:
a count by code cannot quietly include a skip.

Disqualifying problem:
the code variant conflicts with the record ("a warning-level finding of `rust/avoidable-closure`",
`:243-249`),
with the code comment that keeps the findings searchable by the rule name
(`src/closure_step_package.rs:90-94`),
and with the README (`README.md:487-489`).
The field variant avoids all three,
and its delivery is smaller than the old proposal claimed:
a skip already names a `Cargo.toml` and a finding a `.rs` file,
so a consumer can tell them apart
today by file.

#### Option D1: a status of its own for a skip

Design:
a run that emitted any skip exits with a status of its own,
neither `0` nor the `2` of an incomplete
run,
for example `3`.

What it delivers:
a shell can tell "checked and clean" from "not checked" without parsing records.

Disqualifying problem:
if it is keyed on any skip it is red for the packages that skip for a declared prerequisite on a
machine that lacks gtk4,
Qt or the NDK,
and for the packages whose cache is short (M3),
and red in CI for the same packages;
and red in CI for the same packages;
the record says a skip is not an error (`:247`),
so it also needs the amendment.
Keyed by cause (driver gap only) it delivers what J delivers,
with a new public status instead of an
error finding,
which `exit_status` already handles.

#### Option D3: a closing count

Design:
the run prints one closing line,
"N package(s) not checked by rust/avoidable-closure".

What it delivers:
a person reading a long JSONL sees the count.

Disqualifying problem:
it must not go to stdout,
which is JSONL (`src/run_output.rs:111-119`);
the record already gives one finding per package so the report shows how many went unchecked
(`:251-253`),
so D3 duplicates that on a second channel.
It is harmless at `warn` and hidden by `--quiet` unless it goes to stderr,
where the contract does not
cover it.

## Ranking

The measure is what each option delivers to the two readers the repository's rules name:
a person who edits Rust here,
and a pull request.
The edit loop is weighed first,
because a pull request's lint is the same rule on the same machine class,
and because every option's CI delivery waits on a workflow change none of them includes.
Effort to build is a cost,
not a reason to rank lower.

Toolchain configurations:
**K > I**.
K delivers on any machine that has the dated toolchain installed,
whatever the floating channel is,
and the dated name is the one the lock records;
I's delivery depends on the floating channel staying inside what the installed `dylint` compiles
against,
and on a fresh machine today it fails (M2b,
M5).
K's cost is the pin the owner declined (Decision 1's option K states it).
M is not ranked because it is not measured.

Providers:
**B > Z > A > C > H > E > F**.

- B over Z:
  both give every local run a current driver;
  B removes the step a person has to remember,
  which is the harm the issue records,
  while Z is the manual warm that was skipped in practice.
  B's costs (network and a cache write from a lint task) are stated,
  not used to rank it lower.
- Z over A:
  Z works today with the measured configuration;
  A's provider is the paused installer,
  and A adds a lock record for a compiled artifact.
  A otherwise delivers what Z does without the memory step.
- A over C:
  A gives every package a local run selects a driver once installed;
  C gives only the packages its gate names,
  and gives nothing for a local edit.
- C over H:
  C carries the linter in the image,
  so it delivers a pull-request finding without opening the
  workflow's closed tool list and can also be run locally;
  H needs the linter installed in the job and the runner's toolchain arranged,
  and delivers nothing
  locally.
- H over E:
  H's delivery does not depend on a filesystem path;
  E's does,
  and is measured not to load where the builder's toolchain path is absent (M4).
- E over F:
  E delivers a finding on a machine whose toolchain sits at the builder's path;
  F delivers no finding anywhere (M2b,
  M5).

G is not ranked:
the owner rejected it,
and the in-linter route's measured limits are above.
L is a complement to any provider,
not an alternative.

What a skip returns:
**J > D2 (field) > D1 (keyed by cause) > D3 > keep**.

- J over D2:
  J changes what a gate does (a driver-gap skip can fail a run) which answers the issue's harm for CI;
  D2 changes what a script can count,
  not whether a run fails.
- D2 over D1:
  D2's field keeps the code,
  the severity and the exit status,
  and makes a count correct;
  D1 adds a new public status for what an error finding already carries.
- D1 over D3:
  D1 makes a shell see the difference;
  D3 prints a count a person reads.
- D3 over keep:
  D3 at least names the count on a channel the contract leaves free;
  keep leaves the quiet form
  looking clean.

## What the gate read found, and what happened to each finding

The gate read of 2026-11-27,
kept whole in
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/gate-read/review.md`
and read as data,
raised four High,
six Medium and six Low findings.
Each is answered here with what this round ran or read.

### F1 (High), three toolchain names in play

Right,
and sharpened.
The review's inference was that a non-proxy `cargo` might still compile the library and the check;
the measurement shows it cannot get past the library's link step at all (M1,
M2b),
and that the name Dylint files under is the floating one while the compiler that would run is the
dated one (M1).
The proposal now names the three names in its own section,
states the components per toolchain (measured,
not inferred from size),
names the directory segment each option fills,
and puts the pin question to the owner.
The `nightly` versus dated clone difference is measured (M5,
M2b).
The review's own inferred detail ("Dylint spawns `cargo` by name") is confirmed in
`internal/src/cargo.rs:142-146` and by the `execve` logs.

### F2 (High), a rustup proxy exists

Right.
`type -a cargo` lists the shadow,
`.cache/toolchain-bin/cargo` and `/usr/bin/cargo`,
and `/usr/bin/cargo` is a link to `/usr/bin/rustup` (machine state,
2026-11-28).
The old proposal's sentence is removed,
and this round ran the proxy arm in containers (M2a,
M2b,
M5).
The 2026-11-25 record's correction is quoted,
and its "could not be exercised" arm is now measured.
What the review called a design not weighed is now Decision 1 plus the proxy prerequisite.

### F3 (High), every skip treated alike

Right.
`git ls-files '*Cargo.lock'` counts 39,
`rg -c '\.(fuzz|bench)/'` counts 9,
so 30 are selectable,
and six of those carry prerequisite manifests (machine state and `git ls-files`).
M3 measures which of the thirty still skip with a driver present and why:
four skip for a declared prerequisite,
two for a cache gap,
and two fail for an undeclared system
library.
D1 is now keyed by cause in the ranking,
and those packages are named.

### F4 (High), CI cannot start the linter

Right.
The excerpt is quoted (the step's `monochromatic-lint: command not found`,
exit 127),
the closed tool list and the missing install are cited,
and the proposal now says every option pays that CI cost,
not only option A.
The line numbers the old proposal used were corrected against the current workflow.

### F5 (Medium), the clone is optional for the channel named `nightly`

Right.
The source returns `None` for `nightly` (`driver/build/extra_symbols.rs:51-70`),
and the measurement agrees:
0 `git` executions for the `nightly` arm and 3 for the dated one
(M5,
M2b).
The costs section now says the clone belongs to the dated name,
and the crate-relative citation paths are replaced by repository paths at the tag.

### F6 (Medium), where the installer writes its environment

Right.
The file is `.config/mise/conf.d/30-toolchain-env.local.toml` in each worktree
(`package/mpm-shadow/cli/src/toolchain/scope.rs:42`,
`package/mpm-shadow/cli/README.md:797`),
and `task-run` does not read it.
Option A now states that scope,
and the old E-over-B reason that leaned on it is withdrawn (the
provider ranking is new anyway).

### F7 (Medium), the 2026-11-07 amendment

Right.
The amendment's sentences are quoted whole in this proposal,
marked as the owner's words and the record's reading,
and D2's conflict is stated for both variants:
the code variant conflicts,
the field variant does not.
The exit-status half is no longer presented as the owner's choice,
and D3's stream is named.

### F8 (Medium), missing options

Right.
H,
I,
J,
K,
L and Z are designed above,
and M is added as an unmeasured design.
The status quo (Z) is now stated as an option and the baseline.

### F9 (Medium), precedent not engaged

Right.
The 2026-10-30 ranking is cited (`B > C > A > D`,
`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md:362-363`) and its option C is
the family of today's A;
this proposal's provider ranking differs because the measured prerequisites change the delivery:
the probe B landed does not make the rule run (M1's skip),
and the installer-owned warm still needs
the proxy and a compatible toolchain.
"Has never been created" is replaced by "does not exist now",
with the 2026-10-30 warm and the
2026-11-07 counts (516 findings,
508 warnings,
`doc/planning/repository-linter-stays-at-zero.md:19`) beside it.

### F10 (Medium), the asset's absolute rpath

Right.
M5 shows absolute RUNPATH entries and a `NEEDED` library from the dated toolchain;
M4 shows the driver does not load where the builder's toolchain path is gone and loads with
`LD_LIBRARY_PATH` at the other copy.
Option E's "no code change" claim is withdrawn:
it is path-bound.

### F11 (Low), upstream statements

Right,
`dylint_drivers` errors on a `DYLINT_DRIVER_PATH` that is not a directory (`ensure!` at `:72`)
and writes the README only in the home branch (`:77-86`);
the driver build is quiet only when the caller asks (`:143-144`);
the README text is at `:19-25`;
and the 42 MB is this repository's measurement,
not Dylint's estimate
(`package/linter/monochromatic-lint-closure/README.md:428`).
Those statements are corrected here.

### F12 (Low), repository citations

Right;
the ranges and names are corrected against the current files
(the probe's functions at `closure_driver_probe.rs:69`,
`:110`,
`:162`,
`:205`,
`:249`,
`:302`;
`mise.toml:56` and `:684`;
`README.md:487-528` for the skip's contract,
and `closure_driver_probe.rs:302-340` for the every-platform resolve,
where the old proposal cited an unrelated README passage for it;
the pin record's release dates at `:118-121`,
`:380-381`;
the host test at `closure_binary_tests.rs:728-781` with its positive control at `:760-780`;
the in-linter costs at `linter-selective-anonymous-functions.md:150-168`,
`:277-281` and `:304-360`).

### F13 (Low), statements that contradict their source

Right,
and all five are fixed:
the closure `:run` task warms the driver rather than requiring one (`mise.toml:53-75`);
a skip's file is the `Cargo.toml` and a finding's is a `.rs` file;
the root lint task is on an automatic path (`advisory-core.yml:213`);
a `cargo` that cannot be spawned is `Absent` and skips,
while status `2` is for a `cargo` that starts
and fails without cargo's missing-subcommand evidence;
and the 2026-11-23 answer in issue 6316 is a relay by the coordinating session,
which the proposal no
longer turns into an owner quote about the pause.

### F14 (Low), timestamps and counts

Right;
this file carries one status clock read and second-precise times,
the counts match the sections they describe,
and the container times below are the container clock,
not the earlier round's minute figures.

### F15 (Low), option G and option C

Right;
G's reason is restated from the in-linter route's measured limits,
and C is described as a new image,
not as the shape of `test.Containerfile`.

### F16 (Low), unlabelled inference

Right;
the host test's today-failure is labelled as inference (this round did not run it),
and the earlier round's libgit2 cause for the clone refusal is labelled as inference
(the `/tmp` refusal's mechanism was not established;
this round met it again and used `TMPDIR`).

The gate's verdicts on the old ranking's adjacent pairs are answered by the new ranking:
the old A-over-E reason (which was partly build effort) is replaced by the rpath and publish
measurements;
the old E-over-B reasons (per-machine manual step,
no cache outside the repository,
a fresh worktree)
do not hold and the order is reversed;
C over D no longer treats a provider and a skip return as alternatives,
which is why the two are ranked
separately now;
D over F rested on F2 and on F alone fetching nothing,
and F is now a complement;
F over G holds.

## What the owner is asked to pick

1. The toolchain configuration:
   K (the library names the dated toolchain,
   measured to work,
   a pin the record did not approve) or
   I (the components go on the floating channel,
   no pin,
   measured not to work with a fresh nightly
   today).
   M is described but not measured.
2. Whether the lint's whole `cargo` tree may use the rustup proxy
   (through the launcher or `mise` environment,
   or through a linter change that puts the proxy first),
   which every provider needs.
3. The provider:
   B (the lint task warms),
   Z (the documented manual warm),
   A (the installer),
   C (a gate image),
   H (a CI job),
   or E (an asset);
   and whether L (pin `cargo-dylint`) comes with it.
4. What a skip returns:
   keep the contract,
   J (a severity per cause),
   D2 (a field,
   or a code),
   D1 (a status keyed by cause),
   or D3 (a closing count).
5. Whether the 2026-11-23 answer in issue 6316 covers a plan change (a component install or a driver
   row) or only running the installer.
6. Whether the workflow's closed tool list is opened for `monochromatic-lint`
   (needed by H and by every provider's CI delivery),
   or a gate job is added (C).
7. Whether the follow-up run over the four packages the issue names happens,
   and whether its counts go into `doc/planning/repository-linter-stays-at-zero.md` or a record of
   their own.

## Measured, inferred, and not known

Measured in this round:
the machine's toolchains,
components,
versions and `PATH` order;
the four states M1,
M2a,
M2b and M5 end to end;
the skip's three statuses;
the clone counts for both channel names;
the driver's `-V`,
its RUNPATH and its `NEEDED` library;
the six packages' prerequisites and the skip causes M3 records;
the rpath's effect on loading (M4);
and the CI excerpt and workflow configuration as they stand.

Inferred,
and labelled where it appears:
the floating channel's compatibility with this host's exact floating build (untested);
that cargo's `[env]` reaches the linker (option M,
unmeasured);
that the ignored host test would fail today for the reason a host run skips
(the earlier round inferred it and this round did not run it);
and that the clone refusal's mechanism is the container's temporary directory rather than AppArmor
(the setting that avoids it is measured;
the mechanism is not).

Not known:
which provider the owner wants;
whether the owner wants the pin;
how long a whole-repository run over all thirty packages takes (M3's coverage is stated with its
result);
what a CI job would cost in wall time;
and whether a floating-nightly update would break a working K setup (it should not,
because K names
the dated toolchain,
but that was not run across an update).

## What this round did not do

- It changed nothing that ships:
  no source a release or a test compiles,
  no configuration,
  no plan,
  lock or workflow.
- It ran no `cargo dylint` on this account,
  built no driver on this account,
  and wrote nothing under `~/.cargo`,
  `~/.rustup` or `~/.dylint_drivers`;
  every `cargo dylint`,
  driver build and linter run happened in a container.
- It ran no `mise run toolchain:install`,
  no `pnpm install`,
  no release and no workflow.
- It did not run the ignored host test `host_real_run_over_wg_quicker_exempt`.
- It did not measure option M,
  option H's CI job or option C's image.
- It did not touch `doc/handover/`,
  `doc/decision/`,
  or the 2026-11-25 troubleshooting record whose
  correction the coordinating session made.

## The evidence folder

Every log this proposal cites is in
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/`:

- `machine/machine-state.txt`:
  the host's read-only facts,
  with each command.
- `issue/issue-6643.md`:
  the issue as `gh issue view` printed it.
- `ci/ci-run-41274002049-static-step-excerpt.txt`:
  the step's last retrievable log.
- `m1-f1-nonproxy-cargo/`,
  `m2a-proxy-alone-fails-at-rustc-dev/`,
  `m2-f2-proxy-cargo/`,
  `m5-option-k-end-to-end/`,
  `m6-clean-driver-and-probe/`,
  `m4-f10-rpath-and-other-home/`,
  `m3-f3-skips-with-a-driver-1/`,
  `m3-f3-skips-with-a-driver-2/`:
  each measurement's logs,
  under the names the text uses.
- `scripts/`:
  the in-container scripts,
  so each run can be repeated.
