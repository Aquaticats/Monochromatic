# Planted controls name the code that holds their guard

A planted control is a deliberate break,
written down against a place in the Rust source,
that a mutation campaign plants to show the tests notice it.
The table lives in `bin/native-planted-controls-commit.ts`,
`bin/native-planted-controls-entry.ts` and `bin/native-planted-controls-landing.ts` and is run by
`bin/native-planted-controls.ts`.
Two TypeScript unit tests keep the table honest:
`bin/native-planted-controls.unit.test.ts` and `bin/native-planted-controls-harness.unit.test.ts`.

The table goes stale whenever a Rust change rewrites a guard or moves the test that holds it,
and nothing about the change itself fails.
This page is the obligation that follows such a change,
the check that catches it,
and the record of the week the check was not run.

## What one entry holds

- `name` is the sentence a campaign log prints.
- `file` is the source file the campaign edits,
  relative to this package.
- `from` is the text the campaign replaces,
  and it must occur in `file` exactly once.
- `to` is what it becomes,
  and the break must remove the same guard the `from` text guards.
- `failing` is the name of the test that must fail without that guard,
  spelled the way nextest prints it:
  the module path from the crate root,
  then `tests` or the name of the test module,
  then the function name,
  as in `shadow::shadow_ref_write::tests::files_shadow_snapshots_refs_and_points_head_at_the_branch`.

A campaign copies `file` out of the tested image,
replaces `from` with `to`,
runs `cargo nextest run --offline --locked --all-targets --no-fail-fast`,
and requires the line nextest prints for `failing` to be a `FAIL` or a `TIMEOUT`
(`bin/native-planted-controls-outcome.ts`).
A control whose named test passes with the guard planted out is worse than no control:
it says a guard is checked when nothing checks it,
which is what `8202ef15e6` repaired for the index-commit control.

## What a Rust change owes the table

- Text rewritten,
  renamed or read as bytes instead of as text:
  repoint `from` at the line as it stands now.
  `cb235065f9` turned `if token == "--all" {` into `if bytes == b"--all" {`,
  and the all-flag control matched nothing from that commit until 2026-11-16.
- A guard moved into another file or function:
  repoint `file` as well,
  so the campaign edits the file that holds the guard now.
  The signal-relay sender check moved into `read_report` in `src/native/process/signal_report.rs`
  when `699897702a` stopped the handler reading the report itself.
- A test renamed,
  split or moved to another module:
  repoint `failing` at the name nextest prints now,
  module path included.
  `664f516ab1` renamed the shadow-links test to `only_the_named_shared_entries_link`,
  and `7592e20235` moved the shadow ref write into `shadow_ref_write.rs`,
  which changed the module path of its control's test.
- A guard deleted:
  remove the control,
  and say in the commit message and on the entry why the guard no longer exists;
  never point the control at text that merely looks like the old guard.
  `45c4bcadb9` and `3bbfedd2d2` removed the controls whose guard or test the source no longer had.
- A guard added:
  add a control only after a run shows the named test failing with it planted out.
  The unit test alone says nothing about whether a test actually notices anything.

## The check that catches a stale control

The smallest check that fails at the merge which makes a control stale is the planted-controls unit test.
On 2026-11-16 it took 1.3 seconds at four CPUs on this host:

```sh
# Run from the repository top level.
mise run //package/git-policy/cli:test:unit \
  -- bin/native-planted-controls.unit.test.ts bin/native-planted-controls-harness.unit.test.ts
```

It fails with one line per stale control,
naming the control and the file whose text no longer matches,
or the test name the source no longer defines:

```text
only in actual: "the all flag is removed from the commit region: its text occurs 0 times in
src/native/commit/commit_args.rs"
```

The whole package run is the same task with no file names,
and took 11.3 seconds at seven CPUs on the same day:

```sh
mise run //package/git-policy/cli:test:unit
```

`native:ts-checks` runs that whole run beside the type check,
each in a capped scope of its own,
when its command line names no task,
so it is the same check with the package's TypeScript checks around it.

What the unit test reads,
and what it cannot:
it requires `from` to occur exactly once in `file`,
and it looks for `fn <last segment of failing>(` in any Rust source.
It does not verify the module path in `failing`,
so a test moved to another module leaves the unit test green while a campaign reports
`NOT noticed`.
After moving a test module,
compare the full name with what nextest lists:

```sh
# Run in package/git-policy/cli.
mise exec -- cargo nextest list --locked --offline --lib --message-format json \
  | jq --raw-output '."rust-suites"[]."testcases" | keys[]' | grep '::<test function>$'
```

The binary suite's names come from the same command with `--test native_binary` instead of `--lib`.

## The identity of the code that runs and judges the controls

`bin/native-planted-controls-harness.ts` holds `plantedHarnessId`,
a literal digest of `native-planted-controls.ts`,
`native-planted-controls-outcome.ts` and `native-verification-process.ts`.
A campaign's passing record names it,
so a record made by other code than the code that would run now proves nothing.
The unit test requires the literal to equal the digest of the sources as they stand,
prints the digest to write when it does not,
and fails until the literal follows.

Whenever one of the three changes,
replace the literal with the digest the test prints,
and read the change as a design change:
records made under the old identity stop counting,
and the controls run again.
The literal went stale at `e25745423a`,
which gave every gate check a bound through `native-verification-process.ts`;
the digest of the sources became `a428a679…` and the literal stayed `18ac2c8a…` from `4ecd55433e`.

## What runs at a merge, and what of it reads this table

The checks an integration pass runs before a branch reaches `main`
are listed in `/home/user/temp/agent/pi-runs/integration-pass-*-prompt.md`,
and every one of them is Rust work except the package's TypeScript unit tests:

- the build,
  both test compiles and Clippy;
- the guard modules by name;
- the branch's own controls by name;
- the whole library suite,
  one time,
  in a capped scope;
- the whole `native_binary` suite,
  one time;
- and,
  since pass 50 on 2026-11-15,
  the package's TypeScript unit tests of `bin/`,
  one time.

The release gate,
`mise run //package/git-policy/cli:native:test:container`,
runs the library and binary suites with Clippy in bounded containers over a snapshot of the crate,
and that snapshot holds `Cargo.toml`,
`Cargo.lock`,
`README.md`,
`SPEC.md` and `src/native`
(`bin/native-wrapper-snapshot.ts`),
never `bin/`,
so the gate cannot see a stale control.

No workflow under `.github/workflows` runs this package's TypeScript unit tests either:
`advisory-core.yml` runs a fixed fast subset of other packages,
`cargo-publish.yml` builds the crate,
and `cargo-release.yml` dispatches the release.

So exactly one check at a merge reads the planted-control table,
and it has been on the integration pass's list only since 2026-11-15.
That is the whole of why the four controls and `plantedHarnessId` went stale.

## What went stale between 2026-11-11 and 2026-11-15

Each line names the commit that made the control stale,
and the merge that carried it into `main`.
None of those merges ran the package's TypeScript unit tests.

- The all flag (`commit_args.rs`,
  `cb235065f9`,
  merged by pass 14 as `5e044eed53`).
- A branch base (`shadow_refs.rs` became `shadow_ref_write.rs`,
  `9a06be8d0e` and `7592e20235`,
  merged by pass 14 and by pass 23 as `b0af2f97c2`).
- The linked worktree's info directory (`shadow_links_tests.rs`,
  `664f516ab1`,
  merged by `95732afd6d`).
- The signal relay sender check (`signal_relay.rs` became `signal_report.rs`,
  `699897702a`,
  merged by pass 44 as `66d81ec97d`).
- The harness identity (`e25745423a`,
  merged by `362eff0777`).

Pass 50 was the first pass whose brief named the run,
and it listed these failures as known and another agent's,
as passes 51 to 57 did.
That reading is the one that must not be repeated:
a red line in this run says the campaign which plants that guard would plant nothing,
so it is the merge's to answer,
above all when the branch being merged is the one that moved the guard.
Repointing a control is a line or two in `bin/`,
and the pass that sees the red line is the one in a position to make it.
