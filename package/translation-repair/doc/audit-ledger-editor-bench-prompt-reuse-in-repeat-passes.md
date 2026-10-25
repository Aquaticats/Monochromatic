# Audit ledger: the editor bench's prompt reuse in repeat passes

The follow-up of B899 in round 11:
repeat passes of the editor fidelity bench were answered from the prompt reuse layer,
each reuse was booked at its spend ceiling,
and the read step scored the reuses as trials.

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
The only writer of the run meter is `reportSpend` (`provider/src/spend-line.ts`, which calls `noteRunSpend`),
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

`RepliedWithoutCallError` (`messageNamesOnly`, so its message is a fixed sentence naming `PROMPT-REUSE`)
is pushed to the guard's endings,
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
  so every comparison, the units the arms share, the stops and the repeat reading are over those rows.
- The row check fails an arm that has any:
  `N of M rows were answered by prompt reuse and made no call, so none of them is a trial`.
  The billed log now fails it with 62 of 150, shares 88 units and reads no repeat of its replays.
- The README's launch, `n`, stop rule and pre-launch blocks take the `PROMPT-REUSE source=` lines of the high log
  off the number the `TRIALS` line gives.
  The build of this round writes none (an arm that gets one ends),
  so the number taken off is 0 for its logs and exact for a log of an earlier build.
  Each changed block was run in a fresh `bash --noprofile --norc` shell on constructed logs.

### The six nits of re-check 7

1. The 0.0553 reading, in the section "The 0.0553 reading, corrected".
2. The TSDoc of `repeatLinesOf` says an exact test finds the arm under another, at the level the warning prints.
3. `REPEAT_ALPHA` and its denominator in whole numbers state one fact twice:
   the TSDoc says so and a case holds the two together.
4. The README no longer cites `dryrun-r7.txt`.
5. The counts and the wording "rounds 8 and 9" of the round 10 file.
6. The README gives the correlation 0.5 figures (3, 0 and 0 of 6000 at 150 units)
   and says that the 11 benches are 11 of 80000, four of them at the true rate 0.90.

### The 0.0553 reading, corrected

Round 10 called the cell (fix rate 0.9, repeat agreement 0, correlation 1, 176 units) 1.5 standard deviations over 0.05
and within noise.
Re-check 7 ran the same cell at 30000 replicates (seed prefix `edrc7`):
0.0535 at 176 units and 0.0488 at 264.
Pooled with the 4000 of round 10 it reads 0.0537 over 34000 replicates at 176 units,
about three standard deviations over 0.05.
So the false alarm rate of the exact repeat warning is about 0.054 in that cell,
where the trials of one entry read as alike as the model allows and are not independent draws.
Three contrast cells at 176 units read under 0.05:
0.0296 (fix rate 0.9, agreement 0, correlation 0),
0.0430 (fix rate 0.7, agreement 0, correlation 1)
and 0.0379 (fix rate 0.9, agreement 0.5, correlation 1).
The README and `probe/doc/editor-fidelity-decision-rule-simulations.md` say so,
and the noise reading is gone.
The frozen rule text states the bound for independent draws and so stays true.

### The round 11 commits

Oldest first.
Each red commit fails for the reason given,
and the failing lines of each are in the round's scratch directory
(`editor-fidelity-r11/red*-failing-lines.txt`).

- Ruling 1, red `e21000f7d791396c7aa97d87dacf3174289ef38a`
  (stage: the reuse in memory and in the durable store and the digest pin, 4 failing lines;
  probe run: the end-to-end run through the real reuse layer and its payload store, 3;
  client: the real route, 3;
  the pass scope file did not load, 1),
  fix `3b2294553994855ee731f81da512d486723af337`.
- Rulings 2 and 3, red `43c5203919ae5c13e1364b0757c31f682c90db3a`
  (guard: 4 failing lines, probe run: 3),
  fix `394d1e651a379359f84d9c9dffa0a583f6f94c59`.
- The meter pin, no red: `baef63bf1cc4df5981994e33fe0bd0afd77b9dee`.
- Ruling 4, red `9c8974d3a4a74353645e810d53cf62a495332109` (5 failing lines),
  fix `bbfb98d8ca8a44e462a76226b52a86d3f07496ff`.
- Nits 2 and 3: `fe4e278399a8b8007896c61ad97aba963ac35fdd`
  (the pinning case passes at the parent commit; it pins behavior the TSDoc change keeps).
- The README and the simulations doc, nits 1, 4 and 6 and the runbook blocks:
  `94ba1c2e46e448fef2605d0741f2bfd2f688be5a`.

### The round 11 cache statement

`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' dd4b15b75c..HEAD -- '*.ts'` lists no commit.
The same search over `c265e8df95~1..c265e8df95` lists `c265e8df9525f4513b8229482d3e8d932ea630df`,
so the search can show a hit.

This round adds an optional request field that is never sent and that changes no digest of a request without it,
changes the probe's guard, run and read step,
and changes no reply, key or cache record.
No cache version moves.

### Prevention

[Preventing mistakes: spend guards and the attempts a client makes](mistake-prevention-spend-guards-and-client-retries.md)
holds the rule:
a repeat measurement proves that each repeat made its own call.
