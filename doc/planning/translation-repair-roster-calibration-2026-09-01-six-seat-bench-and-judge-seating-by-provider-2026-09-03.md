# Roster calibration: the six-seat bench smoke run and seating judges by provider reach, 2026-09-03

The keyword233 smoke run of the six-seat bench on HEAD,
with the wrap and damage-claim fixes that followed it;
the decision to seat the Hyper-slow judge per provider,
superseding the morning's drop,
and its live witness;
the XIEPT2 postscript run,
where reading the meter once per entry cost two hours,
and Kimi-K3 measured per role.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## keyword233 smoke-tested the six-seat bench on HEAD at 04:44 UTC in 19 minutes

XIEPT2 landed on `4b5c8b438`;
`4ad08d5dc` then unseated Qwen3.8-27B from every judge round,
which no
entry had exercised (wide seats seven to six,
quorum four to three;
late judges eight to seven).
The
advisor asked for an end-to-end run on HEAD before any claim about it,
so keyword233 ran on `4ba479df0`
at overlap 4 into `~/temp/agent/keyword233-smoke-20260903`,
Hyper alone,
launched at 04:25 into a window
still holding about half of XIEPT2's last hour:
`TALLY keyword233 status=SETTLED slices=3 ...
repairIssues=5 repairAccepted=4 repairResolved=4 translateChanged=2 pageChanged=2 pageSilent=0
alignmentFindings=1 selection=contested ms=1164461`;
`verify-published` 1 of 1 pages,
every wording,
at
the promised length.
240 successful calls,
no refusal,
no pacer wait,
7 cuts against 22 on the
2026-09-02 landing (241 calls,
22 minutes).
Six-seat rounds heard 6 of 6 ten times,
5 of 6 three times
and 4 of 6 once,
every one above the quorum of three;
no round declined for want of voices;
one lane
contest settled on neither with all seven usable,
which is a tie,
not a loss.
The page is three
paragraphs of fresh wording,
faithful to the source,
the same facts and the same links as the
2026-09-02 page.
Repair raised 5 issues against 25 on 2026-09-02,
and the logs say where:
not in the
critic seat.
Qwen3.8-27B was abandoned in both critic rounds and both panel rounds on 2026-09-02 as
well,
so it proposed nothing then either;
the critics heard were the same six seats both days,
with 28
claims (19 and 9) on 2026-09-02 and 23 (9 and 14) on the smoke.
The difference is panel acceptance:
panels of 4 of 7 and 5 of 7 heard passed 28 of 28 claims on 2026-09-02;
panels of 6 of 6 and 5 of 6
heard passed 5 of 23 on the smoke.
Whether a fuller panel corroborates less,
or this draw's claims
were weaker,
needs the same claims put to both benches;
the drop of Qwen cost no critic recall on this
entry.

One difference is a defect,
not the bench:
the 2026-09-02 page came out one clause per line and the
smoke page came out as single-line paragraphs,
though both logs say the lanes rewrapped 2 of 3 slices.
The wrap runs inside the lanes,
so a page whose final text comes from the contest or the consolidation
slate (the smoke's contest settled on neither) ships whatever shape that stage produced.
XIEPT2's
alternating shape is the same defect.
The fix that keeps the stated `MD1` policy is one wrap over the
final page text at publish,
and it is filed as tracked work rather than built inside this record.

Read closer,
the site is narrower than "the lanes":
the lanes and the consolidation slate already wrap
(`consolidate-wrap.ts`,
the #162 rule that deciders judge the bytes that ship),
and the smoke's polish
gate lines say what happened:
"consolidation polish gate:
7/7 usable,
settled on polished,
ships
polished" on both slices,
with a gate judge choosing the polish because it "removes the stilted line
breaks".
The polish round handed the refiner's single-line rewrite to the gate beside the wrapped base
and shipped it as written.
`e539f80cc` wraps the refinement before validation and the gate
(line-structured slices excepted),
demotes a refinement that is only the base re-wrapped,
and ships the
wrapped text as `proposedText` so the digest reader's equality holds;
the polish test proves the gate
was asked about the wrapped bytes,
the governed slice stays as written,
and the pure re-wrap demotes (6
FAIL lines with the wrap removed,
none with it).
`1ffbfad9b` moves the settle and publish fixtures to
the wrapped bytes.
Publishing itself stays byte-faithful,
as `publish-fixed.ts` argues.
keyword233
relaunched on that tree at 04:57 UTC to show the page keeps its line breaks.

It did:
`~/temp/agent/keyword233-wrap-20260903`,
`TALLY keyword233 status=SETTLED ... ms=936060` (15.6
minutes,
220 calls,
no refusal,
5 cuts),
`verify-published` 1 of 1 at the promised length.
The log
carries the new line,
"semantic wrap:
rewrapped the polish before its gate,
3 lines as emitted against 8
as written",
the gate then settled on polished with 7 of 7 usable,
and the page's three paragraphs
ship one clause per line (13 body lines against the smoke page's 5).
Reading finding,
left standing
under the single-attempt design:
this draw's first paragraph is in the present tense ("is a
transgender woman from China.
She runs her own Telegram channel,
where she shares ...") while the
third reports her death;
the 2026-09-02 and smoke pages had "was" and "started".
Traced through the
artifact rather than guessed:
the translate lane's text has "was" at every record;
the repair lane's
editor wrote "is" (`repairRegions[].before` "was",
`editorAfter` "is"),
the introduced-defect probe on
that chunk reported "3 regions,
1 added-damage corroborated" and the repair shipped regardless,
the lane
contest chose the repair lane 7 of 7,
the consolidation slate declined so the standing text stayed,
and
the polish round never touched slice 1 (`changed: false`,
no gate).
So the wrap and the polish gate are
clear of it;
the question it raises is what a corroborated added-damage claim is for if the repair
still ships,
and that is filed as tracked work.

keyword233 ran on that tree (`b2a39c09a`,
07:05 UTC,
`~/temp/agent/keyword233-claims-20260903`):
`TALLY keyword233 status=SETTLED ... repairIssues=18 repairAccepted=17 ... ms=653347` (10.9 minutes,
213 calls of which 90 went to Synthetic,
whose weekly meter read 2% at launch,
and 123 to Hyper;
no
refusal;
2 cuts),
`verify-published` 1 of 1 at the promised length,
and the first paragraph in the past
tense this draw.
The probe corroborated nothing on this draw (0 of 5 regions),
so the evidence block
was not shown live;
the wire test is its witness until a draw carries a claim.
Lane contests:
one
settled on repair with 7 of 7 usable,
one on neither.

## The Hyper-slow judge is seated per provider, superseding the morning's drop

Asked with four options,
the owner chose "seat per provider reach".
`corpus-run/run-seats.ts` reads
Synthetic's meter once per entry through the routed client's quota surface and derives the benches:
with Synthetic dry the Hyper-slow judges (`HYPER_SLOW_JUDGES`,
Qwen3.8-27B today) are withheld from
the wide and the late seats;
with Synthetic wet they sit;
an unreadable meter seats the full bench,
since it is not evidence of dryness.
The static benches in `run-config.ts` are the Synthetic-wet ones
again (seven wide,
eight late),
and the two drop sets hold GLM-5.3-Flash alone as on 2026-09-02.
The
lane contest call moved beside the consolidation seam (`pass-contest.ts`) to keep the entry driver
inside its line budget.
Decision record:
`doc/decision/translation-repair-provider-aware-judge-seat.md`.
The unit test proves the withheld seat with the meter dry,
the seated one with it wet,
and the full
bench with the meter unreadable (fails with the rule removed);
the live dry path is the six-seat bench
keyword233 already landed on twice today,
and the live wet path waits for Synthetic's week.

It did not have to wait:
Synthetic's weekly meter read 1.5% at 07:18 UTC,
wet by the router's
threshold,
so keyword233 on `383ece933` (`~/temp/agent/keyword233-seats-20260903`) logged `JUDGE SEATS
synthetic=wet wide=7 late=8 hyper-slow seated=yes` and ran the wet path live:
`TALLY keyword233
status=SETTLED ... repairIssues=33 repairAccepted=25 ... ms=960731` (16 minutes,
110 calls on Synthetic
and 139 on Hyper,
no refusal,
11 cuts),
`verify-published` 1 of 1.
Every Qwen3.8-27B call went to
Synthetic (32 `SPEND provider=synthetic` lines,
no Hyper-spelled `qwen3.8-27b` line at all),
so the
overflow the decision record allows for did not happen at overlap 4.
It sat in 21 judge rounds (2
critic,
2 panel,
6 repair-lane select,
5 translate-lane select,
2 lane contest,
2 consolidation gate,
1 polish gate,
1 consolidation select) and answered 16;
the 5 it was cut from (2 panel,
2
translate-lane select,
1 lane contest) were cut on Synthetic,
at 67 to 85 seconds of reasoning,
so
Synthetic serves it slowly too,
only far less often than Hyper (5 of 21 against 30 of 34 and 21 of 24
on XIEPT2).
The rounds heard 7 of 7 thirteen times and 8 of 8 six times.
The weekly meter fell from
1.5% to 0.87% over the run,
so the next entry may well read dry and seat six.

Read for the probe,
the same run corrected an assumption:
its log line `introduced-defect probe: 3/3
heard over 1 regions, 3 added-damage corroborated` at 07:23:10 came from the naturalness rewrite's
probe on slice 1 (`refinementDefects`,
three probers quoting "is a transgender woman from China.
She
opened",
the tense rule),
not the accuracy probe,
whose regions found nothing.
The lane contest at
07:27:43 chose translate for that slice 5 of 7 on the same tense shift and was shown nothing,
because
`damageClaimLinesBySlice` read only `introducedDefects`.
Fixed at `2530537a2`:
both probes' corroborated
claims become lines,
each naming its edit,
the contest block's prose names the two edits and their
starting texts,
and the driver logs `slice N: M corroborated damage claims shown to the judges` so a
run's log witnesses the block.
Guard test shown to fail with the rewrite read removed.

keyword233 ran on that tree (`2530537a2`,
07:52 UTC,
`~/temp/agent/keyword233-rewrite-claims-20260903`),
Synthetic still wet (`JUDGE SEATS synthetic=wet wide=7 late=8 hyper-slow seated=yes`):
`TALLY keyword233
status=SETTLED ... repairIssues=31 repairAccepted=23 ... ms=845916` (14.1 minutes,
107 calls on
Synthetic and 120 on Hyper,
no refusal,
3 cuts,
one of them Qwen),
`verify-published` 1 of 1 at the
promised length,
the first paragraph in the past tense.
The probes corroborated nothing on this draw (0
of 4 regions,
both edits),
so no `corroborated damage claims shown` line appeared and the unit test
remains the block's only witness;
the contests settled on neither (8 of 8) and on repair (8 of 8).

## XIEPT2 on the postscript rule: the page says it one way, and the seat rule's blind spot cost two hours

The reader-protection sentence of `c7a00eea8` (method words with no substance stay vague;
one page,
one way) had no live witness,
so XIEPT2 ran on `2530537a2` (08:12 UTC,
`~/temp/agent/xiept2-postscript-20260903`):
`TALLY XIEPT2 status=SETTLED ... repairIssues=9
repairAccepted=5 ... translateChanged=27 ... ms=13136520` (219 minutes,
261 calls on Synthetic and
1,936 on Hyper,
no refusal,
184 cuts),
`verify-published` 1 of 1 at the promised length.
The rule
applied:
the scheduled-message obituary now reads "Xiafeng Grape took her own life in an unmonitored
room…" where the ORIGINAL names a lethal drug and a toxic gas,
the narrative postscript reads "She put
on her last light-colored dress and ended her life in the night" where rerun5 had "took medication and
breathed in gas",
and the second obituary keeps `ending her life by "euthanasia"` as the owner decided.
The probes corroborated nothing (0 of 3 regions),
so the contest block again had no live draw.

Where the time went,
measured off the log.
`JUDGE SEATS synthetic=wet` at 08:16:53;
the first
`METERS synthetic=dry` at 08:19:00,
seven minutes in,
with the weekly meter at 0%.
The seat rule reads
once per entry,
so Qwen3.8-27B stayed seated on Hyper for the remaining three and a half hours:
107 of
the 184 cuts were its,
it was abandoned in 102 judge calls (59 select,
18 naturalness review,
9 polish
gate,
7 lane contest,
5 consolidation gate,
4 other),
and 75 rounds waited the full 60 s grace (43
select,
14 naturalness review,
9 polish gate,
5 lane contest,
3 consolidation gate,
1 panel).
Consolidation of the 28 contested slices took 134 minutes (09:37 to 11:51) against 78 for both lanes.
That is the decision record's "the seat follows the state that decides most calls" limitation with a
price on it,
and a diagnosed defect with a fix that keeps the policy:
read the meter again at each phase
boundary (lane contest,
consolidation),
which would have withheld the seat from 08:19 on.

Done at `4797b06a7`:
the lanes keep their reading in the entry driver,
the contest and consolidation
seams read their own as their phase starts,
and the `JUDGE SEATS` line names the phase.
The entry test
counts meter readings across a settled entry and expects three (shown to fail,
"expected 1 to equal
3",
with the later reads removed).
keyword233 ran on that tree (12:00 UTC,
`~/temp/agent/keyword233-phase-seats-20260903`) and witnessed the mechanism live in the other
direction:
`phase=lanes synthetic=dry wide=6 late=7` at 12:00:08 and `phase=lane contest synthetic=dry`
at 12:07:42,
then Synthetic's weekly meter regenerated from 0% to 2% and `phase=consolidation
synthetic=wet wide=7 late=8 hyper-slow seated=yes` at 12:08:51;
Qwen3.8-27B answered 11 calls on
Synthetic in that phase,
the consolidation gates heard 8 of 8 twice,
and the earlier select rounds
heard 6 of 6 seven times.
`TALLY keyword233 status=SETTLED ... repairIssues=12 repairAccepted=8 ...
ms=881936` (14.7 minutes,
35 calls on Synthetic and 206 on Hyper,
no refusal,
6 cuts),
`verify-published` 1 of 1,
first paragraph in the past tense,
no corroborated claim this draw.

## Kimi-K3 measured per role: select-slow on Hyper, fine elsewhere, fine on Synthetic

The decision record named Kimi-K3 the next per-provider candidate and left it,
because the coarse
benches would have cost its critic and panel seats.
Counted per role instead (abandoned in / rounds):

- Toka_ls 2026-09-02,
  731 calls on Synthetic:
  select 0 of 69,
  lane contest 0 of 14,
  critic 0 of 14,
  panel 0 of 13,
  gate 0 of 12.
- XIEPT2 rerun5 2026-09-03,
  1,852 calls on Hyper:
  select 43 of 83,
  lane contest 0 of 28,
  critic 0 of
  10,
  panel 0 of 5,
  gate 0 of 17.
- XIEPT2 postscript run 2026-09-03,
  1,936 calls on Hyper:
  select 38 of 101 (20 of 42 translate-lane,
  18 of 55 consolidation slate,
  none in the repair lane),
  lane contest 1 of 28,
  critic 0 of 9,
  panel
  1 of 5,
  gate 0 of 14.
  Its cut streams averaged 71 s,
  its answered ones 14 s.

So the slowness is the slate prompt on Hyper,
not the model or the provider alone.
The "18 of 24" the
seating record carried for rerun5 was consolidation-slate select,
not lane contest (the provider-aware
record's rejected-alternative bullet mislabeled it),
and it holds:
26 of 47 slate rounds on rerun5,
18
of 55 today.
Rerun5 was mixed-provider for Kimi-K3 (93 calls on Synthetic,
135 on Hyper) and 55 of its
61 cut streams were Hyper-served.
Under the owner's
authorisation to drop a model from a role on evidence:
`HYPER_SLOW_SELECT_JUDGES` withholds Kimi-K3
from both lanes' select seats and the consolidation slate while Synthetic is dry and keeps it as critic,
panel,
contest judge and gate;
`JudgeSeats` carries `selectJudges` and `slateJudges` beside the wide and
late benches,
and the `JUDGE SEATS` line reports all four.
Dry benches:
six wide,
five select,
seven
late,
six slate.

Landed at `493b2f259` (unit test shown to fail with the select filter neutralised).
keyword233 ran on
it (12:23 UTC,
`~/temp/agent/keyword233-select-seats-20260903`) with Synthetic wet at every phase
(1.8% weekly,
`wide=7 select=7 late=8 slate=8` three times),
so the full benches sat and the plumbing
is what the run witnessed;
the withholding branch has the unit test as its witness until a dry
reading.
`TALLY keyword233 status=SETTLED ... repairIssues=22 repairAccepted=19 ... ms=775753` (12.9
minutes,
108 calls on Synthetic and 135 on Hyper,
no refusal,
4 cuts),
select rounds 7 of 7 nine times
and 8 of 8 twice,
`verify-published` 1 of 1,
first paragraph in the past tense,
no corroborated claim.
