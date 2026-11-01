# OpenRouter and the corpus passes: 2026-09-07, 21:19 to 23:38 UTC

What was launched,
read and decided on 2026-09-07 21:19 to 23:38 UTC.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The owner moves Bedrock ahead of Hyper; the third hakureico launch, 2026-09-07, 21:47 UTC

The owner,
21:41 UTC:
"Yes Bedrock sit ahead."
Recorded as the addendum of 2026-09-07 in `doc/decision/translation-repair-openrouter-fallback.md`.
`PROVIDER_ORDER` reads Synthetic,
Bedrock,
Hyper,
OpenRouter (`a317f4e03`,
with `providerRecord` filling in that order,
the README's routing paragraph,
and the tests that had spelled the walk out:
`providerServing` and `routeProviderFor` now reach Bedrock after Synthetic and Hyper after Bedrock,
`otherProviders` and the hold-wait refusal list the four in the new order).
The second hakureico pass was killed by pid at 21:41 UTC under the kill-and-relaunch rule,
33 minutes and 559 calls in,
in the repair lane at chunk 13;
Hyper read 664 at the relaunch,
so the two passes killed today spent 164 of its credits between them.
The probe flags landed with the order,
rebased onto the day's doc commits and fast-forwarded (`8f47f117b`,
`7de4e7b64`,
`a317f4e03`);
the full suite read 0 `FAIL` on the merged tree,
types and oxlint clean.

`hakureico` relaunched at 21:47:26 UTC on `a317f4e03`,
pipeline `b87b238c`,
plain invocation,
fresh runs dir `~/temp/agent/hakureico3-20260907`,
log beside it,
pid 3764858.
`METERS` at launch:
Synthetic dry at 0 percent of the week,
Bedrock 199.99 USD,
Hyper 664,
OpenRouter 0.01 USD.
`JUDGE SEATS phase=preparation` reads
`wide=6 select=5 late=7 slate=6 checkers=3 translators=6 readers=3 roster=8`,
`withheld=hf:Qwen/Qwen3.8-27B,hf:moonshotai/Kimi-K3`,
the Hyper-slow judges held out because Hyper would serve them with Synthetic dry.
The first pass in which Bedrock takes the two seats it shares with Hyper.
The reading follows:
the footnotes first,
then the Bedrock `SPEND` lines and the ledger's sum against `bedrockUsd=`,
then whether Hyper's daily limit named a return and Bedrock carried the shared seats through it.

## The first hakureico page, 2026-09-07, 22:00 UTC, and the thirteenth class

`TALLY hakureico status=SETTLED slices=18` at 22:00:47 UTC,
13.3 wall minutes,
the shortest settled pass on record;
`repairIssues=80 repairAccepted=38 repairResolved=26 repairChanged=3`,
`translateStatus=unfilled`,
`documentsDiffer=3 pageChanged=3 pageSilent=1`,
`selection=contested`,
`DESTINATIONS source=0 page=0 dropped=0`,
`verify-published` exit 0.
Calls:
Bedrock 224 (0.2448 USD by the log's `cost=` sum;
the ledger grew by exactly that,
0.005607 to 0.250384 USD,
and `bedrockUsd=` read 199.77 at the last `METERS`),
Hyper 151 (664 to 593),
Synthetic 0,
OpenRouter 0.
The first pass in which Bedrock took the two seats it shares with Hyper:
its first two calls went to `gemma-4-26b-a4b` and `gpt-oss-120b` at 21:47:27 UTC,
and neither seat touched Hyper all pass.
`SEATS DARK` names Qwen3.8-27B,
thrown 30 of 30,
which is the `markRefused` hold on Hyper and not the model.

### The twelfth class working, and what it uncovered

At 21:57:25 UTC three Hyper calls answered 429 naming a return in 538000 ms;
the ladder ended at once ("past this ladder's reach of 16000ms"),
and `markRefused` held Hyper out for 538000 ms,
to 22:06:24 UTC.
That is the twelfth class as built.
What it uncovered is the thirteenth:
Bedrock stayed wet,
so nothing waited.
The translate lane started at 21:58:46 UTC and every Hyper-only writer
(GLM-5.3-Flash,
Kimi-K3,
minimax-m3,
deepseek-v4-pro-0813,
glm-5.3)
was refused as `NoProviderForModelError` in the same millisecond,
leaving the two Bedrock seats to write;
`every proposal was the incumbent`,
`winner short of the minimum vote weight; keeping the incumbent`,
`declined-indecision` on every slice,
and chunk 17,
the passage only the translate lane owns,
stayed unfilled.
The lane contest at 22:00:14 UTC read `hyper=dry openrouter=dry` (OpenRouter had just refused at 0.01 USD),
settled its three differing slices on repair with the two Bedrock judges,
and the consolidation at 22:00:24 UTC ran every select and refiner round at 0 to 1 ms with nobody heard:
three chunks `quorum-not-met`,
done by 22:00:46 UTC,
five and a half minutes before Hyper came back.

The router already waits out the shortest hold once,
but only when every provider reads dry (`readBudgetsPastHolds`),
because that is the case where the alternative is ending the run.
A phase is the same case one level up.
Fixed at `752bf9a9b`:
`readJudgeSeats` names the benches each phase leans on
(`run-seats-wait.ts`:
wide for preparation and the contest,
readers for pictures,
wide and translators for the lanes,
translators and select judges for the translate lane,
slate and wide for consolidation),
counts the seats a wet provider would serve against each bench's quorum,
and when a bench is short and a provider has named its return,
waits out the shortest running hold once (`RunClient.providerHolds`,
the budgets' own) and reads again;
the `JUDGE SEATS` line carries `waited=`.
The lanes driver takes a `reseatTranslate` reader and the pass gives it one,
so the translate lane reads its seats when it is about to start rather than inheriting a reading minutes old
(the lanes were seated at 21:52 UTC,
the hold began at 21:57 UTC,
the writers were asked at 21:58 UTC).
Per phase rather than per call,
because a call's deadline (360 s) is shorter than a daily-limit hold and would cut the wait it was serving.
Six guards bite
(`run-seats.unit.test.ts`,
`run-seats-wait.unit.test.ts`,
`document-lanes.unit.test.ts`,
`pass-entry.unit.test.ts`,
whose seat-reading count is six now);
types and oxlint clean.

### The page

The mechanical reading (`read-page.mjs`):
0 straight apostrophes in words against 8 curly (archive 0 and 8),
0 straight double quotes against 18 curly (archive 0 and 18),
1 three-dot ellipsis on the page against 0 on the archive,
3 headings and 3 distinct,
3 comments (archive 3),
the PhotoScroll line byte-identical,
none of the refusal vocabulary in the log,
`mdx-downgraded` 0,
`would ship a page` 0,
169 lines against the archive's 158.

The three changed slices,
read against the source:

- Transit.
  Source:
  `不仅关注着城市公交，还经常坐火车，偶尔，她也坐上飞机看看蓝天。`
  Archive:
  `She was interested in city buses, rail transit and national railways.`
  Page:
  `Not only did she follow city buses, she often rode the trains as well.`
  The archive's national railways were its own;
  the page is the source.
  The next line,
  `Though, so many of those journeys were ones she made alone...`,
  renders `虽然，许多时候都是一个人的旅行呢……` with a three-dot ellipsis where the archive had rewritten the sentence without one;
  the corpus's own pages split 39 three-dot to 25 unicode of 92,
  so there is no convention to restore against,
  and the archive file carries none.
  Not a class.
- osu!.
  `Her osu!` then `account stayed online even when she was in a bad mental condition.` on the next line
  is the source (`她的 OSU 也是在线状态`) where the archive said `She was still online`.
  The line breaker took `osu!` for a sentence end and broke inside the sentence;
  Markdown renders the break as a space,
  so the page reads right and the file's one-sentence-per-line habit is off by one line.
  Recorded,
  not a class.
- Games and IKEA.
  `she gradually lost interest in many game-related things due to life pressure, and she no longer posted updates afterwards`
  and `Hanasaka and her companions went to IKEA and ate cheap, tasty little ice cream cones together`
  are the source (`对许多游戏相关的事情逐渐失去了兴趣`;
  `千歌和她的伙伴去过宜家，一起吃了便宜好吃的小甜筒`),
  where the archive had `games` and `an acquaintance`.
  Hanasaka is the archive's rendering of 千歌 throughout.

### The footnotes

The source carries two:
`「Mayday[^1]」` with its definition,
and `HOSTED__WITH__GAE[^2]____` with its definition.
The archive carries neither marker and neither definition
(`> HOSTED__WITH__GAE____`),
and the page carries neither.
The repair lane saw them:
the Mayday slice's candidates carried `[^1]` and the judges argued its placement against the corner brackets
(seven ballots name the marker),
and the slice shipped unchanged.
The definitions are chunk 17,
the passage only the translate lane owns,
which the hold left unfilled.
So the second footnote carrier shipped without its footnotes,
for the reason the thirteenth class names;
the fourth launch on `752bf9a9b` is where the footnote reading happens.

What the page is evidence of:
Bedrock in production for its two shared seats,
the twelfth class closing a daily-limit hold in one line instead of 2,693 attempts,
three faithful repairs,
and the thirteenth class.
What it is not evidence of:
the footnotes,
the translate lane,
or judged consolidation.

## The fourth hakureico launch, 2026-09-07, 22:21 UTC, on the class-thirteen build

`hakureico` relaunched at 22:21:26 UTC on `2b13ef813` (code `752bf9a9b`),
pipeline `d6bfaa95`,
plain invocation,
fresh runs dir `~/temp/agent/hakureico4-20260907`,
log beside it,
pid 3814733.
`METERS` at launch:
Synthetic dry,
Bedrock 199.75 USD,
Hyper 593,
OpenRouter 0.01 USD.
`JUDGE SEATS phase=preparation` reads the third launch's benches with `waited=0ms`.
The reading follows:
the footnotes first
(`[^1]` on Mayday,
`[^2]` on the GAE line,
the two definitions the translate lane owns),
then every `JUDGE SEATS` line's `waited=` and any `short of quorum` line,
then the Bedrock `SPEND` lines and the ledger against `bedrockUsd=`.
Hyper's daily limit reopened at 22:06 UTC and its next closing is unpublished;
a pass that meets it now waits at the next phase boundary instead of settling on nobody.

## The fourth hakureico pass stops INCOMPLETE; the thirteenth class's second face, 2026-09-07, 22:56 UTC

Every `JUDGE SEATS` line read `waited=0ms`
(preparation 22:21 UTC,
pictures 22:21 UTC,
lanes 22:23 UTC,
translate lane 22:42 UTC with six writers reachable,
lane contest 22:49 UTC,
consolidation 22:53 UTC),
the translate lane ran on the full bench,
and `consolidation: 18 contested slices to settle`.
Then at 22:55:54 UTC,
two minutes into consolidation,
Hyper's daily limit answered 429 naming 923 s
(nine bodies,
held out to 23:11:18 UTC);
the phase had already seated,
so the chunks in flight and after ran on the two Bedrock seats
(`absolute naturalness review: 2/7 usable, quorum-not-met`).
Slice 5's contest winner had failed the block floor at 22:50 UTC
(`Your translation is 2 blocks (paragraph, paragraph) and the PAGE AS IT STANDS is 1`,
the games-and-IKEA paragraph split in two,
where the third pass's repair had kept it whole);
consolidation withheld that standing text from the slate,
the slate under the hold declined,
and with nothing valid to ship the entry stopped:
`TALLY hakureico status=INCOMPLETE ms=2125586`,
`ConsolidationStandingIneligibleError: slice 5 ... (slate-declined-standing)`.
That stop is the contract as decided
(an unheard roster is not persisted and a page that would be refused is not written);
the hold is what emptied the slate.
Calls:
Bedrock 373,
Hyper 689 (593 to 305 credits,
the largest single-pass spend on record,
since the translate lane and eighteen consolidations all ran);
ledger 199.39 USD.

The phase-boundary wait of `752bf9a9b` cannot see a hold that begins inside a phase.
Fixed at `e17d0c487` and guarded at `8e64a6ef7`:
the seat reader moves to `run-seats-read.ts` with `awaitBenchQuorum` beside it,
which reads the holds synchronously and returns at once while nothing is held
(no dryness read,
so a pass under wet providers asks its meters exactly as often as before),
and otherwise reads the benches and waits out the shortest hold once when one the phase leans on is short;
`repairPreparedDocument`,
`translateDocument` and `consolidateDocument` take a `beforeSlice` hook awaited before each chunk,
the lanes driver routes it per lane,
and the pass wires it through `lanesHooksFor` and the consolidation seam.
Twelve guards bite across the two commits;
types and oxlint clean.
What remains uncovered is a hold that begins while a chunk's own rounds are in flight:
those rounds lose their Hyper voices and retry three times at once;
the next chunk waits.

## The fifth hakureico launch, 2026-09-07, 23:09 UTC, on the per-chunk-wait build

`hakureico` relaunched at 23:09:43 UTC on `3fc918013` (code `8e64a6ef7`),
pipeline `b34822e2`,
plain invocation,
fresh runs dir `~/temp/agent/hakureico5-20260907`,
log beside it,
pid 3852763.
`METERS` at launch:
Synthetic dry,
Bedrock 199.36 USD,
Hyper 301,
OpenRouter 0.01 USD.
Hyper's hold from the fourth pass (to 23:11:18 UTC) lived in that process's budgets and not in this one's,
so this pass reads Hyper wet,
meets the same refusal on its first Hyper call,
and holds it out for what the body names;
the first chunk after that reads `short of quorum` and waits.
The reading follows:
the footnotes first,
then every `JUDGE SEATS` line's `waited=` and every `chunk resumes` line,
then the Bedrock `SPEND` lines and the ledger,
then Hyper's balance against 301.
Hyper's balance is the day's constraint now:
828 at 21:00 UTC,
301 at 23:09 UTC,
the owner will not recharge it,
and each hakureico pass has spent 70 to 290 of it.

## The fifth pass stops in 67 seconds; the fourteenth class, 2026-09-07, 23:10 UTC

`TALLY hakureico status=INCOMPLETE ms=66920 error=visual evidence incomplete for 2 referenced assets`.
The first Hyper calls at 23:09:44 UTC met the daily limit still running from the fourth pass
(429 naming 94 s;
this process had no hold for it),
and in the same second six OpenRouter calls answered
`402: This request requires more credits, or fewer max_tokens. You requested up to 131072 tokens, but can only afford 2411`.
OpenRouter's meter reads 0.01 USD,
which `openrouterIsDry` reads as wet,
so `markRefused` held it out for the 60 s rate-limit backoff.
At 23:10:00 UTC the pictures phase read
`readers 0 of 4 reachable, quorum 2; holds hyper 77970ms, openrouter 44473ms; waiting 44473ms`:
the thirteenth class waited for the shortest hold,
which was OpenRouter's,
re-seated three readers on OpenRouter,
and every reading was refused for payment again;
the pass stopped before the lanes,
8 Bedrock calls and 0 Hyper calls spent.

A payment refusal is a statement about the balance,
not about the minute.
Fixed at `c9cd537e6`:
`isPaymentRefusal` names HTTP 402;
`markRefused({ paymentRequired })` marks the provider with the meter level it read
(`openrouterUsd=0.01`)
and `read()` folds it as dry while that level stands,
with no timed hold,
so `holds()` reports nothing for it and the seat wait targets the hold that would bring a bench back.
The mark clears when the meter moves,
whichever way,
since the refusal was about the balance read then;
a top-up is the only thing that moves it up.
Four guards bite (`provider-budget.unit.test.ts`);
types and oxlint clean.
With OpenRouter reading dry,
the roster with Synthetic's week spent is Bedrock's two shared seats and Hyper's six,
and Hyper's daily limit is what every phase and chunk now waits out.

## The sixth hakureico launch, 2026-09-07, 23:38 UTC, on the seated build

`hakureico` relaunched at 23:38:03 UTC on `49aca5770` (code `b0b48d6f4`,
the same pipeline `6bb40f9b` as `f9a17ebf1`),
plain invocation,
fresh runs dir `~/temp/agent/hakureico6-20260907`,
log beside it,
pid 3905110.
`METERS` at launch:
Synthetic dry,
Bedrock 199.34 USD,
Hyper 277,
OpenRouter 0.01 USD.
`JUDGE SEATS phase=preparation` reads
`wide=7 select=6 late=8 slate=7 checkers=3 translators=6 readers=3 writers=8 roster=9`,
`withheld=hf:Qwen/Qwen3.8-27B,hf:moonshotai/Kimi-K3`,
`waited=0ms`:
the first pass with `google.gemma-4-e2b` judging and with a `writers` bench of its own.
The reading follows as for the fifth launch,
plus whether OpenRouter's first 402 reads it dry for the rest of the pass
(`refused us for payment` once,
no 60 s cycle),
and whether Hyper's next daily-limit hold is waited out at a chunk boundary
(`short of quorum` then `chunk resumes`).

## Measuring the two Bedrock-only sizes, 2026-09-07, 21:19 UTC

The probes ran the run roster and nothing else,
so a seat that needs a number before it is taken could never be asked for one.
Landed on `translation-repair-class12` at `abdb06c1b` and `35e0b1fad`
(the branch stays unmerged until `hakureico` settles,
under the kill-and-relaunch rule):
`--candidates a,b` names seatable ids that judge beside the seated roster in `judge-fidelity-probe`
and write and judge beside it in `producer-calibrate`,
for that run only;
`--candidates-alone` runs the named ids without the seated roster,
so a probe beside a production pass spends only at the candidates' provider.
An id the roster does not know is refused with the ids it does;
alone over nobody is refused.
Six guards bite (`probe-candidates.unit.test.ts`);
types and oxlint clean;
the full suite green.

### The instrument as it stands

`judge-fidelity-probe` over a throwaway runs dir holding the three settled artifacts on disk
(Huasheng from the fourth pass,
TLL1122,
yulianNyanner from the seventh),
`--cap 36`,
build `35e0b1fad`.
TLL1122 carries no slice of 400 characters;
neither remaining slice states a number the alteration fixture can move;
so the walk built one deletion and one insertion pair on each of two slices,
Huasheng/1 and yulianNyanner/3,
each put four ways:
16 rows.
Two of the four ways share one slate
(the same two texts in the same order,
differing only in which is called the incumbent),
and the select memo answers the second without a call:
7 of 16 rounds closed at 0 ms,
and every judge was asked 8 distinct questions.
The direction column of this probe measures nothing on a warm memo;
recorded,
not fixed here.

### The candidates alone, 21:19 UTC

Judges `google.gemma-4-e2b` and `google.gemma-4-31b`,
nobody else,
16 Bedrock calls,
0.0045 USD by the ledger.
Distinct questions answered with the complete text:

- `google.gemma-4-e2b`:
  7 of 8.
  Deletion 3 of 4,
  insertion 4 of 4,
  never declined;
  the miss chose the damaged text where it sat second on the Huasheng deletion,
  and it chose position two on 5 of 8.
- `google.gemma-4-31b`:
  4 of 8.
  Deletion 2 of 4,
  insertion 2 of 4,
  declined the other 4,
  every one on Huasheng/1,
  saying both candidates add imagery and emotional language the original lacks;
  it never chose the damaged text.

### The seated roster on the same questions, 21:20 UTC

The nine roster models,
72 calls,
`fidelity: 16 of 16 trials chose the complete text`;
per judge,
distinct questions answered with the complete text:

- 8 of 8:
  `hf:moonshotai/Kimi-K3`,
  `deepseek-v4-pro-0813`,
  `hf:zai-org/GLM-5.3-Flash` (no judge seat since 2026-09-02).
- 7 of 8:
  `deepseek-v4-flash-0731` (one damaged pick),
  `glm-5.3` (one decline),
  `hf:Qwen/Qwen3.8-27B` (one decline;
  no judge seat since 2026-09-03).
- 5 of 8:
  `hf:openai/gpt-oss-120b` (three declines),
  `minimax-m3` (one damaged pick,
  two declines).
- 4 of 8:
  `gemma-4-26b-a4b-it` (four declines,
  all on Huasheng/1,
  the family's reading).

Among the seven judges the wide seats hold,
the median is 7 of 8 and the floor 4 of 8.

### The rule, written before the next questions arrive

Two slices are eight questions,
which separates a family's habit of declining from a habit of choosing the damaged text
and not much finer.
The fourth settled artifact,
`hakureico`,
adds one slice when the pass settles,
and the probe is re-run over all four before any seat moves.
The rule for the wide seats,
pre-registered here as the 2026-09-01 rules were:
a candidate joins critic,
panel and judge when,
over the same distinct questions,
it chooses the complete text at least as often as the median seated judge
and chooses the damaged text no more often than the seated judge who chooses it most;
a candidate that declines its way below the floor stays out.
On the eight so far `google.gemma-4-e2b` meets both clauses
(7 against a median of 7;
one damaged pick against `deepseek-v4-flash-0731`'s one and `minimax-m3`'s one)
and `google.gemma-4-31b` meets neither
(4 against 7,
on declines).
The translator seat is `producer-calibrate`'s to give,
by the pooled null as on 2026-09-01,
run with `--candidates` over the whole roster once the pass is done with the meters.
Editor and refiner seats stay where the 2026-09-01 calibration put them;
no candidate is measured for them here.

### The candidates alone over four artifacts, 22:21 UTC

With the third hakureico artifact beside the three,
`--cap 48 --candidates google.gemma-4-e2b,google.gemma-4-31b --candidates-alone` on `752bf9a9b`:
four entries walked,
TLL1122 still too short,
three slices,
24 rows,
12 distinct questions per judge,
24 Bedrock calls.

- `google.gemma-4-e2b`:
  11 of 12.
  Deletion 5 of 6 with one damaged pick,
  insertion 6 of 6,
  never declined;
  position two on 7 of 12.
- `google.gemma-4-31b`:
  6 of 12.
  Deletion 2 of 6,
  insertion 4 of 6,
  declined the other 6,
  never chose the damaged text;
  position two on 3 of 12.

The seated roster's reading over the same twelve is bought after the fourth hakureico pass settles,
since its 108 calls would land on the Hyper quota the pass is spending;
the rule is applied then.

### The seated roster on the same twelve, 23:17 UTC, and the rule applied

The nine roster models over the same three slices,
108 calls (84 Hyper,
24 Bedrock,
no daily-limit refusal),
`fidelity: 24 of 24 trials chose the complete text`;
per judge,
distinct questions answered with the complete text:

- 12 of 12:
  `glm-5.3` (no judge seat since 2026-09-01),
  `hf:Qwen/Qwen3.8-27B` (none since 2026-09-03),
  `hf:moonshotai/Kimi-K3`,
  `hf:zai-org/GLM-5.3-Flash` (none since 2026-09-02).
- 11 of 12:
  `deepseek-v4-pro-0813` (one damaged pick),
  `minimax-m3` (one damaged pick).
- 8 of 12:
  `deepseek-v4-flash-0731` (one damaged pick,
  three declines).
- 7 of 12:
  `hf:openai/gpt-oss-120b` (five declines).
- 4 of 12:
  `gemma-4-26b-a4b-it` (eight declines).

The six judges the wide seats hold read 12,
11,
11,
8,
7 and 4:
median 9.5,
worst damaged count 1.
By the rule written before these numbers:

- `google.gemma-4-e2b`,
  11 of 12 with one damaged pick,
  meets both clauses and JOINS critic,
  panel and judge in both lanes,
  the late bench and the slate.
  It holds no writing seat,
  since no producer calibration has measured it:
  `RUN_WRITERS` names the consolidation writers and the translate lane leaves it out
  (`b0b48d6f4`,
  four guards bite).
  Bedrock serves it alone,
  so it is the third judge that answers while Hyper is held out.
- `google.gemma-4-31b`,
  6 of 12 on declines,
  meets neither and stays out.

Recorded as the addendum of 2026-09-07 in `doc/decision/translation-repair-roster-seating-2026-09-01.md`.
