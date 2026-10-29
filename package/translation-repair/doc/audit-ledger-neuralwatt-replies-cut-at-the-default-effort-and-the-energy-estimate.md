# Audit ledger: NeuralWatt replies cut at the default effort and the picture energy estimate

Calls to NeuralWatt that ran at the provider's default reasoning effort and were cut at the length cap,
the owner's lift of 2026-10-21 that sets the effort for the pictures and then the text of the GLM seat,
the energy estimate that counted a picture as base64 text,
the energy hold recalibrated from the settled calls of a rotation of entries (B808),
and the hold recalibrated again from the first pass on a build that sends the GLM seat at low effort (B836).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B718: a reply the length limit cut before any answer ended that reader's turn on the picture

Red `5dee1b41f9`,
fixed in `8423f77dc2`.

Found on 2026-10-21 (UTC) in the read of the corpus pass for `XingZ6016`,
and fixed in `w258`.

What the run showed:
the pass stopped incomplete because the entry's one picture had a single reading.
`glm-5.3-flash` on NeuralWatt answered `text:""` with `finishReason:"length"`,
having spent its whole completion cap (18316 tokens) reasoning at NeuralWatt's default effort `max`,
for 8188 prompt tokens.
The reader stage recorded `empty-reply` and went on,
and the second reader alone could not corroborate.

The rule the code broke:
`readPastRefusal` re-asked a refusal up to `REFUSAL_ASK_LIMIT` times and treated every empty reply as final.
An empty reply that ended on `stop` is a fact about the picture or the model.
An empty reply that ended on `length` is a fact about one call's budget,
and says nothing about whether the model can read the picture.

The fix:
`readImageAsset` marks an `empty-reply` whose finish reason was `length` with `cutByLength` and the provider that served it (`servedBy`).
`readPastRefusal` then asks once more.
It sends the ask with `otherThan` set to the provider that cut the reply,
so the router serves another wet provider that carries the same model.
Where the router has none (`NoProviderForModelError`) the ask goes again to the same provider.
Every other failure keeps its own handling.
The line names the picture,
the finish reason,
the provider and the route the second ask takes.
The second ask carries the next entry of `IMAGE_READING_PERSPECTIVES`.
The payload store keys a reply on the model and the message bytes,
so an ask with identical bytes would have replayed the stored empty reply.
A second empty reply stays `empty-reply`,
and an empty reply that ended on `stop` is asked once as before.

Cache versions:
the re-ask changes only the transient `unavailable` verdicts,
which are never resumed (`isResumableReading`),
and readings it produces enter the cache under the same key as any reading.
No key moves.
The audit (`cache-account-audit`) lists the commits among the source commits no account names,
which is the standing state of the package.

Not checked:
whether a second ask recovers on `glm-5.3-flash`.
No provider was called in writing this fix.

Open to the owner's veto:
one re-ask,
not more,
and the next perspective as the changed text.

Recurrence:
`design-commitments.md`,
"A reply the length limit cut is re-asked as cut,
not as off the shape" (ledger P10):
the picture readers now follow the same rule the stages' recovery round already holds.

## B719: a NeuralWatt picture call ran at the provider's default reasoning effort

Red `ae7ba82828`,
fixed in `127a8f6eb2`.
It revises P11 for NeuralWatt only.

Found on 2026-10-21 (UTC) in the same read as B718,
and fixed in `w258`.

What the run showed:
`glm-5.3-flash` is a reasoning-only model on NeuralWatt,
and the body carried no effort,
so the provider applied its default `max`.
That is the cause of the 18316-token reasoning run in B718.

What the documents say
(fetched 2026-10-21 from `docs.neuralwatt.com/api/chat-completions.md`,
`api/models.md` and `GET /v1/models`):
the model's reasoning is mandatory,
`supported_efforts` are `max`,
`high` and `low`,
`default_effort` is `max`,
and `reasoning_effort` is a documented string parameter.
`thinking_token_budget` is documented too,
but its support table (2026-08-26) does not name this model,
so it is not used.
NeuralWatt documents no image token accounting.

The owner's ruling of 2026-10-21 lifts the ban on passing parameters for NeuralWatt only
(two sentences of one message,
the dash between them left out here):
"NeuralWatt is a special case of all the providers:
It implements what it documents correctly,
almost always"
and
"I'm lifting the ban on passing params,
but only for NeuralWatt."
The standing rule of 2026-08-25 holds for every other provider and for NeuralWatt text calls.

The fix:
the type that makes a card a NeuralWatt card carries `pictureReasoningEffort` (`low`,
`high` or `max`),
and the GLM seat's NeuralWatt side names `low`.
The NeuralWatt client sends `reasoning_effort` only on a call that carries a picture,
read off the NeuralWatt card side.
No other provider's card type has the field,
so no other client can send it.
`neuralwatt-picture-effort.unit.test.ts` pins the value on the picture call,
none on a text call to the same seat,
none on a picture call to a seat whose card names none,
and that the only card sides naming an effort are NeuralWatt's.
`owner-body-rules.unit.test.ts` now sends a picture through the Synthetic,
Hyper,
OpenRouter and Bedrock clients and expects no forbidden key in any body.

Cache versions:
a reading made at `low` differs from a reading made at `max`,
but a stored reading is not asked again,
and the key names the bytes,
roster,
instruction,
perspectives and threshold,
none of which moves.
New readings are produced under the new effort.
No key moves.

Not measured:
neither `low` nor `max` has a measured picture-reading quality on this seat.
The first keyed run at `low` is the measurement.

Open to the owner's veto:
`low`,
not `high`.

Widened on 2026-10-21 (UTC) by B807:
the lift covers the NeuralWatt text calls of the same seat too,
and the standing rule of 2026-08-25 now holds for every other provider only,
where this entry's ruling paragraph said "and for NeuralWatt text calls".

## B720: the NeuralWatt energy estimate counted a picture as base64 text

Red `7dfd70e7d5`,
fixed in `5d6c111206`.

Found on 2026-10-21 (UTC) in the same read as B718,
and fixed in `w258`.

What the run showed:
a 1271156-byte picture (2400 by 2616) made a 1.7 million byte body,
which the estimate read as 423985 prompt tokens and held at the top bucket,
0.9089 Wh for `glm-5.3-flash`.
The provider reported 8188 prompt tokens,
and the call measured 0.2311 Wh.
Two 576 by 1280 pictures read for 1156 reported prompt tokens each against 26957 and 32466 estimated.

The fix:
`promptTokensOfRequest` takes each inline picture out of the text bytes and counts it by its patches,
round(width / 28) times round(height / 28),
read off the file header
(`imagePixelSizeOf`:
WebP lossy,
lossless and extended,
PNG and JPEG).
2400 by 2616 is 86 by 93,
7998 tokens,
and 7998 plus 190 tokens of text and framing is the reported 8188.
576 by 1280 is 21 by 46,
966,
and 966 plus 190 is the reported 1156.
A picture whose size cannot be read,
or that is not inline,
is held at 8192 tokens,
a little above the largest count reported.
The bucket,
the margin (`NEURALWATT_ENERGY_MARGIN`,
one) and the settlement that replaces the estimate with the measured energy are untouched.

Limits:
the 28 pixel grid and the 190 are a fit to two pictures,
the second the check of the first,
and a guess for other sizes and for the DeepSeek seat,
which has never read a picture here.
NeuralWatt documents no image accounting.
The estimate only sizes the hold a call has in flight.

Cache versions:
the estimate feeds the in-flight hold and the ledger line only.
No cached stage result reads it.
No key moves.

Not done:
`openrouter-abandoned-spend.ts` estimates an OpenRouter body by the same bytes over four,
and a picture there has the same overcount.
The owner's ruling named NeuralWatt.

## B807: a NeuralWatt text call ran at the provider's default reasoning effort

Status:
fixed on 2026-10-21 (UTC),
red `b5fd3e9a3bd0ff7794ede7b1475b33167b69e891`,
fix `109f1b7a7eb927191776a92de8d3df151483ce1c`,
cache versions none.
Found in the read of the corpus pass for `zheermao22` (class K1) and fixed in `w263`.
It widens B719,
which built the owner's lift for pictures only.

What the run showed:
four `glm-5.3-flash` calls on NeuralWatt ran to the 18316-token cap with 0 content characters
(`zheermao22` log lines 514,
533,
1983 and 2209,
at 21:17:32 UTC,
21:20:36 UTC,
21:37:15 and 21:40:06 UTC),
12.0174 Wh,
29 percent of the run's NeuralWatt energy.
The Hyper re-ask of each ran to its own cap three times of four,
and two recovery rounds waited the full writer grace (180 s and 175 s) and heard nothing.
The same call on the letter block capped in `zheermao20` and `zheermao21`,
so it is the third run in a row.

What the code did:
`neuralwatt-client.ts` took the card's `pictureReasoningEffort` only when the request carried a picture,
and sent `reasoning_effort` only then.
B719 had read the owner's two sentences of 2026-10-21
("NeuralWatt is a special case of all the providers:
It implements what it documents correctly,
almost always"
and
"I'm lifting the ban on passing params,
but only for NeuralWatt")
as covering the call kind that had failed that day.
The words name the provider,
not the kind of call.

What was measured,
keyless,
from the `SPEND provider=neuralwatt` lines of every pass log in the agent directory except `XingZ6017`,
which was still being written when this was counted
(`~/temp/agent/tr-fan/w263-scratch/nw-spend2.ts`,
outside the repository;
a picture read is left out by its `read photo` line right after the spend line;
the cap of each model is `model-cards.ts`):

- `glm-5.3-flash`,
  288 text calls at the default effort `max`.
  Cap 18316.
  Capped:
  11 (3.8 percent),
  7 of them on prompts of 11509 to 13908 tokens (the size of the letter block's review,
  which the reads of `zheermao20` to `zheermao22` name) and 4 on prompts of 7064 to 9451 tokens,
  and none a picture (the capped call of `XingZ6016` is the picture of B718,
  8188 prompt tokens,
  not counted).
  Uncapped,
  277 calls:
  p50 2125,
  p75 3950,
  p90 7573,
  p95 10778,
  p99 15731,
  largest 18262.
  Of the uncapped,
  9 passed 12000,
  5 passed 14000 and 2 passed 16000.
  Of the 31 calls past 9158 (half the cap),
  11 capped.
- `deepseek-v4.1-flash`,
  819 text calls at its default effort `high`.
  Cap 13082.
  Capped:
  12 (1.5 percent).
  Uncapped,
  807 calls:
  p50 802,
  p90 2840,
  p99 9121,
  largest 11591.

The choice:
`low` on the GLM seat's text calls.
The brief's test is whether the uncapped calls sit well under the cap or crowd it.
The bulk sits well under (p90 at 41 percent),
but the tail does not:
p99 is at 86 percent of the cap,
the largest uncapped call is 54 tokens short of it,
and a call that reaches half the cap capped in 11 of 31 cases.
`high` is the other candidate.
DeepSeek runs at `high` by default and capped on 1.5 percent of its calls,
so `high` is not shown to cure the runaway.
No measurement exists at `high` or at `low` for text on this seat;
the first keyed run at `low` is the measurement,
and a shallower effort may cost editing quality that the cap-outs never showed.

DeepSeek V4.1 Flash:
no effort is sent.
The repo records its default (`high`,
`neuralwatt-catalog.ts`) and that reasoning is on by default
(`doc/decision/translation-repair-neuralwatt-provider.md`),
and records no `supported_efforts` list for it.
B719 lists the supported efforts for GLM-5.3-Flash alone.
Nothing the repo holds documents a value to send.

The fix:
`NeuralWattCard` carries `textReasoningEffort` beside `pictureReasoningEffort`,
the catalog row copies it,
and the client sends the picture effort for a request that carries a picture and the text effort otherwise.
Only the GLM seat's NeuralWatt side names one.
No other provider's card type has the field.
`neuralwatt-text-effort.unit.test.ts` pins `low` on a plain and on a schema call to the GLM seat,
none on the DeepSeek seat,
and that the only card sides naming a text effort are NeuralWatt's GLM side,
at a level the listing documents.
`neuralwatt-client.unit.test.ts` pins the body of both seats,
and `owner-body-rules.unit.test.ts` still expects no forbidden key in the body of any other provider.
The red cases failed with `expected undefined to equal 'low'` and an empty list of naming card sides.

The retry at `low` before the next-seat re-ask was checked and not built:
B718 re-asks a picture reading once with the next perspective,
which changes the message bytes.
A text call has no such text to change.
The payload store keys a reply on the model and the message bytes alone
(`prompt-uniqueness-client.ts`),
so a second ask of the same bytes returns the stored empty reply,
and the nudged re-ask goes to another provider on purpose
(`nudged-reask.ts`:
"the same bytes asked again would be the same sample").
With `low` sent from the first ask,
a retry at `low` would repeat the first ask.

Cache versions:
none moves.
No slice-cache key hashes a request parameter:
the keys hash the version,
the shape of the run and the texts,
and the reasoning for the call deadline (`perCallTimeoutMs`,
"changes how long a voice has to answer,
not what it is asked") holds for an effort.
The payload store sits under the run's own runs directory (`runs-layout.ts`,
`PROMPT_PAYLOADS_DIR`),
so a new run starts with none,
and a resume of a run begun before the change reads replies already paid for at the old effort,
which is wanted.
This is the reading B719 took for the picture effort ("No key moves").

Limits:
the energy matrix the meter estimates from was drawn at the default efforts,
so a call at `low` holds more than it uses;
the settlement replaces the hold with the measured energy.
The hold is larger than needed,
never smaller.

Open to the owner's veto:
`low`,
not `high`,
for text.
Changing it is the value in `neuralwatt-card-sides.ts`.

Recurrence:
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Provider fields read by their documentation":
an owner's lift is built at the scope its words name.

## B808: the NeuralWatt hold read the pipeline's own use at a third of what a rotation of entries measured

Status:
fixed on 2026-10-22 (UTC),
red `648737d056`,
fix `a55cb37c91`,
note correction `5403f87382`,
cache versions none.
Found in the read of the settled calls of the corpus pass `XingZ6017` and fixed in `w270`.
The note correction is a wording fix:
the module note said the rotation held "1 to 15 calls" in each bucket that keeps the matrix's figure,
and two of those buckets held none,
so it now says at most 15 and none in two.

What the run showed:
the in-flight hold of each NeuralWatt call is the prompt-size bucket's mean in
`package/translation-repair/provider/src/neuralwatt-estimate.ts` times a margin of 1.
The bucket for 4,096 to 16,384 prompt tokens held the means of `zheermao19`
(0.0758 Wh for DeepSeek V4.1 Flash and 0.3725 Wh for GLM-5.3-Flash,
from 140 and 48 calls),
and the bucket for 16,384 to 65,536 tokens held the energy matrix's figures (0.1395 and 0.2009 Wh).
`XingZ6017` (log `~/temp/agent/corpus-pass-XingZ6017.log`,
41,183 lines by `wc`,
23:05 UTC on 2026-10-21 to 03:34 UTC on 2026-10-22)
settled 1033 calls that measured 493.166 Wh against 173.505 Wh held,
a ratio of 2.84:
`glm-5.3-flash` 3.09 over 269 calls and `deepseek-v4.1-flash` 2.57 over 764.

What was measured,
keyless,
from the per-call settlement line
(`NeuralWatt energy <model>: <measured> Wh measured against <estimate> Wh estimated for <n> prompt tokens`,
`neuralwatt-settlement.ts`)
by `~/temp/agent/tr-fan/tr-w270-notes/settle-table.ts`,
outside the repository,
whose output is `settle-table.out` in the same directory.
Completion tokens come from the `SPEND provider=neuralwatt` line printed on the line before each settlement.
No call lacked a measured energy in any of the six logs read.

- The last `METERS` line of `XingZ6017` (line 41082,
  03:33:16.860 UTC on 2026-10-22)
  reads `neuralwattWh=-5.188 neuralwattAccountWh=555.188`.
  The rotation `zheermao22` (190 calls),
  `CuspariaKLSY22` (74) and `XingZ6017` (1033) measured 40.910,
  21.111 and 493.166 Wh,
  which sum to 555.188 Wh,
  exactly that account figure.
  The three runs were all of the day's NeuralWatt use,
  and the 5.188 Wh overrun (0.94 percent of 550 Wh) is the day's total.
- Per bucket over the rotation
  (calls,
  measured mean,
  held mean,
  ratio of measured to held):
  4,096 to 16,384 tokens,
  DeepSeek 558 calls,
  0.1958 Wh against 0.0758 held,
  2.58;
  4,096 to 16,384 tokens,
  GLM 298 calls,
  0.9577 Wh against 0.3725,
  2.57;
  16,384 to 65,536 tokens,
  DeepSeek 385 calls,
  0.3205 Wh against 0.1395,
  2.30,
  all in `XingZ6017`;
  16,384 to 65,536 tokens,
  GLM 30 calls,
  1.2239 Wh against 0.2009,
  6.09,
  all in `XingZ6017`,
  with a standard deviation of 1.60 Wh and a standard error of 0.29 Wh.
- Too few calls for a mean:
  1,024 to 4,096 tokens,
  DeepSeek 7 calls at 0.0126 Wh against 0.0233 held,
  GLM 15 calls at 0.0210 Wh against 0.0389 held (ratio 0.54 for both);
  256 to 1,024 tokens,
  1 DeepSeek call and 3 GLM calls.
  No rotation call fell under 256 tokens or from 65,536 tokens up.
  One call of `XingZ6016` fell in the bucket from 65,536 to 262,144 tokens
  (GLM 0.2311 Wh against 0.9089 held,
  a picture counted as base64,
  which `neuralwatt-picture-tokens.ts` now handles,
  B720).
- Per entry in the 4,096 to 16,384 token bucket,
  DeepSeek then GLM
  (calls in brackets):
  `zheermao22` 0.0936 (129) and 0.5528 (52);
  `CuspariaKLSY22` 0.1699 (54) and 0.6626 (18);
  `XingZ6017` 0.2347 (375) and 1.0734 (228).
  In that bucket the DeepSeek median is 0.0856 Wh,
  the 90th percentile 0.4529 and the largest 3.1088;
  the GLM median is 0.6595,
  the 90th percentile 2.2332 and the largest 5.9387.
  In the 16,384 to 65,536 token bucket the DeepSeek median is 0.1540,
  the 90th percentile 0.8005 and the largest 3.5719;
  the GLM median is 0.5400,
  the 90th percentile 2.8101 and the largest 7.7040.
- The spread is in the output.
  Over the rotation's 951 DeepSeek and 346 GLM calls
  the correlation of a call's energy with its completion tokens was 0.76 and 0.85
  (about 0.12 and 0.18 Wh per thousand completion tokens),
  and with the prompt tokens the provider reported 0.19 and 0.26.
  The prompt-size bucket cannot see what drives a call's cost,
  as the note on `zheermao19` already said.
- The drift series,
  the measured-to-held ratio over the settled calls of each of the six logs in the order they ran:
  `zheermao21` 1.23 (247 calls),
  `CuspariaKLSY21` 2.00 (85),
  `XingZ6016` 0.87 (31),
  `zheermao22` 1.39 (190),
  `CuspariaKLSY22` 1.94 (74),
  `XingZ6017` 2.84 (1033).
  The series reported before this entry,
  1.23 then 1.39 then 1.94 then 2.84,
  left out two runs and read as a climb;
  it is one value per entry,
  and the same entry read 1.23 then 1.39 and 2.00 then 1.94.
- `XingZ6017` differs measurably.
  It sent 385 DeepSeek and 30 GLM calls in the 16,384 to 65,536 token bucket
  (the other five logs sent 8 in all) held at the matrix's figures.
  Its prompts in the 4,096 to 16,384 bucket were longer (mean 14.5k estimated tokens against 7.4k to 10.3k).
  Its replies sat between the other two entries'
  (mean completion tokens:
  GLM 6,047 against 3,860 and 6,490;
  DeepSeek 2,042 against 1,364 and 2,492).
  Its energy per thousand completion tokens was higher
  (GLM 0.178 against 0.143 and 0.102;
  DeepSeek 0.115 against 0.069 and 0.068).
  Why that entry costs more per completion token is a cause not traced;
  its longer prompts are one candidate.

The cause:
the table was the pipeline's measurement of one run of one entry (`zheermao19`)
plus the matrix for everything else.
The entries run since cost between 0.9 and 2.8 times what the table held,
and one of them sent a prompt-size range the table had never been measured on.
The first-order mistake is the one P19 recorded:
a hold taken from a single run's means stands for a workload,
and the workload is a rotation of entries that differ in cost.

The fix:
the two buckets with enough calls now hold the rotation's pooled means:
4,096 to 16,384 tokens,
DeepSeek 0.0758 Wh now 0.1958 and GLM 0.3725 now 0.9577;
16,384 to 65,536 tokens,
DeepSeek 0.1395 now 0.3205 and GLM 0.2009 now 1.2239.
Every other bucket keeps the matrix's figure,
including the 1,024 to 4,096 token bucket,
whose measured means (7 and 15 calls) sit under the matrix's
and would,
if the samples had reached 30,
lower the hold slightly.
`NEURALWATT_ENERGY_MARGIN` stays 1:
the owner's rule is a hold as small as can be justified so free energy is not left unspent,
and the hold is replaced by the measured figure at settlement,
so a hold at the pooled mean is right on average for the workload that produced it.
The module note records the runs,
dates,
counts,
old and new figures,
the per-entry means,
the drift series and a bound on the overrun.
`neuralwatt-estimate.unit.test.ts` pins the new means for every bucket of both models,
and `neuralwatt-picture-estimate.unit.test.ts` pins the 4,096 to 16,384 token GLM hold.
The pooled figures are call-weighted,
so `XingZ6017` supplies 375 of 558 and 228 of 298 calls in the main bucket
and all of the 16,384 to 65,536 token calls.
Replayed per run,
the new table holds `zheermao22` at 1.8 times its use,
`CuspariaKLSY22` at 1.3 times and `XingZ6017` at 0.92 times,
and holds the rotation at its use.
The in-flight hold only matters at the day's end,
when a call is refused if the remainder is under its hold (`neuralwatt-ledger.ts`,
line 432):
a larger hold refuses the last few calls of a light-entry day a little earlier,
which is the cost accepted for a hold that is right on average.
The table drives nothing else:
`NEURALWATT_SMALLEST_ESTIMATE_WH`,
the dryness floor,
reads the first bucket,
which did not change.

The docs that cited the old figures:
`doc/decision/translation-repair-neuralwatt-provider.md` (section "The margin") and the entry P19
in `audit-ledger-providers-routing-and-seating.md` still named 0.0758 and 0.3725 Wh as the table's figures.
Both carry a dated note of 2026-10-22 (UTC) that B808 replaced them,
and the record of what the table held on 2026-10-20 is left as written.

Cache versions:
none moves.
The estimate feeds the in-flight hold and the ledger line only,
and no text a model is shown changed.

Open:

- A hold shaped by the completion a call will produce would fit far better than the prompt bucket
  (correlation 0.76 to 0.85 against 0.19 to 0.26),
  but the completion length is unknown before the send.
  A better shape needs a design,
  for example a hold per effort setting and entry,
  or the completion cap,
  and none was made here.
- The 16,384 to 65,536 token GLM mean rests on 30 calls from one entry.
  The next run that sends calls in that bucket should replace it as `zheermao19` replaced the matrix.
- The cause of `XingZ6017` costing more per completion token than the other entries is not traced.
- The pool is three entries of the corpus.
  A rotation with other entries may move the means again;
  the per-call settlement line is the instrument.

Recurrence:
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Provider fields read by their documentation":
a published per-request figure is a placeholder until the pipeline's own settled calls replace it (P19),
and a single run's means are a placeholder for a workload that is a rotation of entries.

Later note of 2026-10-23 (UTC):
B836 replaced the four means of the two main buckets with those of `XingZ6020`,
the first settled pass on a build that holds the `low` effort of B807.
The figures this entry names
(0.1958 and 0.9577 Wh in the 4,096 to 16,384 token bucket,
0.3205 and 1.2239 Wh in the 16,384 to 65,536 token bucket)
are the record of what the table held from 2026-10-22 to 2026-10-23.

## B836: the NeuralWatt hold after the lift to low effort

The hold read GLM-5.3-Flash at fourteen times its use and DeepSeek at 79 percent.

Status:
fixed on 2026-10-23 (UTC),
red `f5fb8027be`,
fix `898445b025`,
cache versions none.
Found in the read of the settled calls of the corpus pass `XingZ6020`.

What the run showed:
`XingZ6020` (build `1d462324574c`) settled 1,779 NeuralWatt calls
that measured 451.711 Wh against 728.779 Wh held at the table of B808,
a ratio of 0.62:
`glm-5.3-flash` 412 calls at 28.006 Wh against 395.852 Wh held (0.07),
and `deepseek-v4.1-flash` 1,367 calls at 423.705 Wh against 332.926 Wh held (1.27).
B808 had held GLM-5.3-Flash at 0.9577 Wh in the 4,096 to 16,384 token bucket,
the mean of runs that sent its text calls at the provider's default reasoning effort.
B807 (`109f1b7a7e`,
2026-10-21) sends them at `low`,
and `XingZ6020` is the first settled pass on a build that holds it.
The rotation tip `2a336603bf` does not,
so the GLM figures of B808 and of this entry belong to different efforts and are not pooled.

What was measured,
keyless,
from the per-call settlement line
(`NeuralWatt energy <model>: <measured> Wh measured against <estimate> Wh estimated for <n> prompt tokens`),
by `settle-table.ts` outside the repository,
the way B808 did:
- DeepSeek,
  4,096 to 16,384 tokens:
  817 calls,
  248.687 Wh,
  mean 0.3044,
  standard error 0.0130,
  90th percentile 0.7516;
  held 0.1958 (ratio 1.55).
- DeepSeek,
  16,384 to 65,536 tokens:
  539 calls,
  174.478 Wh,
  mean 0.3237,
  standard error 0.0196;
  held 0.3205 (ratio 1.01).
- GLM-5.3-Flash,
  4,096 to 16,384 tokens:
  358 calls,
  25.577 Wh,
  mean 0.0714,
  90th percentile 0.1616;
  held 0.9577 (ratio 0.07).
- GLM-5.3-Flash,
  16,384 to 65,536 tokens:
  43 calls,
  2.392 Wh,
  mean 0.0556;
  held 1.2239 (ratio 0.05).
- Every smaller bucket holds at most nine calls and keeps the matrix's figure.

The fix:
the four buckets with enough calls hold the `XingZ6020` means:
DeepSeek 0.3044 and 0.3237 Wh,
GLM-5.3-Flash 0.0714 and 0.0556 Wh.
Replayed on `XingZ6020`,
the new table holds 451.697 Wh against 451.711 measured.
`neuralwatt-estimate.unit.test.ts` and `neuralwatt-picture-estimate.unit.test.ts` pin the new means.

Kept on the safe side for DeepSeek,
as ruled:
the DeepSeek means are those of `XingZ6020` alone,
which are above the pooled means of the rotation and this run (0.2603 and 0.3224 Wh).
The hold only matters at the day's end,
where a call is refused if the remainder is under its hold,
and a hold above the pooled mean refuses the last few calls a little early rather than overspending the day.
The module note bounds the overrun at the 90th percentile:
about 2.7 Wh for 30 GLM calls and 13.4 Wh for 30 DeepSeek calls.

Limits,
stated in the module note:
- The GLM means rest on a run on a build with the `low` effort.
  A run on a build without it would under-hold:
  replayed on `XingZ6017` (1,033 calls,
  the pre-B807 rotation),
  the new table holds GLM-5.3-Flash at 0.065 of its use (18.285 Wh against 281.721 measured),
  and DeepSeek at 1.13 times its use (238.852 Wh against 211.445).
- One entry supplies all of it.
  The next run on a rotation of entries should replace the means,
  as `zheermao19` replaced the matrix and B808 replaced `zheermao19`.
- One GLM picture read (0.0221 Wh at 8,188 tokens) is held at the same bucket mean,
  which is far above it.
- A reversion of the `low` effort would hold GLM-5.3-Flash at a seventh of its use until a run replaced the means.

Cache versions:
none moves.
The estimate feeds the in-flight hold and the ledger line only,
and no text a model is shown changed.

Recurrence:
B808,
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Provider fields read by their documentation":
a hold is taken from the build that sends the calls,
and a mean from a run on one effort is not pooled with a run on another.

## B1061: the plan-energy line of a run's end printed the account's charge and not the run's own meter beside it

Status:
fixed on 2026-10-29 (UTC) in `tr-w379`,
red `4d49e137b1`,
fix `3de1638c7b`.
Found by the read of `XingZ6022` (build `c6185d4086`) and its check.
No cache version moved:
the report is a log line and keys nothing.

What the run showed:
the end of the run printed (log line 29727) `NeuralWatt PLAN ENERGY SPENT: 1730.5 Wh`,
said the owner's plan was charged that much between the start and the end of the run,
and ended with the clause `anything else done on the account in that time is in this figure too`.
The run's own meter,
the 1,700 `NeuralWatt energy` lines (1,294 on `deepseek-v4.1-flash` and 406 on `glm-5.3-flash`),
adds up to 238.272 Wh,
and the account-wide covered energy since the day's start that `METERS` carried (`neuralwattAccountWh`) agreed with
that running sum to 0.001 Wh.
So the plan figure was 7.26 times the run's draw,
and nothing on the line said what the run had drawn.
`XingZ6021` printed no plan energy for 283 Wh drawn.

What the line and its docs said before:
the line already ended with "anything else done on the account in that time is in this figure too",
and the module note of `provider/src/neuralwatt-quota.ts` named the owner's other use of the covered models.
Neither printed the run's own meter beside the difference,
and the TSDoc of `describePlanSpend` did not say that the figure is not the run's draw.
A reader had to go to the log for the second number,
which is how the first one was read as the run's.

Cause of the 7.26 times:
not determined.
`planSpendBetween` is the end of the account's `kwh_used` less its start,
which the quota documentation calls the energy charged against the plan,
consumed energy times the lane's multiplier.
The pricing multiplier of the standard lane is 1.0 in this package's source (`provider/src/neuralwatt-comments.ts`),
the start figure alone (10.9763 kWh charged in a period that began 2026-10-20T19:15:08Z) is above what nine free days
hold (4.95 kWh),
so the account had been charged beyond its free energy before the run began,
and the by-model usage showed only the run's draw.
Other use of the account and a charge that is not the run's metered energy are both open,
and the log cannot decide between them.

What changed:
- `createNeuralWattLedger` counts the watt-hours of the lines its own object appended (`drawnWh`):
  settled calls at their measured figure or,
  where the stream reported none,
  their estimate,
  and abandoned attempts at their estimate.
  It reads the run's draw and not the day's,
  which also holds the lines an earlier run left in the file,
  and it counts a line only after the append succeeded.
- `NeuralWattClient.drawnWh` returns it,
  and the run-end report (`run/src/corpus-run/run-providers.ts`) passes it to `describePlanSpend`.
- `describePlanSpend` takes `drawnWh` and says on every branch what the run's own meter drew in that time.
  Beside a nonzero difference it says the plan figure is not this run's draw:
  it is what the account was charged,
  which can include energy this run did not draw.
  Without `drawnWh` the sentences are as they were.
- The line for `XingZ6022` would have gained the clause
  `this run's own meter drew 238.596 Wh in that time, so the plan figure is not this run's draw`,
  then the sentence that the figure is what the account was charged,
  which can include energy this run did not draw,
  before the clause it already ended with.
  The 238.596 is the 238.272 of the energy lines and the 0.324 Wh abandoned estimate of the call cut at its
  content bound (line 14463),
  which the ledger carries and the energy lines do not.

Guards shown to fail first:
`neuralwatt-ledger.unit.test.ts` (the draw counts only the lines its own object appended,
with an earlier object's line,
a measured call,
an unreported one,
an abandoned attempt and a refused reservation),
`neuralwatt-client.unit.test.ts` (the client reports the sum of two calls and none before a call),
`neuralwatt-quota.unit.test.ts` (two cases:
the nonzero branch with 1730.5 Wh against 238.272 Wh,
and the zero,
later-start and not-comparable branches).
The four failed with `drawnWh is not a function` and a missing sentence at `4d49e137b1`.
The case of `run-client-neuralwatt.unit.test.ts` that gained two expectations was not run before the fix.
Gates after the fix:
provider and run `test:unit`,
`lint:types` and `lint:oxlint` all passed.

What still stands:
the cause of the 7.26 times.
The next run that reads the quota before and after prints both numbers,
and an account with no other use on the day would show whether the charge follows the run's meter.
