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

Round 7 (B899) read the rule over units,
trials asked again in more than one pass,
raised the floor to 60 units and re-stated the false-pick chance and the power for that design.

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

Round 7 (B899) measured how many trials a wider draw would offer from the same 50 drawn entries
(134 at two of a kind to an entry,
196 without a limit),
found that it adds no clusters to a test whose units are entries,
and did not adopt it.
No donor or number rule was relaxed.
The default cap of a launch is now 1000 asks,
more than three passes over the pool.

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

Round 7 (B899) added `--passes n`,
read a trial asked again as a unit of its own in the same entry,
and made `--trials n` count units through the passes.

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

## B899: the bench may spend 60 USD, and the money buys evidence through replicates read as units

Status:
fixed on 2026-10-24 (UTC) in the commits the section "What changed" names,
open to the owner's veto.
Follows up B894 to B898.

The owner ruled on 2026-10-24 that the bench may spend 60 USD in all,
where it was 20.
`EDITOR_BENCH_MAX_USD` in `probe/src/corpus-run/editor-fidelity-args.ts` is 60,
with the ruling cited where it is defined,
and the arm is refused when its `--max-usd` and `--bench-spent-usd` together pass 60.
The caps of the arms are now 24 USD for high,
18 for medium and 15 for low,
which add to 57.

### Why more money does not buy more trials

The pool holds at most 88 trials from 53 entries (B895),
so a larger amount cannot buy a larger pool unless the draw changes,
and a larger pool of the same entries adds little to a test whose units are entries.
Three designs were compared,
each by one seeded simulation at the same level
(two efforts tested at 0.025 each,
a bound of 0.05 for the pair):

- Replicates,
  in which every arm walks the 88 trials a number of times in one seeded order,
  pass 1 over all trials,
  then pass 2 and so on,
  and a trial's replicates belong to the cluster of its entry.
- A widened draw,
  in which the pool takes more trials from the same 50 drawn entries and the 3 reviewed ones.
  The scoreable trials the pinned clone offers,
  by the number of trials of one kind taken from an entry,
  are 84 for 1
  (the committed pool,
  with 47 insertions and 37 alterations,
  which is the positive control of the count),
  134 for 2,
  157 for 3,
  168 for 4,
  177 for 5 and 196 without a limit,
  and the 4 reviewed trials come on top.
  The donor and number rules were not relaxed to reach these.
- Both.

The simulation (`editor-fidelity-paired-null.test-fixture.ts` and its two sibling fixtures)
gives each trial of a unit an entry anchor with chance the square root of the correlation,
a draw shared by the arms with chance 0 or one half,
and for a trial asked again
the arm's first answer with chance `agreement` and a fresh draw otherwise.
The agreement of a Sonnet card with itself on one trial is not measured,
so every figure is given for 0,
one half,
three quarters and 1.
Each design ran once as its own systemd user unit (`edr7-sim-1`,
`edr7-sim-2` and `edr7-sim-3`) under a memory cap of 4G and a CPU quota of 200 percent,
seed prefix `edr7-sim`,
with the output in `sim-replicates.txt`,
`sim-widened.txt` and `sim-both.txt` of the round's directory.
The null grid had true fix rates from 0.30 to 0.95,
correlations 0,
0.25,
0.5,
0.75 and 1,
shares 0 and one half,
agreements 0,
one half and 1,
1000 replicates a cell,
at 176 units (264 for the third design).
The power curves had 800 replicates a step of 2 points.
The model reproduces round 6:
the widened design at 88 units gives 25.2,
20.4 and 13.7 points where B894 measured 25.0,
20.5 and 13.7.

The smallest difference chosen with chance 0.8,
at a correlation of 0.5 and a medium fix rate of one half,
seven in ten and eighty-five in a hundred:

- Replicates at 176 units,
  by the agreement of a repeat with the first answer:
  17.1,
  13.9 and 10.3 points at 0;
  20.9,
  17.3 and 11.8 at one half;
  22.8,
  19.0 and 12.9 at three quarters;
  24.9,
  20.6 and 13.5 at 1.
- Replicates at 264 units:
  13.6,
  11.5 and 8.7 points at 0;
  19.0,
  15.9 and 11.1 at one half;
  22.3,
  17.8 and 12.1 at three quarters;
  25.6,
  20.6 and 13.7 at 1.
  At 132 units and one half:
  23.7,
  19.3 and 12.8.
  At 88 units (one pass):
  25.6,
  20.4 and 13.5.
- The widened draw at 176 units:
  24.9,
  20.6 and 12.9 points;
  at 200 units (the whole widened pool):
  26.1,
  21.1 and 13.1.
- Both at 200 units:
  26.8,
  21.0 and 12.9 points;
  at 264 units,
  by agreement:
  21.6,
  17.6 and 11.5 at 0;
  23.6,
  18.9 and 12.4 at one half;
  24.4,
  19.8 and 12.7 at three quarters;
  24.7,
  19.9 and 12.9 at 1.

The widened draw gains nothing:
its extra trials come from the same 53 entries,
so under a test that permutes within entries the number of clusters stays 53.
Replicates are never worse than the single pass (at agreement 1 they match it)
and are better at every smaller agreement,
and at the same 264 units they beat both at every agreement below 1.
The design is replicates,
which needs no new pool and no redraw.
The headline in the rule text is the single pass and then 176 units for three agreements.

### The false-pick chance

The worst reading of the null grids was 0.0550 for replicates
(176 units,
agreement 0,
share 0,
correlation 1,
rate 0.45),
0.0500 for the widened draw and 0.0590 for both
(264 units,
agreement one half,
share one half,
correlation 1,
rate 0.6).
A cell of 1000 replicates reads within about 0.007 of its true value,
and the grid has several hundred cells,
so the largest of them lies above 0.05 by the noise:
the test's own bound of 0.05 holds because each of the two efforts is tested exactly at 0.025.
The rule text states 0.055 as the worst reading of the design chosen.

### What changed

- The bench total is 60 USD (red `c6a8eac28aad51d450e7e8b667eef60e06b001c7`,
  fix `3cdaf402da38a7c0039d08852e5202109a513173`).
- The simulation fixture reads entries of any size with an exact convolution table,
  walks the pool pass by pass,
  and models a repeat by `agreement`
  (red `0c94291e5603a41d0bf4d2c05f1248852c96e8d3`,
  fix `100280270928a48e4727c4a850a9c23d52e98385`).
  The fixture is split in three files so that none passes the length cap.
- The probe takes `--passes n` (red `06c0122568fa621965709bdabf80e0ffc3836bf0`,
  fix `1ace082a26f965df879ff3b79d01c78e54325cc1` for the command-line spec in the run package;
  red `a75383cb7e16862576f44b157b8ecf627c37a612`,
  fix `59cd3a9d511c9f8c716924e920cdd164c262d1ac` for the reading in the probe).
  It is a member only when written,
  zero is refused,
  and the default cap grew from 200 to 1000 asks so that three passes over 88 trials are not cut.
- An arm walks the pool pass by pass,
  and `--trials n` counts asks through the passes,
  so a later arm walks the first n units the first arm completed.
  A row of a later pass names it after the entry (`; pass 2`),
  the kept row and the plan carry the pass,
  the planned line says the passes,
  the preflight line says the ceiling is priced over every ask of them
  (red `226ff26981ffc42872c1828ed2c00317cb90bd6c`,
  fix `bcc3462530844ad282176fbb56cda6c8bbfe93ac`).
- The read step parses the pass of a row line (one when it names none),
  keys the rows of an arm by trial and pass,
  so a thrown replicate leaves out its own unit only,
  and names the pass in the trial-order check
  (red `abab3b1b30300cd044ab9f13a0c346f30b4ef088`,
  fix `bf27d2a1f4279d67269900ca7d31e960d4be8984`).
  `decideEffort` needed no change:
  it already groups by entry,
  and a replicate is one more trial in its entry's cluster.
- The SHARED,
  PAIRED,
  KINDS and shared-units lines say units,
  the floor `MIN_SHARED_TRIALS` is 60,
  and the rule text defines a unit,
  says a trial asked again joins its entry's cluster,
  and states seed `edr7-sim`,
  the worst false pick 0.055 and the power figures of the design
  (red `0da0a7831ed7186aae9e73386f614244f489c538`,
  fix `79839a033ba6a9ef316dcadad9c53856a70a0a7b`).
  The README quotes the same text.
- The floor of 60 follows the power figure:
  at 60 units the smallest difference is 30.3 points at one half and 24.3 at seven in ten,
  against 37.3 and 27.6 at 40,
  and at eighty-five in a hundred it is not reached within 14 points at either size.
- Two bare counts of the noun units were found by the source scan of counted nouns once the probe counted it,
  and are fixed:
  a `FragmentLandingError` for a slice of one unit and the line before the first slice of a one-section document
  (red `084ea8f04cb16d801fb5523126e52c5ad847e71f`,
  fix `4fb5acf8e45496336a6f46161ababd26d59a7ab0`,
  with the inventory entry of the source scan in `4602a360ac7928d2ca7984c01056e5621b22d5dd`).
  The asks and passes of the planned and preflight lines use `wordForCount`,
  and the plan lines have their own test file
  (`326bb54c1a132ad46e7314604b01362bcf72c9e1`).

### The runbook and the dry runs

The runbook runs high first with `--max-usd 24 --passes 3`,
which is 264 units,
reads n from its `TRIALS completed n of p` line,
and gives medium (`--max-usd 18`) and low (`--max-usd 15`) `--passes 3 --trials n`.
The keyless dry runs with `--cap 0` for the three arms are `dryrun-r7.txt` of the round's directory:
high prices 264 asks at a worst case of 99.4330 USD (32000 output tokens an ask),
and medium and low,
given `--trials 200`,
price the first 200 of 264 asks and print the partial-matrix warning.
`editor-fidelity-pool-draw --check` found 11 files equal to a fresh draw and 88 of 88 trials scoreable
(`check-r7.txt`).

### Open

The agreement of a card with itself is the one assumption the headline cannot settle.
The bench measures it:
the share of trials on which an arm's second pass gives the answer of its first.
The read step does not print it yet.

### Cache

`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 7cc20e762ec971b5b20daffb1bf404a19ac23cbf..HEAD -- '*.ts'`
lists no commit of this round.
The same search over `c265e8df95~1..c265e8df95` lists `c265e8df9525f4513b8229482d3e8d932ea630df`,
a commit known to move a cache version,
so the search can show a hit.
This round changes the probe,
the run package's command-line spec and two messages (typography and repair),
none of which reads a cache or changes a request body,
a reply or a key.
No cache version moves.

### Slips

The commit body of `100280270928a48e4727c4a850a9c23d52e98385` cites the red as `0c94291e56ff6c1ff6c96bf66ac6f5e4f6b0eef9`,
which is not a commit;
the red is `0c94291e5603a41d0bf4d2c05f1248852c96e8d3`.
The body of `0da0a7831ed7186aae9e73386f614244f489c538` counts 11 failing cases and the run shows 12.
The first search for the cache statement wrote the start commit as a short hash before the pathspec and listed
every commit that touches a version,
and was repeated with the full hash and the range before the pathspec.
The fix `4fb5acf8e45496336a6f46161ababd26d59a7ab0` left the new counted noun out of the source scan's inventory,
so the scan failed until `4602a360ac7928d2ca7984c01056e5621b22d5dd`.
The red commits of the run package,
the probe's arguments,
the walk and the read step fail the type check of their test files until the fix lands,
which is stated in their bodies.
The other process slips of the round are in `ledger-notes.md` of the round's directory.
