# cargo-mutants 27.1.0 exits 3 when any mutant times out, so one stalled loop counter fails a mutation gate

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Status

Diagnosed on 2026-10-05.
The behavior is documented upstream and is not a defect;
the missing piece is an option to accept timeouts,
which upstream issue [#545][issue-545] already requests.
The user chose to exclude two replacement kinds by name in every mutation runner
and to reshape the remaining loops.
`Upstream filing decision` records the prototype of that option
and the comment that carries it on the upstream issue.

## Symptom

A mutation campaign whose mutants are all caught or timed out still exits with status 3,
so a `mise` task that runs it fails.
The summary line names the cause:

```text
TIMEOUT  src/lib.rs:6:15: replace += with *= in sum_below in 0s build + 10s test
9 mutants tested in 13s: 8 caught, 1 timeouts
```

Surface patterns that produced it in this repository:

- A hand-stepped loop counter or offset (`index += 1`, `offset += length`).
  `+=` replaced by `*=` multiplies instead:
  a step of one never moves the counter,
  and an offset at zero never leaves zero,
  so the loop condition never changes.
- A hand-stepped downward walk (`index -= 1`).
  `-=` replaced by `/=` divides by one,
  so the index never moves.
- A walk that follows a lookup until the lookup returns `None`.
  A constant replacement of the lookup's return value makes the walk circular
  (`MarkdownSource::parent` in `package/linter/monochromatic-lint`).

The incidents are recorded in `doc/handover/unified-linter-mutation-survivors.md`,
in the `Timeouts` sections of `doc/handover/unified-linter-processor-survivors.md`
and `doc/handover/cli-git-native-foundation.md`,
and in the `Timeouts removed` section of `doc/handover/unified-linter-mutation-close.md`.

There is one error variant:
exit status 3.
Status 2 (missed mutants) is a different outcome with a different remedy.

## Root cause

All citations are to cargo-mutants `v27.1.0`,
commit `8ab1dc786a1f61a4e370416cc6c68b81a704e917`.

### The exit status depends only on whether a timeout exists

`LabOutcome::exit_code` returns the timeout status as soon as the timeout count is nonzero.
`src/outcome.rs:110`:

```rust
// src/outcome.rs (cargo-mutants v27.1.0)
    pub fn exit_code(&self) -> ExitCode {
        // TODO: Maybe move this into an error returned from experiment()?
        if self
            .outcomes
            .iter()
            .any(|o| !o.scenario.is_mutant() && !o.success())
        {
            ExitCode::BaselineFailed
        } else if self.timeout > 0 {
            ExitCode::Timeout
        } else if self.missed > 0 {
            ExitCode::FoundProblems
        } else {
            ExitCode::Success
        }
    }
```

The function takes no options,
so nothing on the command line or in the config file reaches it.
`src/main.rs:621` returns its value as the process status:

```rust
// src/main.rs (cargo-mutants v27.1.0)
        let lab_outcome = test_mutants(mutants, &workspace, output_dir, &options, &console)?;
        Ok(lab_outcome.exit_code())
```

The timeout branch comes before the missed branch,
so a campaign with both timeouts and missed mutants exits 3, not 2.
A runner that treated status 3 as a pass would therefore also pass missed mutants.

`src/exit_code.rs:26` defines the status and matches the book (`book/src/exit-codes.md`):

```rust
// src/exit_code.rs (cargo-mutants v27.1.0)
    /// One or more tests timed out: probably the mutant caused an infinite loop, or the timeout is too low.
    Timeout = 3,
```

`cargo mutants --help` on the installed 27.1.0 lists five options that mention timeouts
(`--timeout`,
`--timeout-multiplier`,
`--minimum-test-timeout`,
`--build-timeout`,
`--build-timeout-multiplier`).
Each sets a time limit;
none changes what a timeout does to the exit status.

Upstream `main` at commit `9b09f6c6dce010f74b720737c857c4bac6ba91ea` (2026-10-01)
still has the same `exit_code` body and no `accept` option in `src/options.rs`.

### The stalling replacements come from a fixed operator table

The replacements for a compound assignment are literals in the visitor.
`src/visit.rs:593`:

```rust
// src/visit.rs (cargo-mutants v27.1.0)
            BinOp::Add(_) => vec![quote! {-}, quote! {*}],
            BinOp::AddAssign(_) => vec![quote! {-=}, quote! {*=}],
            BinOp::Sub(_) | BinOp::Mul(_) => vec![quote! {+}, quote! {/}],
            BinOp::SubAssign(_) | BinOp::MulAssign(_) => vec![quote! {+=}, quote! {/=}],
```

No option selects operators or genres,
so every `+=` yields a `*=` mutant and every `-=` yields a `/=` mutant.
Each excluded kind has a sibling at the same source position
(`+=` replaced by `-=`,
`-=` replaced by `+=`),
which matters for the workaround's cost.

### Name filters are the only per-kind control

`--exclude-re` is matched against the same name that `--list` prints,
including the path,
line,
column and replacement text.
`src/options.rs:479`:

```rust
// src/options.rs (cargo-mutants v27.1.0)
    pub fn allows_mutant(&self, mutant: &Mutant) -> bool {
        let name = mutant.name(true);
        (self.examine_name_re.is_empty() || self.examine_name_re.is_match(&name))
            && (self.exclude_name_re.is_empty() || !self.exclude_name_re.is_match(&name))
    }
```

`src/options.rs:352` builds that set from the command line and the config file together:

```rust
// src/options.rs (cargo-mutants v27.1.0)
            exclude_name_re: RegexSet::new(args.exclude_re.iter().chain(&config.exclude_re))
                .context("Failed to compile exclude_re regex")?,
```

So a pattern over the replacement text removes one replacement kind everywhere,
without naming files or functions.

## Verification

### Versions

- cargo-mutants 27.1.0,
  the host binary at `~/.cargo/bin/cargo-mutants`,
  SHA-256 `f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0`.
- rustc 1.97.0 (`2d8144b78`, 2026-07-07) inside the image `localhost/monochromatic-lint-test:development`,
  image ID `c778147e8c842e6c53108c9027a691e353c8672986115215d80b2c94027bf389`,
  built by `mise run //package/linter/monochromatic-lint:test:container`.
- Every run:
  no network,
  2 GiB of memory,
  2 CPUs,
  128 processes,
  fixture mounted read-only.

### Fixture

```toml
# <fixture>/Cargo.toml
[package]
name = "timeout-fixture"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]
```

```rust
// <fixture>/src/lib.rs
pub fn sum_below(limit: u32) -> u32 {
    let mut total = 0;
    let mut index = 0;
    while index < limit {
        total += index;
        index += 1;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::sum_below;

    #[test]
    fn sums_the_numbers_below_the_limit() {
        assert_eq!(sum_below(5), 10);
    }
}
```

### Harness

```sh
# doc/troubleshooting/cargo-mutants-timeout-exit-status.md
podman run --rm --network=none --memory=2g --cpus=2 --pids-limit=128 --security-opt label=disable \
  --volume "${HOME}/.cargo/bin/cargo-mutants:/usr/local/cargo/bin/cargo-mutants:ro" \
  --volume "${fixture}:/work/fixture:ro" --workdir /work/fixture \
  localhost/monochromatic-lint-test:development \
  cargo mutants --no-config --no-shuffle --colors=never --timeout 10 --output /tmp/out
```

`${fixture}` is the directory holding the two fixture files.
Append the options named in each case.

### Fails with exit status 3

- The harness as written:
  `9 mutants tested in 13s: 8 caught, 1 timeouts`.
  The timed-out mutant is `src/lib.rs:6:15: replace += with *= in sum_below`,
  the counter step.
  The same replacement on the accumulator at `src/lib.rs:5:15` is caught,
  because it changes the sum and stalls nothing.
- `--timeout 30` instead of `--timeout 10`:
  `9 mutants tested in 47s: 8 caught, 1 timeouts`.
  A longer limit only makes the stalled mutant cost more.

### Exits 0

- The harness plus `--exclude-re 'replace \+= with \*=' --exclude-re 'replace -= with /='`:
  `7 mutants tested in 3s: 7 caught`.
- The harness unchanged,
  against the fixture with the loop written as `for index in 0..limit { total += index; }`:
  `4 mutants tested in 1s: 4 caught`.

### What the patterns remove in this repository

`cargo mutants --list` generates mutants without building or running them,
so the cost of the patterns is countable.
Run from a crate directory:

```sh
# doc/troubleshooting/cargo-mutants-timeout-exit-status.md
cargo mutants --list --no-config | wc --lines
cargo mutants --list --no-config \
  --exclude-re 'replace \+= with \*=' --exclude-re 'replace -= with /=' | wc --lines
```

Measured on 2026-10-05:

- `package/linter/monochromatic-lint`, source at `08b561920`:
  1,672 mutants without the patterns and 1,640 with them.
  The 32 removed are 31 `+=` replaced by `*=` and 1 `-=` replaced by `/=`.
- `package/git-policy/cli`, source at `d1c02273a`:
  1,230 and 1,196.
  The 34 removed are 33 `+=` replaced by `*=` and 1 `-=` replaced by `/=`.
- `package/forbidden-strings/cli` with `--all-features`, source at `fe805727c`:
  526 and 521.
  The 5 removed are 2 `+=` replaced by `*=` and 3 `-=` replaced by `/=`.

For the linter,
a comparison of the two listings line by line showed that
only names containing one of the two replacements disappeared,
nothing was added,
and all 32 sibling mutants (`+=` replaced by `-=`, `-=` replaced by `+=`) remained.

## Verified workarounds

### Exclude the two stalling replacement kinds by name

Pass both patterns to every campaign:

```sh
# doc/troubleshooting/cargo-mutants-timeout-exit-status.md
cargo mutants --exclude-re 'replace \+= with \*=' --exclude-re 'replace -= with /='
```

All three mutation runners do this:
`package/linter/monochromatic-lint/bin/mutate-container.mjs`,
`package/git-policy/cli/bin/mutate-native-container.mjs`
and `package/forbidden-strings/cli/bin/mutate-container.mjs`.
They pass arguments rather than an `exclude_re` config key because they also pass `--no-config`.

Tradeoffs:

- The exclusion is by kind, not by mutant.
  It also removes the replacements that would not have stalled,
  such as the accumulator mutant in the fixture.
  The counts under `What the patterns remove in this repository` are the whole cost;
  how many of those mutants would have been caught, missed or timed out was not measured one by one.
- Each excluded position keeps its sibling replacement,
  so a statement that no test exercises is still reported through `+=` replaced by `-=`.
  What is lost is the second, weaker check on the same statement.
- The patterns do not cover other stalling mutants.
  In this repository,
  `+=` replaced by `-=` stalled three index-stepping scanners in `package/git-policy/cli`,
  and a constant return value made an ancestor walk circular in the linter.
  Those need `Give the loop a shape no replacement can stall`.
- A later cargo-mutants release that renames the replacement text would silently stop matching.
  The listing counts in `What the patterns remove in this repository` are the check to rerun after an upgrade.

### Give the loop a shape no replacement can stall

Iterate a range,
a slice or an iterator adapter,
or use a standard search,
so that progress does not depend on a statement cargo-mutants can rewrite.
The fixture's `for index in 0..limit` variant is the minimal case.
Applied in this repository:

- `package/git-policy/cli`:
  commit `d1c02273a` visits argument slices instead of stepping an index in three scanners,
  for the three `+=` replaced by `-=` timeouts.
- `package/linter/monochromatic-lint`:
  the `Timeouts removed` section of `doc/handover/unified-linter-mutation-close.md` lists each reshaped loop
  and the bounded ancestor walk that turns a circular parent lookup into a typed error.

Tradeoffs:

- Production code changes shape because of a test tool.
  Where the new shape is also the clearer one (a range instead of a hand-stepped index) that is free;
  where it adds an error path,
  as the bounded ancestor walk does,
  the path needs its own test and review.
- It is per loop.
  A new hand-stepped loop reintroduces the failure until the campaign runs again.

## What does not work

- Raising the timeout.
  `--timeout 30` gave the same status 3 on the fixture;
  a stalled loop outlasts any limit.
- Looking for an option.
  The installed `--help` and `src/outcome.rs:110` show there is none in 27.1.0 or on upstream `main`.
- Treating status 3 as a pass in the runner.
  Not run,
  rejected on the source:
  `exit_code` checks timeouts before missed mutants,
  so status 3 can hide missed mutants,
  and a slow shared host can produce timeouts unrelated to any loop
  (`doc/handover/unified-linter-mutation-close.md` records a library suite that took 183.94 seconds
  against a 180 second limit on unchanged tests).
- Calling a timeout "the detection" and leaving it.
  Earlier dispositions in `doc/handover/unified-linter-processor-survivors.md` did this.
  The reasoning about the mutant is sound,
  but the campaign still exits 3,
  so the gate cannot pass.

The maintainer's own suggestions in discussion [#490][discussion-490] are
excluding the mutant or its function,
accepting the timeout,
and adding progress assertions (`debug_assert`) inside the loop.
Only exclusion was tried here,
at the granularity of a replacement kind.

## Upstream filing decision

`.out-of-scope/` has no entry for cargo-mutants or for mutation tooling
(searched for `cargo-mutants` and `mutants` across the directory on 2026-10-05),
so the constraint check applies.

### Existing upstream threads

Searched `sourcefrog/cargo-mutants` issues and pull requests,
open and closed,
for `timeout` and `accept` on 2026-10-05.

- Issue [#545][issue-545],
  "Add flag to consider timeouts as 'caught'",
  open since 2025-08-20.
  This is the same request.
  The maintainer proposed `--accept=timeout`,
  extensible to `--accept=timeouts,missed`,
  and listed the implementation steps
  (`accept: Vec<SummaryOutcome>` in `Options`, `Args` and `Config`,
  use in `LabOutcome::exit_code`,
  book entry,
  unit and integration tests).
  The reporter offered to implement it on 2025-08-20,
  and a second user offered on 2026-07-03.
  No pull request for it exists.
- Issue [#499][issue-499],
  "Add more guidance on handling timeouts",
  open:
  a note to move discussion [#490][discussion-490] into the book.
- Issues #423 ("Notice test failures that accompany hangs") and #194 ("Run one, or a few, tests per subprocess"),
  both open:
  they would let a failing test end a mutant before another test stalls.
  They do not help when the stalled test is the only one that reaches the loop.

A new issue would duplicate #545,
so none is drafted.

### Constraints

1.  Is it upstream's fault?
    Partly.
    The exit status is documented and deliberate;
    the gap is the missing option,
    which the maintainer acknowledged in #545.
2.  Can upstream fix it?
    Yes.
    The maintainer scoped the change in #545,
    and `exit_code` is one function.
3.  Is the use case supported?
    Yes.
    The maintainer named continuous integration as the case for the option.
4.  Would the repository welcome a contribution?
    Yes, as far as its files show.
    `CONTRIBUTING.md` asks for a discussion or bug first (#545 is that),
    `AGENTS.md` gives instructions to coding agents,
    and pull request #559,
    opened by the `Copilot` account on 2025-10-06,
    was merged.
    No ban on assisted contributions was found in `CONTRIBUTING.md`,
    `AGENTS.md`,
    `README.md`,
    `DESIGN.md` or `.github/`.
5.  Will they likely fix it?
    Plausibly.
    The maintainer answered the same day and described the work;
    nothing has landed since August 2025,
    and nobody has declined it.
6.  Is a minimal fix prototyped?
    Yes.
    The patch exists,
    its exit-status cases are measured against a pristine build,
    and its test, format and Clippy results are compared with the pristine tree.
    `Prototype` names the one check that was not run.

### Prototype

Written and verified on 2026-10-05,
except for the item listed as open at the end of this section.

The patch is [cargo-mutants-timeout-exit-status.patch](cargo-mutants-timeout-exit-status.patch),
a `git diff` against `v27.1.0` from a disposable clone.
It adds `--accept` and an `accept` configuration key with the values `timeout` and `missed`,
combines command-line and configuration values,
and makes `LabOutcome::exit_code` skip the timeout and missed branches for accepted outcomes.
Printing,
the summary line
and `mutants.out` are unchanged.

It departs from the maintainer's step list in one place:
the value type is a new enum with two variants instead of `SummaryOutcome`,
because the other `SummaryOutcome` values cannot cause a failing exit status
and its serialized names are part of the JSON output.

Measured in a container without network,
2 GiB of memory and 2 CPUs,
rustc 1.97.0,
with a pristine `v27.1.0` build and the patched build side by side
(the fixture of `Verification`,
plus a variant with an untested function for missed mutants):

- Pristine, timeout only: exit 3.
- Patched, timeout only, no option: exit 3.
- Patched, timeout only, `--accept=timeout`: exit 0.
- Patched, timeout and missed, `--accept=timeout`: exit 2.
- Patched, timeout and missed, `--accept=timeout,missed`: exit 0.
- Patched, timeout only, `accept = ["timeout"]` in `.cargo/mutants.toml`: exit 0.
- Patched, timeout and missed, `--accept=missed`: exit 3.
- Pristine with `--accept=timeout`: usage error, exit 1.
- Pristine with the configuration key: `unknown field`, exit 1.

Also measured on the patched tree:
the new end-to-end test and two neighbouring upstream tests passed
(`accept_missed_exits_successfully_with_uncaught_mutant_in_factorial`,
`uncaught_mutant_in_factorial`,
`emit_config_schema`),
and 270 of 272 unit tests passed.

A second run on an image rebuilt from the saved patch compared both trees with the same toolchain
(rustfmt 1.9.0, clippy 0.1.97, user ID 0):

- The nine cases gave the same exit statuses again.
- `cargo fmt --check` exits 0 on the patched tree and on the pristine tree.
- Unit tests (`cargo test --locked --offline --bin cargo-mutants`):
  the pristine tree passes 261 and fails 2,
  the patched tree passes 270 and fails the same 2,
  `build_dir::test::fail_to_overwrite` and `build_dir::test::fail_to_overwrite_dir_permission_denied`.
  The 9 added tests all pass.
  Both failing tests expect a permission error and the container runs as root;
  that cause is an inference,
  but the failures are measured as present without the patch.
- `cargo clippy --all-targets --all-features -- -D warnings` exits 101 on both trees
  with the same 6 errors at the same positions
  (`redundant reference in format! argument` at `src/mutant.rs:144:31`
  and five positions in `src/output.rs`, lines 130 to 140).
  The patch adds none;
  upstream's own sources do not pass this Clippy version.

A new option changes the help text,
the completions and the configuration schema,
so the upstream integration tests whose names contain `show_help`,
`completions`,
`config` or `option` were run on the patched tree as well:
26 passed and none failed,
among them `show_help`,
`completions_option_generates_something`,
`emit_config_schema` and `example_config_file_can_be_loaded`.
No snapshot under `tests/snapshots/` captures the help text;
`show_help` asserts one substring.

Open, not measured:

- The rest of the upstream integration suite (114 of its 140 tests) was not run,
  and no test runs a timing-out tree through the command line with `--accept=timeout`
  (upstream's own test that lets a mutant hang,
  `mutants_causing_tests_to_hang_are_stopped_by_manual_timeout`,
  is `#[ignore]`);
  the nine cases cover that path outside the test suite.

The clone,
the build context and the case log are under `~/temp/agent/upstream-prototype.p0aROR58/`;
the image is `localhost/cargo-mutants-accept-prototype:v27.1.0`.

### Comment draft

Posted on 2026-10-05,
after the user chose "Post a comment" over keeping it local or opening a pull request:
<https://github.com/sourcefrog/cargo-mutants/issues/545#issuecomment-6005242222>.
The issue,
upstream `main` and the open pull requests were rechecked immediately before posting and were unchanged.
The posted text is this draft with its bracketed sentence removed
(the user gave no review statement,
so the comment claims none),
with the semantic line breaks joined,
because GitHub renders a newline inside a comment paragraph as a line break,
and with the patch inside the `<details>` block in a four-tilde `diff` fence.
The rendered comment was fetched back and shows its four headings and the collapsed patch.

The thread already contains the request,
the design and the step list.
What this draft adds is the verified patch,
the places where the step list needed a different choice,
and a workaround for loop counters that the thread does not mention.
The patch that went inside the `<details>` block is
[cargo-mutants-timeout-exit-status.patch](cargo-mutants-timeout-exit-status.patch).
No pull request is drafted,
because a second user offered on 2026-07-03 to own the change.

~~~md
I tried the step list from this thread against v27.1.0 (8ab1dc7) to see whether it holds together.
It does. The patch is at the end of this comment.
I have not opened a pull request because @jmriesen offered to own the change.

### What the patch does

- Adds `--accept` (comma-separated or repeated) and an `accept` key in `.cargo/mutants.toml`,
  with the values `timeout` and `missed`.
  Command-line and config values are combined, like the other list options since 27.0.0.
- `LabOutcome::exit_code` takes `&Options` and skips the timeout branch or the missed branch
  when that outcome is accepted.
  Nothing else changes: outcomes are still printed, counted in the summary line and written to `mutants.out`.
- Adds unit tests for option parsing, config parsing and `exit_code`,
  one CLI test (`--accept=missed` on `testdata/factorial`),
  a section in the exit codes page of the book, a pointer from the timeouts page, and a NEWS entry.

### Where it differs from the step list

- The value type is a new two-variant enum rather than `SummaryOutcome`.
  The other four `SummaryOutcome` values cannot cause a failing exit code,
  and its serialized names are part of the JSON output,
  so reusing it would either accept values that do nothing or need a second set of names.
- There is no CLI test for the timeout case,
  because the existing test that lets a mutant hang
  (`mutants_causing_tests_to_hang_are_stopped_by_manual_timeout`) is `#[ignore]`d.
  Unit tests of `exit_code` cover that branch.
- The summary line still says "N timeouts".
  The question earlier in this thread about wording is untouched.

### What was run

rustc 1.97.0, in a container without network,
on a small crate whose loop counter mutant (`index += 1` replaced by `index *= 1`) hangs,
and a variant with an untested function:

- v27.1.0 unpatched, timeout only: exit 3.
- Patched, timeout only, no option: exit 3.
- Patched, timeout only, `--accept=timeout`: exit 0.
- Patched, timeout and missed, `--accept=timeout`: exit 2.
- Patched, timeout and missed, `--accept=timeout,missed`: exit 0.
- Patched, timeout only, `accept = ["timeout"]` in the config file: exit 0.
- Patched, timeout and missed, `--accept=missed`: exit 3.

`cargo fmt --check` passes.
The 9 added unit tests and the added CLI test pass.
`build_dir::test::fail_to_overwrite` and `build_dir::test::fail_to_overwrite_dir_permission_denied`
fail in that container with and without the patch (it runs as root),
and clippy 0.1.97 with `-D warnings` reports the same 6 `useless_borrows_in_formatting` errors
in `src/mutant.rs` and `src/output.rs` with and without the patch.
The 26 integration tests whose names contain `show_help`, `completions`, `config` or `option` pass;
the other 114 were not run.

### A workaround that needs no change

For hand-stepped loop counters specifically,
`--exclude-re 'replace \+= with \*='` and `--exclude-re 'replace -= with /='`
remove the two replacements that leave a counter in place,
while `+=` replaced by `-=` still tests the same statement.
That is what I use in the meantime.

The patch and this comment were written with an AI coding assistant, which also ran the checks listed here.
[State what you reviewed yourself before posting.]

<details>
<summary>Patch against v27.1.0</summary>

(paste the patch here inside a `diff` code fence)

</details>
~~~

[issue-545]: https://github.com/sourcefrog/cargo-mutants/issues/545
[issue-499]: https://github.com/sourcefrog/cargo-mutants/issues/499
[discussion-490]: https://github.com/sourcefrog/cargo-mutants/discussions/490
