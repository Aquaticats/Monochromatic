# Audit ledger: the editor bench's prompt reuse, the re-check fixes and the bench outcome

The follow-ups of B899 in rounds 11 to 15:
repeat passes of the editor fidelity bench were answered from the prompt reuse layer,
each reuse was booked at its spend ceiling,
and the read step scored the reuses as trials;
the findings of the re-checks of rounds 12 to 14;
and,
in round 15,
the outcome of the bench itself.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
It is a new file of 2026-10-25,
named for its subject,
because [the exact repeat test file](audit-ledger-editor-bench-exact-repeat-test-and-self-contained-runbook.md)
holds round 10 and its sections are about the rule and the runbook.

## B899, round 11: repeat passes asked the model nothing, and the spend that was never spent

Status:
fixed on 2026-10-25 (UTC) in the commits the section "The round 11 commits" names,
open to the owner's veto.

No new id:
these are follow-ups to B899,
and the free ids from B870 to B899 were left alone because nothing here needed one.
The bench is the same bench,
and its three runs of B899 are the thing that failed.

### The defect

The lead measured it on the billed high arm
(log `editor-fidelity-bench-dd4b15b75c/high.log`,
`--passes 3` over 88 trials):

- Pass 1 made 88 real calls and spent 0.623685 USD.
- From row 89 (pass 2) every ask was answered by a `PROMPT-REUSE source=memory` line
  (`stage/src/prompt-uniqueness-client.ts`):
  the identical prompt returned pass 1's reply in 0 ms.
- Each reuse still wrote `SPEND ... completion=32000 cost=0.37... estimated=unreported-bound`.
  62 reuses booked 23.31 USD that was never spent,
  the `GUARD total` reached 23.938613 of the 24 USD cap,
  and the arm stopped at 150 of 264 units.
- The 62 reuse rows were scored as trials (`TRIALS completed 150`),
  and the read step of the build before this round passed every check on that log
  (`PASS high: row count: 150 rows logged, each once`).

So the repeat of a trial compared the model's answer with itself,
the bench's repeat agreement was 100 percent by construction,
and the money counted against the cap was a number the guard invented.
Neither the guard's tests nor the reuse layer's tests were wrong on their own terms.
The guard booked at the ceiling whenever a reply carried usage and the meter had not moved,
because that is what an unmetered provider looks like,
and a reuse looks exactly like that.
No test put the two layers together over a repeated prompt.

### Repeat passes ask the model afresh (ruling 1)

The seam chosen puts the pass in the request identity.

- `ChatTextRequest` (`provider/src/chat-contract.ts`) has an optional `reuseScope`,
  which is never put on the wire:
  the Claude body is built from named fields,
  and `reaskElsewhereNudged` spreads the request,
  so the scope reaches the reuse layer through a re-ask.
- `modelPromptDigest` (`stage/src/prompt-uniqueness-client.ts`) includes the scope only when it is present,
  so every digest of a corpus run is what it was
  (a test pins the digest of the shared test request at
  `5d94598d27a56145bd58cd6eb45cee47ccc3928da84613ae69cf53b6c4ab70aa`).
- `scopedToPass` (`probe/src/corpus-run/editor-fidelity-pass-scope.ts`) wraps the client a row asks through
  and names the pass,
  counted from one:
  `pass 1`,
  `pass 2`,
  and so on.
  The probe run wraps the client of each pair with it.

The reuse layer then still reuses within a pass and across the retries of one ask,
and still serves a corpus run,
and a repeat pass has a different identity from the first and so reaches the model.

The seam rejected:
dropping the reuse layer from the probe's client stack.
It would need an option through three builders to ask through another client than the one a run is asked through,
and the payload directory of an arm would hold no reply for audit.
It would also make the probe differ from what it measures.

Whether disk reuse can cross arms or bench directories:
it cannot.
The payload store lives at `runDir/payloads`,
and `runDir` is a fresh `mkdtemp` of every invocation (`probe/src/corpus-run/editor-fidelity-probe-run.ts`,
the runs directory resolved at `run/src/corpus-run/run-config.ts`),
so a second arm or a rerun of an arm reads a new empty store.
`source=disk` reuse needs a prior run directory of the same invocation,
which a probe never has.
No further change was needed.

### A reuse books no spend (ruling 2)

The source of the booking is `guarded()` in `probe/src/corpus-run/editor-fidelity-spend-guard.ts`:
when the reply had usage and the meter had not moved,
it booked `unreported-bound`.
The fix happens there:
when the inner client is one of the probe's builders and no attempt was told to the guard
(`told.attempts === 0`) for a reply that came back,
the guard books nothing,
writes no `SPEND` line,
and creates a `RepliedWithoutCallError`.
The run meter is never touched by it,
because the meter is written only by clients that send an attempt.

The corpus runs book no reuse either.
The only writer of the run meter is `reportSpend`
(`provider/src/spend-line.ts`,
which calls `noteRunSpend`),
and its callers are the provider clients and the Bedrock bound ledger,
each inside a sent call,
and the probe's guard for its ceiling-bound line.
The reuse layer calls none of them.
A pin in `stage/src/prompt-uniqueness-client.unit.test.ts` runs a prompt twice through the real layer
and sees the run meter stay where the first reply left it,
with a positive control:
a different scope reaches the inner client and moves the meter to 0.2.
That test passes at its parent commit and has no red.

### The probe refuses to score a row that made no call (ruling 3)

`RepliedWithoutCallError` has a fixed message that names `PROMPT-REUSE` and no input (`messageNamesOnly`).
It is pushed to the guard's endings,
aborts the arm's controller with it,
and is thrown to the stage.
The stage rethrows once the signal is aborted,
and the walk rethrows any abort reason that is not a stated refusal,
so the arm ends with the error,
writes no row for the ask,
prints no `TRIALS` line,
and still prints the `GUARD total` line through `using guardSaid`.
The class was added to the marked classes of `source-scan/src/message-names-only.unit.test.ts`.

### The read step, the stop rule and the README count what made a call (ruling 4)

- `reusedRowFlags` in `probe/src/corpus-run/editor-fidelity-arm-facts.ts`:
  a row follows a `PROMPT-REUSE` body written since the row before it.
  `ArmFacts.reusedRows` and `ArmFigures.reusedRows` hold the count,
  and `ArmFigures.rows` holds only the rows that made a call,
  so every comparison,
  the units the arms share,
  the stops and the repeat reading are over those rows.
- The row check fails an arm that has any:
  `N of M rows were answered by prompt reuse and made no call, so none of them is a trial`.
  The billed log now fails it with 62 of 150 rows,
  its stop and unit figures count the 88 rows that made a call,
  and it prints no repeat reading of its replays.
  The command prints no `SHARED` line for it,
  since a log whose checks fail prints no figures,
  and says on the unit check that 0 shared units remain of the 88 the first arm completed.
- The README's launch block,
  `n` block,
  stop rule and pre-launch block take the `PROMPT-REUSE source=` lines of the high log
  off the number the `TRIALS` line gives.
  The build of this round writes none (an arm that gets one ends),
  so the number taken off is 0 for its logs and exact for a log of an earlier build.
  The pre-launch block takes the spend of the arm from the `GUARD total=` line when the log has no `PROMPT-REUSE` line,
  since a log of the build of this round books no reuse.
  For a log with such lines,
  which is a log of the billed build,
  it sums the `SPEND` lines without `estimated=`,
  because the guard total of the billed log holds 23.314928 USD of ceilings that were never spent:
  it reads 0.0071 USD a unit there,
  where the guard total gives 0.27 and the block said not to launch.
  That subtraction is right only while the `estimated=` lines are the reuses,
  so the block refuses a log where their count differs from the count of reuse lines.
  The first form of this block (round 12) left out every `estimated=` line for every log,
  which understates the spend of a log of the new build,
  whose `estimated=` lines are the `abandoned-bound` lines of attempts the client saw cut
  and the guard's own `unreported-bound` and `abandoned-bound` lines for calls that may have been billed
  (re-check 9,
  minor 1).
  The `n` block and the launch block refuse a log with no `TRIALS` line,
  where the arithmetic had read the missing number as 0 and the launch block would have passed `--trials 0`.
  Each changed block was run in a fresh `bash --noprofile --norc` shell on constructed logs.

### The six nits of re-check 7

1. The 0.0553 reading is in the section "The 0.0553 reading corrected".
2. The TSDoc of `repeatLinesOf` says an exact test finds the arm under another,
   at the level the warning prints.
3. `REPEAT_ALPHA` and its denominator in whole numbers state one fact twice:
   the TSDoc says so and a case holds the two together.
4. The README no longer cites `dryrun-r7.txt`.
5. The counts and the wording "rounds 8 and 9" of the round 10 file.
6. The README gives the correlation 0.5 figures
   (3,
   0 and 0 of 6000 at 150 units)
   and says that the 11 benches are 11 of 80000,
   four of them at the true rate 0.90.

### The 0.0553 reading corrected

Round 10 called the cell 1.5 standard deviations over 0.05 and within noise.
The cell is a fix rate of 0.9,
a repeat agreement of 0,
a correlation of 1 and 176 units.
Re-check 7 ran the same cell at 30000 replicates (seed prefix `edrc7`):
0.0535 at 176 units and 0.0488 at 264.
Pooled with the 4000 of round 10 it reads 0.0537 over 34000 replicates at 176 units,
about three standard deviations over 0.05.
So the false alarm rate of the exact repeat warning is about 0.054 in that cell,
where the trials of one entry read as alike as the model allows and are not independent draws.
Three contrast cells at 176 units read under 0.05:
0.0296 (fix rate 0.9,
agreement 0,
correlation 0),
0.0430 (fix rate 0.7,
agreement 0,
correlation 1)
and 0.0379 (fix rate 0.9,
agreement 0.5,
correlation 1).
The README and `probe/doc/editor-fidelity-decision-rule-simulations.md` say so,
and the noise reading is gone.
The frozen rule text states the bound for independent draws and so stays true.

### The round 11 commits

Oldest first.
Each red commit fails for the reason given,
and the failing lines of each are in the round's scratch directory
(`editor-fidelity-r11/red*-failing-lines.txt`).

- Ruling 1,
  red `e21000f7d791396c7aa97d87dacf3174289ef38a`:
  4 failing lines for the stage cases (reuse in memory,
  reuse in the durable store and the digest pin),
  3 for the probe run through the real reuse layer and its payload store,
  3 for the real route of the client,
  and 1 for the pass scope file,
  which did not load.
  Fix `3b2294553994855ee731f81da512d486723af337`.
- Rulings 2 and 3,
  red `43c5203919ae5c13e1364b0757c31f682c90db3a`:
  4 failing lines for the guard and 3 for the probe run.
  Fix `394d1e651a379359f84d9c9dffa0a583f6f94c59`.
- The meter pin,
  no red:
  `baef63bf1cc4df5981994e33fe0bd0afd77b9dee`.
- Ruling 4,
  red `9c8974d3a4a74353645e810d53cf62a495332109` with 5 failing lines.
  Fix `bbfb98d8ca8a44e462a76226b52a86d3f07496ff`.
- Nits 2 and 3:
  `fe4e278399a8b8007896c61ad97aba963ac35fdd`.
  The pinning case passes at the parent commit,
  because it pins behavior the TSDoc change keeps.
- The README and the simulations doc,
  nits 1,
  4 and 6,
  and the runbook blocks:
  `94ba1c2e46e448fef2605d0741f2bfd2f688be5a`.
- A comment in place of a TSDoc before a return in a test:
  `3b209834c67a5eaa6430204cb230a3b5265c96a4`.
- The ledger,
  the prevention line and the corrected figure:
  `ab17dcbf7478a6776561fb9d2924d2b9a3ca5ba3`.
- Lines broken for the Markdown linter,
  and the README brought back under its bound of 1000 lines:
  `1dba616f89e67b71c1ba18e054caed921021188b`,
  `fe66608549b2c300e0c9067fb108988d23541bff`,
  `34587581a2c4a7c493cff2d1caf6e18add6df006`
  and `55510ed7bea2df768a4faf038b36bbc06becbb50`.
- The suite of the meter pin run one case at a time:
  `564d165033e61cab8d136fbf774eabfb4efd86f9`.

The process globals scan (ledger B79) failed from `baef63bf1cc4df5981994e33fe0bd0afd77b9dee`
to `55510ed7bea2df768a4faf038b36bbc06becbb50`.
The meter pin put a case that diverts `console.info` into a suite that runs its cases beside each other,
and `564d165033e61cab8d136fbf774eabfb4efd86f9` fixed it with `concurrency: 1` on that suite.
The eleven commits between them fail that scan,
so a bisect through them meets a failure that is not the one searched for.
The message of `1dba616f89e67b71c1ba18e054caed921021188b` called that failure one the round did not touch,
and that was wrong.

### The round 11 cache statement

`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' dd4b15b75c..HEAD -- '*.ts'` lists no commit.
The same search over `c265e8df95~1..c265e8df95` lists `c265e8df9525f4513b8229482d3e8d932ea630df`,
so the search can show a hit.

This round adds an optional request field that is never sent
and that changes no digest of a request without it.
It changes the probe's guard,
run and read step,
and changes no reply,
key or cache record.
No cache version moves.

## Round 12: the findings of re-check 8

The independent re-check of round 11 (`editor-recheck8.md`) found no major,
two minor and seven nit,
and each was ruled a fix.

- Minor 1:
  the `n` block printed 0 for a log with no `TRIALS` line,
  not the empty line the README said,
  and the launch block would have passed `--trials 0`.
  Both blocks now print a message on standard error and exit 1.
  Each was run in a fresh shell on a dry run's log and on the billed `high.log`
  (`n` gives 88 on the billed log and refuses the dry run;
  the medium launch got `--trials 88` and `--bench-spent-usd 23.94` on the billed log through a stub of `systemd-run`,
  and no launch on the dry run).
- Minor 2:
  the pre-launch block read 0.2720 USD a unit from the billed log;
  it reads 0.0071 now,
  from the `SPEND` lines without `estimated=`.
- Nit 1:
  the text of the read step no longer says it shares 88 units.
- Nit 2:
  the corrective comment on `1dba616f89e67b71c1ba18e054caed921021188b` is posted by the lead.
- Nit 3:
  the commit list and the break of the scan B79 are in the section on the round 11 commits.
  The fixes of this round broke the same scan a second time
  (see the section on the second break of that scan in round 13).
- Nit 4:
  the pointer by position names its heading.
- Nit 5:
  `isReuseBody` finds the reuse line behind any number of tags.
  The red commit is `7854e18682b5d8c71f9f3d182e78a887b11a3c90` with 2 failing lines for its case of tags,
  the fix `c3f7f4a6343ef7399494071156524929588f9d09`,
  which also removed an article from the `@param bodies` text of `reusedRowFlags`.
  Its second case takes the line from the stage's own writer and passes at the parent.
- Nit 6:
  the articles of the `@param`,
  `@returns` and `@throws` text this work wrote are gone
  (`d1372493f7369a1efb5c3ef5c00d70ed71398e81`,
  and the `@param bodies` text in `c3f7f4a6343ef7399494071156524929588f9d09`).
- Nit 7:
  see the section on the limit of the reuse check.

The README went back under its bound by moving its paragraph on the smallest difference at 60 shared units
to the simulations doc
and by cutting its restatement of the readings above 0.90,
which the rule's text and the doc hold.

### The limit of the reuse check at the guard

The guard can end an arm on a reply with no request behind it only for a client built over its admit,
since only that client tells it of each attempt.
A client given as itself tells the guard nothing,
and a reply of the reuse layer then looks like a reply whose provider wrote no line on the meter,
so the guard counts it at the ceiling as an unreported cost.
Every construction in production passes a builder (`probe-run.ts`),
so the case does not arise in a run,
and the TSDoc of `spendGuardedClient` and the comment at the branch say so.

### Prevention of round 12

[Preventing mistakes: spend guards and the attempts a client makes][prevention]
holds the rule:
a repeat measurement proves that each repeat made its own call.

## Round 13: the findings of re-check 9

The independent re-check of round 12 (`editor-recheck9.md`) found no major,
one minor and six nit,
and each was ruled a fix.
The build the high arm runs is the frozen copy of `85e21653a2ebb055ee807a6edc2fdf1248073abc`
(the section on round 14 says how that copy is named),
and the code changes of this round are in the read step only:
`unusableWarning` and the TSDoc of `isReuseBody`.
The built closure of the entry the arms run (`editor-fidelity-probe`) is the same in a build of that commit
and in a build of this round's head,
so the frozen copy runs what the head runs
(the section on round 14 holds the measurement;
this paragraph first said the changed code was not reachable from the entry,
which is false of the file that holds `isReuseBody`).

- Minor 1:
  the pre-launch block's subtraction of the `estimated=` lines is right only for the billed log.
  The README and the section on the read step,
  the stop rule and the README now say so,
  and the block picks the `GUARD total=` for a log with no `PROMPT-REUSE` line.
  Each form was run in a fresh `bash --noprofile --norc` shell on four logs:
  the billed log
  (0.0071 USD a unit,
  88 units left,
  medium 2487 and low 2064),
  the billed log with three `estimated=abandoned-bound` lines added
  (refused:
  65 lines at a ceiling for 62 reuses),
  a constructed log of the new build with a guard total of 2.5 over 264 units
  (0.0095 USD a unit,
  medium 1861 and low 1545),
  and the same with three `abandoned-bound` lines of 0.322 USD added to its guard total
  (0.0131 USD a unit,
  medium 1343 and low 1114).
  The block on a dry run's log and the `GUARD total` branch are covered by the same runs.
- Nit 1:
  the stop rule block and the pre-launch block refuse a log with no `TRIALS` line like the `n` block
  (message on standard error,
  exit 1,
  nothing on standard output),
  run on a dry run's log and on the billed log (88 for the stop rule).
  The read step no longer prints `0 of -1 units` for such a log:
  `unusableWarning` reads the units as at least zero.
  Red `5fa864ffbdfdf99cdaf41747f7b506e1e8d9f12b` with 2 failing lines (the warning was written),
  fix `b84403bb13fa1c0de816558491dbafa21f6fa8d1`.
- Nit 2:
  a case for a body that opens with `[` and never closes it.
  A mutant that makes that branch of `isReuseBody` return true fails it with 2 failing lines,
  and the code passes it (`fd12b8a29342cd486193a9a7bd5c1dc2788b2ed3`).
- Nit 3:
  see the section on the second break of the scan.
- Nit 4:
  the README has 998 lines after round 12 and 997 after this one;
  the clause that the number taken off is exact for a log of an earlier build stays in the section on the read step,
  where it is true for the count of reuse lines and now carries the correction of the spend;
  the commit `c3f7f4a6343ef7399494071156524929588f9d09` is cited for the article it removed.
- Nit 5:
  the articles of the `@param` of `armWithReuseLine` and the `@returns` of `isReuseBody` are gone
  (`fd12b8a29342cd486193a9a7bd5c1dc2788b2ed3`).
- Nit 6:
  the limit section sits under round 12 and the prevention heading names its round;
  the prevention file holds the limit of the guard and the lesson of the scan.

The README went back under its bound of 1000 lines by moving the units-a-cap-buys paragraph to the simulations doc
and by shortening two paragraphs that restated the doc.

### The second break of the process globals scan

`7854e18682b5d8c71f9f3d182e78a887b11a3c90` added a suite that imports `infoLinesDuring`
from `typography/console-warn-lines.test-fixture.ts` and left its cases beside each other.
The scan failed at that commit and at each commit up to the merge `93a5887a70c3f35e260c36a27d7ba706cf4a1265`,
and `85e21653a2ebb055ee807a6edc2fdf1248073abc` fixed it with `concurrency: 1`.
The scan file `source-scan/src/global-writes-sequenced.unit.test.ts` was run at each of seven commits
in a detached worktree of its own:

- `564d165033e61cab8d136fbf774eabfb4efd86f9`:
  exit 0,
  no finding.
- `7854e18682b5d8c71f9f3d182e78a887b11a3c90`:
  exit 1,
  `console assigned, in a suite that runs cases beside it`.
- `c3f7f4a6343ef7399494071156524929588f9d09`,
  `d1372493f7369a1efb5c3ef5c00d70ed71398e81`,
  `22f2ca82dae29c016ea5d875e8f2cf997a57ed65`
  and the merge `93a5887a70c3f35e260c36a27d7ba706cf4a1265`:
  exit 1,
  the same finding.
- `85e21653a2ebb055ee807a6edc2fdf1248073abc`:
  exit 0.

Five commits fail the scan,
so a bisect through them meets a failure that is not the one searched for.
The messages of those commits do not say so.

### Prevention of round 13

[Preventing mistakes: spend guards and the attempts a client makes][prevention]
holds the limit of the guard on a client given as itself
and the lesson of the scan:
run the source scans before the first commit of a test that imports a fixture writer.

## Round 14: the findings of re-check 10

The independent re-check of round 13 (`editor-recheck10.md`) found no major,
no minor and six nit,
and each was ruled a fix.
The code commits,
oldest first:

- `c91ba55fac66b02d5e8c4acbe2d29a2163bcd086`:
  red for nit 4.
- `bda28a5e72564956941c3e9594ec7a241fd3daf3`:
  the fix for nit 4.
- `a7c9ce80db9b4af0f9b984cd85dc10928256ee4c`:
  the lines over 120 characters wrapped in the two TypeScript files this round touches.

Four commits change Markdown only:

- `88b731cd2a5f321014c12f921fd894596d7816d3`:
  joined short lists onto fewer lines
  to keep the README at its bound,
  and the Markdown lint scan of the source scans fails on a line that holds a comma before its end.
- `1ae50a58edbaf09c2d4e73a40d9878c53a0a9f15`:
  puts those lists back and moves the paragraph on the power of one pass against three
  to the simulations doc.
- `e71f5eb30bc8bcb50fa135bd8650a49aa43e8112`:
  ends the simulations doc with exactly one line feed.
- `eae31928ff0aff8b0d25c19b21505edfef47d7c2`:
  records that line feed in this ledger.

- Nit 1:
  the claim that the changed code is not reachable from the entry is withdrawn.
  `editor-fidelity-arm-facts.ts` holds `isReuseBody` and `reusedRowFlags`,
  and `editor-fidelity-decide.ts`,
  which the entry reaches,
  imports it;
  its change was a TSDoc line that the bundler strips.
  The claim now rests on the measured fact.
  Two scratch copies of the probe package were built with `rolldown` and the package's own config,
  one from the source of `85e21653a2ebb055ee807a6edc2fdf1248073abc`
  and one from `a7c9ce80db9b4af0f9b984cd85dc10928256ee4c`
  (the source of this round's code,
  since `88b731cd2a5f321014c12f921fd894596d7816d3`,
  `1ae50a58edbaf09c2d4e73a40d9878c53a0a9f15`,
  `e71f5eb30bc8bcb50fa135bd8650a49aa43e8112`
  and `eae31928ff0aff8b0d25c19b21505edfef47d7c2`
  change Markdown only).
  The import closure of the built entry `editor-fidelity-probe.mjs` is nine files in each,
  with the same names,
  and all nine are byte for byte the same.
  Neither `index.mjs` nor `editor-fidelity-read.mjs` is in the closure.
  Those two differ between the builds,
  and so does `index.d.mts`,
  which carries the TSDoc,
  and nothing else differs.
  A high arm run from a frozen build of `85e21653a2` therefore runs the entry code of this head.
- Nit 2:
  the README blocks read `SHA` from `git rev-parse --short=10 HEAD`.
  The prepare block of the high arm ran with the HEAD of the worktree the arm runs from at `85e21653a2`,
  so the frozen copy and the bench directory are named for `85e21653a2` with no rename,
  and the launcher pins `SHA=85e21653a2` so a later checkout cannot rename the directory.
  The README now tells the operator to write the prepare block's `SHA` value into the later blocks
  when `HEAD` may move between arms,
  in place of reading `HEAD` again.
- Nit 3:
  on a log with no `PROMPT-REUSE` line and no `GUARD total=` line,
  the pre-launch block died in `awk` with `division by zero` (exit 2).
  It now refuses with `no GUARD total= line in <log>, so the spend is unknown` on standard error and exits 1.
  Run in a fresh `bash --noprofile --norc` shell on the logs of re-check 10:
  the log with no guard line exits 1 with that message and nothing on standard output;
  the billed log gives 0.0071 USD a unit,
  88 units left,
  medium 2487 and low 2064;
  the constructed new-build log gives 0.0095 USD a unit,
  medium 1861 and low 1545;
  the dry run's log,
  the billed log with three `abandoned-bound` lines and the log with one reuse line still refuse as before.
- Nit 4:
  `unusableWarning` wrote `3 of -1 units, over 10 percent` for a log whose pooled `EDITOR` line states
  unusable and refused counts but no count of units.
  It now writes `unusable plus refused is 3 of an unknown number of units (the log states none)`,
  says nothing when none are unusable or refused,
  and keeps the share rule for a known count.
  The case that the red commit of round 13 (`5fa864ffbdfdf99cdaf41747f7b506e1e8d9f12b`) added
  asserts no warning when nothing is unusable or refused,
  and was named for warning of no units;
  it is renamed to what it asserts.
  The message of `c91ba55fac66b02d5e8c4acbe2d29a2163bcd086` says the fix commit of round 13,
  `b84403bb13fa1c0de816558491dbafa21f6fa8d1`,
  added that case,
  and the round 14 section of this ledger said it came with that commit.
  Both are wrong:
  `git log -S` on the test file lists `5fa864ffbdfdf99cdaf41747f7b506e1e8d9f12b`
  and `c91ba55fac66b02d5e8c4acbe2d29a2163bcd086` only,
  and `b84403bb13fa1c0de816558491dbafa21f6fa8d1` changed the source file alone.
  The new case failed at its parent after a rebuild (the tests read `dist`),
  with the actual line `is 5 of -1 units, over 10 percent`
  and 2 failing lines (the case and its suite),
  after its positive control (the whole log warns `5 of 40 units`) passed.
- Nit 5:
  the lines over 120 characters are wrapped in the files this round touches:
  the ledger (four lines,
  two of them links that became reference links),
  the README (two code lines of the pre-launch block),
  and the two TypeScript files.
  The README has 1000 lines,
  after the paragraph on the power of one pass against three moved to the simulations doc.
- Nit 6:
  the commit messages named by re-check 10 are listed in the fixer report of round 14 and nothing is posted.
- Final line feed:
  the push hook reported `final-newline/noncanonical-final-newline` for the simulations doc,
  which ended without a line feed.
  It ends with exactly one since `e71f5eb30bc8bcb50fa135bd8650a49aa43e8112`,
  and the README and this ledger were checked and already did.

## Round 15: the findings of re-check 11 and the outcome of the bench

The independent re-check of round 14 (`editor-recheck11.md`) found no major,
no minor and five nit,
and each was ruled a fix.
The bench had finished,
so this round also records what it found.
The branch merged `origin/main` at `62bcd1593bea80895a5fdd03d19d30a5cb22e52e`
in `39e06b9367ca18a929660b6bbbde3fd4f1788d78`,
with no conflict.
The commits of this round other than the one that holds this section,
oldest first:

- `b27c7bb93dc6d450086be48c8942a8058a3b8f0e`:
  the long lines from `main` wrapped.
- `18f6a3f7e44ea3831c94aaea195f2d681bd71008`:
  the `@returns` of `unusableWarning`.
- `b3d7e293b1f95652e3c6f5b851b75842fd5cf06e`:
  one clause of the decision record broken after its comma.
- `2e3a508520060aa38117d7d7e886b765bd4d86b3`:
  the inline links of `audit-ledger.md` and the retitled index entries.

### The outcome of the bench

The bench ran at `85e21653a2`,
and the read step over its three logs printed these figures.
[The outcome file](../probe/doc/editor-fidelity-bench-outcome.md) holds the same figures for a reader of the probe.

- The arms,
  each over 264 units of 88 trials in 3 passes:
  - High:
    clean fix 125 of 264 (insertion 52 of 150,
    alteration 73 of 114),
    defect only 133 (insertion 92,
    alteration 41).
  - Medium:
    clean fix 112 of 264 (insertion 42 of 150,
    alteration 70 of 114),
    defect only 143 (insertion 99,
    alteration 44).
  - Low:
    clean fix 112 of 264 (insertion 41 of 150,
    alteration 71 of 114),
    defect only 144 (insertion 101,
    alteration 43).
  - Every clean fix was exact (equivalent 0),
    and no arm had a unit unusable,
    refused,
    thrown or abandoned.
- Shared units:
  264 over 53 entries,
  none left out,
  none not completed by every arm.
- High against medium:
  +4.9 points (95 percent interval +0.7 to +9.8,
  seeded bootstrap over 53 entries),
  p 0.0288,
  shown better at 0.025:
  no.
  By kind (secondary,
  no veto):
  insertion +6.7 points over 150 units,
  alteration +2.6 over 114.
- Low against medium:
  +0.0 points (interval -2.8 to +2.7),
  p 0.6016,
  shown better at 0.025:
  no.
  By kind:
  insertion -0.7 points over 150 units,
  alteration +0.9 over 114.
- Repeat agreement,
  trials that gave the same reading in every pass:
  high 80 of 88 (91 percent),
  medium 81 of 88 (92 percent),
  low 83 of 88 (94 percent).
  The read step printed no warning line.
- Spend:
  high 1.860067 USD,
  medium 1.634552 USD,
  low 1.619022 USD,
  5.113641 USD in all,
  under the 60 USD allowance (the caps add to 57).
- Checks:
  49 pass,
  0 fail.
- The verdict line,
  broken here at its punctuation:
  > VERDICT medium (none-shown):
  > no effort was shown better than medium at 0.025,
  > so the default effort stays;
  > that does not show the others are as good

What it means:
medium stays the editor's effort,
and no code changes.
The high arm's +4.9 points came with p 0.0288 and missed the level of 0.025,
so high was not shown better,
and that does not show it is not better.
The low arm was not shown better either.

No second medium arm was run.
The rule places one between the medium arm and the low arm in run order,
and the low arm had run when the logs were read,
so a run after the read would have been chosen after seeing the results.

### The five nits

- Nit A:
  the ledger credited `b84403bb13fa1c0de816558491dbafa21f6fa8d1` with adding the case of round 13.
  `5fa864ffbdfdf99cdaf41747f7b506e1e8d9f12b` added it,
  and the section for round 14 now says so.
  The message of `c91ba55fac66b02d5e8c4acbe2d29a2163bcd086` carries the same slip and stands.
- Nit B:
  the section for round 14 named its Markdown commits as "the commits after those" and "the next commit".
  It now names `88b731cd2a5f321014c12f921fd894596d7816d3`,
  `1ae50a58edbaf09c2d4e73a40d9878c53a0a9f15`,
  `e71f5eb30bc8bcb50fa135bd8650a49aa43e8112`
  and `eae31928ff0aff8b0d25c19b21505edfef47d7c2`.
- Nit C:
  the `@returns` of `unusableWarning` carried articles and said what the function returns.
  It now says why the lines exist,
  without articles
  (`18f6a3f7e44ea3831c94aaea195f2d681bd71008`,
  TSDoc only).
- Nit D:
  `1ae50a58edbaf09c2d4e73a40d9878c53a0a9f15` moved the reason for taking one pass to the simulations doc
  and left the README with the recommendation alone.
  The README now gives the reason in a clause
  (a repeat that gives the first answer again adds no power),
  and still points at the section of the simulations doc that holds the figures.
- Nit E:
  the index entries for this file still called it the follow-up of round 11.
  Both indexes now say it holds rounds 11 to 15
  (`2e3a508520060aa38117d7d7e886b765bd4d86b3`).

### Room in the README

The README was at its bound of 1000 lines,
and it needed a pointer to the outcome file.
The bullet on the high fix rate warning ended with the benches that print neither warning,
which the simulations doc holds in full in its section on that warning:
0 of 4000 at 264 units,
1,
1 and 2 of 6000 at 150 units,
0 of 4000 and 3,
0 and 0 of 6000 at correlation 0.5,
and 11 of 80000 in all,
four of them at the true rate 0.90.
The README dropped that duplicate and keeps the pointer paragraph that names it.
No line was joined.

### The long lines from main

The DeepSeek work that landed on `main` left lines over 120 characters in three files:

- `doc/decision/translation-repair-direct-providers.md`:
  4 lines.
- `package/translation-repair/doc/audit-ledger-entries-by-id.md`:
  75 lines.
- `package/translation-repair/doc/audit-ledger.md`:
  54 lines.

They were wrapped in `b27c7bb93dc6d450086be48c8942a8058a3b8f0e` with no change of wording,
apart from one sentence that says how the two JSON result lines were broken.
Of the 133 lines,
129 were index link lines,
2 were prose and 2 were JSON.
That commit turned every long index link into a reference link.
The source scan of the doc line limit then failed on the index `audit-ledger.md`,
because it looks for `](` followed by each ledger file's name in that index's text,
so `2e3a508520060aa38117d7d7e886b765bd4d86b3` gives that file inline links again,
with the link text broken at a comma or a space and the target kept whole on the last line.
`audit-ledger-entries-by-id.md` keeps its reference links,
with the definitions at the end of the file.
`b3d7e293b1f95652e3c6f5b851b75842fd5cf06e` breaks a clause that `b27c7bb93dc6d450086be48c8942a8058a3b8f0e`
left running on past a comma in the decision record.

[prevention]: mistake-prevention-spend-guards-and-client-retries.md
