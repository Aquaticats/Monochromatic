# Audit ledger: the editor bench's paired rule, its shared trials and the guard's limits

The decision rule that replaced a floor with a paired exact test per effort,
the pool that holds every trial the pinned clone offers,
the read step that walks three or four logs by trial and absorbs a failed trial in the trial and not the arm,
the spend guard's one-call-at-a-time assertion and the cases that pin it,
and the retry-ladder hook the cache statement of the round before did not name.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
It is a new file of 2026-10-24,
because [Audit ledger: probes that read one editor card](audit-ledger-editor-card-probes.md) stood at 982 lines.
These entries follow up B885 to B893 of that file.

## B894: the floor of B891 counted arms as independent, so the rule is a paired exact test per effort

Status:
fixed on 2026-10-24 (UTC) in `41d678124838475fa00916f1b0273b3aa7936b3a`,
`4e415d7cd0adaaa76d1450579c911a4373ae45c5` and `6404700c3457eb51a60d63bc76bbe5685aff875d`,
red `0891cf0530d0bfd295c4002b46cbe02fb0665e80` and `09a0f38ba542af5cc90e2db6265b6fe85371791d`,
open to the owner's veto.
Follows up B891,
whose floor,
kind clause and enumeration fixture it replaces.

The rule of B891 compared pooled clean fix counts against a floor of 12 trials taken from an enumeration that treated
the arms as independent.
The arms are not independent:
they walk the same trials,
and the same trial is easy or hard for every arm.
A count against a floor also threw away that pairing,
needed a second medium arm to calibrate itself,
and let a kind clause veto a gain on a handful of trials.

The rule is now a one-sided exact paired permutation test of the high effort against medium and of the low effort
against medium,
each at 0.025,
over the trials every arm completed.
The labels are permuted within each entry and the entries are the units,
because the two trials of an entry are not two independent draws
(`probe/src/corpus-run/editor-fidelity-paired.ts`).
Each effort prints its paired difference,
a 95 percent interval from a seeded bootstrap over entries,
the exact p-value and whether it is shown better.
The rule text states the tie-break:
when both efforts are shown better the larger estimate is chosen,
and when the estimates are equal the cheaper effort.
It also states that a medium verdict means no effort was shown better than medium,
which is not a finding that the others are as good.
The kind clause no longer vetoes:
the paired estimate for each kind of defect is printed as a secondary line.
`decideEffort` and `decisionRuleText` stay one source
(`probe/src/corpus-run/editor-fidelity-decide.ts`),
and the README quotes the text,
which a comparison of the two in this round found equal at 2052 characters.
A second medium arm is still readable:
it joins the labels the test permutes,
so a label's contribution is the label count times its total minus the sum of all totals.

False picks and power were measured by one seeded simulation of the bench over the committed pool in its order
(`probe/src/corpus-run/editor-fidelity-paired-null.test-fixture.ts`,
seed `edr6-sim`),
run once as the systemd user unit `edr6-sim` with a memory cap of 4G and a CPU quota of 200 percent.
The unit was started with `XDG_RUNTIME_DIR` passed and `--wait`,
which the brief's command did not include;
it ran 3 minutes 31 seconds,
peaked at 154 MB and exited.
The null grid had true fix rates from 0.30 to 0.95 in steps of 0.05,
within-entry correlations 0,
0.25,
0.5,
0.75 and 1,
shared trial draws with chance 0 and one half,
and 30,
60 and 88 shared trials.
The chance of choosing another effort when none is better was at most 0.0470,
at 60 trials,
a share of 0,
a correlation of 0.75 and a rate of 0.4 over 1000 replicates;
at 88 trials the worst was 0.0465 over 2000 replicates and at 30 trials 0.0400.
Two efforts each tested at 0.025 give a bound under 0.05 for the pair,
and the simulation stays under it.
The smallest difference chosen with chance 0.8,
at 88 trials and a correlation of 0.5,
is 25.0 points when medium fixes half (23.4 with a share of one half),
20.5 points at seven in ten (19.6)
and 13.7 points at eighty-five in a hundred (13.5).
A smaller difference cannot be told apart by this bench.
The fewest shared trials the read step accepts,
`MIN_SHARED_TRIALS`,
is 40:
at 40 trials and a rate of one half the smallest difference is 36.7 points,
and at seven in ten 27.9,
while at 30 trials and a rate of seven in ten the chance of 0.8 was not reached within 30 points (0.699).
At eighty-five in a hundred a bench of 40 trials reached a power of 0.235 at 14 points,
so such a bench can tell apart only a large difference.
The null grid did not include 40 trials,
so no false-pick figure is claimed for that size beyond the test's own level.
The raw output of the run is `sim-r6.txt` in the round's directory.

The cases are in `editor-fidelity-paired.unit.test.ts`,
`editor-fidelity-paired-null.unit.test.ts` and `editor-fidelity-decide.unit.test.ts`.
`editor-fidelity-decide-null.test-fixture.ts` and its unit test,
the fixture and the pin of the floor of B891,
were removed in `6404700c3457eb51a60d63bc76bbe5685aff875d`.

Slips:
`4e415d7cd0adaaa76d1450579c911a4373ae45c5` added the simulation fixture and its test together,
with no red run of the test before the fixture,
and `0891cf0530d0bfd295c4002b46cbe02fb0665e80` did not carry the seed case that
`41d678124838475fa00916f1b0273b3aa7936b3a` corrected,
so the red tests are not byte-identical in the fix.

## B895: the pool held 40 trials, too few for a paired test, so it holds every trial the clone offers

Status:
fixed on 2026-10-24 (UTC) in `084ee1d4dbf0ca160e503b51bdacd5df7b395e08`,
red `4fa640800528958aa0564c1de37409a4f3c0bc68`,
open to the owner's veto.
Follows up B888.

The seeded draw wanted 40 trials and stopped there,
so the bench's paired test had too few entries to separate efforts.
The draw now aims for at least 150 (`POOL_TRIALS_WANTED`) and takes entries in the seeded order,
at most two trials to an entry,
until the pool holds that many or the clone has no more.
The clone at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379` offers 88 trials from 53 entries:
35 entries with both kinds of defect,
15 with an insertion only and 3 with an alteration only.
The replay reads 81 references and finds all 88 trials scoreable.
Reaching 150 is not possible under the draw's rules.
Of the 93 entries the clone lists only 50 are eligible,
and a draw that wants 1000 trials still stops at 84 drawn trials (47 insertions and 37 alterations)
beside the 4 reviewed ones,
because the rest are left out for want of a donor sentence (25),
a donor span (16),
a shared number (45),
an entry of another shape (2),
an unreadable entry (1) or a scripted editor that could not score them (6).
No rule was loosened to reach 150,
which would change what a trial is,
and the owner may veto this.
The pool files grew from four drawn parts to ten (`editor-fidelity-pool-drawn-1.ts` to
`editor-fidelity-pool-drawn-10.ts`),
and the default trial cap of a launch is now 200,
more than the pool holds.
`editor-fidelity-pool-draw --check` replays the committed files against a fresh draw and reads every committed reference
through the hash-verifying reader;
its output for the final tree is `check-r6.txt` in the round's directory.

## B896: one failed ask failed a whole arm, so the read step walks three or four logs by trial

Status:
fixed on 2026-10-24 (UTC) in `e995ede8c0361e97c89060860f8d61ce89ebbf04`,
`cc51b80284199e9870e0d95297c0759550a406ff` and `6404700c3457eb51a60d63bc76bbe5685aff875d`,
red `dc490458b8c438178d55e95004ba45e716143bef`,
`6473e88f45093ea8b1446e50a4a6d25967db65f9` and `09a0f38ba542af5cc90e2db6265b6fe85371791d`,
open to the owner's veto.
Follows up B889.

The read step failed an arm in which any ask ended at the output-token cap or threw,
so one bad trial could throw away an arm that cost several dollars.
The arms also had to be four,
two of them medium,
and run in a fixed order that did not let a later arm learn how far the first got.

The arms now run high first,
under its cap,
then medium and then low,
each once (a second medium is read if one is given).
The probe takes `--trials n`,
the number of leading pool trials to walk,
refused beside a `--cap` above 0 and allowed beside `--cap 0` for the dry run of a later arm
(`run/src/corpus-run/editor-fidelity-command-line.ts` for the command line,
`probe/src/corpus-run/editor-fidelity-args.ts` for the reading).
A guard refusal ends the arm as capped and not failed:
the arm prints a `CAPPED at trial k` line and a `TRIALS completed k of p` line,
keeps the rows it finished and exits 0.
The runbook reads n from the first log and gives it to the others,
so every arm walks the same first n trials.
Each row line names its entry,
which is the cluster the test permutes within.

The read step finds the trials every arm completed.
A trial that ended at the output-token cap scores as unusable for its arm,
and a trial in which any arm's ask threw is left out of every arm's comparison and counted.
The `SHARED` line prints how many trials are shared,
how many entries they come from,
how many were left out as thrown and how many were not completed by every arm.
The read step fails only when fewer than 40 shared trials remain,
and also when a later arm walked other trials than the first or in another order.
The `STOPS` line of each arm is now a count for each stop reason and the rows that did not end ok,
where it was one entry for every row.

The cases are in `editor-fidelity-arm-bench.unit.test.ts`,
`editor-fidelity-arm-stops.unit.test.ts`,
`editor-fidelity-arm-report.unit.test.ts` and `editor-fidelity-decide.unit.test.ts`.

## B897: the spend guard's one-call-at-a-time rule was a comment, and three of its edges had no case

Status:
fixed on 2026-10-24 (UTC) in `f57ce00fef9802a9ae2dc8d83a1d2452d2ffd898`,
red `43ec3c64f3b07e48b949746894a1684bc708f005`,
open to the owner's veto.
Follows up B885.

The guard keeps the attempts the client told it of for the call in flight,
which is sound only if calls do not overlap.
The comment said the probe asks one row at a time.
The probe seats one editor card alone (`editorModelIds: [cardId]` in `editor-fidelity-trial.ts`)
and walks its rows with an overlap of one,
so they do not overlap,
and the guard now asserts it:
a call asked while another is in flight throws `ConcurrentGuardCallError` before the inner client is reached,
without ending the arm,
and the next call after the first settles is admitted.
The place is held by a disposable handle taken before anything is admitted.

The case that pins the attempt-only boundary is new:
with 0.5 USD already on the meter,
an attempt bound of 0.5 under `--max-usd` 1 is sent and a bound of 0.51 is refused.
It passed at once.
A mutant that changes `<=` to `<` in the guard's one comparison
(`(spentUsd + ceilingUsd) < maxUsd` in `refuseOver`),
built and run on the guard's test file,
fails it and the older case that brings a whole call exactly to the amount;
the failing lines are `mutant-failing-lines.txt` in the round's directory,
and the file was restored with `git checkout` from the commit.

Neither the typed-decision exchange nor the quota read of the client the probe builds sends a request.
The client is built over the Claude key alone (`cardsClientFrom`),
so the router has no decisions client:
`routedDecide` throws `NoProviderForModelError` for want of one before any transport
(`provider/src/provider-router-decide.ts`).
The quota read is `synthetic?.quotas ?? unconfiguredSyntheticQuota` (`run/src/corpus-run/run-config.ts`),
which answers an exhausted snapshot and makes no request without a Synthetic key.
A case drives both over a scripted transport and sees no exchange,
after a chat exchange on the same client did reach the transport as the positive control.
Neither needed a guard,
and none was added.

## B898: the retry-ladder hook the cache statement did not name

Status:
recorded on 2026-10-24 (UTC).
Follows up B893.

The cache statement of B893 said the round's changes reach "the Claude client (a hook it calls before an attempt)".
The hook is `await beforeAttempt?.()` in the shared retry ladder,
`exchangeWithRetry` in `provider/src/transient-retry.ts`.
Every provider client reaches the ladder:
the direct,
Claude,
DeepSeek,
Synthetic,
NeuralWatt,
Hyper,
OpenRouter and Bedrock clients all call it.
Only the Claude client hands it a `beforeAttempt`
(through `claude-attempt-admit.ts`),
so a client that passes none sends exactly as before:
an absent hook is a no-op.

Cache:
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' -- '*.ts'` over
`5f6f303a75b06c2b3c941200f23c7bc1271caba3..HEAD`,
every commit of round 6,
lists none.
The same search over `c265e8df9525f4513b8229482d3e8d932ea630df~1..c265e8df9525f4513b8229482d3e8d932ea630df` lists
`c265e8df9525f4513b8229482d3e8d932ea630df`,
a commit known to move a cache version,
so the search can show a hit.
This round's source changes are in the probe and in the run package's command-line spec,
which read no cache,
and they change no request body,
no reply and no key.
No cache version moves.
