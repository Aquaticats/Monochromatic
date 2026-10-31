# Audit ledger: the XingZ6022 pronoun count and the slates that could not decide

The owner's answer of 2026-10-29 to the question of the ambiguous `their` on the page of the corpus pass `XingZ6022`,
"Keep the rule;
measure first (Recommended)",
and the count that followed (B1051),
and the purchase that still paid for slates that could not decide (B1053),
found while placing the swap of B1050.
B1050 is filed in
[Audit ledger: the two-ballot translate text and the lead's adoptions of 2026-10-29](audit-ledger-the-two-ballot-translate-text.md).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1051: `their` shipped for the person beside a plural subject, as the neutral pronoun rule asks

Status:
recorded on 2026-10-29 (UTC),
read offline,
no code changed.
The owner question was answered on 2026-10-29 (UTC):
"Keep the rule;
measure first (Recommended)".
Measured the same day,
below;
the rule stands.

The defect:
page line 88 reads "XingZ60's parents dressed up their appearance to match a traditional male gender expression at
their funeral",
and line 90 "threw their ashes",
where `their` means the person who died but the parents are the subject of the same sentence.
The archive had `her`.

Which path shipped it:
slice 13,
the repair stand-in.
The archive was disputed (two accepted claims,
log line 25959),
the repair lane and the translate lane wrote the same `their` in all three places,
the contest opened no round (1 of 7 seats),
and the slate's single judge named the translate text alone,
which differs from the stand-in only in the spelling
`behaviour` (lines 21067,
26091).
So neither lane's text could have mended it.

The cause is not a stage fault.
The original writes `TA` at each of these places ("在 TA 的葬礼上把 TA 的外貌",
"把 TA 的骨灰",
"TA 父母"),
and the house rule renders `TA` as singular they and calls a pronoun left in the other form a fault
(`house-rule/src/house-policy.ts:91`;
the floor that withheld the archive at slice 14 is
`translate-floor/src/translate-neutral-pronoun.ts`).
The archive's `her` was the hand translation breaking that rule.
Both lanes obeyed it,
and nothing in the rule or in any sheet tells a writer to name the person where singular they sits beside a plural
noun it could be taken for (`rg 'ambigu' house-rule/src translate-floor/src` finds no such text).
No deterministic check can decide the ambiguity,
because it turns on who the sentence is about.

Owner question (answered),
with the options as the lead ranked them:
- Option A,
  keep the rule as it stands.
  It breaks no ruling,
  and costs nothing.
  It leaves sentences like line 88,
  whose number was not counted (19 page lines hold `their`;
  how many are ambiguous
  was not measured).
- Option B,
  add one sentence to the pronoun rule in `house-policy.ts`:
  where singular they sits in a sentence with a plural noun it could be taken for,
  the sentence repeats the person's name or the word "the deceased" for that pronoun.
  It mends line 88 for any run that reads the rule.
  It changes sheet text that every stage showing the house rule carries,
  so the cache versions of those stages move by the cached-decisions rule,
  and it cannot be checked without model calls.
- Ranking:
  Option A > Option B.
  Option A leads because the prevalence is unmeasured and Option B moves many caches for one sentence of one page.
  Measure how many corpus sentences put singular they beside a plural noun first,
  and the order may change.

Owner answer,
2026-10-29 (UTC),
question tool,
"Keep the rule;
measure first (Recommended)":
no rule change,
and a count of how many shipped-page lines in settled pages put singular `they`,
`their` or `them` for the person beside a plural noun or a plural subject,
so that the reader cannot tell who is meant.

Method,
keyless and with no model call,
by scratch scripts under `~/temp/agent/tr-fan/w381-rulings/` (`gather.ts`,
`count.ts`,
`sample.ts`,
`they-lines.ts`,
`parents.ts`,
`funeral.ts`):
- Settled pages are the shipped English pages of the runs under `~/temp/agent`
  (a run directory that holds `fixed/people/<id>/page.en.md` and the `attempts.json` a corpus pass writes beside it:
  241 pages in 239 runs,
  of 40 entries),
  and the 92 pages `data/people/*/page.en.md` of the pinned corpus (`a41fc607ea`,
  the tree clean).
  The attempts file tells a corpus pass from a copied tree,
  and it does not tell a probe or verification run made with the pass from the numbered passes;
  the runs of `XingZ60` that decide the result are all numbered passes.
- A line is a non-blank physical line outside front matter and fenced code.
  A hit is a sentence of a line that holds a they-form (`they`,
  `their`,
  `theirs`,
  `them`,
  `themselves`,
  `themself` or a contraction of `they`)
  and,
  in the same sentence,
  a plural person noun from a closed list of 49 words and phrases (`parents`,
  `friends`,
  `people`,
  `others`,
  `both`,
  `we` and the like) or two capitalized names joined by "and".
  What the list lacks is not counted.
- Whether a they-form stands for one person cannot be read off a page.
  The house rule renders the original's `TA` as singular they and nothing else (`house-policy.ts:91`),
  so only an entry whose original writes `TA` can carry the defect by the rule.
  Of the 40 entries in the runs,
  one does (`XingZ60`);
  of the 92 in the pinned corpus,
  two do (`XingZ60` and `lxyddice`).

Counts:
- The latest page of each of the 40 entries in the runs:
  5,602 lines,
  289 with a they-form,
  80 hit lines,
  of which 18 are in the one page of an entry that writes `TA`
  (`corpus-pass-XingZ6022`,
  488 lines,
  50 with a they-form).
- Every page of the 241 in the runs:
  49,601 lines,
  2,737 with a they-form,
  825 hit lines in 164 pages.
  The 21 pages of `XingZ60` hold 393 of them,
  240 by distinct sentence,
  because the same entry went through 21 passes.
- The pinned corpus:
  5,767 lines,
  446 with a they-form,
  150 hit lines in 49 pages.
  The two pages whose original writes `TA` hold 12 of them,
  all in `XingZ60`.
  `lxyddice` has none,
  and no corpus pass has run it.

Hand check of 43 hits,
all read:
the 18 of `corpus-pass-XingZ6022`,
the 13 sentences (12 lines) of the pinned corpus pages whose original writes `TA`,
and 12 drawn with a fixed seed (381) from the 200 hit lines of the entries that do not write `TA`.
- 2 of the 43 are the defect:
  page lines 88 and 90 of `~/temp/agent/corpus-pass-XingZ6022/fixed/people/XingZ60/page.en.md`,
  the two sentences of this entry.
  The false-positive rate of the count is 41 of 43,
  95%
  (16 of 18 on the one page that can carry the defect,
  89%;
  13 of 13 in the pinned corpus,
  whose hand translation writes `her` and `TA's`;
  12 of 12 in the entries that do not write `TA`).
- The false positives are plural they for a group ("some people blamed themselves",
  line 201),
  generic they ("whatever they want",
  line 95),
  "both sides ... them" (line 146),
  a possessive of the person beside a plural noun where the reader can tell ("in the eyes of their parents",
  line 93),
  and a they-form for a quotation's or a poem's own subject (lines 446 and 529).
- Sample of hits read,
  as path and line:
  `corpus-pass-XingZ6022` `XingZ60` lines 11,
  88 (defect),
  90 (defect),
  93,
  95,
  146,
  150,
  199,
  201,
  379,
  422;
  `a41fc607ea` `XingZ60` lines 70 and 102;
  and from the other entries,
  `xiept2-stub-20260903/XIEPT2` line 298,
  `a41fc607ea` `hulicaijia` line 57,
  `Zha_Ke` line 56,
  `windward0032` line 294,
  `Everythings99` line 33,
  none of them the defect.
- All 50 lines of that page that hold a they-form were also read in full.
  Two are the defect (88 and 90),
  both found by the count,
  so the count missed nothing on that page.
  Lines 106 and 392 use a they-form for the person with no plural noun beside it,
  and are not the defect.

Result:
- The defect is two lines in the latest settled page of one entry of 40,
  two lines in 5,602 (0.04%).
  Of the 92 pages in the pinned corpus,
  none carries it.
- It is not a new fault of the last pass.
  Across the 21 runs of `XingZ60`,
  the sentence about the parents' funeral writes `their` for the person in 13 pages,
  and the sentence about the ashes writes `their ashes` in 13,
  26 lines in 14 pages.
  The other pages wrote `her` or the name in those sentences.
- The count is a lower bound on what the list names and no bound on the sentences it lacks.
  The 188 hit lines of the entries that do not write `TA` that were not read (12 of 200 were) are not known to be
  free of it,
  although the house rule gives such an entry no singular they.
  `lxyddice` writes `TA` and has never gone through a pass,
  so its rate is unknown.

Ranking after the measure:
Option A > Option B stands.
The order was open on an unmeasured prevalence,
and the measure puts it at two lines on one page,
while Option B changes the sheet text every stage that shows the house rule carries,
so it moves their cache versions and cannot be checked without model calls.
A pass over `lxyddice` would show whether a second page carries the defect.

Cache versions:
none moved,
because no stage output changed.

## B1053: the purchase still pays for writers and a slate that cannot decide over an unendorsed standing nothing overrules

Status:
recorded on 2026-10-29 (UTC),
measured from the log of `XingZ6022` and read from the code.
The owner question was answered on 2026-10-30 (UTC):
"Skip when below floor (Recommended)".
Fixed in `d05477f0c4` (the fix),
with its red cases in `27dc5a78da`
and a correction of one comment of the fix in `ad9ec58831`.

Found while placing the swap of B1050.
The purchase skips the slate when fewer judges are reachable than a winner needs
(`benchBelowFloor` in `consolidate-slice-buy.ts`),
but for an eligible standing that skip needs an empty lane offer
(`laneTexts.length === 0`).
An eligible standing the contest did not endorse has the lane texts on the offer,
so the purchase asked the writers and the slate however few judges were reachable.

Measured on the log:
75 slices of `XingZ6022` had a consolidation slate round,
91 rounds in all,
and every one heard 1 judge of 7
(`select round: 1/7 heard`,
with `bench short of quorum: seats reachable 1 of 7` beside them).
One judge alone never decides (2026-10-25),
so none of those rounds could end in anything but the standing,
and since B1050 the swap overrules the standing where the detector flags it.
How many calls the 75 slates cost was not counted;
the 65 calls and 10.679 Wh of B1020 are the figure for the 16 slices with a refused standing,
not for these.

After B1050 the purchase skipped only the slices where the swap fires.
Skipping the writers and the slate for every eligible unendorsed standing whose bench is below the floor would change
what is bought and no outcome,
by the same ruling,
and the settlement would be marked and never cached as the other bench-below-floor settlements are.

Owner answer,
2026-10-30 (UTC),
question tool,
"Skip when below floor (Recommended)":
skip writers and slate for an eligible standing the contest did not endorse
whenever fewer judges are reachable than a winner needs.
The settlement is marked and never cached,
as the other bench-below-floor settlements are.
Where the B1050 swap fires
(the tense detector flags the standing and not the two-ballot translate text)
it still swaps,
and the skip covers the rest.
The basis is that one judge alone never decides (2026-10-25),
so such a slate can only keep the standing.

What changed,
in `consolidate/src/consolidate-slice-buy.ts`:
- `benchBelowFloor` now holds for an eligible standing whose offer is empty
  or whose standing the contest did not endorse,
  where it held for an empty offer only.
  An endorsed eligible standing with lane texts on the offer is still asked.
  The driver never builds that shape,
  because `laneTextsForSlate` returns no lane text for a standing that is both endorsed and eligible,
  so nothing was ruled about it and nothing was changed there.
- The settlement of the skip is given no lane texts.
  `settleConsolidation` takes the exit for a slate with nothing valid on it only when the offer is empty,
  so a settlement that held the lane texts would have gone on to ask the slate judges with no writer heard.
  The swap is read first and keeps the lane texts,
  so it ships as before.
- The log line is unchanged for an empty offer.
  With lane texts it adds a last clause,
  `2 lane texts on the offer go unjudged`,
  to the same line.
  The line names the slice,
  the reachable count (`1 of 7 slate judges reachable`)
  and the needed count (`below the floor of 2`).

The order of the reads (the open note of round w381):
`presentTenseSwapOf` ran before the `slateReachable` check,
at every purchase with an eligible standing the contest did not endorse,
an entry that records a death and a translate text on the offer with two ballots,
whatever the bench.
Its result was used only when the bench was short and the winner was offered,
so the parse of both texts was unused at every purchase on a bench that met the floor.
It is now read only where its answer is used:
the bench is short,
the winner is offered,
and the standing is eligible.
The settlement is the same in every case;
what differs is the read.
`tenseCandidateGroups` is not free of effects,
since it writes a warning when the shared slice grammar refuses a text
and rethrows any other error,
so a purchase on a healthy bench no longer writes that warning for a read nobody used,
and no longer meets an error from it.
`shipWinnerBeforeSlate` and `swapAfterUnseatedSlate` read the detector where their results decide,
and are unchanged.

Guards shown to fail first:
`consolidate/src/consolidate-bench-floor-lane-offer.unit.test.ts`
(three of six cases failed on the old code:
the two-lane offer and the repair-text-only offer,
each of which made 12 calls and ended in an interruption instead of buying nothing,
and the skip line,
which the old code did not write;
the other three are the controls:
the swap still ships,
a bench that meets the floor or was not counted still buys,
and the empty-offer skip is unchanged).
The cases run with 7 slate judges seated,
as the bench of `XingZ6022` held.
This is stated from the code and the tests,
not from a replay of the run.

Cache versions:
none moved.
- `CONSOLIDATE_CACHE_VERSION` (43) stays.
  The skipped settlement carries `benchBelowFloor`,
  which `consolidationWorthResuming` refuses,
  so it is never stored.
  A slate bought on a bench of one judge for an unendorsed eligible standing could not end in a consolidation,
  and polish returns not-run for such a standing,
  so none of those was stored either.
  A record written before was written on a bench that met the floor,
  and the key does not hold the bench,
  so it is read before any purchase and is still right.
- The artifact generation (22) stays,
  and no key was added or changed.
  The skipped slice is recorded with the existing `benchBelowFloor` mark on the consolidation slice,
  terminal `incumbent-only`,
  no verdicts and no gate,
  which `parseBenchFloorMark` already reads.
- The versions of the translate stage,
  the lane contest,
  the consistency round and the page titles stay:
  no sheet,
  floor,
  threshold or settlement rule of those stages changed,
  and the change only withholds a purchase.

Ids used:
B1051 and B1053.
B1050 is in
[Audit ledger: the two-ballot translate text and the lead's adoptions of 2026-10-29](audit-ledger-the-two-ballot-translate-text.md).
B1052 and B1054 are in
[Audit ledger: found while fixing, names, headings and math on the page](audit-ledger-found-while-fixing-page-names-headings-and-math.md).
B1055 to B1059 are unused.
