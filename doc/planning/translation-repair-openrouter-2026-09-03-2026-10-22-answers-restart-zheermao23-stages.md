# OpenRouter and the corpus passes: the stages of zheermao23 on build 8864316eb2 (2026-10-22)

What the stages of `zheermao23` (entry `zheermao101`) did on 2026-10-22:
the stage outcomes,
the verdict of every contested slice,
whether the new rulings fired,
the recurrence of the known classes,
the side checks
and the open items.
The run facts,
the NeuralWatt meter,
the seats,
the Mercury recheck and why the repair counts rose are in
[the run of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-run.md),
and the log line behind every figure is in
[the evidence of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-evidence.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## zheermao23 stages, 2026-10-22

### Stage outcomes

Preparation.

- The reference step:
  `REFERENCE 1` is a Telegram post that the cache returned with 25 characters (line 214);
  the `ATTESTED` step then dropped the `gemma-4-26b-a4b-it` reading because the 60-character quote
  "I am ArtsEpiphany,
  an admin from the online forum Transland."
  was found in the archive and not in the reference
  (line 230),
  and ended with `heard=6 answered=1 verified=0 needed=3` (line 231).
  A 25-character reference cannot hold a 60-character quote,
  so the drop follows from the short cached text;
  why the cache holds 25 characters was not traced.
  The artifact records the drop as
  `reference attestation discarded 1 of 1 item whose quotes were not found word for word`.
- Block pairing:
  section 1 paired 8 of 8 original and 8 of 8 translation blocks (line 245;
  `minimax-m3` returned an unusable pairing naming "translation block 8,
  and there are 8",
  line 246,
  and again at lines 339 and 607);
  section 2 paired 14 of 14 and 14 of 18 (line 260),
  and 14 of 19 once the block reviews had revised the archive (line 624).
  Section 1's pairing round was run twice more from stored replies (`PROMPT-REUSE source=memory`,
  6 lines at 332 to 337 and 6 at 600 to 605;
  1 ms rounds at lines 338 and 606),
  while the rounds of section 2 ran again (32,071 ms at line 354 and 44,820 ms at line 623).
- Pictures:
  3 of 3 gathered (line 269);
  `gemma-4-26b-a4b-it` read the three photos through OpenRouter (531,
  482 and 504 characters;
  log lines 277,
  301 and 323;
  `zheermao22`:
  530,
  478 and 419),
  and each photo was `corroborated by 5 of 5 readers`,
  at overlap 0.988,
  0.990 and 0.973 (lines 288,
  306 and 331).
  `google.gemma-4-31b` failed outright on each (lines 287,
  305 and 330).
- Archive block reviews (the four blocks of the email and the letter):
  - Email (`block/20`,
    24 lines):
    5 of 7 heard after 274,557 ms,
    of which 180,002 ms in grace on a cut Qwen stream (line 385);
    the reply of `inception/mercury-2.5` was dropped because its 416-character quote is no span of the expected section
    (line 387);
    the composite was not built,
    since four revisions overlap on lines 13 and 15 and more (line 388);
    the select round chose candidate 3 of `deepseek-v4.1-flash`,
    which won weight 4.5 (6 ballots,
    line 424),
    and the block was replaced with one note between translators kept (line 427).
  - Email label (`block/19`):
    5 editorial-context replies were dropped (lines 449 to 453),
    one admissible revision remained (line 454),
    and candidate 1 of Kimi won weight 5.5 (line 482).
  - Letter (`block/14`,
    36 lines):
    the composite was not built,
    since two revisions overlap on 13 lines (line 509);
    the select round heard 5 of 6 after 206,799 ms,
    of which 120,001 ms in grace on a cut Qwen stream (lines 536 and 545);
    candidate 3,
    the incumbent,
    won weight 3 (5 ballots,
    line 543),
    and the artifact records the block as `reviewed and retained`.
    So the archive letter stood.
    In `zheermao22` the letter shipped one reviewer's partial revision.
  - Letter label (`block/13`,
    1 line):
    3 editorial-context replies were dropped (lines 566 to 568),
    the composite was not built (the two revisions overlap on the one line,
    line 569),
    and candidate 2,
    credited to `glm-5.3` and `deepseek-v4.1-flash` together,
    won weight 4 (line 598).
    A `composite(a + b)` label on a select line names one text that two or more producers proposed:
    the ledger files carry it as a `producers` array
    (24 of the candidates in the 40 ledger files have more than one producer;
    for example `["hf:zai-org/GLM-5.3-Flash","deepseek-v4.1-flash"]` in the first ledger file,
    matching the line 1213 label).
    It is a different thing from the `compositeOfDisjointRevisions` step of lines 388,
    509 and 569,
    which built nothing.
  - The three composite refusals are all overlap refusals;
    none is the line-count refusal of `zheermao22` (23 against 24 and 34 against 36 lines).
  - Findings recorded:
    12,
    12,
    10 and 11 (lines 425,
    483,
    544 and 599).

Lanes.

- Both lanes began at 12:50:35 UTC over 15 slices and 3 aligned units with 63 alignment findings (lines 634 and 635).
- Repair lane:
  status `repaired`,
  slices changed 0,
  2,
  3,
  4,
  5,
  8,
  9,
  11 and 14 (9);
  21 of 21 accepted issues resolved;
  14 of 14 introduced-defect probe rounds `3/3 heard`;
  no refinement rolled back.
- Translate lane:
  status `complete`,
  slices changed 0,
  2,
  3,
  4,
  5,
  6,
  9,
  11 and 12 (9).
  The slice boundaries are those of `zheermao22`
  (58 and 67 source characters for slices 1 and 2;
  80 and 45 in `zheermao21`).
- The lane texts differ in 10 of 15 slices (line 3154);
  the five that do not are slices 1,
  2,
  7,
  10 and 13.

### Slice verdicts and disputes

Per contested slice,
the lane contest verdict (ballot lines),
then the consolidation terminal and the polish gate
(from the artifact's `laneSelection` and `consolidation` fields and the gate lines):

- Slice 0:
  `translate` won,
  5 of 6 usable (line 3283);
  terminal `slate-endorsed-standing`;
  no gate;
  polish unchanged.
- Slice 3:
  `translate` won,
  4 of 6 usable (3334);
  `slate-endorsed-standing`;
  polish unchanged.
- Slice 4:
  `translate` won,
  5 of 5 (3230);
  `consolidated`,
  gate `consolidated` 4 of 5 (3633);
  polish unchanged.
- Slice 5:
  `translate` won,
  5 of 6 (3197);
  `consolidated`,
  gate `consolidated` 4 of 5 (3469);
  polish unchanged.
- Slice 6:
  `translate` won,
  5 of 6 (3219);
  `slate-endorsed-standing`;
  polish changed,
  `polished 5, base 0` (3815).
- Slice 8:
  `translate` won,
  5 of 6 (3257);
  `slate-unjudged-standing`;
  polish changed,
  `polished 5, base 0` (3851).
- Slice 9:
  `translate` won,
  5 of 6 (3271);
  `slate-unjudged-standing`;
  polish unchanged.
- Slice 11:
  settled on `neither`,
  archive declined,
  5 of 6 (3308);
  `consolidated`,
  gate `neither` 4 of 5,
  "settled on neither over a standing a majority of contest ballots called flawed,
  so the proposal the slate chose ships" (4000 and 4001);
  polish unchanged.
- Slice 12:
  `translate` won,
  5 of 6 (3322);
  `gate-kept-standing`,
  gate `neither` 4 of 5 (4040);
  polish changed,
  `polished 3, base 2` (4158).
- Slice 14:
  settled on `neither`,
  archive endorsed,
  4 of 6 (3342);
  `gate-kept-standing`,
  gate `neither` 4 of 5 (4050);
  polish changed,
  `polished 4, base 1` (4147).

Totals:
8 slices went to the translate lane,
2 settled on neither,
and none went to the repair lane
(`selection=contested`).
The consolidation terminals read 3 `slate-endorsed-standing` (slices 0,
3 and 6),
2 `slate-unjudged-standing` (slices 8 and 9),
3 `consolidated` (slices 4,
5 and 11) and
2 `gate-kept-standing` (slices 12 and 14).
Four polish gates ran (slices 6,
8,
12 and 14),
and all four shipped the polished text.
On slices 12 and 14 the consolidation gate kept the standing and the polish then changed it;
the polished wording is on the shipped page for slices 6,
12 and 14 ("The two were close" at page line 43,
"she held to her decision... and in the end" at line 140,
"all that remains is a long sigh..." at line 148),
and slice 8's at line 91,
so the gate-then-polish order is what shipped.

Disputes.

- `translate-archive-disputed` printed for slices 0,
  9 and 11 (lines 2682 to 2684),
  with 4,
  4 and 5 accepted disputing claims
  ("accuracy/addition at any severity,
  any other accuracy claim the panel settled at major or worse"),
  and again at the start of each slice's consolidation (lines 3353,
  3708 and 3839).
  On all three the log words it as "the repair lane's text stands in for [the archive rendering]"
  (classes one hundred seven and one hundred seventy-six),
  and at consolidation as "the stand-in is the consolidation's incumbent".
- `introduced repetition in slices 9, 11 (archive 0, shipped 2): "One Among Us members"` printed from both lanes
  (lines 2676 and 3144).
- What the contest ballots said of the archive rendering (a script over the `lane contest ballot` lines):
  `flawed` on all ballots of slices 0 (5),
  3 (4),
  4 (5),
  5 (5),
  6 (5),
  9 (5) and 11 (5);
  `publishable` on all ballots of slice 8 (5) and slice 14 (4);
  and split on slice 12 (3 flawed,
  2 publishable).
  On slice 11,
  where a strict majority called the archive flawed,
  the log announces that a tied slate is run off (line 3900),
  but no tie occurred.

### Whether the new rulings fired

The rulings and fixes since `zheermao22` are those the `XingZ6017` read names:
the first-content window of 180 s (B804),
the notes passed to the polish judges and writers (B805 to B807),
the composite of archive revisions by line difference (B803),
the recalibrated NeuralWatt estimate (B808),
the Mercury first-content window of 90000 ms (B809),
and the examined and rejected hold on re-asking abandoned seats (B810).
The rulings of the `zheermao22` read
(polished text wins ties,
reason before vote everywhere,
tied finalists run off,
Gemma readers lifted,
the KaTeX shim)
stand.

- Reason before vote.
  The key is now `assessment`,
  not `reason`.
  A script over the 892 reply payloads finds no top-level `reason` key on any provider.
  Of the replies that carry an `assessment` and a vote key (`best`,
  `choice`,
  `acceptable`,
  `isAcceptable`,
  `verdict`,
  `verdictOnArchive` or `disposition`):
  Hyper 250 assessment-first and 0 vote-first,
  OpenRouter 75 and 0,
  Synthetic 41 and 0.
  So the ruling is met on every provider,
  Hyper included
  (`zheermao22`:
  Hyper 13 reason-first and 176 vote-first of 189 replies with a `reason` key).
  Whether Hyper's order is the model's or a serialization is still not testable:
  299 of 300 multi-key Hyper objects are in alphabetical order,
  and now so are 99 of 99 on OpenRouter and 72 of 72 on Synthetic,
  because `assessment` sorts before every vote key,
  so alphabetical order and the wanted order coincide.
  The open item U1 (the Hyper sort) therefore cannot be tested from this run.
  The one multi-key Hyper object out of order is `verdicts` before `groups`.
  21 replies did not parse as JSON (Hyper 11,
  OpenRouter 4,
  Synthetic 6),
  the four `max_tokens` replies among them.
- Polished text wins ties.
  Not exercised.
  Four polish gates ran (five in `zheermao22`):
  `polished 5, base 0` (slice 6),
  `polished 5, base 0` (slice 8),
  `polished 4, base 1` (slice 14) and `polished 3, base 2` (slice 12);
  none tied,
  and the new log line names the rule on each:
  `settled on polished by rule polished-lead` (lines 3815,
  3851,
  4147 and 4158).
  This record says "no tie occurred",
  not "worked".
- Tied finalists run off.
  Not exercised.
  No `judges tied` line printed,
  and no run-off round started.
  The one line about it (3900) is the announcement of the rule on slice 11.
- Ledger B731 (the polish gate sheet shows the accepted claims):
  fired.
  Four ballots on slice 8 cite the repair panel's flagged claim
  ("flagged by the repair panel",
  "the repair panel noted"):
  Kimi,
  `gemma-4-26b-a4b-it`,
  `glm-5.3` and `inception/mercury-2.5` (lines 3846,
  3847,
  3848 and 3850;
  a pattern search over the 20 polish ballots).
  The claim is slice 8's one accepted issue
  (`style/awkward-phrasing`,
  "survived with difficulty through rescues"),
  and the shipped page carries the fix at its line 91
  ("but each time she was rescued and pulled through with great difficulty").
  In `zheermao22` the one verified lost claim of the class K3 was on slice 8 too (a different claim).
- B805 to B807 (notes to the polish judges and writers):
  consistent with the slice 8 result,
  where the polish writers' text carries the claim's fix (the artifact's slice 8 `polish` field:
  `changed: true`,
  contributor `hf:zai-org/GLM-5.3-Flash`);
  the writers' sheets were not read.
- B803 (composite by line difference):
  attempted three times (`archive composite not built` at 12:39:48 UTC,
  12:44:45 UTC and 12:49:20 UTC),
  refused three times,
  each time for overlapping revisions,
  and the refusal names the overlapping lines
  (for example "lines 13,
  15 (candidates 1 and 2)").
  It built nothing.
  The line-count refusal that `zheermao22` met did not occur.
- B804 (the first-content window):
  recorded in the artifact (`streamFirstContentMs: 180000`,
  `streamIdleMs: 120000`);
  no stream reached it
  (the longest first content in the run was 122,849 ms).
- B808 (NeuralWatt estimate):
  not exercised (no NeuralWatt call).
- B809 (the Mercury window):
  not exercised (see "The Mercury recheck").
- B810:
  nothing to apply;
  the `SEAT` tally still does not count an abandoned call as asked (Qwen `asked=104 ... abandoned=6`).
- Ledger B719 and B718 (reasoning effort on a NeuralWatt picture call):
  not exercised.
- The Gemma readers lift:
  fired (see "Stage outcomes");
  the `JUDGE SEATS` field `seeing=` reads 5 with six readers in preparation and 4 after Synthetic went dry.
- The KaTeX shim:
  not exercisable on this entry;
  no line of the log mentions `katex`.
- B713 (the writers' window):
  the launch line prints it (`WRITER GRACE built in`),
  and the letter-email block review waited 180 s (line 385).
- B714 (kept findings):
  fired (`alignmentFindings` 63).
- B715 (anchor and logged drops):
  fired;
  the nine `archive review dropped` warns include the first drop of a source-supported reply (line 387).
- B716 (the composite carries its contributors' assessments):
  not exercised (no composite built).

### Recurrence of the known classes

The code was not re-traced.
The classes are those of the `zheermao22` read and the `XingZ6017` read,
and each status is what the log and the artifact show.

- K1 (NeuralWatt text calls ending at the cap with no content):
  not exercisable,
  because NeuralWatt took no call.
  The same shape recurred on Hyper,
  cheaper:
  four Hyper calls ran to the completion cap with `0 content chars`
  (`glm-5.3-flash` three times at 18,316 tokens,
  `minimax-m3` once at 10,822;
  stream lines 372,
  493,
  527 and 517;
  `finishReason: max_tokens` in four payloads),
  65,770 completion tokens in all.
  All four fell in the time of the archive block reviews (12:36 to 12:46 UTC;
  the stream lines carry no stage tag),
  and each was re-asked on OpenRouter within 5 to 8 s
  and answered usably (`reaskElsewhereNudged`,
  lines 374 and 384,
  495 and 505,
  519 and 532,
  529 and 535).
  No recovery round waited out a grace for them,
  where `zheermao22` lost 180 s and 175 s to two such rounds.
  There is no energy cost on Hyper.
  4 of the 892 stored replies ended on a token limit
  (`zheermao22`:
  14 of 961).
- K2 (a stream that sends bytes and no content,
  cut only by the grace):
  absent for Mercury (0 of 121 streams).
  The grace cut instead six Qwen streams that reasoned without answering
  (see "Seats,
  units and meters"),
  the N2 shape of `XingZ6017`.
  Qwen was withheld from the lane contest and consolidation once Synthetic went dry,
  and from then on no Qwen stream ran on Synthetic;
  the 11 Qwen calls on Hyper in the translate lane were all completed.
- K3 (a repair-lane fix for a minor accepted claim lost when the lane contest settles `neither`):
  the one verified instance of `zheermao22` (slice 8) did not recur;
  the claim was carried (see B731).
  Whether the other accepted claims were carried on the lane-won path was not checked,
  and the contest settled `neither` on slices 11 and 14.
- K4 (the composite refuses a revision that changes the line count):
  did not occur;
  the three refusals are for overlap.
- N1 (a hung stream holds a quorum that equals the live bench):
  two contest rounds closed `4/6` with no grace after 62.4 s and 24.6 s.
  Neither is shown to be a hung stream;
  the log does not show which voice was slow,
  so this is a candidate and not a recurrence.
- U1 (the Hyper key order):
  cannot be tested from this run (see "Whether the new rulings fired").
- U2 (the slice boundaries moved):
  the boundaries match `zheermao22`;
  what moved them between `zheermao21` and `zheermao22` is still untraced.
- Two Hyper `gemma-4-26b-a4b-it` contest replies that ran to 3,831 tokens in `zheermao22`:
  no reply of this run ended on `max_tokens` for that model.
- `select-self-vote (glm-5.3)`:
  10 findings against 10;
  the vote of a writer for its own candidate is weighted at 0.5 by design.

Defects and regressions against `zheermao22`,
in the order this read ranks them:

- The plan energy figure,
  326.9 Wh with no NeuralWatt request in the log,
  six times the largest figure of the sibling reads.
  Not a code defect:
  the owner's dsh harness shares the NeuralWatt account
  (owner,
  2026-10-22).
- The OpenRouter balance,
  3.99 USD at the end.
- Qwen runaway reasoning,
  six cuts,
  780 s of grace;
  one writer round waited 180 s on it.
- Repair slice 0,
  846 s,
  the slowest repair slice of the two runs.
- `repairIssues` 42 against 31,
  with 18 rejected against 7.
- 29 irregular panel ballots from `gemma-4-26b-a4b-it`.
- The Hyper `glm-5.3-flash` stream that sent its first content after 122,849 ms.

Against these:
the Mercury stall did not recur,
the letter's archive text stood instead of a partial revision,
the lost-claim case on slice 8 did not recur,
no quorum failed,
no recovery round waited out a grace,
and the run was 29.6 min shorter.

### Traces and side checks

Four files of the worktree were read as files,
and nothing was run in it.
The branch ref file reads the `START` tip,
and the working tree was not compared with it.

- `package/translation-repair/shared-contract/src/model-cards.ts:431`:
  the Mercury card names `firstContentMs: 90_000` (ledger B809).
- `package/translation-repair/provider/src/openrouter-client.ts:461` to `462`:
  the request carries the card's own first-content window where the card names one.
- `package/translation-repair/provider-stream/src/stream-idle-guard.ts:89` and `:106`:
  `STREAM_FIRST_CONTENT_MS = 180_000` and `STREAM_IDLE_MS = 120_000`,
  which are the artifact's recorded `callConfig` values.
- `.git/refs/heads/translation-repair-rebased`:
  `8864316eb29b9848d95ceeaf4c039671c0c430e8`.

### Open items and what the read did not do

Open items:

- The 326.9 Wh of plan energy is settled as the owner's dsh harness on the same NeuralWatt account
  (owner,
  2026-10-22).
- Whether the 90000 ms Mercury window works:
  no run has had a Mercury stall since it landed.
- Why the cached reference text holds 25 characters.
- Why `gemma-4-26b-a4b-it` returns so many irregular panel ballots,
  and whether the count rose against `zheermao22`.
- The `SEAT` asks against the stream and `SPEND` lines
  (unreconciled by seat).
- The U2 cause (the slice boundaries).

What the read did not do:

- It did not read the shipped page line by line,
  so it has no departure list and no class count by departure.
- It did not read any prompt
  (the run directory holds replies,
  not prompts),
  so what the polish and contest sheets showed is known only from the ballot texts that quote them.
- It did not trace code for the classes.
- It did not run a spread measurement,
  and compared with `zheermao22` only by the figures of its read.
- It did not check the host for the cause of any slowness.
- It did not probe any provider.
