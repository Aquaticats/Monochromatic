# Translation repair history, first handover document: milestone two accumulation procedure, lexical-graded runs 1 to 20 and the Chinese-anchored judge, 2026-07-17

The procedure for the milestone two accumulation loop,
the first twenty budgeted runs graded by word overlap,
the analysis of the stubborn misses,
and the user decision to grade restoration against the Chinese source with a bilingual judge ensemble.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Resume procedure for the milestone two accumulation loop

RESUME PROCEDURE (historical,
loop closed at milestone-two
declaration).
The user ran an unattended milestone-two accumulation
loop,
"spare no expense,
only stop when genuinely blocked;
back in a
few hours" (2026-07-17).
Standing procedure was one run at a time
(never concurrent;
one stream per model is the fastest dispatch on
this plan):
1. Launch a budgeted benchmark pass:
   `cd <worktree> && mise exec -- node <scratchpad>/repair-benchmark-run.ts`
   where `<scratchpad>` is this session's scratchpad dir.
   The driver
   uses a 25-minute run budget,
   4-minute per-call deadlines,
   a 45-minute
   outer safety net,
   oversamples 6 shuffled small entries (700 to 2500
   chars;
   XIEPT2,
   BI4PBV,
   and shi_Yumiaoya quarantined),
   and grades
   with the zh-anchored judge as headline plus the lexical grader for
   comparison.
2. On completion (task notification re-invokes you):
   read the run's
   output file,
   extract SCORECARD and the per-entry `judgments`
   (headline),
   `detection`,
   and `lexical` lines.
3. Append the run to the milestone-two run log below,
   update the
   JUDGE ACCUMULATED and detection tallies,
   commit with
   `/usr/bin/git` (docs(*) message),
   push (auto-push on).
4. Relaunch the next pass.
   Repeat until the user returns or a genuine
   blocker appears.
Artifacts persist in this session's scratchpad across compaction:
`repair-benchmark-run.ts` (driver),
   `repair-benchmark-accumulator.jsonl`
(one line per run with judge+detection+lexical per seed),
`repair-benchmark-result-<ts>.json` (full per-run dumps;
   from run 33
they also carry `repairedText` per dispatched record,
   commit
`66e0d0f42`),
   and `partial-needle-analysis.ts` (offline per-sentence
coverage report over non-restored seeds in enriched artifacts,
   zero
quota;
   run it when a stable-partial entry re-draws).
THE HEADLINE NUMBER is judge `seededRepairRate` (zh-anchored,
   strict
restored/judged);
   `seededRepairRateLenient` adds partial;
`lexicalRepairRate` is the retired vocabulary grader kept for contrast.
Do NOT set temperature or reasoning effort;
   chat calls MUST stream.
Open question already answered by the user:
   grade against the Chinese
source (done).
   No open questions currently block the loop.

## Lexical-graded runs 1 to 20

Run log (25-minute budget each,
retries active from run 2 on):
run 1 (2026-07-17,
pre-retry):
0.75 (3/4 seeds),
2 entries
dispatched,
both repaired.
run 2 (2026-07-17,
retries active,
1964 s):
0.75 (3/4),
entries
Hangmster (34/34 issues resolved,
1 of 2 seeds) and Everythings99
(27/29,
2 of 2);
one panel round lost four voices and retry round
one recovered them.
run 3 (2026-07-17,
1872 s):
0.5 (3/6),
THREE entries fit the same
budget (retries keep stages complete instead of limping):
DarlinChit
2/2,
s5ehfr9 1/2,
TLL1122 0/2 while still shipping a 29/31-resolved
repair;
critic and panel each lost four voices once and retry round
one recovered both fully.
run 4 (2026-07-17,
1679 s,
no retries needed):
0.33 (2/6):
Katerina 2/2 (40/40 issues resolved),
TLL1122 0/2 AGAIN (same
entry,
same misses as run 3:
reproducible entry-specific failure),
luxuanwen3 near-zero restoration (1/13 and 2/8 words) despite a
23/26-resolved repair.
run 5 (2026-07-17,
1655 s,
detection grading active):
repair 1/5,
detection 2/6;
the split works immediately:
SevenBird both seeds DETECTED,
one restored 8/11,
one editor
under-restoration 6/14 (just under threshold);
wangzihao980 both seeds UNDETECTED,
the same entry as milestone
one's single ensemble miss,
confirming interior omissions as the
critics' weak class at the detection layer;
XIEPT2 returned `blocked-non-translation` and a structural probe
proved the block CORRECT (7365-char zh prose vs 1218-char en
heading skeleton,
a third still CJK):
the guard works on real
corpus data,
and XIEPT2 joins BI4PBV in benchmark quarantine.
run 6 (2026-07-17,
1813 s):
repair 3/4,
detection 4/4:
qiqi233345 2/2;
SS3B_0016 both detected,
the 12-word needle came
back 3/12 (editor compression on the longest needle again) while
the 11-word one restored 6/11.
Accumulated:
15/29 seeds restored (0.52);
detection 6/10 over the
graded runs.
The misses have named classes:
editor compression on long needles (detection fine),
critic
detection on interior omissions (wangzihao980),
and one correctly
blocked non-translation (XIEPT2,
quarantined).
QUANTIFIED (over all 29 graded seeds):
needles over eleven
distinctive words restore at 6/16 versus 9/13 for shorter ones;
missed seeds average a 0.25 return ratio versus 0.67 for restored
ones,
so misses are real failures,
not threshold noise.
REMEDY SHIPPED (commit `a69cfdbe1`):
the editor prompt now forbids
summarizing or compressing omitted content ('a short paraphrase of
a long omission is not a fix');
run 7 predates it,
run 8 onward
measures it.
run 7 (2026-07-17,
1422 s,
last pre-remedy baseline):
repair 4/6,
detection 6/6,
all three entries repaired;
misses are keyword233's
15-word needle (4/15) and CuspariaKLSY's 7-word needle (1/7),
the
first SHORT-needle editor miss,
so compression is dominant on long
needles but not exclusive to them.
Accumulated:
19/35 repair (0.54);
detection 16/20 (0.80) over
graded runs.
run 8 (2026-07-17,
1826 s,
FIRST with the anti-compression rule):
repair 5/6 (0.83),
detection 6/6,
best multi-entry pass yet.
Long needles went 3/4 this run versus 6/16 pre-remedy:
chunchun_yudong's 17-word needle (longest graded) restored 12/17,
SevenBird's 14-word needle that failed 6/14 in run 5 passed 8/14.
The miss is s5ehfr9's 13-word needle again (2/13;
3/13 in run 3),
a repeat offender worth an entry-specific look.
Accumulated:
24/41 repair (0.59);
post-remedy sample is one run,
keep accumulating before crediting the rule.
run 9 (2026-07-17,
2068 s):
repair 3/4,
detection 4/4:
Rentable_A 2/2 with one PERFECT 7/7 restoration,
a2581911655's
16-word needle passed 9/16 while its 18-word needle missed again
(6/18;
5/18 in run 1).
POST-REMEDY TALLY after runs 8 and 9:
8/10 (0.80) versus 16/31
(0.52) pre-remedy.
run 10 (2026-07-17,
1778 s):
repair 0/4,
detection 2/4,
the
hardest pass yet:
Anilovr both seeds detected but under-restored
(2/7 and 2/8,
SHORT needles,
post-remedy),
and wangzihao980
reproduced its both-seeds-undetected failure from run 5,
making it
a STABLE detection hole (interior omissions defeat the critics on
this entry across independent runs).
Post-remedy tally corrected:
8/14 (0.57) versus 0.52 pre-remedy;
the anti-compression rule's benefit is no longer clearly
established,
keep accumulating.
Accumulated:
27/49 repair (0.55);
detection 26/32 (0.81) over
graded runs.
run 11 (2026-07-17,
1717 s):
repair 3/4,
detection 4/4:
MioCardMeow 2/2 with one PERFECT 8/8 and a 13-worder at 7/13;
s5ehfr9's 13-word needle missed for the THIRD straight time
(3/13,
2/13,
3/13):
a stable editor-side repeat offender with
detection fine,
the editor-layer counterpart to wangzihao980's
detection hole.
Accumulated:
30/53 repair (0.57);
post-remedy 11/18 (0.61) versus
0.52 pre-remedy;
detection 30/36 (0.83).
run 12 (2026-07-17,
1488 s):
the retry ladder's sternest live
test:
one chunk lost ALL SEVEN critics in a round,
six again on
retry round one,
and five panelists in another round;
the ladder
recovered quorum every time (zero quorum-unmet findings) and the
single entry that fit the budget completed with 51/51 issues
resolved,
detection 2/2,
repair 1/2.
Retries traded coverage
(0.17) for completeness,
as designed.
Accumulated:
31/55 repair (0.56);
post-remedy 12/20 (0.60);
detection 32/38 (0.84).
run 13 (2026-07-17,
1573 s):
repair 3/4,
detection 4/4:
SU5ZI2MO1's 20-word needle,
the longest graded yet,
restored
11/20 (a class that reliably failed pre-remedy);
xixi_yuexi's
11-worder missed just under threshold at 5/11.
Accumulated:
34/59 repair (0.58);
post-remedy 15/24 (0.63);
detection 36/42 (0.86).
run 14 (2026-07-17,
1639 s):
repair 3/4,
detection 4/4:
coin one perfect 7/7 and one miss (2/11);
xixi_yuexi 2/2 including
the 11-worder that missed run 13 now passing 6/11 (nondeterminism
cuts both ways at the editor too).
Accumulated:
37/63 repair (0.59);
post-remedy 18/28 (0.64);
detection 40/46 (0.87).
run 15 (2026-07-17,
2217 s):
repair 3/4,
detection 4/4;
the
EDITOR retry fired for the first time and recovered a lost editor
voice (pre-quorum that chunk would have shipped unchanged).
Uekawakuyuurei's 9-word seed missed at exactly 3/9 in two
independent runs (12 and 15):
a third stable repeat offender,
editor-layer.
Accumulated:
40/67 repair (0.60);
post-remedy 21/32 (0.66);
detection 44/50 (0.88).
run 16 (2026-07-17,
2042 s):
repair 3/4,
detection 4/4 through a
second whole-roster critic wipeout recovered in two retry rounds;
lxy's 16-word needle near-missed at 7/16,
SevenBird 2/2 again.
Accumulated:
43/71 repair (0.61);
post-remedy 24/36 (0.67);
detection 48/54 (0.89).
The rates have stabilized:
repair around
0.6 overall and about two thirds post-remedy,
detection just
under 0.9.
run 17 (2026-07-17,
2302 s,
calm weather,
zero retries):
repair
3/4,
detection 4/4;
Everythings99 restored both 14-worders;
a2581911655's 18-word needle missed a third time (7/18 after 5/18
and 6/18):
the longest needles trend upward post-remedy but the
18-word class still sits under threshold.
Accumulated:
46/75 repair (0.61);
post-remedy 27/40 (0.68);
detection 52/58 (0.90).
run 18 (2026-07-17,
1858 s):
repair 3/6,
detection 6/6;
TLL1122's
classification settles:
both seeds DETECTED yet under-restored
(6/15 and 5/12),
so it is editor-side,
not a detection hole;
Katerina 2/2 again;
the checker retry recovered two lost voices.
Accumulated:
49/81 repair (0.60);
post-remedy 30/46 (0.65);
detection 58/64 (0.91).
run 19 (2026-07-17,
1729 s):
repair 4/6,
detection 6/6;
chunchun_yudong's 17-worder restored 12/17 a second time,
while
s5ehfr9's 13-worder missed a FOURTH time at a near-constant 3/13
and keyword233's 15-worder repeated its exact 4/15:
the specific
editor misses look entry-deterministic despite editor
nondeterminism,
suggesting the failing needles share a content
property (for the next design pass to identify).
Accumulated:
53/87 repair (0.61);
post-remedy 34/52 (0.65);
detection 64/70 (0.91).
run 20 (2026-07-17,
2272 s):
repair 2/4,
detection 4/4;
a2581911655's 18-worder missed a fourth time at exactly 7/18 while
its 16-worder restored a third time;
new entry ttttsuuukikoo_
(81 issues,
the largest issue count yet) split 1/1.
Accumulated:
55/91 repair (0.60);
post-remedy 36/56 (0.64);
detection 68/74 (0.92).

## Needle-property analysis of the stubborn misses

NEEDLE-PROPERTY ANALYSIS (2026-07-17,
offline,
structural only):
no single structural feature (quote marks,
footnote refs,
parentheticals,
digits,
comma density,
paragraph position,
document position) separates the stubborn misses from reliable
restorations;
both classes span the same ranges.
Combined with
the near-constant per-seed return ratios across independent runs
(3/13 four times,
7/18 twice,
4/15 twice),
the leading hypothesis
is that the editor consistently produces a faithful-but-terse
re-translation of a compact zh sentence,
and vocabulary overlap
against the wordier ORIGINAL EN under-credits it.
CONSEQUENCE:
the lexical seededRepairRate is a LOWER BOUND on true
repair quality.

## Restoration judge anchored on the Chinese source

USER DECISION (2026-07-17):
grade restoration against the Chinese
source.
DONE (commit `81271a63d`):
the headline seededRepairRate is
now a bilingual ensemble JUDGE anchored on zh (`restoration-judge.ts`,
`restoration-judge-wire.ts`).
Judges read the original Chinese,
the
deleted sentence as a content pointer,
and the repaired text,
then
rule restored/partial/absent,
tolerating terse-but-faithful
rewording and requiring zh grounding.
No single judge decides:
the roster (GLM-5.2,
Qwen,
Kimi by default) fans out with
retry-to-quorum and each seed's verdict is the conservative lower
median (an even split rounds toward the less-credited verdict).
Scorecard now reports judgedSeeds,
restoredSeeds,
partialSeeds,
seededRepairRate (strict,
zh-anchored),
seededRepairRateLenient
(restored+partial),
plus the lexical* fields for comparison.
The lexical grader moved to `lexical-restoration.ts`;
the benchmark
takes an injectable `judge` seam (tests stub it) and a
`judgeModelIds` roster.
All prior run numbers (0.60 repair) were
LEXICAL;
the judge rate supersedes them from the next live run and
the two rates print side by side so the gap is visible.
JUDGE LIVE BOUNDARY CHECK (2026-07-17,
`judge-boundary.ts`):
on the
saved DarlinChit repaired text,
3/3 judges heard,
quorum met,
both
seeds ruled `restored` in 15 s for ~2 quota units.
The live judge
works;
the overnight loop resumes on it from run 22.
