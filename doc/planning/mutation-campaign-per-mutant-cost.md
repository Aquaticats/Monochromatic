# Why a git-policy-cli mutation campaign was estimated at 14 hours, and how to cut it

Status:
A and B accepted by the owner on 2026-11-01 ("A,
then B (Recommended)") and implemented on the branch
`worktree-agent-ab8921fc669ce25df`,
tested only against the launcher's stand-in `podman` and cargo-mutants;
the real
one-file measurement of each is the owner's to run before the queue resumes (the commands are in the handover report
and in the launcher README,
"Skipping the baseline" and "Two passes").
C and D are not done.
Correction to the section "A":
the golden `git-policy-baseline-skip` is `NATIVE_MUTATION_BASELINE_SKIP`,
which
leaves one named test out of every run;
the launcher had no baseline skip before this work,
and the new one is cargo-mutants'
`--baseline skip`,
taken only when the gate's evidence proves the same image passed.
The process limit of the gate and mutation containers is now 512,
not 128,
because the baseline of 2026-11-01 exited 4
when the binary suite's `fault_enospc` and `forwarding` tests failed on `fork` with `EAGAIN`.

## Question

The campaign over the files the 0.5.6 and 0.5.7 releases touched (about 1,500 mutants) was estimated at a dozen hours
or more by the agent that started it.
The owner:
"expecting 14h for a mutation run is really sus.
We should dig into
that as well."

## Measurements

Source:
the three running campaigns of 2026-11-02 (UTC),
each a `native:mutation:scoped` unit in a container with
`--cpus 4 --memory 3g`,
read from each attempt's `mutants.out/outcomes.json` and per-mutant logs with
`~/temp/agent/mut-m057/phase-times.ts` and `~/temp/agent/mut-m057/first-failure.ts`.

- Every mutant runs the whole package suite:
  `cargo nextest run --verbose --package=monochromatic-git-policy-cli@0.5.6 --offline --locked`,
  the same command as
  the baseline (the library and binary suites together,
  about 3,350 tests at the time).
- Baseline test phase:
  265.1 s,
  267.7 s and 268.6 s;
  each file's campaign runs its own baseline.
- Mutant build phase:
  median 16.0 to 16.4 s (25,
  22 and 16 mutants).
- Mutant test phase:
  median 39.8 s,
  54.6 s and 60.6 s;
  maximum 118 s,
  224 s and 259 s.
  A missed mutant runs the
  whole suite.
- Where a caught mutant first fails,
  as the position of the first `FAIL` line in nextest's completion count:
  median
  43%,
  60% and 60% of the suite.
- Whether the first failing test belongs to the mutated file's own test module:
  `repo_read_query` 1 of 28 (27 elsewhere,
  mostly `repo_read_hook`,
  which calls it),
  `object_abbreviation` 12 of 20,
  `repo_read_layout` 8 of 16.

## Reading

- About 70 s per mutant (16 s build,
  a median of about 55 s of tests) over about 1,500 mutants is about 29 unit-hours,
  so about 7 hours at four concurrent units,
  before missed mutants (about 265 s each) and the per-file baselines
  (about 265 s each,
  about 30 files,
  so about 2 more unit-hours).
  The estimate of a dozen hours is consistent with that;
  the cost is real,
  not a measurement error.
- The driver is the test phase:
  every mutant runs the whole suite,
  in an order unrelated to the mutated code,
  so the
  first failure arrives about halfway through.
- Running only the mutated file's own tests first would not be enough:
  for `repo_read_query` the killing tests are
  almost all in its callers.

## Options

These are complementary,
not alternatives.

### A. Skip the per-file baseline when the gate already passed on the same image

The launcher already supports a baseline skip (golden `git-policy-baseline-skip`).
A passing gate on the same image is the baseline.
Saves about 265 s of unit time per file campaign,
about 2 unit-hours for 30 files.
Risk:
none beyond trusting the gate's result for that image.

### B. Two passes: the tests that reach the mutated file first, the whole suite only for survivors

Pass 1 runs a nextest filterset of the test modules that use the mutated module (from the crate's `use` graph,
plus
the file's own tests).
Pass 2 runs the whole suite only on pass-1 survivors,
seeded with pass 1's caught list through
the launcher's existing `previously_caught.txt` mechanism,
so nothing is scored twice and nothing a full run would
catch is lost.
Expected effect,
not measured:
most caught mutants finish in seconds instead of about 55 s,
so the test phase drops by
well over half.
Risk:
the filterset must be generated per file;
a wrong filterset only costs time,
because pass 2 still runs
everything on survivors.

### C. One pass with likely killers first

Order the whole suite so the tests of modules that use the mutated file run first.
Depends on nextest supporting test ordering or priorities in the locked version;
not checked.
Smaller saving than B when the first killer is late in a module's own list.

### D. More concurrent units

Shortens wall time but not CPU time;
the host is shared with other sessions.

## Ranking

A > B > C > D:
A is free and certain;
B gives the largest cut without losing a kill;
C depends on an unverified nextest
feature and saves less;
D only moves the cost.
Recommended order:
A,
then B,
each measured on one file against the numbers in `Measurements` before the next campaign.
