# Translation repair history 2026-08-24 to 2026-08-25: the verification command that read an empty runs directory as a green run, and the combined build and test of the parked items

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

The verification command that stopped reading an empty runs directory as a green run,
with its guard proof and the second copy of the lister it turned up,
the combined build and test of all the parked items on one tree,
and the shorter landing sequence that followed from it.
The naming of the artifact family and the landing rehearsal are in `translation-repair-history-2026-08-24-to-25-two-lane-artifact-family-names-and-the-landing-rehearsal.md`.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## `#217` is built, GFP-proven and parked (2026-08-25)

Built in the fork worktree `/var/home/user/worktrees/verify-empty`,
checked out at `e8430d094`,
because the change edits `src/` and the calibration's slice cache is keyed on the pipeline digest.
It cannot be committed from there:
that worktree has no `node_modules/.bin/git`,
so the policy wrapper is absent.
It is parked as `~/temp/agent/verify-empty-217-218.tar.gz`,
which carries `#218` as well,
and is applied over the main worktree in the landing sequence.
The tarball was checked against the main worktree before parking:
all six modified files are byte-identical to the fork's base commit,
and all three new files are absent there,
so it applies without clobbering anything.

### What was wrong

`namesUnder` answered an absent directory with an empty array and printed the absence with `console.error`.
Stdout therefore read `verify-published: matched=0 settledWithNoPage=0 pageWithNoArtifact=0`,
then `verify-published: 0 of 0 pages carry every wording their artifact promised`,
and `process.exitCode` stayed 0.
An empty runs directory and a run whose every page agreed produced the same report and the same exit code,
so a mistyped `TRANSLATION_REPAIR_RUNS_DIR` read as a green run.

`errorName` compounded it.
It answers `error.name`,
which is `Error` for every filesystem failure,
so a directory that was never created and one at mode 000 printed the same `(Error)`.

### What landed

Absence became a kind rather than an empty list,
in two new modules.

-   `src/corpus-run/directory-listing.ts` holds `DirectoryReading`,
    `namesIn` and `filesystemReason`.
    `filesystemReason` reads `error.code`,
    so the report says `ENOENT` or `ENOTDIR` instead of `Error`.
-   `src/corpus-run/published-tree-listing.ts` holds `settledEntryIds`,
    `publishedEntryIds`
    and the verdict `whatThereIsToVerify`.

`verify-published.ts` gained a second exit code,
`NOTHING_WAS_VERIFIED = 2`,
kept separate from `PUBLISHED_TREE_DISAGREES = 1`:
a disagreement says the run shipped something wrong,
while the new code says the run was never examined,
and a gate that treats those alike either ships an unchecked run or refuses a good one.

An absent published tree is deliberately NOT one of the nothing-verified cases.
Beside real artifacts it means every settled entry is unpublished,
which is the most serious finding this check can make,
so it stays a finding with a count rather than collapsing into silence.

### The lister already had a second copy

`editor-standing-read.ts` carried its own `DirectoryReading` union and its own `namesIn`,
with the same `errorName` weakness and the same `console.error` plus empty-list shape.
Both now call the shared module,
following the rule `error-name.ts` records for itself:
lift at the point a further caller would be written.
Its one refusal message now names `ENOENT` rather than `Error`.

### Evidence

Boundary cases,
run against the built CLI on `mktemp` fixtures:

```text
absent runs directory        exit 2   NOTHING VERIFIED, no artifacts directory under the run (ENOENT)
run dir with no artifacts/   exit 2   NOTHING VERIFIED, no artifacts directory under the run (ENOENT)
artifacts/ present, empty    exit 2   NOTHING VERIFIED, the artifacts directory holds no settled artifact
artifacts, no published tree exit 1   NO PUBLISHED TREE (ENOENT). All 2 entries the run settled are unpublished
six agreeing pages           exit 0   matched=6, 6 of 6 pages carry every wording
```

The last row is the positive control,
and it is not invented:
it uses the six real artifacts from `~/translation-repair-runs-20260817`,
with each page synthesised from that artifact's own `shippableReplacements`
and checked with `pageCarriesEveryWording` before being written.
Without it,
every non-zero exit above would prove only that the check can refuse.

GFP:
with the verdict reverted to reading an unreadable listing as an empty one,
the two guard cases fail and the CLI reproduces `#217` exactly,
printing `0 of 0 pages carry every wording` and exiting 0 on an empty run.
Restored from copies kept in `~/temp/agent/217-gfp/`,
rebuilt,
and both pass again.

`published-tree-listing.unit.test.ts` covers 14 cases across five suites:
`namesIn` on a present directory,
an absent one and a file;
`filesystemReason` on a coded error,
an uncoded error and a thrown string;
both id listers on present and absent directories;
and all four verdict branches.

### One unrelated fix came with it

`probe-telemetry-report.unit.test.ts` imported `@monochromatic-dev/module-test`
rather than the `/ts` subpath every other test file uses,
which is what `ST3` requires.
It was the only such file in the package.
The fork exposed it because only this package's `dist/` is built there,
so the non-`/ts` path had nothing to resolve to:
one `TS2307` that cascaded into 12 `no-unsafe-call` and `no-unsafe-member-access` warnings.
Adding `/ts` cleared all 13.

## The parked work is now build-and-test verified together, not just apply-clean (2026-08-25)

`#210`,
`#211`,
`#212`,
`#217` and `#218` were each parked separately,
and each was checked only for whether its files applied cleanly over the main worktree.
Applying cleanly is not the same as compiling,
and none of them had ever been built together.

All five are now extracted into `/var/home/user/worktrees/verify-empty` on top of `e8430d094`,
and the combined tree was taken through the whole gate:

```text
build      exit 0
lint       exit 0   0 warnings, 0 errors  (oxlint type-aware plus tsc)
test:unit  exit 0   668 PASS, 0 FAIL
```

Six hundred and sixty-eight suites is nine more than the `#217` tree alone,
which is the `#210` and `#212` suites arriving.

The combined tarball is `~/temp/agent/parked-combined-20260825.tar.gz`,
42 files,
and it SUPERSEDES both `~/temp/agent/spend-telemetry-210.tar.gz`
and `~/temp/agent/verify-empty-217-218.tar.gz`.
Extract only the combined one;
extracting an older tarball afterwards would
overwrite files with their pre-combination contents.

### Why they were combined rather than kept apart

`#210` touches `provider-barrel.ts`,
`pipeline-barrel.ts`,
`ballot-barrel.ts`,
`candidate-select.ts`,
`editor-ensemble.ts`,
`judge-fidelity.ts`,
`refine-stage.ts`
and `translate-judge.ts`,
all of which further work is likely to touch.
Building each item against the same untouched base and landing them in sequence
would have let a later tarball silently overwrite an earlier one's edits to a shared file,
because a tarball carries whole files rather than a diff.
Building each item on top of the previous removes that hazard entirely.

No source file changed on the branch between the `#210` tarball's base and `e8430d094`:
the last commit touching `package/module/translation-repair/src/` (moved on 2026-10-18 to
`package/translation-repair/*/src/`) is `91fe0d0e6`,
and everything after it is documentation.
So extracting the older tarballs onto this base discarded nothing.

### The landing sequence is correspondingly shorter

Read the standing,
extract `parked-combined-20260825.tar.gz`,
build,
lint,
types,
test,
then commit in item order so each item keeps its own message.
The build,
lint and test steps have now been run once already and passed,
so a failure there after landing would mean the main worktree differs from this fork,
which is itself the thing worth knowing.
