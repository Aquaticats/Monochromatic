# Who provides the Dylint driver `rust/avoidable-closure` needs, and what a skipped rule returns

Purpose:
answer,
with file and line and with measurements,
who puts the Dylint driver on a machine and how,
what the linter does when each piece is absent,
what a skipped rule returns to a reader or a script,
and what this machine has today;
then set out the decisions the owner has to make,
each one answerable alone,
each option designed to its disqualifying problems,
ranked by what it delivers for the linter's target workload.
Status:
a proposal,
unapproved and non-final,
for issue 6643.
The owner declares completion;
no part of it is approved,
and the options here are not adopted.

This revision was written after the clock read of 2026-11-28T07:00:36-05:00.
It answers two same-vendor gate reads,
each a second pass by another Claude instance and neither an independent review:

- the read of 2026-11-27,
  which raised 4 High,
  6 Medium and 6 Low findings,
  kept whole in
  `package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/gate-read/review.md`;
- the read of 2026-11-28,
  which raised 4 High,
  8 Medium and 6 Low findings,
  kept whole with its probes in
  `package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/gate-read-second-2026-11-28/`
  (`review.md` and the `probe-*.txt` files beside it;
  that read's own scratch folder expires a day after its last change).

The section "The two gate reads,
finding by finding" answers the second read's findings one by one,
with the line or the run each answer rests on,
and says which findings this revision found wrong.

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
the round adds only the evidence folder below and the logs written into it.

Every claim about behaviour carries what it rests on:
a `path`,
a symbol or heading in that file,
a command run with its date,
or a log in that folder.
Where a claim is inference,
it says so.
Quotations are copied from their file and checked against it by
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/scripts/check-quotations.ts`
over `scripts/quotations.tsv`.

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
The third is met in the form the merge now allows:
the run of 2026-11-27 over all thirty selectable lock directories,
with a driver for the dated toolchain,
counted 1674 findings over seven packages,
1611 of them in the four packages the issue names
(the counts are in the section "M3" below).
Whether that run is the record the counts should live in is the owner's to say (question 8).

## How the linter runs the rule

`rust/avoidable-closure` is not a rule of the linter's own analysis.
The linter's README,
section "Closure check"
(`package/linter/monochromatic-lint/README.md`),
states that it runs the type-aware Dylint lint `avoidable_closure`
of `package/linter/monochromatic-lint-closure` over each Rust package that has a selected file,
and names `doc/decision/rust-closure-lint-dylint.md` as the decision.
The root configuration selects it for every Rust file,
with the library path relative to the configuration file
(`monochromatic-lint.config.jsonc`,
the block whose comment reads "Warnings until no finding is left"):

```jsonc
// monochromatic-lint.config.jsonc
"rust/avoidable-closure": {
  "severity": "warn",
  "library": "package/linter/monochromatic-lint-closure",
},
```

After the per-file rules,
`run_closure_step_at` (`package/linter/monochromatic-lint/src/closure_step.rs`)
does this,
in order:

1. It groups the selected Rust files by the nearest `Cargo.toml` above them
   (`group_packages`,
   `src/closure_packages.rs`),
   so the unit of work is the package and not the file,
   and a directory whose name ends in `.fuzz` or `.bench` is exempt.
2. It returns a note and nothing else under `--fix`
   (`run_closure_step_at`,
   the `fix` branch at the top),
   because the lint has no machine fix and a rewrite would make the compiler's positions stale.
3. It reports a configuration error,
   at processing-failure severity,
   when the configured library directory holds no `Cargo.toml`
   (`missing_libraries`,
   `src/closure_step.rs`);
   that is a fault of the repository and not of the machine.
4. It runs `cargo dylint --version` once
   (`dylint_probe`,
   `src/closure_cargo.rs`).
5. It decides on a machine-wide gap:
   a `cargo` program that cannot be started at all,
   or `cargo dylint --version` answering with cargo's own `no such command` for `dylint`,
   skips every selected package
   (`ProbeFailure::Absent`,
   `dylint_probe`,
   `src/closure_cargo.rs`);
   a `cargo` that starts and exits 126 or 127,
   or fails without that evidence,
   fails the run with status `2`
   (`ProbeFailure::Broken`,
   same function);
   a compiler without the `rustc-dev` component skips every selected package
   (`rustc_dev_missing`,
   `src/closure_cargo.rs`,
   the gap branch in `run_closure_step_at`).
   While `MONOCHROMATIC_LINT_CARGO` names a replacement program,
   the component check does not run
   (`rustc_dev_missing` returns `None` for any program name but the bare `cargo`),
   because the replacement stands for the whole toolchain.
6. It checks the Dylint driver once per distinct library
   (`driver_gaps`,
   `src/closure_driver.rs`,
   called from `run_closure_step_at`),
   and skips the packages of a library whose driver cannot be had
   (`driver_state`,
   same file,
   which asks `active_toolchain` in `src/closure_driver_probe.rs`).
7. For every remaining package it reads the prerequisites,
   checks the locked dependencies offline,
   and then runs the lint
   (`check_package`,
   `src/closure_step_package.rs`):
   a missing prerequisite or a cold cargo cache skips that package
   (`dependencies_missing`,
   `src/closure_cargo.rs`,
   and the prerequisite probe of `src/closure_probe.rs`).

The run itself is one process per package
(`run_dylint`,
`src/closure_cargo.rs`):
`cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked --message-format json`,
with `CARGO_NET_OFFLINE=true` set on the child
(`run_dylint`,
same file) and `DYLINT_RUSTFLAGS="-W avoidable_closure -W unfulfilled_lint_expectations"`
(`DYLINT_FLAGS`,
same file).
Only the library and the binaries are compiled,
so `#[cfg(test)]` modules,
`tests/`,
examples and benchmarks are not checked
(README,
section "What a run does").

### What `cargo dylint` itself does, and in which order

The linter calls one program;
`cargo dylint` then does three things,
and the order matters for every option below.
At upstream tag `v6.4.0`
(the version `toolchain.lock.json` records for `cargo:cargo-dylint`
and the version the driver crate is pinned to):

1. It computes the toolchain name once,
   as the first word of `rustup show active-toolchain` run in the library directory
   with `RUSTUP_TOOLCHAIN`,
   `CARGO` and `RUSTC` removed
   (`internal/src/rustup.rs`,
   `parse_toolchain` and the sanitizing implementation;
   called for a `--path` library in `dylint/src/library_packages/mod.rs`).
2. It builds the lint library before it looks for a driver.
   The library is built while the `--path` library is resolved
   (`dylint/src/name_toolchain_map/maybe_library.rs` calls
   `library_packages::build_library`),
   and only afterwards does `check_or_fix` ask the driver builder for a driver
   (`dylint/src/lib.rs`).
   The measured order in the containers of this round was the library first,
   and `~/.dylint_drivers` was still absent when the library build failed
   (`.../m1-f1-nonproxy-cargo/log/warm.err.dylint-link-failure.txt`).
   The library is built with `cargo build --release` into the package's own
   `target/dylint/libraries/<toolchain>` directory,
   with `RUSTUP_TOOLCHAIN`,
   `CARGO` and `RUSTC` removed and `RUSTFLAGS` cleared
   (`dylint/src/library_packages/mod.rs`;
   `internal/src/rustup.rs`).
3. It builds the driver when `<drivers>/<toolchain>/dylint-driver` is absent or reports a version below
   `cargo-dylint`'s,
   or when running the driver's `-V` fails at all
   (`dylint/src/driver_builder/mod.rs`,
   the branch that warns and returns `true` on a failed `-V`),
   from a throwaway crate that pins `dylint_driver = "=<version of cargo-dylint>"`
   and whose `rust-toolchain.toml` names the toolchain and the components
   `llvm-tools-preview` and `rustc-dev`,
   with the same three variables removed and an absolute rpath into
   `<the toolchain's rustup path>/lib`.
4. It checks the package with `RUSTUP_TOOLCHAIN` set to the computed name
   and `RUSTC_WORKSPACE_WRAPPER` set to the driver
   (`dylint/src/lib.rs`).

Two consequences of that order and those sanitised environments are what the containers measured,
and they are the reason this machine has no findings from the rule:

- The library build runs with `RUSTUP_TOOLCHAIN` removed,
  and the library's own `.cargo/config.toml` sets `rustflags = ["-C", "linker=dylint-link"]`
  (`package/linter/monochromatic-lint-closure/.cargo/config.toml`).
  `dylint-link` reads `RUSTUP_TOOLCHAIN` itself
  (`dylint-link/src/main.rs`),
  so without a `cargo` that puts the variable back (a rustup proxy does),
  the library's every link fails with
  `Error: environment variable not found: RUSTUP_TOOLCHAIN`
  (measured,
  `.../m1-f1-nonproxy-cargo/log/warm.err.dylint-link-failure.txt`;
  `.../m2-f2-proxy-cargo/log/lib-nonproxy.err.dylint-link.txt`).
- The compiler that builds the library and runs the check is the one the `cargo` in `PATH`
  resolves to,
  not the toolchain the linter's caller exported as `RUSTUP_TOOLCHAIN`
  (that variable is removed for the build and the check's own value is set by Dylint):
  a direct toolchain link selected the dated compiler,
  and a rustup proxy selected rustup's default
  (measured:
  `.../m2-f2-proxy-cargo/log/execve-counts.txt` counts the floating toolchain's `rustc`,
  and `.../m1-f1-nonproxy-cargo/log/execve-counts.txt` counts the dated one).

A third consequence belongs to the clone:
whether the driver build clones `rust-clippy` depends on the channel word of that toolchain name.
`clippy_utils_has_extra_symbols` (`driver/build/extra_symbols.rs` at `v6.4.0`)
returns `None`,
and so skips the clone,
when the channel is exactly `nightly`,
and when the dated channel is older than `EXTRA_SYMBOLS_SINCE`;
a dated channel at or after that floor clones.
The floating name `nightly-x86_64-unknown-linux-gnu` has the channel word `nightly`,
so it takes the same no-clone arm as the dated name `nightly-2026-11-16` does not.
Measured on both sides:
the dated name ran 3 `git` executions and the floating name 0
(`.../m7-k-with-minimal-floating-channel/log/c-execve-counts.txt`,
`.../m8-option-i-on-the-host-like-floating-build/log/b-execve-counts.txt`).

## What each absence produces

Every absence below is a distinct branch in the source,
and they do not behave the same.

- No `cargo`,
  or no `cargo-dylint` behind it:
  `ProbeFailure::Absent` (`dylint_probe`,
  `src/closure_cargo.rs`),
  whose only accepted evidence is cargo's own `no such command` for `dylint`.
  The run skips every selected package.
- A `cargo` that starts and exits 126 or 127,
  or fails without that evidence:
  `ProbeFailure::Broken` (`dylint_probe`),
  a `core/processing-failure` finding and exit status `2`,
  because a skip that exits `0` would hide a stale install
  (README,
  section "Machine requirements and skips").
- A compiler without `rustc-dev`,
  as the linter's own `rustc --print sysroot` sees it:
  a machine-wide skip
  (`rustc_dev_missing`,
  `src/closure_cargo.rs`,
  the gap branch of `run_closure_step_at`).
  This check runs before any driver is looked for,
  so it decides alone whether the rule can run.
- A driver that is absent or older than `cargo-dylint`,
  with the pinned `dylint_driver` crate unresolvable offline:
  one skip per package of that library
  (`driver_state` and `driver_gaps`,
  `src/closure_driver.rs`).
- A package whose locked dependencies are not in the cargo cache:
  a skip naming `cargo fetch --locked`
  (`dependencies_missing`,
  `src/closure_cargo.rs`).
- A prerequisite named in `monochromatic-lint.prerequisites.jsonc` that the machine lacks:
  a skip naming the task that makes it
  (`src/closure_probe.rs`,
  README,
  section "Prerequisite manifest").
- A check that cannot be made at all,
  such as no readable `cargo dylint --version`,
  no `rustup`,
  or no known driver directory:
  a note that appears only under `--debug`,
  and the packages still run
  (`driver_state`'s `Unknown` arm,
  README,
  section "Machine requirements and skips").

Two more states exist that the list above does not cover,
and both were measured in the earlier round:

- A `cargo dylint` run can fail after the probe passed,
  because the library build or the check failed for a reason no probe reads
  (the missing `RUSTUP_TOOLCHAIN` of `dylint-link`,
  or a library that the compiler cannot compile).
  That is a `core/processing-failure` finding and status `2`,
  not a skip (`check_package`,
  `src/closure_step_package.rs`).
- The linter's driver probe can answer `Ready` while the run still cannot build the library,
  because the probe reads the driver and the crate cache and never builds the library
  (measured,
  `.../m5-option-k-end-to-end/log/no-file-linter.jsonl`,
  and illustrated again in M6).

## What a skipped rule returns

This is the part the issue's title asks for,
and it is decided by three places in the code.

1. The skip is a diagnostic of the same rule code as a real finding:
   `Diagnostic::new(RULE, Severity::Warn, ...)` in `skip_finding`
   (`src/closure_step_package.rs`),
   where `RULE` is `rust/avoidable-closure`.
   Its file name is the package's `Cargo.toml` and its position is the first byte of it.
   Its message starts `rust/avoidable-closure skipped <package>: ` and ends
   "Nothing in this package was checked by this rule,
   so a run without findings says nothing about its closures."
   (`skip_finding`,
   same file).
   The severity is fixed at `warn` on purpose:
   the function's own comment says the configured severity belongs to the closure reports.
   The shape itself comes from the owner's answer of 2026-10-19:
   "A package whose prerequisites are missing on the machine is skipped,
   and the linter says so."
   (`doc/decision/rust-closure-lint-dylint.md`,
   section "Answers of 2026-10-19 to the proposal").
2. Skips are merged into the same finding list as findings before output
   (`run_command.rs`,
   the line that extends the findings with `closure.skips`),
   so a consumer that reads only the code cannot tell them apart;
   the file can:
   a skip names the package's `Cargo.toml`,
   and a real finding of this rule names a `.rs` file
   (`findings_from`,
   `src/closure_findings.rs`).
3. The exit status counts the skip as a warning and nothing else
   (`exit_status`,
   `src/run_output.rs`):
   a warning alone leaves `0`;
   `--max-warnings` counts it,
   and exceeding the limit gives `1`;
   only a finding marked as a processing failure gives `2`.
   `--quiet` and `--silent` hide it with the other warnings
   (`run_output`,
   the filter that drops a `Warn` finding under `--quiet`,
   same file;
   README,
   section "Machine requirements and skips").

That the quiet form hides a skip is a consequence the owner was never asked about,
and the record says so in the coordinator's own words:
"A consequence the owner has not been asked about:
`--quiet` and `--silent` now hide a skip with the other warnings,
where the message on standard error was kept by `--silent`."
(`doc/planning/monochromatic-lint-skip-warning-and-fence-marker.md`).
It is part of Decision 5 below,
not something the contract already settled.

Measured in the containers of the earlier round,
with the released `monochromatic-lint` 0.5.2-alpha.24 over a copied `package/wg/cli-quicker-exempt`,
with no driver and a cold registry
(`.../m5-option-k-end-to-end/log/`):

- The default run printed one JSONL record,
  `{"code":"rust/avoidable-closure","severity":"warn","filename":"pkg/Cargo.toml",...}`,
  and exited `0`.
- `--max-warnings 0` printed the same record and exited `1`.
  A gate can therefore make a skip fail today with no code change,
  at the price of failing every package that skips.
  The three statuses are re-measured with their status lines kept in M10 below,
  because the earlier round kept the records and not the statuses.
- `--quiet` printed nothing and exited `0`,
  so the quiet form of a run looks exactly like a clean run.
- The skip those probes measured was the machine-wide `rustc-dev` gap,
  not the driver gap:
  the floating channel was minimal at that point of the script,
  and `skip-default.jsonl` says "the active compiler lacks the rustc-dev component".
  The driver gap's own words are measured in M7's arm B and M8's arm A.
- `--rules` reports the rule as not suppressible with one option,
  `library`
  (`src/run_modes_rule_table.rs`;
  the installed binary printed
  `{"id":"rust/avoidable-closure","fixable":false,"fixClass":null,"options":["library"],"suppressible":false}`
  on 2026-11-27 at 21:12 -05:00 in the previous round),
  so a project cannot turn a skip off without turning the whole rule off.
- A skip's message names the command that would fill the caches,
  built by `warm_command` (`src/closure_driver.rs`):

  ```sh
  env --unset=CARGO_NET_OFFLINE cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked
  ```

  and the record of 2026-10-30 shows that this command,
  run by hand with network access,
  is what filled the driver cache then
  (`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
  the workaround section).

## Who provides the driver today

No component of this repository builds or installs a Dylint driver.
Each part below states what it does and does not do.

### The toolchain plan, the lock and the install task

`toolchain.plan.jsonc` holds three rows that matter here,
and none of them is a driver:

- `rust` (`toolchain.plan.jsonc`,
  the `"rust"` row):
  `rust@nightly` through the `rustup-toolchain` route,
  the components `clippy`,
  `rust-src`,
  `llvm-tools`,
  `rustc-dev` and `rustfmt`,
  the targets `aarch64-linux-android` and `x86_64-linux-android`,
  and `RUSTUP_TOOLCHAIN` set to `{version}` in the row's environment.
- `cargo:cargo-dylint` (same file):
  `cargo:cargo-dylint@latest` through `cargo-binstall`,
  exposing the bin `cargo-dylint`.
- `cargo:dylint-link` (same file):
  the same for `dylint-link`.

`toolchain.lock.json` records what those rows resolved to on this machine:
`nightly-2026-11-16` with the same five components,
`RUSTUP_TOOLCHAIN` as `nightly-2026-11-16`,
`cargo-dylint` 6.4.0
and `dylint-link` 6.4.0
(the `rust`,
`cargo:cargo-dylint` and `cargo:dylint-link` entries).
The task that installs all of it is `toolchain:install` (`mise.toml`);
`toolchain:install:global` (same file) is a different task,
it converges the machine-global tools with `mpm-shadow install --global`.
A `bootstrap` task exists and is a thin alias of `toolchain:install`
(`mise.toml`,
`[tasks.bootstrap]`),
and the commented-out `[hooks]` entry would have called it on directory entry
(`mise.toml`,
the comment above `enter`).
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
(`warm_command`,
`src/closure_driver.rs`),
and the linter's own check is offline by construction:
`dylint_drivers_directory` joins `.dylint_drivers` under the home directory,
`driver_site_from_environment` reads `DYLINT_DRIVER_PATH`,
`active_toolchain` asks `rustup show active-toolchain` in the library directory with `RUSTUP_TOOLCHAIN`,
`CARGO` and `RUSTC` removed,
`driver_version` runs the driver's `-V`,
and `driver_unresolved` resolves a throwaway crate that pins `dylint_driver = "=<version>"`
with `cargo metadata --offline`
(all in `package/linter/monochromatic-lint/src/closure_driver_probe.rs`).
The linter reads the driver cache,
mirrors Dylint's staleness rule,
and refuses to write anything
(README,
section "Machine requirements and skips").

### The tasks that do name the rule's needs

- `//package/linter/monochromatic-lint:test:host`
  (`package/linter/monochromatic-lint/mise.toml`)
  is the one ignored binary test,
  `host_real_run_over_wg_quicker_exempt`
  (`src/closure_binary_tests.rs`),
  which asserts a real run over `package/wg/cli-quicker-exempt` has no skip and at least one finding,
  and whose positive control runs the same library with `--all-targets`
  to show that the absence of test findings can be told from a probe that could not find them
  (same function).
  It needs `cargo-dylint`,
  the `rustc-dev` nightly and a warm cargo cache.
  This round did not run it.
- `//package/linter/monochromatic-lint-closure:run`
  (`package/linter/monochromatic-lint-closure/mise.toml`)
  runs `cargo dylint --path <this package> --manifest-path <directory>/Cargo.toml -- --all-targets`
  with no offline setting,
  so it is a warming command as well as a lint:
  it builds the driver when none is current
  (`driver_builder::get`).
- The root `lint:monochromatic-lint` task (`mise.toml`)
  depends on `fetch:rust-dependencies` (same file),
  which runs `cargo fetch --locked` per package
  (`fetch:rust-dependencies`).
  That fills the package locks and not the driver crate,
  which belongs to no lockfile
  (`src/closure_driver.rs`,
  the module comment).

### The workflows

`rg --line-number --ignore-case 'dylint' .github` returns no hit at all,
so no workflow names the tool,
the driver or the rule.
The one workflow that runs the linter is `.github/workflows/advisory-core.yml`,
and it cannot start the linter today.

- The `advisory core` job installs the repository tools with the closed name list
  `tools: file-enforcer mpm-shadow cargo-shadow` (`advisory-core.yml`),
  and `package/monorepo-task/core-exec/src/repo-tools-setup-archives.ts`
  accepts exactly `mpm-shadow`,
  `file-enforcer`,
  `cargo-shadow` and `git-policy-cli`
  (`repo-tools-setup.ts` rejects any other name).
  `monochromatic-lint` is not one of them,
  and nothing before the lint step installs it.
  The same job does build the linter later,
  in its "Build fast subset" step
  (`task-run //package/linter/monochromatic-lint:build`,
  `advisory-core.yml`),
  which produces a binary in the job's build folder and puts nothing on `PATH`.
- The "Static changed paths" step (`advisory-core.yml`) calls
  `task-run lint:monochromatic-lint -- "${files[@]}"`.
  The last retrievable log of that step in which it reached the call
  is run `41274002049` of 2026-11-01,
  and it printed
  `.github/CODE_OF_CONDUCT.md`,
  then `task-run: line 1: monochromatic-lint: command not found`,
  then `Error: task-cli: step exited 127`
  (`.../ci/ci-run-41274002049-static-step-excerpt.txt`).
  The step reports `success` only because of `continue-on-error: true`;
  the "Enforce advisory result" step fails the run on the step's outcome
  (`advisory-core.yml`).
  The linter never started,
  so no package was probe-checked in CI at all.
- The job installs the locked Rust toolchain with
  `rustup toolchain install "$toolchain" --profile minimal --no-self-update`
  (`advisory-core.yml`,
  the "Install the pinned Rust toolchain" step),
  which carries neither `rustc-dev`,
  nor `llvm-tools`,
  nor `cargo-dylint`.
  `task-run` does not see the toolchain environment either
  (`doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md`),
  so an exported `DYLINT_DRIVER_PATH` or `RUSTUP_TOOLCHAIN` in the lock would not reach the step.
- The recent history is silent:
  `gh run list --workflow advisory-core.yml --limit 200` returned 200 `push` runs in the window the first read
  checked,
  199 of them with the `advisory core` job skipped by the workflow's own budget gate
  (`advisory-core.yml`,
  the budget condition of the job),
  as that read recorded on 2026-11-27.

The first thing missing in CI is therefore not a driver but the linter,
and after that the toolchain components,
a `cargo` that restores the toolchain variable and the network for a first driver build.
Every CI option below pays that same cost.
The neighbouring proposal of the same day prices the same surface and says what a hosted job cannot do:
its option A installs "`monochromatic-lint` and its Dylint driver installed the way `repo-tools-setup`
installs the other tools",
and its "What it cannot do" says a hosted job cannot replace
the "closure check,
which need every package's dependencies and a Dylint driver per package"
(`doc/planning/github-hosted-runners-for-package-checks-and-a-guest.md`).
That is the CI answer's own limit and is why Decision 4 offers it as one surface and not as the provider.

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
- `type -a cargo` lists,
  in this order:
  this repository's shadow launcher
  (`/home/user/Monochromatic/.cache/cargo-shadow-bin/cargo`,
  whose `readlink -f` is `.cache/toolchain-launchers/cargo-shadow`,
  a launcher that starts `cargo-shadow` from this repository),
  then `/home/user/Monochromatic/.cache/toolchain-bin/cargo`,
  then `/usr/bin/cargo`.
  `readlink -f /home/user/Monochromatic/.cache/toolchain-bin/cargo`
  is the dated toolchain's `bin/cargo`,
  and `readlink -f /usr/bin/cargo` is `/usr/bin/rustup`,
  so a rustup-proxy `cargo` is installed system-wide and is on `PATH`,
  third.
- `type -a rustc` has no repository launcher in its way:
  the first `rustc` is `.cache/toolchain-bin/rustc`,
  a link straight into the dated toolchain,
  and the proxy `rustc` comes later.
- `ls -d /home/user/.dylint_drivers` answers
  "No such file or directory",
  so no driver exists now;
  the records below show one existed before.
- The daily toolchain update timer stands
  `disabled; inactive (dead)` (`systemctl --user status toolchain-daily-update.timer`,
  read by the second gate read on 2026-11-28;
  it is kept in `.../gate-read-second-2026-11-28/probe-daily-update-timer-and-lock-state.txt`).
  No record this round read says why it was disabled.
  It matters here because the lock's nightly is what a generated toolchain file would name,
  and the timer is what moves that nightly.

The 2026-11-25 record said the opposite of the third point,
and the coordinating session corrected it on 2026-11-27:
that record now holds both the original sentence
("this host has no rustup proxy in `PATH` at all",
`doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md`)
and the correction that `/usr/bin/cargo` is a link to `/usr/bin/rustup`.
Its second workaround also said the proxy arm was never run
("so the arm could not be exercised",
same record).
This round and the round before it ran it,
in containers,
and the results are below.

Four toolchain names are in play,
and the rest of this proposal keeps them apart:

- the floating channel,
  `nightly-x86_64-unknown-linux-gnu`,
  which is rustup's default here,
  whose compiler is `5df8c02af` (2026-11-16),
  and which has no `rustc-dev` or `llvm-tools`;
- the dated name the lock records,
  `nightly-2026-11-16`,
  whose full form is `nightly-2026-11-16-x86_64-unknown-linux-gnu`
  and which carries the components;
- a dated name a person pins by hand,
  which is the same shape as the one above but need not be the lock's;
- the name `rustup show active-toolchain` prints for the library directory with `RUSTUP_TOOLCHAIN` removed,
  which is the floating one today and the one Dylint files a driver under for the library as it stands.

The driver directory the linter's own message names is
`/home/user/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver`
(the issue's capture,
and the same message reproduced in the containers of both rounds),
so a driver filed under `nightly-2026-11-16` (the directory the recorded workaround used)
is never read by the linter as the library stands.

## What changed since the rule last ran

The rule's last recorded runs on a machine of this project's
are the warm of 2026-10-30
(`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
the section that records the applied warm)
and the repository-wide count of 2026-11-07,
which reported 516 findings of `rust/avoidable-closure`,
508 warnings and 8 errors
(`doc/planning/repository-linter-stays-at-zero.md`).
Today the rule runs on nothing here.
Five things differ,
and each has its record:

1. The machine is not the one those runs used.
   The operating system was reinstalled after the 2026-11-04 record:
   `/var/log/pacman.log` begins at `2026-11-16T20:30:16-05:00` with a Calamares installer transaction,
   the root filesystem is dated `2026-11-16 20:29:37 -0500`
   and the home folder `2026-11-16 20:32:08 -0500`
   (`doc/troubleshooting/fcitx-flatpak-xiaohe.md`,
   section "Observed state").
   The records of 2026-10-30 name `/var/home/user`;
   the checkout is `/home/user` today.
2. The driver cache went with it.
   `.../machine/machine-state.txt` records `ls -d /home/user/.dylint_drivers`
   answering "No such file or directory",
   and the 2026-10-30 record shows the driver being written under
   `~/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver`
   on the machine before the reinstall.
3. The floating channel lost the components.
   The 2026-10-18 owner answer added `rustc-dev` to the shared floating nightly
   (`doc/decision/rust-closure-lint-dylint.md`,
   the section on the root toolchain change),
   and the 2026-10-30 record's arms ran with the default nightly in use.
   Today `rustup component list --installed --toolchain nightly` lists
   `cargo`,
   `clippy`,
   `rust-std`,
   `rustc` and `rustfmt` and no `rustc-dev`
   (`.../machine/machine-state.txt`).
   So the linter's `rustc-dev` probe passes or fails today by what `rustc` resolves to,
   not by what the floating channel carries.
4. The `cargo` first on `PATH` is a repository launcher,
   not the proxy the 2026-10-30 harness put first.
   That record's harness set `PATH` to `~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`
   (`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
   the harness block),
   where `~/.cargo/bin` holds rustup's proxies;
   today two repository programs come before `/usr/bin/cargo`
   (`.../machine/machine-state.txt`).
5. What the 2026-11-07 run used as its `cargo` and as its driver is not recorded.
   The planning record states the count and the commit it was taken at,
   and nothing about the toolchain or the driver;
   this round could not recover it,
   and does not guess.
   What that run does prove is that on that machine the rule ran and reported.

The reading that fits all five is the one the records support and no more:
a machine on which the driver had been warmed (2026-10-30),
whose floating channel carried `rustc-dev` (the 2026-10-18 answer),
and whose `cargo` was the rustup proxy (the harness's `PATH`),
ran the rule;
the reinstall of 2026-11-16 removed the warm driver and the channel's components,
and the `cargo` first on `PATH` today is not the proxy.
Whether the 2026-11-07 count was taken in that state or in another is unknown,
because the record does not say.

## What this round measured

Every container ran as rootless `podman`,
each inside its own
`systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`,
from the image `localhost/dylint-probe-base:2026-11-28b`
(built this round from `docker.io/cachyos/cachyos-v4:latest`
with rustup,
`strace`,
`jq`,
`binutils`,
`git`,
`cargo-dylint` and `dylint-link` 6.4.0,
and the toolchains the measurements name).
Each container was removed at the end of its run,
and no container of this round is left.
Nothing of this account's home was mounted:
the library,
the test package and the released linter binary were copied in with `podman cp`.
The in-container scripts are in `.../scripts/`
(`m7-in-container.sh`,
`m8-in-container.sh`,
`m10-in-container.sh`),
and each measurement's log is in the folder named under it.

Two statements hold for every run below,
and they are the states the gate read asked to have written down:

- The `cargo` and `rustc` in `PATH` are the rustup proxies (`/usr/bin/cargo` and `/usr/bin/rustc`,
  links to `/usr/bin/rustup`),
  which is the state a fresh machine and a hosted runner have,
  and the state that a yes to Decision 2's first mechanism creates on this host.
- No driver exists in `~/.dylint_drivers` before the first arm that has one,
  and the container's cargo registry is empty.

The linter binary is the released `monochromatic-lint` 0.5.2-alpha.25,
whose sha256 begins `34ca8f26d6`
(printed in each `run.txt`);
the earlier round used 0.5.2-alpha.24,
and the closure sources did not change between them
(`git diff --stat 8638a53928..HEAD -- package/linter/monochromatic-lint/src` is empty).

### M7: option K measured alone

Folder:
`.../m7-k-with-minimal-floating-channel/`.
The state before it:
the floating channel is the one the image's build day resolved,
`rustc 1.102.0-nightly (c82ce1563 2026-11-27)`,
with the components `cargo`,
`rust-std` and `rustc` only (`log/floating-components-before.txt`);
the dated toolchain `nightly-2026-11-16` carries `rustc-dev`,
`llvm-tools`,
`rust-src` and the two Android targets (`log/dated-components.txt`);
the library directory holds option K's file,
`channel = "nightly-2026-11-16"` with `llvm-tools-preview` and `rustc-dev`;
and no component was added by any step of this run.
The control shows the two names the probes can read:
`rustc --print sysroot` from the linter's working directory answers the floating sysroot with the variable unset
and the dated one with it set (`log/probe-sysroot-unset.txt`,
`log/probe-sysroot-set.txt`),
and `rustup show active-toolchain` in the library directory answers the dated full name either way
(`log/k-active-without-env.txt`,
`log/k-active-with-env.txt`).

- Arm A,
  the linter with the variable unset and no driver:
  one skip per package,
  "the active compiler lacks the rustc-dev component",
  status `0` (`log/a-linter-unset-before-driver.jsonl`).
  The library's own toolchain file does not reach this probe,
  because the probe runs `rustc`,
  not `rustup`,
  in the linter's directory.
- Arm B,
  the linter with `RUSTUP_TOOLCHAIN=nightly-2026-11-16` and no driver:
  one skip per package,
  the driver gap for
  `/root/.dylint_drivers/nightly-2026-11-16-x86_64-unknown-linux-gnu/dylint-driver`,
  status `0` (`log/b-linter-set-before-driver.jsonl`).
- Arm C,
  the warm run that builds the driver:
  `cargo dylint --path lint-closure-k --manifest-path pkg/Cargo.toml -- --locked`
  with `CARGO_NET_OFFLINE` unset,
  status `0` after 123 s,
  the library built in 47.13 s and the driver in 1 m 13 s (`log/c-warm.err`),
  3 `git` executions (`log/c-execve-counts.txt`),
  the driver filed under the dated name with `-V`
  `nightly-2026-11-16-x86_64-unknown-linux-gnu 6.4.0` (`log/c-driver-tree.txt`,
  `log/c-driver-version.txt`),
  and a single RUNPATH entry into the dated toolchain (`log/c-driver-readelf.txt`).
  The run's own warnings are 6 (`log/c-warm.err`,
  the line `generated 6 warnings`).
- Arm D,
  the linter with the variable set and the driver present:
  6 findings at the positions the 2026-11-27 round found,
  status `0` (`log/d-linter-set-after-driver.jsonl`).
- Arm E,
  the linter with the variable unset and the driver present:
  the rustc-dev skip again,
  status `0` (`log/e-linter-unset-after-driver.jsonl`).

What M7 settles:
K is a configuration in which the name,
the compiler and the driver agree,
and the rule runs (arm D);
but K alone does not make the linter's own probe read a compiler with `rustc-dev`,
so on a machine whose `rustc` is the proxy (and whose floating channel is minimal,
which is what rustup installs) K skips without an exported `RUSTUP_TOOLCHAIN` (arms A and E)
and runs with it (arm D).
The exported variable,
not the driver,
is what decides (arm E has the driver and still skips).
The earlier round's K run passed only because its script had added `rustc-dev` to the floating channel
before the run and never exported the variable;
that arm was therefore not K alone,
and it is replaced by M7.

### M8: option I measured on this compiler

Folder:
`.../m8-option-i-on-the-host-like-floating-build/`.
The state before it:
the floating channel is made to hold the build this host's floating channel holds,
`rustc 1.102.0-nightly (5df8c02af 2026-11-16)`,
which is the nightly of 2026-11-17
(`https://static.rust-lang.org/dist/2026-11-17/channel-rust-nightly.toml`
names that version;
rustup names a toolchain by its directory,
so the dated install of 2026-11-17 is copied over the floating name inside `RUSTUP_HOME`,
`log/toolchains-before.txt` and `log/toolchains-after.txt`);
it carries `rustc-dev`,
`llvm-tools`,
`rust-src`,
`cargo`,
`rust-std` and `rustc` (`log/floating-components.txt`);
the library has no toolchain file;
`rustup show active-toolchain` in it answers the floating name (`log/library-active-toolchain.txt`);
and no driver exists (`log/a-linter-before-driver.jsonl` names the absent one).

- Arm A,
  the linter with no driver:
  one skip per package,
  the driver gap for
  `/root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver`,
  status `0`.
  The rustc-dev probe passed,
  because the floating channel carries the component and the proxy `rustc` reads it.
- Arm B,
  the warm run that builds the library and the driver:
  status `0` after 91 s,
  the library built in 46.88 s and the driver in 38.85 s (`log/b-warm.err`),
  0 `git` executions (`log/b-execve-counts.txt`),
  245 executions of the floating `rustc` and none of the dated one (same file),
  the driver filed under `nightly-x86_64-unknown-linux-gnu` with `-V`
  `nightly-x86_64-unknown-linux-gnu 6.4.0` (`log/b-driver-tree.txt`,
  `log/b-driver-version.txt`),
  and a single RUNPATH entry into the floating toolchain (`log/b-driver-readelf.txt`).
- Arm C,
  the linter offline with the driver present and no exported variable:
  6 findings at the same six positions as M7's arm D,
  status `0` (`log/c-linter-after-driver.jsonl`).
- Arm D,
  the same with `--max-warnings 0`:
  status `1` (`log/d-linter-max-warnings.jsonl`,
  the status line in `run.txt`).

What M8 settles:
I runs the rule end to end on the compiler this host's floating channel carries,
with no toolchain file,
no exported variable and no clone,
for 91 s of first-run build against K's 123 s,
and it produces the same six findings.
The earlier round's disqualification of I was measured on a nightly 11 days newer
(`c82ce1563`,
2026-11-27) on which `dylint` 6.4.0 fails with `error[E0603]`;
the same failure is a feature of that nightly and not of I,
and K generated from the lock inherits it as soon as the lock moves there.
What M8 does not settle is a future nightly:
the compiler that builds the library is the floating channel,
so a nightly newer than the installed `dylint` supports breaks I until Dylint publishes,
which is the owner's recorded "Wait for Dylint" situation.

### M10: the statuses a skip returns, with the status lines kept

Folder:
`.../m10-skip-statuses/`.
The state before it:
the floating channel is minimal (no `rustc-dev`),
no driver exists,
and the proxy is first for `cargo` and `rustc`.

- Arm A,
  the machine-wide `rustc-dev` skip:
  the default run printed one record and exited `0` (`log/a-default.jsonl`);
  `--max-warnings 0` printed the same record and exited `1` (`log/a-max-warnings.jsonl`);
  `--quiet` printed nothing and exited `0` (`log/a-quiet.jsonl` is 0 bytes,
  `log/a-sizes.txt`).
- Arm B,
  the driver-gap skip with `RUSTUP_TOOLCHAIN=nightly-2026-11-16`:
  the same three results (`log/b-default.jsonl`,
  `log/b-max-warnings.jsonl`,
  `log/b-quiet.jsonl`,
  `log/b-sizes.txt`).

What M10 settles:
the three statuses the earlier round reported are the three a skip really returns,
for both skip causes,
with the status lines now in the folder.
A gate can fail a skip today through `--max-warnings 0` with no code change,
and the quiet form of a run prints nothing and exits `0` for a skip as for a clean run.

### The earlier round's measurements, and what their logs do and do not hold

Folders under `.../`:
`m1-f1-nonproxy-cargo/`,
`m2a-proxy-alone-fails-at-rustc-dev/`,
`m2-f2-proxy-cargo/`,
`m4-f10-rpath-and-other-home/`,
`m5-option-k-end-to-end/`,
`m6-clean-driver-and-probe/`,
`m3-f3-skips-with-a-driver-1/` and
`m3-f3-skips-with-a-driver-2/`.
They are cited for what their logs print and for nothing else.
Where this revision needs a figure or a status they do not hold,
it uses M7,
M8,
M10 or the source,
and the sentences that rested on an unkept figure are gone.

- M1 (a non-proxy `cargo` first):
  the keep log prints the skip's words and the floating toolchain name,
  the two `rustup show active-toolchain` answers,
  the library build's `dylint-link` failure
  (`Error: environment variable not found: RUSTUP_TOOLCHAIN`),
  the absence of `~/.dylint_drivers` afterwards,
  the `execve` counts (the dated `rustc`,
  no `git`),
  and the dated-name driver build's refusal
  (`fatal: destination path '/tmp/.tmpEjtkLF' already exists and is not an empty directory`).
  Its `PATH` held `/opt/nonproxy/cargo` and `/opt/nonproxy/rustc`,
  links into the dated toolchain;
  this host's first `cargo` is instead the repository's shadow launcher,
  whose hand-off lands on the same non-proxy binary
  (the shadow's README,
  section "Toolchain and registry settings",
  and `.../machine/machine-state.txt`);
  the shadow itself was not run this round,
  so M1 models the child's environment
  through the binary the shadow hands off to and not through the shadow.
  The skip's words match the issue's line,
  and the cause of the missing crate is a different one
  (the container's registry is empty;
  on the host the cache is warm and the driver crate is absent),
  so M1 reproduces the words and not the host's exact cause.
  Its planted-driver arm planted nothing
  (`driver built:` empty,
  `install: cannot stat ''`),
  so "no check runs,
  whatever driver exists" rests on the source order,
  which is what the order section above cites.
- M2a (a proxy first,
  the floating channel still minimal):
  `E0463` for `rustc_driver` and five more crates,
  the floating `rustc`'s execution count,
  a driver-gap skip with a cold-registry cause,
  and 113 s for the warm run.
  It claims nothing beyond that.
- M2b (a proxy first and the components on the floating channel):
  `E0603` at `dylint_linting-6.4.0/src/lib.rs:794` against the 2026-11-27 nightly,
  the same library build failing at `dylint-link` without the proxy,
  a dated driver build with the proxy that finished in `2m 19s` (139 s,
  `log/dated-driver.err.last-4.txt`) with 3 `git` executions,
  and the non-proxy driver build failing with the same variable message.
  Its script emulated the library and driver steps with direct `cargo build` calls,
  not with `cargo dylint`;
  the figures 24 s,
  46 s,
  1 s,
  82 s and 63 s the earlier text carried are in no kept line
  and are not used here,
  and the folder keeps no `run.txt`.
- M4 (the driver's RUNPATH):
  prints every step it claims:
  the control exit `0` with the builder's path present,
  the RUNPATH of two absolute entries and `NEEDED librustc_driver-5adf19dcd2f7d879.so`,
  exit `127` with `cannot open shared object file` where the toolchain sits at `/opt/rustup`,
  and exit `0` with `LD_LIBRARY_PATH`.
  What it settles is the bare load of the driver,
  which is how `cargo dylint` probes it with `-V`;
  how the driver loads as a child of `cargo` at check time was not measured.
- M5 (option K end to end,
  before M7 replaced it):
  prints the dated full name from the toolchain file,
  the driver build's line and times (library 53.41 s,
  driver 1 m 39 s),
  3 `git` executions,
  241 dated and 4 floating `rustc` executions,
  the driver's `-V`,
  its RUNPATH and `NEEDED` lines,
  six findings at six positions,
  the `nightly` arm failing at `dylint_driver-6.4.0/src/lib.rs:185` with `E0603`,
  and the no-file skip naming `anstyle-wincon`.
  Its driver build exported a floating `RUSTFLAGS`,
  which is why its RUNPATH had two entries;
  M7's has one.
  It kept no `run.txt`,
  its three skip probes measured the rustc-dev skip and not the driver gap,
  and the components were added to the floating channel before its K arm.
  M7 and M10 replace it.
- M6 (a driver built without Dylint's rpath,
  and the probe once its resolution succeeds):
  prints the driver's missing RUNPATH and its load error (75 s,
  3 `git`),
  the probe fetch (4 s),
  offline metadata exit `0` with an empty error file,
  and exit `2` with `no matching package named dylint_linting found` for both runs.
  That the probe answered `Ready` is inferred from the absence of a skip;
  the failure's cause is the container's empty registry,
  so M6 is an illustration of the class and not a case from a host.
- M3 (the thirty selectable lock directories,
  with a driver for the dated toolchain):
  the state it describes is a container with the proxy first,
  the library naming the dated toolchain,
  the host's registry cache copied in,
  network available and `RUSTUP_TOOLCHAIN` exported,
  one driver built at the start of each container,
  and one linter invocation per directory after
  `cargo metadata --offline --locked` (fetching with network first when that failed).
  `log/results.tsv` has one row per directory in each folder,
  with its status,
  wall time,
  metadata result and closure records.
  All thirty ran.
  Recounted by this round from those files:
  seven packages produced 1674 closure findings
  (`package/git-policy/cli` 1502,
  `package/mpm-shadow/cli` 72,
  `package/zfs/request-to-openzfs-kernel-device` 44,
  `package/linter/monochromatic-lint` 31,
  `package/cpu-temperature/cli` 13,
  `package/cargo-shadow/cli` 6,
  `package/wg/cli-quicker-exempt` 6),
  fifteen produced none,
  six skipped and two failed closure with status `2`,
  so twenty-two of the thirty were checked.
  The four packages the issue names account for 1611 of the 1674.
  The wall times of the thirty rows sum to 1688 s;
  one linter invocation ran at a time,
  each with `CARGO_BUILD_JOBS=4` (`scripts/m3-in-container.sh`);
  `package/linter/monochromatic-lint` alone took 418 s,
  `mpm-shadow` 250 s and `git-policy` 142 s.
  What the folder does not hold,
  and this revision no longer states as measured:
  the cause of the two failures
  (each kept failure record ends at "The end of cargo's messages was:"),
  the cause of the two cache skips beyond the 400-character record each skip message keeps,
  any `git fetch` failure text,
  and the statuses or times of M2b,
  M3 and M5.
  `results.tsv` records `still-failed` for `package/ide/app` and `package/terminal/app`
  after the script's own networked `cargo fetch --locked`,
  so in that container a network fetch did not close the gap;
  whether one would on a machine whose `git` can reach those remotes is a hypothesis,
  and the earlier text's "a cache gap a network fetch would close" is withdrawn.
  Whether the 1502 are distinct positions was not checked by a command;
  the number is the count of finding records for that package.
  The fifteen clean packages rest on their rows' status and the absence of a closure record,
  not on a per-package note
  (the `checked N report(s)` notes appear only under `--debug` and are not kept).

## What the measurements imply

1. Three things decide whether the rule runs,
   and no option may assume they agree:
   the name Dylint files a driver under (a `rustup show active-toolchain` answer in the library directory),
   the compiler that builds the library (the `cargo` in `PATH`),
   and the compiler the linter's own `rustc --print sysroot` probe reads.
   Measured:
   M7's control shows the two names,
   M7's arms A and E show the probe failing while the library's file names the dated toolchain,
   and M1 and M2b show the library build failing without a proxy `cargo`.
2. Every option needs a `cargo` that restores `RUSTUP_TOOLCHAIN` for Dylint's children.
   Without it the library's own link step fails before any driver is consulted (M1,
   M2b),
   so no provider,
   task,
   asset or cache can deliver anything.
   `MONOCHROMATIC_LINT_CARGO` does not solve it:
   it replaces only the outer program and turns off the component check
   (`cargo_program` and `rustc_dev_missing`,
   `src/closure_cargo.rs`),
   while Dylint spawns `cargo` by name from `PATH` (`internal/src/cargo.rs`).
3. Every option needs a toolchain `dylint` 6.4.0 can compile against,
   and that is a fact about the compiler,
   not about the file that names it.
   Measured working:
   `nightly-2026-11-16` (`5f4effd57`,
   2026-11-13) in M7,
   and `5df8c02af` (2026-11-16) in M8.
   Measured failing:
   `c82ce1563` (2026-11-27) with `E0603` in both the library and the driver (M2b,
   M5),
   and a floating channel without components with `E0463` (M2a).
4. A driver must be filed under the name rustup reports in the library directory,
   and that name follows the toolchain file when there is one.
   The recorded workaround's `nightly-2026-11-16` directory is short of the full name,
   and the linter's probe never reads it (M1's message,
   M7's arm B).
5. The clone is a cost of naming a dated toolchain
   (3 `git` executions,
   1 m 13 s of driver build in M7),
   and the floating name avoids it
   (0 `git` executions,
   38.85 s in M8),
   because the channel word is what the clone decision reads.
6. A published driver carries an absolute RUNPATH into the builder's toolchain (M5,
   M7,
   M8,
   M4),
   so it loads only where that path exists.
7. Eight of the thirty selectable packages are not checked with a driver present (M3):
   six skip and two fail.
   Four of the six skip for a declared prerequisite and two for a cache the container could not fill;
   the two failures' cause is not in the kept records.
   A status keyed on "any skip" is red for the six.
8. The linter's driver probe can answer "the driver is absent" for a cache reason that has nothing
   to do with the driver (`.../m5-option-k-end-to-end/log/no-file-linter.jsonl`),
   and it can answer `Ready` while the run still cannot build the library (M6,
   an illustration).
9. CI cannot start the linter at all today,
   so every CI delivery waits on a workflow change that no provider option includes.
10. A skip costs a gate nothing unless the gate is below the warning count it sets,
    and the seven packages of M3 with findings are already over any plausible limit:
    1674 findings,
    1611 of them in the four packages the issue names.
    So a skip made visible by a new status or a new severity matters where the rest is clean,
    and not as the only reason a run is red.

## The decisions, each answerable alone

Seven decisions are separable,
and this proposal keeps them apart;
an eighth question asks where the counts live.
A provider and a skip return are complements and not alternatives:
a provider changes whether the rule runs,
and a skip return changes what an absent provider says.
The toolchain answer and the provider answer are complements for the same reason:
the first decides which compiler and which driver directory,
the second decides who builds the driver.
Nothing in one ranking is a reason against an answer in another.

### Decision 1: which toolchain the linter's library names, and whether it is pinned

What the owner recorded,
in their own words
(`doc/planning/toolchain-always-latest.md`,
section "Owner's words",
verbatim entries of 2026-11-07):

> "Rust nightly should always be latest."
> "Move anyway,
> fix forward" (when a new nightly breaks a package).
> "Make the plan file always track latest and set up a systemd cronjob to update EVERYTHING (not just rust) daily."
> "Wait for Dylint" (the closure lint plugin `package/linter/monochromatic-lint-closure` stays as it is).

The same sentence opens `doc/planning/rust-nightly-always-latest.md`,
recorded on 2026-11-07:
"Rust nightly should always be latest."
Neither record says the floating nightly is a decision the owner declined or adopted;
the decision record says exactly what happened:

> Still not approved:
> any other change to the root toolchain declaration,
> a pin of the nightly,
> an edit to the IDE package,
> a push,
> or a publish.

(`doc/decision/rust-closure-lint-dylint.md`,
section "Root toolchain change approved on 2026-10-18")

and,
of the same day's answers,

> Adopted as defaults without asking,
> open to veto:
> the floating nightly stays (no pin),
> only rustc's `expect` suppresses the Dylint lint (no comment directives),
> generated code is skipped by the lint itself (spans outside the package's own source),
> and the IDE keeps error severity because it has no findings.

(same record,
section "Answers of 2026-10-19 to the proposal")

So "no pin" is a default a session adopted without asking and left open to veto,
not an owner decision,
and a pin was never refused:
it was listed as not approved,
which is a different thing.
The record also says stage 1 kept the lint's toolchain package-local
("Stage 1 kept the lint's toolchain package-local (a dated nightly in the package)",
same record,
section "Root toolchain change approved on 2026-10-18"),
and that the 2026-10-18 answer replaced that with `rustc-dev` on the shared floating nightly.
So a package-level toolchain file is the arrangement an owner answer moved away from,
and "always latest" with "Wait for Dylint" is the arrangement it moved to.
The closure package's README states the coupling the file would remove and the cost of following the lock:
"The exact nightly is whatever `mise.lock` resolves for the `rust` entry",
and "The library is coupled to one nightly ... a new nightly in `mise.lock` means rebuilding and
rerunning the tests"
(`package/linter/monochromatic-lint-closure/README.md`,
sections on the toolchain and the known limits).

Three answers,
each a different mechanism.

#### Answer K1: a toolchain file generated from the lock

How it works:
`package/linter/monochromatic-lint-closure/rust-toolchain.toml`
carries `channel = "nightly-2026-11-16"` (the `tools.rust.version` the lock records at build time)
and the components `llvm-tools-preview` and `rustc-dev`,
written from `toolchain.lock.json` by a generator the owner would name,
so that the file cannot disagree with the lock.
No rule in `file-enforcer.config.jsonc` writes a toolchain file today,
so the generator is new work whichever part gets it;
who writes it is one of the things a yes to K1 leaves undecided.
Every `rustup` command in that directory then answers the dated full name,
Dylint files the driver under that name,
and the linter's `active_toolchain` reads the same name.
What changes for good:
the directory has one more managed file,
and the library's toolchain is no longer the repository's floating channel
but whatever the lock named on the day it was generated.
What it gives up:
the library always compiles with a compiler one lock-refresh behind the floating channel,
and the file names the toolchain in a second place
(its content is the lock's,
but its presence is a policy the lock does not state).
Measured:
with the file present,
a proxy-first `PATH` and a minimal floating channel,
the rule skips for the compiler the linter probes (M7 arms A and E),
because the probe reads `rustc --print sysroot` in the linter's own directory,
where no toolchain file applies;
with `RUSTUP_TOOLCHAIN` exported it delivers 6 findings (M7 arm D),
and a warm build costs 123 s,
of which the driver is 1 m 13 s and 3 `git` executions (M7 arm C).
Also measured (M11):
on a machine that lacks the dated toolchain,
`rustup show active-toolchain` in that directory installs it and the file's components
(rustup 1.29.3 warns that this auto-installation is deprecated).
So K1 installs what it names,
at the price of a network download on first use.
What a yes leaves undecided:
which field of the lock the file is generated from and when it is regenerated,
whether the component list lives in the file or stays in the plan,
and whether the exported variable of Decision 2 is part of the same answer.

#### Answer K2: a toolchain file with a date pinned by hand

How it works:
the same file and the same mechanism,
with the channel a date a person wrote and nothing regenerates.
What changes for good:
the compiler is frozen until someone edits the file,
and the lock can move without moving the library.
What it gives up:
the pin diverges from "always latest",
from the lock the plan installs,
and from what the repository's lean cleaner keeps:
`pins_in` (`package/cargo-shadow/cli/src/lean_rustup_toolchains.rs`)
reads `toolchain.lock.json`,
`.cache/toolchain.env.json`,
and the `rust-toolchain`/`rust-toolchain.toml` files at a checkout's root,
so a package-level file is not read at all,
and a date the lock does not name is not kept by anything but rustup's own settings.
Delivers and costs as K1,
with the extra exposure that the file and the lock can disagree.
What a yes leaves undecided:
which date,
and who moves it when `dylint` needs a newer compiler.

#### Answer I: the components on the floating channel, and wait for Dylint

How it works:
no new file.
`rustc-dev`,
`llvm-tools-preview` and `rust-src` are installed into the floating channel
(the component list the owner approved on 2026-10-18),
and the plan's `rust` row already carries that list,
so `toolchain:install` puts them back after an update.
Dylint's toolchain name is then the floating one,
the driver is filed under it,
and the linter's probe reads a compiler that has the component.
What changes for good:
nothing in the repository;
the machine carries about 500 MB more of components for a channel that moves daily.
What it gives up:
the compiler that builds the lint library changes with every nightly,
so a nightly that moves rustc's internal API breaks the library until Dylint publishes a release
(the closure package's README warns of exactly this).
That is the owner's "Wait for Dylint" situation,
and the answer to it is the owner's recorded one:
wait,
and do not pin.
Measured (M8):
with the floating channel holding this host's compiler (`5df8c02af`,
2026-11-16) and the components,
the rule runs end to end with no file,
no exported variable and no clone:
91 s of warm build (library 46.88 s,
driver 38.85 s),
0 `git` executions,
and 6 findings at the same six positions as K1's arm D (M8 arms B and C).
Its driver is filed under `nightly-x86_64-unknown-linux-gnu`,
which is the directory the issue's own message names.
The failure the earlier round called disqualifying was measured on a nightly 11 days newer
(`c82ce1563`,
2026-11-27),
on which `dylint` 6.4.0 fails with `error[E0603]` (M2b,
M5);
that is a property of the newer nightly,
and K1 inherits it the moment the lock moves there.
What a yes leaves undecided:
what to do on a nightly that breaks the library (wait for Dylint,
as recorded,
or something else),
and whether the components are added by the plan's row alone or also by a task.

### Decision 2: how the toolchain variable gets back to Dylint's children

Two needs are separate,
and only the first is about the driver:

- the `cargo` that Dylint spawns must restore `RUSTUP_TOOLCHAIN` for the library's link and the driver's build
  (Dylint removes it and only a rustup proxy puts it back);
- the linter's own `rustc --print sysroot` must read a compiler that carries `rustc-dev`.

Measured:
a non-proxy `cargo` fails the link (M1,
M2b);
the proxy first satisfies the first need,
and then the second need is decided by what `rustc` resolves to or by an exported variable
(M7 arms A and D,
M10 arm A,
M8 arm A).

#### Mechanism T: the lint task's own environment puts the proxy first

How it works:
the `lint:monochromatic-lint` task runs the linter with `PATH` holding the rustup proxies before
the repository launchers,
and with `RUSTUP_TOOLCHAIN` set to the toolchain Answer I or K1 names.
What changes for good:
nothing outside the task;
a full-path call to the installed binary does not see it.
What it gives up:
the closure lint's build bypasses `cargo-shadow`,
so the repository's trust checks and its lean
containment do not cover that build;
and a person or agent who calls the binary directly gets neither.
Measured:
this is the state M7 and M8 ran in,
and it delivers the rule in both answers (M7 arm D,
M8 arm C).
What a yes leaves undecided:
whether `RUSTUP_TOOLCHAIN` is set (K1) or the components make it unnecessary (I),
and whether the shadow is given a rule for that tree.

#### Mechanism N: no change; the person runs the lint with the proxy first

How it works:
today's state of the 2026-10-30 record:
a shell whose `PATH` puts `~/.cargo/bin` (the proxies) first,
as that record's harness did.
What it gives up:
the issue's harm,
exactly:
it works when someone remembers.
Delivers the rule when done (the same state as Mechanism T,
measured in M7 and M8).

#### Design CL: the linter puts the proxy first for the `cargo dylint` child

How it works:
a rust change setting the child's `PATH` (or its program) so the proxy is what Dylint spawns,
with tests.
What it delivers over Mechanism T:
every linter invocation,
including a full-path call.
What it gives up:
`cargo-shadow` is bypassed for that tree,
and a program's own toolchain choice is overridden for
that child.
Not measured;
what a yes leaves undecided:
whether the child's `PATH` or its program is changed,
and what replaces the shadow's checks for the lint's build.

#### Design SH: the shadow restores the variable from the lock for every cargo it starts

How it works:
`cargo-shadow` reads `toolchain.lock.json` for the toolchain of every `cargo` it execs
and exports `RUSTUP_TOOLCHAIN`,
which today it does only for its own catalog calls
(`cargo_toolchain` and the `RUSTUP_TOOLCHAIN` it sets on the command in `src/cargo_run.rs`).
What it delivers over CL:
the repository's trust guard stays in the chain,
because the child is still the shadow's hand-off.
What it gives up:
the shadow's documented promise that "the user's own cargo command is not changed",
`+toolchain` and `rust-toolchain` files for every cargo command in the checkout,
and the toolchain of package builds unrelated to this lint.
Not measured;
what a yes leaves undecided:
whether the lock's toolchain overrides an explicit `+toolchain`,
and what every other cargo command does differently afterwards.

### Decision 3: who provides the driver on a person's machine

Each answer assumes Decision 1 and Decision 2,
and each is measured with the same warm command.

#### Answer B: the lint task warms the driver before the offline run

How it works:
the root `lint:monochromatic-lint` task gains a step after `fetch:rust-dependencies` that runs
`warm_command`'s command once per library directory,
without `CARGO_NET_OFFLINE`.
`fetch:rust-dependencies` (the task that runs `cargo fetch --locked` per package) is the precedent:
it already fetches,
already needs the network,
and exists to stop a cold-cache skip.
What changes for good:
a lint run writes `~/.dylint_drivers` and the cargo registry,
which `mise run lint` has never done,
and it needs the network and `git` (for a dated name) at least once per `cargo-dylint` move
or nightly update.
Measured:
the warm command is what builds the driver (M7 arm C,
M8 arm B).
What a yes leaves undecided:
whether a lint run fails when the network is not there,
which package warms the driver,
and who owns the staleness record.

#### Answer Z: the documented manual warm

How it works:
the skip message names the warm command,
the linter's README tells a person to run it once with network access and again after each
`cargo-dylint` upgrade,
and the 2026-10-30 record shows it done with the owner's go-ahead.
What it gives up:
it is the design the issue shows failing in practice:
a person has to remember,
and the record's own history has three releases in three days.
Measured:
it delivers when run (M7 arm D,
M8 arm C are that state).
What a yes leaves undecided:
nothing further;
it is today's answer.

#### Answer A: a plan row that builds the driver at install time

How it works:
a new `toolchain.plan.jsonc` row runs Dylint's build once for the toolchain the library directory
reports and places the binary at `<driver root>/<toolchain>/dylint-driver`,
recording the pair (cargo-dylint version,
toolchain) so either move invalidates it.
It needs a route that compiles,
which the plan has none of today
(its routes are `rustup-toolchain`,
`cargo-binstall`,
`archive`,
`bin`-sourced,
`mpm-pnpm`,
`mpm-uvx` and
`android-sdk`,
and every one of them installs a released artifact rather than building one),
and it needs the installer's `cargo` to restore the toolchain variable.
What it delivers,
once the installer runs:
a driver for every local run with no network call and no compile at lint time,
and no memory step.
What it gives up:
a compile inside the installer,
and a lock record for a compiled artifact with no archive digest.
Measured:
not measured this round;
the warm command it would run is the one M7 and M8 measured.
What a yes leaves undecided:
the new route type,
which driver root it exports,
and whether a run outside mise reads that root.

#### Option F is a complement to B, Z and A, not an alternative

F pre-fills the cargo caches so the first `cargo dylint` can build the driver offline:
it fetches the throwaway driver crate
(`dylint_driver = "=<cargo-dylint version>"` with `anyhow` and `env_logger`)
for every platform in `fetch:rust-dependencies`.
The repository's own record weighs it directly:
its arm E measured "empty driver directory,
the warmed registry copy from arm C,
offline.
A warm registry alone makes the offline run build the driver and pass."
(`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
section "Verification";
arm C,
online,
took 24.6 s for the driver build on the floating name).
Where F bears:
under Answer I the driver build clones nothing (M8:
0 `git` executions),
so a warm registry is all it needs and F makes the first run offline-sufficient;
under Answer K1 or K2 the driver build clones `rust-clippy` (M7:
3 `git` executions),
so no cargo setting makes that run offline and F alone cannot deliver it.
F is therefore offered to whichever provider the owner picks,
and it is not ranked against them.

### Decision 4: how a pull request gets the rule

Both answers need a workflow change,
and neither was measured.

#### Answer C: a prepared image that the job runs

How it works:
a container image bakes the toolchain with the components,
`cargo-dylint`,
`dylint-link`,
a driver,
and the released linter,
and one job runs it over the changed files.
The image is not the shape of `package/linter/monochromatic-lint/test.Containerfile`,
which is a pinned stable image with `COPY vendor`,
`CARGO_NET_OFFLINE=true` and no nightly,
`rustc-dev` or Dylint;
a driver image needs network at build and a nightly with the components.
What it delivers:
the same finding in CI and on a person's machine,
because the image is the program,
without touching the job's tool list;
the measured traps a fresh image meets are the components (M2a) and a writable temporary
`TMPDIR` for the clone (M1's refusal was an existing directory,
and later containers used `TMPDIR`).
What it gives up:
an image to rebuild on every nightly or `cargo-dylint` move,
and a job that pulls it.
What a yes leaves undecided:
how the checkout enters the image,
whether the run has network,
and where the image lives.

#### Answer H: the existing job provisions the tools itself

How it works:
the job installs the lock's nightly with `rustc-dev`,
`llvm-tools-preview` and `rust-src`,
installs `cargo-dylint` and `dylint-link` at the lock's versions,
puts the proxy `cargo` first,
opens the closed tool list for `monochromatic-lint`,
restores and saves the caches Dylint recommends,
and runs the linter over changed Rust files.
The caching guidance is Dylint's own README at `v6.4.0`:
"If you run Dylint in CI,
we recommend caching certain directories to avoid rebuilding libraries,
rebuilding drivers,
or redownloading toolchains."
It lists `~/.cargo/git/checkouts/`,
`~/.dylint_drivers/` and `~/.rustup/toolchains/`,
and also `~/.cargo/bin/`,
the registry directories and `target/dylint/libraries/`.
What it delivers:
pull-request findings without an image pipeline,
which is the surface the issue says never ran at all.
What it gives up:
four separate things on the job must be right at once
(the tool list,
the components,
the proxy,
the caches),
and none of it serves a local edit.
What a yes leaves undecided:
which of the four steps are changed,
how the cache is keyed,
and what the job costs in wall time.

### Decision 5: what a skip returns

The record's contract is the baseline:
a skip is a warning finding of `rust/avoidable-closure` at the package's `Cargo.toml`,
counted by `--max-warnings` and hidden by `--quiet`
(`doc/decision/rust-closure-lint-dylint.md`,
section "Amendment of 2026-11-07",
the record's reading of the owner's words).
The owner's words are "Fetch then warn."
Everything else in that block is the record's reading,
and the quiet-form consequence was never put to the owner
(`doc/planning/monochromatic-lint-skip-warning-and-fence-marker.md`).

Four parts,
each answerable alone and combinable:

- Part 1,
  severity:
  keep `warn` (measured:
the statuses M10 records),
  or make it configurable by cause (option J):
a rule option beside `library` lets a configuration make a driver-gap skip an `error`
  and leave prerequisite skips at `warn`.
  It changes the rule's configuration surface and amends the record's fixed severity.
  Its mechanism is already there:
  `DriverState::Missing` is told apart from a package's other skips in `driver_gaps`.
- Part 2,
  a marker:
  keep the record shape,
  or add a field such as `"checked": false` (option D2,
  the field variant),
  which keeps the code,
  the severity and the exit status and lets a count exclude a skip by a field rather than by a file name.
  Its limit is stated in the record:
a skip already names a `Cargo.toml` and a finding a `.rs` file,
  so a consumer can tell them apart today.
  The code variant (`rust/avoidable-closure-skipped`) conflicts with the record,
  with the comment that keeps findings searchable by the rule name (the comment above `skip_finding`),
  and with the linter's README;
  it is not offered.
- Part 3,
  an exit status:
  no new status,
  or a status of its own for a run that omitted the rule (option D1),
  keyed by cause unless the owner wants it keyed on any skip.
  Keyed on any skip it is red for the six prerequisite and cache packages M3 names;
  keyed by cause it delivers what option J delivers with a new public status instead of an error finding.
- Part 4,
  a closing count:
  none,
  or one closing line,
  "N package(s) not checked by rust/avoidable-closure" (option D3).
  It must not go to stdout,
  which is JSONL;
  on stderr it is unaffected by `--quiet`,
  which today drops warnings from the record stream only.
  The record already gives one finding per package,
  so the count is a second channel for the same fact.

The record's contract ("keep") is the baseline for all four parts.
A measured reason to take Part 1 or Part 3:
the quiet form of a run prints nothing and exits `0` for a skip as for a clean run (M10 arm A and B).
A measured reason not to expect much from it:
the seven packages with findings already carry 1674 findings,
so a gate that fails on a skip alone is a gate on a clean tree.

### Decision 6: whether to pin `cargo-dylint`

Option L replaces `@latest` for the two Dylint tools with an exact request,
the 2026-10-30 record's option D.
What it delivers:
a bump becomes deliberate,
so whichever provider is picked rebuilds only when the owner moves the pin.
What it gives up:
the owner's "update EVERYTHING (not just rust) daily" for these two tools,
and no driver comes with it.
It is a complement to any provider and any toolchain answer,
and the owner decides it separately.
It is measured only in its absence:
the driver is rebuilt when `cargo-dylint` moves (the staleness rule),
and the last three releases
came within three days (the pin record).
What a yes leaves undecided:
which version to hold now.

### Decision 7: what the 2026-11-23 answer in issue 6316 covers

The comment is quoted here so the owner is asked about a text they can see.
Asked on 2026-11-23 whether to run `mise run toolchain:install` in the main checkout,
the coordinating session wrote:

> Because we need to make mpm-shadow prefer system package managers on non-atomic distros first.

(`gh issue view 6316 --comments`,
kept in `.../gate-read-second-2026-11-28/probe-issue-6316-comments.txt`)

and the same comment says the tool links wait for that issue.
So the answer as written is about running the installer in the main checkout while a prerequisite is
unbuilt;
it is not a statement about a new plan row or a new component install,
and the earlier draft's "which the owner has paused" was a reading of it that the session made,
not the owner's words.
What a yes leaves undecided:
which of Answers A and I need the installer to run,
and whether the pause is about the checkout only.

## The rankings, each with its criterion

### Decision 1, the toolchain

Criterion:
findings the rule produces on a machine whose `cargo` and `rustc` are the rustup proxies,
with no exported variable and no hand step,
which is the state a fresh machine and a runner have.
Measured in one container instrument,
one option a run:
I delivers 6 findings (M8 arm C);
K1 delivers 6 findings only with `RUSTUP_TOOLCHAIN` exported and skips without it (M7 arms A,
D,
E);
K2 delivers exactly what K1 delivers,
with a date the lock does not own.

**I > K1 > K2**.

- I over K1:
  both deliver the same six findings when K1's variable is set,
  and I needs neither the file nor the variable;
  I's measured warm build is 91 s with no clone against K1's 123 s with 3 `git` executions,
  and I files the driver under the directory the issue's own message names.
- K1 over K2:
  both are the file and the same mechanism;
  K1 follows the lock,
  so it cannot drift from the toolchain the plan installs and the cleaner keeps,
  and K2 can name a date nothing regenerates and nothing keeps
  (`pins_in` reads a package-level file not at all).

What this ranking does not weigh:
the owner's "always latest".
On that recorded preference I is also the answer the owner's words point at,
and the breakage policy for I is the recorded "Wait for Dylint".
If the owner prefers a compiler that moves only when the lock moves,
K1 is the ranking's second and the reason is K1's own (it follows the lock).

### Decision 2, the mechanism

Criterion:
run shapes the mechanism covers,
and what it takes away from programs it does not own.

**T > N > CL and SH,
which are not ranked against them.**

- T over N:
  both are the same state and deliver the same result (measured);
  T removes the step a person must remember,
  which is the issue's harm,
  and N keeps it.
- CL and SH are designs,
  not ranked:
  neither was measured,
  and each covers more run shapes than T while taking something away from a program it does not own
  (CL the shadow's checks for the lint's build,
  SH the toolchain of every cargo command).
  The rule that a rank follows from measurement or a stated criterion is why they sit outside the order.

### Decision 3, the local provider

Criterion:
what each delivers today,
with the parts that exist.

**B > Z > A.**

- B over Z:
  both hand the run a driver through the same measured command (M7 arm C,
  M8 arm B);
  B does it inside the lint task so no one has to remember,
  which is the harm the issue records;
  B's costs (network and a cache write from a lint task) are stated and not used to rank it lower.
- Z over A:
  Z delivers today with parts that exist and is measured;
  A delivers nothing today because the installer it needs is the paused one,
  and while paused every run still skips.
  This reason is A's alone:
  neither B nor Z touches the installer.
  When the pause lifts,
  A's delivery is above Z's (no memory step,
  no clone at lint time),
  and the order becomes A > B > Z for the reason each answer's delivery gives.

### Decision 4, the pull-request surface

Criterion:
how much of the job's own state has to be right for the finding to appear.

**C > H**,
with "neither" below both.

- C over H:
  C's result is one artifact the job runs,
  so the runner's `PATH`,
  its toolchain and its cache do not decide it;
  H's result is the sum of four steps that must all be right
  (the closed tool list,
  the components,
  the proxy,
  the caches),
  and today none of them is.
  C's costs (an image pipeline,
  a rebuild per move) are stated and not used to rank it lower.
- H over neither:
  H at least delivers the rule on the pull-request surface it is aimed at;
  "neither" delivers nothing,
  and the issue's file says the surface never ran at all.

### Decision 5, the skip's parts

These are four parts and not one ranking;
each part's own reason is above.
Where two parts conflict:
a status keyed on any skip and a severity keyed by cause cannot both be taken,
because one makes every skip red and the other leaves prerequisite skips at `warn`.

## What the owner is asked to pick

Each question is one the owner alone can answer,
and each says what a yes leaves undecided.

1. Which toolchain the linter's library names,
   and whether it is pinned:
   I (the components on the floating channel;
   no file;
   wait for Dylint when a nightly breaks it),
   K1 (a file generated from the lock),
   or K2 (a file naming a date by hand).
   A yes to I leaves undecided what to do on a breaking nightly beyond waiting,
   and whether the plan's row is the only installer of the components.
   A yes to K1 leaves undecided the lock field it is generated from and when it is regenerated.
   A yes to K2 leaves undecided the date and who moves it.
2. How the toolchain variable reaches the lint's `cargo` tree:
   T (the lint task's own environment puts the proxy first and sets the variable),
   N (no change;
   the person runs it that way),
   or one of the two designs (the linter sets its child's `PATH`;
   the shadow restores the variable
   for every cargo it starts),
   which are described and unmeasured.
   A yes to T leaves undecided whether the shadow gets a rule for that tree.
   A yes to either design leaves undecided what replaces what it takes away.
3. Who provides the driver on a person's machine:
   B (the lint task warms it),
   Z (the documented manual warm),
   or A (a plan row that builds it).
   A yes to B leaves undecided the offline failure and which package warms it.
   A yes to Z leaves undecided nothing further.
   A yes to A leaves undecided the new route type,
   the driver root and whether a run outside mise reads it.
   F (pre-filling the caches) can be added to any of the three and is not one of them.
4. Whether a pull request gets the rule:
   C (a prepared image the job runs),
   H (the job provisions the tools itself),
   or neither.
   A yes to C leaves undecided how the checkout enters,
   whether the run has network,
   and where the image lives.
   A yes to H leaves undecided which of its four steps change and how the cache is keyed.
5. What a skip returns,
   in four parts:
   severity (keep `warn`,
   or configurable by cause,
   J);
   a marker field (`"checked": false`,
   or none);
   an exit status (none,
   or one keyed by cause,
   D1);
   and a closing count (none,
   or one on stderr,
   D3).
   Any combination is reachable except a status keyed on any skip together with a severity keyed by cause.
   A yes to J leaves undecided the option's name and which causes it covers.
   A yes to the field leaves undecided the field's name.
   A yes to a status leaves undecided the number.
   A yes to a count leaves undecided its wording.
6. Whether `cargo-dylint` and `dylint-link` are pinned to exact versions (L).
   A yes leaves undecided which version to hold now.
   It takes the two tools out of the owner's "update EVERYTHING daily".
7. What the 2026-11-23 answer in issue 6316 covers:
   the installer run in the main checkout only,
   or a plan change too (Answer A's row,
   Answer I's component install).
   A yes to the wider reading leaves undecided which of the two it covers first.
8. Where the counts of M3 live:
   in `doc/planning/repository-linter-stays-at-zero.md` beside the 2026-11-07 count,
   or in a record of their own.
   A yes leaves undecided whether the host run the issue asked for is repeated on this machine and when.

## Combinations that cannot be built

- Answer I with option M.
  M sets `RUSTUP_TOOLCHAIN` to a dated name in the library's cargo configuration,
  so the library would be built by the dated compiler while Dylint files the driver under the
  floating name,
  and the two would disagree about the compiler.
  M's other premise (that cargo's `[env]` reaches the linker) was not measured;
  the read's note that `force = true` is unnecessary because Dylint removes the variable first is
  correct and is why M is not offered as an answer here.
- Answer I with option E.
  A driver built on the dated nightly carries an absolute RUNPATH into that toolchain (M4,
  M7),
  so it loads under I only if the dated toolchain is installed at the builder's path,
  which is the machine state I avoids.
- Answer K2 with the lean cleaner.
  The cleaner keeps a toolchain the lock or a checkout-root `rust-toolchain` file names
  (`pins_in`),
  so a hand-pinned date the lock does not name is a toolchain the cleaner may delete after its grace
  period.
- Answer K1 or K2 with option F (pre-filled caches) and no network at all.
  The driver build for a dated name clones `rust-clippy` (M7:
  3 `git` executions and 1 m 13 s),
  so the first run needs `git` and the network whatever the cargo caches hold.
- A status keyed on any skip with a severity keyed by cause,
  as above.
- `MONOCHROMATIC_LINT_CARGO` with the component check.
  The linter skips that check while the variable is set (`rustc_dev_missing`),
  so taking the replacement cargo gives up the check rather than satisfying it.

## The two gate reads, finding by finding

### The read of 2026-11-28, High findings

#### H1, the pin question is framed on a misreading

Checked:
both files were read whole,
and the record's lines were read at the commit the read names and at `HEAD`.
The finding is right in all three parts.
The word "declined" and the phrase "the owner adopted" were wrong:
the record says "Still not approved" and "Adopted as defaults without asking,
open to veto".
The owner's own words were not cited anywhere in the earlier draft;
they are quoted above,
and the daily-update half of them ("Make the plan file always track latest ...") is what a pin sets
aside.
The earlier draft put a file generated from the lock and a hand-written date under one label;
Decision 1 now separates them (K1 and K2)
and adds the answer the owner's words point at (I).
"Wait for Dylint" is stated as the breakage policy for I and as the answer that needs no pin.
What is not claimed:
that the owner decided against a pin (they did not),
or that the package-level file was a mistake (it was stage 1,
moved away from by the owner's answer).
Proposal changed:
Decision 1,
the ranking's criterion,
question 1,
and the combinations section.

#### H2, K was never measured alone, and I's disqualification is another machine's snapshot

Checked and run:
M7 measured K alone with the components removed from the floating channel and the proxy first for
`cargo` and `rustc`.
The finding's mechanism is right and is now measured:
K skips (arms A and E) without an exported variable and runs with it (arm D),
and the earlier K run was not alone because its script had added the components to the floating
channel.
M8 measured I on the compiler this host's floating channel carries,
which is what the finding asked for.
That part of the finding is answered by the run and the earlier draft's label is wrong:
I delivers 6 findings,
in 91 s,
with no clone (arms B and C of M8).
What survives of "disqualifying":
a nightly newer than the installed `dylint` supports (measured on `c82ce1563`) breaks both I and F,
and the answer to it is the owner's "Wait for Dylint".
The sentences "K delivers on any machine that has the dated toolchain installed,
whatever the floating
channel is" and "none was measured" for K's disqualifying problem are gone.
Proposal changed:
M7 and M8 sections,
Decision 1,
the Decision 1 ranking,
and implication 1.

#### H3, the sets are not separable

Checked:
this was a property of the earlier draft's text,
and every instance the finding names is present in it.
The picks are now:
toolchain (Decision 1),
the variable's mechanism (Decision 2),
local provider (Decision 3),
pull-request surface (Decision 4),
skip parts (Decision 5),
the `cargo-dylint` pin (Decision 6),
and the issue-6316 reading (Decision 7).
F is a complement of B,
Z and A and is not ranked against them (the section above the Decision 3
answers says so and the ranking does not include it);
L is question 6 on its own.
Pairs that cannot be taken together are listed in the combinations section.
The interaction the finding names (a yes to Decision 2's Mechanisms changing what Answer K1 needs)
is stated in Decision 2 and in the combinations section:
with the proxy first and the variable unset,
F skips (M7 arm A),
and with the components on the channel,
I's probe passes (M8 arm A).
Proposal changed:
the whole decisions and questions structure.

#### H4, option F's last place rests on a reason that applies to every option

Checked:
the earlier draft's F reason ("it provides no driver and cannot make the driver build work:
the build fails without the proxy") does apply to B,
Z,
A,
C,
H and E,
so it did not separate F from them.
The record's arm E is quoted in full in Decision 3's F section,
with arm C's 24.6 s.
The finding's own qualification is right too:
under a dated name the driver build clones `rust-clippy` (M7's 3 `git` executions),
so F cannot make that run offline,
while under the floating name it can (M8's 0 `git` executions).
F is now a complement with its delivery stated per Decision 1 answer,
and it is not in any ranking;
the ranking that remains is C > H for the CI surface,
and its last place's reason (four steps must be right) is H's alone.
Proposal changed:
the Decision 3 F section and the removal of F from the ranking.

### The read of 2026-11-28, Medium findings

#### M1, what changed since the rule last ran, and answers in words only

Checked:
the 2026-10-30 harness's `PATH` and toolchain,
the 2026-11-07 count,
the machine-state facts,
and the reinstall record.
The finding is right:
the earlier draft answered in words ("cited",
"beside it") and said nothing about what changed.
What changed is now its own section:
the reinstall of 2026-11-16 (`doc/troubleshooting/fcitx-flatpak-xiaohe.md`),
the driver cache gone with it,
the floating channel's components gone with it,
the proxy no longer first on `PATH`,
and the 2026-11-07 run's own `cargo` and driver unrecorded.
The 2026-10-30 warm with the toolchain-exposed cargo and the 516 findings of 2026-11-07 are both
stated in that section with their paths.
Its "which the owner declined" is answered under H1.
Proposal changed:
the section "What changed since the rule last ran",
and the 516 and ranking figures are now in the body.

#### M2, statements with no line in a kept log

Checked:
every figure and cause the finding lists was grepped in the folder.
The finding is right on each item:
the statuses and times of M2b,
M3 and M5 are not there;
the M5 probes measured the rustc-dev skip,
not the driver gap;
`fontconfig` appears in no kept file;
the two cache-skip records stop inside the message;
`results.tsv` records `still-failed` after the networked fetch;
"distinct positions" has no command behind it;
and the fifteen clean packages rest on absence.
What this round did:
M10 re-measured all three skip statuses for both causes with the status lines kept;
M7 and M8 kept their statuses and times;
the unkept figures are gone from the text;
the two failure causes and the two cache-skip causes are stated as not recorded;
"a cache gap a network fetch would close" is withdrawn and replaced by the `still-failed` row;
and "distinct positions" is replaced by "finding records".
M3's container state is stated where M3 is described.
Proposal changed:
the M3 and M5 descriptions,
the skip section,
and implication 7.

#### M3, which `cargo` runs, and the shadow launcher

Checked:
the machine state,
the shadow's README section "Toolchain and registry settings",
and `cargo_toolchain` and the command the shadow builds (`src/toolchain.rs`,
`src/cargo_run.rs`).
The finding is right that the earlier draft stated the host's state as "both non-proxy programs"
while the first `cargo` is a launcher.
What the proposal now says:
the first `cargo` is the repository's shadow launcher,
which starts the first `cargo` on `PATH` that is not a shadow under its own name with the environment
as inherited;
the next candidate is `.cache/toolchain-bin/cargo`,
a link into the dated toolchain,
so the child Dylint spawns is the same non-proxy binary M1 modelled,
and the shadow's own `RUSTUP_TOOLCHAIN` handling is only for its catalog calls.
That is read in the shadow's README and source and not run in a container;
the proposal says so.
The option the finding asks to weigh (the shadow restoring the variable for every cargo it starts)
is Decision 2's Design S,
with what it takes away.
What a proxy-first tree skips is stated there too:
the shadow's trust checks and lean containment for that build.
Proposal changed:
the machine-state section,
M1's description,
and Decision 2.

#### M4, options named or half-described, and upstream's silent rebuild

Checked:
the plan's route list,
the linter's `MONOCHROMATIC_LINT_CARGO` handling,
`is_outdated` at `v6.4.0`,
and the Dylint README's cache list.
The finding is right on each mechanism it names.
What changed:
Answer A states the route problem and the driver root;
Answer B names the package that warms and what an offline failure does (undecided,
and now said so);
Answer I names who installs the components (the plan's row) and when (at install,
and by rustup's
auto-install when a directory's file names an absent toolchain,
measured in M11);
Answer C states how the checkout enters as undecided;
Answer H names the wider cache list and what the job's four steps are;
option M is described with cargo's `[env]` semantics and its missing measurement;
D3 states its stream (stderr);
D1 states its keying (by cause,
unless the owner wants any skip);
D2 offers the field only and says why the code variant is not offered.
E's silent rebuild is stated:
an asset whose `-V` fails is replaced by a local build (`is_outdated` warns and returns `true`),
which needs the proxy,
the network and a clone.
Proposal changed:
the Decision 3 and Decision 4 answers,
the skip parts,
and the combinations section.

#### M5, adjacent pairs leaning on state or effort

Checked:
each pair the finding names.
The old order is gone.
The provider ranking is now among local providers only (B,
Z,
A),
its pairs rest on delivery with the pause stated as A's zero delivery today,
and the reversal when the pause lifts is written down.
The CI pair (C over H) rests on how much of the job's state must be right.
The skip parts are four separate answers and not one ranking.
The "two readers the repository's rules name" sentence is gone;
the criterion is stated as the proposer's choice
(the repository's rules name a "target workload" and no two readers).
No pair now cites effort or a state that changes as its reason,
except A's pause,
which is stated as delivery (nothing runs while it holds) and is A's alone.
Proposal changed:
the whole ranking section.

#### M6, the seven questions

The finding is right about all seven.
Each question is now one decision,
stated with what a yes leaves undecided,
and the ones that were bundled are split:
the toolchain from the provider,
the variable's mechanism out of the provider pick,
L out of the provider pick,
and the CI surface out of the local one.
Question 5 quotes the issue-6316 comment so the owner reads the text they are asked about.
The `--quiet` consequence the owner never saw is now part of Decision 5 and its question,
with the record that says so.
The issue's third request is stated as met in the form M3 gives it,
with what is still missing (a host run) and where the counts would live (question 8).
The workflow's later build of the linter is mentioned where the CI section describes the job.
Proposal changed:
the questions,
the issue section,
and the skip section.

#### M7, records that bear and are not cited

Checked each at the passage named:
`doc/planning/monochromatic-lint-skip-warning-and-fence-marker.md` (quoted in the skip section),
the owner's 2026-10-19 sentence that a package with missing prerequisites "is skipped,
and the linter
says so" (`doc/decision/rust-closure-lint-dylint.md`;
it is the sentence behind the skip's shape and is
now cited where the skip is described),
`doc/planning/github-hosted-runners-for-package-checks-and-a-guest.md`
(its advisory job installs `monochromatic-lint` and its driver the way `repo-tools-setup` installs
the others,
and its "What it cannot do" says a hosted job cannot replace the closure check;
that is cited in the CI section as the neighbouring proposal that prices the same surface),
and the closure package's README,
whose sections on the exact nightly and on the coupling to one
nightly bear on Answers F and P and are now cited there.
The handover line the finding calls stale (`doc/handover/questions-for-the-owner.md`) is not cited
here and is not edited:
this round may not edit `doc/handover/`,
and the report says the line needs
the coordinating session's edit.
Proposal changed:
the citations named here.

#### M8, M3's own facts

Checked by recounting the folder's files:
seven packages with findings totalling 1674,
1502 of them in `package/git-policy/cli`,
1611 in the four packages the issue names,
a wall-time sum of 1688 s over the thirty rows,
and the configuration's own comment "Warnings until no finding is left".
All of it is now in the M3 description and in implication 10,
with the disk cost of a warm build (`package/ide/app/target/dylint/debug` at 2.09 GiB,
`doc/planning/lean-build-dirs.md`) beside it.
The finding's point that a skip matters only where the rest is clean is stated as implication 10.
Proposal changed:
the M3 description and implication 10.

### The read of 2026-11-28, Low findings

#### L1, line citations off at the merged commit

Right.
The linter README and the mpm-shadow README citations now name sections ("Closure check",
"What a run does",
"Machine requirements and skips",
"Prerequisite manifest",
"Toolchain and registry settings") instead of line numbers,
which cannot drift;
the Dylint cache list is cited by its own README at the tag without a line range,
and the repository's own files are cited by path and symbol.
The one range that remains is the evidence folder's own logs,
which do not move.
Proposal changed throughout.

#### L2, file names, order and unlabelled files in the evidence

Right.
The two names are `run.txt` in their folders and are cited that way.
The measurement sections now run in the order of the round's own work (M1,
M2a,
M2b,
M4,
M5,
M6,
M3) and then the second round's (M7,
M8,
M10,
M11),
and each says which round it belongs to.
The two runs in the earlier round's `execve-counts.txt` are described as two arms in one file,
and their counts are cited with that caveat;
this round writes one counts file per run (`c-execve-counts.txt`,
`b-execve-counts.txt`).
The M2a script bugs are not used by any sentence,
and the folder's README says so.
Proposal changed:
the citation names and the M5 description.

#### L3, text defects in the options

The duplicated "and red in CI for the same packages" line is gone.
D3's stream is decided and stated:
stderr,
where `--quiet` does not drop it (the filter drops warnings from the record stream only).
Proposal changed:
Decision 5,
Part 3 and Part 4.

#### L4, "reproduced" and the cold registry

Right.
M1's words match the issue's line and the cause of the missing crate differs;
the M1 description says so.
M6 is labelled an illustration of the class.
Proposal changed:
the M1 and M6 descriptions.

#### L5, small statements of tool behaviour that overreach

The clone's date floor is stated (`extra_symbols.rs`:
`None` for the channel word `nightly` and for a
dated channel before `EXTRA_SYMBOLS_SINCE`).
Option C's temporary-directory sentence is corrected:
M1's refusal was `destination path '/tmp/.tmpEjtkLF' already exists and is not an empty directory`,
its cause was not established,
and later containers used `TMPDIR`.
M's `force = true` reason is corrected:
Dylint removes `RUSTUP_TOOLCHAIN` before the build,
so the stated reason for `force` was off,
and M is described with cargo's `[env]` semantics only.
K's component list is not inert,
and the finding is wrong on that point:
on rustup 1.29.3 a `rustup show active-toolchain` in a directory whose file names an absent toolchain
installs that toolchain and the file's components,
warning that auto-installation is deprecated (M11).
The finding's source for its claim is the rustup 1.28.0 announcement;
this round measured the behaviour of the rustup on the machine.
Proposal changed:
the `cargo dylint` order section,
the M1 and C descriptions,
option M,
and Answer K1.

#### L6, letters reused, and the cleaner

The letters are no longer the only names:
every answer has a word name ("a toolchain file generated from the lock") and a letter,
and the 2026-10-30 record's letters are named in Decision 6 where they are reused.
The cleaner question is answered:
`pins_in` reads the lock,
`.cache/toolchain.env.json`,
and a checkout-root `rust-toolchain`/`rust-toolchain.toml`,
so it does not read a package-level file;
Answer K1 is kept because the lock names the same nightly,
and Answer K2's date is kept by nothing the repository controls.
Proposal changed:
the answer names and the combinations section.

### The read of 2026-11-27, its sixteen findings

That read's sixteen findings are answered in the earlier draft's own section,
and this revision keeps the answers where they still hold.
What changes in them:
its F9 (precedent) answer named a ranking this revision no longer gives,
so the ranking is restated rather than referred to;
its F13 answer said the issue-6316 answer is not an owner pause,
and the comment is now quoted (Decision 7);
its F2 and F1 requests (the proxy arm,
the shadow) are met by M7,
M8 and the shadow's source,
as above.
Nothing in this revision rests on that read's text for a measured fact.

## Measured, inferred, and not known

Measured,
with the log beside it:
the machine's toolchains,
components,
versions and `PATH` order (`.../machine/machine-state.txt`);
Answers K1 and I end to end with the proxy first (M7,
M8);
the statuses a skip returns for both skip causes (M10);
what rustup does with a file naming an absent toolchain (M11);
the library and driver build times and the clone counts for a dated and a floating name (M7,
M8);
the driver's `-V`,
its RUNPATH and its `NEEDED` library (M7,
M8,
M4);
the thirty selectable packages' outcomes and counts (M3,
recounted);
the rpath's effect on loading (M4);
and the workflow's state (the workflow itself and the 2026-11-01 run excerpt).

Inferred,
and labelled where it appears:
the shadow's hand-off (read in its README and source,
not run);
that M1's child environment is the host's,
through that hand-off;
that the 2026-11-07 count was taken in the state the records describe;
that option M's `[env]` reaches the linker (unmeasured);
and that the ignored host test would fail today (it was not run).

Not known:
which answers the owner wants;
what a nightly that moves rustc's API does to Answer I beyond waiting for Dylint;
what a CI job costs in wall time;
and why the daily toolchain update timer is disabled.

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
- It did not run `cargo-shadow` in a container,
  so the shadow's hand-off is read and not run.
- It did not measure option M,
  the linter-child `PATH` design,
  the shadow-restores-the-variable design,
  Answer H's CI job or Answer C's image.
- It did not touch `doc/handover/`,
  `doc/decision/`,
  or the 2026-11-25 troubleshooting record whose correction the coordinating session made.

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
- `gate-read/review.md`:
  the same-vendor read of 2026-11-27,
  whole.
- `gate-read-second-2026-11-28/`:
  the same-vendor read of 2026-11-28,
  whole,
  with the probes that read ran.
- `m1-f1-nonproxy-cargo/`,
  `m2a-proxy-alone-fails-at-rustc-dev/`,
  `m2-f2-proxy-cargo/`,
  `m4-f10-rpath-and-other-home/`,
  `m5-option-k-end-to-end/`,
  `m6-clean-driver-and-probe/`,
  `m3-f3-skips-with-a-driver-1/` and
  `m3-f3-skips-with-a-driver-2/`:
  the earlier round's measurements,
  with what their logs do and do not hold stated in the text.
- `m7-k-with-minimal-floating-channel/`,
  `m8-option-i-on-the-host-like-floating-build/`,
  `m10-skip-statuses/` and
  `m11-rustup-does-not-install-an-absent-toolchain/`:
  this round's measurements,
  each with `run.txt` and its logs.
- `scripts/`:
  every in-container script,
  so each run can be repeated,
  and the quotation checker with its list.
- `upstream/dylint-v6.4.0-excerpts.txt`:
  the upstream lines this proposal cites,
  with the tag and the tarball's digest.
