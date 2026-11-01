# Translation repair handover 2026-09-06: the queue, items two to seven

Items two to seven of the numbered queue:
the one-transcript decision,
the queue order,
the roster,
the published check,
the components and the Synthetic meter.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## Queue items two to seven

2.  DECIDED (owner,
    2026-09-16,
    "I'll go with whatever you recommend"):
    the archive review reads one transcript per corroborated picture,
    the one the other readers carry most,
    landed in `36b79bfa0` and recorded in
    [the one-transcript decision](../decision/translation-repair-archive-review-one-transcript-2026-09-16.md).
    MEASURED on Mio15 against Mio14
    (the pass log's "Mio15 read" heading):
    ordinary blocks cost fewer cap cuts (5 over 4 blocks against 15 over 6),
    the first chat block is unchanged (7 of 12 seats unreadable against 6),
    so the lever is spent on the block that matters.
    The doubled re-ask cap is the open owner question,
    put to the owner with the Mio15 read on 2026-09-16;
    do not build it unasked.
3.  THEN THE QUEUE AS BEFORE,
    one entry at a time,
    each read before the next is launched:
    `hulicaijia` (the ruby,
    the Sakura line,
    the footnote;
    its 2026-09-09 INCOMPLETE was class twenty-six,
    fixed in `40aba2fdb`),
    then `shi_Yumiaoya`,
    `yingying` and `XingZ60`.
    Freeze the dist under `node_modules/.frozen-dist-<commit>` and launch with
    `launch-pass.mjs <runs-dir> <entry> <frozen-dir>`.
    V4.1 Flash was seated as a writer by measurement on 2026-09-19
    (one `producer-calibrate 40 --candidates deepseek-v4.1-flash` run,
    1h52m,
    about 0.24 USD,
    read with `read-standing.mjs`,
    seated by the pooled-null rule).
    A calibration never runs concurrently with a pass.

4.  THE ROSTER STANDS AS THE CALIBRATIONS LEFT IT (`028432713`,
    `a5e0efc7f`,
    `9f8dcccd2`;
    the addenda in `doc/decision/translation-repair-roster-seating-2026-09-01.md`).
    Eleven seats,
    eight translators (V4.1 Flash in,
    Mercury and the anchor judge out),
    eleven consolidation writers,
    stage quorum 4.
    The owner curates additions to the community glossary,
    but the handoff's claim that the file was empty was wrong.
    `git show 2cfad6236:package/module/translation-repair/src/community-glossary.ts` (moved on 2026-10-18 to
    `package/translation-repair/glossary/src/community-glossary.ts`) already carries
    自切 as `self-surgery` and 超天酱 as `KAngel` or `Needy Streamer Overload`.
    A call through frozen `5b228736b` emits both terms for `yuki418330012` and none for `Mio`,
    whose source contains neither.
    No glossary seed is pending and no source change was needed.
    `gemma-4-26b-a4b-it` read z +6.39 above the pooled null on the latest calibration.
    This was not its first upward result:
    the seating decision's 2026-09-08 addendum records z +4.14,
    beside Qwen's +5.25.
    Whether upward producer evidence should change weighting or seat breadth remains open;
    no weighting change has been made.
5.  `verify-published` on a runs dir holding only declines prints `NOTHING VERIFIED` and exits 2;
    right for any corpus pass (pages stand beside the declines),
    wrong on a decline-only dir;
    a one-line change when it matters.
6.  Then the components no read page has met
    (the double-quoted path is met by `Arita`).
7.  Synthetic's weekly meter is a ROLLING WINDOW,
    not a calendar week (measured 2026-09-08:
    0 percent at 12:26 UTC after the ninth pass spent its 5.8 percent,
    2 percent at 13:07 UTC with no top-up,
    then falling again as the calibration spent it),
    so it returns on its own as the oldest usage ages out and reads dry only while it stands at zero,
    while its five-hour window refills on its own
    (2750 of 2750 at the sixth pass's tally,
    wet at one reading of six);
    Hyper's daily quota is unpublished and closed once today after about a thousand requests of one pass,
    and a pass that meets it holds Hyper out until the instant the refusal names.
    At 18:20 UTC on 2026-09-09 (the seventh `Mio` launch):
    OpenRouter 271.65 USD (the owner's final top-up;
    the third `noname` spent 1.87,
    the Mercury calibration 0.98 by round 21),
    Bedrock 186.49 USD (0.19 on that pass,
    8.77 today),
    Synthetic at zero of its rolling week since 17:25 UTC,
    Hyper 0 since 12:45 UTC on 2026-09-08 and never to be recharged (the owner's words);
    the whole account is under "Costs".
    Bedrock and OpenRouter are the two wet providers and together reach every bench at every phase,
    measured on the merged build;
    OpenRouter is the one whose spend is per token.
