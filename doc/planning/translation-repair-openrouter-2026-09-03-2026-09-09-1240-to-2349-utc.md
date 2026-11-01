# OpenRouter and the corpus passes: 2026-09-09, 12:40 to 23:49 UTC

What was launched,
read and decided on 2026-09-09 12:40 to 23:49 UTC.
Entries:
Mio8.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The second noname page confirms the pairing on a bench OpenRouter left, and the sixth Mio stops at slice 3, 2026-09-09, 12:40 UTC

`TALLY noname status=SETTLED slices=13 repairStatus=repaired repairIssues=71 repairAccepted=58 repairResolved=32
repairFindings=199 repairChanged=9 translateStatus=unfilled translateChanged=5 documentsDiffer=11 pageChanged=8
pageSilent=1 alignmentFindings=8 selection=contested ms=2010335`
at 12:32 UTC on `efc9a4f3c`,
33.5 minutes,
`~/temp/agent/noname2-20260909`,
launched on Bedrock and OpenRouter and finished on Bedrock alone after the 12:22 UTC payment refusal
(55 refusals counted,
10 reviews `quorum-not-met`).
THE TWENTY-THIRD CLASS IS CONFIRMED AT THE PAIRING:
`block-pairing contested target (target 0: source 1 outvotes source 0, 7 to 3)`,
the section 0 relation is `{source: 1, target: 0}`,
slice 0 is the heading alone (`sourceChars=5`),
the admission read it `coverage=absent` and the translate lane owned it.
The first pass had the same nine voices and kept the heading on the paragraph;
this one reads seven to three for the paragraph pair.
AND THE HEADING STILL DOES NOT SHIP:
two translators of eight were reachable and both rendered `## Introduction`;
the judges who voted chose it
(`deepseek-v4-flash-0731` at weight 1,
`google.gemma-4-e2b` at 0.5,
"complete,
faithful,
and natural English"),
but `winner drew only weight 1.5 across 5 ballots (3 abstentions); keeping the fallback`,
twice,
then a follow-up round at 0.5,
and `slice 0: no translation in the archive and none produced (no-candidate-backed); the passage stays missing`,
`source-passage-unfilled (slice 0, no-candidate-backed)` on the tally.
`MIN_SELECTION_WEIGHT` is 2 (`candidate-select-model.ts`:
"two is the smallest number that makes a selection an agreement rather than an opinion"),
and on Bedrock alone the select bench's whole reachable weight is 2 to 2.5
(`gpt-oss-120b` at 1,
`gemma-4-26b-a4b` at 0.5,
`gemma-4-e2b` at 0.5 or 1 by bench),
so a winner is seated only when every reachable judge names it,
the shape the eighth `hakureico` pass showed on 2026-09-08
("the contest asks a minimum ballot weight rather than a majority of its bench").
The rest of the page is what a thin bench makes:
the first paragraph rewritten by the lanes ("This page is used to record those without names,
the victims who lost their lives due to systemic violence against transgender people.
They may not even necessarily be transgender themselves."),
faithful and plainer than the archive's,
the archive's expanded first heading kept where the first pass had rendered the source's,
the fullwidth colon and the `知乎` label kept,
`verify-published` matched at 4402,
front matter the archive's,
`pageAssembly` empty.
Not readiness evidence,
by the 2026-09-08 rule that a page from a bench short of quorum is read for its findings and not for its wording.

THE SIXTH MIO STOPS INCOMPLETE AT SLICE 3 (12:33 UTC,
33.5 minutes,
`~/temp/agent/mio6-20260909`):
`the standing text failed the deterministic publication rule and the consolidation left nothing valid to ship
(slate-declined-standing)`.
The archive's slice 3 is the paragraph "She suffered from major depression ..." and a bulleted list of three
things she did that year;
the source's is the paragraph alone,
ending "还曾：" with the following sections as the list.
The list is a kind the source lacks,
so under the block floor's bound (class ten) the page is the floor and a candidate must carry it.
The repair lane shipped the slice unchanged (`select 3/7` short of quorum),
the contest chose the translate lane's one-paragraph rendering,
which `fails publication invariants`,
the consolidation withheld it from the slate as ineligible,
three producers of nine proposed,
and the judges could not seat one:
`winner drew only weight 1 across 3 ballots (1 abstentions); keeping the fallback`.
Nothing valid,
so `ConsolidationStandingIneligibleError` stopped the entry under the 2026-09-04 decision.
The archive's own paragraph and list,
valid by construction as the page as it stands,
were not a fallback:
that decision was taken on `luxuanwen3`,
where the incumbent was the ineligible thing,
and it says every exit that would keep the standing throws.
On a full bench the slate's nine producers are told the finding and one of them keeps the list;
whether a thin bench should stop the entry or keep the incumbent for the slice is the second of the questions
the handover records under "The thin-bench questions".

THE QUEUE PAUSES.
The 2026-09-08 rule stands:
no launch on Bedrock alone,
and no launch at all until a meter besides Bedrock's moves.
The third `hulicaijia` (pid 712107) runs on to its tally
(71 repair and 70 translate slices computed by 12:33 UTC,
46 `winner short of the minimum vote weight`,
56 slices shipping unchanged)
and is read for its shapes and its findings,
not its wording.
When OpenRouter moves,
`noname` relaunches first,
for the heading;
then `Mio` and `hulicaijia`.
Two questions for the owner,
recorded in the handover under "The thin-bench questions":
whether a deciding bench short of quorum should stop the entry (the fifteenth class widened),
seat winners by a share of what is reachable,
or be widened on Bedrock by measuring `gemma-4-31b` for the judge seats;
and whether an ineligible standing over a valid incumbent should keep the incumbent for that slice instead of
stopping the entry.

THE THIRD HULICAIJIA ENDS ERROR AT THE HEADING FLOOR (13:04 UTC,
65.7 minutes,
`~/temp/agent/hulicaijia3-20260909`,
51 payment refusals):
`entry hulicaijia would render 13 distinct source heading(s) as 12 distinct page heading(s)`.
The source and the archive each carry thirteen distinct headings.
The translate lane's cached renderings are thirteen distinct too,
but one of them is `## 相遇` as `## Soulmate`,
the archive's rendering of the neighbouring `## 初识`
(the archive has `## Meeting` for `## 相遇`),
so the page composed per slice carried `## Soulmate` twice and `heading-distinctness.ts` refused it,
as it was built to on 2026-09-06.
A heading that is another section's name is what one Bedrock translator and three judges produced;
no page shipped.
The entry's two in-process retries stopped in 5.9 s and 0.4 s under the fifteenth class:
`writing bench unreachable: editors 0 of 3 reachable, floor 2; refiners 0 of 3 reachable, floor 2`,
since no editor or refiner has a Bedrock seat.
Nothing runs now.
The ruby,
the Sakura line and the footnote are read on a full-bench pass.

## The owner asks where 200 USD went, and six levers land, 2026-09-09, 16:50 UTC

The owner,
after the payment refusal of 12:22 UTC:
"What the hell is going on,
I just topped up 200USD to openrouter yesterday and today it's already dry?
We're being token inefficient at an unsustainable level."
Then,
after a question that should not have been asked:
"Of course we need to do everything in our power to NOT bleed.
Of course we need to always fix and relaunch.
I have topped up OpenRouter one final time.
Mercury 2.5 is out and approved."
Then "No,
there are obviously more token efficiency fixes",
and "Do not ignore the fact that if you make whatever sent to Hyper/Synthetic more efficient,
Hyper/Synthetic would last longer and therefore fewer requests would need to fallback to openrouter in the first place."

### Where the 200 USD went

Between the top-up of 2026-09-08 11:27 UTC (200.01 on the meter) and the refusal of 2026-09-09 12:22 UTC (1.51),
the meter moved 199.92 USD.
The `SPEND` lines of every pass log in that window sum to 141.62.
The rest,
58.30,
was never logged:
streams the rounds abandoned 120 s after quorum,
2,135 of them since the top-up,
1,088 of them Qwen3.8-27B (3.0 GB of raw stream) in select,
critic,
panel and lane-contest seats,
served by CoreWeave,
Parasail and Phala,
which OpenRouter's streaming page lists neither among the providers that stop billing on a cancelled stream nor among
those that do not,
while Alibaba and MiniMax are listed as billing the whole response.
Reckoned at each model's raw characters per completion token (Qwen 377,
deepseek-v4-pro 386,
glm-5.3 103,
the others near 135),
the abandoned streams come to about 58 USD (Qwen about 24,
glm-5.3 about 18,
prompts about 5),
which closes the gap.
Two more findings from the same logs:
69.70 USD of the logged spend went to 17 passes killed before their page was read,
and `deepseek-v4-pro-0813` cost 1.87 times its listing price
(Parasail 1.85 and CoreWeave 2.11 over 3,894 calls;
55.14 USD paid against 29.56 at listing),
`deepseek-v4-flash-0731` 1.43 and `glm-5.3-flash` 1.52,
because the default load balancing spread calls over endpoints priced at twice the cheapest.

### The six levers, in commit order

1. `037d1f650`:
   every OpenRouter call carries `max_tokens` at a measured ceiling
   (the 99th percentile of `completion_tokens` over that model's completed OpenRouter calls since the top-up),
   and Qwen3.8-27B and glm-5.3 leave the OpenRouter catalog
   (24 percent of Qwen's calls there were abandoned and billed to the end;
   glm-5.3 was the second bleeder and Hyper serves both).
2. `4f87555fc`:
   every abandoned OpenRouter stream writes a `SPEND ... estimated=abandoned` line reckoned from the characters
   delivered before the cut,
   so the meter and the log can be reconciled without a second reckoning.
3. `1fe7ca2fe`:
   `provider.sort: 'price'` on every OpenRouter request
   (the routing page says sorting disables load balancing and picks the cheapest;
   zero data retention,
   `require_parameters` and the ignore list still apply),
   and the `SPEND` line carries `cached=N` off `usage.prompt_tokens_details.cached_tokens`,
   since DeepSeek's prompt caching is automatic and reads at a tenth of the input price.
   The third `noname` pass shows the routing at work:
   `deepseek-v4-pro-0813` on NextBit,
   `deepseek-v4-flash-0731` on DeepInfra,
   and `cached=512` on the second minimax call of the pass.
4. `b71a55385`,
   the fan-out window:
   every `gatherStageVoices` round asks quorum plus one seat,
   from a bench rotated deterministically by the prompt,
   and the rest only when a voice is lost;
   quorum is unchanged.
   On the first `noname` pass 1,430 seats were asked across 229 rounds where quorum needed about 810,
   and every surplus seat waited out a grace window and,
   on the per-token provider,
   was generated and billed to the end.
   Three consequences were decided with it:
   a fixture scripting every seat asks for the whole bench through a `fanOut` knob on the select,
   critic,
   chunk-critic and coverage stages;
   the coverage verdict takes its majority over the seats the gather asked,
   since a seat the window spared was never silent;
   and a judge bench whose window could not carry a unanimous self-written slate on self-votes alone
   (four seats:
   three halves fall short of the minimum of 2) asks the whole bench,
   which is one seat more (`candidate-select-fanout.ts`).
5. `78ea8c8c7`,
   the cap on every provider:
   the table moves to `completion-cap.ts` keyed by roster id and every client sends it,
   Synthetic (a weekly token allowance),
   Bedrock (a credit never topped up),
   Hyper (under its own per-model ceiling) and OpenRouter.
   Measured over every `SPEND` line in the pass logs (142,437 completed calls across the four providers;
   `~/temp/agent/cap-measure-20260909.txt` holds the table):
   each cap is the highest 99th percentile any provider with at least 100 calls of the model recorded,
   floored at the pooled 90th (3,831),
   and every cap cuts under one percent of that model's completed calls
   (`minimax-m3` the most,
   274 of 34,018,
   whose Hyper replies ran to the 32,000 ceiling that provider already enforced).
   The caps:
   GLM-5.3-Flash 18,316;
   Qwen3.8-27B 20,894;
   Kimi-K3 10,921;
   gpt-oss-120b 3,831;
   minimax-m3 10,822;
   gemma-4-26b-a4b-it 3,831;
   deepseek-v4-pro 9,128;
   deepseek-v4-flash 16,543;
   glm-5.3 22,067;
   gemma-4-e2b 3,831;
   gemma-4-31b 8,194.
6. `33a023445`,
   the window for the six stages that read their own round
   (the lane contest,
   section and block pairing,
   the consolidation gate,
   the naturalness review and the polish gate;
   356 seats on the first `noname` pass for quorums of about half):
   `runWindowedRounds` asks quorum plus one seat,
   the rest only when a voice is lost,
   up to the same three retry rounds the other stages have and these never had,
   and returns one outcome per seat asked in roster order.
   The naturalness confirmation now challenges exactly the seats the discovery asked,
   at the discovery's quorum,
   since the artifact reads one requested roster across both readings and a challenge put to the reviewers who
   approved is what a confirmation is.
   A provider down for every seat now costs such a stage up to four rounds of the client's five transient attempts,
   as it already costs the other sixteen stages.

Not done,
and next:
Mercury 2.5 (`inception/mercury-2.5`,
0.04 and 0.15 USD per million,
one endpoint,
approved by the owner) as an OpenRouter-only roster id,
seated for judge seats by the fidelity probe and for writer seats by the producer calibration,
so the cheapest seat on the per-token provider carries the judging that today falls on deepseek-v4-pro at 9.128
thousand tokens of ceiling and 1.74 USD per million out.

### The third noname launch, 16:45 UTC

The queue resumed on `33a023445` with the meters reading synthetic wet at 2 percent of the week,
bedrock wet at 186.68 USD,
hyper dry,
openrouter wet at 273.99 USD remaining after the owner's final top-up.
The pass runs from a frozen copy of the built `dist`
(`package/module/translation-repair/node_modules/.frozen-dist-33a023445`,
which the scratch launcher takes as its third argument),
so a rebuild for the Mercury seating cannot change the pass under way;
the kill-and-relaunch rule still applies to the build the pass carries,
not to the files on disk.
What the pass is read for,
besides the `## 简介` heading above the first paragraph:
the seats each round asks against its bench (`retry round N asking X of Y pending voices` where a voice was lost),
the `cached=` counts on DeepSeek calls,
the endpoints the price sort picks,
and whether any `estimated=abandoned` line appears at all now that every stream is capped and windowed.

### Mercury 2.5 judges 14 of 14, 17:00 UTC

`judge-fidelity-probe --cap 48 --candidates inception/mercury-2.5 --candidates-alone` on `3224ff347`
over a throwaway runs dir holding the three settled artifacts on disk
(gqt,
hakureico,
noname):
three entries walked,
28 rows,
fourteen distinct questions
(six deletions,
six insertions,
two alterations;
`gqt/3` states no number the alteration fixture can move),
14 asks,
14 usable,
75 s of wall clock.
The roster verdict reads `declined` on every row,
as it must for one judge under the minimum weight of 2;
the per-judge tally (scratch `fidelity-tally.mjs`) is the reading:

- `inception/mercury-2.5`:
  14 of 14 chose the complete text.
  Deletion 6 of 6,
  insertion 6 of 6,
  alteration 2 of 2;
  no damaged pick,
  no decline;
  position two on 7 of 14.

By the rule written on 2026-09-07,
a candidate at the maximum meets both clauses whatever the seated median reads,
so it joined the judge seats at once (`fcc8ca197`):
critic,
panel and judge in both lanes,
the late bench and the slate.
The seated roster's reading of the same fourteen is bought beside it
(`judge-fidelity-probe --cap 48` over a second copy of the same artifacts,
detached,
pid 875777,
log `~/temp/agent/probe-seated-20260909.log`)
and recorded here when it lands,
for the record's sake rather than the decision's.
Its writing seats wait on
`producer-calibrate 40 --candidates inception/mercury-2.5`
(detached at 17:06 UTC,
pid 875801,
frozen build `3224ff347`,
log `~/temp/agent/producer-calibrate-mercury-20260909.log`,
about three hours by the 2026-09-08 precedent),
read by the scratch `read-standing.mjs` against the pooled null.

## The owner answers the three recorded questions, 2026-09-09, 17:25 UTC

Asked "What other things do you need to ask me?
Ask me now or decide they're not necessarily design decisions",
the three questions the handover recorded were put in one form
("The thin-bench questions" and "The glossary question"),
each with its options,
pros and cons and the ranking.
Everything else pending
(Mercury's writing seats,
its completion cap,
the prompt-volume lever,
the launch order)
is decided by measurement or already answered,
so nothing else was asked.
The answers:

- A deciding bench short of quorum:
  a share of the reachable weight,
  against the recommended stop INCOMPLETE.
  Rule and rejected options in `doc/decision/translation-repair-short-bench-share.md`:
  the minimum weight scales as `MIN_SELECTION_WEIGHT` times reachable over quorum,
  two ballots the floor,
  benches at quorum unchanged.
- An ineligible standing over a valid incumbent:
  keep the incumbent for the slice,
  as recommended.
  Addendum in `doc/decision/translation-repair-ineligible-standing.md`.
- The glossary:
  both the corpus glossary file and the archive's rendering as a candidate the judges must weigh.
  Rule in `doc/decision/translation-repair-community-glossary.md`:
  the glossary rides in the identity context,
  and a deterministic block on the judge sheets names each candidate lacking the community rendering.

Built in that order,
each guard shown to fail first:
`db5927630` (the share;
the gather names the seats the router refused,
the select minimum scales,
ballot counting moved to `candidate-select-count.ts`),
`1463cd359` (the incumbent;
the artifact's shipped kind `incumbent` was needed so the page writes the incumbent rather than the lane's
refused wording,
which a bare "unchanged" would have done),
`b7a0b4f5f` (the glossary;
the editor selections' task and criteria moved to `editor-selection-sheet.ts`).
The running third `noname` carries none of them (frozen `33a023445`);
the next launch carries all three.

The seated roster's reading of the same fourteen landed at 17:18 UTC
(pid 875777,
`judge-fidelity-probe --cap 48` over a second copy of the same three artifacts,
28 rows,
roster verdict clean on 20 and declined on 8,
record `~/temp/agent/probe-seated-20260909/judge-fidelity-probe/`),
with Synthetic at the bottom of its week and Hyper dry,
so `glm-5.3` was lost on every ask
(`OPENROUTER_DROPPED_SEATS` holds it since `037d1f650` and no other provider served it)
and the other seats declined often.
Complete-text picks over the distinct questions each judge answered:

- `hf:moonshotai/Kimi-K3` 8 of 8,
  no damaged pick.
- `google.gemma-4-e2b` 9 of 11,
  2 damaged picks.
- `deepseek-v4-pro-0813` 6 of 9.
- `hf:zai-org/GLM-5.3-Flash` 5 of 9.
- `hf:Qwen/Qwen3.8-27B` 4 of 10.
- `deepseek-v4-flash-0731` 4 of 10.
- `minimax-m3` 3 of 6,
  1 damaged pick.
- `hf:openai/gpt-oss-120b` 2 of 6.
- `gemma-4-26b-a4b-it` 1 of 6.
- `glm-5.3` no usable ask.

The seated median lies between 4 of 10 and 5 of 9;
the most damage-prone seated judge chose the damaged text twice.
Mercury's 14 of 14 with no damaged pick clears both clauses by measurement,
not only by the maximum argument.

## The third noname page is read and the Kimi-K3 leak closes, 2026-09-09, 18:40 UTC

THE PASS:
launched 16:45 UTC on frozen `33a023445`,
SETTLED at 18:08 UTC after 83 minutes,
13 slices,
`pageChanged=9`,
Synthetic wet at 2 percent of the week at launch and dry from 17:25 UTC inside the translate lane,
Hyper dry throughout,
Bedrock and OpenRouter wet.
Meters:
OpenRouter 273.99 to 271.68 USD,
Bedrock 186.68 to 186.49.

THE LEVERS,
measured with the scratch `pass-spend.mjs` beside the two earlier noname passes of the day:

- Seats asked 893 over 900 calls,
  21 retry rounds;
  the second noname asked 1,999 seats over 715 calls with 222 retry rounds,
  the first 1,322 over 1,355.
  Select rounds asked 318 seats over 66 rounds (4.8 a round;
  the first pass asked 7.1).
- Cached prompt tokens 602,368 across the OpenRouter DeepSeek and MiniMax calls (`cached=` on the SPEND line),
  none before the price routing.
- Abandoned streams 28 of 900,
  reckoned at 0.44 USD:
  23 of them DeepSeek-flash reasoning streams at 5,000 prompt and 7,500 to 14,500 completion tokens
  (0.05 USD in all,
  0.10 USD for every DeepSeek-flash call of the pass),
  two of them Kimi-K3 on OpenRouter at 0.28 and 0.08 USD.
- OpenRouter spend 1.87 USD against the second noname's 1.72 and the first's 5.04,
  but 1.14 of it is 22 Kimi-K3 calls the router sent to OpenRouter after Synthetic dried mid-phase,
  at 3 and 15 USD per million (Makora,
  Morph,
  DeepInfra,
  Together and Sail Research endpoints,
  the price sort finding nothing cheaper);
  the other 382 OpenRouter calls cost 0.73 USD.
  The seat reader withholds Kimi-K3 at phase start when OpenRouter would serve it (2026-09-03) and the
  router did not honour that mid-phase;
  closed in `19b6043a3` (the reach says OpenRouter does not serve a withheld model;
  addendum of `doc/decision/translation-repair-provider-aware-judge-seat.md`).
- `glm-5.3` asked 53 times and threw 53 (Hyper dry,
  dropped from OpenRouter since `037d1f650`):
  a dark seat every day Hyper's limit is spent,
  which the select minimum of `db5927630` now counts as unreachable rather than as a lost voice.
  Qwen3.8-27B threw 52 of 125 after Synthetic dried,
  for the same reason.

THE PAGE,
by the seven steps:
`verify-published` 1 of 1 at the length the artifact implies;
`DESTINATIONS noname source=3 page=3 dropped=0`;
front matter as the archive has it;
the fourth `PhotoScroll` path repaired (the archive wrote `'${path} /photos/kuan4.webp'` with a space);
the `## Introduction` heading above the first paragraph,
which the twenty-third class had dropped on the first two passes;
straight quotes 1 and curly 11 on page and archive alike,
ellipses 0 and three-dot 1 alike;
two em dashes where the archive has none,
one carrying the source's own `ーー` in the memorial tweet and one a comma,
inside the archive's convention (38 of the 92 archive pages use one).
The two passages the archive paraphrased read faithful on the page:
"They may not even have been transgender themselves" for
"ta 们或许甚至未必是跨性别本身",
and "Nine days later,
on March 18" where the archive added a year the source does not state.
Refusal vocabulary in the log:
three slices whose standing lacked contest endorsement and shipped with the finding (2,
3 and 12);
no `fails the deterministic`,
no `StandingIneligible`,
no `mdx-downgraded`.
Contest:
8 slices to the translate lane,
4 settled neither;
consolidation:
one slice consolidated (10),
the rest kept their standing.

## The ninth Mio launches on the structural-withdrawal proof, 2026-09-09, 23:49 UTC

Frozen build `5cb66ffaa`,
pass pid `1425902`,
runs `~/temp/agent/Mio9-20260909`,
log `~/temp/agent/Mio9-20260909.log`.
The terminal-only managed supervisor is `proc_5d73`.
Source is held still while the pass runs;
any necessary source change requires kill and relaunch.

Prelaunch and launch balances:
OpenRouter 269.40 USD,
Bedrock 186.06 USD,
Synthetic weekly zero,
Hyper zero.
The meter sample is `~/temp/agent/Mio9-prelaunch-meters-20260909.log`;
the required daily-cost helper output is `~/temp/agent/Mio9-costs-before-20260909.out`.
The launch confirms overlap 4,
built-in writer grace and Kimi withheld.

Read both screenshot translations and the linked closing poem before advancing to `hulicaijia`.
The guard's `assembly-structure-single-withdrawal` finding proves the selective branch was exercised;
absence of that finding is not failure when no selected replacement broke the page.
Either way,
read the actual complete page and all ordinary verification checks.
The new build has passed the full suite with last line `unit exit 0`,
types,
oxlint and edited markdown.

## Mio8 stops on a link lost by blanket assembly withdrawal, 2026-09-09, 23:46 UTC

The eighth `Mio` on frozen `5b228736b` stopped INCOMPLETE at 23:32 UTC after 5,673,650 ms:
`entry Mio would drop 1 source destination(s)`.
No page or artifact was published.
The daily-cost helper ran after the terminal state;
its output is `~/temp/agent/Mio8-costs-after-20260909.out`.
Summing actual `SPEND` lines by provider gives OpenRouter 0.615210094 USD over 550 logged calls,
Bedrock 0.1817423025 USD over 362,
and Synthetic 159 calls with no per-call price.
Abandoned-stream estimates remain ceilings,
not invoices.
Meters moved from OpenRouter 270.03 to 269.41 USD and Bedrock 186.24 to 186.06 USD.
Synthetic returned during the pass and ended at 0.3868101893939394 percent of its rolling week;
Hyper stayed at zero.

The missing destination is the Bilibili link in the closing poem,
not a failed translation of that poem.
The cache's slice 17 contains the chosen complete linked poem.
Slice 16 contains a selected malformed `PhotoScroll` with `photos=[` instead of `photos={[`,
retained after its author's unresolved repair.
At 22:46:27 UTC the translate assembly logged `assembly-structure-unattributable mdx-downgraded`,
then `assembly-withdrew-every-replacement (15 slices)`.
The valid poem went with the malformed component;
repair had no archive text for the poem,
so the final page could not carry its destination.
This is class twenty-seven.
The publisher refused the defective result as designed.

The author-defense policy in `translate-repair-wire.ts` is deliberate,
so it is not changed.
The missing proof is at assembly:
absence of a footnote identifier does not establish that no replacement can be identified by a concrete trial.
Guard `3e1fec4be` reproduces the failure with a shared component and a valid source-only linked quotation:
the prior build withdraws both indices `[0, 1]` where only the component must be withdrawn.

Fix `ad6d506a6` tests each single withdrawal by splicing the whole document,
requiring no introduced structural or footnote defect against the archive,
and accepting the first successful trial in supplied replacement order.
It runs only when ordinary footnote attribution found no culprit and a structural regression exists.
No model call is added.
When no single trial repairs the document,
the existing blanket fallback remains;
a lone replacement keeps that same diagnostic too.
`dfc02d538` extracts the shared regression readers;
`48ef99c42` formats the proof;
`5cb66ffaa` covers the footnote condition.
The tests also cover no single withdrawal succeeding and valid container halves spanning slices,
so an isolated-slice parser cannot stand in for whole-page proof.

Build and types pass.
Oxlint reports zero warnings and errors.
The full suite ends `unit exit 0` (`~/temp/agent/class27-unit-20260909.out`),
including the guard that failed first.
The next action is a fresh frozen-build `Mio` relaunch,
not `hulicaijia`.
Class twenty-five still needs a published page read;
this interrupted pass cannot close it at the user boundary.

## The open threads are checked against code and recorded measurements, 2026-09-09, 22:00 UTC

No source changed while `Mio` ran.
The glossary was not empty at handoff:
`git show 2cfad6236:package/module/translation-repair/src/community-glossary.ts` (moved on 2026-10-18 to
`package/translation-repair/glossary/src/community-glossary.ts`) contains both seed entries,
and importing frozen `5b228736b` and calling `communityTermLines` against the pinned source pages returns
both entries for `yuki418330012` and an empty list for `Mio`.
The two named terms are absent from Mio's source,
which explains its absent glossary heading without inventing a missing-curation blocker.
The glossary decision and README had the right state;
the handover and seventh-Mio reading are corrected.
Further curation remains the owner's,
but drafting those same seeds would duplicate existing code.

The "first seat ever above the pooled null" claim also contradicts the seating record:
its 2026-09-08 producer-calibration addendum records Gemma at z +4.14 and Qwen at +5.25,
both above that calibration's 2.81 threshold.
The latest Gemma result is z +6.39.
No special weighting rule is adopted here;
seat breadth or weighting based on upward producer evidence remains an owner design question,
not a reason to alter the running pass.

## The eighth Mio launches with picture support before archive review, 2026-09-09, 21:58 UTC

The class-twenty-five build is frozen at
`package/module/translation-repair/node_modules/.frozen-dist-5b228736b`.
The canonical launcher started `Mio` alone at pid `1178354`,
runs `~/temp/agent/Mio8-20260909`,
log `~/temp/agent/Mio8-20260909.log`.
The process-managed supervisor (`proc_8ef1`) watches pid lifetime and emits only a terminal notification.
Its scratch scripts are `~/temp/agent/supervise-translation-pass-20260909.mjs` and
`~/temp/agent/watch-translation-pass-20260909.mjs`.
A terminal notification is not a settlement verdict;
the log and page decide that.

Launch meters:
OpenRouter 270.03 USD,
Bedrock 186.24 USD,
Synthetic weekly zero and five-hour 2750/2750,
Hyper zero.
The log confirms the unchanged overlap and writer-grace defaults,
Kimi withheld,
and archive front-matter authority.
No source changes are planned while the pass runs;
a necessary source change requires killing pid `1178354` and relaunching from a fresh frozen build.

Read for:
the `photo6.webp` and `photo7.webp` chat translation retained through preparation and on the shipped page;
the `photo3.webp` translation retained;
completed pictures bought once across preparation and lanes;
the horizontal rules,
links,
poem and front matter;
then every remaining seven-step and three-check item.
Run the daily-cost helper after the terminal state,
read actual pass spend by provider rather than its mixed-provider total label,
and record the reading before `hulicaijia` starts.
The remaining queue is `hulicaijia`,
`shi_Yumiaoya`,
`yingying`,
`XingZ60`.

## Picture support reaches archive review before removal, 2026-09-09, 21:46 UTC

The takeover confirmed the clean documents tip `2cfad6236`,
code tip `40aba2fdb`,
corpus pin `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
and no running pass.
The canonical files live in the translation-repair worktree,
not the main repository.

The class-twenty-five guard (`aba89b50c`) scripts an archive review that removes a greeting unless its
source sheet contains the corroborated Chinese picture text.
On the pre-fix build,
`pass-readiness-boundaries.unit.test.ts` failed because the archive block was removed.
The section-context guard (`d62c76b4b`) failed because the transcription was absent.
It also checks unrelated pictures,
textless and unavailable evidence,
deduplication,
each corroborating reader's wording and target-only sections.

The handoff's cache assumption did not survive source inspection:
`corpus-run/slice-cache-namespace.ts` writes persisted readings to disk but leaves its open `resumed` map unchanged.
The reuse guard (`a72361aa6`) failed with four reader calls against the first reading's measured two.
Completed evidence is therefore retained by an entry-scoped reader,
not inferred from disk persistence.
A newly exposed reference is still read even when its slice also names an already completed picture.
The same pinned entry is the lifetime boundary;
failed readings are not retained as completed evidence,
and cross-entry cache keys still include picture bytes and reader roster.

Implementation:
`f6cd6e5e7` supplies picture support before archive review and limits it to references in the aligned source section;
`07a99e23a` binds that reader through the real pass and retains completed evidence for the later picture phase.
The review prompt explicitly permits exact source anchors from corroborated transcriptions.
No authority or roster policy changed.

The first full suite passed all three new guards and ended with `unit exit 0`
(`~/temp/agent/translation-repair-class25-20260909/class25-unit-20260909.log`).
Type-checking exposed incomplete test fixtures,
corrected in `009eacc3e`.
Oxlint found formatting findings and the pass-entry module one code line over its limit;
these remain work in progress,
not waived.
An independent review requested a guard through `settleEntry` proving the shared callback wiring,
which is being added before launch.
No provider generation has been purchased during this work.

Verification finished at 21:55 UTC:
`865859305` extracts cache retirement into the cache module without changing its post-publication boundary;
`76d27dc31` resolves formatting;
`5b228736b` covers textless and unavailable prior readings.
The first integration fixture did not reach archive review,
so its failure was not accepted as evidence.
`9e7df9b40` minimizes it to one shared component and one unclaimed archive block.
That fixture fails against frozen `a5e0efc7f` because the review sees no transcription
(`~/temp/agent/translation-repair-class25-20260909/class25-entry-red2-20260909.log`),
and passes against the new build,
checking ordering,
the actual sheets,
one reader invocation and both lane stages.
The final full suite ends `unit exit 0`
(`~/temp/agent/class25-final-unit-20260909.out`);
additional prior-reading branch tests pass;
types pass;
oxlint reports zero warnings and errors.

Prelaunch meters at 21:55 UTC:
OpenRouter 270.03 USD,
Bedrock 186.24 USD,
Synthetic weekly zero and five-hour 2750/2750,
Hyper zero.
The live sampler log is `~/temp/agent/class25-prelaunch-meters-20260909.log`;
the required daily-cost helper ran into `~/temp/agent/class25-prelaunch-costs-20260909.out`.
Its `TOTAL logged OpenRouter cost` label is misleading:
it includes Bedrock and unit-test fixtures from the preceding session,
including `cat/whiskers` at 1.25 USD of simulated calls.
No daily provider-spend conclusion is drawn from that total.
New test logs are kept outside its top-level dated `.log` scan;
pass costs will be read from each provider's rows in the actual pass log.

## The calibration is read, the anchor judge loses a seat, and two classes open, 2026-09-09, 21:00 UTC

THE MERCURY CALIBRATION printed `STANDING over 40 rounds` at 20:02 UTC after 176 minutes
(pid 875801,
frozen `3224ff347`,
1.82 USD on OpenRouter,
every Mercury call itself billed `cost=0` by the Inception endpoint).
`inception/mercury-2.5` read 18 of 101 disinterested ballots,
z -0.43 against a 19.5 percent pooled null,
not separated from it,
so it takes the translator and consolidation seats (`028432713`);
`deepseek-v4-pro-0813` read z -3.18,
across the Bonferroni threshold below the null for the second calibration running,
so it leaves the translator seat (`a5e0efc7f`).
Both addenda are in `doc/decision/translation-repair-roster-seating-2026-09-01.md`.
Translators are eight either way,
so the stage quorum stays 4.

CLASS TWENTY-FOUR,
measured rather than assumed:
the price sort of `037d1f650` moved `deepseek-v4-pro-0813` from Parasail to NextBit and
`deepseek-v4-flash-0731` from Wafer and Inceptron to Sail Research,
and those endpoints serve the models WITHOUT their reasoning when no reasoning parameter is sent,
which the owner's standing instruction forbids sending.
Over the second `noname` pass Parasail reasoned on 57 of 57 deepseek-pro streams at a median of 1,397
completion tokens;
over the third `noname`,
the seventh `Mio` and the calibration,
NextBit reasoned on 0 of 342 at a median of 70 to 107,
with a third of its replies under 50 tokens.
Three fidelity probes over the same fourteen questions answered whether that costs fidelity:

- `deepseek-v4-pro-0813` on NextBit,
  no reasoning:
  11 of 14,
  no damaged pick,
  3 declines,
  0.017 USD.
- `deepseek-v4-pro-0813` on CoreWeave,
  reasoning on every stream:
  4 of 14,
  no damaged pick,
  10 declines,
  0.116 USD.
- `deepseek-v4-flash-0731` on Sail Research,
  no reasoning:
  5 of 14,
  no damaged pick,
  9 declines.

The cheap endpoint reads BETTER on this instrument,
and the expensive one declines two and a half times as often,
so the routing stands and no endpoint was ignored.
The measurement build that would have ignored NextBit and DeepInfra was reverted.

THE SEVENTH `Mio` SETTLED at 20:16 UTC after 116 minutes on frozen `19b6043a3`,
15 slices,
`pageChanged=12`,
1,141 calls,
0.59 USD on OpenRouter and 0.19 on Bedrock,
33 abandoned streams reckoned at 0.08.
The three answers of 17:25 UTC and the reach fix all behaved:
zero Kimi-K3 calls on OpenRouter with `withheld=hf:moonshotai/Kimi-K3` on every dry-Synthetic seat line,
183 seat losses recorded as unreachable rather than as lost voices,
slice 3's ineligible standing replaced by its incumbent
("the incumbent passes it and stands in as the wording the slate may keep,
without contest endorsement"),
slice 14's incumbent failing too and the slice withheld from the slate,
no `select-short-bench` finding,
and no `COMMUNITY TERMS` line.
Correction during takeover:
this was not an empty glossary.
The committed file at `2cfad6236` already contains 自切 and 超天酱,
and a call through frozen `5b228736b` emits both for `yuki418330012` but none for `Mio`,
whose source contains neither term.
The seed was already built as the glossary decision records;
no source change or owner input is needed to make those entries available.

THE PAGE IS NOT CLEAN,
and the defect is CLASS TWENTY-FIVE.
The archive renders the two WeChat screenshots of section one as a 1,900-character `Translation:` blockquote;
the page ships without it.
The archive block review revised it away as
"a translator-added factual insertion not present in the original Chinese text",
which is true of the source TEXT and false of the section,
whose pictures carry exactly that conversation:
the review ran at 18:25 to 18:29 UTC inside preparation,
and the readers transcribed `photo6.webp` and `photo7.webp` at 18:30 UTC,
one phase later.
The second such block (`photo3.webp`,
"Quit my QQ groups") survived only because its section's source names the chat in prose.
Everything else on the page reads faithful:
`DESTINATIONS Mio source=4 page=4 dropped=0`,
front matter as the archive has it,
the horizontal rules in place (three,
against the archive's three),
the bare `space.bilibili.com` credit carried,
the memorial poem rendered where the archive carried only its one-line farewell,
straight quotes 0,
ellipses 3 against the archive's 4,
and ten spaced em dashes against the archive's two
(14 of the 93 archive pages use the spaced form,
68 occurrences in all).

CLASS TWENTY-SIX closed the same evening.
`hulicaijia` launched at 20:23 UTC on frozen `a5e0efc7f`,
Synthetic read wet at 0.04 percent of its week and dried two minutes later,
and the archive block review sized its quorum at 6 of an 11-seat bench,
heard 5,
re-asked the three refused seats over four rounds and threw `provider-unavailable` at 382 seconds:
INCOMPLETE with four seats able to answer.
Fixed in `40aba2fdb` (`stage-reachable-quorum.ts`),
which sizes every gather's quorum on the reachable bench with a two-voice floor and never re-asks a
refused seat;
the addendum is in `doc/decision/translation-repair-short-bench-share.md`.
