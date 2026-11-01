# OpenRouter and the corpus passes: the evidence of XingZ6021 on build bcff23b8ce (2026-10-25)

The log lines behind the refusals,
errors,
lost voices and short-bench rounds of `XingZ6021`:
every `[warn]` line accounted for,
the seat refusals by stage,
the voices lost,
the rounds that closed on too few voices,
and the pipeline findings that are not refusals.
The run's headlines,
timeline,
providers and meters are in
[the run of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md).
The seven checks of the build's fixes and the page are in
[the stages of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md).
The defect candidates and the owner's rulings are in
[the defects of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).

The first reader's scripts and the check are described in the run file.
Times are UTC;
line numbers are of the log.

## Refusals, errors and the lines the brief names

Every `[warn]` line (7,692) falls in a shape of `warns.ts` (139 distinct shapes,
none left over) and in a bucket of `warnbuckets.ts` (none in "other").
The check made its own census and found 222 shapes by its own normalization,
which is not comparable with the first reader's 139,
and no shape that the read does not name in kind.
There is no `[error]` line,
no `STALLED`,
no `INCOMPLETE`,
no `REATTEMPT` and no `TALLY ERROR`
(`rg --count` finds only line 198's `HTTP 401`,
the Bedrock note,
and four stream lines whose text `429 content chars` is a character count,
not a status).
The retry ladder of `XingZ6020` has a different line shape from this build's,
so `httpcodes.ts` finds no `RETRY` line and no `<- model: HTTP N` line here;
the one retry is counted by `rg --count "retrying in"` (1,
line 4184),
and no HTTP status came back from any provider.

### Control lines (no level)

198 of the 228 non-empty lines without a level are build output before line 199.
Of the 30 from line 199 on,
11 are continuation lines of multi-line judge reasoning (lines 11601 to 13396),
and the rest are `OVERLAP` (200),
`TALLY` (21929),
`DESTINATIONS` (21930),
`DONE` (21931),
`NeuralWatt plan energy` (21933),
13 `SEAT` lines (21934 to 21946) and `Finished in` (21947).

### Seat refusals

The line "no provider can take X:
every provider serving this model is out of budget,
seat unreachable" appears in
5,773 lines,
of which 5,770 match the seat-refusal shape and sum,
per model,
to exactly the `SEAT` unreachable counts
(853,
1,054,
974,
423,
973,
1,
812 and 680 are 5,770;
the check's `c9.ts`).
The other three are `reaskElsewhereNudged` warns at lines 912,
1235 (Haiku:
"no provider other than anthropic serves it with budget") and 1700
(Kimi:
"no provider other than synthetic serves it with budget"),
which reuse the phrase for a re-ask that had nowhere to go.
They are not seat refusals,
and they explain the gap of three that the first reader left untraced.

- The six seats that only Hyper,
  OpenRouter or Bedrock serve make 4,278 of the 5,770 unreachable counts
  (`minimax-m3` 1,054,
  `google.gemma-4-e2b` 973,
  `gemma-4-26b-a4b-it` 974,
  `inception/mercury-2.5` 853,
  `glm-5.3` 423,
  `google.gemma-4-31b` 1);
  Kimi 812 and Qwen 680 follow from line 5436 on.
- By stage (`seatrefusals.ts`):
  select 2,024;
  panel 1,186;
  translate 720;
  produceConsolidations 627;
  critic 490;
  absolute-naturalness-review 211;
  coverage 150;
  block-pairing 120;
  refiner 80;
  probe 42;
  consolidate-gate 35;
  polish-gate 30;
  checker 14;
  editor 12;
  section-pairing 10;
  page-consistency 6;
  title-lexicon 5;
  reference-attest 5.
- Each refusal is a seat that a dry provider could not serve at the moment the stage asked;
  none was a provider's refusal of a prompt.
  Refusal replies (zero content with a short body):
  none (`models.ts`).
  Zero-content completed streams that are not refusals:
  14 on NeuralWatt `deepseek-v4.1-flash` and 1 on DeepSeek's own API
  (their bodies were not read).

### Voice lost

- 34 Qwen streams abandoned after quorum (`cut-mid-reply`):
  select 10,
  critic 7,
  panel 7,
  produceConsolidations 4,
  introduced-defect-probe 2,
  coverage 2,
  checker 1,
  page-title-lexicon 1 (`warns.ts`).
- 7 further voices lost:
  2 `deepseek-v4.1-flash` block-pairing `schema-mismatch` (finish_reason=length,
  lines 472 and 725;
  the re-ask failed too),
  2 `claude-haiku-5-5` coverage `schema-mismatch` (lines 913 and 1236),
  1 Kimi critic `schema-mismatch` (finish_reason=length,
  line 1701),
  1 `deepseek-v4.1-flash` critic ended at its content bound (line 6607)
  and 1 Kimi transport failure that a retry then recovered (line 4184).
- 134 `went dark` lines are one per model per refresh when a model's providers all read dry (`refuseDarkModel`);
  they are state notices,
  not failures.

### Rounds that heard too few voices

Counted by `noquorum.ts` and `warns.ts` (first reader),
with the panel rows corrected by the check.

- Rounds that closed on a short bench and then did not reach their floor ("no quorum (1 of 2 needed)"):
  select 90 (summed 24.1 min,
  largest 95 s,
  lines 17400 to 21779),
  absolute-naturalness-review 31 (9.0 min),
  introduced-defect probe 21 (4.7 min),
  checker 7 (1.3 min),
  consolidate-gate 5,
  polish-gate 5,
  critic 1,
  page-consistency 1.
- Rounds that closed short but reached their floor of two:
  panel on 2 of 8,
  165 rounds (lines 5798 to 10650),
  on 3 of 8,
  7 rounds and on 4 of 8,
  42 rounds
  (41 before the dry reading and 1 after it,
  line 5493);
  214 `panel: bench short of quorum` lines in all (the check's `c10.ts`).
  The first reader's "on 4 of 8,
  49 rounds" included the 7 rounds that heard 3.
  Select on 2 of 8,
  212 rounds;
  translate on 2 of 8,
  120 rounds;
  produceConsolidations on 2 of 9,
  86 rounds,
  on 3 of 9,
  4 rounds and on 4 of 9,
  1 round
  (the first reader wrote 91 on 2 of 9 in one place and 86 plus 5 in another;
  the check's split is 86,
  4 and 1);
  critic 88;
  coverage 30;
  plus the one-off rounds of preparation.
- 71 lane-contest slices and 10 slate slices were closed without a round on a bench under its floor.

### Pipeline findings that are not refusals

- 66 + 66 `standing lacks contest endorsement` lines (two shapes per slice) and 35 `final-selection-unendorsed` lines
  (21893 to 21927):
  the 66 slices whose standing text no judge endorsed (`standingUnendorsed=66` in the `TALLY`).
- 50 publication-rule lines:
  candidates that failed the deterministic rule were returned to their author or withheld
  (for example untranslated `TA`,
  a dropped footnote,
  a block count that differs from the source);
  40 page-assembly lines
  (36 slices rewritten,
  one summary line:
  20 `quote-style-unified`,
  8 `heading-series-unified` renamings to `Part N:`,
  7 `canadian-form-rewritten`,
  2 `handle-gloss-placed`,
  1 `title-reference-unified`,
  1 `blockquote-quotes-unwrapped`).
- 43 refine rollbacks and stage-quorum-unmet lines;
  14 `introduced repetition` lines (17084 to 17097);
  12 `straightenProseQuotes` refusals (10 plus 2);
  5 `challenge round declined` (4 `indecision`,
  first lines 17830 and 18793,
  and 1 `rejection`,
  line 20274);
  5 alignment lines;
  4 stub markers stripped;
  3 `translate-archive-disputed` (12402 to 12404);
  1 `FOOTNOTES` relabel (571);
  1 irregular Haiku panel ballot (4435);
  1 `repair-stripped-worse-voted` (5725);
  5 `ineligible-standing-withheld`,
  each with a matching `consolidate gate: settled on neither over an ineligible standing, so the proposal the slate
  chose ships (1 usable ballot)` line (the first at 17654 and 17856),
  which is the one-ballot consolidation path of N4.
