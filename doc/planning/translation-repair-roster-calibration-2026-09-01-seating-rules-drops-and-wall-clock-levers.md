# Roster calibration: the seating rules, the drops made on the owner's authorization and the wall-clock levers

The seating rules written before the 40-slice numbers arrived;
the owner's authorization to drop a model from a role and the drops made on it;
the wall-clock measurement behind the fast-iteration principle,
and the stale text the seating edit fixes.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## Seating rules, written before the 40-slice numbers arrive

Revised at 22:10Z on the reviewer's points,
while the 40-slice run stood at slice 6 of 40 with no refiner round
printed yet,
so still before any number they decide on existed.
Revised again at 22:25Z (slice 12 of 40) to remove the numeric gates on the owner's note that testing against a
magic number is discouraged:
the rules now rank and report,
and the numbers beside a rank are evidence a reader
weighs,
not thresholds a rule trips on.

1.  Editors are seated from the 40-slice EDITOR standing alone.
    The 6-slice run is a same-day replicate that shows the band;
    its editor rounds are never pooled in.
2.  Editors:
    the top three by availability-adjusted share of disinterested ballots in the 40-slice EDITOR
    standing (printed share times candidates over rounds;
    every round seats the same judges,
    so the product weights rounds about equally).
    Two readings go beside the rank that the ballot-level z cannot give:
    the slice-clustered round-win bootstrap
    (round-win share and top-three inclusion probability),
    and production-window reliability from the
    180000 ms instrument (voices lost per ask in the producer calibration,
    by role).
    Where the third and fourth seats are not separated by the bootstrap,
    the seat goes to the one that lost
    fewer voices under the production window,
    and the record says the seat was decided on reliability.
3.  Refiners:
    the REFINER rounds of the 6-slice and 40-slice runs are pooled as slice clusters
    (same build,
    same instrument,
    same 300000 ms window;
    slices from two strides are distinct clusters),
    and the refiners are the top three by availability-adjusted share over the pooled rounds,
    with the same two beside-readings and the same reliability tie-break.
    The record states how many slices reached a rewriter;
    if none did,
    the refiners are the editors,
    recorded as an unvalidated transfer assumption,
    and a refiner-targeted calibration is filed as follow-up.
4.  Checkers stay Qwen3.8-27B,
    Kimi-K3 and gpt-oss-120b.
    No instrument ranks checkers:
    `checker-sensitivity` asks whether checkers can say no on fixtures,
    and the `run-config.ts` note already says the writer and editor instruments are far from checking.
    The gap is recorded as a gap.
5.  Wide roles (critics,
    panel,
    judges,
    translators) stay the whole roster.
    Only the owner blocklist removes a model from the roster;
    writers under the null still judge,
    since neither instrument measures judge accuracy.
    One question goes to the owner rather than being decided here:
    the producer standing places
    `gpt-oss-120b` and `deepseek-v4-flash-0731` under the null as writers with full availability
    (40 of 40 candidates each),
    which is a quality finding about the translator seat specifically.
    `RUN_TRANSLATE_MODELS` was measured at full width,
    so narrowing it is a design change,
    not a seating.
6.  `glm-5.3`'s reliability is read off the 180000 ms instrument,
    because that is the corpus pass's window,
    and by role:
    10 percent of translate asks lost as a writer,
    29 percent of select asks lost as a judge.

## Owner authorization of 2026-09-01 and the drops made on it

The owner wrote mid-run:
"I authorize you to drop any model from any role,
as long as you have evidence.
These doesn't count as design decisions."
That settles the question rule 5 had put to the owner and narrows the owner-blocklist-only clause to roster
membership:
a role drop on measured evidence is part of this seating.

Latency of completed streams per model,
from the two logs (`>180 s` counts completed streams that ran past the
production window's length,
an upper bound on further cuts under it,
since the window starts at quorum):

-   `glm-5.3`:
    producer run p50 61 s,
    p90 166 s,
    max 221 s,
    14 cut and 4 completed past 180 s of 77;
    editor run p50 66 s,
    p90 172 s,
    max 278 s,
    2 cut and 4 past 180 s of 55.
-   `GLM-5.3-Flash`:
    producer run p50 66 s,
    p90 153 s,
    6 cut and 4 past 180 s of 83;
    editor run p50 52 s,
    p90 199 s,
    none cut and 6 past 180 s of 54.
-   `Kimi-K3`:
    producer p50 26 s,
    p90 104 s,
    1 cut and 1 past 180 s;
    `Qwen3.8-27B`:
    p50 33 s,
    p90 77 s,
    none.
-   Every other model:
    p90 under 50 s,
    nothing past 120 s except two gemma streams in the editor run.

Schema-mismatch lines by model and stage,
recovery re-asks included:
`deepseek-v4-flash-0731` at select 8 (producer) and 4 (editor),
every one a numeric string where the guard wants
a number (`"best": "8"`);
`Qwen3.8-27B` at select 2 (editor run,
one answer with a bad control character).

Drops made before the 40-slice standing:

-   Translator seat:
    `hf:openai/gpt-oss-120b` and `deepseek-v4-flash-0731` leave
    `RUN_TRANSLATE_MODELS.translatorModelIds`.
    Evidence:
    the 40-round producer standing,
    5 of 207 and 5 of 208 disinterested ballots,
    z -4.53 and -4.55
    against the pooled null,
    at 40 of 40 candidates each,
    so the finding is about writing,
    not availability.
    Both keep every other seat.
    With seven translators and eight judges every slate keeps at least two disinterested judges
    (`assertJudgeableProducerRoster`'s floor) and the translator quorum is 4 of 7.
-   Judge seat,
    both lanes:
    `glm-5.3` leaves `judgeModelIds`.
    Evidence:
    under the production window it lost 11 of 38 select asks (29 percent),
    each loss holding the round
    for the whole window after quorum and delivering about 2 M raw characters of reasoning that never reached an
    answer,
    at the roster's highest output rate;
    its completed select streams are the roster's slowest.
    It keeps the critic,
    panel and translator seats,
    where no production-window measurement exists
    (the editor calibration ran at 300000 ms and lost 1 of 54 there);
    the four-entry pass logs abandons by role
    and supplies that evidence.
-   Not dropped:
    `deepseek-v4-flash-0731` as a judge loses about a tenth of its select ballots to the numeric
    string shape,
    a parser tolerance question filed as follow-up rather than evidence about its judgment;
    `GLM-5.3-Flash` as a judge lost 6 of 83 asks (7 percent) under the production window and none under
    300000 ms,
    and stays.

## Wall clock, and the owner's fast-iteration principle

The owner,
on seeing the pass's `hard=25200000ms` (the 420-minute per-entry cap raised on 2026-08-17 after
entries were measured clearing at seven hours):
"A 7h run is pretty unacceptable,
since it goes against the
principle of fast iteration at this early stage of the project."
Recorded as the `FIT` rule in `AGENTS.md`.

Where the time goes,
measured on the two calibration logs and the first hours of the 40-slice run
(`<stage> round: N/M heard, T total, Q to quorum, G in grace` lines):

-   Producer calibration,
    80 rounds:
    83 percent of summed round time is waiting after quorum
    (select rounds mean 138 s with 25 s to quorum;
    translate rounds 98 s with 14 s to quorum).
-   Editor calibration,
    first 81 rounds of the 40-slice run:
    75 percent
    (panel 213 s with 59 s to quorum;
    select 91 s with 17 s;
    critic 68 s with 20 s;
    editor 69 s with 19 s).
-   In the rounds where everyone answered,
    the last voice arrived a median 46 to 66 s after quorum,
    p90 141 to 158 s.
    A 60 s window keeps every voice in 43 to 62 percent of such rounds,
    90 s in 62 to 72,
    120 s in 74 to 82;
    the voices a shorter window drops are the two GLM models,
    whose completed streams run
    p50 52 to 66 s and p90 153 to 199 s against p90s under 110 s for every other seat.

Levers applied on that measurement,
all within the role-drop authorization or the launch-time overrides the
drivers already carry:

-   `glm-5.3` leaves every nine-wide seat (critic,
    panel,
    judge),
    landed at the commit that introduced
    `RUN_WIDE_SEATS`;
    the eight remaining reach quorum at 4.
-   The four-entry pass launches with `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=60000` and
    `TRANSLATION_REPAIR_SLICE_OVERLAP=4`,
    the owner's original 60 s figure of 2026-08-14 now backed by the
    last-voice distribution,
    and the overlap the calibration arms measured at half the wall clock.
    The built-in constants are untouched;
    `doc/decision/translation-repair-straggler-grace.md` keeps the 180 s
    built-in until a decision moves it,
    and the pass prints both overrides at launch.
-   The two launches run concurrently rather than one after the other.
-   The 40-slice calibration in flight is left to finish (about two hours at its 300 s window);
    its standing is computed in memory at the end,
    so stopping it would keep only the winner lines.

What a 60 s window costs is measured by the pass itself:
it logs every abandon by model and stage,
and the standing of the pass's own rounds says whether the dropped voices were the ones that mattered.

## Stale text the seating edit fixes in `run-config.ts`

-   `RUN_ROSTER` says "EIGHT MODELS NOW";
    the roster is nine.
-   `RUN_READER_MODELS` says "Four of the eight read images";
    four of the nine do.
-   The quorum paragraph stops at ten;
    nine needs 5 voices and puts `minBallotWeight` at 3 of 9.
-   If two GLM-family models take editor seats,
    the provider-coverage paragraph must also name the family
    correlation,
    since it currently argues from providers alone.
