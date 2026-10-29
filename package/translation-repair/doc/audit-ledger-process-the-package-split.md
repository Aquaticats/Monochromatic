# Audit ledger: process mistakes, the package split

What went wrong in the work of splitting `package/module/translation-repair` into the packages under
`package/translation-repair` (the move commit is `4bd1aea8d5`,
of 2026-10-18 (UTC)):
a dry run that never linted (M239),
slips of the move scripts (M240 and M241),
a worktree without its policy input (M242),
briefs that left out who owns a shared helper and which scans to run (M243 and M248),
shell and commit rules broken again (M244 to M247),
README wording written from module names (M249),
a handover past its line limit (M250),
a control read at the wrong commit (M251),
estimates and probes of the lint cost (M252 and M253),
a comparison that read key order as a difference (M254),
a link checker that read text as Markdown (M255),
and,
after the split,
a baseline lint run that linted the changed plugin (M262).
All are the split lead's own unless the entry says an agent's.
The defects the work found in the moved code,
configs and scans are filed in
[Audit ledger: the package split, configs, scans and messages](audit-ledger-package-split-configs-scans-and-messages.md).

Written on 2026-10-18 (UTC) from the split lead's notes (`s245/split/ledger-notes.md`),
the evidence files they cite under `s245/split/`,
which were not reopened for this file,
and the messages of the commits named in each entry.
The sizes and counts are the notes' own.
M262 was written on 2026-10-19 (UTC) from the lead's notes after the split (`s245/ledger-notes-after-split.md`).
Paths here are written in the layout after the split.
Where the notes name no prevention,
the entry says so.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M239: the dry runs of the move built and ran the suite but never linted

Status:
happened in the dry runs before the move commit,
and was found after it by a slice agent that linted `provider`.
The move commit `4bd1aea8d5` landed oxlint findings in 30 of 36 packages
(the census at the head of `w167` is `s245/split/lint/census.txt`):
- 526 `no-import-type-side-effects`,
- 440 duplicate imports,
- 40 TSDoc `{@link import('@monochromatic-dev/...')}` read as tags,
- 12 arrow functions in scaffolded rolldown configs,
- 5 barrels over 300 lines,
- 1 readonly parameter,
- and a `test-fixture` lint task that matches no file (B655).

The notes record that the dry runs built the packages and ran the suite and did not lint.

The fix:
the agent in `w172` cleared 23 packages,
and the six runner-side packages followed.
The log holds 29 commits of the form
`style(translation-repair-<package>): clear the lint output the split left in the package`,
5 of `... clear the import findings from the package's tests and fixtures`,
3 that split a public index into sibling barrels (`probe`,
`corpus-pass` and `run`),
and one that takes the run of an address as read-only where it is read (`d539ddc4ce`).

Prevention:
a move or rewrite script's dry run runs the same checks as the gate:
the build,
the lint,
the types,
the suite and the scans.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"A move's dry run".

## M240: the move script dropped a trailing re-export and doubled a newline

Status:
happened in the move.
`own.ts` dropped a trailing type re-export,
and left a double newline at the end of `adjudication/src/document-preparation.ts`.
The git policy wrapper caught it before the move commit.

Prevention:
the notes name none,
and the entry is a record.

## M241: a partial re-scaffold read the file it had overwritten

Status:
happened in the move.
A partial re-scaffold read the `build-entries.ts` it had itself overwritten,
and wrote a config with no runners.
The fix was to read the file from git `HEAD`.

Prevention:
the notes name none,
and the entry is a record.

## M242: the forbidden-strings rules file was missing in the new worktree

Status:
happened in the new worktree `w165`:
commits failed until the symlink to the rules file was made.
The cause and the remedy are recorded in `doc/troubleshooting/cli-git-temporary-worktree-policy-inputs.md`.

Prevention:
the troubleshooting doc holds the remedy,
and the notes name no other.

## M243: briefs for two slices did not say who owns the shared fixtures

Status:
happened in the briefs for the parallel slices B and C.
They did not say who owns `runBuiltCommand` and `runProbeOver`,
and both agents changed them in incompatible shapes:
slice C added an optional `builtDirectory`,
and slice B made `packageEntry` required.
The merge conflicted and needed a reconciliation agent (`w173`).

The merged form requires the owning package's entry from every caller and drops the directory fallback
(`2ec8ff1581`),
for the reason its message gives:
a default that stands in for an input is the defect family the package guards against.

Prevention:
a brief that splits work across agents names the owner of every shared helper the slices may touch.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Slices of one move".

## M244: five `;` chains in one session

Status:
happened five times in the split lead's session,
against the shell rule that has been recorded since M1.
- The lockfile commit used a `;` chain.
- The cherry-pick of `w172`'s commits did:
  a pipe into `rg 'CONFLICT'`,
  then `;`,
  then `git status`.
- A lint probe did:
  a redirect of its output,
  then `;`,
  then an `echo` of the exit status.
- The rerun of the census and audit control did the same (the fourth),
  and was sent in the same batch as the edit it read;
  it happened to read the edited file.
- The alias search did:
  a pipe into `rg --count`,
  then `;`,
  then a second `rg` (the fifth).

The agent in `w172` used one too (M247).

Prevention:
the notes' conclusion is that the rule needs a mechanical check and not memory.
The recurrence is added to "What enforces it" of "Shell commands" in
`mistake-prevention-shell-edits-commits-and-agents.md`.

## M245: a count of matches counted characters

Status:
happened in the split lead's session.
`rg --count-matches '.'` and `rg --count-matches ''` printed 27,234 and 11,802,
counts of characters and not of files or lines.
They were rerun with a real count before the figures were used.

Prevention:
a count of files or lines uses `--count`,
or a script over `--files-with-matches`.
Recorded in "Shell commands" of `mistake-prevention-shell-edits-commits-and-agents.md`.

## M246: two agents amended their own unpushed commits

Status:
happened in the split.
The docs agent amended one of its own unpushed commits,
against the rule that a commit is never amended.
The merge agent (`w173`) did the same for a subject over 120 characters.

Prevention:
the briefs now say "never amend,
even unpushed".
Recorded in "Commit messages" of `mistake-prevention-shell-edits-commits-and-agents.md`.

## M247: the final agent committed before its message check finished

Status:
happened in the split.
The agent in `w172` committed `68f97bcded`,
cherry-picked as the `test-fixture` exports fix (B650),
before its message check finished.
One body line is about 150 characters,
and the same command used a `;`.
The commit was not amended.

Prevention:
the notes name none beyond the rules of M244 and M246.

## M248: a slice never ran the scans, and six findings reached the merge

Status:
happened in slice C (`w169`).
It never ran `source-scans` after its changes,
and six findings in its coverage-census code surfaced only in the merge agent's checks:
a duplicate `runnerEntrySources`,
an error class,
a position pointer,
two top-level describes
and an unlisted blankness test.

The fixes are in `6d0bb011aa`
(the error class is marked as writing a message from named parts,
the position pointer names the list it means,
and the five copies of `runnerEntrySources` that nothing calls are dropped),
`d5ba7f1a4e` (the scan's marked classes and named parts,
the two allowances for the dead copies removed,
and the allowance for a listed-lines test following its function),
`f810d2a127`
(the cache-account sources test file held to one top-level describe)
and `d2021694c5`
(each census family test file held to one top-level describe).

Prevention:
every slice brief lists the scans in its checks.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Slices of one move".

## M249: README wording written from module names

Status:
happened in the README work of the split.
The README agent wrote some role wording from module names without opening the modules
(`consistency`,
`artifact` and `page-check` among them),
and the notes owed a reading against the code.

The reading was made in four commits,
`e3131ee526`,
`341f1a5d4d`,
`d4e562fa00` and `68dd8e92a0`,
each titled "check the ... READMEs against the code".
Their messages say they reworded the descriptions of what each module does to match its source,
and brought the export,
runner,
dependency and user lists in line with `src/index.ts`,
`mise.toml` and `package.json`.
The notes do not list the individual faults found.

Prevention:
the notes name none beyond the reading itself.

## M250: the handover grew past the line limit, and the guard that read it was not run

Status:
happened in the split lead's handover edits.
They took `doc/handover/translation-repair-handover-2026-09-06.md` to 1,076 lines,
against the owner's limit of 1,000.
The guard ("Markdown docs past the line limit") lives in `source-scan:test:unit`
and not in the `source-scans` task the lead had run,
so only the final agent's suite run caught it.

The fix (`3d196ce7f6`):
the checkpoints of 2026-10-03 to 2026-10-15 are filed in a part of their own
(main file 618 lines,
part 486),
both lists of the snapshot name the part,
and the Contents list,
which had also left out the 2026-10-13 part,
now names it.

Prevention:
`wc --lines` is read on every Markdown file a commit touches before the commit.
Recorded in "Docs that only grow" of `mistake-prevention-docs-and-references.md`.

## M251: the control of the census and the audit read the renames at the wrong commit

Status:
happened in the control script `w169-notes/control.ts`,
which read git's renames end to end,
from `2083d7bec1` to `HEAD`.
Two test fixtures moved as exact renames in `4bd1aea8d5`
(`built-pipeline-digest.test-fixture.ts` to `glossary`,
and `prose-texts.test-fixture.ts` to `source-scan`)
and were edited afterwards past git's rename-similarity threshold.
The control at `8d09e8e092` reported them as missing and unexplained (DIFFER),
though the audit read the same 1,639 files.

The fix:
the control reads the renames at the move commit
and chains the family-internal renames to `HEAD`
(`s245/split/control/census-audit-control.ts`),
and the rerun matches.
The result is recorded in `389f03897c`.

Prevention:
a rename-based control is read at the commit that moved.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Scans and controls across a move".

## M252: the plan estimated a lint cost from small slices of the old package

Status:
happened in the plan of the split,
which predicted 3 to 5 seconds of lint per package.
The measure (`s245/split/lint/times-paired.txt`,
same load,
three rounds) found leaf packages at 2.4 to 10 seconds,
`corpus-pass` at 32 to 35,
`probe` at 25 and `bench` at 22,
and the family linted package by package at 333 to 352 seconds,
against 25.5 seconds for the package before the split.
File count does not predict it.

The first guess,
that oxlint's type-aware rules read each package's source import closure,
was refuted by `cost-probe.txt`:
one oxlint over the whole family takes 233 to 263 seconds with the type-aware rules on
and 257 to 260 with them off,
and `corpus-pass` alone ran 33 seconds cold and 5 warm.
The hypothesis that held (`cost-probe2.txt`) was the persistent cache of the
`prefer-readonly-parameter-type` plugin:
one root for the repository (`node_modules/.cache/prefer-readonly-parameter-type`),
capped at 256 MiB and 4,096 entries,
which held 3,582 entries in 46 project directories and 276,255,501 bytes after the runs,
over its byte cap,
so that packages evict each other.
The planning docs `fdfe00c7a3` and `9c15bc2dd4` hold the measures.

The fix:
the handover records the owner's choice on the cache bounds (`303b816f7b`),
and `939176402f` raises them to 1 GiB and 16,384 entries,
`06987c9cce` covers the count,
byte and age bounds with tests,
and `96a615549e` states the bounds and their reason in the plugin's README.
With the bounds raised in a scratch worktree,
a warm family lint takes 70.9 seconds against 312 seconds,
and the family needs 8,269 entries and 508 MB,
about twice the old bounds;
cold stays 311 seconds (`43a206e02e`).
Sharing the analysis across projects is a proposal (`83270f8980`),
a prototype the owner chose (`da4c8dcab9`) and the prototype's measured results (`59f6acf347`).

The lesson:
the plan estimated a lint cost from small slices of the old package,
and the cost that scales was not the cost of the slices.

Prevention:
the notes name the lesson only.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Estimating and measuring a cost".

## M253: the first raised-bound lint probe edited a file that never ran

Status:
happened in the probe `cost-probe4.txt`.
It edited the plugin's `src` and checked the plugin's own export map,
but the root `oxlint.config.ts` loads the built `@monochromatic-dev/config-oxlint`,
whose `dist/final/node/plugin-prefer-readonly-parameter-type.mjs` bundles the plugin.
The edit never ran,
and the warm round's 312.84 seconds measured the old bounds.
The lead took it first as proof that the bound was not the cause,
and started reading validation code.
It was found when every deletion path in the source pointed back at eviction.

Prevention:
before an edit is measured,
the loaded artifact is searched for the edited value
(here `retained.count>65536` in the rebuilt bundle).
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Estimating and measuring a cost".

## M254: a comparison of closures read key order as a difference

Status:
happened in the split lead's first cross-project comparison of closures.
It serialized each `dependencyDigests` map with `JSON.stringify`,
so key order,
which differs by project,
read as 708 differing closures.
With sorted entries,
0 differ.

Prevention:
maps are compared as sorted entries,
and one example is read before a count is trusted.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Estimating and measuring a cost".

## M255: a link checker read quoted output as Markdown

Status:
happened in the docs agent's work.
Its link checker reported an escaped bracket in quoted output as a dead link
(`doc/planning/translation-repair-toka-ls-reading-2026-09-02.md`,
line 207).
The line is quoted output and holds no link.
The handover commit `a3f38fc909` records that the reported dead link is an escaped bracket and not a link.

Prevention:
a link checker parses Markdown and not text.
Recorded in "Searches and censuses that miss" of `mistake-prevention-guards-searches-and-claims.md`.

## M262: the first baseline run of the whole-repository lint comparison linted the hardened plugin

Status:
happened after the split of 2026-10-18 (UTC),
in the comparison of the lint of every package under the baseline bundle and under the hardened bundle of the shared
analysis (`w177-notes/fullrepo/`).
The saved baseline copy of `package/config/oxlint/dist` kept its old file times.
The lint task of the root `mise.toml` runs `ensureOxlintConfig` first,
which rebuilds that `dist` when a plugin source is newer than the oldest output,
so the first baseline run rebuilt the bundle from the hardened plugin source and linted with it.
The run is kept as `w177-notes/fullrepo/base-cold-invalid-rebuilt/`
and was not used as the baseline.

Prevention:
a bundle is swapped in with fresh file times,
and the live bundle is proved after every package,
not only at the start of a phase.
`mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Estimating and measuring a cost".
