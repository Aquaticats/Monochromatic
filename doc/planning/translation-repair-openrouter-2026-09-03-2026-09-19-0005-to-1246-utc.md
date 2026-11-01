# OpenRouter and the corpus passes: 2026-09-19, 00:05 to 12:46 UTC

What was launched,
read and decided on 2026-09-19 00:05 to 12:46 UTC.
Entries:
XingZ616,
XingZ615,
XingZ614,
XingZ613,
XingZ612,
XingZ611,
XingZ610,
XingZ608,
v4.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## XingZ616 launches on classes sixty-five and sixty-six, 2026-09-19, 12:46 UTC

Frozen `6eada935a` (classes forty-one to sixty-six and the tail budget),
pid 1017938,
runs `~/temp/agent/XingZ616`,
log `~/temp/agent/XingZ616.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Read first the translate lane's `translate-candidate-refused` findings in the artifact
(the withheld shapes)
and the closing poem's slice
(91 on XingZ615's roster,
the `> 热情的仿佛炽热的骄阳` quote with its attribution and the `</details>`):
its contest winner passing the invariants,
no `slate-declined-standing`,
the poem on the page with the attribution inside the quote;
then the admission,
the translate assembly,
the memorial disclosure block with its four paragraphs,
the bilibili link,
the nine definitions,
the run time against XingZ608's 2h16m,
then the seven steps and the three checks.

## Classes sixty-five and sixty-six, 2026-09-19, 12:40 UTC: a respaced copy of the original, and a refused candidate on the translate slate

Found by XingZ615's consolidation.
The pass stopped INCOMPLETE at 12:24 UTC,
2h41m in,
at the consolidation of slice 91,
the closing poem of the `七句破题` block
(five quoted verses,
the attribution `<p>` inside the quote,
then `</details>`;
two blocks by the rule).
Two defects,
one behind the other.

Class sixty-six is why the standing was ineligible:
the translate lane's slate carried a candidate that moved the attribution line out of the quote
(three blocks,
`blockquote, paragraph, paragraph`,
against the original's two),
which the deterministic rule had refused and its author's repair turn had not resolved;
the lane keeps such a candidate on the slate
(the repair turn of 2026-08-15),
the judges do not apply the rule,
and they chose it over two renderings that kept the attribution inside the quote.
The lane contest chose translate 6 of 8 and its guard wrote
`contest winner fails publication invariants and remains retryable`
and moved on;
the consolidation withheld the standing.

Class sixty-five is why the slate then had nothing:
the repair lane ships nothing for an insertion slice,
the translate text was the withheld standing,
and of the consolidation proposals the floor accepted exactly one,
the Chinese original returned with a space after each bare `>` line
(payload `c0ed84346b`,
12:21:27 UTC).
`untranslatedFindings` compared the candidate to the original character for character after trimming the ends,
so the respaced copy passed.
Three of four judges abstained
(`not a translation; a verbatim copy of the Chinese original`),
gemma-4-e2b chose it twice,
weight 1 against the minimum of 2,
`slate-declined-standing`.
(The judges' abstention reasons are not logged;
they were read from the payload cache.)

Fixed in `6eada935a`
(guards red first at `636d56a8c`,
`translate-untranslated.unit.test.ts` and `translate-floor.unit.test.ts`):
the untranslated check compares in all but whitespace
(`translate-untranslated.ts`,
the finding now reads "in all but whitespace"),
and `translate-floor.ts` withholds from the translate slate every candidate the rule still refuses after its repair turn,
recorded as `translate-candidate-refused (model): reasons`;
a rule that cannot say keeps the candidate,
and the repair turn and the model's own defence are unchanged.
The 2026-08-15 decision that an invalid candidate goes back to its author once is kept;
what changes is that after that turn a text the page cannot carry no longer reaches the judges.
The owner may veto this reading.

XingZ615 otherwise:
class sixty-four not exercised
(Inception served Mercury 952 of 952,
no pass-on,
no hold);
the repair lane 41 minutes at a mean of 91 s a slice
(XingZ613 295 s,
XingZ608 62 s),
the translate lane 45 minutes at 137 s,
the lane contest 113 of 120 differ,
consolidation from 11:40 UTC;
the `其九：空白` slice settled on its first slate round
(gpt-oss's proposal,
weight 2 of 4),
so class sixty-three was not exercised either;
the admission at 10:05 UTC with the tail on the bound and nothing unresolved;
no footnote revert or withdrawal at the translate assembly;
Jev 252 asked,
252 usable;
10.34 USD by the meters
(Bedrock 143.81 to 139.53,
OpenRouter 210.46 to 204.40),
8,070 seats,
246 retry rounds.

## XingZ615 launches on class sixty-four, 2026-09-19, 09:50 UTC

Frozen `fc428eaee` (classes forty-one to sixty-four and the tail budget),
pid 985599,
runs `~/temp/agent/XingZ615`,
log `~/temp/agent/XingZ615.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
XingZ614 was killed at 09:50 UTC,
12 minutes in,
still in preparation
(no admission,
no Mercury pass-on yet),
0.04 USD by the meters.
Admission read at 10:05 UTC,
15 minutes in:
the tail admitted on the bound
(31 slices of 120,
5,144 source code points at expansion 2.93,
expected 15,083),
no `placement unresolved`,
no Mercury pass-on and no hold yet
(Inception's limit had lifted by launch).
Read next the repair lane's pace against XingZ613's
(`SLICE-COST lane=repair` mean 295 s there,
62 s on XingZ608)
and the `held out for another` refusals where Inception limits Mercury
(a `passed on` line,
then held refusals for 60 s,
then one more ask);
then the consolidation of the `其九：空白` slice
(`translate-runoff (finalists 2 of N)` where the first round splits,
no `slate-declined-standing`);
then the admission,
the translate assembly,
the memorial disclosure block with its four paragraphs,
the bilibili link,
the nine definitions,
the run time against XingZ608's 2h16m,
then the seven steps and the three checks.

## Class sixty-four, 2026-09-19, 09:45 UTC: the rate-limited seat is asked again on every round

Measured on XingZ613's lanes
(the subsection under the class sixty-three heading).
Fixed in `fc428eaee`
(guard red first at `17084ca5e`,
`provider-router-model-hold.unit.test.ts`):
`upstream-model-hold.ts` keeps a hold per model,
the router starts it on an upstream-model refusal
(class sixty-two's passed-on 429)
for `UPSTREAM_MODEL_HOLD_MS`,
60 s,
the provider hold's own backoff,
and refuses the model at call time as `NoProviderForModelError`
(`its upstream endpoint rate-limited this model; held out for another Nms`)
until it ends,
so a round counts the seat unreachable and neither asks nor waits for it;
the provider and its other models are untouched,
and the hold restarts on the next refusal.
The router's pass-on line now names the hold.

## XingZ614 launches on class sixty-three, 2026-09-19, 09:38 UTC

Frozen `4193495fa` (classes forty-one to sixty-three and the tail budget),
pid 966877,
runs `~/temp/agent/XingZ614`,
log `~/temp/agent/XingZ614.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Read first the consolidation of the `其九：空白` slice
(84 on XingZ613's roster):
`translate-runoff (finalists 2 of N)` where the first round splits,
a run-off that decides,
no `slate-declined-standing`;
then the admission,
the lanes,
the translate assembly,
the memorial disclosure block with its four paragraphs,
the bilibili link,
the nine definitions,
the run time against XingZ608's 2h16m,
then the seven steps and the three checks.

## Class sixty-three, 2026-09-19, 09:30 UTC: the run-off over every backed candidate is the same question

Found by XingZ613's consolidation.
Slice 84 is the `### 其九：空白` heading with the greeting line and the centred h3,
three content lines the archive renders as `### __` and the h3 alone,
so the standing failed the line rule
("2 lines of content where the ORIGINAL has 3")
and was withheld from the slate
(the third ineligible-standing addendum).
Four valid proposals went to a bench at quorum
(4 of 5 reachable,
glm-5.3 dry;
gemma-26b at half weight):
gemma-26b and Mercury for candidate 4,
deepseek for 1,
gemma-e2b for 3,
the leader at 1.5 against the absolute minimum of 2.
The class fifty-three challenge narrowed the run-off to the candidates that drew a ballot,
which was three of the four,
and the same panel split the same way
(1.5,
1,
1,
one abstention),
so the slate settled `slate-declined-standing` and the entry stopped INCOMPLETE at 3h02m over a heading whose every rendering was valid
("Part Nine",
"The Ninth",
the heading kept in Han).
Fixed in `4193495fa`
(guard red first at `9176c9819`,
`translate-runoff-leaders.unit.test.ts`):
`runoffFinalists` offers the leaders alone,
the candidates tied at the top on a tie,
the leader and its runner-up on a plurality under the minimum
(runner-up by weight,
then ballots,
then slate position),
so the challenge is a two-candidate question a bench of whole and half ballots settles unless it abstains.
The class fifty-three shape
(two candidates named once each of three)
narrows as before;
every candidate tied at the top,
a rejection,
or fewer than two backed still offer the whole slate.
XingZ613 otherwise:
class sixty-two live
(54 pass-on lines against Mercury 2.5,
no provider hold,
no bench stop),
Mercury asked 935 and threw 147;
Jev 276 asked,
276 usable;
no footnote revert or withdrawal on the translate assembly;
lane contest 112 of 119 slices differ;
7.81 USD by the meters
(Bedrock 148.02 to 143.85,
OpenRouter 214.16 to 210.52),
8,025 seats,
262 retry rounds.

### The rate-limited seat slows every round (class sixty-four, open)

XingZ613's repair lane took 1h39m against XingZ608's 35m
(119 slices at a mean of 295 s against 120 at 62 s).
Measured over the round lines,
the stages Mercury sits on waited in grace for its rate-limited ladder after quorum:
select 14.1 s mean grace against 5.8,
panel 22.5 against 1.0,
critic 25.0 against 4.1,
editor 32.2 against 0.0,
introduced-defect-probe 30.0 against 0.3,
checker 17.8;
the stages without it
(lane contest,
naturalness review,
coverage)
ran as before.
Class sixty-two lets the seat go dark for the round,
but the next round asks it again and waits its five-attempt ladder out.
The remedy is the class sixty-two hold scoped to the model:
a model whose upstream endpoint rate-limited it is held out for the rate-limit backoff and refused at call time as unreachable,
so rounds neither ask nor wait for it until the hold ends.

## XingZ613 launches on class sixty-two, 2026-09-19, 06:22 UTC

Frozen `c340b496b` (classes forty-one to sixty-two and the tail budget),
pid 919708,
runs `~/temp/agent/XingZ613`,
log `~/temp/agent/XingZ613.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Admission read at 06:51 UTC,
29 minutes in:
class sixty-two live,
54 "passed on its upstream endpoint's rate limit" lines against Mercury 2.5,
no `markRefused] openrouter`,
no `writing bench unreachable`;
the tail admitted on the bound
(31 slices,
88 to 118 of 119,
5,175 source code points at expansion 2.94,
expected 15,215);
no `placement unresolved` on any slice.
No container deficit this run:
the roster paired the memorial block's third and fourth paragraphs together with the archive's one
(section 8,
"paired 7 of 7 original and 6 of 6 translation blocks across 7 relations",
relation `4>3`),
so the block has no source-only slice and the repair lane carries both paragraphs as one content slice;
read the block on the page for all four paragraphs.
Read next the lanes:
`passed on its upstream endpoint's rate limit` lines against Mercury 2.5 with no `markRefused] openrouter` beside them,
no `writing bench unreachable`,
then the translate assembly,
the memorial disclosure block with its four paragraphs,
the bilibili link,
the nine definitions,
the run time against XingZ608's 2h16m,
then the seven steps and the three checks.

## Class sixty-two, 2026-09-19, 06:15 UTC: one model's upstream rate limit holds the whole provider out

Found by XingZ612's lanes.
Inception's shared pool rate-limited Mercury 2.5 from 06:06 UTC,
and OpenRouter passed each refusal on as
`429 {"error":{"message":"Provider returned error","code":429,"metadata":{"raw":"inception/mercury-2.5 is temporarily rate-limited upstream. ...","provider_name":"Inception","provider_error_code":"rate_limit_exceeded","limit_source":"upstream_provider_shared_pool",...}}}`.
The transport ladder rode five attempts in 10 s and handed the reply to the router,
which read every 429 as the provider's budget refusal
(`isBudgetRefusal`)
and held the whole of OpenRouter out for 60 s
(`markRefused`,
"refused us while its meter reads wet",
16 holds between 06:06 and 06:12 UTC).
Every OpenRouter seat read unreachable for the minute,
the editors and refiners have no seat elsewhere while Synthetic and Hyper are dry,
and the seat wait stopped the entry at the hold's end:
`writing bench unreachable at lanes: editors 0 of 3 reachable, floor 2; refiners 0 of 3 reachable, floor 2; no provider has named its return`.
Three attempts,
31 minutes,
2.4 minutes and 1.7 minutes,
the same wall each time;
the meters never moved off wet
(Bedrock 149.20 to 148.02,
OpenRouter 215.18 to 214.17,
2.19 USD).
Mercury was asked 262 times and threw 89.
Fixed in `c340b496b`
(guard red first at `79b3ee845`,
`provider-budget-refusal.unit.test.ts`):
a 429 whose body opens with `"message":"Provider returned error"` is one model's upstream endpoint refusing,
not the account's budget
(`isUpstreamModelRefusal`);
`isBudgetRefusal` excludes it,
the router logs
`passed on its upstream endpoint's rate limit for this model; the seat is lost this round and the provider stays where its meter puts it`
and rethrows,
so the seat is dark for that round and the provider stays where its meter puts it.
A bare 429 and a 402 stay budget refusals.
The OpenRouter error page could not be fetched
(client-rendered);
the shape is taken from the live body.
XingZ612's admission at 06:00 UTC had confirmed class sixty-one live
(slice 70 admitted inside details of slices 67 to 71,
5 blocks against 4)
with no slice unresolved,
the first pass with none.

## XingZ612 launches on class sixty-one, 2026-09-19, 05:39 UTC

Frozen `75f67fbd4` (classes forty-one to sixty-one and the tail budget),
pid 895538,
runs `~/temp/agent/XingZ612`,
log `~/temp/agent/XingZ612.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Admission read at 06:00 UTC,
21 minutes in:
class sixty-one live,
`insertion-container-deficit-admitted (slice 70 inside details of slices 67 to 71: the original writes 5 blocks there, the archive 4)`;
the tail admitted on the bound
(31 slices,
5,144 source code points at expansion 2.98,
expected 15,318,
last pair renders 173);
no `placement unresolved` on any slice,
the first pass with none.
Read next the translate assembly,
the memorial disclosure block on the page with its four paragraphs,
the bilibili link,
the nine definitions,
the run time,
then the seven steps and the three checks.

## Class sixty-one, 2026-09-19, 05:30 UTC: the interior budget refuses a paragraph the archive's block is short of

Found by XingZ611's admission,
17 minutes after launch,
and named a class by the owner.
Class sixty read as designed:
all 31 tail slices admitted on the bound,
the five splits among them
(95,
106,
107,
113 and 118)
included,
nothing unresolved in the tail.
Slice 70 was found absent
(split on XingZ608 and XingZ610)
and left "placement unresolved after the single round (verdict absent)":
the interior keeps the whole-page budget,
and on this page that budget is zero,
since the archive's translated half runs at 2.99 code points per source code point against the median's 2.65.
Slice 70 is the third paragraph of the memorial disclosure block
(172 source characters,
"her departure is an indictment of society and a call to each of us").
The archive's block is a translation of another revision of that memorial,
the Metal Gear Solid one,
three paragraphs where the original writes four and a summary none of whose sentences the current original carries;
the roster paired what it could and left the third paragraph source-only.
The translate lane bought nothing for it
("the page has no room to be missing it")
and it would have shipped as a recorded gap.
Fixed in `75f67fbd4`
(guard red first at `1083b9163`,
`insertion-container-deficit.unit.test.ts`):
inside a container both of whose halves the archive carries,
the original's block count against the archive's is the deterministic second signal the budget could not give,
and absent verdicts inside are admitted while the deficit lasts,
in document order,
before the container halves
(`insertion-container-deficit.ts`,
the finding `insertion-container-deficit-admitted (slice N inside details of slices A to B: the original writes S blocks there, the archive T)`).
A container the archive carries with as many blocks as the original admits nothing,
and a split with an anchored claim is not an absent verdict.
XingZ611 was killed at 05:37 UTC,
1 hour and 42 minutes in,
both lanes over
(no withdrawal,
no footnote revert on the translate assembly),
6.91 USD by the meters
(Bedrock 151.81 to 149.25,
OpenRouter 219.58 to 215.23),
5,675 seats,
111 retry rounds.

## XingZ611 launches on class sixty, 2026-09-19, 03:55 UTC

Frozen `de25b4ed8` (classes forty-one to sixty and the tail budget),
pid 856143,
runs `~/temp/agent/XingZ611`,
log `~/temp/agent/XingZ611.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Read first the admission:
the `untranslated tail:` line,
then no `placement unresolved` on slices 89 to 119
(a split there now carries `insertion-split-in-tail` and is admitted on the bound),
then the translate assembly,
the bilibili link,
the nine definitions,
the run time,
then the seven steps and the three checks.

## Class sixty, 2026-09-19, 03:50 UTC: a split verdict inside the admitted tail stays unresolved

Found by XingZ610's admission,
19 minutes after launch.
The tail line read as designed:
"31 slice(s) after the last agreed pair,
5175 source code points at expansion 2.99,
expected 15455,
last pair renders 173:
admitted on that bound".
Twenty-eight of the 31 were admitted;
slices 102,
108 and 113 were left "placement unresolved after the single round (verdict split)",
since the tail rule admitted absent verdicts and unanchored splits alone,
and each of the three had one voice anchor a claim of coverage on the page
(the cached replies quote archive lines the pairing assigned to nobody,
the archive's own "This version is NOT the formal version" note among them).
Slice 113 is the second footnote definition,
which split on XingZ608,
XingZ609 and XingZ610 alike;
on XingZ608 it shipped as a recorded gap.
Slices 102 and 108 were absent on both earlier runs.
Fixed in `de25b4ed8`
(guard red first at `6420d96e8`,
`insertion-tail-split.unit.test.ts`):
inside a tail the bound admits,
a split is no majority and the bound decides it;
a verdict a majority carried still stays out.
The finding `insertion-split-in-tail (slice N, full F, partial P, absent A of K asked; no majority, the tail bound decides)`
names the tallies beside the corroboration.
XingZ610 was killed at 03:53 UTC,
29 minutes in,
69 repair chunks started,
2.08 USD by the meters
(Bedrock 152.96 to 151.85,
OpenRouter 220.62 to 219.65),
2,074 seats,
26 retry rounds.

## XingZ610 launches on the tail budget, 2026-09-19, 03:24 UTC

Frozen `5bda732de` (classes forty-one to fifty-nine,
the deepseek route,
the untranslated tail's own budget),
pid 834115,
runs `~/temp/agent/XingZ610`,
log `~/temp/agent/XingZ610.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8` on the launch line,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Read first the admission's `untranslated tail:` line
(31 slices after the last agreed pair,
about 5,100 source code points at the page's own expansion of about 2.9,
expected about 14,900 against the last pair's rendering,
"admitted on that bound"),
then every tail slice's `insertion-corroboration (slice N, tail admitted, ...)` finding
and no `placement unresolved` on slices 89 to 119,
then the translate lane's assembly
(no `assembly-footnote-reverted unresolved-reference`,
no `assembly-withdrew-every-replacement`),
then the bilibili link in the poem attribution on the page,
then the nine definitions on the page under the original's labels,
then the run time against XingZ608's 2h16m,
then the seven steps and the three checks.

## The untranslated tail is budgeted from the pairing, 2026-09-19, 03:20 UTC: the owner's "Do #1"

The owner answered the shortfall budget question at 02:55 UTC with "Do #1":
a budget per aligned region,
the untranslated tail's own expectation read off the pairing's last agreed pair.
Built in `5bda732de`
(guards red first at `c6c869267`),
after a measurement that narrowed the region to the tail.

The measurement,
over every run that left both an artifact and a log
(the artifact's comparison rows give each slice's archive text,
the repair lane's `SLICE-START` lines its source size,
the admission's `coverage=` lines the roster's verdict):
a window of one paired neighbour each side of an insertion run,
read at the page's own paired expansion,
refused passages the pages needed.
hakureico's last line
(73 source code points,
wanting 172 at the page's 2.36)
was refused on a neighbour 44 code points over its expectation;
xiept2 lost 8 of 25 interior passages the same way;
shi_Yumiaoya's slice 4 and noname's opening heading likewise.
One paragraph's expansion is noisy by more than a small passage's size,
so a neighbour-sized window cannot read a merge at that scale.
The whole-page budget averages that noise away,
which is why the decision of 2026-08-23 measured clean on 92 pairs;
its blind spot is the page whose archive stops early and runs long.

The tail is the one region the pairing makes exact.
`readUntranslatedTail` (`coverage-tail.ts`) takes every source-only slice after the last agreed pair,
the page's own expansion over its paired slices
(the corpus median when nothing was paired),
and the tail's expectation as its source at that expansion.
The second signature stays model-free,
as the decision requires:
the pairing is a roster's work,
so "nothing after the last pair" is not the deterministic reading;
size is.
A tail whose expectation exceeds the last agreed pair's whole rendering cannot have been merged into it,
the one place a tail could hide,
and is admitted on that bound
(`insertion-corroboration (slice N, tail admitted, ...)`).
A smaller tail stays with the whole-page budget,
where it was.
The interior's whole-page budget leaves an admitted tail's source out,
so the tail's absence is not counted twice.
The roster-free `admitInsertions` path,
which has no coverage verdict as a first signal,
is unchanged.
On XingZ60 the tail is slices 89 to 119,
about 5,100 source code points at the page's own 2.9,
expected about 14,900 against slice 88's rendering of a few hundred:
admitted whole,
where the whole-page budget had 10,794 for everything.

XingZ609 (frozen `06b395ac7`) was killed at 03:24 UTC under always-kill-and-relaunch,
35 minutes in,
118 repair chunks done,
the translate lane not entered.
Its admission had confirmed class fifty-nine live:
`insertion-definition-admitted (slice 117 defines 4, referenced by the page ...)`
and `(slice 118 defines 9, referenced by slice 94 ...)`,
with slice 112 still unresolved on the budget,
which the tail rule now covers.
Spend by the meters:
Bedrock 154.76 to 152.96,
OpenRouter 222.07 to 220.62,
3.25 USD.

Frozen `06b395ac7` (classes forty-one to fifty-nine,
the deepseek route),
pid 775640,
runs `~/temp/agent/XingZ609`,
log `~/temp/agent/XingZ609.log`,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8` on the launch line,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
no cron job,
the background waiter alone reports the exit.
Read first the admission's `insertion-definition-admitted` lines for the definition slices 112 to 119
(each names the label and the referencing slice),
then the translate lane's assembly
(no `assembly-footnote-reverted unresolved-reference`,
no `assembly-withdrew-every-replacement`),
then the bilibili link in the poem attribution on the page,
then the nine definitions on the page under the original's labels
with the orphan trim's findings for any definition no marker on the page references,
then the run time against XingZ608's 2h16m,
then the seven steps and the three checks.

## Class fifty-nine, 2026-09-19, 02:45 UTC: the shortfall budget refuses the definitions the shipped markers reference

Found by the XingZ608 read.
The whole-page shortfall budget
(`coverage-corroboration.ts`;
expected English at the corpus median of 2.65 code points per source code point,
less what the page carries)
is spent over the absent-voted passages in document order,
and a page's footnote definitions stand last.
XingZ60's budget is 10,794 code points
(16,650 source code points against 33,329 on the archive page);
the passages at slices 89 to 111,
4,043 source code points,
spent 10,714 of it,
and none of the eight definition slices
(112 to 119,
63 to 192 source code points each)
fitted the 80 that remained.
Every one was refused,
"the page has no room to be missing it",
while the translate lane wrote slices carrying `[^3]`,
`[^7]`,
`[^8]` and `[^9]`.
The lane's assembly then reverted every carrier of a marker with no definition
(`assembly-footnote-reverted unresolved-reference gfm 3`,
`7`,
`8`,
`9`),
slice 95 among them,
the poem attribution holding the page's only rendering of the bilibili link,
and the entry stopped INCOMPLETE at publish on the dropped destination.
XingZ605 refused the same tail
(112 to 117);
XingZ607 attempt 1 refused slices 99 and 102 instead,
1,953 code points that left the tail room.

Fixed in `06b395ac7`
(guards red first at `db5f0124e`):
`admitReferencedDefinitions` (`insertion-referenced-definitions.ts`) runs after the container halves
and admits every unresolved definition whose label the standing page or an admitted slice's source references
and nothing on the shipping page defines,
finding `insertion-definition-admitted (slice N defines L, referenced by slice M ...)`.
A marker the page will carry with no definition under its label is the page's own graph saying the definition is absent,
the same kind of independent signal as a missing destination,
which the admission already reads past the budget.
A definition nothing references stays with the budget,
and a definition the page already carries under that label is left alone.
Definitions a slice brings in beside the referenced one that no marker on the page references
are trimmed by class twenty-one's orphan guard at assembly.

An open question stands beside the fix,
recorded in the handover's "The shortfall budget question":
the budget under-reads a page whose archive stops early and runs long.
XingZ60's archive ends at "Under Construction" with the source's second half untranslated,
and its translated half runs at about 3.3 English code points per source code point against the corpus median of 2.65,
so the whole-page shortfall reads 10,794 where the untranslated half alone
(slices 89 to 119,
about 5,100 source code points)
expects about 13,600.
The budget refused nothing on XingZ608 that class fifty-nine does not now admit,
but a longer untranslated tail would run out of budget on prose,
which no structural rule admits.

## XingZ608 read, 2026-09-19, 02:35 UTC: INCOMPLETE at 2h16m at publish, the run time verified

Frozen `eaaf39ec3`,
2 hours and 16 minutes
(`ms=8153159`),
`status=INCOMPLETE`,
`error=entry XingZ60 would drop 1 source destination(s)`,
one attempt,
no page written.
Spend by the meters:
Bedrock 158.98 to 154.76,
OpenRouter 227.93 to 222.07,
10.08 USD;
`pass-spend.mjs` logs 7,958 seats and 284 retry rounds.

The run time first.
Preparation 00:16 UTC,
pictures 00:24 UTC,
repair lane 00:33 UTC,
translate lane 01:10 UTC,
lane contest 01:41 UTC,
consolidation 01:45 UTC,
publish 02:31 UTC:
the whole pipeline in 2h15m where XingZ607 attempt 1 reached consolidation at 5h05m and the deadline at 7h.
deepseek-v4.1-flash was served by Morph 680 times,
Krea 291 and Makora 4,
never DeepInfra or Wafer,
at a median of 11.1 s,
11.0 s and 17.9 s;
no HTTP 429 at eight in flight.
The endpoint route and the overlap of 8 are kept.

Then the stop.
The admission admitted slices 89 to 111 and refused 112 to 119,
the footnote definitions,
on the whole-page shortfall budget
(class fifty-nine,
the heading dated 2026-09-19 02:45 UTC);
no `insertion-container-half-admitted` line was needed
(slice 99 was admitted on its own split verdict with no anchored claim,
slice 102 on an absent one),
and no half shipped alone.
The translate lane wrote 112 slices and its assembly reverted the four marker carriers,
nothing else:
class fifty-eight's advancing withdrawal was not exercised
(no structural break),
and no blanket withdrawal fired.
Jev's select ballots:
agreed with the round winner 87 of 106 in repair,
44 of 65 in translate,
31 of 39 in consolidation.
Two consolidation slices shipped the archive's wording without contest endorsement
(slices 53 and 66,
`final-selection-unendorsed`),
and slice 14's standing was withheld from the slate for the untranslated pronoun and a valid proposal shipped
(class fifty-four's path).
The page's footnotes,
components,
summary and profile link are read on XingZ609.

Frozen `eaaf39ec3`,
pid 725364,
runs `~/temp/agent/XingZ608`,
log `~/temp/agent/XingZ608.log`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
`TRANSLATION_REPAIR_SLICE_OVERLAP=8` on the launch line
(the pass-overlap decision's variable,
which overrides the four in either direction;
XingZ607 met 5 HTTP 429 refusals in 6,289 calls at four and no Bedrock throttle),
no cron job,
the background waiter alone reports the exit.
Read first the run time against XingZ607's 8h41m
(one attempt,
under the seven-hour deadline,
the phase boundaries),
the endpoints serving deepseek-v4.1-flash
(`served by "Morph"`,
never DeepInfra or Wafer)
and the 429 count at eight,
then the admission lines for slices 99 and 102
(`insertion-container-half-admitted`),
the translate lane's assembly
(no `assembly-withdrew-every-replacement`),
the five archive-absent disclosure blocks on the page with the bilibili link,
then the seven steps and the three checks.

## deepseek-v4.1-flash routed off DeepInfra and Wafer, 2026-09-19, 00:05 UTC: the endpoint was the slow seat

Measured on XingZ607 for the run-time work
(the task the seven-hour deadline opened).
Every judge seat but one answers under 14 s at the ninetieth percentile
(minimax 4 s,
gpt-oss 14,
gemma-26b 3,
mercury 8,
jev 0);
deepseek-v4.1-flash answered at 30 s median and 129 s at the ninetieth,
set the quorum time in 553 of the run's 2,320 rounds
(689 of the 1,173 minutes of quorum wait,
249 of them in select rounds,
165 in lane contests,
70 at the consolidate gates),
held 15 rounds to the full 360 s timeout,
and was abandoned 93 times after the 120 s grace.
The completion cap held it:
its longest completed reasoning was 56 thousand characters and every cut stream sat at the 13,082-token pooled ceiling
(the "1.4 million characters" of the attempt 1 heading were raw stream bytes,
not reasoning).
By endpoint the picture splits:
Morph served 262 calls at 11 s median and 25 s at the ninetieth,
DeepInfra 701 at 45 s and 158 s with 89 of the 97 cut streams,
Wafer 27 at 26 s and 122 s with the other 8.
XingZ605,
served by Morph alone,
was the fastest XingZ60 run at 4h08m;
XingZ604 and XingZ607 ran on DeepInfra.
Fixed in `ac93f7978` by the class twenty-four route:
`ignoredEndpoints: ['deepinfra', 'wafer']` on the card,
slugs from the gateway's providers listing of 2026-09-18,
the catalog and client tests updated
(`4cd9bf5c1`).
The other levers stay measured and unpulled:
the refiners' 37 minutes of grace after quorum were deepseek's stragglers,
and the lane width moves by the launch variable.

## Class fifty-eight, 2026-09-19, 00:10 UTC: two structural breaks defeat the single-withdrawal proof and empty the lane

The second half of the XingZ607 withdrawal.
The assembly guard's counterfactual withdraws one replacement at a time and keeps the first that leaves the page whole;
a page with two breaks fails every single withdrawal and the guard withdrew all 88 replacements.
Fixed in `eaaf39ec3`
(guards red first at `35b2125d2`):
`MdxParseError` carries the line and column the parser named,
`strict-refusal-offset.ts` reads them as an offset into the whole text,
and `advancingStructuralWithdrawal` withdraws the replacement that moves the first refusal furthest later
(`assembly-structure-advancing-withdrawal slice N: reverting this replacement moves the first strict-parse refusal from offset A to B`),
so the next round reads what is left,
one break at a time.
The blanket withdrawal remains for a defect no withdrawal moves.
