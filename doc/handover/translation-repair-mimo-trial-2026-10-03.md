# Translation repair: trial handover to MiMo, 2026-10-03

Part of the [translation repair handover](translation-repair.md).

Claude Code wrote this on 2026-10-03 (UTC) for MiMo 2.6 Pro,
which works this package for about 48 hours from 2026-10-03 while the owner is away.
Read this whole file before running any command.
It is the one document you must read in full;
every other link is for when a step sends you there.
A session resuming the trial reads "Trial log" first:
its newest line names the last commit and the step in progress.
Write that line after every commit,
so a new session picks up mid-step.
Everything stated as measured was run on 2026-10-03 in a cloud copy of the repository
(Node 26.10.0,
pnpm 12.5.1,
mise 2026.10.0);
everything else names where it comes from.

The trial ended on 2026-10-05.
"Outcome:
merged on 2026-10-05" says what became of its work,
and which of this file's instructions turned out wrong.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Translation repair MiMo trial: log of 2026-10-05, the coverage attributions re-verified](translation-repair-mimo-trial-2026-10-03-log-2026-10-05-attribution-reverification.md):
  The trial log from 01:49 to 05:22 UTC on 2026-10-05,
  when the eight ledger paragraphs blaming the census's attribution were found unverified and re-opened;
  it holds the correction of the record,
  the byte-level method that replaced it,
  and the left-open clusters re-verified one by one afterwards.
- [Translation repair MiMo trial: log from 2026-10-04 20:45 UTC to 2026-10-05 01:20 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-2045-to-2026-10-05-0120-t8-batches.md):
  The trial log from 20:45 UTC on 2026-10-04 to 01:20 UTC on 2026-10-05:
  T8 batches on the clusters of three and four uncovered stretches,
  each batch a closing line and the baseline census that follows it.
- [Translation repair MiMo trial: log of 2026-10-04 from 14:42 to 20:37 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-1442-to-2037-t8-batches.md):
  The trial log from 14:42 to 20:37 UTC on 2026-10-04:
  T8 batches on clusters of four and five uncovered stretches,
  each batch a closing line and the baseline census that follows it.
- [Translation repair MiMo trial: log of 2026-10-04 from 06:30 to 14:34 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-0630-to-1434-t8-batches.md):
  The trial log from 06:30 to 14:34 UTC on 2026-10-04:
  T8 batches on clusters of five,
  six and seven uncovered stretches,
  each batch a closing line and the baseline census that follows it.
- [Translation repair MiMo trial: log of 2026-10-04 from 02:42 to 06:05 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-0242-to-0605-t8-batches.md):
  The trial log from 02:42 to 06:05 UTC on 2026-10-04:
  T8 batches on clusters of seven,
  eight and nine uncovered stretches,
  each batch a closing line and the baseline census that follows it.
- [Translation repair MiMo trial: log of the B123 fix, 2026-10-04](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-b123-fix.md):
  The two trial log lines from 01:37 and 02:26 UTC on 2026-10-04 that record B123,
  the footnote relabel refusing a page with an unpositioned link literal:
  the red case and the design settled by a probe,
  then the fix and the cache-version measurement.
- [Translation repair MiMo trial: log from 2026-10-03 18:20 UTC to 2026-10-04 01:13 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-03-1820-to-2026-10-04-0113-first-t8-batches.md):
  The trial log from 18:20 UTC on 2026-10-03 to 01:13 UTC on 2026-10-04:
  the first whole-suite census and the first T8 batches,
  on clusters of nine and ten uncovered stretches,
  from the group cluster to the openrouter cluster.
- [Translation repair MiMo trial: log of 2026-10-03 from 16:20 to 18:14 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-03-setup-and-b126-fix.md):
  The trial log from 16:20 to 18:14 UTC on 2026-10-03:
  the handover written,
  the merge of main,
  the setup and the starting suite,
  the B126 fix and the flaky fixture that followed it.

## The mistakes this package punishes most

These five come from the package's own record of what went wrong
([`mistake-prevention.md`][prevent] and the ledger's M entries);
the first was measured again while writing this file.

1.  The tests read the build,
    not the source.
    Every `src/**/*.unit.test.ts` imports `../dist/final/node/index.mjs`,
    so an edit to a `.ts` file changes no test result until the package is built again.
    Measured on 2026-10-03:
    the fix for the failing case in "First:
    fix B126" left the case failing until a rebuild,
    and passed after it.
    Build before every test run
    (`buildAndTest` does both).
2.  A run is read from its log,
    never from its exit status alone.
    Count the `[FAIL]` and `[PASS]` lines the log prints (ledger M2,
    M12,
    M19).
    A file stops at its first failing suite,
    so one `[FAIL]` can hide cases that never ran (ledger B99).
3.  Shell calls stay small:
    at most three steps joined by `&&`,
    never `;`,
    never a shell loop,
    and package tasks start from the worktree,
    where the package exists (ledger M1,
    M107,
    M110;
    `AGENTS.md` rule 1CB).
    A step that needs more becomes a script file run on its own.
4.  A failing test is a finding,
    not an obstacle.
    Never change an expectation to match the code,
    skip a case,
    loosen a lint rule
    or delete a test to get green.
    If an expectation is wrong,
    the commit message says why,
    with the evidence.
5.  Commit after every step,
    naming each file in the commit's pathspec,
    and never amend,
    rebase
    or force-push (`AGENTS.md` rules GCE,
    CLG,
    CPN,
    GCA).

## What the package is

`package/module/translation-repair` (moved on 2026-10-18 to the packages under `package/translation-repair`) takes a
Chinese original and its existing English translation (the archive),
asks a roster of models to critique and repair it slice by slice,
and ships a repaired page.
It is not production ready
([`translation-repair-readiness-signal.md`](../planning/translation-repair-readiness-signal.md)).
Since 2026-09-27 the work has been the owner's audit of the whole package:
"audit the whole translation-repair pkg for all the mistakes we've made and fix all of them".
Every finding,
its evidence,
fix and guard is an entry in the [audit ledger][ledger],
lettered by area.
This trial adds to three letters:
B for a defect in the package (the ledger's "Recurring code families" section),
M for a mistake made while fixing ("Process mistakes in this audit"),
and T for a gap in the tests ("Test suite").
These three sections moved on 2026-10-23 from `audit-ledger.md`
to the [areas file][ledger-areas] of the ledger,
`audit-ledger-areas-of-the-audit-and-the-files-filed-under-each.md`,
where each stands under the same heading.

## Where things stand on 2026-10-03

- Branch `translation-repair-rebased`,
  last code commit `98e94e584`,
  in which the owner committed two test files left by agents that stopped part way.
  This file arrives in the docs commit after it.
- The suite has exactly one failing case on that commit,
  and it is a real defect,
  now B126:
  in `src/contributor-name-authority.unit.test.ts`,
  "READS NO NAME from a link whose label is empty,
  which shows a reader nothing,
  rather than its markup (T8)"
  expects `['Pebble']`
  and gets `['[](https://example.test/whisker)', 'Pebble']`
  (measured).
  The new case in `src/corpus-run/canadian-spelling.unit.test.ts` passes (measured).
- The last full suite before that commit,
  `14e4e2cea` on the owner's machine,
  printed 1,545 PASS lines and no FAIL line (its commit message).
- T8,
  code no unit test runs,
  is the open entry the next corpus pass waits for.
  The last whole-suite census,
  `census-3G9C58` at `d0cca211d`,
  held 590 stretches of library source no test ran.
  Batches since closed 81 of them,
  and bundles 1 and 2 of the remaining 509 landed (`23090a91d`,
  `ba6da4640`).
  The cut of the other bundles was never committed,
  so the next batch starts from a fresh whole-suite census
  (the ledger's T8 entry,
  its last paragraphs;
  the entry moved on 2026-10-23 to the [areas file][ledger-areas] of the ledger,
  under "Test suite").
- B123 is open:
  on a page holding a footnote and a `www.` link literal the parser leaves unpositioned,
  the footnote relabel refuses the whole page.
  It reproduces on the build of `98e94e584` (measured);
  no pinned page triggers it (a 2026-10-02 census).
- No corpus pass has run since 2026-09-28.
- Ledger numbers:
  B126 is taken by the failing case,
  so the next new defect is B127 and the next process mistake is M111.

## Your authority and the owner's choices

The owner answered on 2026-10-03,
before leaving:

- Authority:
  "Same authority as Claude,
  but I don't think it will finish the audit fast enough (in 48h) anyway."
  Claude's standing orders come with it.
  The next corpus pass waits for T8 to close,
  under the owner's directive of 2026-09-28 to prefer the quality of the end result,
  and only the owner may veto that order,
  so in practice this trial launches no corpus pass.
  Unit tests,
  censuses,
  builds
  and every task that spends no quota are yours without asking.
- Branch:
  commit to a new branch,
  `translation-repair-mimo-trial`,
  cut from the docs commit carrying this file.
  Never commit to or push `translation-repair-rebased`.
- The failing case is your first step,
  "First:
  fix B126".

The owner is away and answers nothing until the trial ends,
so no step waits on the owner.
This file has no stopping point:
when a step closes,
the next starts,
and the owner expects T8 to outlast the 48 hours.
A turn never ends on a summary,
a plan
or a question (ledger M101).
A choice the stated rules and a measurement settle is settled:
make it and record it (`AGENTS.md` rules QDF,
QGR,
QNX).
A choice that still turns on preference alone
takes the option that serves the quality of the shipped page,
the owner's standing directive of 2026-09-28;
act on it,
and record it under "Decisions for the owner to review" with the options,
the evidence and the reason,
so the owner can veto it on return.

## Setup, once

Run each line as its own call,
from the worktree the snapshot's "Repository state" names
(`/var/home/user/worktrees/translation-repair` when it was written;
confirm with `git rev-parse --show-toplevel`).

```sh
# from the worktree root
git status --short
git fetch origin translation-repair-rebased
git switch --create translation-repair-mimo-trial --no-track origin/translation-repair-rebased
git log --max-count 1 --format='%h %s'
```

- A path `git status --short` prints outside `package/module/translation-repair` (moved on 2026-10-18 to the packages
  under `package/translation-repair`) and `doc` is someone else's work in progress:
  leave it,
  never stage it,
  and never restore or stash it (`AGENTS.md` rule EC1).
- A path it prints under either is carried onto the trial branch by the switch:
  read it with `git diff`,
  commit it there as found,
  alone,
  as `chore(module-translation-repair): commit changes found at the trial's start`,
  naming each path,
  and note it in "Trial log".
  If the switch refuses because such a change conflicts,
  cut the branch where the worktree stands
  (`git switch --create translation-repair-mimo-trial --no-track`),
  commit the change the same way,
  then `git merge --no-edit origin/translation-repair-rebased` so the branch carries this file.
- `--no-track` matters.
  Without it the new branch tracks `origin/translation-repair-rebased`
  (measured on 2026-10-03:
  `branch 'probe-track' set up to track 'origin/translation-repair-rebased'`),
  and the auto-push after each commit would follow that upstream.
  With it the branch has no upstream,
  and cli-git's first auto-push runs `git push --set-upstream origin HEAD`
  (`package/git-policy/cli/README.md`,
  "Post-commit auto-push"),
  which creates `origin/translation-repair-mimo-trial`.
- The log line must show the docs commit that added this file.

Then make a folder for logs and take the starting suite:

```sh
# from the worktree root
mkdir --parents "${HOME}/temp/agent/mimo-trial"
chmod 700 "${HOME}/temp/agent/mimo-trial"
mise run //package/module/translation-repair:buildAndTest > "${HOME}/temp/agent/mimo-trial/suite-start.log" 2>&1
```

Moved on 2026-10-18 from `//package/module/translation-repair:buildAndTest`
to `//package/translation-repair/<package>:buildAndTest`.

Read the log:
`rg --count '\[FAIL\]' <log>` and `rg --count '\[PASS\]' <log>`.
Expect exactly the one failing case of "First:
fix B126",
which prints three `[FAIL]` lines
(the case,
its suite and the file's root suite).
Any other `[FAIL]` line on this machine is a finding of the same kind:
take the next B number and fix it on the steps of "First:
fix B126",
right after B126 and before the census,
which refuses a failing suite.
In the cloud copy the suite took about four minutes after the build.

If you are not on the owner's machine:
the cloud copy ran `mise trust`,
`mise install node pnpm`
and a `pnpm install` filtered to this package and its dependencies,
and there 14 test files besides the B126 one failed
(measured in the cloud copy).
Every failure message read there names the missing pinned corpus at `~/one-among-us/data`,
a temporary git repository the container's `git` could not make,
the shallow clone,
or a case that needs a user other than root;
one `pass-entry` case
(an outcome of `resumable-failure` where `stopped` was expected)
was not traced.
On the owner's machine the same suite printed no FAIL line at `14e4e2cea`,
and the census refuses a suite that fails,
so work on the owner's machine.

## First: fix B126, the empty-label link

The symptom is in "Where things stand on 2026-10-03".
The cause,
read from the source:
`contributorForm` in `src/contributor-name-authority.ts`
finds a link's label end with `unmarked.indexOf('](')`
and returns the token unchanged when `labelEnd <= 1`.
That one check joins two different cases:
`-1`,
no `](` at all,
where the token is a plain name such as `[Whisker`,
and `1`,
an empty label `[](...)`,
where nothing shows to a reader.
For the empty label the visible form is the empty string,
which `archiveContributorNameForms` already drops through its `nonempty` filter,
and the function's `@returns` already says "empty for empty token".
Changing the check so that only a missing `](` returns the token,
then building,
made the file pass in the cloud copy (5 PASS lines,
0 FAIL,
over three test files);
that edit was reverted,
so the fix is yours to make.

Keep the fix to that check and the words around it.
Moving this reader onto the Markdown parse is a larger change no finding asks for.

Steps:

1.  Read `src/contributor-name-authority.ts` and its test file whole before editing (ledger M6).
2.  Fix the check.
    Keep the TSDoc true:
    if a comment or `@returns` no longer says what the code does,
    reword it.
3.  Run the full suite (`buildAndTest` into a new log),
    not named files:
    `archiveContributorNameForms` feeds `prepareDocument`,
    which dozens of test files reach (ledger M105).
    Expect `[FAIL]` 0.
4.  Decide whether a cache version moves.
    The change alters what the contributor floor protects,
    and `document-preparation.ts`,
    `page-identity-lines.ts`
    and `archive-replacement-candidates.ts` read it.
    A cached slice changes only if a pinned archive page has an empty-label link on a contributor line.
    Measure it on the pinned corpus:
    `git -C ~/one-among-us/data grep --line-number --fixed-strings '[](' a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
    then read each hit for a contributor label
    (`CONTRIBUTOR_LABELS` at the top of the source file).
    No hit on such a line means no cached decision changes,
    and the ledger entry says so as inference from that search.
    A hit means reading "Cached decisions" in [`mistake-prevention.md`][prevent] before committing.
5.  Run the lint and the source scans
    ("Commands",
    the lint and source-scan lines),
    and read each result line.
6.  Write ledger entry B126 in the [audit ledger][ledger],
    after B125,
    on B125's model:
    what was wrong,
    where,
    how it was found
    (a T8 bundle agent's case,
    red in `98e94e584`),
    the evidence,
    the fix,
    the cache decision with its measurement,
    and a "Recurrence:"
    line naming the family in [`mistake-prevention.md`][prevent] whose rule covers it,
    adding one sentence to that family's "What happened".
    "Text that shows nothing" and "Structure read off the parse" are the nearest;
    pick the one whose rule would have prevented it,
    or say neither does.
7.  Commit the fix and its docs,
    with a message on the model of `14e4e2cea`:
    `fix(module-translation-repair): B126, read no contributor name from a link whose label is empty`,
    then a body saying what changed in each file,
    the cache decision,
    and "Checks on this tree:"
    with each count read from its log.

## Second: the whole-suite census

With the suite green and the tree committed,
take the baseline every T8 batch reads against:

```sh
# from the worktree root
mise run //package/module/translation-repair:coverage-census > "${HOME}/temp/agent/mimo-trial/census-base.log" 2>&1
```

Moved on 2026-10-18 from `//package/module/translation-repair:coverage-census`
to `//package/translation-repair/coverage-census:coverage-census`.

- It refuses a failing suite.
  A census taken with uncommitted changes in the package says so on its first line,
  and a later reading refuses it as a baseline,
  so commit first.
- It writes about 8 GB of coverage under `~/.cache/translation-repair/coverage`,
  deleted once read (the task's description);
  check free disk first.
- Its last lines name the census:
  `census written to <dir>/census.json`.
  Write that path and the counts on the `library source:` line under "Trial log".

Rank the clusters.
A cluster is the files of one directory whose names share their first word,
so `page-headings.ts` and `page-name-glossary.ts` are cluster `page`,
and `src/corpus-run/archive-casing-restore.ts` is in `corpus-run/archive`.
This command (tested on 2026-10-03 against a census log)
reads the report's "library source by cold lines" rows and prints the 15 largest clusters,
by stretches,
then lines:

```sh
# from the worktree root; the last argument is the census log
node --input-type=module -e "
import { readFileSync } from 'node:fs';
const lines = readFileSync(process.argv[1], 'utf8').split('\n');
const from = lines.indexOf('library source by cold lines:');
const to = lines.indexOf('functions never called in package source, outermost:');
const clusters = new Map();
for (const row of lines.slice(from + 1, to)) {
  const match = /^  (src\/(?:.*\/)?)([^/-]+)[^/]*\.ts: (\d+) stretch(?:es)?, (\d+) lines?/.exec(row);
  if (match === null) throw new Error('unread row: ' + row);
  const key = match[1].slice(4) + match[2];
  const sum = clusters.get(key) ?? { stretches: 0, lines: 0, files: 0 };
  clusters.set(key, { stretches: sum.stretches + Number(match[3]), lines: sum.lines + Number(match[4]), files: sum.files + 1 });
}
const ranked = [...clusters].sort((a, b) => (b[1].stretches - a[1].stretches) || (b[1].lines - a[1].lines));
for (const [key, sum] of ranked.slice(0, 15)) console.log(key, sum.stretches, 'stretches', sum.lines, 'lines', sum.files, 'files');
" "${HOME}/temp/agent/mimo-trial/census-base.log"
```

List one cluster's stretches,
each with its lines and the function it is when the whole function never ran:

```sh
# from the worktree root; arguments: the census.json path, then the cluster
node --input-type=module -e "
import { readFileSync } from 'node:fs';
const [path, cluster] = process.argv.slice(1);
const census = JSON.parse(readFileSync(path, 'utf8'));
const prefix = 'src/' + cluster;
const inCluster = (source) => source === prefix + '.ts' || (source.startsWith(prefix + '-') && !source.slice(prefix.length + 1).includes('/'));
for (const s of census.stretches.filter((s) => inCluster(s.source))) console.log(s.source + ':' + s.startLine + '-' + s.endLine, s.name);
" <census.json> <cluster>
```

On 2026-10-03 the second command listed 52 stretches for one cluster
where the first command's row counts summed to the same 52.

## Third: T8 batches, one cluster at a time

Take the largest cluster.
A cluster over about 30 stretches is split by file,
one batch per group of files,
so a batch stays reviewable.
Before writing anything,
read how earlier batches recorded their work:
`git show --stat 23090a91d` (bundle 1)
and `git show --stat dc9f0449b` (the `page` batch).
Their messages are the model for yours.

For each stretch,
read its lines and the function around them,
then decide which of three it is.

1.  Reachable,
    and the code does the right thing.
    Add a case that runs it and checks what it does,
    in the module's own `*.unit.test.ts`;
    a module with no test file of its own and no export reaching the package index
    is cased through an exported caller,
    as the `page` batch did.
    Assert the result,
    never only that it ran;
    for a throw,
    give an input cast past its type and assert the error class.
    Fixtures are cat-themed inventions;
    no corpus text goes into a test.
2.  Reachable,
    and the code does the wrong thing.
    That is a defect:
    take the next B number,
    commit a red case first,
    `test(module-translation-repair): B<n> red, <what goes wrong>`,
    with its `[FAIL]` line read from a log,
    then the fix,
    `fix(module-translation-repair): B<n>, <what it does now>`,
    with its ledger entry and the cache decision of "First:
    fix B126" step 4.
    `804ae3e85` and `ca242a2c4` are a model pair.
3.  Unreachable.
    Name what makes it so:
    the check in the same function that returns first,
    or what every caller guarantees.
    Then replace a fallback no input reaches with `nonNullishOrThrow` from `@monochromatic-dev/module-or-throw`,
    or a throw whose message starts `unreachable:`,
    or remove the dead code;
    never write a case that pins it.
    A claim that something is unreachable rests on reading or a probe,
    never on hope:
    `23090a91d` rejected an agent's "unreachable" after a probe reached it.

Every stretch ends as one of the three.
There is no fourth outcome:
T8 closes only when every library stretch has a case or is removed
(ledger T8;
the entry moved on 2026-10-23 to the [areas file][ledger-areas] of the ledger,
under "Test suite").
A stretch that resists placement gets a probe on the built package,
as `23090a91d` did,
until it is placed.
A reachable arm that needs a failure the host will not produce on demand is still cased,
through a parameter the function already takes,
or the narrowest one added so a test can supply that failure.

Close the batch in this order:

1.  Commit the cases and any source changes as soon as they pass,
    `test(module-translation-repair): T8 <cluster>, <what the cases pin>`,
    the body listing per file which stretch each case is written for,
    what was found unreachable and why,
    and what was left.
    Until the reach census of step 2 shows a stretch ran,
    a message says which stretch a case is written for,
    never that it reaches it (ledger M102).
2.  Prove the batch's reach with a census of its own test files against the baseline,
    one `--source` per source it claims:

    ```sh
    # from the worktree root, with every change committed
    mise run //package/module/translation-repair:coverage-census -- --baseline <census.json> --source src/<a>.ts --source src/<b>.ts src/<a>.unit.test.ts > "${HOME}/temp/agent/mimo-trial/reach-<cluster>.log" 2>&1
    ```

    Moved on 2026-10-18 from `//package/module/translation-repair:coverage-census`
    to `//package/translation-repair/coverage-census:coverage-census`.

    Its `against <census.json> at <commit>: ...` line decides it
    (format measured on 2026-10-03):
    `still cold 0`,
    `cold since then 0`
    and `not loaded 0`.
    Each count above zero is followed by lines naming the stretches;
    case each one,
    or prove it unreachable and remove it,
    commit,
    and run it again until all three read 0.
    A source the batch edited is read as a whole:
    its line must say this run loaded it and left nothing cold.
3.  Run the full suite (`buildAndTest`),
    `[FAIL]` 0,
    then the lint and the source scans,
    each result line read.
4.  Record the batch in one docs commit,
    `docs(module-translation-repair): record T8 <cluster>`:
    a paragraph at the end of the T8 entry of the [audit ledger][ledger]
    (the T8 entry moved on 2026-10-23 to the [areas file][ledger-areas] of the ledger)
    naming the cluster,
    the commits,
    the census reading
    and any B entries found,
    and one line under "Trial log" in this file.
5.  Take a whole-suite census at that commit as the next batch's baseline,
    and name it under "Trial log"
    (the practice the snapshot's 2026-09-29 checkpoint records for every batch).

Then the next cluster.

## Fourth: B123, the unpositioned link literal

Take it after the first T8 batch has landed.
The ledger's B123 entry has the cause,
the probe inputs and their results.
Write the red case through `applyFootnoteRelabel` first,
then design the fix,
build it,
and land it on the steps of "First:
fix B126".
Settle the design as "Your authority and the owner's choices" says:
by the stated rules and measurement,
and where two designs still differ by preference alone,
by the quality of the shipped page,
recorded under "Decisions for the owner to review".
Then back to the next T8 cluster.

## Commands

Run each from the worktree root,
redirecting long output to a log file under `${HOME}/temp/agent/mimo-trial/`,
then read the result line named.

- Build:
  `mise run //package/module/translation-repair:build` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:build`)
  (measured;
  rolldown prints `Finished` at the end).
- Build and full suite:
  `mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:buildAndTest`)
  (measured;
  read the `[PASS]` and `[FAIL]` counts).
- Named test files,
  after a build:
  `mise run //package/module/translation-repair:test:unit src/<a>.unit.test.ts src/<b>.unit.test.ts` (moved on
  2026-10-18 to `//package/translation-repair/<package>:test:unit`)
  (measured).
- Lint:
  `mise run //package/module/translation-repair:lint` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:lint`),
  whose result line is `Found 0 warnings and 0 errors.`
  (from the owner's commit messages;
  the cloud copy could not build the lint's own config package,
  so it was not run there).
- Types:
  `mise run //package/module/translation-repair:lint:types` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:lint:types`)
  after any TypeScript edit (`AGENTS.md` rule CM6;
  not run in the cloud copy).
- Source scans:
  `mise run //package/module/translation-repair:source-scans` (moved on 2026-10-18 to
  `//package/translation-repair/source-scan:source-scans`)
  for every commit touching the package's source,
  docs,
  README or task file,
  since several scans read the docs;
  35 `[PASS]` lines and no `[FAIL]` on `98e94e584` (measured).
- Markdown lint,
  for any Markdown you edit:
  `mise run lint:markdown <file> <file>`
  (measured;
  it prints nothing beyond its command line when clean).
- Coverage census:
  "Second:
  the whole-suite census" and "Third:
  T8 batches,
  one cluster at a time".
- Cache versions before any launch:
  `mise run //package/module/translation-repair:cache-account-audit` (moved on 2026-10-18 to
  `//package/translation-repair/slice-cache:cache-account-audit`)
  (the prevention doc's "Before a run launches").

## Rules from `AGENTS.md` that bite in this package

`AGENTS.md` at the repository root holds every rule;
these are the ones the code here trips most.

- TSDoc on every declaration,
  locals included (TSD),
  directly before it (TD3).
  Match the blank-line form a file already uses inside its TSDoc blocks (ledger M103).
- `nonNullishOrThrow`,
  never `!` (PP5).
  No `switch` (PP9).
  `async`/`await` only (PP1).
- Text is ordered with `compareCodePoints` or `textsInCodePointOrder` from `code-points.ts`,
  never `localeCompare` or a bare `sort()`
  ([`mistake-prevention.md`][prevent],
  "Text by code point").
- No new regular expression without the justification comment of rule RG3.
- Tagged loggers,
  never raw `console` in library code (TLG).
- Never raise or disable the max-lines limit;
  split the file (MXL).
- Commit messages:
  Conventional Commits with scope `module-translation-repair` (GCG),
  never a `#` before a number (TID),
  never `Closes` with an issue.
- Markdown:
  a line break at each clause,
  no tables,
  no dashes as punctuation,
  `-` for bullets,
  numbered items padded as `1.  `,
  and no reference by position;
  name the heading or the file instead (WR2,
  WR5,
  MD1 to MD8).

## Never

- Never launch a corpus pass while T8 is open,
  unless the owner says so,
  as "Your authority and the owner's choices" explains.
- Never print an API key,
  read `/proc/<pid>/environ`,
  or set `thinking`,
  `budget_tokens` or `reasoning_effort`
  (the snapshot's "Standing constraints").
- Never push to `translation-repair-rebased`,
  force-push,
  amend,
  rebase
  or reset a commit that exists.
- Never edit a dated section of an older handover or a closed ledger entry,
  except to add a correction that names itself as one.
- Never trust that a script named in an older doc still exists:
  `~/temp/agent` is wiped at any time,
  so check with `ls` and write your own when it is gone.

## Where the rest lives

- [The audit ledger][ledger]:
  every finding;
  T8's entry has the census method,
  its history and every batch
  (the entry moved on 2026-10-23 to the [areas file][ledger-areas] of the ledger,
  under "Test suite").
- [`mistake-prevention.md`][prevent]:
  each family of mistakes,
  its rule,
  and "Before a run launches".
- [The package README](../../package/translation-repair/README.md),
  "Where the rest lives":
  configuration,
  roster,
  providers,
  seats.
- [The 2026-09-06 snapshot](translation-repair-handover-2026-09-06.md):
  the operating history,
  "Repository state",
  "How a pass is launched now"
  and "Standing constraints".
- [The corpus-pass runbook](../runbook/translation-repair-corpus-pass.md),
  for the launch this trial does not make.

## Outcome: merged on 2026-10-05

Claude Code read the trial branch on 2026-10-05 (UTC),
on the owner's instruction to build on it by merging or to discard it,
and merged it:
`translation-repair-rebased` moved from `28303c42a` to `7f069adc9` by fast-forward,
all 303 commits,
and was pushed.
The trial checkout and its branch are left in place for the owner to remove.
This section records that decision and stays open to the owner's veto.

### What the decision rests on

Measured on the trial tip `7f069adc9`,
and again in the main checkout once it was merged:

- the whole suite printed 1,564 `[PASS]` lines and no `[FAIL]`;
- the lint ended `Found 0 warnings and 0 errors.`;
- all 35 source scans passed;
- the Markdown lint found nothing in `audit-ledger.md`,
  `mistake-prevention.md` or this file.

Read by hand:

- the whole production diff
  (63 files,
  776 lines added and 379 removed);
- every test hunk that removed a line:
  no expectation was changed to fit new code and no case was removed;
- the share of weak assertion shapes in the added cases
  (24 `toContain` in 396 added expectations,
  against 1,652 in 14,703 across the package when the trial began).

### What this file got wrong

"Third:
T8 batches,
one cluster at a time" told the trial that every census stretch ends cased,
unreachable and removed,
or a fixed defect,
and to take the census again until its counts read zero.
A guard on a state no input produces must stay and can never run,
so that rule had no place for it,
and the trial took guards out,
flattened named errors to `nonNullishOrThrow`,
and added parameters to production functions so cases could force a guarded state.
The package ledger records each one and what replaced it:
`M112` for the first removal and the rule it left,
`M113` for the family,
and `B127` for the one removal that changed a result.
The corrected rule is in `mistake-prevention.md`,
"Guards a census wants gone".
The rest of the trial's production diff and its added cases are being read against that rule as this is written;
the ledger carries what that reading finds.

By the end of 2026-10-05 (UTC) that reading had reached ledger entries `B128` to `B167`,
with `M113` extended for each removal read against its callers.
Still being read then:
the trial's edits to the OpenRouter client,
the rendering audit's screen and corroboration,
`preservation-tokens.ts`,
`retry-stated-wait.ts`,
`source-only-breaks.ts`,
`stage-fanout-window.ts` and the export surface;
and the reviewers' findings on test files no fix had yet touched.

By 2026-10-06 (UTC) that reading was complete.
The modules on that list are read in ledger `B176` to `B179`,
with the trial's production edits fixed in `6d60ce1a2`
and its cases on the audit and the OpenRouter client in `96ef8323e`.
The reviewers' findings on the trial's added cases were cross-checked one by one:
of 173,
155 had been applied by earlier fixes,
12 were ruled out with reasons,
and the last 6 were fixed in `e1c84094e` (ledger `B331` to `B335`).

### Choices the trial made that the owner may veto

- The `gitOutput` catch in `corpus-source.ts` wrapped every throwable as `CorpusReadError`
  (`33fc7a459`,
  recorded under "Decisions for the owner to review").
  Reversed on 2026-10-05 in `ba243b3c3`:
  the catch wraps a subprocess failure alone again,
  since the trial's evidence for the wider wrap was incomplete (ledger `B156`).
  The reversal is as open to veto as the wrap was.
- Parameters added to production functions so a case could reach a branch:
  `fold` on `anchorLocatedSpan`,
  `needle` on `deleteOneSentence`,
  `anchor` on `insertBorrowedSentence`,
  and `ocrTool` on `readImageWithOcr`.
  The first three are removed again (`fa86b527d`);
  the fourth gave way to a required program runner in `d550bf04f` (ledger `M113`,
  `B147`).

## Trial log

Newest first.
One line per step or batch:
the date and UTC time,
what landed with its commit,
the counts read from the logs,
and the step in progress.
A batch also writes one line before its first commit,
and after any correction or decision inside it,
naming the uncommitted work and the reasoning behind it,
so a fresh context resumes from this file alone.

Filed in:

- [Translation repair MiMo trial: log of 2026-10-05, the coverage attributions re-verified](translation-repair-mimo-trial-2026-10-03-log-2026-10-05-attribution-reverification.md)
- [Translation repair MiMo trial: log from 2026-10-04 20:45 UTC to 2026-10-05 01:20 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-2045-to-2026-10-05-0120-t8-batches.md)
- [Translation repair MiMo trial: log of 2026-10-04 from 14:42 to 20:37 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-1442-to-2037-t8-batches.md)
- [Translation repair MiMo trial: log of 2026-10-04 from 06:30 to 14:34 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-0630-to-1434-t8-batches.md)
- [Translation repair MiMo trial: log of 2026-10-04 from 02:42 to 06:05 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-0242-to-0605-t8-batches.md)
- [Translation repair MiMo trial: log of the B123 fix, 2026-10-04](translation-repair-mimo-trial-2026-10-03-log-2026-10-04-b123-fix.md)
- [Translation repair MiMo trial: log from 2026-10-03 18:20 UTC to 2026-10-04 01:13 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-03-1820-to-2026-10-04-0113-first-t8-batches.md)
- [Translation repair MiMo trial: log of 2026-10-03 from 16:20 to 18:14 UTC](translation-repair-mimo-trial-2026-10-03-log-2026-10-03-setup-and-b126-fix.md)

## Decisions for the owner to review

Each entry is a choice already made and acted on,
never an open question:
the options,
the evidence,
the one taken,
why,
and its commit.

- 2026-10-04,
  the `gitOutput` catch's rethrow (`33fc7a459`):
  options were keeping the rethrow,
  with the T8 stretch staying open on it,
  or wrapping every throwable as the sibling `readCorpusBytes` catch
  already does.
  Evidence:
  nano-spawn funnels every throwable through its `getErrorInstance`
  and wraps it as `SubprocessError`
  (`source/result.js`,
  `source/spawn.js`;
  its one raw escape needs `options.input`,
  which the corpus call never sends),
  and a probe on the built package delivered a `CorpusReadError` of kind
  `other` for a NUL-byte `gitPath`,
  an empty one and a missing binary
  (`~/temp/agent/mimo-trial/corpus-rethrow-probe.mjs`).
  Taken:
  the wrap,
  since the `kind` field still distinguishes `missing-object` from
  `other`,
  a catcher reads one failure class either way,
  and T8 has no fourth outcome for a stretch.
  The owner may veto and restore the rethrow,
  which returns the stretch to T8's open count.

[ledger]: ../../package/translation-repair/doc/audit-ledger.md
[ledger-areas]: ../../package/translation-repair/doc/audit-ledger-areas-of-the-audit-and-the-files-filed-under-each.md
[prevent]: ../../package/translation-repair/doc/mistake-prevention.md
