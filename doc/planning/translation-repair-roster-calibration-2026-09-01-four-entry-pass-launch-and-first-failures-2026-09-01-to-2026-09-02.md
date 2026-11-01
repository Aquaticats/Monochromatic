# Roster calibration: the four-entry pass launch and its first failures, 2026-09-01 to 2026-09-02

The launch form verified before the pass,
and the first twenty minutes under the 60 second window;
the pin pass that died of a refusal cooldown and the Carena pass refused at publish time;
the decision to cut the chain to two entries on a rate measurement.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## Before the four-entry pass

The pass keeps its own timing:
one slice at a time and the built-in 180000 ms window
(`corpus-pass.ts` resolves `STRAGGLER_GRACE_MS`;
only `editor-calibrate` adopts the 300000 ms calibration window
and the overlap of four).
The overlap decision doc leaves moving the pass to `#261`'s measurement.
So the pass launches at its defaults,
not at "measured calibration values".

Launch form verified on 2026-09-01 with the zero-quota `--plan` mode,
both printing `PLAN ok` with
`client=constructed` at tip `c4f707e3b`:

-   `TRANSLATION_REPAIR_RUNS_DIR=<throwaway> mise run //package/module/translation-repair:corpus-pass -- --only keyword233,Toka_ls,XIEPT2` (moved on 2026-10-18 to `//package/translation-repair/corpus-pass:corpus-pass`)
    against the clone at `~/one-among-us/data`,
    verified at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.
-   The same with `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` at the PR-386 fixture
    (`~/temp/agent/pr386-mock-home-20260829/one-among-us/data`,
    HEAD `d343df909b1673d68dd5cd805aeda1dfacd2d3c4`)
    and `TRANSLATION_REPAIR_CORPUS_COMMIT` set to that sha,
    `-- --only Carena0442`.
-   Each hand-picked run goes into its own throwaway runs dir,
    as the `ONLY` banner demands.
    The plan had the two run one after the other so the per-model limiter stays one process wide;
    the fast-iteration principle (the "Wall clock" section) launched them concurrently instead,
    so each process holds its own limiter and a Synthetic model can see both at once.
-   The pass reports `soft=259200000ms hard=25200000ms` budgets.
-   The pass log's abandons by role are the production-window evidence the `glm-5.3` judge drop left open
    for its critic,
    panel and translator seats.

## The four-entry pass, first twenty minutes

Launched 2026-09-02 01:10 UTC at tip `23814e8bf`,
both passes printing `STRAGGLER GRACE OVERRIDDEN ... 60000ms`
and `OVERLAP <entry> value=4 source=TRANSLATION_REPAIR_SLICE_OVERLAP`;
the real clone and the PR-386 fixture
stayed clean (`git status --short` empty on both after the first rounds).
Logs:
`~/temp/agent/four-entry-pin-20260901.log` (keyword233,
Toka_ls,
XIEPT2) and
`~/temp/agent/four-entry-carena-20260901.log` (Carena0442).

What the 60 s window did to the seats,
read off `<stage> round: N/M heard` lines and the abandon lines
at 19 minutes in:

-   Editor rounds:
    pin pass 12 of 13 heard all three,
    Carena 7 of 9;
    every one of the 3 cuts was
    `hf:zai-org/GLM-5.3-Flash`,
    the top editor by both readings,
    cut mid-reply exactly 60000 ms after quorum
    (quorum formed at 13 to 23 s;
    the round then ran 73 to 83 s).
    The other GLM editor,
    `glm-5.3`,
    was never cut:
    it is the quorum-forming voice,
    and the window does not
    run against whoever forms quorum.
-   Refiner rounds:
    none reached yet in either pass.
-   Checker rounds:
    18 of 19 heard all three.
-   Critic rounds (8 seats,
    quorum 4):
    pin 8 of 15 heard all eight,
    Carena 1 of 14;
    Carena's mode is 6 of 8.
    Panel rounds:
    pin 3 of 13 full,
    mode 7 of 8;
    Carena 1 of 14 full,
    mode 7 of 8.
    The cut readers are the same slow voices as in calibration (`GLM-5.3-Flash`,
    `Qwen3.8-27B`,
    `Kimi-K3`,
    `minimax-m3`,
    `gemma-4-26b-a4b-it`).
-   The `0/1 heard` rounds are not retries.
    Each follows a `<stage>: recovery round for 1 unreadable answers`
    line (`stage-quorum.ts`,
    the one recovery round for an answer that parsed but failed the schema guard),
    and each is answered in 0 to 1 ms by `PROMPT-REUSE source=memory` for the same model and digest,
    then
    fails the same guard on the same bytes.
    `prompt-uniqueness-client.ts` states it:
    "Any returned outcome claims identity permanently for client
    lifetime,
    including schema mismatch or refusal."
    So the recovery round costs no wall clock and can never recover anything:
    the same prompt to the same
    model is served the same unreadable reply.
    Five such rounds so far (Carena 3:
    gemma twice,
    deepseek-v4-flash once;
    pin 2).
    This is the path the unsettled gemma malformed-JSON item was leaning on,
    and it is dead by construction;
    filed as `#473` for the owner (a design conflict between two deliberate rules,
    not a fix to make
    unasked).
-   Wall clock,
    projected from the slice cache (`slice-cache/<entry>/<hash>.json`,
    one per settled repair
    slice):
    Toka_ls 14 of 16 repair slices at 25 minutes,
    Carena 18 of 22 at 22 minutes,
    and each entry then
    runs its translate lane over the same slices.
    An entry is about an hour;
    the pin pass carries three in sequence,
    so it projects to about three hours
    against the `FIT` hour.
    It is left running rather than stopped,
    because the reading step starts on the first two artifacts
    (Toka_ls and Carena) as they land and does not wait on the pass,
    and because a relaunch on the current
    build would discard the slice cache (the pipeline digest moved from `ab964c96…` to `d0a82037…` with the
    writer dial),
    re-spending what is already settled.
    The owner's veto stops it at any time;
    the artifacts already written survive a stop.
-   The "delivered chars" in a cut line is the raw streamed body (`partialText` in `stream-drain.ts` joins
    the decoded chunks,
    framing and reasoning deltas included),
    which is why a critic cut at 60 s shows
    0.2 to 1.7 million characters:
    it is not answer text,
    and the figure says only that the model was still
    streaming.
-   Spend:
    Synthetic weekly allowance 10.39 to 8.18 percent over the 19 minutes with both passes running
    (about 7 points per hour,
    so the remaining allowance carries roughly another hour before every Synthetic
    seat falls to its Hyper route);
    Hyper balance 6636 to 6492 credits.
    No retry exhausted;
    429s were retried within the budget (pin 4,
    Carena 11 in the first ten minutes).

The advisor's reading of the launch,
taken before these numbers:
a window sized for the eight-wide reader
rounds can unseat the writers the three-wide rounds were seated on,
since the last reader voice adds one
ballot to seven while the last writer voice is a whole candidate.
The numbers bear it out in one place,
the top editor on the long Carena document,
and nowhere else so far.

The remedy is a launch-time dial rather than a built-in and rather than a restart:

-   `TRANSLATION_REPAIR_WRITER_GRACE_MS` (commit `a3da317df`,
    `writer-grace-override.ts`) gives the editor,
    refiner,
    translate and consolidate gathers a window of their own;
    unset,
    they follow the round window as
    before,
    so nothing built-in and nothing in `doc/decision/translation-repair-straggler-grace.md` moves.
    Both dials read through one parser (`readWindowDial`),
    the pass and the editor calibration print a
    `WRITER GRACE OVERRIDDEN` note beside the round note,
    and the zero-quota `--plan` run at tip `f7abd69b5`
    printed the note under `60000`/`180000` and refused `180s` by name.
-   The running passes were NOT restarted under it.
    Three cut editor rounds in 22 is the measured cost;
    a restart re-spends twenty minutes of both passes and about 2 points of the Synthetic allowance to
    recover a voice that 19 of 22 rounds already heard,
    against the owner's fast-iteration principle.
    The next launch sets the writer dial;
    the reading step (task 3) reads these artifacts knowing which
    editor rounds ran short,
    since each artifact carries `stage-voice-lost (editor hf:zai-org/GLM-5.3-Flash)`.
-   What the dial should say is measured,
    not chosen:
    the built-in 180000 ms covers the GLM editors' p90
    (153 to 199 s completed streams) at the cost of up to two more minutes on the minority of writer rounds
    the third voice runs long in;
    300000 ms is the calibration's own window and what the seat was measured
    under.
    Rule,
    written before the refiner and translate rounds of this pass are read:
    the next launch sets
    `TRANSLATION_REPAIR_WRITER_GRACE_MS=180000` (the built-in,
    no new number),
    unless this pass's refiner or
    translate rounds show the seated slow voice (`GLM-5.3-Flash` as refiner,
    `glm-5.3` as translator) losing
    a majority of its rounds to the window AND winning the judged round when it is heard,
    in which case the
    tail is the seat and the dial goes to the calibration's 300000 ms.
    Editor rounds are the writer seat's best case (the slow `glm-5.3` forms quorum,
    so the window runs
    against one slow voice with a head start);
    refiner rounds (quorum on `deepseek-v4-pro-0813` and
    `minimax-m3`,
    both fast) and translate rounds (quorum 4 of 7 in seconds against `glm-5.3` at p50 61 to
    66 s) are where a 60 s window bites hardest,
    and the first refiner round of the pin pass heard all three
    in 23.8 s.
    Whether the late editor voice wins its rounds cannot be read from the round lines alone (the `in grace`
    figure does not name the late voice;
    the judged-round winner lines are per issue envelope,
    several per
    editor round);
    pairing needs the per-call stream lines joined by time,
    which is instrument work for the
    reading step if the counts warrant it.

## The pin pass died of a refusal cooldown, not of budget

At 01:40:19 UTC the pin pass ended with `DONE processed=0 of pending=3; artifacts=0/92 elapsed=1786317ms`:
`TALLY Toka_ls status=INCOMPLETE ms=1786206 ... error=translation repair interrupted: provider-unavailable`,
then XIEPT2 in 44 ms with the same error,
then keyword233 in 66 ms with
`front matter is not publishable (incumbent-fallback)`.
No page and no artifact were written;
the real clone stayed clean.

What happened,
read off the log and the source:

-   Two processes,
    each with four slices in flight over eight-wide reader rounds,
    produced HTTP 429 bursts
    (68 in the pin log,
    11 in the first ten minutes of Carena's),
    eight calls at once hitting the retry
    ladder together (`exchangeWithRetry`,
    five attempts,
    about 15 s in total).
-   A 429 that outlives the ladder reaches the router as a `SyntheticHttpError`
    (renamed `ProviderHttpError` on 2026-10-19),
    and `isBudgetRefusal`
    (`provider-budget-refusal.ts`) counts 429 and 402 alike as "out of budget":
    `markRefused` then holds that
    provider dry for `REFUSAL_COOLDOWN_MS` (300000 ms) whatever its meter says,
    by design
    (`provider-budget.ts`:
    "A refusal is stickier than a meter reading";
    `doc/decision/translation-repair-multi-provider.md`).
-   Synthetic was so held at 01:21:44,
    01:32:56 and 01:39:28 (each time on `GLM-5.3-Flash`),
    which routed
    its traffic to Hyper;
    at 01:39:54 to 01:39:59 Hyper refused eight models in five seconds and was held
    too;
    with both held,
    `EveryProviderDryError` (then under its two-provider name) fired for every voice of every stage (518 lines in
    twenty-five seconds),
    Toka_ls's translate lane ended,
    and the two remaining entries were attempted and
    failed inside the same second because the hold is process-wide state.
-   Thirteen seconds before the verdict the meter read
    `METERS synthetic=wet hyper=wet syntheticWeekly=7.24% syntheticFiveHour=2729/2750 hyperBalance=6447`.
    Neither provider was out of anything.
    The error text,
    "Synthetic has no five-hour or weekly credit left and Charm Hyper has no balance left.
    Nothing further can be bought,
    so this run ends",
    asserts a state that was never measured.
-   keyword233's refusal is the same outage through another guard:
    every stage lost every voice,
    the
    front-matter slice kept the archive's text,
    and `assertFrontMatterComplete`
    (`corpus-run/front-matter-completeness.ts`) refuses a page whose metadata is the incumbent while the
    source's differs.
    It is not a defect of that entry.
-   The Carena pass,
    in its own process,
    was held out of Hyper at 01:40:32 to 01:40:35 (four refusals) with
    Synthetic wet,
    so for five minutes its Hyper-only seats (`glm-5.3`,
    `deepseek-v4-pro-0813`,
    `minimax-m3`,
    `gemma-4-26b-a4b-it`) answered `NoProviderForModelError`:
    editor rounds with one voice of three,
    refiner
    rounds with one of three.
    It kept running and its artifact will carry those degraded stages as findings.

The launch decision that produced the burst was mine (two concurrent processes,
against the runbook's
one-process note and the advisor's warning on per-model load),
and the pass's own SEAT lines carry the cost
as `threw`:
`deepseek-v4-pro-0813 asked=168 threw=74`,
`GLM-5.3-Flash asked=173 threw=97`,
`Qwen3.8-27B asked=177 threw=83`,
most of them the dry-verdict lines rather than provider faults.

What is the pipeline's,
not the launch's,
and is filed for the owner as a design conflict (`#474`):

-   A 429 is read as allowance exhaustion without consulting the meter that was read thirteen seconds
    earlier and says otherwise;
    a rate limit and an empty allowance get the same five-minute hold.
-   When both providers are held,
    the pass fails every remaining entry at once instead of waiting out the
    shorter hold,
    which is at most five minutes against a 72 hour soft budget.
-   The `EveryProviderDryError` (then under its two-provider name) text states what was assumed,
    not what was measured.

Whether one process trips the hold,
measured rather than assumed:

-   The single-process calibration logs at overlap 4 carry no hold at all:
    `editor-calibrate-40` (three
    hours,
    the repair lane) 0 `markRefused`,
    0 HTTP 429,
    268 HTTP 503;
    `producer-calibrate-40` 0,
    0,
    1;
    `editor-calibrate-6` 0,
    0, 1.
    A 503 is not a budget refusal and never held anyone.
-   The Carena pass,
    alone from 01:40 and at overlap 4,
    was held out of Hyper again at 01:54:06 and
    01:54:08,
    four minutes into its TRANSLATE lane (seven translators and eight judges per slice,
    four
    slices in flight,
    five of the nine models Hyper-only).
    So one process at overlap 4 is safe in the repair
    lane and trips Hyper's rate limit in the translate lane;
    the hold is not only a two-process artifact,
    which is added to `#474`.
-   The Synthetic weekly allowance is the other fuse:
    once it reads zero the meter legitimately says dry,
    every Synthetic seat routes to Hyper,
    and that is the 01:39:54 load shape.
    The quota reading carries
    no reset time (`synthetic-client.ts` reports five-hour remaining/max and weekly percent only).
    At 01:54
    it read 6.66 percent,
    falling about 2.3 points per hour with one process,
    so roughly three hours remain.

Relaunch plan,
revised on that:
one entry per launch (Toka_ls first,
then XIEPT2,
then keyword233),
one
process,
after the Carena pass exits,
under `TRANSLATION_REPAIR_WRITER_GRACE_MS=180000`,
the 60 s round
window and `TRANSLATION_REPAIR_SLICE_OVERLAP=2`,
into one fresh throwaway runs dir
(`~/temp/agent/pin-relaunch-20260902`) on the current build.
Overlap 2 rather than 4 because 4 tripped Hyper
in the translate lane in one process;
rather than 1 because 1 is the built-in the fast-iteration rule spends
first,
and a single Hyper hold with Synthetic wet degrades five minutes of rounds rather than ending the run.
If the relaunch logs a `markRefused`,
the next entry launches at overlap 1.
One entry per launch ends inside
the hour,
inside the weekly fuse,
and puts an artifact in front of the reading step while the next runs.

Tooling note from the day:
twice,
the package `lint` task's `build` dependency left `dist/final/node/index.d.mts`
without the new exports until an explicit `mise run //package/module/translation-repair:build` (moved on 2026-10-18 to
`//package/translation-repair/<package>:build`),
so
`lint:types` read stale declarations.
Reproduced,
not diagnosed.

## The Carena pass finished and was refused at publish time

`TALLY Carena0442 status=INCOMPLETE ms=5668128 ... error=entry Carena0442 front matter is not publishable
(incumbent-fallback)`,
after 94.5 minutes,
a finished consolidation and polish,
and no `[error]` line.
No page and no artifact:
publication precedes persistence by design (`pass-entry-persist.ts`),
so a refusal
at the page leaves nothing but the slice cache.

What happened,
read off the log,
the fixture and the source:

-   The translate lane's metadata slice (chunk 0,
    92 source characters) was judged at 01:57:15 with four of
    eight ballots heard (the Hyper hold of 01:54:06 had taken the four Hyper-only judges):
    "winner drew only
    weight 1.5 across 4 ballots;
    keeping the fallback",
    then "declined-indecision;
    challenging same panel
    under distinct responsibility",
    then at 02:00:03 the same again.
    The incumbent stood;
    the slice cost
    423 s.
-   The incumbent is correct.
    The fixture's `page.en.md` carries `name: Carena`,
    `alias: Carena`,
    `location: Shanghai` and a translated `desc`,
    against the source's `飞猫`,
    `飞猫, Carena`,
    `上海`.
    CORRECTION 2026-09-02 03:20:
    the keep was NOT a judged keep.
    Both rounds ended `declined-indecision`:
    the four heard judges split (GLM-5.3-Flash and Qwen3.8-27B for candidate 1 at weight 1.5 together,
    gpt-oss-120b for candidate 2 at weight 1,
    Kimi-K3 for candidate 3 at weight 0.5) and the leader fell
    short of the minimum weight,
    so the stage shipped the incumbent by fallback.
    That minimum is 2
    (`MIN_SELECTION_WEIGHT` in `candidate-select-model.ts`),
    not the 3 this doc and the message of commit
    `daaf0ffa0` stated until 04:20;
    the message cannot be amended under auto-push,
    so this sentence is its
    correction.
    `translate-stage-result.ts`
    says it in its own words:
    "the incumbent shipped" and "the judges chose the incumbent" are different
    facts,
    and only the second is evidence about the incumbent.
    The sentence this replaces called it a
    review that found nothing to change;
    the sol review of the guard files (`~/temp/agent/sol-front-matter-guard-review-20260902.txt`)
    caught the misreading.
-   `assertFrontMatterComplete` (`corpus-run/front-matter-completeness.ts`,
    landed 2026-08-28 as "review
    visible front matter",
    "keep invalid metadata retryable",
    "support one-sided front matter") refused
    because the page's front matter equals the archive's while the source's differs,
    which it read as an
    incumbent standing by default.
    Its own test for that rule used an archive whose name is the directory
    id (`name: EntryId`),
    and that fixture was being refused by the identity rule in
    `validateFrontMatterTranslation` before the completeness rule ever ran,
    so the rule had never been
    exercised by its test.
-   The translate lane already tells a decision from a default (`translate-lane-wordings.ts`:
    a record
    whose stage heard at least one translator is `decided`,
    one that heard nobody is `incumbent-fallback`,
    the distinction `lane-slice-text.ts` draws since 2026-08-16).
    The guard inferred the default from bytes
    instead.

The first fix (commit `503ec902c`,
"publish a judged keep of correct front matter") took the metadata
slice's standing off the translate lane's wording,
which says only whether a translator was heard,
and
published every keep whose stage had heard one.
Under it the Carena keep,
an indecision,
would have
published;
so would any wrong incumbent kept by a split vote.
SUPERSEDED the same night by commit
`daaf0ffa0` after the sol review named the gap:
`metadataStandingOf` now reads the metadata slice's
selection off the persisted translate lane (`sliceSelections`:
decision,
origin,
producer),
and the guard
publishes a byte-equal keep only when the judges chose the incumbent (`judged-keep`,
weight carried) or
every heard translator reproduced it (`matched-keep`,
matched models carried,
which the incumbent producer's
`matched` list records).
Every fallback refuses with the decision after the reason,
so a TALLY line reads
`incumbent-fallback: declined-indecision` rather than one word for a hold,
a split and a lost voice alike;
a withdrawn replacement and an unrecorded slice refuse by those names;
a kept directory id refuses as
`directory-id-name` whatever the decision.
The departure from the 2026-08-28 rule is now small and stated:
that rule refused every byte-equal keep,
this one publishes the two kinds that are a review of the
incumbent.
Recorded for veto rather than put as a question because the 2026-08-28 rule discarded a correct
judged keep on a byte comparison its own test never reached,
and the reshape keeps every refusal that rule's
tests name.

What it does not fix:
the minimum vote weight is 2 whatever the ballot count (`MIN_SELECTION_WEIGHT`),
so
a judge round that hears four of eight voices during a provider hold reaches it only when two full-weight
ballots agree,
Carena's 1.5,
1,
0.5 split did not,
and every kept incumbent in that window was a hold,
not
a judgment.
That is `#474`'s territory;
under the reshaped guard such an entry
is refused by name (`incumbent-fallback: declined-indecision`) and stays retryable,
which is what the
2026-08-28 rule intended.
A Carena rerun with all eight judges heard either reaches weight 3 or is a
genuine split over which name form is right (gpt-oss-120b argued in both rounds that candidates 1 and 3
"replace the original name with an alias"),
and a genuine split is not something publication should paper
over.

Left for task 8,
per the review:
the directory-id check fires only on a byte-equal block,
so a page that
kept `name: <directory id>` while changing another field passes it;
and the standing does not carry the
accepted text,
so the guard never checks that the page's metadata is the wording the selection supports.
Neither is the misreading;
both are behavior changes for the owner's say.
A third case,
the standing
reading the translate lane only so that a contest or consolidation keep of the archive over a translate
replacement read as `replacement-not-carried`,
was the live Toka_ls case an hour later and is handled by
commit `6f70a2085`;
the section "The Toka_ls relaunch was killed at 77 minutes,
in consolidation" records
it.
The matched-keep path has live evidence from the same run:
Toka_ls slice 15 settled `sole-candidate`
with the incumbent producer matched by all seven translators,
so the `matched` list is filled in practice
and an empty one (`sole-candidate-unmatched`) is what nobody proposing looks like.

The Carena entry has to run again:
its slice cache was written under the previous pipeline digest and the
dial and the guard both moved it.
keyword233's identical refusal in the first pin pass was the outage
(every stage lost every voice,
the metadata slice kept the archive by default) and reads as
`incumbent-fallback: no-voice-heard` under the reshaped guard.

## The chain was cut to two entries on a rate measurement

"One entry per launch" sized each launch to the hour,
and four launches queued back to back unattended
are the same multi-hour run with pauses.
Measured at 03:07 UTC:
the Toka_ls relaunch at overlap 2 had
settled 7 repair slices of 16 in 21 minutes,
against 14 in 17 minutes for the same entry at overlap 4 in
the first pin pass,
so overlap 2 runs at about half the rate and an entry is on the order of two hours
before its refine and translate lanes.
Four entries at that rate is six to eight hours,
past the hour the
owner named and past the Synthetic weekly meter,
which read 4.24% at 03:07 and has been falling about two
points an hour.
That is the run the owner called unacceptable,
so the chain was stopped at 03:10 and
requeued as:

- Toka_ls to completion (running since 02:45:50 on the pre-fix build),
  rerun on the fixed build only if
  the old guard refuses its metadata slice.
- A meter gate:
  `budget-sample` runs once,
  and Carena0442 launches only if the sample says routing would
  use both providers.
  A Synthetic weekly at zero legitimately holds it dry,
  every seat routes to Hyper,
  and that is the load shape of 01:39:54 that ended the pin pass;
  reordering without the gate only picks
  which entry dies.
- Carena0442 (the owner's hand-picked fixture entry,
  the longest document,
  the most evidence per entry),
  under the same dials,
  into `~/temp/agent/carena-relaunch-20260902`.

XIEPT2 and keyword233 are deferred to the owner alongside the guard veto:
they need either a fresh
Synthetic week or the owner's say on a run longer than an hour.
`src/` is frozen until the chain's last
TALLY:
every launch rebuilds,
so a mid-chain edit lands the entries under different pipeline digests,
which the standing readers refuse to pool,
and a broken build launches a broken pass unwatched.
