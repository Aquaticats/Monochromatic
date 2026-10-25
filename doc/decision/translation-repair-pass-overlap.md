# The corpus pass keeps four slices in flight when nothing overrides it

Taken 2026-09-06 on the four matched pairs the overlap-dial work asked for,
all measured on 2026-08-27 and 2026-08-28 and recorded in
`package/translation-repair/README.md` and
`doc/planning/translation-repair-corpus-overlap-measurement.md`,
and never acted on because the 2026-09-01 hold froze the legacy pipeline
before the reading was turned into a default.

THIS MOVES A VALUE THE OWNER LEFT AT ONE.
Question 11 of `doc/planning/translation-repair-open-decisions.md` was answered on 2026-08-26 with option A:
the editor calibration goes to four,
and,
in the sheet's record of it,
the pass keeps one slice in flight until the overlap dial is measured on the pass.
That deferral asked for evidence on the pass itself,
not for a preference,
and the evidence has existed since 2026-08-28.
It is flagged here for a cheap veto:
`TRANSLATION_REPAIR_SLICE_OVERLAP=1` reproduces the sequential driver for any launch,
and `PASS_OVERLAP` in `package/translation-repair/corpus-pass/src/corpus-run/pass-overlap.ts` is one number.

## What was decided

-   `corpus-pass` keeps four slices in flight in every per-slice driver when
    `TRANSLATION_REPAIR_SLICE_OVERLAP` is unset
    (`PASS_OVERLAP` in `src/corpus-run/pass-overlap.ts`).
    The variable still overrides it in either direction.
-   The launch line still prints `OVERLAP <entry> value=<n> source=<fallback or variable>`,
    so a log says which driver ran.
    Since 2026-10-25 it carries `at=<time> attempt=<n>` after the entry,
    as every per-entry line of the pass does (ledger B960).
-   The straggler window is not moved by this record.
    The round window is the owner's 120000 ms of 2026-09-03
    (`doc/decision/translation-repair-straggler-grace.md`),
    and the writer rounds' 180000 ms remains a launch dial
    whose built-in status that record names as the owner's call.

## The evidence

Every pair in this section ran both arms on one pipeline digest into separate run roots,
with both subscription providers wet throughout,
and is read as wall clock over the sum of stream time,
since single-run wall clock moves 37 percent on provider speed alone
(`doc/decision/translation-repair-calibration-overlap.md`,
arm A against arm A2).
The run-to-run band on that normalized figure is about 0.03.

-   `keyword233`,
    three slices,
    the smoke pair.
    Overlap 1:
    38.50 minutes over 1.68 call-hours,
    normalized 0.382,
    9 voices unheard.
    Overlap 4:
    31.12 minutes over 1.78 call-hours,
    normalized 0.291,
    8 unheard.
    Wall down 19.2 percent,
    normalized down 23.9 percent.
-   `Toka_ls`,
    fifteen slices.
    Overlap 1:
    313.24 minutes over 12.44 call-hours,
    normalized 0.420,
    61 unheard,
    peak 10 calls in flight.
    Overlap 4:
    104.37 minutes over 11.82 call-hours,
    normalized 0.147,
    the same 61 unheard,
    peak 37 in flight.
    Wall down 66.7 percent,
    normalized down 64.9 percent,
    call sum down 5.0 percent.
-   `Zha_Ke`,
    four slices.
    Overlap 1:
    129.95 minutes over 4.862 call-hours,
    normalized 0.445,
    68 unheard.
    Overlap 4:
    36.49 minutes over 2.947 call-hours,
    normalized 0.206,
    6 unheard.
    Wall down 71.9 percent,
    normalized down 53.7 percent.
-   `Weideriche_`,
    first attempts,
    both arms ending at the same truncated-completion pairing refusal rather than a page.
    Overlap 1:
    54.04 minutes over 2.293 call-hours,
    normalized 0.393,
    15 unheard.
    Overlap 4:
    31.51 minutes over 2.502 call-hours,
    normalized 0.210,
    10 unheard.
    Wall down 41.7 percent,
    normalized down 46.6 percent.
-   `ArtsEpiphany`,
    one slice,
    the null control:
    0.268 against 0.325,
    byte-identical pages,
    nothing to overlap.
    Provider variation,
    as expected.

The normalized effect is 0.091,
0.273,
0.239 and 0.183 on the four pairs that had slices to overlap,
which is three to nine bands wide,
in the same direction every time,
and the voice count never worsened.

Since the dial landed,
every page that shipped ran at four through the variable:
`luxuanwen3` in 60 minutes,
`SS3B_0016` in 56,
`Uekawakuyuurei` in 53 and `MTF_0615` in 126 on 2026-09-04,
and the fixed-build `Toka_ls` pages of 2026-08-27.
The last entry settled at overlap 1 was `Weideriche_` on 2026-08-28.
The owner named a seven-hour run unacceptable;
`Toka_ls` at overlap 1 took 5.2 hours.

## What it costs

Concurrency overflows the subscription seats,
and the overflow is served by whichever provider is next in the routing order:
Hyper on the 2026-08-27 pairs (metered spend up 104.2 percent on `Toka_ls`,
76.3 percent on `Zha_Ke`,
137.5 percent on `Weideriche_`),
and since 2026-09-03 OpenRouter after Hyper,
which bills per token.
Under the quality-over-cost guideline that is a measured routing consequence and not a constraint
(`doc/decision/translation-repair-openrouter-fallback.md`),
and `SPEND_CEILING_USD` bounds what one run can put on OpenRouter.
It is recorded so that a reader of a spend report knows where the paid calls came from.

## What this does not decide

-   Whether writer rounds keep a longer window than reader rounds by default.
    `doc/decision/translation-repair-straggler-grace.md`,
    addendum of 2026-09-02,
    leaves that to the owner.
-   Whether four is the best value.
    No pair ran at two or eight;
    the launches of 2026-09-02 at two and three
    (`~/temp/agent/toka-rerun-20260902.log`,
    `toka-rerun2-20260902.log`) were relaunches under a starved Synthetic meter,
    not matched arms.
    The variable exists so a later matched pair can move it.
