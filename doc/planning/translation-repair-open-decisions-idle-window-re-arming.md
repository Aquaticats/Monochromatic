# Translation repair open decisions: re-arming the idle windows below the deadline (question 10)

Question 10 on whether to arm the first-byte and idle windows below the 360000 ms deadline,
what a false trip costs,
and the three options with the ranking.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

Note of 2026-10-21 (UTC):
the windows were armed at 180000 ms (first content) and 120000 ms (idle) and count generated text only.
The constants this question names as `STREAM_FIRST_BYTE_MS` and `STREAM_IDLE_MS` at 600000 are
`STREAM_FIRST_CONTENT_MS` at 180000 and `STREAM_IDLE_MS` at 120000;
see "The windows armed on 2026-10-21" in
`doc/decision/translation-repair-runaway-call-termination-stall-window-premise-and-straggler-cuts-remeasured.md`.
The rest of this record is kept as it was written.

## Question 10: whether to re-arm the idle windows below the deadline

`STREAM_FIRST_BYTE_MS` and `STREAM_IDLE_MS` stay at 600000 after `#121`'s re-derivation
(`doc/decision/translation-repair-runaway-call-termination.md`),
which corrected the STATED REASON
for that value without finding a number below it that clears every regime measured.
This question
is the one the correction explicitly left open:
given the median is now fast,
is any of the
360000 ms the deadline currently spends on a dead call worth trying to reclaim.

WHAT A FALSE TRIP ACTUALLY COSTS,
which softens but does not remove the risk:
`stream-idle-guard.ts`
aborts on a controller it owns rather than the caller's signal,
so `exchangeWithRetry` treats a
tripped guard as transient and re-dispatches at transport level on a roughly one-second backoff.
No voice is lost the way a straggler-grace cut loses one.
What IS spent is a full fresh call:
real
wall-clock time and quota,
and if the model reliably pauses near the same point in its own reply,
a retry can walk into the same trip again rather than clearing it.

### Options

A.
 **Leave both constants at 600000,
effectively disabled.**
(Current state,
landed in `#121`.)
    -   For:
zero demonstrated risk against every regime read,
including the closest historical
        near-miss,
a completed call at 347099 ms three weeks before this traffic under a different
        roster.
The 360000 ms deadline already ends a truly dead call within six minutes,
achieving
        the guard's stated purpose with a coarser instrument.
    -   Against:
a call whose first byte never arrives runs the full six minutes even though the
        new median says most of that wait is now abnormal,
and the idle guard exists specifically
        to notice silence faster than the deadline does.

B.
 **Arm `STREAM_FIRST_BYTE_MS` (and `STREAM_IDLE_MS`) at a value grounded only in the CURRENT
    traffic,
past its observed maxima (183755 ms firstByte,
124992 ms gap),
setting aside the
    older RUN 014 regime as no longer representative.**
    -   For:
meaningfully faster than 360000 ms at catching a dead call,
and grounded in the
        largest,
freshest sample read,
7079 streams this month.
    -   Against:
this is a bet that RUN 014's regime,
a completed call at 347099 ms,
cannot recur,
        which is a bet about a two-person provider's infrastructure that the audit explicitly
        declined to make:
different stage shapes,
provider capacity,
routing,
and time of day are
        all unexcluded between the two readings.
A false trip is soft,
a retried call rather than a
        lost voice,
but it is not free:
each one re-buys a whole exchange.

C.
 **Arm both constants just under the deadline,
clearing every regime on record with minimal
    margin (roughly 350000).**
    -   For:
technically active and carries none of option B's regime risk,
since it clears even
        RUN 014's uncensored maximum.
    -   Against:
leaves ten seconds or less of daylight before the deadline would end the same call
        anyway,
so the benefit over option A is bounded by that same margin.
Worth the
        implementation and re-test cost only if there is an operational reason to want the guard to
        fire and log rather than relying on the deadline,
which nothing read for `#121` establishes.

### Ranking

A > C > B.

-   A over C,
    because C costs a real change for a benefit bounded at single-digit seconds per dead
    call;
    it is worth doing only for a reason beyond the seconds saved,
    such as wanting the guard's
    own logging on a dead call rather than the deadline's,
    and nothing gathered here shows that
    reason exists.
-   C over B,
    because C carries zero regime risk,
    clearing even RUN 014's 347099 ms,
    while B is the
    only option that knowingly accepts a documented historical near-failure as unlikely to recur,
    on a provider this codebase has already twice caught treating a small sample's maximum as a
    bound.

WHAT I WOULD DO IF YOU DELEGATE THIS:
A,
unchanged.
Revisit C only if there is a stated operational
reason to want the guard armed for its own sake,
and revisit B only after the July-to-August regime
shift gets an actual explanation,
which nothing read for `#121` provides.

DELEGATE
