# OpenRouter and the corpus passes: 2026-09-18

What was launched,
read and decided on 2026-09-18.
Entries:
XingZ607,
XingZ606,
XingZ605,
XingZ604,
XingZ603,
XingZ602.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## Class fifty-seven, 2026-09-18, 23:55 UTC: a container half ships without its partner and the translate lane withdraws every slice

Found by the XingZ607 read.
The archive carries six of the original's eleven disclosure blocks.
Two of the five it never carried
(the vampire-poem block at source line 452 and 锦心's block at line 509)
had their opening halves,
the `<details>` tag with its `<summary>`,
left unfilled at the insertion admission:
`slice 99: coverage=split` and `slice 102: coverage=split`,
`placement unresolved after the single round (verdict split); not admitted`,
one voice having anchored the poem's title in the archive's other summary.
Their bodies and closing halves were admitted `absent`,
translated,
and assembled:
the lane's page carried two `</details>` with no opening,
the strict grammar refused it at the first
(`mdast-util-mdx-jsx/unexpected-closing-slash`,
reproduced offline from the slice cache with the frozen parser),
the guard's one-slice counterfactual could not repair a page with two,
and it withdrew all 88 replacements:
`assembly-structure-unattributable mdx-downgraded`,
`assembly-withdrew-every-replacement (88 slices)`,
then `translate changed 0/120 slices`.
The same withdrawal stands in the XingZ602,
XingZ604 and XingZ605 logs,
masked by the earlier stops;
XingZ606 downgraded twice and blamed a slice each time.
Fixed in `6036e6618`
(guards red first at `55cd8f7e7`,
the pair reader's own cases at `caac6e3a0`):
`container-half-pairs.ts` reads the source of every slice for lone container tags and pairs each opening half with its closing half;
`insertion-container-halves.ts` admits the unadmitted half of any container one of whose slices was admitted on its own evidence
(`insertion-container-half-admitted (slice 99 beside slice 100: one container's halves ship together, and slice 100 is admitted)`);
`assembly-container-halves.ts` withholds,
at the translate lane's assembly and at the composed page's,
a half whose partner ships nothing
(`assembly-container-half-withheld (slice N beside slice M: ..., and slice M ships nothing)`).
Measured over XingZ607's own slice records:
with the two lone closing halves withheld the lane's page parses as MDX.
Recorded as an addendum to the absence-verdict decision.

## XingZ607 attempt 2 read, 2026-09-18, 23:20 UTC: INCOMPLETE at 1h41m, one source destination dropped, the translate lane empty

Frozen `139a7f99a`,
attempt 2 from 21:39 to 23:20 UTC
(`ms=6064867`),
`status=INCOMPLETE`,
`error=entry XingZ60 would drop 1 source destination(s)`,
no page written,
`attempts.json` reads 2.
Spend by the meters over both attempts:
Bedrock 163.38 to 158.98,
OpenRouter 235.95 to 227.93,
12.42 USD.
The dropped destination is the bilibili link in the poem attribution
(source line 435,
slice 95),
which only the translate lane rendered
(gemma-26b's candidate,
`[Bird in a Cage](https://www.bilibili.com/video/BV17Q4y1s7j1)`);
the lane had withdrawn every replacement at its assembly
(class fifty-seven),
so the page carried none of the five archive-absent disclosure blocks and the destination floor was the first floor to say so.
Slice 113
(a paragraph inside 盖棺定论's block,
which the archive does carry)
also split and stays a recorded gap;
it opens no container.
Classes fifty-four to fifty-six held
(the attempt 1 heading);
the page-level checks
(the nine definitions,
the components,
the `details` summary,
the profile link)
wait for a page.
The 1h41m of attempt 2 were consolidation
(slices 70 to 84 bought,
the rest replayed),
so the entry's whole work was 8h41m and two attempts.

## XingZ607 attempt 1, 2026-09-18, 21:39 UTC: the seven-hour entry deadline in consolidation, both pronoun paragraphs shipped

Attempt 1 ran the whole seven hours
(`TALLY XingZ60 status=ERROR ms=25200004 aborted=true error=refused by StreamCutShortError`,
the per-entry deadline in `pass-entry.ts`)
and the attempt queue re-queued the entry at once
(`REATTEMPT XingZ60 queued: 444 more cache records than it had`,
`attempts.json` reads 2);
attempt 2 replays the cached slices under the same pid and was into the lane contests within four minutes.
Classes fifty-four to fifty-six held:
the funeral paragraph
(slice 13,
standing withheld at 20:13 UTC)
went through its gate at 20:16 UTC told the standing cannot ship,
three ballots for the consolidation
(GPT-OSS,
minimax,
gemma-26b,
each naming the untranslated TA)
against gemma-e2b's "correctly preserves the pronoun 'TA' as dictated by the corpus rules",
`settled on consolidated, ships consolidated`;
the teachers paragraph
(slice 33,
withheld at 20:44 UTC)
the same at 20:47 UTC,
three to one with gemma-e2b again the one,
`exit=computed` at 20:52 UTC.
No slice stopped.
Where the seven hours went,
phase by phase:
pictures and preparation 14:40 to 14:55 UTC,
repair lane 14:55 to 17:50 UTC,
translate lane 17:50 UTC to about 19:20 UTC,
lane contests to 20:00 UTC,
consolidation 20:00 UTC to the deadline at 21:39 UTC with 56 of the slices started and chunk 69 the furthest.
Slice time by lane
(`SLICE-COST`,
concurrent):
repair 2.0 minutes a slice over 240,
translate 1.5 over 232,
consolidation 7.1 over 56.
Inside the consolidation phase the select rounds took 147 minutes of wall time
(116 to quorum),
the refiners 68
(37 of it grace),
the naturalness review 50,
the gates 33 and 25.
deepseek-v4.1-flash is the slow seat:
93 of its 845 calls abandoned,
13 calls at the 360 s timeout
(6 select,
5 lane-contest,
2 consolidate-gate),
15 streams cut between 450 thousand and 1.4 million characters,
2.52 USD of the 9.52 logged,
the most of any seat.
GLM-5.3-Flash on Synthetic ran one coverage stream to 10.5 million characters.
Spend by the meters at the deadline:
Bedrock 163.38 to 160.08,
OpenRouter 235.95 to 230.05,
9.20 USD;
`pass-spend.mjs` logs 7594 seats and 227 retry rounds.
The page read waits on attempt 2.

## XingZ607 launches on class fifty-six, 2026-09-18, 14:40 UTC

Frozen `139a7f99a` (classes forty-one to fifty-six plus the decision seat),
pid 571308,
runs `~/temp/agent/XingZ607`,
log `~/temp/agent/XingZ607.log`,
providers at launch `synthetic=dry bedrock=wet hyper=dry openrouter=wet`,
meters Bedrock 163.38 and OpenRouter 235.95,
no cron job,
the background waiter alone reports the exit.
It resumes from XingZ606's slice cache.
Read first the consolidation of the teachers paragraph
(the slice whose standing carries the untranslated pronoun;
its gate sheet now says the standing cannot ship,
and every gate ballot is logged with its reason:
`consolidate gate ballot <model>: <choice>: <reason>`),
then any other `CANNOT SHIP` gate and how it went,
then the page's teachers and funeral paragraphs
(no TA left standing),
then the nine footnote definitions and their labels,
the components,
the `details` summary,
the profile link,
then the seven steps and the three checks.

## Class fifty-six, 2026-09-18, 14:36 UTC: the gate keeps an ineligible standing it was never told cannot ship, and logs no reasons

Found by the XingZ606 read.
A third paragraph of the entry,
the one on the parents being teachers,
carries the untranslated pronoun in the archive and in the contest winner,
so its consolidation ran with the standing withheld:
the slate chose GPT-OSS's valid proposal 2 to 1.5
(the challenge of class fifty-five was not needed),
and the consolidate gate went 4 of 5 usable for the standing.
A gate refusing at quorum stops the slice under the 2026-09-04 rule,
and the log carried only the tally,
so why four judges preferred a text the rule refuses could not be read afterwards.
The gate sheet showed the judges the standing as a rendering to keep,
with nothing saying it fails the deterministic publication rule or that keeping it stops the entry,
and the comparison they were asked for was against a text that was never going to ship.
Fixed in `139a7f99a`
(guards red first at `90e5e2a19`):
`readStandingVerdict` returns the refusal it already logged
(`standingRefusal`),
the driver,
the buy and the settlement carry it to the gate subject,
and the gate sheet says
`CANDIDATE "standing" CANNOT SHIP: the deterministic publication rule refused it (...)`,
that choosing it stops the entry with no page,
and that a rendering the rule refuses is not the safer choice.
Every gate ballot is now logged with its model,
choice and reason.
A gate that still refuses at quorum still stops the slice;
whether the sheet changes the ballots is what XingZ607 measures.

## XingZ606 read, 2026-09-18, 14:30 UTC: INCOMPLETE at 3h04m on slice 33, the gate keeping the teachers paragraph's standing

Frozen `adca69d4e`,
3 hours and 4 minutes
(`ms=11050301`),
`status=INCOMPLETE`,
`error=slice 33: the standing text failed the deterministic publication rule and the consolidation left nothing valid to ship (gate-kept-standing)`,
no page written,
`attempts.json` reads 1.
Spend by the meters:
Bedrock 166.41 to 163.38,
OpenRouter 239.86 to 235.97,
6.92 USD;
`pass-spend.mjs` logs 6446 seats and 155 retry rounds
(select 411 rounds,
1971 asked,
1737 heard;
translate 168 rounds,
676 asked,
497 heard).
Phases:
preparation 11:18 UTC,
pictures 11:25 UTC,
lanes 11:31 UTC,
translate lane 13:05 UTC,
lane contest 13:56 UTC,
consolidation 14:03 UTC,
stop 14:22 UTC;
Synthetic and Hyper dry throughout
(`wide=7 select=8`),
the fastest of the three XingZ60 consolidations to be reached.
The stop is class fifty-six
(the heading dated 2026-09-18 14:36 UTC).
The funeral paragraph raised no warning in this consolidation,
so classes fifty-four and fifty-five were not exercised on it;
the gate over the whole consolidation went
6 consolidated,
4 neither and 4 standing across 14 slices.

What held:
`image0.webp` corroborated by 3 of 3 readers at overlap 0.590;
eleven `declined-indecision` challenges in the translate lane,
none a run-off.
Jev live:
`SEAT typesafe/jev-1.13 asked=224 usable=223 unusable=0 threw=1`
(one HTTP 520 from OpenRouter at 14:13 UTC,
the voice lost),
agreeing with the round's winner on 100 of 119 repair select rounds,
56 of 70 translate slates and 14 of 18 consolidation slates.
No page,
so the definitions,
components,
`details` summary and profile link could not be read.

## XingZ606 launches on class fifty-five, 2026-09-18, 11:20 UTC

Frozen `adca69d4e` (classes forty-one to fifty-five plus the decision seat),
pid 512931,
runs `~/temp/agent/XingZ606`,
log `~/temp/agent/XingZ606.log`,
providers at launch `synthetic=dry bedrock=wet hyper=dry openrouter=wet`,
meters Bedrock 166.41 and OpenRouter 239.86,
no cron job,
the background waiter alone reports the exit.
It resumes from XingZ605's slice cache
(slice 13's consolidation was not cached,
its settlement threw).
Read first slice 13's consolidation
(a `translate-declined-retried` finding with a run-off,
or a slate at quorum on the first ask,
then a gate at quorum or `undecided-gate-ships-proposal`,
never a stop),
then the page's rendering of the funeral paragraph
(no TA left standing),
then the nine footnote definitions and their labels,
the components,
the `details` summary,
the profile link,
then the seven steps and the three checks.

## Class fifty-five, 2026-09-18, 11:15 UTC: a tied consolidation slate over an ineligible standing declines without the challenge round

Found by the XingZ605 read.
Slice 13 again:
the standing withheld as ineligible
(the translate lane won the contest 4 of 5 with the untranslated pronoun still in its text),
the slate offered three writers' proposals and the repair lane's text,
all four valid,
and the judges split 2 to 2
(DeepSeek and Mercury for one "her" rendering,
Gemma E2B and GPT-OSS for the repair lane's).
`judgeTranslateSlate` declined `declined-indecision`,
the consolidation named it `slate-declined-standing`,
and the entry stopped INCOMPLETE at 4h08m,
before class fifty-four's gate path was ever reached.
The translate lane has re-asked a declined slate once under `decline-challenge` since class fifty-three,
narrowing a tie to the candidates that drew a ballot;
the consolidation called the judge directly and had no second ask.
Fixed in `adca69d4e`
(guard red first at `c6ec06788`):
`settleConsolidation` routes the slate through `judgeSlateWithRetry` when the standing is ineligible,
so a tie or rejection is challenged once with the run-off before the slice is given up;
an eligible standing keeps the single round,
since there a decline keeps text the contest endorsed.
The guard's fixture now takes a scripted judge reply
(a function over the sheet and the call count)
so one round can tie and the challenge round can settle.

## XingZ605 read, 2026-09-18, 11:12 UTC: INCOMPLETE at 4h08m on slice 13, the slate tied

Frozen `0d0747200`,
4 hours and 8 minutes
(`ms=14889531`),
`status=INCOMPLETE`,
`error=slice 13: the standing text failed the deterministic publication rule and the consolidation left nothing valid to ship (slate-declined-standing)`,
no page written,
`attempts.json` reads 1.
Spend by the meters:
Bedrock 169.57 to 166.42,
OpenRouter 242.13 to 239.87,
5.41 USD;
`pass-spend.mjs` logs 6220 seats and 108 retry rounds
(select 412 rounds,
2001 asked,
1889 heard;
translate 179 rounds,
706 asked,
511 heard).
Phases:
preparation 07:01 UTC,
pictures 07:05 UTC,
lanes 07:13 UTC,
translate lane 09:47 UTC,
lane contest 10:50 UTC,
consolidation 10:55 UTC,
stop 11:08 UTC;
Synthetic dry throughout,
Hyper wet at launch and dry by the consolidation
(`wide=7 select=8`).
The stop is class fifty-five
(the heading dated 2026-09-18 11:15 UTC);
class fifty-four's gate path was never reached.

What held:
`image0.webp` corroborated by 4 of 4 readers at overlap 0.603;
no `translate-runoff` finding
(class fifty-three still untested live).
Jev live:
`SEAT typesafe/jev-1.13 asked=254 usable=254 unusable=0 threw=0`,
245 ballots logged,
agreeing with the round's winner on 117 of 156 repair select rounds,
46 of 62 translate slates and 3 of 4 consolidation slates
(`jev-agree.mjs`).
The nine footnote definitions,
the components,
the `details` summary and the profile link could not be read:
no page.
Elsewhere in the consolidation a slate over an eligible standing
(the rescue-work paragraph)
drew only weight 1.5 across 5 ballots with MiniMax declining all four candidates over the declared name of a linked person,
and kept its standing as designed.

## XingZ605 launches on class fifty-four, 2026-09-18, 07:03 UTC

Frozen `0d0747200` (classes forty-one to fifty-four plus the decision seat),
pid 474604,
runs `~/temp/agent/XingZ605`,
log `~/temp/agent/XingZ605.log`,
providers at launch `synthetic=dry bedrock=wet hyper=wet openrouter=wet`,
meters Bedrock 169.57 and OpenRouter 242.13,
no cron job,
the background waiter alone reports the exit.
It resumes from XingZ604's slice cache
(slice 13's consolidation was not cached,
its settlement threw).
Read first slice 13's consolidation
(the `undecided-gate-ships-proposal` finding,
or a gate at quorum either way,
never a stop),
then the page's rendering of the funeral paragraph
(singular they or the name where the original writes TA,
no TA left standing),
then the nine footnote definitions and their labels,
the components,
the `details` summary,
the profile link,
then the seven steps and the three checks.

## Class fifty-four, 2026-09-18, 06:58 UTC: the consolidate gate's indecision keeps an ineligible standing over the proposal the slate chose

Found by the XingZ604 read.
The archive's funeral paragraph carries the original's neutral pronoun untranslated
("in the eyes of TA's parents"),
which the deterministic publication rule refuses,
and the lane contest settled 5 of 7 on the translate lane's text,
which carries it too;
so the consolidation's standing was ineligible and,
under the 2026-09-09 addendum,
the incumbent could not stand in.
The slate offered the writers' proposals and the repair lane's text
(which had rendered TA as "their"),
the slate judges chose GPT-OSS's proposal 3 of 4
("her parents",
"gender expression"),
and the consolidate gate went 5 of 7 usable with neither rendering at its quorum of two over the other.
The gate's rule that indecision keeps the standing text fell back to the ineligible standing,
`requireShippableTerminal` refused `gate-kept-standing`,
and the entry stopped INCOMPLETE at 4h53m over a text the judges had already endorsed.
Fixed in `0d0747200`
(guard red first at `b8b32e28d`):
`shipPastUndecidedGate` (`consolidate-ineligible-standing.ts`) resolves a `neither` verdict over an ineligible standing toward the consolidation,
recording `undecided-gate-ships-proposal`,
under the owner's 2026-09-04 rule that the best valid proposal is preferred.
A gate that refuses the consolidation at quorum still keeps the standing and still stops the slice;
an eligible standing keeps the slice on the same indecision as before.
Addendum in `doc/decision/translation-repair-ineligible-standing.md`.

## XingZ604 read, 2026-09-18, 06:55 UTC: INCOMPLETE at 4h53m on slice 13, the first pass with a typed-decision judge

Frozen `c14c7f176`,
4 hours and 53 minutes
(`ms=17589011`),
`status=INCOMPLETE`,
`error=slice 13: the standing text failed the deterministic publication rule and the consolidation left nothing valid to ship (gate-kept-standing)`,
no page written,
`attempts.json` reads 1.
Spend by the meters:
Bedrock 172.62 to 169.57,
OpenRouter 247.06 to 242.14,
7.97 USD;
`pass-spend.mjs` logs 6655 seats and 401 retry rounds
(select 480 rounds,
2266 asked,
1854 heard;
translate 189 rounds,
731 asked,
489 heard).
Phases:
preparation 01:56 UTC,
lanes 02:10 UTC,
translate lane 05:19 UTC,
lane contest 06:33 UTC,
consolidation 06:38 UTC,
stop 06:47 UTC;
Synthetic went dry during the lanes and the contest and consolidation ran `wide=7 select=8`.
The stop is class fifty-four
(the heading dated 2026-09-18 06:58 UTC).

What held:
`image0.webp` corroborated by 5 of 6 readers at overlap 0.870
(class fifty-two live on a wider bench);
slice 14 translated on its first rounds
(GLM-5.3-Flash's candidates at weight 3 and 5 of 6,
one `declined-indecision` challenge elsewhere in the lane),
so class fifty-three's run-off was never needed and stays untested live;
no `translate-runoff` finding anywhere.

Jev live:
`SEAT typesafe/jev-1.13 asked=221 usable=221 unusable=0 threw=0`,
217 ballots logged
(130 repair-lane select rounds,
82 translate-lane slates,
5 consolidation slates),
every call on the decisions route.
Measured against the round's winner
(`jev-agree.mjs` over the log):
repair select 110 of 124 agree,
translate slate 53 of 76,
consolidation 4 of 5.
Its probabilities ranged from split
(0.49 against 0.41 on its first ballot,
confidence 0.24)
to decisive
(0.98,
confidence 0.97);
the log line carries them.
The nine footnote definitions,
the components,
the `details` summary and the profile link could not be read:
no page.

## XingZ604 launches on the Jev seat, 2026-09-18, 01:56 UTC

Frozen `c14c7f176` (classes forty-one to fifty-three plus the decision seat),
pid 211894,
runs `~/temp/agent/XingZ604`,
log `~/temp/agent/XingZ604.log`,
Synthetic,
Bedrock and OpenRouter wet,
Hyper dry at launch,
meters Bedrock 172.63 and OpenRouter 247.07,
no cron job,
the background waiter alone reports the exit.
The JUDGE SEATS line at 01:56 UTC reads `wide=8 select=9`:
the ninth select seat is `typesafe/jev-1.13`,
the first pass with a typed-decision judge on the select benches.
It resumes from XingZ602's slice cache.
Read first slice 14's translate rounds
(a winner,
or a `translate-runoff (finalists N of M)` finding,
never an ERROR),
then Jev's ballots in the select rounds
(reason prefix `typed decision`,
the probabilities on the line),
then the pictures line,
then the nine footnote definitions and their labels,
the components,
the `details` summary,
the profile link,
then the seven steps and the three checks.

## XingZ603 killed, 2026-09-18, 01:56 UTC: 60 minutes into the repair lane, the seat landed

Frozen `012b9ccc3`,
killed by pid at 01:56 UTC under always-kill-and-relaunch once `c14c7f176` seated Jev.
It had reached repair chunk 18 of the repair lane
(chunks 14 to 18 repaired,
chunk 10 unchanged)
and never entered the translate lane,
so class fifty-three is still untested live.
Spend by the meters:
Bedrock 173.08 to 172.63,
OpenRouter 247.86 to 247.07,
1.24 USD;
`pass-spend.mjs` logs 1238 seats and 28 retry rounds
(select 40 rounds,
196 asked,
179 heard).

## Jev 1.13 seated as a select judge, 2026-09-18, 01:47 UTC: 12 of 12 through the production probe

`ad20acc84` put `typesafe/jev-1.13` on the roster as a decision-only seat
(a `decisions` side on the card,
no chat catalog,
held `judge-unmeasured`,
`writer-unmeasured`,
`reader-unmeasured`),
`bc6fdc9b9` built the transport
(`decisions-client.ts` posting `{model, state, questions}` to the decisions endpoint through the retry policy,
`decision-contract.ts` reading the typed answers,
`stage-decision-call.ts` turning one into a stage voice,
`candidate-select-decision.ts` asking the select question as one `choice` over the candidate numbers with `0` for none)
and threaded it from `decideBestCandidate` through `gatherStageVoices` to `attemptStageCall`,
which routes a decision seat to `client.decide` instead of `chatJson`.
The router refuses `decide` while OpenRouter is dry and marks its refusals on the budgets.

The production probe ran at 01:40 UTC
(`mise run judge-fidelity-probe -- --candidates typesafe/jev-1.13`,
runs `~/temp/agent/jev-probe`,
log `~/temp/agent/jev-probe.log`)
over the 16 reviewed rows of `gqt` and `MTF_0615`
(the probe itself warned the matrix partial,
16 of 28,
the `Y1Ran` reference not on disk here),
Jev beside the ten seated judges.
Every one of Jev's 12 calls went through `provider=openrouter model=typesafe/jev-1.13` on the decisions route
(1173 to 1211 prompt tokens,
38 output tokens,
338 to 685 ms,
0.000049 to 0.000051 USD each),
the fan-out window spared it on the four `gqt` insertion rounds.
It chose the reviewed reference 12 of 12 times,
no damaged pick,
no decline,
probabilities 0.75 to 0.96 on the reference
(least sure on gqt's deletion rows at 0.75 to 0.81,
as the scratch measurement had found).
Peers on the same rows:
DeepSeek V4.1 Flash 12 of 12,
Mercury 12 of 12,
Qwen 10 of 10,
GLM-5.3-Flash 10 of 10,
Kimi,
GPT-OSS and MiniMax 8 of 8,
Gemma 26B 8 clean and 2 declined of 10,
Gemma E2B 8 clean and 2 damaged of 10;
glm-5.3 unreachable throughout
(Hyper dry).
Jev meets the admission bar
(clean count at the peer median,
damaged count below the peer maximum),
so `c14c7f176` removes `judge-unmeasured` from its card
(guards red first at `1e2c18de4`)
and `RUN_DECISION_JUDGES` seats it after every wide seat on `RUN_SELECT_JUDGES`,
seated when OpenRouter is wet and unseated when it is dry.
Only the candidate-select stage has a typed question for it;
consolidation slates,
lane contests,
panels and gates still call chat,
so a decision seat there is a lost voice,
never a vote.
Writer and reader holds stay:
a typed decision writes and reads nothing.
Runbook section "Add a decision-only model" in `882e4f8c2`.

## XingZ603 launches on class fifty-three, 2026-09-18, 00:55 UTC

Frozen `012b9ccc3` (classes forty-one to fifty-three),
pid 4181820,
runs `~/temp/agent/XingZ603`,
log `~/temp/agent/XingZ603.log`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch,
meters Bedrock 173.08 and OpenRouter 247.86,
no cron job,
the background waiter alone reports the exit.
It resumes from XingZ602's slice cache.
Read first slice 14's translate rounds
(a winner,
or a `translate-runoff (finalists N of M)` finding,
never an ERROR),
then the pictures line,
then the nine footnote definitions and their labels,
the components,
the `details` summary,
the profile link,
then the seven steps and the three checks.

## Jev 1.13 approved on OpenRouter, 2026-09-18, 00:29 UTC: a typed-decision model, probed live

The owner:
"<https://openrouter.ai/typesafe/jev-1.13> is now up and approved on openrouter."
Jev is TypeSafe's System One model:
text in,
typed answers out
(a choice with a probability per option,
a yes-probability the docs call a noul,
a score over ordered levels),
no prose and no reason.
It is served through `POST https://openrouter.ai/api/alpha/decisions`
with `{model, state, questions}`,
each question `{type: 'choice' | 'noul' | 'score', instructions, criteria}`,
answering `{answers, usage: {input_tokens, output_tokens, cost}, model, provider}`
(OpenRouter's OpenAPI document,
schemas `DecisionsRequest` and `DecisionsResponse`;
TypeSafe's models page:
32k tokens of state plus the longest question,
0.042 USD per million input tokens,
output free).
A cat-themed live probe at 00:32 UTC answered in 1216 ms for 0.000023 USD:
the faithful candidate chosen at probability 1,
the detail-dropping candidate flagged at 0.98,
the meaning-changing candidate scored "changes the meaning" at 0.99.
It cannot take a chat completion,
so it fits only the seats whose ballot is a choice
(the select and slate judges,
the lane contest,
the panel's votes,
the polish gate,
the attestation confirm round),
through a decisions transport beside the chat clients
and a per-stage question the stage builds from the same sheet;
the judge fidelity probe seats it or not,
as for every candidate.
Open as the work of seating `typesafe/jev-1.13` in the vote-shaped seats by measured fidelity;
nothing is wired yet.

Measured at 01:05 UTC on the source-reviewed fidelity matrix
(`reviewedFidelityTrials` over the three reviewed references,
28 rows:
deletion and insertion in both directions and both positions,
alteration on the one reference that reviews it),
asked as one choice per row with the ungoverned translate criteria in the state
(the same task and criteria `runFidelityTrial` puts on the sheet,
a scratch script outside the package):
28 of 28 chose the reviewed reference,
no decline,
probabilities from 0.83 to 0.99 on the clean side,
mean 600 ms per row,
0.00137 USD for the matrix.
The one bench where it read least surely was gqt's deletion pair
(0.83 to 0.89 for the clean text,
0.09 to 0.12 for declining).
That is the bar DeepSeek V4.1 Flash and Mercury met on admission,
so the transport and the select-seat adapter are worth building;
the production probe re-measures it through the selector once wired.

## Class fifty-three, 2026-09-18, 00:45 UTC: a tied slate with nothing to fall back on is challenged over the same slate and stops the entry

Found by the XingZ602 read.
Slice 14 of XingZ60 is a content slice whose archive text carries the pronoun untranslated
("TA once said"),
which `validateTranslatedSlice` refuses
(measured with the frozen build on 2026-09-18),
so the translate stage ran it with `effectiveIncumbentKind` absent.
Five candidates went to the judges,
four of five were heard,
three named three different candidates at weight one against a minimum of two,
and the decline challenge put the identical slate to the same panel,
which split the same way.
The retry settled the slate as `no-candidate-backed` and raised the absence error;
the backstop in `translate-slice-attempt.ts` refuses to record a content chunk as unfilled and rethrew it;
the entry stopped `status=ERROR` at `ms=9459888`
(2 hours and 38 minutes)
and the REATTEMPT re-bought the translate lane,
where the same slice found a winner on the second try.
Fixed in `012b9ccc3`
(guard red first at `5ae2e79ad`,
shown red against the frozen `24372215a` build):
the judge records on the absence error which candidates a tie backed
(`runoffFinalists` in `translate-runoff.ts`,
a tie's finalists when fewer than the slate drew a ballot),
and the retry's challenge round offers only those,
naming the run-off in the findings
(`translate-runoff (finalists N of M)`).
A second tie still settles as before;
a rejection,
fewer than two backed candidates or a fully backed slate leave the whole slate on the second sheet.

## XingZ602 read, 2026-09-18, 00:56 UTC: ERROR at 2h38m on slice 14, the reattempt killed at 5h57m in consolidation

Frozen `24372215a`.
Attempt 1 ran from 18:56 to 21:34 UTC and stopped
`status=ERROR ms=9459888 error=slice has no translation in the archive and produced none (no-candidate-backed)`
on slice 14:
class fifty-three.
Before that,
the pictures phase corroborated `image0.webp` by 4 of 4 readers at overlap 0.570,
so class fifty-two held on the picture that stopped XingZ601 at 0.297.
Attempt 2 ran from 21:34 UTC,
resumed 143 cache records,
re-bought slice 14 with a winner,
reached the lane contest at 00:24 UTC and the consolidation at 00:47 UTC,
and was killed by pid at 00:56 UTC under always-kill-and-relaunch once `012b9ccc3` landed,
5 hours and 57 minutes after launch,
three consolidation slices in.
Spend by the meters:
Bedrock 177.43 to 173.08,
OpenRouter 253.32 to 247.91,
9.76 USD;
`pass-spend.mjs` logs 10034 seats and 348 retry rounds
(translate stage alone 173 rounds,
703 asked,
518 heard).
No page was written.
The run time is the open concern:
attempt 2 alone took 3 hours and 22 minutes to reach the consolidation on a 577-line entry,
against the owner's 7-hour ceiling.
