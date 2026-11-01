# OpenRouter and the corpus passes: 2026-09-07, 11:14 to 21:08 UTC

What was launched,
read and decided on 2026-09-07 11:14 to 21:08 UTC.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The third Huasheng launch ends at the publisher, 2026-09-07, 11:14 UTC: the eleventh class

`Huasheng` ended `INCOMPLETE` at 11:14 UTC after 9,014,980 ms (150.2 minutes),
2,539 calls (Hyper 2,158,
Synthetic 381),
89 voices abandoned after quorum,
no voice lost to a dry provider,
Hyper 2080 to 1275,
weekly Synthetic 2 to 0 percent within the first hour,
zero USD,
no page.
The Synthetic seats threw once the week dried
(Qwen3.8-27B 58 of 373 asks,
Kimi-K3 16,
GLM-5.3-Flash 11)
and the pass went on without them.

Slice 21,
the poem,
passed the lane contest under the either-rendering floor,
had its consolidation standing withheld once by the line-structured rule
(7 lines against 8)
and shipped from the valid proposals;
the tenth class is closed where it was found.
The container halves passed again.
No slice was unreadable.

The entry then stopped at the publisher:
`entry Huasheng front matter is not publishable (directory-id-name)`.
The rule,
the owner's of 2026-09-02 in `doc/decision/translation-repair-front-matter-guard.md`,
refuses a page whose visible `name` is the directory id while the source's is not,
written for
archives whose metadata was never translated and still name the folder.
The source names the person 椛笙,
whose pinyin is Huasheng;
the archive's `name` is `Huasheng`;
the judges' front-matter ballot chose `Huasheng` over the original script by the sheet's own identity
rule,
and the floor refused it because the string is the directory id.
The rule cannot tell a folder name left untranslated from a romanisation that coincides with the
folder name,
and for this entry every correct rendering coincides.

Measured today:
22 archives name the directory.
8 do so because the handle is the person's name in the source too,
as the decision record says.
Of the other 14,
the directory id is itself a rendering of the source's name for 7:
`Huasheng` (椛笙),
`lintong` (林童),
`Kotori` (琴里),
`MioCardMeow` (澪卡喵),
`MocaKawai` (摩卡好可爱),
`noname` (无名逝者),
`donotexist_A` (不存在);
the other 7
(`DarlinChit`,
`dogesir_`,
`homoyamakaze`,
`interrgned`,
`lxyddice`,
`Weideriche_`,
`XingZ60`)
are handles the source does not use as the name,
the directory-id shape.
Under the rule as written the first 7 cannot ship unless a lane renders the name some other way.

A design question on a decided rule,
recorded for the owner:

- A,
  judged and read,
  not floored:
  `directory-id-name` becomes a finding the publisher logs and the reading checks,
  and the judges' identity rules decide the name.
  Pro:
  dissolves the 7;
  the judges see the source name and the sheet's rules;
  the owner's own premise of 2026-09-02 was to stop caring about metadata differing from the source.
  Con:
  a folder name the judges keep on one of the seven handles ships,
  and only the reading catches it.
- D,
  a romanisation check:
  the id-equal name passes when it is a pinyin reading of the source's name.
  Pro:
  deterministic where it applies.
  Con:
  a new dependency with heteronyms,
  and it covers 2 of the 7 (`Huasheng`,
  `lintong`),
  not the Japanese reading,
  the literal renderings or `noname`.
- C,
  exempt the id-equal name where the source's name appears among the page's aliases in original
  script.
  Pro:
  no dependency,
  and the identity rule already puts the source name in the alias.
  Con:
  the seven handles can pass the same way,
  so it is A with a weaker reading.
- B,
  keep the rule and tell the producers to render the name otherwise.
  Con:
  for 椛笙 there is no other correct rendering.

Ranking A > D > C > B:
A over D because D covers two of seven and adds a dependency;
D over C because C admits what A admits with less honesty about it;
C over B because B cannot be satisfied for this entry.
Nothing is changed in code until the owner answers.
Hyper stands at 1275 credits;
a Huasheng pass on Hyper alone costs about 800.

## The owner answers on the name, 2026-09-07, 11:30 UTC: the pinyin check and the alias exemption

The owner's words:
"pinyin check and alias exemption (see both the original Chinese and the original English,
there's gotta be a English rendering in the frontmatter)".
Landed in `912dbe2dc`,
`corpus-run/directory-id-name.ts`,
recorded as an addendum to `doc/decision/translation-repair-front-matter-guard.md`.
The id-equal visible name stands when any of three holds:
the id,
lower-cased and reduced to its Latin letters,
is a pinyin reading of the source's name with every heteronym of every character allowed
(`pinyin-pro`,
a new catalog dependency,
MIT,
no dependencies of its own,
reads 椛 as hua and 单 as dan,
shan or chan);
the source itself carries the id among its aliases;
or the page or the archive carries an alias in Latin script other than the id,
the English rendering the owner said the front matter has to have.
What stays refused is a folder name with no Latin rendering beside it on either side.

Measured over the pinned corpus with the rule as landed
(scratch `name-census.mjs`):
23 archives name their directory,
8 in the source too,
and the other 15 all stand,
`Huasheng` and `lintong` by pinyin among them and the seven handles by the archive's own alias
(Sakuya,
Lan Gou,
Qian Yu Mao Tou,
Danpian,
lxy,
Zihe,
Lili);
none falls.
Suites:
directory-id-name,
front-matter-completeness,
publish-fixed 0 `FAIL`;
oxlint 0 and 0;
types clean;
the lockfile diff is the one package.
Guard shown to fail with every clause neutralised,
5 `FAIL` lines in the rule's suite and 3 in the completeness suite,
0 restored;
full suite 937 `PASS`,
0 `FAIL`.

## The fourth Huasheng launch, 2026-09-07, 15:43 UTC, on the class-eleven build

`Huasheng` launched at 15:43:52 UTC on `32dd30437`,
pipeline `a4c73f76`,
plain invocation,
fresh runs dir `~/temp/agent/huasheng4-20260907`,
log beside it,
after the full suite on that build read 937 `PASS` and 0 `FAIL`.
Meters at launch:
Synthetic 1.98 percent of the week,
Hyper 1272,
OpenRouter 0.32 USD,
every seat filled.
Hyper carries the pass once Synthetic dries,
at about 800 credits,
so this is the last pass the meters allow before a refill or a top-up.
The reading follows,
the front matter's name first,
then the container,
the poem and both footnote conventions.

## The fourth Huasheng launch finds the twelfth class, 2026-09-07, 19:40 UTC: Hyper's daily limit

The monitor timed out at 19:40 UTC with the pass alive and in consolidation,
begun at 18:01 UTC on an eight-model roster and heavily degraded.
Measured off the log at 19:40 UTC:
`createRequestPace` had written "window full (1000 starts in 3600000ms)" 1,028 times since 16:42 UTC;
`exchangeWithRetry` had logged `HTTP 429` 2,693 times;
`markRefused` had held Hyper out 831 times,
60 s each,
"while its meter reads wet";
`HTTP 402` from OpenRouter,
"This request requires more credits",
463 times since 17:01 UTC;
and `EveryProviderDryError` 34 times.
Per ten-minute bucket,
Hyper's streams carried content until 16:59 UTC (83 to 202 per bucket)
and none at all from 17:00 UTC on (106 to 285 empty streams per bucket),
while the meter read `hyper=wet` at a balance of 909 the whole time,
unchanged.
Consolidation chunks 1,
3 and 4 each took about 75 min to settle,
every one on "winner short of the minimum vote weight;
keeping the repaired text",
with every producer round reading "0/6 heard" and "retry round 2 for 6 lost voices".

The refusal's own words,
logged where a voice was lost:
`You've hit your daily rate limit. Please try again in 2h25m18s.`,
84 bodies,
every wait counting down to one instant:
17:27:55 UTC plus 2h25m18s is 19:53:13 UTC,
19:39:19 UTC plus 14m40s is 19:53:59 UTC.
Hyper's daily limit,
not its hourly one,
and its wording names its return.

Two sites read it wrong.
`retryAfterMsOf` in `transient-retry.ts` scanned "try again in ",
a run of digits,
then an `s`;
`14m40s` is digits then `m`,
so it read no wait and the ladder retried on its jitter,
five attempts a call.
`markRefused` in `provider-budget.ts` then took a 429 on a wet meter for a concurrency limit,
which the pin pass of 2026-09-02 had taught it,
and held Hyper out for the 60 s backoff while OpenRouter read wet,
which sent the next calls back into the same wall.
Neither the pacer nor the meter could see it:
the pacer counts starts,
and the balance does not move on a refusal.

The fix,
class twelve,
in three clauses:
`retryAfterMsOf` reads hours,
minutes and seconds as Hyper writes them
(`2h25m18s`,
`14m40s`,
`2h`);
the ladder ends at once on a wait past its own widest backoff window
(`longestBackoffMs`,
base times two to the retry limit,
16 s in production),
returning the reply for the router to hold the provider out;
and `markRefused` takes `statedWaitMs` from the refusal (`statedWaitMsOf` in `provider-budget-refusal.ts`,
called by the router and the re-ask)
and holds the provider out for at least that long,
whatever the meter reads.
The existing hold-wait then does the right thing on its own:
a caller facing only held providers waits out the shortest hold,
so a voice keeps trying whichever provider comes back first and is lost at its own deadline,
not before.
Guards:
the ladder suite gains the composite parse and an ENDS-AT-ONCE case on the daily wording,
and its WAITS case moves to a policy whose reach covers the one second it names;
the budget suite gains two cases,
a named wait past the backoff and a named wait where the meter alone would hold nothing;
the router suite gains a case passing the named wait through.
Neutralised one at a time they fail 3,
2,
3 and 2;
restored,
0.

Landed on branch `translation-repair-class12` at `31e67a100`,
built and proven in a throwaway worktree (`~/temp/agent/tr-class12-20260907`),
because the kill-and-relaunch rule forbids superseding a running pass's build,
the run was five minutes from Hyper's return,
and the page it would ship is the class evidence this launch exists for.
It lands on `translation-repair-rebased` when the pass ends,
before the next launch.
The worktree's `git worktree add` wedged every cli-git call for ten minutes under the worktree-copy settlement,
and its `cli-git.config.ts` had to be trusted and the forbidden-strings scanner copied in before a commit went through;
the system git answered throughout.

Hyper came back at 19:54 UTC as the bodies said.
Chunk 6 then settled in 43 min,
chunk 9 in 9 min,
chunks 19 to 24 in 4 to 7 min each.

## The owner adds Amazon Bedrock, 2026-09-07, 19:50 UTC

The owner,
in four messages while class twelve was being written:

> I have added TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY to main worktree's secrets file.
> You can copy it.
> I have 200USD of credits in my Amazon Bedrock account.
> With ZDR enabled,
> only these models are usable (and approved):
> Claude Sonnet 5,
> Gemma 4 E2B,
> Gemma 4 31B,
> Gemma 4 26B-A4B,
> GPT OSS 120B.
> Once again,
> any number of provider being dry or lacking API key is expected and normal operation.

> The credits in my Amazon Bedrock account will expire early next year,
> so you're allowed to use it as much as you like.
> But I will NEVER top it up.

> 1. Please do not introduce Amazon Bedrock SDK,
> Anthropic SDK,
> OpenAI SDK because these are poorly written.
> Use raw fetch.
> 2. I retract Sonnet 5.
> Looks like Amazon didn't properly display it as not supporting ZDR.

The key was copied into this worktree's `.env.local.json` with `sops set`,
value never printed,
136 characters on both sides and equal.
The sops call needs `SOPS_AGE_KEY_FILE=~/.config/mise/age.txt`,
the identity mise decrypts with;
without it sops finds no key.

### What the account answered, read-only, before any code

`GET /foundation-models` with the bearer key answered 200 in us-east-1,
us-west-2,
eu-west-1,
eu-central-1 and ap-northeast-1.
None listed a Gemma 4;
`anthropic.claude-sonnet-5` was listed as `INFERENCE_PROFILE` only,
with profiles `us.` and `global.`.
`bedrock-runtime`'s `/openai/v1/models` answered 404.
`bedrock-mantle.us-east-1.api.aws/v1/models` listed 55 models including all five named,
every one with `data_retention.mode: "none", source: "account"`;
us-west-2's mantle list had the Gemma sizes and gpt-oss but not Sonnet 5.

One request per model on mantle's `/v1/chat/completions`,
streamed with usage and a json_schema response format:
gpt-oss-120b answered 200 with the schema honoured,
usage on the last chunk,
reasoning deltas before content,
and no `data: [DONE]`;
every Gemma 4 answered 400 "model isn't supported on this route";
Sonnet 5 answered 400 "does not support the '/v1/chat/completions' API".
Sonnet 5 on mantle's `/v1/responses`:
the same refusal;
on mantle `/v1/messages`:
404;
on bedrock-runtime,
chat and Converse,
as `us.` and `global.` profiles:
403 "anthropic.claude-sonnet-5 is not available for this account".
The owner retracted it while this was being read.
Gemma 4 on mantle's `/v1/responses` hung for three minutes with no event;
with `background: true` the account answered "This account requires store=false for this model".

The model cards settled it:
"Gemma 4 models are available only on the bedrock-mantle endpoint",
and "On bedrock-mantle,
this model is served at /openai/v1",
with the programmatic base `https://bedrock-mantle.{region}.api.aws/openai/v1`.
On that route,
streamed with usage and the schema,
all three Gemma 4 sizes answered 200,
schema honoured,
usage on the last chunk,
`data: [DONE]` present,
no reasoning frames;
gpt-oss-120b on that route answered "isn't supported on this route".
So two routes on one host,
decided per model,
and two terminators.
The cards also say reasoning effort is honoured and recommend `reasoning_effort: high` for Gemma 4;
the client sends none,
by the owner's standing instruction of 2026-08-25,
and the probe's answers carried no leaked reasoning.

Prices,
read 2026-09-07:
Gemma 4 E2B 0.04 in and 0.08 out per million tokens,
Gemma 4 31B 0.14 and 0.40,
Gemma 4 26B-A4B 0.13 and 0.40,
on Bedrock's pricing page for US regions;
gpt-oss-120b 0.1545 and 0.618,
which that page listed for Sydney only.
Claude Sonnet 5 is 2 and 10 on Anthropic's page,
recorded here although retracted.
Mantle's quota page:
no requests-per-minute quota,
per-model token quotas published for one model only,
"their throughput is governed by internal service capacity",
retry with backoff on throttling,
ramp gradually.
Context windows off the cards:
128K for E2B and gpt-oss-120b,
256K for 31B and 26B-A4B;
a maximum output is published for gpt-oss-120b alone,
16K.

### What was built

Branch `translation-repair-class12`,
commit `7b532ae31` and a test commit after it,
in the same throwaway worktree and for the same reason.

- `bedrock-catalog.ts`:
  the four served spellings,
  each with the roster seat it stands in for
  (`google.gemma-4-26b-a4b` is `gemma-4-26b-a4b-it`,
  `openai.gpt-oss-120b` is `hf:openai/gpt-oss-120b`,
  the other two are their own),
  its route,
  its stream end,
  its prices and its context;
  `readsImages` false on every row for the reason the OpenRouter catalog keeps gemma off the picture readers,
  a transcription through this stack being unmeasured,
  so the reader sub-roster stays the measured four.
- `bedrock-ledger.ts`:
  the meter this provider does not have.
  An append-only JSON-lines file under `~/.local/state/translation-repair/bedrock-spend.jsonl`
  (`TRANSLATION_REPAIR_BEDROCK_LEDGER` overrides the path),
  one line per priced call,
  summed on every read against `BEDROCK_CREDIT_USD`,
  200 (`TRANSLATION_REPAIR_BEDROCK_CREDIT_USD` overrides it;
  an unreadable override throws).
  A line that will not read throws naming its number;
  a file that does not exist reads as nothing spent.
  Durable outside the runs directory because every launch uses a fresh one and the credit is spent across all of them.
- `bedrock-cost.ts`:
  usage times the catalog's prices,
  since the wire reports no cost and the account exposes no balance to a bearer key.
- `bedrock-stream-end.ts`:
  the sentinel check on the Gemma route,
  a usage-chunk check on the gpt-oss route,
  and the sentinel supplied to the shared reader where the route sends none.
- `bedrock-client.ts`:
  the OpenRouter client's shape over the same fetch transport,
  minus the `provider` field and the credits endpoint,
  plus the route per model,
  the terminator per model,
  the cost computed here,
  the ledger written after every priced call,
  and `credits` read off the ledger.
- `provider-name.ts`:
  `bedrock` third in `PROVIDER_ORDER`,
  ahead of OpenRouter,
  since prepaid and expiring money is spent before money that would be bought.
  Open to veto;
  moving it is one line.
- `roster-id.ts` and `roster-reach.ts`:
  `BEDROCK_ONLY_ROSTER_IDS` (`google.gemma-4-e2b`,
  `google.gemma-4-31b`),
  eleven roster ids,
  `bedrockIdFor`,
  reach and vision reach with the fourth column.
  Seatable is not seated:
  which roles the two new sizes take is a decision on evidence,
  as the roster calibration of 2026-09-01 was,
  and is asked under "What is asked of the owner".
- `budget-routing.ts`,
  `provider-budget.ts`,
  `run-config.ts`,
  `required-providers.ts`,
  `budget-sample.ts`,
  `spend-read.ts`,
  `spend-cost.ts`,
  `meter-sample-read.ts`:
  the fourth meter (`bedrock=` and `bedrockUsd=` on the `METERS` line,
  absent on older lines),
  the fourth key,
  the fourth client,
  the USD bucket admitting Bedrock seats.
- `provider-router.ts`:
  the refusal loop is bounded by the providers that serve the call,
  not the order's length.
  With a fourth provider that serves two of eleven models,
  the old bound asked once more after the last serving provider refused
  and ended on a no-provider error instead of the refusal itself;
  the router suite's FORWARDS TWICE case caught it before the fix.
- Under the line budget:
  `provider-meters.ts` takes the meter states,
  `meterRecordOf` and the read of all four meters out of `provider-budget.ts`;
  `run-providers.ts` and `run-config-error.ts` take the provider construction and its refusal out of `run-config.ts`;
  `bedrock-barrel.ts` takes the Bedrock surface out of `provider-barrel.ts`.
- Tests:
  five new suites (catalog,
  cost,
  ledger on a disposable directory,
  stream end,
  client on a recorded transport)
  and the fourth column added to 185 provider-keyed fixtures across 12 suites,
  each keeping its case's meaning
  (bedrock dry where no client is configured,
  holding nothing,
  absent on old lines).
  Seven guards neutralised one at a time fail two or three cases each;
  restored,
  0.
  Types clean,
  oxlint 0 and 0.

### What is asked of the owner

Where the two new Gemma 4 sizes sit.
The roster calibration of 2026-09-01 seated models by measured fidelity,
and nothing has been measured for `google.gemma-4-e2b` or `google.gemma-4-31b`.
The recommendation is to run the existing probes on them through Bedrock
(`judge-fidelity-probe`,
`producer-calibrate`),
which the credit covers many times over,
and seat by the numbers;
until then the provider serves the two seats the roster already names.

## The fourth Huasheng page, 2026-09-07, 20:49 UTC

`TALLY Huasheng status=SETTLED slices=25` at 20:48:51 UTC,
a page of 9,436 characters,
`DESTINATIONS source=0 page=0 dropped=0`,
`verify-published` matched 1 of 1 at the length its artifact implies,
305 wall minutes,
of which 2h53m were the daily-limit stall.
Meters:
Hyper 1272 to 829,
Synthetic 1.98 to 1.02 percent of the week,
OpenRouter 0.32 to 0.01 USD;
3,110 calls,
Hyper 1,581,
Synthetic 531,
OpenRouter 200;
94 voices abandoned after quorum.

The mechanical reading (`read-page.mjs`):
0 straight apostrophes in words against 26 curly (archive 0 and 27),
0 straight double quotes against 16 curly (archive 0 and 18),
0 ellipses of either form on page and archive alike,
3 headings and 3 distinct,
the PhotoScroll line byte-identical to the source's,
0 comments,
none of the refusal vocabulary in the log,
`mdx-downgraded` 0,
`would ship a page` 0.

The class targets,
in the order the launch record asked for them:

- The name (class eleven):
  the front matter reads `name: Huasheng`,
  an alias line carrying `Huasheng(Peanut)`,
  `Little Huasheng` and the third alias romanised,
  and `location: Nantong, Jiangsu`;
  the id stands beside two Latin renderings and the floor let it through,
  where the third launch's publisher refused it.
- The container (class nine):
  `<details>` at line 97,
  the bold summary at 98,
  `</details>` at 108,
  both halves in place around three paragraphs.
- The poem (class ten):
  lines 147 to 164 carry the source's shape exactly,
  paragraph for paragraph and `<br/>` for `<br/>`,
  where the archive had five soft-break paragraphs;
  the either-rendering rule accepted the original's shape and the judges chose it.
- The footnotes:
  the source carries one,
  `[^1]` on the Kundera passage,
  and the page carries it inline at line 141 with its definition at 180.
  The archive had added a second,
  `[^2]`,
  a translator's easier version anchored on a sentence of its own five-line rendering;
  the page keeps that definition at 182 and has no inline `[^2]`,
  since its paragraph is shaped as the source's one.
  An unreferenced footnote definition renders nothing,
  so the page shows no defect,
  and the translator's addition is gone with its anchor:
  the cost the block-floor decision names,
  seen for the first time.
  Not a class;
  recorded.

Two smaller readings.
The intro quote's signature comes out as `> By:` and `> Li’an` on two lines where the source has one,
a soft break inside the blockquote that renders as one line.
The contributors line names `Lian` where the signature has `Li’an`;
the contributors line is the publisher's,
from the archive's front matter.

What the page is evidence of:
the eleventh,
tenth and ninth classes closed on the entry that found them,
and no thirteenth at the publisher.
What it is not evidence of:
judge quality.
Synthetic was dry from 17:00 UTC,
Hyper answered nothing from 17:00 to 19:54 UTC,
and four consolidation chunks settled on nobody;
the roster that judged this page is not the roster a production pass would have.

## The first hakureico launch, 2026-09-07, 21:00 UTC, on the four-provider build

Class twelve and the fourth provider fast-forwarded onto `translation-repair-rebased` at 20:55 UTC
(`263b7ca73`),
the four doc records followed (`c2b8103a5`,
`a97f39183`),
and the run-config refusal cases learned to clear the fourth key as they had learned the third on 2026-09-03
(`ee577ee0c`;
the merged tree's full suite had read 5 `FAIL` on exactly those cases in this worktree,
whose secrets file carries the key,
and 0 after).
Types clean,
oxlint 0 and 0,
the full suite green with those four.
`budget-sample` at 21:00 UTC read every meter wet:
Synthetic 0.97 percent of the week,
Hyper 828,
Bedrock 200.00 USD off an empty ledger,
OpenRouter 0.01 USD.

`hakureico` launched at 21:00:35 UTC on `ee577ee0c`,
pipeline `80958d2c`,
plain invocation,
fresh runs dir `~/temp/agent/hakureico1-20260907`,
log beside it,
pid 3692614.
The second footnote carrier after Huasheng,
chosen over the math pair for the footnote observation on the fourth Huasheng page.
The first pass with Bedrock in the order:
its two seats,
`gemma-4-26b-a4b-it` and `hf:openai/gpt-oss-120b`,
route there once Synthetic dries,
which at 0.97 percent is soon,
and the ledger writes its first lines.
The reading follows:
the footnotes first,
then the Bedrock seats' `SPEND` lines and the ledger's sum against the `METERS` line.

## The first hakureico launch was on the wrong roster; the second, 2026-09-07, 21:08 UTC

The first launch's `JUDGE SEATS` line read `roster=11 wide=9 translators=9`,
and its first eight `SPEND provider=bedrock` lines named `google.gemma-4-e2b` and `google.gemma-4-31b`,
twenty to 154 completion tokens each:
ballots.
The record under "The owner adds Amazon Bedrock" says seatable is not seated,
and `roster-id.ts` says the same over `BEDROCK_ONLY_ROSTER_IDS`,
but `RUN_ROSTER` in `run-config.ts` was the whole of `ROSTER_MODEL_IDS`,
which the fourth provider had grown to eleven,
so every role derived from it took the two unmeasured sizes:
judges,
critics,
panel and translators.
A defect in what landed at `7b532ae31`,
not in the pass.

Fixed at `645c8787b`:
`RUN_ROSTER` leaves the Bedrock-only ids out until the probes measure them,
guarded in `run-seats.unit.test.ts`
(the guard read 2 `FAIL` with the filter neutralised and 0 restored;
types and oxlint clean).
Under the kill-and-relaunch rule the first pass was killed by pid at 21:04 UTC,
four minutes in,
having spent 0.0011 USD of the credit on the eight calls
(the ledger's sum;
`bedrockUsd=200.00` at two decimals).

`hakureico` relaunched at 21:08:36 UTC on `645c8787b`,
pipeline `8cf0b4fa`,
plain invocation,
fresh runs dir `~/temp/agent/hakureico2-20260907`,
log beside it,
pid 3706395.
`JUDGE SEATS phase=preparation` reads
`wide=7 select=7 late=8 slate=8 checkers=3 translators=7 readers=4 roster=9 withheld=none`,
the roster of 2026-09-01 with every provider wet.
Bedrock now serves only the two seats the roster already names,
and those only third in line:
Hyper serves both,
so when Synthetic dried at 21:24 UTC
(the week at 0 percent after the seated fidelity run and this pass)
the two seats went to Hyper,
60 and 40 calls by 21:35 UTC,
and the pass had made no Bedrock call at all.
Bedrock takes them when Hyper holds,
which today means its daily limit,
or when Hyper's balance is gone;
the launch record's "route there once Synthetic dries" was wrong about the order.
Whether prepaid and expiring Bedrock money should sit ahead of Hyper's credits for the seats both serve
is the order question the fourth-provider record left open to veto,
and it is put to the owner in the handover.
