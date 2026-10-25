# OpenRouter and the corpus passes: the stages of XingZ6020 on build 1d46232457 (2026-10-22 to 2026-10-23)

What the read of `XingZ6020` found about the two attempts stage by stage,
about the page that shipped,
and about what the code does with a slice that ends unfilled.
Entry:
`XingZ6020` (entry `XingZ60`).
The log,
the providers,
the meters and the cost of the Synthetic hold are in
[the run of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md).
The defects with their fixes are in
[the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md);
the refusals,
errors and the points no log settles are in
[the evidence of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-evidence.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).

A first reader wrote the read and a second reader checked it,
as the run file describes.
Where a section says "first reader only",
the check did not recompute the figure or the quotation.
Times are UTC and every clock time carries its zone;
line numbers are of the log unless a line is called a page line,
a source line or an archive line.

## Timeline of the two attempts

"Idle" means no model stream was in flight (each stream taken as the interval from its line time minus its elapsed
time to its line time,
meter reads excluded),
counting gaps of 30 s or more;
it was 0.0 min before 00:28 UTC on 2026-10-23.

### Attempt 1 (19:32:42 UTC on 2026-10-22 to 02:32:42 UTC on 2026-10-23)

- 19:32:42 UTC to 19:53:20 UTC (20.6 min),
  preparation (lines 199 to 1319):
  front matter from the archive;
  `JUDGE SEATS phase=preparation short of quorum: wide 3 of 7 reachable, quorum 4; no provider has named its return`
  at the first reading (line 212),
  so the run started with a roster that could not reach the judges' quorum;
  page-title lexicon round 129 s with 120 s in grace on Qwen (line 236);
  block pairing in 24 rounds (19:35 UTC to 19:44 UTC);
  one picture read by `hf:zai-org/GLM-5.3-Flash` (lines 527 to 534);
  coverage in 31 rounds (19:45 UTC to 19:53 UTC),
  8 of them with 100 s or more in grace.
- 19:53:20 UTC (line 1319):
  `both lanes over 118 slices, repair first`.
- 19:53:20 UTC to 23:20:57 UTC (207.6 min),
  the repair lane's chunk pass:
  118 `SLICE-START lane=repair`,
  118 closes,
  the first at 19:57:51 UTC (line 1580),
  the last at 23:20:57 UTC (line 16050).
  88 slices `computed` (median about 530 s,
  slowest slice 85 at 1,517,824 ms,
  line 16050) and 30 `no-translation`
  (the tail,
  which the repair lane cannot repair because the archive has no text there).
  `XingZ6018` did the same chunk pass in 50.6 min with Hyper in the roster,
  so this pass took four times as long.
  The largest measured part is the wait after quorum,
  which the check corrected.
  The first reader's `slicegrace.ts` matched only lines shaped `[repair slice N] <stage> round:`,
  so it missed the 229 panel rounds (`[runPanelStage] [reviewPacket] [cluster/...] panel round:`,
  191.5 min of grace).
  The first reader found 455 of 793 summed slice minutes (57 percent) in grace;
  the check found 647.6 of 792.8 min (81.7 percent),
  with a median of 409 s of grace per slice against a median slice of 530 s
  (select 229.4 + critic 114.8 + probe 77.1 + checker 34.1 + panel 191.5 + two recovery rounds 0.7;
  the union of grace intervals within a slice equals the sum,
  so nothing is counted twice).
  The straggler was Qwen on Synthetic:
  the only voices abandoned after quorum were 131 Qwen calls (125 at the 120 s ordinary grace,
  6 at the 180 s writer grace) and one DeepSeek call (`voicelost.ts`;
  the check agrees on 132 lines,
  Qwen 131 and DeepSeek 1,
  with grace of 120,000 ms 126 times and 180,000 ms 6 times).
  This is D5 in the defects file.
- 23:20:57 UTC to 00:11:11 UTC (50.2 min):
  refine pass (48 slices with a winning rewrite,
  3 rolled back by the damage probe,
  3 rolled back by the recheck with no log line;
  see D4),
  checker rounds,
  the introduced-defect probe;
  `repair shipped: 80/118 slices changed, 389 issues` (line 19094,
  00:11:11 UTC).
- 00:11:15 UTC (line 19203) to 02:32:42 UTC,
  the translate lane:
  106 `SLICE-START lane=translate`,
  four slices in flight at a time (the pass overlap of four).
  - 00:11:15 UTC to 02:06:11 UTC:
    the 88 slices that have an archive text,
    all `computed`.
  - 00:28:47 UTC (line 19925):
    the first Synthetic `/quotas` reply of 269 characters (it had been 285 or 286).
    00:28:48 UTC (lines 19938 and 19939):
    the first Synthetic HTTP 429.
    00:29:00 UTC (line 19970):
    `synthetic: refused us while its meter reads unreadable ... held out for 300000ms`.
    From here the translate lane re-seated under a hold 86 times in this attempt,
    each time with
    `translators 2 of 8 reachable, quorum 4; select 1 of 7 reachable, quorum 4; ...
    waiting Nms for the shortest hold to end`
    (first at line 20031,
    00:29:33 UTC;
    `holdwaits.ts`).
  - 02:06:11 UTC to 02:31:14 UTC:
    slices 88 to 105 (18 slices) opened and all closed `exit=unfilled` (lines 25913 to 28089).
    Twelve more (106 to 117) were never opened.
  - 02:32:42.8 UTC (line 28091):
    the hard deadline fell while the lane sat in an 89.7 s hold wait (line 28090).
    The check placed the deadline at 02:32:42.829 UTC,
    inside the wait that would have ended at about 02:32:44.1 UTC.
  Idle time in the window:
  76.0 of 141.5 min.

### Attempt 2 (02:32:43 UTC to 06:05:33 UTC on 2026-10-23)

- 02:32:46 UTC to 02:33:16 UTC (30 s),
  preparation replayed from cache (lines 28093 to 28561;
  coverage replayed in 18 s).
- 02:33:16 UTC to 02:38:17 UTC (5.0 min),
  `JUDGE SEATS phase=lanes short of quorum: wide 1 of 7 reachable ... waiting 299137ms for the shortest hold to end`
  (lines 28563 to 28566):
  four readings,
  one 5 min wait.
  The check added that this wait precedes a repair-lane replay that asked nobody
  (88 `resumed`,
  30 `no-translation`,
  lines 28585 to 28854).
- 02:38:18 UTC (lines 28584 to 28849):
  repair lane replayed,
  88 `resumed` and 30 `no-translation`;
  `repair shipped` again (line 28854).
- 02:38:32 UTC to 03:17:17 UTC (38.7 min),
  translate lane:
  88 `resumed` and 30 slices (88 to 117) tried again,
  all `unfilled`,
  in eight batches (seven of four slices and a last of two),
  one batch per hold cycle.
  The batches started at 02:38:32 UTC,
  02:43:47 UTC,
  02:49:05 UTC,
  02:54:24 UTC,
  02:59:42 UTC,
  03:05:02 UTC,
  03:10:22 UTC and 03:15:39 UTC.
  The read gave the range of the unfilled closes as lines 29398 to 32763;
  the check found the first at slice 91,
  line 29309,
  so the range is lines 29309 to 32763.
  Idle in the window:
  26.8 of 39.0 min.
- 03:17:17 UTC (line 32773):
  `repair repaired, translate changed 41/118 slices; neither was chosen over the other`;
  lines 32774 to 32803 record the 30 gaps.
- 03:17:18 UTC to 04:30:15 UTC (73 min),
  lane contest over 63 slices:
  63 rounds,
  every one `1/7 usable, settled on neither` (lines 32824 to 34784;
  the check found the first such line at 32912 and 32824 as the first contest-tagged line),
  52 hold waits.
  The check named two waits the read did not:
  3.6 min between 03:17:18 UTC (line 32811) and the first contest-tagged line 32824 (03:20:57 UTC),
  and 4.7 min between 04:30:16 UTC (line 34785) and 04:34:58 UTC.
  Idle 71.4 of 77.7 min (to the start of the next stage).
- 04:34:59 UTC to 06:00:25 UTC (85.4 min),
  consolidation of 63 slices (lines 34785 to 39831):
  59 hold waits,
  64.2 min idle.
- 06:00:25 UTC to 06:05:15 UTC,
  `JUDGE SEATS phase=page consistency short of quorum ... waiting 288272ms` (line 39832),
  then page consistency,
  assembly (06:05:33.4 UTC) and `publish: wrote 88 slices into a page of 34246 characters`
  (line 39939,
  06:05:33.798 UTC).
  The check explained the "characters" as code points:
  the page is 34,246 code points and 34,637 bytes.

## What the page says

Shipped page:
`fixed/people/XingZ60/page.en.md`,
538 lines,
34,637 bytes (34,246 code points),
front matter `name: XingZ60`.
Read against `people/XingZ60/page.md` (577 lines,
41,720 bytes) and the archive `page.en.md` (358 lines) at the pin `a41fc607ea`.
The first reader's read is targeted (structure,
the tail,
tense,
pronouns,
footnotes,
the first block and two slices in depth) and not a line-by-line read of all 118 slices.
The check examined the counts,
the tail,
the markers and the footnotes;
it did not examine the tense and pronoun readings or the critic claim tallies,
which are first reader only.

### What the page holds

- Front matter,
  the opening quotation,
  the `[!Tip]` notice,
  the introduction,
  "Engagement in Trans Aid",
  and the friends' memories from Part One (Irako) to the first block of Part Nine ("__",
  the source's 空白,
  Blank).
  Nine part headings,
  in the house style `Part One: Irako` to `Part Nine: __`
  (assembly unified them,
  lines 39880 to 39884;
  the check counted them at page lines 55 to 510).
- The artifact's `lanes.translate.delivery` has 118 rows
  (`replacement-shipped` 41,
  `incumbent-retained` 46,
  `replacement-withdrawn` 1,
  `gap-remains` 30 for slices 88 to 117,
  all with `incumbentKind: absent` and `shippedText: ""`);
  `comparison` has 88 rows with an archive text and 30 `gap-remains`.
- The page ends at line 538 with the first `</details>` of Part Nine,
  after the funeral notices of 2 January 2024.
- Source lines 363 to 577 are not in it:
  the "Seven Sentences" poem and its dedication (`七句破题`),
  the caged-bird poem,
  the dialogue sections,
  the "Maybe All Transgender People Are Really Vampires" poem in full,
  Part Ten (锦心) and the closing section `致曾划过夜空的流星`,
  and the nine footnote definitions (source lines 561 to 577).
  The tail starts at source line 363 (`<h3 align = "center">七句破题</h3>`,
  the start of slice 88).
  The source has ten friend sections (`### 其一` to `### 其十：锦心`) and a closing section;
  Part Ten and the closing section are not on the page.
  The tail is 5,277 of 16,733 code points (31.54 percent),
  12,337 of 41,720 bytes.
  The summed slice texts of slices 88 to 117 are 5,131 of 16,326 (31.4 percent);
  the 407 code points between the two figures are separators no slice carries,
  so 31.5 percent stands.
- Nothing on the page says it stops early.
  The check scanned the page for `Seven Sentences`,
  `Part Ten`,
  `Under Construction`,
  `unfinished`,
  `incomplete`,
  `missing`,
  `truncat`,
  `Footnote` and `meteor` and found nothing;
  `Vampires` is found once,
  at page line 516,
  as a footnote reference's neighbour inside a `<summary>`,
  the only trace of the tail's poem.
  The archive's own `Under Construction` marker is stripped by design (lines 217 and 28109),
  and the pass put no notice of its own in its place.
- Destinations:
  the source carries two (source line 35 `one-among.us/profile/GLaDOSister` and source line 435,
  the bilibili link `https://www.bilibili.com/video/BV17Q4y1s7j1` in slice 93);
  the page carries one;
  log line 39941 says `DESTINATIONS XingZ60 source=2 page=1 dropped=1`,
  and lines 39937 and 39938 name the bilibili link.
  Dropping it is a consequence of the lost tail and not a separate fault.
  The read's sentence "Three bilibili destinations" was garbled and the check rewrote it.

### Which lane's words the page carries

- `laneRelation` over the 118 slices (artifact `comparison`):
  `repair-only` 42,
  `both-agree` 20,
  `both-differ` 18,
  `archive-stands` 5,
  `translate-only` 3,
  `gap-remains` 30.
- The lane contest decided nothing:
  63 of 63 contested slices `quorum-not-met`,
  one usable ballot each (`laneSelection.slices`,
  `shippedlane.ts`).
- Consolidation (63 slices):
  the read counted 49 `slate-declined-standing` shipped unchanged,
  6 `slate-declined-standing` shipped the incumbent,
  1 `gate-kept-standing`,
  and 7 `consolidated` and shipped changed.
  The check read the artifact's shipped kinds as `unchanged` 50,
  `incumbent` 6,
  `consolidated` 7,
  which sums to the same 63.
  The standing text in a contested slice is the repair lane's,
  so the page carries the repair lane's words where the lanes differ,
  because nothing could choose otherwise.
- Example,
  slice 4 (page lines 29 to 33):
  the page reads "Then in September 2023 she went through it again and started her transition anew.
  Afterward,
  she then got a diagnosis for her identity."
  The source says only 最后再次翻车是 2023.09。后面就确诊了。
  ("The last time it went wrong again was 2023.09.
  Afterward,
  [she] was diagnosed.").
  The translate lane's text for the slice ("The last time it happened again was September 2023.
  Afterward,
  she got a diagnosis.")
  is the closer one,
  and it did not ship.
  The first reader's account:
  a refinement of the slice won weight 2
  (line 16251,
  `refinement from hf:zai-org/GLM-5.3-Flash won weight 2`),
  both judges at lines 16246 and 16247 name "started her transition anew" and "a diagnosis for her identity" as
  added content,
  and the recheck then rolled the refinement back with no log line,
  so the page kept the wording the judges had named.
  The ledger later corrected that account (B829,
  `doc/audit-ledger-page-assembly-and-corpus-run-driver.md`):
  in the artifact,
  slice 4 has six `refine-rolled-back` findings,
  `chunk.refined` is false and `chunk.changed` is true,
  `comparison.repairText` equals `chunk.repairedText`,
  and the consolidation ships that text.
  So the page carries the editors' text from before the refinement,
  which is what the rollback restores:
  the rollback took effect,
  and the sentence on the page is not the rolled-back refinement.
  The check confirmed the rollback with no log line
  (line 16390 Xiaomi `not-fixed` and line 16391 Kimi `fixed` for issue `ea4de6864f...`)
  and did not examine where the phrase came from;
  the ledger's finding postdates both readers and is the one that stands.

### Tense and pronouns

First reader only.

- The recurring issue of the brief,
  a life that has ended told in the present,
  is still on the page.
  Present-tense statements about XingZ60 stand at page lines 74 ("XingZ60 is an “abstract” person"),
  104 ("XingZ60 has this idea"),
  108 ("XingZ60 seems to have split personality.
  She said she has at least three personalities"),
  226 ("XingZ60 still hopes that I will ..."),
  236 ("XingZ60 plays "Genshin Impact""),
  465 ("he is a rational person") and 481 ("XingZ60 is actually quite indifferent to life and death").
  The critics had filed 42 tense claims in 23 slices (`quality.ts`,
  `tense.ts`:
  28 from Kimi,
  10 from DeepSeek,
  4 from Qwen;
  25 of them `fluency/inconsistency`,
  16 `fluency/grammar`,
  1 `accuracy/mistranslation`);
  29 accepted cluster lines list one.
- Pronouns:
  the source uses the neutral 星璃璃 or TA throughout the Lingyu section.
  The page uses she or her at lines 108 and 112,
  "him/her" at line 116,
  and he,
  his or him at lines 110,
  118,
  166 to 169 (another friend),
  228,
  250,
  252,
  263,
  456 to 467 and 526.
  The critics filed 64 `policy/pronoun` claims (61 major).
- Part Nine's heading is the archive's `__`,
  while its body calls the contributor "Blank" (page line 528).

### Footnotes and markup

- Five footnote references (`[^1]` at line 128,
  `[^2]` at line 230,
  `[^4]`,
  `[^5]` and `[^6]` all on line 516) have no definition on the page.
  The check counted the definitions on the page at zero.
  The source has nine references (source lines 94,
  156,
  166,
  339 three times,
  389,
  415 and 435) and nine definitions (lines 561 to 577,
  in slices 115,
  116 and 117).
  The page lacks `[^3]` (source line 166,
  slice 44) and `[^7]` to `[^9]` (slices 91 to 93).
- The check corrected the reason.
  The five dangling references are not caused only by the lost tail:
  the archive carries the same references (`page.en.md` lines 98,
  160 and 334 three times) and no definitions,
  because it stops at line 358 with the stub marker.
  The page inherited all five unresolved from the archive,
  and only filling the tail could have resolved them.
  The pass's shipping check reported only destinations (`DEFECTS XingZ60 checks=destinations`).
  This is D7 in the defects file.
- `assembly-footnote-reverted unresolved-reference gfm 3 (round 1)` (lines 32770 and 39877) is the assembly
  declining one footnote change for the same reason.

### Critic claims before the stop (attempt 1)

First reader only.

- 535 claims in 83 slices (`claims.ts`):
  filers Kimi 254,
  DeepSeek 173,
  Qwen 108.
  Largest classes:
  `accuracy/mistranslation` minor 80 and major 66,
  `style/awkward-phrasing` minor 79,
  `policy/pronoun` major 61,
  `fluency/grammar` minor 36,
  `fluency/inconsistency` minor 34,
  `accuracy/omission` minor 29 and major 16,
  `terminology/wrong-term` minor 22 and major 11.
- Cluster verdicts:
  accepted 278 (3 critical,
  87 major,
  188 minor),
  needs-human 75,
  rejected 59 (`quality.ts`).
  `TALLY`:
  389 issues,
  254 accepted,
  250 resolved;
  one chunk left an accepted issue open (slice 23,
  5 of 6,
  line 5132).

## What the code does with a slice that ends unfilled

The check traced this at build `1d462324574c`,
in answer to the read's question whether `SETTLED` with `translateStatus=unfilled` was meant to be publishable.

- `translate-lane/src/translate-slice-attempt.ts:209` to `:215`:
  a `TranslateAbsenceError` over an insertion chunk (a chunk with no archive text) becomes `kind: 'unfilled'`;
  any other failure is rethrown.
- `translate-lane/src/translate-slice-buy.ts:168` to `:191`:
  an unfilled attempt logs
  `slice N: no translation in the archive and none produced (<reason>); the passage stays missing and the slice is NOT cached`
  (`:174` to `:178`) and returns `kind: 'unfilled'`.
  The comment at `:169` to `:173` says the next run asks again.
  The log has 48 such lines,
  18 in attempt 1 and 30 in attempt 2
  (the read's "NOT cached by design,
  line 28069" is slice 105's line).
- `translate-lane/src/translate-assemble.ts:307`:
  `status: (unfilled.length === 0) ? 'complete' : 'unfilled'`.
- `translate-floor/src/slice-delivery-decide.ts:142` to `:162`:
  an outcome that is not `decided` with `incumbentKind === 'absent'` gives `{ kind: 'gap-remains' }` and
  `documentText: wording.incumbentText` (empty).
- `page-check/src/corpus-run/publish-fixed.ts:168` to `:195`:
  `shippableReplacements` returns no row for a silent slice at an absent archive span (`:187` to `:188`),
  so `spliceSlices` leaves the archive's text as it was.
  The "wrote N slices" figure on `:458` is `replacements.length`,
  which is why it says 88.
- `page-check/src/corpus-run/publish-completeness.ts:31` to `:38` builds the 30
  `source-passage-unfilled (slice N, no-candidate-backed): the page ships without this passage, recorded as a gap`
  warnings,
  at log lines 32774 to 32803.
- `slice-cache/src/corpus-run/settled-tally.ts:190` to `:192` counts `pageSilent`;
  `:196` writes the literal `status=SETTLED`.
  `translateStatus=unfilled` and `pageSilent=30` are the only places the status carries the gap.

### Whether any check should have caught the missing tail

By the code and the recorded owner rulings,
no check refused it;
the gap was recorded and never shown on the page.

- Page agreement (`page-check/src/corpus-run/page-agreement.ts:98` to `:150`,
  `published-page-check.ts`) compares the page with the artifact,
  and the artifact says `gap-remains`,
  so the page agrees.
  `published-page-check.ts:95` to `:104` says silent slices are "counted rather than checked" and left for a reader
  to weigh against `pageSilent`.
- `verify-published` (`run/src/corpus-run/verify-published-entry.ts:238` to `:274`) prints `silent=<n>` and
  `missing=<n>`;
  the exit stays 0 by the ruling in `doc/design-commitments-shipping-and-the-archive.md` ("A run always ships").
  For this page it would read `silent=30 missing=0`.
- Publish-time checks (`publish-fixed.ts:326` to `:408`):
  page-agreement,
  front-matter,
  archive-original,
  contributor-names,
  destinations,
  headings.
  Only `destinations` fired,
  and only because slice 93's link sat in the lost tail.
  `assertHeadingsStayDistinct` (`page-pass/src/corpus-run/heading-distinctness.ts:5` to `:21`) refuses two source
  headings rendered as one;
  it does not look for a source heading with no counterpart,
  so the absent Part Ten heading passes.
- Footnote integrity (`page-check/src/corpus-run/page-footnote-integrity.ts:9` to `:17`,
  `:37` to `:74`) reads defects "the page carries and the archive did not";
  the five dangling references are in the archive too,
  so it cannot fire.
- Coverage:
  the 31 preparation rounds (log lines 784 to 1316) ask about omissions between paired archive and source blocks.
  The preparation did notice the tail:
  the artifact's `preparation.alignmentFindings` holds
  `alignment structure-mismatch (source section 13: source-only (roster-unpaired); anchored for insertion at offset
  33184)` and the same for section 14,
  and `section-pairing paired 13 of 15`.
  They are counted in `alignmentFindings=45` and were not printed in the log
  (`rg` for `structure-mismatch` and `roster-unpaired` finds nothing).
  Ledger B830 made the driver write them to the log.
- The design is written down:
  `publish-completeness.ts:3` to `:16` (the page ships without the passage since 2026-09-01,
  with the gap recorded) and ledger A8
  (`doc/audit-ledger-page-assembly-and-corpus-run-driver.md:142` to `:146`).
  That comment frames the case as an "insertion ... recovered supplementary content";
  here the missing passage is 31.5 percent of the page and every footnote definition.
  Whether the design was meant to cover a truncated archive's whole tail is not stated anywhere the check read.
  The check's answer to the read's "not traced":
  shippable by design,
  premise unexamined for this scale.
- The owner then ruled,
  on 2026-10-23,
  what a shipped page carries for such a slice:
  "Source text + note"
  (ledger B827;
  the full ruling is in `package/translation-repair/doc/design-commitments-shipping-and-the-archive.md`).
  The fix and the other changes made for this read are in the defects file.
