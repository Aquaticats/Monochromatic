# OpenRouter and the corpus passes: the stages of XingZ6021 on build bcff23b8ce (2026-10-25)

What the read of `XingZ6021` found about the seven fixes that build `bcff23b8ce` carries,
about the page that shipped,
and about what the run could not test.
Entry:
`XingZ6021` (entry `XingZ60`).
The log,
the providers,
the meters and the timeline are in
[the run of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md).
The Shinonome defect in full,
the other nine defect candidates,
the owner's rulings and the second reader's corrections are in
[the defects of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).

A first reader wrote the read and a second reader checked it,
as the run file describes.
Where a section says "first reader only",
the check did not recompute the figure or the quotation.
Times are UTC;
line numbers are of the log unless a line is called a page line,
a source line or an archive line.

## The seven checks of the build's fixes

Each check is stated as exercised or not,
and whether it behaved as intended.
The check's corrections are applied.

### 1. Source text under a note (`pageSourceShipped`)

Not exercised.
The fix is the owner's ruling of 2026-10-23:
a slice with no archive and no backed wording ships its source text under
"This passage has not been translated yet.
The original text follows.",
footnote definitions included.

- `pageSourceShipped=0` (line 21929),
  and `rg --count` for the note text finds nothing in the log,
  the artifact or the shipped page.
  All 118 slices carry wording.
  The 30 tail slices (88 to 117) were written by the translate lane,
  and each has `delivery.kind = replacement-shipped` over `incumbentKind = absent` in the artifact (the check's `c8.ts`).
  So the note's rendering,
  its plain-text behaviour and whether it swallows text could not be checked from this run.
  The code is `page-check/src/corpus-run/source-ships.ts:43` (`UNTRANSLATED_PASSAGE_NOTE`).
- The path was a near miss and not a distant one.
  The check found that seven tail slices logged a failed select round before the round that backed them,
  either `winner was named by 1 ballot against a floor of 2; keeping the fallback` or `no-candidate-backed`:
  slices 89,
  95,
  101,
  103,
  106,
  114 and 115 (slices 95 and 103 matched three lines each).
  Slice 95 shows the shape (lines 16105 to 16111,
  16219 to 16224 and 16389 to 16394):
  Haiku named one candidate and DeepSeek the other in the first two rounds,
  each time one ballot short of a winner,
  and the third round named a winner at weight 1.5 with two ballots.
  A different draw there would have ended `no-candidate-backed` and put the note and the source text on the page.
  All thirty ended backed,
  at weight 1.5 on 23 and at weight 2 on 7.
- The footnote rule that the brief ties to this (D7 of `XingZ6020`):
  the source has 9 references and 9 definitions
  (source lines 94,
  156,
  166,
  339 three times,
  389,
  415,
  435 and 561 to 577);
  the shipped page has 9 references (page lines 128,
  200,
  212,
  387 three times,
  437,
  463 and 483) and 9 definitions (page lines 728 to 744);
  none is missing or unused (`footnotes.ts`).
  The HTML tags balance (`tags.ts`) and both links survive (`DESTINATIONS`,
  line 21930).
  The check did not recount the footnotes or the tags.
  The build also has a publish check for footnotes (`page-check/src/corpus-run/publish-fixed.ts:342`,
  `page-footnotes`);
  it did not fire (no `publish: shipping with defect` line).

### 2. D1, the wait that cannot help

Partly exercised;
behaved as intended on the branch that ran.

- No hold ever ran (no refusal replied,
  `markRefused` appears zero times),
  so the wait that `XingZ6020` made 228 times never arose;
  every one of the 161 `JUDGE SEATS` info lines reads `waited=0ms`.
- The new wording printed on six `JUDGE SEATS` warn lines:
  `late reads under its floor of 2 and no hold runs that could lift it`
  (lines 17099,
  17102,
  17279,
  17282 and 21838;
  emitted at `run/src/corpus-run/run-seats-ceiling.ts:219`)
  and `no provider has named a return the pass can wait for, so the phase runs on what is reachable` (line 12399).
- Not exercised:
  a short bench with a hold running whose end would not lift it,
  which is the case the fix was written for.

### 3. D2, tail slices tried twice to no gain

Not an issue in this run.

- Each lane started each slice once:
  118 `SLICE-START lane=repair`,
  118 `lane=translate`,
  101 `lane=consolidation` and the same number of closes;
  `attempts.json` reads 1.
  The tail was filled in one pass:
  the translate lane closed 118 of 118 `computed`,
  and the repair lane closed the 30 tail slices as `no-translation` (lines 10404 to 10491),
  which is its job.
  `translateStatus=complete`,
  `pageSilent=0`.
- The select floor (two ballots) was met on all 30 tail slices by two ballots each.
  How little independent support that is appears under N6 in the defects file.

### 4. D3, stages that ran in full on one voice

Exercised;
fixed for the lane contest,
only partly fixed for the consolidation.
The figures are the check's where they differ from the first reader's.

- Lane contest:
  71 slices closed with `1 of 7 seats reachable, below the floor of 2; no round is opened and the slice keeps its incumbent`
  (lines 17107 to 17253;
  `lane-contest/src/lane-contest-stage.ts:446`)
  and 30 closed `sole-lane`;
  0 rounds and 0.06 s of wall clock,
  against 73 min and 63 rounds in `XingZ6020`.
- Consolidation slate skip:
  fired for 10 slices only (88 to 97,
  lines 20164 to 20393,
  `slate judges reachable, below the floor of 2; no slate is bought and the standing is kept`).
  It cannot fire when lane texts exist,
  by its own condition (`consolidate/src/consolidate-slice-buy.ts:254` to `:257`,
  `laneTexts.length === 0`),
  so for the other slices the stage still ran:
  90 slate `select` rounds heard 1 of 7 (no quorum,
  24.1 min summed),
  91 `produceConsolidations` rounds in all (86 on 2 of 9,
  4 on 3 of 9,
  1 on 4 of 9;
  34 min summed),
  31 absolute-naturalness-review rounds heard 1 of 7 (9.0 min),
  5 plus 5 gate rounds and 1 page-consistency round.
- The consolidation of 101 slices took 21.1 min and 383 NeuralWatt calls (69.6 Wh) and changed 6 slices
  (13,
  14,
  33,
  44,
  84 and 101).
  The first reader wrote that each went through a single gate ballot;
  the check corrected that:
  slices 13,
  14,
  33,
  44 and 84 have one usable ballot in the artifact's `consolidation.slices[].gate`,
  and slice 101 has two (`usable=2`,
  gate round `2/7` at line 20793,
  inside the 2 percent window of Synthetic).
- Terminals:
  `slate-declined-standing` 82 unchanged and 1 incumbent (slice 2),
  `consolidated` 6,
  `incumbent-only` 10,
  `slate-unjudged-standing` 2 (slices 106 and 111).
- The floor-skipping was effective where the code reaches;
  it left about 21 min and 70 Wh of consolidation on benches that could not decide.
  The ledger has the fix of this build as B835
  (`doc/audit-ledger-short-benches-and-the-last-live-seat.md:362`,
  scope statement at `:390` to `:402`),
  which skips only slices with an eligible standing and no lane text;
  this run is the first measurement after it.

### 5. D4, a rollback with no log line

Not recurring:
every rollback logged.

- 18 repair-lane refinements won
  (`refinement from X won weight`,
  lines 10873 to 12378;
  slices 7,
  17,
  20,
  23,
  26,
  27,
  28,
  32,
  43,
  45,
  52,
  53,
  57,
  60,
  62,
  77,
  81 and 82),
  and each has a warn:
  15 `refine-probe-unheard (N of 3 probers heard)` (`repair/src/refine-probe-verdict.ts:148`),
  2 `refine-recheck-unheard` (slices 7 and 17,
  lines 10915 and 11083,
  `repair/src/refine-recheck.ts:226`)
  and 1 `refine-rolled-back-by-probe` (slice 57,
  line 12207).
  The check's own census of warn shapes agrees (15,
  2 and 1).
- All 118 chunks end `refined=false`:
  no refinement shipped,
  because the probe and the checkers heard one voice of three once Synthetic was dry (N5 in the defects file).
- Five more `refinement from X won weight` lines belong to consolidation polish
  (slices 33,
  100,
  103,
  109 and 111,
  lines 18923,
  20856,
  20911,
  21277 and 21432);
  what became of them was not traced.

### 6. E2, the NeuralWatt hold and the NeuralWatt day

Not exercised.

- NeuralWatt read `wet` on 102 of 102 readings,
  no hold named it,
  there is no `markRefused neuralwatt`,
  and the lowest `neuralwattWh` was 267.881 (line 21775).
- By our own meter (the sum of the 1,436 `NeuralWatt energy` lines) the run drew 283.068 Wh of 550 Wh (51.5 percent);
  the account meter read 280.882 at its last reading.
  The day began 19:15:08 UTC on 2026-10-24 (line 204),
  3 h 34 min before the run's first line,
  so the whole run lay in one NeuralWatt day.
  No hold was set and none was needed.
- The step of `neuralwattReckonedWh` to 0.324 Wh is N7 in the defects file.

### 7. S1, whether the build read Synthetic's credit throughout

Yes.

- All 102 `METERS` lines carry `syntheticWeekly=<percent>`;
  there is no `budget unreadable`,
  `treating as spendable` or `violated expectations` line.
  The build read `0%` as dry (line 5431,
  00:01:55.805 UTC),
  named the return (`syntheticWeeklyReturn=2026-10-25T00:44:35.000Z`),
  saw `2%` and `wet` at 00:44:43 UTC (line 20414) and spent it,
  and read `0%` and `dry` again at 00:50:18 UTC (line 20781).
- Every one of the 404 Synthetic streams started under a `wet` reading
  (`drycalls.ts`:
  370 completed with content,
  33 cut with no content,
  1 cut with content),
  and the log has no 429 and no refusal reply.
  The details are in the Synthetic section of the run file.

## What the run could not test

- The source-text note.
  `pageSourceShipped=0`:
  no slice reached the path that puts source text on the page under the note,
  so the note's wording,
  its rendering as plain text,
  the footnote definitions it carries and whether it swallows page content are untested on the shipped build.
  Seven tail slices came within one ballot of that path (check 1).
- The NeuralWatt hold.
  NeuralWatt was never dry in this run (`neuralwatt=wet` on 102 of 102 readings),
  so a hold that names NeuralWatt,
  and the wait or the stop that follows it,
  was never reached.
  The day anchor lay 18 h 20 min past the end of the run.
- A short bench with a hold running whose end would not lift it (D1's case):
  no hold ran.
- A run in which Synthetic stays wet throughout.
  The credit ran dry 72.4 min in,
  so no arm of this run had the voices of Kimi and Qwen on every stage,
  and the effect of the faster Claude bench on the low rate of repair edits (14 of 118)
  cannot be separated from the effect of the dry credit.
- The personal sections of the page (tense,
  pronouns,
  names) beyond the Shinonome section,
  the HTML tag balance and the footnote counts were not examined by the check.
  The quotations of the page in the first reader's lists below were not compared with the source by the check.

## The frozen build and footnote lines in quotes and list items

The re-check of the follow-ups to D and X1 (its finding R2) reported that the frozen build `bcff23b8ce`,
which this run used,
carries the live-expression form for footnote lines in quotes and list items.
The filer received that finding in the brief for this record;
the re-check's own text was not among the filer's inputs,
so the form and its trigger are not described here beyond that sentence.

Whether any slice of this run took those shapes was measured on the corpus copies of the run
(`rg` over `source.page.md`,
`archive.page.en.md` and `shipped.page.en.md`):

- Footnote definitions behind a quote or list marker
  (pattern `^\s*(>|[-*+]|[0-9]+\.)\s*\[\^[0-9]+\]:`):
  none in any of the three copies.
  Footnote definitions at the start of a line (pattern `^\[\^[0-9]+\]:`):
  9 in the source and 9 in the shipped page.
- Footnote references inside a quote or list line
  (pattern `^\s*(>|[-*+]|[0-9]+\.)\s.*\[\^`,
  over the source and the shipped page):
  one line in each,
  none in a list item.
  Source line 166 is a `>` line:
  the remark about the song "Trans 什么之光" with the reference `[^3]` (read with the match width limit lifted).
  The shipped page line 212 is `> It’s called something like Trans something or other Lights[^3].`,
  which holds no brace expression.
- So no definition line of this run was in a quote or a list item,
  and one reference sat in a quote.
  Whether that one reference in a quote meets the form that the re-check found
  was not checked,
  because the form is not described to the filer.
  The path that would put footnote definitions under the source-text note was not reached at all (check 1).

## What the page says

Method (first reader):
the shipped page (`fixed/people/XingZ60/page.en.md`,
745 lines) was read once from top to bottom against the source (`source.page.md`,
577 lines) and the archive (`archive.page.en.md`,
358 lines),
with the artifact's per-slice `comparison` table (`shippedlane2.ts`) and the run's own critic lines beside it.
Not every sentence of the long personal sections (Ann,
LingYu) was compared word by word.
Page line numbers are lines of the shipped page.
The check examined the Shinonome section in full (defects file);
the rest of this section is the first reader's reading,
and the check did not compare these quotations with the source.

### What the page holds

- 118 slices,
  48,105 characters written (`publish`,
  line 21928);
  the file is 48,712 bytes.
  Against `XingZ6020`,
  the thirty tail slices (88 to 117,
  about page lines 411 to 726,
  ending at the credit line) are now present and read as faithful renderings in the passages compared;
  the footnote definitions (page lines 728 to 744) are present.
- Footnotes:
  9 references and 9 definitions
  (the archive page had 5 references and no definitions).
- Canadian spelling:
  `behaviour`,
  `centre`,
  `grey`,
  `honour`,
  `fulfil` and `-ize` forms appear;
  the only `center` hits are HTML `align = "center"` attributes (page lines 66,
  68,
  385,
  411,
  431,
  487 and 498).
  Page assembly rewrote `behavior` to `behaviour`,
  `center` to `centre` and `gray` to `grey` where they were prose (lines 21864 to 21870).
- The only Han text left on the page is a translator's HTML comment at page line 52,
  carried from the archive;
  the source's own commented-out markup is not kept.
  The check classed this as by design (N9 in the defects file).

### Which lane's words the page carries

The artifact's `comparison` table has the lane relation for every slice (`laneRelation`):
`translate-only` 90,
`archive-stands` 14,
`both-differ` 7,
`both-agree` 3,
`repair-only` 4.
The folded-text match of each slice against the shipped page (`shippedlane2.ts`,
which lower-cases and drops every non-letter so that quote-style and Canadian-form edits do not hide a match) gives:

- 51 slices where the translate lane wrote text of its own ship the repair or archive wording instead
  (slices 16 to 19,
  24,
  25,
  27,
  29 to 32,
  35 to 40,
  42,
  43,
  45,
  46,
  48 to 50,
  52 to 55,
  57 to 60,
  63 to 65,
  67 to 72,
  74,
  75,
  77 to 79,
  81 to 83,
  85 and 87).
- 31 slices ship the translate lane's wording:
  27 of the 30 tail slices (88 to 98,
  100,
  102 to 105 and 107 to 117) and slices 20,
  21,
  33 and 44.
- 6 slices (15,
  23,
  62,
  99,
  101 and 106) match neither,
  so page assembly or consolidation rewrote them (3 of these are tail slices);
  the remaining 30 slices are `archive-stands`,
  `repair-only`,
  `both-differ` or `both-agree`.
- This agrees with the endorsement chain of N1 and D3:
  the slate had one judge,
  so the translate lane's text could not win (`slate-declined-standing` 82 times;
  `standingUnendorsed=66`);
  the 51 slices were not each matched to a terminal.
- Backing of the translate lane's winners (`decideBestCandidate ... won weight`):
  23 of the 30 tail winners and 42 of 66 head winners are DeepSeek candidates at weight 1.5,
  which is one independent Haiku ballot of 1.0 plus DeepSeek's own ballot at the half weight
  (`house-rule/src/candidate-select-model.ts:55`,
  `SELF_VOTE_WEIGHT = 1 / 2`);
  7 tail and 24 head winners are GLM candidates at weight 2 from two full ballots.
  The check counted the tail winners itself (1.5 on 23 and 2 on 7) and confirmed the count.
  The select rule is met in every case;
  the observation is only how little independent support a "backed" wording has on a bench of two (N6).

### Mistranslations and omissions that the run's own critics flagged

Each was filed as a claim in the repair lane and adjudicated `needs-human`,
because the panel could not decide,
so the text stayed.

- The Shinonome section (page lines 293 to 306,
  slices 67 to 70) is the defect N1;
  it is told in full in the defects file.
- Page line 188:
  `I was free that day, otherwise I would have gone straight to the hospital.`
  (source line 144 `我那天没空`,
  "I was busy that day");
  flagged at lines 7326 and 7330,
  `needs-human` at 7602.
- Page line 276:
  `selfish desires that I am not ashamed of`
  (source line 220 `令我自己感到不耻的私心`,
  desires I am ashamed of:
  the meaning is reversed);
  and `In your last words, write what you want to say ...` (a past statement turned into an instruction);
  flagged at 8666 to 8672,
  `needs-human` at 8974 and 8975.
- Page line 208:
  `very addictive` for `很坐牢` (source line 164:
  "feels like a prison sentence",
  tedious).
- Page lines 360 to 368:
  `very mature` for `亚撒西` (source line 314;
  "Yasashii" is used later),
  `teasing me` for `打 mai` (source line 320),
  `flip over` for `翻车` (source lines 21,
  23 and 318;
  the meaning is a crash or a relapse),
  and `spiritual exploration` for `心别探索` (source line 318;
  the source has a typo for `性别`,
  gender);
  flagged at lines 10019 to 10041.
- Page line 387:
  `And they hated their future more miserably`
  (source:
  pitied their future,
  hated their present),
  a space before `[^4]` and `[^5]`,
  and `Sabathani` in the text against `sabachthani` in footnote 4.
- Flagged by the critics and left:
  page line 324 `my departure from XingZ60 happened again` (slice 73,
  a major `accuracy/mistranslation` by Haiku,
  line 9356),
  page line 336 `you both won` for `都` (slice 74,
  a minor one by DeepSeek,
  line 9382),
  and page line 340 `Even the brain can understand it` (slice 75,
  two major ones,
  lines 9571 and 9572:
  the source says the person presumably understands).
- Found on reading,
  with no claim looked up:
  page line 150 `causing her to become addicted` for a friend who is `he`;
  page line 128 `always in chains`;
  page line 360 `Just 72 days left an indelible mark on your life`.
- Page line 228:
  `> I will always be with you, sir.` for the Japanese lyric `煌めきを纏ったら どこまでも君と。` (source line 172):
  "sir" is unsupported;
  page assembly noted `blockquote-quotes-unwrapped` (line 21891).
- Page line 52:
  `Trans aid work had always been a part in her daily life.` is a sentence the source does not have
  (it follows the archive).

### Tense and pronouns

- Present tense for a life that has ended,
  in prose
  (Canadian English asks for the past):
  page lines 60,
  77,
  96,
  104,
  108,
  196,
  206,
  208,
  351 (`he is a rational person`),
  362,
  364 and 368 (twice);
  and address to the dead in the present at page lines 256 and 258 (`you always provide`,
  `you always say`).
- Pronouns for a neutral `TA` mix she,
  he,
  him,
  her,
  their and `him/her` within paragraphs:
  the LingYu section (page lines 108 to 116,
  136,
  140,
  184,
  186,
  198,
  208 and 226) and the wing (page lines 349 to 353);
  the blockquote at page lines 182 to 186 mixes `her` and `his`.
  Page line 208 reads `he would always force herself to stop ... between me and him/her`.

### Names and markup

- Names:
  `HiYku` (Part Three) for `绘都`;
  `Ann` (Part Four) against the credit line `Wuchang (Impermanence)` at page line 726;
  `Blank` and `__ /「」` (page line 387) against `Kongbai (Blank) / Yinhao (Quotation Mark)` (page line 554);
  `Fake Haku` (page line 326) against `KouHaku` (page line 336).
- Markup:
  `<em>Paranoia</em>` (page line 518) beside `*Paranoia*` (page line 494);
  the title `*Maybe All Transgender People Are Really Vampires*` inside an HTML `<p>` at page line 502,
  where the asterisks print literally;
  `《...》` kept at page line 387.
- Headings:
  page assembly renamed `### Irako` to `### Part One: Irako` and seven others (lines 21855 to 21862),
  and the heading `Engagement in Trans Aid` adds `Trans`,
  which the source (`参与救助`) does not have.
- Page consistency ran on one voice (`1/7 heard`,
  line 21850) and wrote nothing,
  so these name and pronoun differences were not unified by that stage (N10 in the defects file).
