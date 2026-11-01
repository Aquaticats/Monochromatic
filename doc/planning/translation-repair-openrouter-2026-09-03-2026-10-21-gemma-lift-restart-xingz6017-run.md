# OpenRouter and the corpus passes: the run of XingZ6017 on build 2a336603bf (2026-10-22)

What was read on 2026-10-22 of the run of the third pass after the Gemma readers were lifted,
the first long entry of the restart:
the log,
the run facts,
where the 268.7 min went,
the verdict on the known classes,
the lost claims checked by hand,
the ten classes named,
and what was not done.
Entry:
XingZ6017 (entry `XingZ60`).
The providers in the order they ran dry,
the stragglers,
the NeuralWatt energy and the picture readings are in
[the providers of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-providers.md),
and the shipped page with its departures from the Chinese is in
[the page of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).
The same build's first two passes are in
[the run of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md)
and [the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md).

## XingZ6017 read, 2026-10-22, 04:07 UTC: SETTLED in 268.7 min over 118 slices; three providers ran dry in sequence

### Verdict in short

The pass settled in 268.7 min over 118 slices.
The page is better than the unfinished archive in 64 of the 88 slices that have an archive counterpart,
equal or mixed in 22,
and equal in wording but worse in one respect in 2;
the other 30 slices have no archive counterpart and are judged against the source alone.
The page keeps its own inconsistencies:
one factual error about when she died,
one game and one contributor called by two names,
and 29 em-dash runs the source does not carry.
Three providers ran dry in sequence,
OpenRouter alone carried the run for about 114 min,
and the consolidation took 99.2 min,
37 percent of the pass.
Ten classes are named (N1 to N10).

The build predates two fixes.
The run's build was `2a336603bf`.
Commit `e336d8efc7` (the first-content window of 180 s,
and the notes passed to the polish judges and writers;
ledger B804 to B807)
and commit `f829fc31b6` (the composite of archive revisions by line difference;
ledger B803)
landed after the run started.
Their merge commits are dated 02:20:38 UTC and 00:14:48 UTC on 2026-10-22,
and the run began at 23:05 UTC on 2026-10-21.
`git merge-base --is-ancestor` finds neither commit an ancestor of `2a336603bf`,
and both are ancestors of the tip `24c736ea80` this record was filed from.
The run therefore did not have them.
Its K2 split (52 mercury abandonments,
43 Qwen,
2 deepseek,
1 minimax)
and its K3 lost claims (6 verified)
are a baseline from before those fixes,
not recurrences after them.
The same holds for K4,
which did not run here.

What the log's end shows about NeuralWatt needs one correction to the first reading of it.
That no stop line fired when NeuralWatt went dry at 01:39:35 UTC is the designed behaviour:
dry is a routing verdict,
and calls already in flight settle.
It is not a defect.
The defect is that the energy estimate under-read the measured energy by 2.84 times,
which ledger B808 recalibrated on 2026-10-22 (UTC).
Two more of the findings went to ledger entries on 2026-10-22 (UTC):
rounds held open by a hung stream (B809),
fixed in part by a first-content window of 90000 ms on the Mercury card that is open to the owner's veto,
and seats re-asked after their calls were abandoned (B810),
where a hold was examined and rejected because it forwent more answers than it saved cuts.
The coverage rounds over an untranslated tail (B811) were examined and kept:
they are the first of the two signatures the absence verdict requires,
so the length bound alone cannot stand in for them.

### Run facts

`XingZ6017` (entry `XingZ60`) ran on build `2a336603bf`
and pipeline `sha256-tree-v1:5d4265dfab83b5a6e61b5b8c23d597519c7ca24293065473c883ec82af54c78c`.
Its `START` line is log line 196.
The first timestamped line is line 198 at 23:05:27.851 UTC on 2026-10-21,
and the publish line is line 41165 at 03:34:09.402 UTC on 2026-10-22
(118 slices into a page of 49,429 characters).
The brief gave 23:05:12 UTC as the start of the pass.
The log has 41,183 lines (`wc --lines`);
the `DONE` line is 41168 and the 12 `SEAT` lines (41171 to 41182) and "Finished in 16137.14s" (41183) follow it.
The reader read the pass log,
the run directory
(the artifact `XingZ60.json` at 12.9 MB,
`prompt-payloads/` with 6845 reply-only files,
and `fixed/people/XingZ60/page.en.md`),
and the source entry
(`page.md` of 577 lines,
`page.en.md`,
the unfinished archive,
of 358 lines,
`info.yml`,
`photos/image0.webp`,
which was viewed as an image,
and `profile.jpg`).
No file in any git checkout was edited,
no pass,
probe,
gate,
build,
test or provider call was run,
no `.env*`,
auth or `environ` file was read,
and no process was started or left running.
Code was read with `git show` and `git grep` at `2a336603bf`.
The scratch scripts and extracts of the read are not kept in the repository;
every count in this record was made by a script over the log or the artifact,
or by an `rg --count` over the log.
The log crosses midnight UTC:
times from 00:00 UTC onward fall on 2026-10-22.

Two measurements of an early draft of the read were wrong and are corrected here.
A count of calls that ended with `0 content chars` matched the text as a suffix and found 93 events;
parsing the number finds the true 13.
The reconciliation of the NeuralWatt meter had dropped negative readings;
with a leading minus allowed it reads 222 meter lines and a last reading of -5.188 Wh.

The `TALLY` line (log line 41166) reads:

```text
TALLY XingZ60 status=SETTLED slices=118 repairStatus=repaired repairIssues=316 repairAccepted=222 repairResolved=209 repairFindings=2376 repairChanged=80 translateStatus=complete translateChanged=114 documentsDiffer=107 pageChanged=114 pageSilent=0 alignmentFindings=22 selection=contested ms=16120404
```

The `DONE` line (41168) reads `elapsed=16121634ms`,
which is 268.7 min.
`DESTINATIONS source=2 page=2 dropped=0` is line 41167.
The log has 0 `[error]` lines,
3794 `[warn]` lines and 29,299 `[info]` lines.

Beside the two reads of the same build (`zheermao22`,
`CuspariaKLSY22`):

- Slices:
  118 here,
  15 in `zheermao22`,
  4 in `CuspariaKLSY22`.
- Pass time:
  268.7 min here,
  79.0 min and 39.3 min there.
- Per slice:
  2.3 min here (lanes run 4 slices wide),
  5.3 min and 9.8 min there.
- Contested slices:
  107 here.
  The 11 slices with no selection record are the 8 whose lanes agreed (`both-agree`)
  and the 3 where the archive stands (`archive-stands`);
  the 107 are `translate-only` 35,
  `both-differ` 71 and `repair-only` 1.
  Slice 2 is one of the 11 and still carries a refusal disposition (D19).
  `documentsDiffer=107`.

The archive in the source is an unfinished English version:
it has no counterpart for the last 30 slices (88 to 117).
Insertion admission therefore saw `untranslated tail: 30 slice(s) after the last agreed pair, 5131 source code points
at expansion 2.94, expected 15074, last pair renders 147: admitted on that bound`
(log line 1201,
23:30:32.269 UTC).
The lane state shows 88 slices with an incumbent and 30 with none;
repair delivery is `replacement-shipped` for 80,
`incumbent-retained` for 8 and `gap-remains` for 30,
and translate delivery is `replacement-shipped` for 114 and `incumbent-retained` for 4.
Lane selection over the 107 contested slices:

- lane-won translate 49,
- lane-won repair 8,
- sole-lane translate 30,
- settled-neither with the archive endorsed 9,
- settled-neither with the archive declined 9,
- settled-neither with no archive stance 2.

Consolidation ended as `consolidated` 30,
`gate-kept-standing` 33,
`slate-declined-standing` 20,
`slate-endorsed-standing` 18 and
`slate-unjudged-standing` 6.
Page assembly rewrote 9 slices and withdrew none (log lines 41155 to 41164).
Page consistency ran one group and changed no slice (`group 1 of 1 ... changed slices=0`,
lines 41153 and 41154).
The log has no `archive block` or `archive composite` line (0 matches),
so the archive block review did not run (K4).

### Wall clock

The brief's first question was where the 268.7 min went.
The phase boundaries are from the log markers,
all UTC.

- Preparation before pictures:
  23:05:27 to 23:16:50 UTC,
  11.4 min.
  It holds the page-title lexicon (to 23:07:56 UTC,
  with mercury cut at the 120 s grace),
  section pairing (23:08:03 to 23:08:26 UTC)
  and block pairing in 24 rounds (23:08:28 to 23:16:50 UTC,
  three of them waiting 120 to 127 s for mercury).
- Pictures plus insertion admission:
  23:16:50 to 23:30:32 UTC,
  13.7 min.
  The pictures took 65 s (23:16:50 to 23:17:55 UTC).
  The remaining 12 min are 30 per-slice `coverage` rounds over the untranslated tail
  (23:18:40 to 23:30:32 UTC,
  42 round-min,
  25.8 of them in grace,
  9 rounds at 100 s or more),
  after which the tail was admitted on a length bound (line 1201).
- Repair chunk pass:
  23:30:32 to 00:36:14 UTC,
  65.7 min.
  88 slices were computed and 30 had `no-translation`.
  The mean slice took 131 s and the median 87 s.
- Repair refine pass:
  00:36:14 to 01:07:21 UTC,
  31.1 min.
  Rounds are serial per slice (refiner,
  select,
  checker,
  probe).
- Translate lane:
  01:07:24 to 01:48:01 UTC,
  40.6 min.
  The mean slice took 81 s.
- Lane contest:
  01:48:01 to 01:54:38 UTC,
  6.6 min.
  Every slice but one was done by 01:52:07 UTC;
  slice 40 held the phase to 01:54:37 UTC.
- Consolidation:
  01:54:38 to 03:33:50 UTC,
  99.2 min,
  over 107 slices.
- Page consistency,
  assembly and publish:
  03:33:50 to 03:34:09 UTC,
  0.3 min.

The phases add to 268.8 min.
Preparation,
25.1 min of it,
is 9 percent of the pass;
repair is 96.8 min (36 percent),
translate 40.6 min (15 percent),
contest 6.6 min (2 percent),
consolidation 99.2 min (37 percent).
In slice-minutes the repair lane cost 258,
the translate lane 160 and consolidation 392;
with at most 4 slices in flight per lane,
wall time is about a quarter of slice-minutes.

#### Consolidation slowed in two steps

The mean consolidation cost of a slice,
by the half-hour in which it ended,
was
110 s at 01:30 UTC,
149 s at 02:00 UTC,
315 s at 02:30 UTC,
346 s at 03:00 UTC and
104 s at 03:30 UTC.
The slowest slices were 95 (881 s),
85 (846 s) and 72 (807 s).
The rise from 149 s to 315 s starts at the moment Synthetic became wet again (02:39:44 UTC),
and the cause is traced in the section on N2 in
[the providers of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-providers.md).

#### Where the waiting was

Scripts parsed every round line
(`N/M heard, T ms total, Q ms to quorum, G ms in grace`,
or `no quorum`).
Over the whole pass there are 1900 rounds,
990 round-minutes,
444 of them in grace,
and 118 rounds with 100 s or more of grace.
By stage the largest are `select` (534 rounds,
265 round-min),
`absolute-naturalness-review` (129 rounds),
`produceConsolidations` (117 rounds,
70.4 grace-min),
`panel` (203 rounds) and `refiner` (132 rounds).

Round-minutes by window,
with the rounds that ended at the grace limit (119 s or more in grace),
the rounds where quorum itself was held for 250 s or more,
and the remaining grace:

- Preparation:
  49 round-min;
  22 grace-min in 11 rounds that ended at the limit;
  no quorum-held round;
  12 other grace-min.
- Repair chunk:
  263 round-min;
  52 grace-min in 26 rounds at the limit;
  5 quorum-held rounds (25 min);
  67 other grace-min.
- Repair refine:
  113 round-min;
  8 grace-min in 4 rounds;
  5 quorum-held rounds (26 min);
  53 other grace-min.
- Translate:
  158 round-min;
  10 grace-min in 5 rounds;
  none quorum-held;
  27 other grace-min.
- Lane contest:
  19 round-min;
  none at the limit;
  1 quorum-held round (5 min);
  no other grace.
- Consolidation:
  388 round-min;
  130 grace-min in 57 rounds at the limit;
  13 quorum-held rounds (66 min);
  62 other grace-min.

Round-minutes overlap (several rounds run at once),
so they are not wall minutes.
Scaling each window's share by its phase wall time (a proportional estimate,
weak for preparation) gives
about 62 min of wall clock spent on rounds that ended at the grace limit,
about 32 min on rounds held by a hung stream with quorum unmet,
and about 60 min in grace that ended when the last voice was heard.
In consolidation alone the estimate is about 33 min at the limit,
17 min held and 16 min in other grace:
about half of that phase was waiting on streams that never answered.
That is an estimate from proportions;
the wall-clock overlap was not measured exactly.

### Class verdict on the known classes

- K1,
  text calls ending at the cap with no content:
  present,
  13 events,
  29.91 Wh
  (in the providers file).
- K2,
  streams with no content cut only by the grace:
  present,
  52 mercury abandonments
  (45 at 120 s,
  7 at 180 s),
  all `0 answer, 0 reasoning`;
  43 Qwen (runaway reasoning),
  2 deepseek,
  1 minimax.
  This split is a baseline from before the fixes of commit `e336d8efc7`,
  which the run did not have.
- K2 variant,
  the dead stream that holds the quorum:
  present,
  24 rounds of 250 s or more (N1).
- K3,
  an accepted minor repair claim lost because the polish writers never saw it:
  present,
  6 verified lost claims in the section "K3,
  hand-checked".
  This is a baseline from before the notes passed to the polish judges and writers by `e336d8efc7`
  (ledger B805),
  which the run did not have.
- K4,
  archive block review ran with no unclaimed blocks:
  not exercised (0 `archive block` lines).
  The tail went through insertion admission instead.
  The composite fix of `f829fc31b6` (ledger B803) was not in the build either.
- Hyper alphabetical key order:
  98 of 99 Hyper vote-bearing replies are vote-first,
  and 237 of 304 Hyper multi-key objects are in alphabetical order.
  Elsewhere none is alphabetical:
  NeuralWatt 240 of 460 objects reason-first,
  OpenRouter 2773 of 3288 (acceptable 505,
  best 1507,
  choice 761),
  Synthetic 39 of 73.
- Bedrock:
  one 401 ask,
  then held out;
  `bedrock=wet` on all 222 `METERS` lines and `bedrock=dry` on 30 of 31
  `JUDGE SEATS` lines.
- Abandoned calls not counted by the seat tally:
  confirmed
  (the providers file;
  examined and rejected as ledger B810).

### K3, hand-checked

The word heuristic that counted accepted-issue regions with editor-added words
(211 regions with editor-added words;
16 with every added word absent from the final slice and 81 with some missing)
is an upper bound,
because it counts synonyms as losses.
The reader hand-checked the 16 and the minor partials with a real missing word
against the repair delivery,
the polish text and the lane selection.

The 16 rows are 14 distinct edits (slice 24 appears four times,
slice 40 three times).
Of the 14:

- Carried in meaning,
  not lost:
  slice 5 (tense:
  "had always been"),
  slice 13 ("behaviour" removed with a restructure),
  slice 16 ("another leaned female"),
  slice 40 ("the way they thought"),
  slice 48 ("Addendum."
  for "A supplement."),
  slice 53 ("looked up to" in the past).
- A wrong editor fix the page rightly refused:
  slice 9 (editor "Yi Liang Zi" for the Irako signature).

The ones whose point is plainly not on the page:

- Slice 0,
  "in my mouth" for 含着 (page line 14):
  the claim was accepted 2 to 2 `needs-human`
  and again through a repair region;
  verdict `settled-neither`,
  archive endorsed;
  the final polish text lacks it.
- Slice 12,
  "The dead come first":
  verdict `settled-neither` with no archive stance;
  the final is the archive text ("The dead are the greatest",
  page line 110).
- Slice 21,
  the "Xingyun (Nebula)" and "Huanmeng (Illusory Dream)" glosses:
  lane-won translate;
  the contest ballot called the repair lane's parentheticals unsupported,
  so this loss is a judged one.
- Slice 24,
  "everywhere in chains" (Rousseau,
  footnote [^1]) and "the world becomes a disguised arena":
  lane-won translate;
  the page reads "always in shackles" and "the world became a disguised Colosseum"
  (page line 236 area).
  "Sorrows and joys do not connect" is carried in part ("are not shared").
- Slice 71,
  "Xiao Xu" for the 小序 signature:
  lane-won translate;
  the page reads a rendering of the literal "Short Preface"
  (page line 735).
- Slice 33,
  "down here" in "they couldn't do it down here either":
  partial,
  minor;
  the page has "dropped a tier ... into my life".

Verified:
6 claims in 5 slices are lost in substance (slices 0,
12,
21,
24 twice,
71),
with slice 33 and one slice-24 claim partial.
Four of the six (slices 21,
24 twice, 71) are on the lane-won path,
which is the K3 path as the brief defined it;
the other two (slices 0 and 12) are `settled-neither` paths where the archive or the neither-polish stood.
The reader did not re-trace in code whether the polish writers saw the claims on the lane-won path;
the path is read from the verdict in the artifact.
The other 65 partial rows that the heuristic flags are word variations that were sampled and found carried
(slices 68,
69,
70,
73,
79 sampled,
and 14 and 44 read by eye).
Slice 1 "final wishes" is a judgement call:
the contest judges called "final" unsupported (the page reads "own wishes"),
so it is counted neither lost nor kept.

### Recurrence of earlier findings

- The comment-only stream (K2):
  present again on the unfixed build,
  52 mercury cuts (baseline before the fixes of `e336d8efc7`).
- The cap reading with no content on NeuralWatt (K1):
  recurred in text calls,
  13 events;
  the picture call was fixed by `B719`.
- Lost accepted minor claims (K3):
  present again on the unfixed build,
  6 verified (baseline before the fixes of `e336d8efc7`).
- The archive block review (K4):
  not exercised.
- Hyper alphabetical key order:
  recurred (237 of 304).
- Bedrock credential:
  recurred (one 401,
  held out).
- The comparison with `zheermao22` and `CuspariaKLSY22`:
  both had few slices.
  This is the first read of a 118-slice entry,
  so the provider exhaustion is new at this length.

### New classes

N1,
a hung stream holds the quorum:
when the live bench is about four voices,
quorum equals the bench and the grace never starts.
24 rounds waited 250 s or more (lines 15503,
16005,
16732,
17268,
17524,
27349).
It is the K2 mechanism with a different bound:
K2 is cut by the grace,
this is cut by the gateway at about 301 s.
Fixed in part as ledger B809,
by a first-content window of 90000 ms on the Mercury card (open to the owner's veto).

N2,
a provider returning at a low balance re-seats a seat that then costs grace:
Qwen came back at 02:39:44 UTC (2 percent weekly),
40 of its calls were cut with no answer
between 02:43:28 and 03:29:10 UTC,
and consolidation slices ran 315 to 346 s.
The tally never counts abandoned calls,
so the seat is never withheld.
Examined and rejected as ledger B810.

N3,
the NeuralWatt estimate (prompt-based) under-reads by 2.84 times over the pass,
because the cost sits in reasoning output it does not see,
and the free day ran out mid-run at 01:39:35 UTC.
The ratio follows the entry rather than climbing with the date:
over the six runs on this build it read 1.23,
2.00,
0.87,
1.39,
1.94 and 2.84 in the order they ran,
and `XingZ60` alone sent most of its calls with 16,384 prompt tokens or more.
Recalibrated as ledger B808.
That no stop line fired at the dry reading is the designed behaviour and not part of this class.

N4,
`minimax-m3` content overrun in `select` rounds:
20 warn lines `ended a call that exceeded its content bound ... against a bound of 32000`
(first line 7014 at 00:05:16 UTC,
last line 40798 at 03:32:10 UTC;
up to 62,494 characters),
all on `minimax/minimax-m3`,
and the seat tally shows `threw=21` for it.
The time cost was not measured.

N5,
insertion admission asked 30 per-slice `coverage` rounds over a tail that was then admitted on a length bound
(line 1201):
about 12 min of preparation,
9 rounds at 100 s or more of grace,
including two Qwen reasoning streams of 197 s and 130 s and a mercury stream of 294 s ended by a 502 at 23:30:25 UTC.
Examined as ledger B811 and kept unchanged:
the rounds are the first of the two signatures `doc/decision/translation-repair-absence-verdict.md` requires,
and a majority `carried` or `partly-carried` tail verdict keeps a row out whatever the bound says.
Ending a round once its outcome is fixed could save at most about 7 of the 12 min,
which the straggler-grace decision (`doc/decision/translation-repair-straggler-grace.md`) does not yet allow,
and the arrival order of voices that would measure the saving is not logged.

N6,
the consistency pass reaches too little:
`group 1 of 1 ... changed slices=0` while D02,
D03,
D04,
D08,
D09 and D10 sit on the shipped page.
A fix direction:
let the consistency pass take the glossary of rendered terms
(game name,
断档,
药圈,
好死喵,
螐儿)
and the contributor-name restoration read the credit line as well as headings and signatures.

N7,
typography changes with no finding:
65 slices differ in quote style between the artifact and the page,
2 are logged (D20;
policy or gap not traced),
and 42 `refused by the page grammar`
warns mean some HTML regions were not normalized at all.

N8,
decisions below quorum:
44 `no quorum ... every ask settled` rounds
(11 lane contest,
15 select,
4 polish gate,
2 translate,
12 absolute-naturalness),
plus 11 `quorum-not-met` naturalness reviews.
They settled,
as designed
(the wording is printed by `stage-round-line.ts:99`),
but a lane contest decided by 3 of 7 voices is a thin decision.

N9,
OpenRouter spend:
9.218 USD against 0.13 and 0.07 on the prior reads,
paid after Hyper and NeuralWatt ran dry.
It is not a defect;
it is a budget fact the owner should see.

N10,
dash density on the page:
29 em-dash runs in 19 slices with no source dash (D18).

### What changed against the earlier reads

- Slices:
  118 against 15 and 4;
  the first entry long enough to exhaust three providers.
- Picture reads:
  four readers,
  GLM finished at 1619 tokens instead of the cap;
  Gemma now reads through OpenRouter.
- The page:
  no hallucinated archive content remains in slices 48 and 67 to 70.
- Energy:
  2.84 times estimate against 1.94.

### Open items and what the read did not do

- The 49 Wh of plan energy not explained by this pass's overshoot.
- Whether the polish writers saw the lost claims (K3) on the lane-won path:
  not traced in code.
- Whether N2's wasted time would have been saved by not seating Qwen:
  not tested.
- The minimax overrun's time cost,
  the outcome of the sixth `deepseek` K1 re-ask,
  and the `quorum-not-met` stage list:
  not measured.
- Wall-clock overlap of the waiting rounds:
  estimated by proportion,
  not measured.
- Whether the reduced prompt-token counts on two picture calls mean a down-scaled image:
  inference.
- The reader read no prompt (the run directory holds replies,
  not prompts),
  did not render the page,
  ran no `agent-browser` check,
  and probed no provider.
- The source entry's directory is not a git checkout,
  so the source was read as it stands in the working tree.
