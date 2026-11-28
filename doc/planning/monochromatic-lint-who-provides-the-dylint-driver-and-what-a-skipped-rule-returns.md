# Who provides the Dylint driver `rust/avoidable-closure` needs, and what a skipped rule returns

Purpose:
the owner has to decide who builds the Dylint driver that `rust/avoidable-closure` loads,
which compiler the lint's library is built with,
and what a run says when the rule did not check a package.
This file states what is measured,
what is read from source and records,
and what is only inferred,
and puts each decision to the owner with the cost of every answer.
Status:
a proposal,
unapproved and non-final,
for issue 6643.
The owner declares completion;
nothing here is approved or adopted,
and nothing here is built.

Four same-vendor gate reads have been taken of this file,
each a second pass by another Claude instance and none an independent review:
the read of 2026-11-27 (4 High,
6 Medium,
6 Low),
the read of 2026-11-28,
second (4 High,
8 Medium,
6 Low),
the read of 2026-11-28,
third (3 High,
8 Medium,
6 Low),
and the read of 2026-11-28,
fourth (1 High,
6 Medium,
6 Low).
All four are kept whole with their probes in
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/`
(`gate-read/`,
`gate-read-second-2026-11-28/`,
`gate-read-third-2026-11-28/` and
`gate-read-fourth-2026-11-28/`).
Every finding of all four is answered,
one by one with the line or the run each answer rests on,
in
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/gate-read-third-2026-11-28/findings-answered.md`,
together with the reports of the revisions that answered them.
That file also says which findings a revision found wrong;
this one found none of the fourth read's findings wrong,
and changed what each of them names.
None of those answers is needed to answer the questions in "The questions for the owner".
This fourth revision changed text and records only:
it ran no container,
no build and no test,
and every measurement named here is the third round's.

## What you are asked, and what this proposal recommends

Eight questions follow in this section,
one per line of decision,
each with the answer this proposal recommends,
the answers it was chosen over,
what that recommendation rests on,
and what a yes leaves undecided.
The mechanism and cost of every answer,
and the reason for every adjacent pair in each ranking,
are in the decision sections named beside each question.

1. **Which compiler the lint library is built with.**
   Recommended:
   **K1**,
   a `rust-toolchain.toml` in `package/linter/monochromatic-lint-closure`
   generated from the lock's `rust` entry,
   over K2,
   a date written by hand in the same file,
   and I,
   the components on the rustup channel named `nightly`.
   Basis:
   measured on one image and one day.
   With K1's file and the toolchain variable exported,
   the rule returned six findings on the copied package (M15a arm C);
   with the components on a fresh install of the channel named `nightly`,
   the same linter returned one processing failure and status 2 (M12 arm A).
   **K1 returns findings only while the lock lags the channel.**
   Once the lock names a compiler Dylint 6.4.0 cannot build on,
   K1 returns the same `E0603` and status 2 as I,
   measured with a file naming `nightly-2026-11-28`,
   whose compiler is the channel's `c82ce1563` (M16).
   What the channel names on this machine is measured and what adding the components does to it is not:
   this host's floating channel is `5df8c02af 2026-11-16` with no `rustc-dev`
   (`machine/machine-state.txt`),
   and a fresh `nightly` install resolved `c82ce1563 2026-11-27` on the day of the runs (M12).
   The arm that would settle what
   `rustup component add ... --toolchain nightly` does to this host's older channel
   is named in "What is true today about the compiler and Dylint",
   and it was not run.
   A yes leaves undecided who writes the file,
   which lock field it is written from,
   and when it is rewritten (Decision 1).

2. **How the toolchain variable reaches the programs Dylint starts.**
   Recommended:
   **T**,
   the lint task's own environment,
   with the rustup proxies first in `PATH` and `RUSTUP_TOOLCHAIN` naming the toolchain of question 1,
   over N,
   no change with a person setting it by hand,
   CL,
   the linter putting the proxy first for its child,
   and SH,
   the shadow exporting the lock's toolchain for every cargo it starts.
   Basis:
   measured state in the arms that exported the variable (M15a arm C,
   M16);
   the arms that left it unset are M12,
   M14 arm A and M15a arms A,
   B and D,
   so it is not the state every measurement ran in.
   That a `mise` task can create the state is read and not run.
   A yes leaves undecided whether `cargo-shadow` gets a rule for the lint's build,
   and whether a person calling the binary by its full path gets the rule at all (Decision 2).

3. **Who builds the driver on a person's machine.**
   Recommended:
   **B**,
   a step in the root lint task that runs the linter's own warm command
   before the offline run,
   over Z,
   the documented manual warm,
   and A,
   a plan row that builds the driver at install time.
   Basis:
   the command is measured and the step is not.
   Run by hand in a container,
   the warm command built the driver and the rule then returned the six findings (M15a arms B and C);
   under Answer I the same command fails with the `E0603` before a driver exists (M12 arm B);
   K2's date is warmed by the same command as K1's,
   on the date K2 names.
   That a `mise` task can run that command as a step is read and not run,
   as for T.
   A yes leaves undecided what a lint run does without the network,
   and which package's task warms the driver (Decision 3).

4. **Whether a pull request gets the rule at all.**
   Recommended:
   **C**,
   a prepared image the job runs,
   over H,
   the job provisioning the tools itself,
   and neither.
   Basis:
   not measured.
   The job's state is read from `.github/workflows/advisory-core.yml`:
   it cannot start the linter today,
   and its toolchain step installs a minimal nightly with no components.
   A yes leaves undecided how the checkout enters the image,
   whether that run has the network,
   and where the image lives (Decision 4).

5. **What a run says when the rule did not check a package.**
   Four parts are choices,
   each answerable alone and combinable,
   and a fifth is a defect this work fixes rather than a choice:
   the severity,
   a severity per cause (J) over keeping one `warn`;
   a marker field,
   `"checked": false` (D2) over none;
   the exit status,
   one keyed by cause (D1) over one keyed on any skip or over none;
   a closing count,
   one line on standard error (D3) over none;
   and a run that checked nothing and printed nothing,
   which is fixed without asking (issue 6667),
   with the owner asked only whether a toolchain the library names and the machine lacks
   reports as a warning-level skip or as a processing failure,
   the processing failure being this proposal's recommendation.
   Basis:
   measured for today's statuses (M10) and for the silent pass (M14 arm A);
   the four recommended changes,
   and the fix,
   are not measured.
   A yes to each part leaves undecided the option's name,
   the field's name,
   the status number,
   the line's wording,
   and the report's shape.

6. **Whether the two Dylint tools stay at `@latest`.**
   Recommended:
   keep `@latest`,
   over L,
   an exact request for both tools.
   Basis:
   line read of the plan's rows and of the record's "update EVERYTHING (not just rust) daily";
   the cost of the alternative is stated,
   and nothing here is measured.
   Nothing moves either row by itself today:
   the daily update timer is disabled,
   so a person runs `toolchain:update`,
   which is one command for both rows;
   a pin needs an edit of the rows and the lock.
   A yes leaves undecided nothing further;
   a yes to a pin leaves undecided which version to hold (Decision 6).

7. **Where the counts of the rule's one measured repository-wide run live.**
   Recommended:
   beside the 2026-11-07 count in `doc/planning/repository-linter-stays-at-zero.md`,
   over a record of their own.
   Basis:
   measured counts from the kept files of that run (M3,
   recounted);
   a placement,
   not a measurement.
   A yes leaves undecided whether that run is repeated on this machine (Decision 7).

8. **Which name `latest` should mean for the library's compiler.**
   Recommended:
   **the lock's dated name**,
   over the name of the rustup channel `nightly`.
   Basis:
   line read of the plan row,
   the lock and the owner's recorded words;
   nothing here is measured.
   The lock's name moves when `toolchain:update` runs,
   and today that is only when a person runs it:
   the timer is disabled.
   Nothing in the repository moves the channel either,
   and no commit records a move of it.
   A yes to the lock's name leaves undecided who runs the update,
   and whether the timer is re-enabled.
   A yes to the channel leaves undecided who moves the channel,
   and when (Decision 8).

The one thing the questions cannot be answered without,
and which this revision states first because an earlier one stated it wrongly:
`toolchain:install` installs a **dated** toolchain and adds the row's components to **that** toolchain.
It does not put components on the rustup channel named `nightly`,
and nothing in the repository moves that channel.
The lines are in "The plan row installs a dated toolchain,
and puts the components on it".

### Which questions come first

- **Questions 1 and 2 first,
  together,
  with question 8.**
  Decision 1's ranking assumes Decision 2's answer T,
  and T's value is question 1's name,
  so neither can be answered as if the other stood still.
  Question 8 asks which name the owner wants `latest` to mean,
  which is the preference behind question 1's answers;
  it starts no work and can be answered at any time.
- **Question 3 follows them.**
  The drivers it ranks are built with the compiler question 1 chooses,
  and under Answer I the warm command fails before a driver exists (M12 arm B).
- **Questions 4,
  5 and 7 can wait.**
  Question 4's answers are unmeasured and the job cannot start the linter today,
  so a CI answer waits on work no answer here includes.
  Question 5 changes what an absent rule says,
  not whether it runs,
  and its fifth part is a defect the work fixes without a question.
  Question 7 changes where a count lives.
- **Question 6's recommendation is the state today.**
  Keeping `@latest` is the plan's current row;
  nothing starts unless the owner chooses a pin.

### What each kind of claim rests on

- **Measured**:
  a log in the evidence folder,
  named as `M<n>` with its arm and file.
- **Line read**:
  a path and the symbol or the section in it.
  No line numbers:
  they move with the file,
  and an earlier read of this proposal found citations that had.
- **Inferred**:
  a conclusion drawn from a source or a record that no run here confirms.
  Every one is also listed under "Measured,
  inferred,
  and not known".
- **Not known**:
  stated as such where it matters.

### What would change the recommended order

- A `cargo-dylint` release that compiles on the channel's current compiler.
  Upstream has merged that change and released nothing since 2026-11-10
  (measured,
  `probes/gh-dylint-releases-and-pull-request-2203.txt`).
  Such a release would let Answer I deliver without a new file,
  and would make question 1 a choice between two working answers.
- **The lock moving to a compiler Dylint 6.4.0 cannot build on.**
  That happens at the next lock move past the break,
  and nothing moves the lock on a schedule:
  the timer stands `disabled; inactive (dead)` on this machine,
  read by the second gate read and kept in
  `gate-read-second-2026-11-28/probe-daily-update-timer-and-lock-state.txt`.
  Measured for that state,
  a file naming `nightly-2026-11-28` returns the same `E0603` and status 2 as I (M16).
  So K1's lead lasts until the lock moves past the break,
  and not longer:
  the next `toolchain:update` a person runs resolves today's channel,
  and K1 then returns status 2 with I until a `cargo-dylint` release carries upstream's fix.
- Whether the lint task inherits the lock's exported `RUSTUP_TOOLCHAIN`,
  a dated name.
  It was not measured.
  If it does,
  question 2's answer T is one step cheaper than stated,
  and Answer I needs that value overridden to the channel name rather than set.

## The issue, and what it asks

Issue 6643 is titled
"monochromatic-lint:
rust/avoidable-closure is skipped on every host run because its Dylint driver is absent,
and a skip reads as a pass".
It opens:

> `rust/avoidable-closure` runs on no host run of the linter on the coordinating session's machine,
> so every count of findings taken there leaves that rule out.

Its record of the harm:

> Rounds and second passes of 2026-11-27 each noted the skip and went on,
> and a run "without findings" was read as clean more than once before the line was noticed.

It asks three things:
who puts the driver on a machine,
the options with what each costs,
and one run of the rule over the Rust packages the work owns with its findings counted.
The third is met in the form the merge allows,
and the counts are in "M3",
in "The earlier rounds' measurements,
and what their logs hold":
the run of 2026-11-27 over all thirty selectable lock directories with a driver for the dated toolchain,
seven packages with findings,
1674 findings in total and 1611 of them in the four packages the issue names.
Whether that is the record the counts should live in is question 7.

## How the rule runs

`rust/avoidable-closure` is not a rule of the linter's own analysis.
The linter's README,
section "Closure check"
(`package/linter/monochromatic-lint/README.md`),
states that it runs the type-aware Dylint lint `avoidable_closure`
of `package/linter/monochromatic-lint-closure` over each Rust package that has a selected file.
The root configuration selects it for every Rust file,
with the library path relative to the configuration file
(`monochromatic-lint.config.jsonc`):

```jsonc
// monochromatic-lint.config.jsonc
"rust/avoidable-closure": {
  "severity": "warn",
  "library": "package/linter/monochromatic-lint-closure",
},
```

After the per-file rules,
`run_closure_step_at` (`package/linter/monochromatic-lint/src/closure_step.rs`) does this,
in order:

1. It groups the selected Rust files by the nearest `Cargo.toml`
   (`group_packages`,
   `src/closure_packages.rs`),
   so the unit of work is the package and not the file,
   and a directory whose name ends in `.fuzz` or `.bench` is exempt.
2. It returns a note and nothing else under `--fix`,
   because the lint has no machine fix.
3. It reports a configuration error,
   at processing-failure severity,
   when the configured library directory holds no `Cargo.toml` (`missing_libraries`).
4. It runs `cargo dylint --version` once (`dylint_probe`,
   `src/closure_cargo.rs`).
5. It decides on a machine-wide gap:
   no `cargo` at all,
   or cargo's own `no such command` for `dylint`,
   skips every selected package (`ProbeFailure::Absent`);
   a `cargo` that starts and exits 126 or 127,
   or fails without that evidence,
   returns one processing failure and status `2` (`ProbeFailure::Broken`,
   measured in M17,
   where an exported `RUSTUP_TOOLCHAIN` naming a toolchain rustup does not have
   makes `cargo dylint --version` exit 1:
   one record at the configuration file,
   status 2);
   a compiler without `rustc-dev` skips every selected package (`rustc_dev_missing`).
   While `MONOCHROMATIC_LINT_CARGO` names a replacement program the component check does not run,
   because the replacement stands for the whole toolchain.
6. It checks the driver once per distinct library (`driver_gaps`,
   `src/closure_driver.rs`),
   which asks `rustup show active-toolchain` in the library directory (`active_toolchain`,
   `src/closure_driver_probe.rs`)
   and skips the packages of a library whose driver cannot be had (`driver_state`,
   same file).
7. For every remaining package it reads the prerequisites,
   checks the locked dependencies offline,
   and runs the lint (`check_package`,
   `src/closure_step_package.rs`).

The run itself is one process per package (`run_dylint`,
`src/closure_cargo.rs`):
`cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked --message-format json`,
with `CARGO_NET_OFFLINE=true` on the child and
`DYLINT_RUSTFLAGS="-W avoidable_closure -W unfulfilled_lint_expectations"`.
Only the library and the binaries are compiled,
so `#[cfg(test)]` modules,
`tests/`,
examples and benchmarks are not checked
(README,
section "What a run does").

### What `cargo dylint` itself does, and in which order

The linter calls one program;
`cargo dylint` then does four things,
and the order matters for every answer in this file.
At upstream tag `v6.4.0`,
the version `toolchain.lock.json` records for `cargo:cargo-dylint`
and the version the driver crate is pinned to:

1. It computes the toolchain name once,
   as the first word of `rustup show active-toolchain` run in the library directory
   with `RUSTUP_TOOLCHAIN`,
   `CARGO` and `RUSTC` removed
   (`internal/src/rustup.rs`,
   `parse_toolchain`,
   and the sanitizing implementation).
2. It builds the lint library before it looks for a driver,
   with `cargo build --release` into the package's own `target/dylint/libraries/<toolchain>`,
   with the same three variables removed and `RUSTFLAGS` cleared
   (`dylint/src/library_packages/mod.rs`,
   `internal/src/rustup.rs`).
   Measured order:
   the library failed and `~/.dylint_drivers` was still absent (M12 arm B).
3. It builds the driver when `<drivers>/<toolchain>/dylint-driver` is absent or reports a version below
   `cargo-dylint`'s,
   or when the driver's `-V` fails at all
   (`dylint/src/driver_builder/mod.rs`),
   from a throwaway crate that pins `dylint_driver = "=<version of cargo-dylint>"`
   and whose `rust-toolchain.toml` names the toolchain and the components
   `llvm-tools-preview` and `rustc-dev`,
   with an absolute rpath into `<the toolchain's rustup path>/lib`.
4. It checks the package with `RUSTUP_TOOLCHAIN` set to the computed name
   and `RUSTC_WORKSPACE_WRAPPER` set to the driver (`dylint/src/lib.rs`).

Three consequences of that order and those sanitised environments are measured:

- The library build runs with `RUSTUP_TOOLCHAIN` removed,
  and the library's own `.cargo/config.toml` sets `rustflags = ["-C", "linker=dylint-link"]`.
  `dylint-link` reads `RUSTUP_TOOLCHAIN` itself (`dylint-link/src/main.rs`),
  so without a `cargo` that puts the variable back the link fails with
  `Error: environment variable not found: RUSTUP_TOOLCHAIN`
  (measured:
  `dylint-link --version` alone prints that error in the probe image,
  M12 run log;
  the library build's failure is in `m1-f1-nonproxy-cargo/log/warm.err.dylint-link-failure.txt`).
- The compiler that builds the library is the one the `cargo` in `PATH` resolves to in the library directory,
  not the toolchain the linter's caller exported:
  a direct toolchain link builds with the dated compiler,
  and a rustup proxy builds with the toolchain the library directory's file names,
  or with rustup's default
  (measured:
  241 dated `rustc` executions in M15a arm B,
  and the floating `rustc` in M8 arm B).
- Whether the driver build clones `rust-clippy` depends on the channel word of that toolchain name.
  `clippy_utils_has_extra_symbols` (`driver/build/extra_symbols.rs` at `v6.4.0`)
  returns `None`,
  and so skips the clone,
  when the channel is exactly `nightly`,
  and when the dated channel is older than `EXTRA_SYMBOLS_SINCE`;
  a dated channel at or after that floor clones.
  Measured on both sides:
  the dated name ran 3 `git` executions (M15a arm B,
  87 s),
  the floating name none (M8 arm B).

### What each absence produces

Every absence in this list is a distinct branch in the source,
and they do not behave the same.

- No `cargo`,
  or no `cargo-dylint` behind it:
  `ProbeFailure::Absent`,
  whose only accepted evidence is cargo's own `no such command` for `dylint`.
  The run skips every selected package.
- A `cargo` that starts and exits 126 or 127,
  or fails without that evidence:
  `ProbeFailure::Broken`,
  one `core/processing-failure` finding at the configuration file and exit status `2`
  (source,
  and measured in M17).
- A compiler without `rustc-dev`,
  as the linter's own `rustc --print sysroot` sees it:
  a machine-wide skip (`rustc_dev_missing`),
  which runs before any driver is looked for
  (measured,
  M10 arm A,
  M15a arms A and D).
- A driver that is absent or older than `cargo-dylint`,
  with the pinned `dylint_driver` crate
  unresolvable offline:
  one skip per package of that library (`driver_state`,
  `driver_gaps`).
- A package whose locked dependencies are not in the cargo cache:
  a skip naming `cargo fetch --locked` (`dependencies_missing`).
- A prerequisite named in `monochromatic-lint.prerequisites.jsonc` that the machine lacks:
  a skip naming the task that makes it (`src/closure_probe.rs`,
  README section "Prerequisite manifest").
- **A driver whose toolchain name cannot be read at all**,
  such as a library file naming a toolchain
  rustup will not install:
  `driver_state` returns `Unknown`,
  the reason is a note that appears only under `--debug`,
  and the packages still run.
  Measured end to end with auto-install off and a library file naming an uninstalled toolchain:
  the linter printed **no record at all and exited 0**,
  and its `--debug` notes said the driver check did not run and that the package produced
  "0 report(s) from the compiler" (M14 arm A).
  The arm's state:
  the image's floating channel carries `rustc-dev`,
  `llvm-tools` and `rust-src` (the `Containerfile`),
  and `RUSTUP_TOOLCHAIN` was unset.
  That is Answer I's compiler state,
  not K1's or K2's recommended one:
  with the floating channel lacking `rustc-dev` and the variable unset,
  the linter skips visibly at its machine-wide check (M15a arms A and D).
  That the run checked nothing is inferred rather than logged:
  the arm kept no output or status from `cargo dylint`,
  and a `cargo dylint` that failed would have added a processing failure (`check_package`),
  so it exited 0 and printed nothing;
  the positive control is M15a arm C,
  where the same package returns six findings with the right driver and compiler.
  That is a run that checked nothing and said nothing:
  none of the four skip choices in Decision 5 catches it,
  issue 6667 holds it,
  and Decision 5's fifth part is its fix.
- A `cargo dylint` run that fails after the probe passed,
  because the library build or the check failed
  for a reason no probe reads:
  a `core/processing-failure` finding and status `2`,
  not a skip
  (`check_package`;
  measured in M12 arm A and M16 arm B).
- A driver probe that answers `Ready` while the run still cannot build the library,
  because the probe reads the driver and the crate cache and never builds the library
  (source,
  `driver_state`;
  illustrated by M6 in the earlier round).

### What a skipped rule returns

Three places in the code decide it.

1. The skip is a diagnostic of the same rule code as a real finding:
   `Diagnostic::new(RULE, Severity::Warn, ...)` in `skip_finding`
   (`src/closure_step_package.rs`),
   where `RULE` is `rust/avoidable-closure`.
   Its file name is the package's `Cargo.toml`,
   and its position is the first byte of it.
   Its message starts `rust/avoidable-closure skipped <package>: ` and ends
   "Nothing in this package was checked by this rule,
   so a run without findings says nothing about its closures."
   The severity is fixed at `warn` on purpose:
   the function's comment says
   "the severity is fixed because the configured severity belongs to the closure reports".
   The shape comes from the owner's answer of 2026-10-19:
   "A package whose prerequisites are missing on the machine is skipped,
   and the linter says so."
2. Skips are merged into the same finding list as findings before output,
   so a consumer that reads only the code cannot tell them apart;
   the file can:
   a skip names the package's `Cargo.toml`,
   a real finding of this rule names a `.rs` file.
3. The exit status counts the skip as a warning and nothing else (`exit_status`,
   `src/run_output.rs`):
   a warning alone leaves `0`;
   `--max-warnings` counts it and exceeding the limit gives `1`;
   only a finding marked as a processing failure gives `2`.
   `--quiet` and `--silent` hide it with the other warnings:
   `if options.silent || (options.quiet && finding.severity == Severity::Warn)`
   (`src/run_output.rs`;
   README,
   section "Machine requirements and skips").

That the quiet form hides a skip is a consequence the owner was never asked about,
and the record says so in the coordinator's own words:

> A consequence the owner has not been asked about:
> `--quiet` and `--silent` now hide a skip with the other warnings,
> where the message on standard error was kept by `--silent`.

(`doc/planning/monochromatic-lint-skip-warning-and-fence-marker.md`.)
It is part of Decision 5,
not something the contract already settled.

Measured in the containers of the earlier rounds,
with the released linter over a copied
`package/wg/cli-quicker-exempt` and no driver (M10,
with the status lines kept):

- The default run printed one JSONL record and exited `0`.
- `--max-warnings 0` printed the same record and exited `1`.
  A gate can make a skip fail today with no code change,
  at the price of failing every package that skips.
- `--quiet` printed nothing and exited `0`,
  so the quiet form of a run looks exactly like a clean run.
  `--silent` hides warnings by the same source line;
  M10 ran `--quiet` only.
- The skip's message names the command that would fill the caches,
  built by `warm_command`
  (`src/closure_driver.rs`):

  ```sh
  env --unset=CARGO_NET_OFFLINE cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked
  ```

- `--rules` reports the rule as `"suppressible":false` with the single option `library`,
  so a project cannot turn a skip off without turning the whole rule off.

## What the repository installs today

No component of this repository builds or installs a Dylint driver.

### The plan row installs a dated toolchain, and puts the components on it

`toolchain.plan.jsonc` holds the `rust` row,
and its own comment states the mechanism:

> Floating nightly channel,
> resolved to the dated toolchain string;
> the pin rides in the MPM rustup package id because that adapter keeps only the id.

The row's `packageId` is `"{version}"`,
its `env.RUSTUP_TOOLCHAIN` is `"{version}"`,
and its components are `clippy`,
`rust-src`,
`llvm-tools`,
`rustc-dev` and `rustfmt`
(`toolchain.plan.jsonc`,
the `"rust"` row).
`expand_version` (`package/mpm-shadow/cli/src/toolchain/model.rs`)
describes itself as
"A copy of the declaration with `{version}` replaced by `version` in `packageId` and in every `env` value",
and `rustup_channel()` in the same file returns
"the declaration's `packageId` when it has one,
else the locked version".
The lock holds the expanded text:
`nightly-2026-11-16`,
in `version`,
in `packageId`,
and in `env.RUSTUP_TOOLCHAIN` (`toolchain.lock.json`,
the `"rust"` entry).

The installer then uses that channel for the toolchain **and** for the components:

- `install_rust_toolchain` runs `rustup toolchain install --no-self-update <channel>`,
  where `channel` is `tool.rustup_channel()`
  (`package/mpm-shadow/cli/src/toolchain/install_rust_android.rs`).
- `install_rust_components` runs `rustup component add <name> --toolchain tool.rustup_channel()`
  for each declared component,
  and `rustup target add <triple> --toolchain tool.rustup_channel()`
  for each declared target (same file).

So `toolchain:install` today installs `nightly-2026-11-16-x86_64-unknown-linux-gnu`
with `rustc-dev`,
`llvm-tools`,
`rust-src`,
`clippy` and `rustfmt`,
and the two Android targets.
It does not add anything to the rustup channel named `nightly`.
That is the opposite of what an earlier revision of this file said,
and it is why Answer I is stated with a plan row that names the channel
and an owner for moving it.

### Nothing in the repository moves the floating channel

- `git grep --line-number 'rustup update' -- ':!doc' ':!*.md'` returns no hit
  (`probes/repository-plan-lock-and-installer-lines.txt`).
- The only installs of a floating `nightly` are workflow lines,
  `rustup toolchain install nightly --profile minimal --no-self-update`:
  17 in `.github/workflows/cargo-publish.yml` and 1 in `.github/workflows/crate-release.yml`.
  They run on hosted runners and install nothing on this machine.
- `package/cargo-shadow/cli/src/lean_rustup_toolchains.rs` says in its header that the repository
  "installs a dated nightly per `toolchain:update` and rustup never removes one",
  and that "Stable releases and the channel names are never touched".
- `pins_in` in the same package reads `toolchain.lock.json`,
  `.cache/toolchain.env.json`,
  and the `rust-toolchain`/`rust-toolchain.toml` files at a checkout's root,
  so a package-level toolchain file is not read at all.

So the channel named `nightly` on this machine moves only when a person runs rustup.

### The linter's message, the warm command, the tasks and the workflows

- The linter reads the driver cache,
  mirrors Dylint's staleness rule,
  and refuses to write anything
  (README,
  section "Machine requirements and skips";
  `dylint_drivers_directory`,
  `driver_site_from_environment`,
  `active_toolchain`,
  `driver_version` and `driver_unresolved` in `src/closure_driver_probe.rs`).
- `//package/linter/monochromatic-lint:test:host` is the one ignored binary test,
  `host_real_run_over_wg_quicker_exempt` (`src/closure_binary_tests.rs`),
  which asserts a real run over `package/wg/cli-quicker-exempt` has no skip and at least one finding.
  It needs `cargo-dylint`,
  a nightly with `rustc-dev`,
  and a warm cargo cache.
  This round did not run it.
- `//package/linter/monochromatic-lint-closure:run` runs
  `cargo dylint --path <this package> --manifest-path <directory>/Cargo.toml -- --all-targets`
  with no offline setting,
  so it is a warming command as well as a lint.
- The root `lint:monochromatic-lint` task depends on `fetch:rust-dependencies`,
  which runs `cargo fetch --locked` per package.
  That fills the package locks and not the driver crate,
  which belongs to no lockfile.
- The workflows cannot start the linter today.
  `advisory-core.yml` installs the closed name list `tools: file-enforcer mpm-shadow cargo-shadow`,
  and `repo-tools-setup` accepts exactly `mpm-shadow`,
  `file-enforcer`,
  `cargo-shadow` and `git-policy-cli`.
  The last retrievable step log that reached the call,
  run `41274002049` of 2026-11-01,
  printed `task-run: line 1: monochromatic-lint: command not found` and `Error: task-cli: step exited 127`
  (`ci/ci-run-41274002049-static-step-excerpt.txt`).
  The step reports `success` only because of `continue-on-error: true`.
  The job's toolchain step installs `rustup toolchain install "$toolchain" --profile minimal --no-self-update`,
  which carries neither `rustc-dev` nor `llvm-tools` nor `cargo-dylint`.
  `task-run` does not see the toolchain environment either
  (`doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md`).

### The machine's state, and the four toolchain names

Read-only looks of the earlier rounds,
kept in `machine/machine-state.txt`,
with the correction the coordinating session made on 2026-11-27:

- `rustup toolchain list`:
  `nightly-x86_64-unknown-linux-gnu (active, default)` and
  `nightly-2026-11-16-x86_64-unknown-linux-gnu`.
- The floating channel holds `cargo`,
  `clippy`,
  `rust-std`,
  `rustc` and `rustfmt`:
  no `rustc-dev`,
  no `llvm-tools`,
  no `rust-src`.
  Its compiler is `1.102.0-nightly (5df8c02af 2026-11-16)`.
- The dated toolchain the lock records holds `cargo`,
  `clippy`,
  `llvm-tools`,
  `rust-src`,
  the two Android `rust-std` targets,
  `rustc`,
  `rustc-dev` and `rustfmt`.
  Its compiler is `1.102.0-nightly (5f4effd57 2026-11-13)`.
- `type -a cargo` lists,
  in this order:
  this repository's shadow launcher,
  then `.cache/toolchain-bin/cargo` (a link into the dated toolchain),
  then `/usr/bin/cargo`,
  which is a link to `/usr/bin/rustup`.
  `type -a rustc` has no repository launcher in its way:
  the first `rustc` is `.cache/toolchain-bin/rustc`,
  a link into the dated toolchain,
  and the proxy comes later.
- `ls -d ~/.dylint_drivers` answers "No such file or directory":
  no driver exists now.
- The daily toolchain update timer stands `disabled; inactive (dead)`.

The toolchain names in play,
which the rest of this file keeps apart:

- the floating channel,
  `nightly-x86_64-unknown-linux-gnu`,
  whose compiler is `5df8c02af` today,
  and which has no `rustc-dev` or `llvm-tools`;
- the dated name the lock records,
  `nightly-2026-11-16`,
  whose full form is
  `nightly-2026-11-16-x86_64-unknown-linux-gnu`,
  and which carries the components;
- a dated name a person pins by hand;
- the name `rustup show active-toolchain` prints for the library directory,
  which is the floating one while there is no file there,
  and the file's name once there is.

The driver directory the linter's message names is
`~/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver`
(the issue's capture,
and the same message in M8 arm A),
so a driver filed under the short `nightly-2026-11-16` name is never read by the linter.
The driver that is read is filed under the full name:
`/root/.dylint_drivers/nightly-2026-11-16-x86_64-unknown-linux-gnu/dylint-driver` (M15a arm B).

## What the owner recorded

### The owner's words, and the question each answered

`doc/planning/toolchain-always-latest.md`,
section "Owner's words",
records them "Recorded 2026-11-07,
verbatim,
in the order given".
The parentheses are the recorder's gloss,
and they are quoted with the words:

> - "Rust nightly should always be latest."
> - "Move anyway,
>   fix forward" (when a new nightly breaks a package).
> - "Make the plan file always track latest and set up a systemd cronjob to update EVERYTHING (not just rust) daily."
> - "Wait for Dylint" (the closure lint plugin `package/linter/monochromatic-lint-closure` stays as it is).
> - "Everything also covers the pnpm and cargo dep locks,
>   and the global toolchains too."

What each answered:

- The first three answered the question `doc/planning/rust-nightly-always-latest.md` leaves open,
  "how to keep it there":
  the compiler keeps moving,
  breaks are fixed forward,
  and the plan file tracks latest
  through the daily update
  (the timer that would run it is disabled today;
  see "What is true today about the compiler and Dylint").
- "Wait for Dylint" answered the question that record puts as three options for the closure package's own red:
  patch the package source now,
  wait for a Dylint release,
  or retire the lint.
  That record ranked "patch now > wait for Dylint > retire",
  and says of waiting
  "the package stays red until then,
  and it was already red".
  The owner chose waiting,
  which is a choice about the closure package's own build,
  not about pinning or not pinning the lock.
- "Everything also covers the pnpm and cargo dep locks,
  and the global toolchains too" was recorded with the
  others;
  the record does not say which question that one answered,
  and nothing in this file turns on it.

So the words do not say which toolchain the lint library should name,
and no sentence among them says "do not pin":
that phrase is not in any owner record this round read.

### What the record's "floating nightly" is

`doc/decision/rust-closure-lint-dylint.md`,
section "Root toolchain change approved on 2026-10-18":

> Add `rustc-dev` to the shared floating nightly.

and,
in the same section,
what refreshing the lock did:

> Changing the component list changes the locked options of the `rust` entry,
> so refreshing the lock also moved the floating nightly from `nightly-2026-10-17` to `nightly-2026-10-18`.

"The shared floating nightly" there is the lock's `rust` entry,
which holds a dated toolchain name
and moves when `toolchain:update` re-resolves it
(the daily timer that would run that task is disabled today).
That is the arrangement the plan implements today (the section on the plan row).
It is not the rustup channel named `nightly`,
and the record never names that channel.

The same record says what was not approved,
and what was adopted without asking:

> Still not approved:
> any other change to the root toolchain declaration,
> a pin of the nightly,
> an edit to the IDE package,
> a push,
> or a publish.

> Adopted as defaults without asking,
> open to veto:
> the floating nightly stays (no pin),
> only rustc's `expect` suppresses the Dylint lint (no comment directives),
> generated code is skipped by the lint itself (spans outside the package's own source),
> and the IDE keeps error severity because it has no findings.

So "no pin" is a default a session adopted and left open to veto;
a pin was listed as not approved,
which is neither approved nor refused.
The record also states the coupling every answer in this file has to live with:

> Dylint builds the target with the toolchain that built the library.

And the shape of a skip:

> A package whose prerequisites are missing on the machine is skipped,
> and the linter says so.

## What is true today about the compiler and Dylint

Measured today,
2026-11-28:

- The channel manifest `https://static.rust-lang.org/dist/channel-rust-nightly.toml`,
  fetched at 08:04:48 -05:00,
  is dated `2026-11-28` and names
  `1.102.0-nightly (c82ce1563 2026-11-27)`;
  the channel's commit-hash file prints
  `c82ce15637153c025d29abbaff6035a177361788`
  (`probes/todays-nightly-channel-manifest.txt`).
- The newest `dylint` release is 6.4.0 of 2026-11-10.
  Upstream pull request 2203,
  "Infer early diagnostic output type across nightlies",
  merged 2026-11-24,
  says it "keeps the driver and linting utility compatible with nightly toolchains that moved ErrorOutputType
  out of rustc_session::config",
  and `master` is 51 commits past `v6.4.0`
  (`probes/gh-dylint-releases-and-pull-request-2203.txt`).
  Nothing in that fix is released.

What each answer to question 1 returns on that compiler:

- Answer I,
  the components on the channel:
  the library builds with `c82ce1563` and fails,
  with `error[E0603]: enum ErrorOutputType is private` at `dylint_linting-6.4.0/src/lib.rs:794`
  (M12 arm B);
  the linter then returns one `core/processing-failure` per package and status 2,
  and the arm held one package (M12 arm A).
- Answer K1,
  a file naming the lock's toolchain:
  it builds with the dated compiler,
  which compiles 6.4.0,
  and returns the six findings on the copied package (M15a arms B and C).
  If the lock moves to a nightly at or past the compiler that moved `ErrorOutputType`,
  K1 fails exactly as I does:
  measured with a file naming `nightly-2026-11-28`,
  whose compiler is `c82ce1563`,
  where the warm command failed with the same `E0603`
  and the linter returned status 2 (M16 arms A and B).
- Answer K2,
  a file naming a hand-picked date:
  same mechanism as K1 on that date,
  and nothing installs that date (the section on the plan row and the cleaner).

That compiler is the fresh install's,
not this host's.
This host's floating channel carries `5df8c02af 2026-11-16` and no `rustc-dev`
(`machine/machine-state.txt`),
and Dylint 6.4.0 compiles on `5df8c02af` (M8).
What `rustup toolchain install --no-self-update nightly` followed by
`rustup component add rustc-dev llvm-tools-preview rust-src --toolchain nightly`
does to that installed channel is not measured,
and the two outcomes differ for the owner:
if the channel stays at `5df8c02af`,
Answer I returns the six findings on this host today;
if it moves to the channel's current `c82ce1563`,
Answer I returns status 2 here as M12 does.
M8 does not settle it,
because it copies a dated install that already holds the components over the floating name
(`scripts/m8-in-container.sh`).
One container arm would close it:
a floating channel frozen at `5df8c02af`,
the components added by those two installer commands,
then the linter.
It was not run,
because this revision makes no new measurement.

So,
in the measured state (driver built,
the variable exported),
only K1 and K2 return findings,
and both return them only while the compiler they name is one Dylint 6.4.0 builds on:
K1 follows the lock,
so its findings last until the lock moves past the break (M16),
and K2 keeps them only on a date someone installs and nobody moves.
The fix is a `cargo-dylint` release carrying pull request 2203,
or a locally built `cargo-dylint`:
the library could take `dylint_linting` from a Git revision,
but the driver is built by `cargo-dylint` from `dylint_driver = "=<its own version>"` on crates.io,
which the library cannot patch (source,
the driver build step).
That last sentence is inference from the two build steps,
not a run.

## What this round measured

Every container ran as rootless `podman`,
each inside its own
`systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`,
from the image `localhost/dylint-probe-2026-11-28c`
(`third-round-2026-11-28/Containerfile`:
today's floating `nightly` with the components Answer I installs,
the lock's `nightly-2026-11-16` with its components,
`nightly-2026-11-17` minimal,
and `cargo-dylint` and `dylint-link` 6.4.0).
Each container was removed at the end of its run;
the host script prints the image digest and the container names it used.
The linter binary was the released `monochromatic-lint` 0.5.2-alpha.25,
whose sha256 is `34ca8f26d618be2cbfce78bfd209aec0ae2d1c2faf7f00f4144374df37de485a`,
printed in the run logs of M12,
M13,
M14,
M15a and M17
(M15b runs no linter,
and M16 and M10 print no digest),
copied from the machine-global install;
the closure sources did not change between it and `HEAD`
(`git diff --stat 8638a53928..HEAD -- package/linter/monochromatic-lint/src` is empty).
The library and `package/wg/cli-quicker-exempt` were copied in with `podman cp`,
and no part of this account's home was mounted.

The two states every measurement starts from,
and which the questions depend on:

- `cargo` and `rustc` in `PATH` are the rustup proxies (`/usr/bin/cargo` and `/usr/bin/rustc`,
  links to `/usr/bin/rustup`),
  which is the state a fresh machine and a hosted runner have.
- No driver exists in `~/.dylint_drivers` before the first arm that has one,
  and the container's cargo registry holds only what an arm fetched.

### M12: Answer I on the channel's current compiler

State:
the floating channel is today's `nightly` with `rustc-dev`,
`llvm-tools` and `rust-src`
(`m12/log/floating-components.txt`),
the library has no toolchain file,
Dylint's computed name is the floating one (`m12/log/library-active-toolchain.txt`),
and no driver exists.
The three caches a warm run would fill were filled first,
with the network,
so that the linter's driver probe would resolve the crate and reach the library build
instead of skipping for a cache it cannot fill offline.

- Arm A,
  the linter with no exported variable,
  no driver,
  and the caches filled:
  status **2**,
  one record at `pkg/Cargo.toml`,
  `"code":"core/processing-failure"`,
  `"severity":"error"`,
  whose message begins
  "rust/avoidable-closure could not check this package:
  `cargo dylint` exited with exit status:
  1" and ends with the tail of the compiler's `E0603` message against `dylint_linting`,
  as the linter keeps the end of cargo's messages:
  the middle of the diagnostic,
  its `help` block and the `rustc --explain E0603` line,
  without the `error[E0603]` header line
  (`m12/log/a-linter.jsonl`;
  the header line and the `dylint_linting-6.4.0/src/lib.rs:794` path are in arm B's `b-warm.err`).
- Arm B,
  the warm command itself,
  to keep the first compiler error:
  status 1 after 0 s,
  `error[E0603]: enum ErrorOutputType is private` at
  `dylint_linting-6.4.0/src/lib.rs:794`,
  and no `~/.dylint_drivers` directory afterwards
  (`m12/log/b-warm.err`,
  `b-error-lines.txt`,
  `b-driver-tree.txt`).

What M12 settles:
on the compiler the channel carried that day,
Answer I returns no findings and status 2 for each package it is given,
and the run's message carries the end of the compiler's first error.
The variable does not reach that library build:
Dylint removes `RUSTUP_TOOLCHAIN` for its children (source,
"What `cargo dylint` itself does"),
and in this state the linter's own probe passes on the channel's components
whether the variable is unset or names the channel;
no arm of M12 sets it.
A `--max-warnings` gate is irrelevant there:
the status is already 2.

### M15a: Answer K1 alone, on the same image and the same day

State:
the floating channel was reinstalled with rustup's minimal profile
(`m15a/log/floating-components.txt`:
`cargo`,
`rust-std`,
`rustc` only),
the library holds a file with `channel = "nightly-2026-11-16"` and
`components = ["llvm-tools-preview", "rustc-dev"]`,
the dated toolchain carries the same components (`m15a/log/dated-components.txt`),
and no driver exists.

- Arm A,
  the linter with the variable unset and no driver:
  one skip per package,
  "the active compiler lacks the rustc-dev component",
  status `0`.
- Arm B,
  the warm command with the network:
  status `0` after 87 s,
  the library and the driver built for
  `nightly-2026-11-16-x86_64-unknown-linux-gnu`,
  3 `git` executions,
  241 executions of the dated `rustc` and none of the floating one,
  the driver's `-V` printing `nightly-2026-11-16-x86_64-unknown-linux-gnu 6.4.0`
  (`m15a/log/b-warm.err`,
  `b-rustc-counts.txt`,
  `b-driver-version.txt`),
  and one RUNPATH entry into the dated toolchain,
  which this arm's log does not hold and M13's does
  (`m13/log/b-driver-readelf.txt`).
- Arm C,
  the linter with `RUSTUP_TOOLCHAIN=nightly-2026-11-16` and the driver present:
  six findings at offsets 7146,
  1649,
  1962,
  2526,
  4978 and 2292 of the copied package,
  status `0` (`m15a/log/c-linter-set-after-driver.jsonl`),
  the same six positions M7 and M13 found.
- Arm D,
  the linter with the variable unset and the driver present:
  the rustc-dev skip again,
  status `0`.

What M15a settles:
with K1's file,
the run reaches the findings only when the variable is exported;
the driver,
the computed name and the library's compiler all agree on the dated toolchain,
and the linted package compiles with that compiler (241 dated `rustc` executions).
It reproduces M7's arms on the same image and day as M12,
so the two results stand side by side
under one instrument:
**K1 with the variable,
six findings;
I,
status 2.**

### M15b and M14: what rustup does with a file's components

rustup 1.29.3 (2026-11-11),
the version the image and this machine carry.

- A file naming an absent toolchain,
  with `rustup show active-toolchain`:
  rustup installed the toolchain and the file's components,
  warning that "the missing active toolchain ... has been auto-installed" and that
  "auto-installation is deprecated for most `rustup` commands" (M14 arm B,
  M11).
  The components it installed,
  itemised rather than counted:
  `cargo`,
  `clippy`,
  `llvm-tools`,
  `rust-docs`,
  `rust-std`,
  `rustc`,
  `rustc-dev`,
  `rustfmt` (`m14/log/b-installed-components.txt`):
  the default profile's six plus the two the file declares.
- A toolchain present without a declared component,
  with `rustup show active-toolchain`:
  rustup downloaded 2 components and added them
  (`m14/log/c-components-before.txt`,
  `c-components-after-show.txt`).
  A later `cargo --version` in that directory added nothing more.
- The supported step,
  `rustup toolchain install` with no argument in that directory:
  it synced the channel,
  downloaded the 2 components the file declares,
  and reported "the active toolchain ... has been installed" (M15b arm A).
  A bare `rustup show active-toolchain` afterwards prints the name (M15b arm B).
- With auto-install off (`RUSTUP_AUTO_INSTALL=0`) and a file naming an uninstalled toolchain,
  the linter printed no record and exited `0` (M14 arm A,
  and the section on absences).

What these settle:
K1's first use installs what it names,
in both cases rustup can meet,
and the supported step does the same when the deprecated auto-install is off.
The second-read finding that the component list "is inert" is wrong for this rustup,
and the reply that cited M11's count alone was thinner than the log:
the components are now itemised,
and the present-but-lacking case is measured.
What they do not settle:
whether rustup 1.30,
which the deprecation points at,
keeps any of this;
the supported step is the answer that does not depend on it.

### The earlier rounds' measurements, and what their logs hold

- M7 (option K measured alone),
  M8 (option I on this host's compiler),
  M10 (the three statuses a skip returns for both skip causes) and M11
  (what rustup does with a file naming an absent toolchain) are in their folders,
  each with `run.txt` and its logs.
  M11's folder is named `m11-rustup-does-not-install-an-absent-toolchain`,
  which is the opposite of what its log shows;
  M15b and M14 now carry that measurement with better names,
  and M11's log is still cited for the rustup version and the warning text.
- M10's `run.txt` prints no linter digest,
  and M11 keeps no `log/` folder;
  the digest is printed in this round's M12,
  M13,
  M14,
  M15a and M17 logs,
  and M10 ran the same released 0.5.2-alpha.25 whose sha256 begins `34ca8f26d6`,
  which its own log does not show.
- M3,
  the thirty selectable lock directories of 2026-11-27,
  recounted this round from the kept files:
  seven packages produced closure findings,
  six skipped,
  two failed closure with status 2,
  and fifteen produced none,
  so twenty-two of the thirty were checked;
  `closure-findings.txt` holds one line per package with its count,
  and the counts are 1674 in total,
  1502 in `package/git-policy/cli`,
  72 in `package/mpm-shadow/cli`,
  44 in `package/zfs/request-to-openzfs-kernel-device`,
  31 in `package/linter/monochromatic-lint`,
  13 in `package/cpu-temperature/cli`,
  and 6 each in `package/cargo-shadow/cli` and
  `package/wg/cli-quicker-exempt`;
  the four packages the issue names account for 1611;
  the thirty rows' wall times sum to 1688 s,
  with one linter invocation at a time and `CARGO_BUILD_JOBS=4`
  (`m3-f3-skips-with-a-driver-1/log/`,
  `m3-f3-skips-with-a-driver-2/log/`,
  recounted in `probes/m3-counts-recounted.txt`).
  What the folder does not hold:
  the cause of the two failures beyond the 400 characters each
  record keeps,
  the cause of the two cache skips beyond their records,
  and any `git fetch` failure text.
  `results.tsv` records `still-failed` for `package/ide/app` and `package/terminal/app`
  after the script's own networked `cargo fetch --locked`,
  so in that container a network fetch did not close the gap;
  whether one would on a machine whose `git` can reach those remotes is a hypothesis.
  Whether the 1502 are distinct positions was not checked by a command;
  the number is the count of finding records for that package.

## What the measurements imply

1. Three things decide whether the rule runs,
   and no answer may assume they agree:
   the name Dylint files a driver under (a `rustup show active-toolchain` answer in the library directory),
   the compiler that builds the library (the `cargo` in `PATH`),
   and the compiler the linter's own `rustc --print sysroot` probe reads.
   Measured:
   M15a's control shows the two names,
   M15a's arms A and D show the probe failing while the library's file names the dated toolchain,
   M12 shows the library failing on the channel's compiler,
   and M15a arm B shows the package built
   by the library's compiler.
2. Every answer needs a `cargo` that restores `RUSTUP_TOOLCHAIN` for Dylint's children.
   Without it the library's own link step fails before any driver is consulted,
   so no provider,
   task,
   asset or cache can deliver anything.
   `MONOCHROMATIC_LINT_CARGO` does not solve it:
   it replaces only the outer program and turns off
   the component check,
   while Dylint spawns `cargo` by name from `PATH`.
3. Every answer needs a compiler Dylint 6.4.0 can compile against,
   and that is a fact about the compiler,
   not about the file that names it:
   measured working,
   `nightly-2026-11-16` (`5f4effd57`) in M15a
   and `5df8c02af` in M8;
   measured failing,
   `c82ce1563` in M12 and M16 and,
   earlier,
   M2b and M5.
4. A driver must be filed under the name rustup reports in the library directory,
   and that name follows the toolchain file when there is one (M15a's control).
5. The clone is a cost of naming a dated toolchain:
   3 `git` executions and a longer driver build
   in M15a arm B;
   the floating name runs none (M8 arm B).
   The two figures cannot be compared as a timing:
   they are one run each,
   on different compilers,
   in different containers,
   with no spread measured.
6. A published driver carries an absolute RUNPATH into the builder's toolchain (M13,
   M4,
   M5),
   so it loads only where that path exists.
7. Eight of the thirty selectable packages are not checked with a driver present (M3):
   six skip and two fail.
   A status keyed on "any skip" is red for the six.
8. A run can check nothing and say nothing:
   with the driver's toolchain name unreadable,
   the packages run,
   produce no record,
   and exit 0 (M14 arm A).
   That is measured in the arm's state,
   a floating channel with `rustc-dev` and the variable unset;
   in K1's and K2's recommended state it is reachable by inference only
   (Decision 5's fifth part).
9. CI cannot start the linter at all today,
   so every CI answer waits on a workflow change
   that no provider answer includes.
10. A skip costs a gate nothing unless the gate is below the warning count it sets,
    and the seven packages of M3 with findings are already far over any plausible limit.
    So a skip made visible by a new status or a new severity matters where the rest is clean,
    and not as the only reason a run is red.

## Decision 1: which toolchain the lint library names

What the owner decides:
which compiler builds `package/linter/monochromatic-lint-closure` and under which name Dylint files its driver.
The three answers share one constraint:
the compiler must be one Dylint 6.4.0 can compile against (measured,
the section on what is true today).

Each answer is stated with:
how it works,
what it changes for good,
what it gives up,
an example with its visible result,
the cost,
and what a yes leaves undecided.

### Answer K1: a toolchain file generated from the lock

**How it works.**
A new managed file `package/linter/monochromatic-lint-closure/rust-toolchain.toml`
holds `channel = "nightly-2026-11-16"` (the `rust` entry's version in `toolchain.lock.json`)
and `components = ["llvm-tools-preview", "rustc-dev"]`,
written by a generator from the lock.
Every `rustup` command in that directory then answers the dated full name,
Dylint files the driver under that name,
and the library is built by that compiler.
No rule in `file-enforcer.config.jsonc` writes a toolchain file today,
so the generator is new work,
and who writes it is one of the things a yes leaves undecided.

**What it changes for good.**
The directory has one more managed file,
and the three names that must agree do agree by construction:
the file names the toolchain the plan installs,
the driver is filed under that name,
and the linter's probe reads that compiler when the variable is exported.
The library's compiler is the lock's nightly,
which is the toolchain the owner's 2026-10-18 answer
put `rustc-dev` on.
It also puts back what that record removed:
the record says
"The package therefore has no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override",
and the package README says the same,
so a yes to K1 reverses that arrangement for this package
and the README's sentence changes with it.

**What it gives up.**
The library's compiler moves only when the lock moves;
with the timer disabled,
nothing moves it until a person runs `toolchain:update`.
The file names the toolchain in a second place:
its content is the lock's,
but its presence is a policy the lock does not state.

**Example and its result.**
Input:
the copied `package/wg/cli-quicker-exempt`,
the library with
`channel = "nightly-2026-11-16"` and `components = ["llvm-tools-preview", "rustc-dev"]`,
`RUSTUP_TOOLCHAIN=nightly-2026-11-16`,
no driver.
Result:
the run prints six `rust/avoidable-closure` records at offsets 7146,
1649,
1962,
2526,
4978,
2292 and exits `0`;
the driver appears at
`~/.dylint_drivers/nightly-2026-11-16-x86_64-unknown-linux-gnu/dylint-driver`
after a first-run build of 87 s (M15a arm B),
and with the driver present the same run prints the six findings (arm C).
Without the variable the same run prints one skip per package and exits `0` (M15a arms A and D).

**Cost.**
A generated file and the generator,
a `rust-toolchain.toml` in the package's tree,
and first use's download of the dated toolchain with its components when it is absent
(measured,
M14 arm B and M15b arm A,
both on rustup 1.29.3;
the same install happens through the deprecated auto-install or through `rustup toolchain install`).

**What a yes leaves undecided.**
Which field of the lock the file is generated from and when it is rewritten;
whether the component list lives in the file or stays in the plan;
and whether the exported variable of Decision 2 is part of this answer.

### Answer K2: a toolchain file with a date pinned by hand

**How it works.**
The same file and the same mechanism,
with the channel a date a person writes,
and nothing regenerating it.

**What it changes for good.**
The library's compiler is frozen until someone edits the file,
and the lock can move without moving the library.
It is the same package-level file K1 puts back,
where the 2026-10-18 record says the package has
"no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override",
and it is the kind of pin that record lists as
"a pin of the nightly" under "Still not approved",
neither approved nor refused.

**What it gives up.**
The pin diverges from the lock the plan installs and from what the lean cleaner keeps:
`pins_in` reads `toolchain.lock.json`,
`.cache/toolchain.env.json`,
and a checkout-root
`rust-toolchain`/`rust-toolchain.toml`,
and not a package-level file.
A date the lock does not name is installed by nothing the repository runs,
so the first use depends on rustup's auto-install,
which its own warning calls deprecated
(measured:
a file naming a toolchain installs it,
M14 arm B and M15b arm A).

**Example and its result.**
Input:
as K1,
with `channel = "nightly-2026-11-16"` written by hand rather than generated.
Result:
the same six findings as K1 while that date is installed (same mechanism,
one measurement,
M15a).
With a date nothing has installed and auto-install off,
rustup cannot answer for the file;
in the state M14 arm A ran in (the components on the floating channel,
the variable unset) the linter prints no record and exits `0`.
In K2's recommended state the variable names the same absent toolchain,
and the linter fails at its version probe with one record at the configuration file
and status 2 (M17).
K2 was not measured on its own date,
so both statements are the mechanism read from the sources and the other arms.

**Cost.**
The same file and the same first-use download,
plus a date that someone must move when Dylint needs
a newer compiler.

**What a yes leaves undecided.**
Which date,
and who moves it.

### Answer I: the components on the rustup channel named `nightly`

**How it works.**
The plan row's `packageId` names the channel,
so the installer's own component step puts `rustc-dev`,
`llvm-tools-preview` and `rust-src` there.
The installer functions already act on whatever channel that field names:
`install_rust_toolchain` runs `rustup toolchain install --no-self-update <that channel>`,
`install_rust_components` runs `rustup component add <name> --toolchain <that channel>`,
and a test in the same package asserts exactly that for a `packageId` of `nightly`
(`rust_components_and_targets_use_the_package_id_channel_that_was_installed`).
Today's row does **not** do that:
its `packageId` is `{version}`,
so it installs `nightly-2026-11-16` and adds the components
to that toolchain (the section on the plan row).
What Answer I needs,
then,
is a plan row that names the channel,
a decision on what the lock and `env.RUSTUP_TOOLCHAIN` record for it
(the plan's `env` is `{version}`,
a dated name that would then be installed by nothing,
which is M17's shape),
and an owner for moving the channel,
which nothing in the repository does.
With the components present,
Dylint's name is the floating one,
the driver is filed under it,
and the linter's probe reads a compiler that has `rustc-dev`.

**What it changes for good.**
The machine carries about 500 MB more components,
on a channel that no repository task moves
(the dated toolchain with the same components is 799M and the floating channel 274M,
`machine/machine-state.txt`).
The installation gains a step whose target is a channel rather than a lock entry,
so the components and the compiler can move apart.

**What it gives up,
and what it returns on a compiler Dylint cannot use.**
The library is built by whatever the channel holds.
A nightly that moves rustc's internal API breaks the library,
and the channel the runs were made on is such a nightly:
with the components present and that compiler,
the run returned one `core/processing-failure`
per package and status 2,
and the arm held one package (measured,
M12 arm A).
That message names the package's `Cargo.toml` and carries the end of the compiler's first error;
the run is red and nothing is checked.
Every linted package must also compile on that compiler,
because Dylint builds the target
with the toolchain that built the library (the owner record,
"Known limits"),
so a package can pass its own build and fail the rule's run.
Nobody is told by the daily update,
when it runs:
that update checks each package's own `lint:clippy`,
reports a newly red package once,
and never reports "failed then failed" again
(`doc/planning/toolchain-always-latest.md`,
section "Checking every package after the update"),
and it does not run the rule over other packages at all.
Whoever runs the root lint task is the one who meets the breakage.
On a compiler Dylint cannot compile against,
I stays red until a `cargo-dylint` release carries
the fix:
the library could take `dylint_linting` from a Git revision,
but the driver is built by
`cargo-dylint` from `dylint_driver` on crates.io,
which the library cannot patch
(source;
the last sentence is inference).

**Example and its result.**
Input:
the copied package,
the library with no toolchain file,
the floating channel holding the compiler that day's fresh install resolved,
with `rustc-dev`,
`llvm-tools` and `rust-src`,
no exported variable,
no driver.
Result:
one record at `pkg/Cargo.toml`,
`"code":"core/processing-failure"`,
`"severity":"error"`,
whose message begins
"rust/avoidable-closure could not check this package:
`cargo dylint` exited with exit status:
1" and carries `error[E0603]: enum ErrorOutputType is private` against `dylint_linting`;
the run exits `2` and prints no findings (measured,
M12 arm A).
On a channel frozen at an older compiler the same state returns the six findings
(measured,
M8 arms B and C,
on `5df8c02af`),
which is the result the previous revision ranked first.

**Cost.**
A plan row that names the channel and its owner,
the decision on what the lock records for it,
an owner for the channel's daily move,
about 500 MB of components,
and a red root lint run for every Rust package
until a `cargo-dylint` release exists.

**What a yes leaves undecided.**
What the lock and `env.RUSTUP_TOOLCHAIN` record for the channel;
who moves the channel;
what to do on a nightly that breaks the library;
and whether the plan's row or a separate task installs the components.

### The ranking of Decision 1

Criterion:
findings the rule returns,
on a machine whose `cargo` and `rustc` are the rustup proxies,
with Decision 2's answer T exporting the variable,
on the compiler each answer names.

**What is measured**,
in one image,
one linter and one day (M12,
M15a,
M16):

- K1 with a file naming `nightly-2026-11-16`,
the variable exported and no driver:
the warm command built the driver and the rule returned the six findings
(M15a arms B and C).
- I with the components on a fresh `nightly` install and no exported variable:
one processing failure and status 2 (M12 arm A).
The variable does not reach the library build,
because Dylint removes it for its children (source,
"What `cargo dylint` itself does"),
and in this state the linter's probe passes on the channel's components with it unset or set;
no arm of M12 sets it.
- K1 with a file naming the compiler the channel carried that day and the variable exported:
the same `E0603` and status 2 (M16 arm B).

**What that supports**:
K1 over I,
and only while the lock's compiler is one Dylint 6.4.0 builds on.
Once the lock reaches the break,
K1 and I are equal:
both return status 2 for each package they are given,
and neither delivers before a `cargo-dylint` release carries upstream's fix.

**What is not measured**:
K2 on its own date,
and any answer on this host's floating channel with the components added.

**K1 > K2 > I**,
on the state measured.

- **K1 over K2.**
  Both are the same file and the same mechanism,
  and on the date it names K2 returns what K1 returns
  (inference from that identity;
  K2 was not run on its own configuration).
  The difference is what installs and records the date.
  K1's date is the lock's whenever the generator has run,
  so it is the toolchain the plan installs and the lock names;
  this machine has that toolchain installed (`machine/machine-state.txt`).
  K2's date is installed by nothing the repository runs,
  and under the ranking's own answer T an uninstalled date fails before any driver is looked for:
  one record at the configuration file and status 2 (M17).
  With the variable unset its first use depends on rustup's auto-install,
  which rustup's own warning calls deprecated (M14 arm B,
  M15b arm A).
  What the lean cleaner keeps is wider than the lock:
  it keeps a toolchain any checkout pins (the lock's `rust` version,
  `.cache/toolchain.env.json`'s `RUSTUP_TOOLCHAIN`,
  a checkout-root `rust-toolchain` file),
  one rustup names as default or an override,
  one a process runs or has in its environment,
  and one younger than the grace period
  (source,
  `lean_rustup_toolchains.rs`);
  a package-level file is not read.
  So neither answer is shielded from the cleaner once the lock has moved past its date:
  a K1 file the generator has not rewritten names a toolchain the cleaner may retire
  when the grace period ends,
  and K2's hand date becomes such a name at the next lock move.
  That removal is not measured;
  it is the cleaner's rule read against a file it does not read.
  K2 is also the pin the 2026-10-18 record lists as "still not approved".
- **K2 over I.**
  On a date Dylint 6.4.0 builds on,
  K2 returns the six findings by the mechanism M15a measured;
  on a date past the break it returns status 2 by the mechanism M16 measured.
  I returns status 2 on the compiler the channel carried the day M12 ran,
  and would return findings only on a channel frozen at an older compiler
  or after a `cargo-dylint` release.
  Both statements about K2 are inference:
  it was not run on its own date.
- **I is ranked last on today's delivery,
  not on a preference.**
  If a release carrying upstream's fix lands,
  I and K1 both deliver,
  and which to prefer becomes question 8's preference rather than delivery.
  What I still needs is a plan row that names the channel,
  a decision on what the lock records for it,
  and an owner for moving the channel;
  that channel moves only when a person runs rustup.
  Those are costs to state,
  not reasons to rank it lower.

What this ranking does not weigh:
which mechanism should represent "latest" for this library,
which is question 8.
The repository's way is a dated name the lock records,
so K1 follows it;
neither mechanism moves on its own today:
the lock moves when a person runs `toolchain:update` (the timer is disabled),
and the channel moves when a person runs rustup.
The difference is that a move of the lock is a commit a reader can find
and a move of the channel is recorded nowhere.
The owner may still prefer the channel for a reason this proposal cannot measure;
that is the preference question 8 asks,
not a result the ranking can produce.

## Decision 2: how the toolchain variable gets to Dylint's children

What the owner decides:
how `RUSTUP_TOOLCHAIN` reaches the library's link step and Dylint's driver build,
and how the linter's own `rustc --print sysroot` comes to read a compiler with `rustc-dev`.
The two needs are separate and are met together only by the first mechanism.

Measured:
a non-proxy `cargo` fails the link (M1,
M2b);
the proxy first satisfies the first need,
and the second need is decided by what `rustc` resolves to or by an exported variable
(M15a arms A and C,
M10 arm A,
M8 arm D).

### Mechanism T: the lint task's own environment puts the proxy first

**How it works.**
The `lint:monochromatic-lint` task runs the linter with `PATH` holding the rustup proxies before
the repository launchers,
and with `RUSTUP_TOOLCHAIN` naming the toolchain of Decision 1.
The variable reaches Dylint's children through the proxy,
and reaches the linter's own `rustc` probe because the probe inherits the environment.

**What it changes for good.**
A task's environment,
and nothing outside it:
a full-path call to the installed binary does not see it.

**What it gives up.**
The lint's build bypasses `cargo-shadow`,
so the repository's trust checks and lean containment
do not cover that build,
and a person or agent who calls the binary directly gets neither.

**Example and its result.**
Input:
the container state with the proxies first and `RUSTUP_TOOLCHAIN=nightly-2026-11-16`.
Result:
six findings and status `0` (measured,
M15a arm C);
the same run without the variable
prints one rustc-dev skip per package and status `0` (M15a arms A and D).
That a `mise` task can create this environment is read,
not run:
M15a and M7 set the variable and the `PATH` by hand in a container.

**Cost.**
Two lines in a task,
and the shadow bypass that already happens today for any build the shadow
does not start.

**What a yes leaves undecided.**
Whether `cargo-shadow` gets a rule for that tree,
and whether the lock's own exported
`RUSTUP_TOOLCHAIN` (a dated name) has to be overridden or is simply the same value.

### Mechanism N: no change; a person runs the lint with the proxy first

**How it works.**
Today's 2026-10-30 arrangement:
a shell whose `PATH` puts the proxies first and which exports the variable,
as that record's harness did.
**What it gives up.**
The issue's harm,
exactly:
it works when someone remembers.
**Example and its result.**
The same state as T,
set by hand:
six findings (M15a arm C is that state,
reached by hand).
**Cost.**
None beyond the memory step.
**What a yes leaves undecided.**
Nothing further;
it is today's answer.

### Design CL: the linter puts the proxy first for the `cargo dylint` child

**How it works.**
A change in the linter setting the child's `PATH` or its program,
so the proxy is what Dylint spawns,
with tests.
**What it changes for good.**
Every linter invocation,
including a full-path call,
gets the proxy.
**What it gives up.**
`cargo-shadow` is bypassed for that build by the linter itself,
and the second need is not met:
the linter's own `rustc` probe still reads the floating compiler,
so under K1 the run still skips without the components on the channel or an exported variable (M15a arm D).
**Example and its result.**
None:
not measured.
The nearest measurement is the state T reaches,
M15a arm C.
**Cost.**
A Rust change in the linter and its tests.
**What a yes leaves undecided.**
Whether the child's `PATH` or its program changes,
and what replaces the shadow's checks for that build.

### Design SH: the shadow restores the variable from the lock for every cargo it starts

**How it works.**
`cargo-shadow` reads `toolchain.lock.json` for the toolchain of every `cargo` it execs and exports
`RUSTUP_TOOLCHAIN`,
which today it does only for its own catalog calls
(`cargo_toolchain` and the variable it sets on the command in `src/cargo_run.rs`).
**What it changes for good.**
The repository's trust guard stays in the chain,
and every cargo command in the checkout gets the lock's toolchain.
**What it gives up.**
The shadow's documented promise that "the user's own cargo command is not changed",
`+toolchain` and `rust-toolchain` files for every cargo command in the checkout,
the toolchain of package builds unrelated to this lint,
and the second need,
which is still unmet (as for CL).
**Example and its result.**
None:
not measured.
**Cost.**
A Rust change in the shadow,
its tests,
and the behaviour change in "How it works".
**What a yes leaves undecided.**
Whether the lock's toolchain overrides an explicit `+toolchain`,
and what every other cargo command does differently afterwards.

### The ranking of Decision 2

Criterion:
the rule's runs that reach findings on this machine,
counting both needs,
and what each mechanism takes away from a program it does not own.

**T > CL = SH > N.**

- **T over CL.**
  T meets both needs for every run the task starts,
  and is measured (M15a arm C).
  CL meets the first need for more run shapes,
  but leaves the second unmet,
  so under K1 it still ends in the rustc-dev skip (M15a arm D) unless something else supplies the variable.
  That is fewer of the rule's runs reaching findings,
  and it is not measured.
- **CL and SH tie.**
  Both meet the first need,
  both leave the second unmet,
  and neither is measured.
  They differ in what they take away,
  which the repository's rules forbid using to rank one lower;
  the owner's choice between them is a choice about that cost,
  not about delivery.
- **CL and SH over N.**
  N reaches findings only when a person sets the state up by hand,
  which is the harm the issue records;
  both designs would do it for every run they cover.

## Decision 3: who provides the driver on this machine

Each answer assumes Decision 1 and Decision 2,
and each is measured with the same warm command
where a measurement exists.
The driver today does not exist on this machine (`~/.dylint_drivers` is absent,
the section on the machine's state).

### Answer B: the lint task warms the driver before the offline run

**How it works.**
The root `lint:monochromatic-lint` task gains a step,
after `fetch:rust-dependencies`,
that runs the linter's own warm command once per library directory,
without `CARGO_NET_OFFLINE`:

```sh
env --unset=CARGO_NET_OFFLINE cargo dylint --path <library> --manifest-path <Cargo.toml> -- --locked
```

`fetch:rust-dependencies` is the precedent:
it already fetches,
already needs the network,
and exists to stop a cold-cache skip.
**What it changes for good.**
A lint run writes `~/.dylint_drivers` and the cargo registry,
which `mise run lint` has never done,
and it needs the network and `git` (for a dated name) at least once per `cargo-dylint` move
or per lock move.
**What it gives up.**
A lint run that has no network fails or skips;
the choice is stated as undecided.
**Example and its result.**
Input:
the K1 state with the network and an empty driver directory.
Result:
the warm step builds the library and the driver in 87 s (3 `git` executions) and the lint run then
prints the six findings (measured,
M15a arms B and C).
Under Answer I on the channel's compiler of that day the same step fails with `E0603` and builds no driver (M12 arm B):
B cannot deliver what the compiler cannot build.
**Cost.**
Network and a cache write inside a lint run;
a first run that is slower by the build.
**What a yes leaves undecided.**
What a lint run does without the network,
which package warms the driver,
and who owns the staleness record.

### Answer Z: the documented manual warm

**How it works.**
The skip message names the warm command,
the linter's README tells a person to run it once with network access
and again after each `cargo-dylint` upgrade,
and the 2026-10-30 record shows it done:
`The owner authorized the recommended order ("I run sync + warm").`
**What it changes for good.**
Nothing:
it is today's answer.
**What it gives up.**
The issue's harm:
a person has to remember,
and the record's own history has three
releases in three days.
**Example and its result.**
The same warm command run by hand:
six findings afterwards (M15a arms B and C are
that state,
reached by hand).
**Cost.**
A person's step,
once per machine and per move.
**What a yes leaves undecided.**
Nothing further.

### Answer A: a plan row that builds the driver at install time

**How it works.**
A new `toolchain.plan.jsonc` row runs Dylint's build once for the toolchain the library directory reports
and places the binary at `<driver root>/<toolchain>/dylint-driver`,
recording the pair (cargo-dylint version,
toolchain) so either move invalidates it.
It needs a route that compiles,
which the plan has none of today:
its rows use the routes `archive`,
`bin-sourced`,
`cargo-binstall`,
`mpm-pnpm`,
`mpm-uvx`,
`android-sdk` and `rustup-toolchain`,
and every one of them installs a released artifact rather than
building one (`toolchain.plan.jsonc`).
It also needs the installer's `cargo` to restore the toolchain variable,
and the installer to run in the checkout.
**What it changes for good.**
A driver for every local run with no network call and no compile at lint time,
and no memory step.
**What it gives up.**
A compile inside the installer,
and a lock record for a compiled artifact that has no archive digest.
**Example and its result.**
None:
not measured,
and not buildable today for want of a route.
The warm command it would run is the one M15a arm B measured.
**Cost.**
A new route type in the installer,
a new plan row,
and the installer's own run.
**What a yes leaves undecided.**
The new route type,
which driver root it exports,
and whether a run outside `mise` reads that root.

### Option F is a complement to B, Z and A, not an alternative

F pre-fills the cargo caches so a first `cargo dylint` can build the driver offline:
it fetches the throwaway driver crate (`dylint_driver = "=<cargo-dylint version>"` with `anyhow`
and `env_logger`) for every platform in `fetch:rust-dependencies`.
The repository's own record weighs it:
its arm E measured that
"A warm registry alone makes the offline run build the driver and pass.";
the online arm C took 24.6 s for the driver build on the floating name
(`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`,
section "Verification").
Where F bears,
measured:
under Answer K1 the driver build clones `rust-clippy` (3 `git` executions,
M15a arm B),
so no cargo setting makes that build offline and F alone cannot deliver it;
under Answer I the build would need no clone on the floating name (M8 arm B),
but on that day's channel compiler the build fails before any cache matters (M12 arm B).
F is offered to whichever provider the owner picks,
and it is not ranked against them.

### The ranking of Decision 3

One cost belongs beside all three answers and none of them avoids it:
a warm build writes a target directory for every linted package under the library's toolchain,
and one such directory measured 2.09 GiB (`package/ide/app/target/dylint/debug`,
`doc/planning/lean-build-dirs.md`).
It is a cost of the rule's own machinery and not a difference between the answers.

Criterion:
whether a lint run reaches findings without anyone remembering a step,
with the parts that exist today.

**B > Z > A.**

- **B over Z.**
  Both hand the run the same measured driver (M15a arms B and C).
  B runs the command inside the lint task,
  so a run with nobody at the keyboard still reaches findings;
  Z reaches them only when a person ran the command,
  which is the harm the issue records.
  B's costs,
  the network and a cache write from a lint run,
  are stated and not used to rank it lower.
- **Z over A.**
  Z delivers today with parts that exist and is measured.
  A delivers nothing today:
  the plan has no route that compiles (line read),
  and the installer would have
  to run before it could.
  That is a statement about today's delivery,
  not about the reading of the
  2026-11-23 comment in issue 6316,
  which this revision does not treat as the owner's words.
  When a route exists and the installer runs,
  A's delivery is better than B's and Z's (no compile at lint time,
  no network),
  and the order becomes A > B > Z.

## Decision 4: how a pull request gets the rule

Both answers need a workflow change,
and neither was measured.
The job cannot start the linter today (the section on the tasks and the workflows).

### Answer C: a prepared image that the job runs

**How it works.**
A container image bakes the toolchain with the components,
`cargo-dylint`,
`dylint-link`,
a driver,
and the released linter,
and one job runs it over the changed files.
The image is not the shape of `package/linter/monochromatic-lint/test.Containerfile`,
which is a pinned stable image with `COPY vendor`,
`CARGO_NET_OFFLINE=true` and no nightly,
`rustc-dev` or Dylint;
a driver image needs the network at build and a nightly with the components.
**What it changes for good.**
The same finding in CI and on a person's machine,
because the image is the program,
without touching the job's tool list.
**What it gives up.**
An image to rebuild on every nightly or `cargo-dylint` move,
and a job that pulls it.
**Example and its result.**
None measured.
The traps a fresh image meets are the components (M2a)
and a writable temporary directory for the clone:
M1's refusal was an existing directory,
and later containers set `TMPDIR` (design,
not a measured failing case).
**Cost.**
An image pipeline,
a pull on every run,
and the image's own size,
which must carry a driver
and the nightly's components.
**What a yes leaves undecided.**
How the checkout enters the image,
whether the run has the network,
and where the image lives.

### Answer H: the existing job provisions the tools itself

**How it works.**
The job installs the lock's nightly with `rustc-dev`,
`llvm-tools-preview` and `rust-src`,
installs `cargo-dylint` and `dylint-link` at the lock's versions,
puts the proxy `cargo` first,
opens the closed tool list for `monochromatic-lint`,
restores and saves the caches Dylint recommends,
and runs the linter over the changed Rust files.
The caching guidance is Dylint's own README at `v6.4.0`:
"If you run Dylint in CI,
we recommend caching certain directories to avoid rebuilding libraries,
rebuilding drivers,
or redownloading toolchains."
It lists `~/.cargo/git/checkouts/`,
`~/.dylint_drivers/`,
`~/.rustup/toolchains/`,
`~/.cargo/bin/`,
the registry directories and `target/dylint/libraries/`.
**What it changes for good.**
Pull-request findings without an image pipeline,
on the surface the issue says never ran at all.
**What it gives up.**
Four separate things on the job must be right at once (the tool list,
the components,
the proxy,
the caches),
none of it serves a local edit,
and the lock's nightly can move under it.
**Example and its result.**
None measured.
**Cost.**
Four workflow steps,
a cache key,
the disk a cached driver and target directories take
(2.09 GiB for one warm target directory),
and the job's wall time,
which was not measured.
**What a yes leaves undecided.**
Which of the four steps change,
how the cache is keyed,
and what the job costs.

### The ranking of Decision 4

Criterion:
how much of the job's own state has to be right for the finding to appear.

**C > H > neither.**

- **C over H.**
  C's result is one artifact the job runs,
  so the runner's `PATH`,
  its toolchain and its cache
  do not decide it.
  H's result is the sum of four steps that must all be right,
  and today none of them is.
  C's costs,
  the pipeline and a rebuild per move,
  are stated and not used to rank it lower.
- **H over neither.**
  H at least delivers the rule on the pull-request surface it is aimed at;
  neither delivers nothing.
  None of the three is measured,
  so this ranking rests on the mechanism and not on a run.

## Decision 5: what a skip returns

The record's contract is the baseline:
a skip is a warning finding of `rust/avoidable-closure` at the package's `Cargo.toml`,
counted by `--max-warnings` and hidden by `--quiet`
(`doc/decision/rust-closure-lint-dylint.md`,
section "Amendment of 2026-11-07").
The owner's words are "Fetch then warn."
Everything else in that block is the record's reading,
and the quiet-form consequence was never put to the owner.

Four parts are choices,
each answerable alone and combinable.
Part 5 is not a choice about today's behaviour:
it is a defect,
held by issue 6667,
that the work fixes without asking,
and the only thing left for the owner there is the shape of its report.
Parts 1 to 4 change the record's contract;
part 5 does not.

### Part 1: severity

- **Keep `warn`** (the record's contract).
  How it works:
  unchanged.
  Result:
  measured,
  the statuses in M10.
  Cost:
  none.
- **Option J:
  a per-cause severity** (a rule option beside `library`,
  so a configuration can make a
  driver-gap skip an `error` and leave a prerequisite skip at `warn`).
  How it works:
  `DriverState::Missing` is already told apart from a package's other skips in `driver_gaps`,
  so the cause is known where the finding is built.
  What it changes for good:
  a configuration surface,
  and the record's fixed severity.
  What it gives up:
  the record's rule that a skip is always a warning.
  Result:
  none measured;
  the closest measurement is that `--max-warnings 0` already fails every skip (M10).
  What a yes leaves undecided:
  the option's name,
  and which causes it covers.

**Ranking:
J > keep `warn`.**
Criterion:
what a gate can distinguish.
J lets a gate fail on a driver-gap skip and pass a prerequisite skip.
Keeping `warn` leaves one severity for every cause,
so the only gate that acts on a skip is one set below
`--max-warnings`,
and it fails the six prerequisite skips of M3 together with the driver gap.

### Part 2: a marker field

- **None** (the record's shape).
  Result:
  a skip already names a `Cargo.toml` and a finding a `.rs` file,
  so a consumer that reads file names
  can tell them apart (source and the measured records of M10 and M15a).
- **Option D2:
  a field such as `"checked": false`.**
  How it works:
  one more key on the skip's record;
  the code,
  the severity and the exit status do not change.
  What it changes for good:
  a consumer that reads records and not paths can exclude a skip by a field.
  What it gives up:
  a schema addition every consumer may read.
  Result:
  none measured.
  What a yes leaves undecided:
  the field's name.

The code variant (`rust/avoidable-closure-skipped`) is not offered:
it conflicts with the record,
with the comment that keeps findings searchable by the rule name,
and with the linter's README.

**Ranking:
D2 > none.**
Criterion:
what a consumer that reads records and not paths can tell.
The field delivers a distinction that consumer cannot get today;
`none` keeps the file-name convention
as the only way.

### Part 3: an exit status

- **None** (the record's contract):
  a skip leaves status `0` unless `--max-warnings` is set (M10 arm A).
- **Option D1,
  keyed by cause**:
  a status of its own for a run that omitted the rule because its driver
  was missing,
  leaving prerequisite skips at `0`.
  How it works:
  `exit_status` counts a finding's kind;
  the driver gap is already a distinct kind.
  What it gives up:
  a new public status every consumer must learn.
  Result:
  none measured.
- **A status keyed on any skip**:
  red for the six prerequisite and cache packages M3 names.
  Result:
  measured for the gate form,
  `--max-warnings 0` exits `1` on any skip (M10 arms A and B).

**Ranking:
by cause > any skip > none.**
Criterion:
the reds a gate gets that are true.
By cause is red exactly when the rule was omitted for the reason the owner names;
any skip is also red for six packages whose own prerequisites are missing,
which is a true skip but not
the harm the issue is about;
none leaves the silent pass the issue records.

### Part 4: a closing count

- **None.**
- **Option D3:
  one closing line on standard error,**
  "N package(s) not checked by rust/avoidable-closure".
  How it works:
  a line printed after the findings;
  not on standard output,
  which is JSONL.
  What it changes for good:
  a person reading a run sees the count without parsing records.
  What it gives up:
  a second channel for a fact the records already carry.
  Result:
  none measured.
  `--quiet` drops warnings from the record stream only,
  so the line would survive it
  (source,
  `src/run_output.rs`).
  What a yes leaves undecided:
  the line's wording.

**Ranking:
D3 > none.**
Criterion:
what a person sees without parsing JSONL.
The line delivers the count to a reader of the run;
`none` leaves it to the records.

### Part 5: a run that checked nothing and printed nothing

This is a defect,
not a behaviour to choose between.
With the driver's toolchain name unreadable,
the linter returns `Unknown`,
notes the reason only under `--debug`,
runs the packages,
prints no record and exits `0` (M14 arm A).
Issue 6667 holds it,
and a round on that issue is finding the cause and building the fix
(`monochromatic-lint-a-package-the-closure-rule-did-not-check-never-ends-as-clean`).
"Leave it" is not offered:
the revision that found it calls it a defect,
and the record's own standard for a run that could not check a package
is that it "is not a silent message with exit status 0".

That it checked nothing is inferred,
not logged:
the arm kept no output or status from `cargo dylint`,
and a `cargo dylint` that failed would have added a processing failure (`check_package`),
so it exited 0 and printed nothing;
the positive control is M15a arm C,
where the same package returns six findings with the right driver and compiler.

The state the arm ran in:
the image's floating channel carries `rustc-dev`,
`llvm-tools` and `rust-src` (the `Containerfile`),
and `RUSTUP_TOOLCHAIN` was unset.
That is Answer I's compiler state,
not K1's or K2's recommended state:
there the floating channel lacks `rustc-dev`,
so with the variable unset the linter skips visibly at its machine-wide check
(M15a arms A and D),
and with T's variable naming an absent toolchain it fails at the version probe
(one record at the configuration file,
status 2,
M17).
A route to the silent pass under K1 with T is inference:
the variable names a toolchain that is present,
the library's file names one the cleaner has removed,
and the linter's driver probe removes the variable before asking rustup
(`closure_driver_probe.rs`),
reads the file's name,
fails,
and returns `Unknown`.
No kept run has that combination.

The one thing left to decide is the shape.

- **A processing failure** (recommended):
  a `core/processing-failure` at the package's `Cargo.toml`,
  status `2`,
  naming the reason the linter could not read the toolchain name.
  How it works:
  the `Unknown` branch builds a finding instead of a note.
  What it changes for good:
  a run that looks clean and checked nothing becomes red,
  and its status stops a gate.
  What it gives up:
  the record's rule that a missing prerequisite is not an error,
  for this cause;
  a run whose toolchain name cannot be read now fails rather than proceeding.
  Result:
  the shape is M14 arm A's silent run;
  the fix is not measured.
  What a yes leaves undecided:
  which of the `Unknown` branch's other causes keep status `2`,
  and whether the record's amendment is rewritten to name this cause.
- **A warning-level skip:**
  a `warn` finding of `rust/avoidable-closure` at the package's `Cargo.toml`
  naming the package and `rustup toolchain install <the name>`,
  status `0` unless `--max-warnings` is set,
  and hidden by `--quiet`.
  How it works:
  the same skip machinery every other missing prerequisite uses.
  What it changes for good:
  the silent pass becomes a record,
  and the record's contract stands unchanged.
  What it gives up:
  the run still exits `0`,
  and `--quiet` hides the record exactly as it hides the other skips.
  Result:
  none for this cause;
  the shape is the one M10 measured for both skip causes.
  What a yes leaves undecided:
  the wording of the command the skip names,
  and whether the record's list of skip causes is widened to include this one.

**Ranking:
a processing failure > a warning-level skip.**
Criterion:
whether a gate can tell a run that checked nothing from a clean one.
A processing failure gives status `2` and a record at the package;
a warning skip gives a record and status `0` unless the gate sets `--max-warnings`,
which it can only do at the price of failing every other missing prerequisite too (M10,
M3).

Which the record points to:
its amendment of 2026-11-07 gives a warning-level skip to a missing prerequisite,
and keeps status `2` for "Failures that are not a missing prerequisite".
A toolchain the library names and the machine lacks is a prerequisite,
with the fix rustup itself prints,
so the record points to the warning-level skip for this cause,
and the processing failure is a departure the owner's yes would decide.
The record's own list of skips (a machine-wide gap,
a missing driver) does not name this cause;
the `Unknown` branch also covers causes that are not missing prerequisites at all,
such as an unreadable rustup answer or an unreadable `cargo dylint --version`,
and those are failures the record keeps at status `2`.
A round on issue 6667 builds the fix.

### How the parts combine

Any combination of the four choices is buildable,
and each can be answered alone.
Part 5's fix is built whatever the four are,
and the shape the owner picks combines with any of them.
One pair conflicts in intent:
a severity keyed by cause together with a status keyed on any skip,
because the first leaves prerequisite skips at `warn` and the second makes them red.
"Both,
in this order" is reachable:
part 5's fix first,
then the four choices on top of it,
or the other way round.

## Decision 6: whether the two Dylint tools stay at `@latest`

The plan's rows are `cargo:cargo-dylint@latest` and `cargo:dylint-link@latest`,
both through `cargo-binstall` (`toolchain.plan.jsonc`).

- **Keep `@latest`** (recommended).
  How it works:
  unchanged.
  `toolchain:update` moves the two rows when it runs,
  and today it runs only when a person does (the timer is disabled);
  the driver is rebuilt when `cargo-dylint`'s
  version moves,
  because Dylint rebuilds a driver whose `-V` reports a version below `cargo-dylint`'s
  (source,
  `dylint/src/driver_builder/mod.rs`).
  What it gives up:
  a release that breaks the driver arrives without a person's decision.
  Result:
  none measured;
  the staleness rule is source.
- **Option L:
  an exact request for both tools.**
  How it works:
  an exact version in the two rows.
  What it changes for good:
  a bump becomes deliberate,
  so whichever provider is picked rebuilds only when
  the owner moves the pin.
  What it gives up:
  the owner's "update EVERYTHING (not just rust) daily" for these two tools,
  and no driver comes with the pin.
  Result:
  measured only in its absence:
  the driver is rebuilt when `cargo-dylint` moves,
  and the last three releases came within three days (the pin record).
  What a yes leaves undecided:
  which version to hold now.

**Ranking:
keep `@latest` > L.**
Criterion:
how soon the rule picks up an upstream fix.
Today's compiler needs upstream's merged fix (the section on what is true today),
and `@latest` picks the fix up on the next `toolchain:update`,
with no edit of the plan or the lock;
a pin needs an edit of both,
and holds the compiler-breaking release until the owner makes it.
Nothing moves either row by itself today:
the daily update timer is `disabled; inactive (dead)`,
so the next move is a person running `toolchain:update`.
The cost of `@latest`,
an unwanted move,
is stated and not used to rank it lower.
The owner may still prefer the pin,
and that is a preference this proposal asks for,
not a result.

## Decision 7: where the counts live, and whether the host run repeats

- **Beside the 2026-11-07 count in `doc/planning/repository-linter-stays-at-zero.md`** (recommended).
  How it works:
  a new subsection with the seven packages' counts and the state M3 ran in.
  What it changes for good:
  the two counts of the same quantity sit together and can be compared.
  What it gives up:
  nothing;
  the file's own purpose is that count.
  Result:
  measured counts (M3,
  recounted).
- **A record of their own.**
  How it works:
  a new file under `doc/planning/`.
  What it gives up:
  the comparison with the 2026-11-07 count,
  which a reader would have to make across files.

**Ranking:
the existing record > its own record.**
Criterion:
whether a reader can compare the two counts of the same quantity.
A repository-wide count is not a package fact,
so `doc/planning/` is where it belongs either way;
the existing file already carries the rule's earlier count.

The second half of the question is not a ranking:
whether the run is repeated on this machine.
A yes leaves undecided when,
and by whom.

## Decision 8: which name `latest` should mean for the library

What the owner decides:
whether the library's compiler should follow the lock's dated name
or the name of the rustup channel `nightly`,
as the repository's way of keeping the compiler latest.
Whether each delivers findings is Decision 1's question;
this one asks which mechanism the owner wants preferred when both would work,
and it starts no work by itself.

### The lock's dated name

**How it works.**
A file in the package names the lock's `rust` entry and is regenerated from the lock
(K1's file),
so every rustup command in that directory answers the dated full name.
The name moves when `toolchain:update` re-resolves the lock and the file is rewritten.

**What it changes for good.**
The library's compiler is the toolchain the plan installs,
and every move is a commit in `toolchain.lock.json` that a reader can find.
It also puts a package-level `rust-toolchain.toml` back,
where the 2026-10-18 record says the package has
"no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override".

**What it gives up.**
The compiler is as new as the last lock move,
and today nothing moves the lock on a schedule:
the timer stands `disabled; inactive (dead)`,
so the lock moves only when a person runs `mise run toolchain:update`.
The record's status line says the timer was enabled on 2026-11-07;
why it is off is not known.

**Example and its result.**
Input:
the lock as it stands,
`nightly-2026-11-16`,
and a file generated from it.
Result:
the library builds with `5f4effd57`
and the rule returns the six findings (measured,
M15a);
the next `toolchain:update` resolves the channel's `c82ce1563`,
and the same file then returns status 2 (measured for that compiler,
M16).

**Cost.**
A generated file and its generator,
and the person or task that runs the update.

**What a yes leaves undecided.**
Who runs `toolchain:update`,
and whether the timer is re-enabled.

### The rustup channel named `nightly`

**How it works.**
The components go on the channel (Answer I's installation),
and the library's compiler is whatever build the channel holds when a command runs.

**What it changes for good.**
The library follows the channel a person's rustup moves,
and no lock field names the compiler.

**What it gives up.**
Nothing in the repository moves that channel or records its move:
a new nightly arrives with no commit,
and a break arrives the same way.
Adding the components may itself move the channel,
which is unmeasured (question 1).

**Example and its result.**
Input:
the channel as a fresh install resolves it,
`c82ce1563`,
with the components.
Result:
the library fails with `E0603` and the run returns status 2 (measured,
M12 arm A).
On a channel frozen at `5df8c02af` the same state returns the six findings (measured,
M8 arms B and C).

**Cost.**
A plan row for the channel's components,
the decision on what the lock records,
an owner for moving the channel,
about 500 MB of components,
and a compiler no commit records.

**What a yes leaves undecided.**
Who runs rustup for the channel,
and when.

### The ranking of Decision 8

Criterion:
whether a move of the compiler is recorded and repeatable,
which decides what a reader can check after a break.

**The lock's dated name > the channel.**

- A move of the lock is a commit in `toolchain.lock.json` a reader can find,
  and the plan installs exactly the toolchain the file names (line read of the plan row and the lock).
  A move of the channel is a rustup command on one machine,
  recorded nowhere in the repository,
  and nothing in the repository runs it.
- The lock's mechanism names the same compiler on every machine that runs `toolchain:install`;
  the channel's gives each machine whatever build its own rustup last fetched.
- The lock's cost,
  a compiler as new as the last update,
  and the channel's,
  a compiler nobody records,
  are both stated,
  and neither is used to rank an option lower.

The owner's recorded words want the compiler always latest.
Both mechanisms can serve that wish;
the difference this ranking turns on is who moves the compiler and whether the move is recorded.

## How the decisions depend on each other

- **Decision 1's ranking assumes Decision 2's answer T.**
  Without the variable,
  K1 and K2 skip (M15a arms A and D),
  and I fails on the channel's compiler of that day for its own reason
  (M12).
  The two decisions are therefore not independent as ranked,
  which is why questions 1 and 2 are answered together;
  the assumption is stated here rather
  than inside the ranking.
- **Decision 8 is the preference behind Decision 1.**
  It decides which of question 1's answers to prefer when both would work,
  and it changes no delivery.
  An owner may prefer the channel and still take K1 while I returns status 2,
  or prefer the lock's name and still take I once a `cargo-dylint` release lands.
- **Under Answer I,
  T's value is the floating channel's name.**
  The lock records `RUSTUP_TOOLCHAIN` as the dated name in the `rust` entry's environment.
  Whether the lint task receives that value was not measured.
  If it does,
  T must override it rather than set it,
  and that is one more line in the task.
- **Decision 3 and Decision 4 deliver only under an answer to Decision 1 whose compiler builds Dylint.**
  Today that is K1 or K2;
  under I the warm command fails before a driver exists (M12 arm B).
  Decision 3 is answered after 1 and 2 for that reason;
  Decision 4 can wait,
  because its own answers are unmeasured and the job cannot start the linter today.
- **Decision 6 interacts with Decision 3:**
  a move of `cargo-dylint` invalidates the driver,
  so a pinned tool makes a rebuild deliberate and `@latest` makes it automatic.
- **Decision 5 is independent of all of them:**
  it changes what an absent rule says,
  not whether it runs,
  and its fifth part is a defect the work fixes without asking.
- **Decision 7 is independent:**
  it changes where a count lives.

## Combinations, and what cannot be built

Buildable,
with the extra work named:

- **Answer I with Answer C or Answer H.**
  Both CI answers install the lock's nightly with its components and file a driver for that name.
  Under I they would have to install the channel plus components and file the driver under the channel's name,
  which is a different set of steps,
  not the ones described,
  and neither would follow the plan row.
- **Answer A with Answer I.**
  A files a driver for "the toolchain the library directory reports",
  which under I is the channel's name.
  The asset then goes stale at the next nightly move,
  and nothing in the repository moves or rebuilds it.
- **Design CL or Design SH with Answer K1.**
  Both meet the first need and leave the second unmet,
  so the run still skips (M15a arm D).
  They become sufficient only with the components on the floating channel (Answer I)
  or with an exported variable (Mechanism T).
- **Option F with any provider.**
  F fills caches;
  it provides no driver and no compiler.
  Under K1 the driver build clones `rust-clippy`,
  so F alone cannot make the first run offline;
  under I today the build fails before the caches matter.
- **The channel as question 8's preference with Answer K1 or K2.**
  A preference changes no delivery:
  an owner who prefers the channel can take K1 while I returns status 2,
  and move to I once a `cargo-dylint` release lands.
  The other way round is buildable too:
  an owner who prefers the lock's name can take I while it works,
  and the preference says what to return to.

Not buildable as stated:

- **Answer K1 or K2 with option F and no network at all.**
  The driver build for a dated name clones `rust-clippy` (3 `git` executions,
  M15a arm B),
  so the first run needs `git` and the network whatever the cargo caches hold.
- **A severity keyed by cause with a status keyed on any skip.**
  The first leaves prerequisite skips at `warn`;
  the second makes the same skips red.
- **`MONOCHROMATIC_LINT_CARGO` with the component check.**
  The linter skips that check while the variable is set,
  so taking the replacement cargo
  gives up the check rather than satisfying it.

Two combinations whose result is not known:

- **Mechanism T with Answer I when the lint task inherits the lock's exported `RUSTUP_TOOLCHAIN`.**
  The task would have to override a dated value with the channel's;
  not measured.
- **Answer A with the lean cleaner.**
  The cleaner keeps a toolchain a checkout pins,
  one rustup names as default or an override,
  one a process runs or has in its environment,
  and one younger than the grace period;
  an asset built for a name outside all of those may be deleted or left behind,
  and the cleaner's own check is a process scan,
  not a driver read;
  not measured.

## The questions for the owner

Each question is one the owner alone can answer,
and says what a yes leaves undecided.
Which to answer first is stated in "Which questions come first".

1. **Which compiler should build the closure lint's library,
   and so which name should its driver be filed under?**
   The lock's dated nightly,
   by a file generated from the lock (K1);
   a date written by hand in the same file (K2);
   or the rustup channel named `nightly`,
   with the components installed there (I).
   Measured in the state with the driver built and the variable exported:
   K1 returns the six findings while the lock names `nightly-2026-11-16`,
   I returns status 2 on a fresh install of the channel (M12 against M15a),
   and K1 returns status 2 once its file names the channel's compiler (M16).
   On this host the channel is older than the fresh install's,
   and what adding the components does to it is not measured
   (the section "What is true today about the compiler and Dylint").
   The owner's recorded wish that the compiler always be latest is a separate question (question 8):
   the lock moves when a person runs `toolchain:update`,
   and the channel moves when a person runs rustup.
   A yes to K1 leaves undecided who writes the file,
   which lock field it is written from,
   when it is rewritten,
   and whether the README's sentence about the package having no `rust-toolchain.toml` changes with it.
   A yes to K2 leaves undecided which date,
   who moves it,
   and whether the record's refusal of a pin is lifted.
   A yes to I leaves undecided what the lock and `env.RUSTUP_TOOLCHAIN` record for the channel,
   who moves the channel,
   and what to do on a nightly that breaks the library.
2. **How should the toolchain variable reach the lint's `cargo` tree?**
   The lint task's own environment,
   with the proxies first (T);
   no change,
   with a person doing it (N);
   the linter setting its child's `PATH` or program (CL);
   or the shadow exporting the lock's toolchain for every cargo it starts (SH).
   Only the first meets both of the rule's needs in a measurement (M15a arm C,
   and its value is question 1's name);
   the two designs meet the first and leave the second unmet under K1 (M15a arm D).
   That a `mise` task can create that environment is read and not run.
   A yes to T leaves undecided whether the shadow gets a rule for that tree.
   A yes to either design leaves undecided what replaces the shadow's checks for that build,
   and how the linter's own compiler probe comes to read a compiler with `rustc-dev`.
3. **Who should build the driver on a person's machine?**
   A step in the root lint task that runs the warm command (B);
   the documented manual warm (Z);
   or a plan row that builds it at install time (A).
   Only B removes the memory step with parts that exist today;
   A has no route in the plan that compiles today.
   A yes to B leaves undecided what a lint run does without the network,
   and which package warms the driver.
   A yes to Z leaves undecided nothing further.
   A yes to A leaves undecided the new route type,
   the driver root,
   and whether a run outside `mise` reads it.
   F,
   pre-filling the caches,
   can be added to any of the three and is not one of them.
4. **Should a pull request get the rule?**
   A prepared image the job runs (C);
   the job provisioning the tools itself (H);
   or neither.
   Neither answer was measured;
   today the job cannot start the linter at all.
   A yes to C leaves undecided how the checkout enters the image,
   whether that run has the network,
   and where the image lives.
   A yes to H leaves undecided which of its four steps change,
   how the cache is keyed,
   and what the job costs in wall time.
5. **What should a run say when the rule did not check a package,
   and what should it exit with?**
   Four parts,
   each answerable alone and combinable:
   the severity (a severity per cause,
   J,
   over keeping `warn`);
   a marker field (`"checked": false`,
   D2,
   over none);
   an exit status (one keyed by cause,
   D1,
   over one keyed on any skip or over none);
   and a closing count on standard error (D3,
   over none).
   A fifth is not a choice:
   a run whose toolchain name cannot be read prints no record and exits `0`,
   which is a defect (issue 6667),
   and the work fixes it without asking;
   the owner is asked only whether a toolchain the library names and the machine lacks
   reports as a warning-level skip (the record's shape for a missing prerequisite)
   or as a processing failure,
   which this proposal recommends.
   The record points to the warning skip for that cause,
   and the processing failure is a departure from it.
   Any combination of the four choices is buildable except a severity per cause with a status keyed on any skip.
   A yes to J leaves undecided the option's name and which causes it covers.
   A yes to the field leaves undecided the field's name.
   A yes to a status leaves undecided the number.
   A yes to a count leaves undecided the line's wording.
   A yes to either shape leaves undecided which of the `Unknown` branch's other causes
   keep status `2`.
6. **Should `cargo-dylint` and `dylint-link` stay at `@latest`,
   or be pinned to exact versions (L)?**
   Keeping `@latest` follows the owner's "update EVERYTHING (not just rust) daily"
   and picks up upstream's fix on the next `toolchain:update` a person runs,
   with no edit of the plan or the lock;
   a pin makes a bump deliberate
   and takes the two tools out of that update.
   Today the timer is disabled,
   so neither row moves without a person either way.
   A yes to L leaves undecided which version to hold now.
7. **Where should the counts of the rule's repository-wide run live,
   and should the run be repeated on this machine?**
   Beside the 2026-11-07 count in `doc/planning/repository-linter-stays-at-zero.md`,
   or in a record of their own.
   A yes leaves undecided whether the host run the issue asked for is repeated,
   and when.
8. **Which name should `latest` mean for the library's compiler:**
   **the lock's dated name,
   or the name of the rustup channel `nightly`?**
   The lock's name is written by a generated file and moves when `toolchain:update` runs,
   which today is only when a person runs it (the timer is disabled);
   the channel moves only when a person runs rustup,
   and nothing in the repository moves it or records a move.
   The record's words want the compiler always latest;
   the record's 2026-10-18 change put `rustc-dev` on the lock's dated name.
   A yes to the lock's name leaves undecided who runs the update,
   and whether the timer is re-enabled.
   A yes to the channel leaves undecided who moves the channel,
   and when.

## Measured, inferred, and not known

Measured,
with the log beside it:

- the machine's toolchains,
  components,
  versions and `PATH` order (`machine/machine-state.txt`);
- today's channel compiler and its commit (`probes/todays-nightly-channel-manifest.txt`);
- Dylint's releases,
  pull request 2203 and how far `master` is past 6.4.0
  (`probes/gh-dylint-releases-and-pull-request-2203.txt`);
- Answer I on the compiler a fresh `nightly` install resolved that day:
  one processing failure,
  status 2 (M12);
- Answer K1 on the same image and day:
  a skip without the variable,
  six findings with it (M15a);
- K1 with the lock moved to a compiler Dylint cannot build on:
  the same `E0603` and status 2 (M16);
- rustup's behaviour with a file's components,
  itemised,
  for an absent toolchain and for a present
  toolchain missing a component,
  and through the supported step (M14 arms B and C,
  M15b);
- the silent pass when the toolchain name cannot be read (M14 arm A);
- a `cargo` whose version probe fails:
  one processing failure at the configuration file,
  status 2 (M17);
- the statuses a skip returns for both skip causes (M10);
- the library and driver build times and the clone counts for a dated name (M15a) and a floating name (M8);
- the driver's `-V`,
  its RUNPATH and its `NEEDED` library (M13,
  M8,
  M4);
- the thirty selectable packages' outcomes and counts (M3,
  recounted from its kept files);
- the workflow's state (the workflow itself and the 2026-11-01 run excerpt).

Inferred,
and labelled where it appears:

- the shadow's hand-off (read in its README and source,
  not run);
- that M1's child environment is the host's,
  which M1 reached through the non-proxy binary;
- that the 2026-11-07 count was taken in the state the records describe;
- that the ignored host test would fail today (it was not run);
- that on a compiler Dylint cannot build,
  Answer I stays red until a `cargo-dylint` release,
  because the library can take `dylint_linting` from a Git revision but the driver is built by
  `cargo-dylint` from crates.io;
- that Mechanism T can be created by a `mise` task (the state is measured,
  the task is not run);
- that option F makes a first run offline-sufficient under Answer I (no kept run is an offline driver build
  under I with a pre-filled registry);
- that Answer C's temporary-directory trap fails a build (no kept run shows it failing);
- that K1's first use installs its components on rustup 1.30 (measured on 1.29.3 only);
- that K1 delivers over K2,
  from the same mechanism on two dates and from what installs each date
  (K2 was not run on its own configuration);
- that the lean cleaner retires a toolchain a stale K1 file names,
  from its rule read against a file it does not read (no run shows the removal);
- that the silent pass is reachable under K1 with T,
  from the linter's driver probe and the cleaner (no kept run has that combination).

Not known:

- which answers the owner wants;
- whether the lint task inherits the lock's exported `RUSTUP_TOOLCHAIN`;
- what a CI job costs in wall time;
- why the daily toolchain update timer is disabled;
- what `rustup toolchain install --no-self-update nightly` and
  `rustup component add rustc-dev llvm-tools-preview rust-src --toolchain nightly`
  do to this host's installed floating channel,
  which one container arm would close;
- whether the counts M3 records are distinct positions in `package/git-policy/cli`,
  which was not checked
  by a command.

## What this round did not do

The third round ran the measurements;
this fourth revision ran none.
It read sources and the kept logs,
and changed text and records.

- It changed nothing that ships:
  no source a release or a test compiles,
  no configuration,
  no plan,
  lock or workflow.
  Its only script edit is a comment in a kept measurement's script (`m16-in-container.sh`),
  which changes no line of any run.
- It ran no container,
  no `cargo`,
  no `cargo dylint`,
  no driver build and no test,
  so the arm that would close question 1's open half was not run
  (it is named in "What is true today about the compiler and Dylint").
- It ran no `mise run toolchain:install`,
  no `pnpm install`,
  no release and no workflow.
- It did not run the ignored host test `host_real_run_over_wg_quicker_exempt`.
- It did not run `cargo-shadow` in a container,
  so the shadow's hand-off is read and not run.
- It did not measure Design CL,
  Design SH,
  Answer H's CI job,
  Answer C's image,
  or option A's route.
- It did not touch `doc/handover/` or `doc/decision/`.
- The silent pass M14 arm A measured is issue 6667,
  created after the third revision;
  this revision does not change that issue.

## The evidence folder

Every log this file cites is in
`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/`:

- `machine/machine-state.txt`:
  the host's read-only facts,
  with each command.
- `issue/issue-6643.md`:
  the issue as `gh issue view` printed it.
- `ci/ci-run-41274002049-static-step-excerpt.txt`:
  the step's last retrievable log.
- `gate-read/review.md`,
  `gate-read-second-2026-11-28/`,
  `gate-read-third-2026-11-28/` and `gate-read-fourth-2026-11-28/`:
  the four same-vendor reads,
  whole,
  with the probes each ran.
- `gate-read-third-2026-11-28/findings-answered.md`:
  every finding of all four reads,
  answered,
  and the reports of the revisions that answered them,
  this one's included.
- `m1-f1-…/` through `m6-…/`:
  the first round's measurements.
- `m7-k-with-minimal-floating-channel/`,
  `m8-option-i-on-the-host-like-floating-build/`,
  `m10-skip-statuses/`,
  `m11-rustup-does-not-install-an-absent-toolchain/`:
  the second round's measurements,
  with the limits of their logs stated in the text.
- `third-round-2026-11-28/`:
  this round's measurements (`m12/`,
  `m13/`,
  `m14/`,
  `m15a/`,
  `m15b/`,
  `m16/`,
  `m17/`,
  each with its `run.txt` and `log/`,
  and a `README.md` saying which state each arm
  stands for),
  the `Containerfile` and scripts that made them,
  and the `probes/` this file cites.
- `scripts/`:
  every in-container script of the first two rounds,
  so each run can be repeated,
  and the quotation checker with its list.
- `upstream/dylint-v6.4.0-excerpts.txt`:
  the upstream lines this file cites,
  with the tag and the
  tarball's digest.
