# Current roster changes

Part of [the family README](../README.md).

Adding,
removing or changing a model is one card in `package/translation-repair/shared-contract/src/model-cards.ts` since
2026-09-16:
every provider catalog,
the completion-cap table,
the abandoned-spend ratios and every seat hold derive from the cards,
the served-id lists in `package/translation-repair/shared-contract/src/roster-id.ts` type them,
`mise run roster-card -- <provider> <served id>` prints a card fragment off the provider's live listing,
and the unit tests sit models by role-named seats from
`package/translation-repair/test-fixture/src/roster-seats.test-fixture.ts`.
The steps are
[the roster-change runbook](../../../doc/runbook/translation-repair-roster-change.md)
and the reasons
[the model-card decision](../../../doc/decision/translation-repair-model-cards.md).

DeepSeek V4 Pro 0813 and V4 Flash 0731 left the roster on 2026-09-16 at the owner's instruction
("DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash");
both spellings on both providers are blocklisted,
and V4.1 Flash took the editor and refiner seats V4 Pro held,
unmeasured in those roles until the next editor calibration.
The roster is twelve models.

`deepseek-v4.1-flash` is routed off DeepInfra and Wafer since 2026-09-19
(`ignoredEndpoints` on its card):
on XingZ607 Morph answered its judge calls at 11 s median,
DeepInfra at 45 s with 89 of the 97 cut streams,
Wafer at 26 s with the rest,
and on DeepInfra the seat set the quorum time in 553 of 2,320 rounds
(the pass log's heading on routing `deepseek-v4.1-flash` off DeepInfra and Wafer,
dated 2026-09-19).
On 2026-09-23 the price sort fell to endpoints that reason at length by default,
so the card also ignores OpenInference (class ninety-one,
XingZ624 and XingZ625),
DekaLLM and Sail Research (class ninety-three,
XingZ626),
and its `preferredEndpoints` names Morph ahead of the price sort.
Class ninety-three also names Wafer ahead of the price sort for `hf:zai-org/GLM-5.3-Flash`.
The request shape is under `TRANSLATION_REPAIR_OPENROUTER_API_KEY` in [Configuration](configuration.md).

`typesafe/jev-1.13` joined on 2026-09-18 as the first decision-only seat:
a model OpenRouter serves through its decisions endpoint
(typed `choice`,
`noul` and `score` answers over a state)
and not through chat.
Its card has a `decisions` side and no provider side,
it is off `RUN_ROSTER` and every chat bench,
and it sits on `RUN_SELECT_JUDGES` after every wide seat once measured.
The production judge fidelity probe on 2026-09-18 chose the reviewed reference on all 12 rows it was asked,
no damaged pick or decline,
level with DeepSeek V4.1 Flash and Mercury on the same rows,
so the judge hold came off in `c14c7f176`.
It votes only in the candidate-select stage,
the one stage with a typed question;
writer and reader holds stay because a typed decision writes and reads nothing.
The steps are the runbook's "Add a decision-only model" section.

DeepSeek V4.1 Flash was approved on Hyper and OpenRouter on 2026-09-11.
It has one distinct catalog identity,
`deepseek-v4.1-flash`,
served as that ID on Hyper and `deepseek/deepseek-v4.1-flash` on OpenRouter at the approval
and as `deepseek-v4.1-flash-1017` and `deepseek/deepseek-v4.1-flash-1017` since 2026-10-20
(the 1017 migration paragraph gives the reasons).
Both actual compiled clients passed the streamed structured-output probe.
V4 Flash 0731 was a separate version whose ratings,
writer exclusion and endpoint exclusions did not transfer,
and it is off the roster since 2026-09-16.

The new version retains the pooled completion cap of 13082.
Its reported image capability does not itself enable a reader seat.
The source-reviewed judge comparison on 2026-09-11 selected the reference on all fourteen distinct questions,
with no damaged pick or decline,
against a peer median of fourteen and maximum damaged count of one.
It now joins judge and general text/preparation seats once under its shared identity.
Writer and image-reader holds remain until their separate measurements.
Existing peer order,
writer/reader arrays and the reader-only Gemma 4 31B seat are unchanged.
The [admission record](../../../doc/planning/translation-repair-deepseek-v41-flash-2026-09-11.md)
contains availability,
prices,
verification and remaining work.

DeepSeek V4.1 Flash moved to the 1017 weights (`deepseek-ai/DeepSeek-V4.1-Flash-1017`) on 2026-10-20,
on the owner's instruction:
"Migrate to deepseek v4 flash 1017 where possible."
Hyper and OpenRouter list the 1017 weights under their own ids,
so the card's `hyper.id` is `deepseek-v4.1-flash-1017`
and its `openrouter.id` is `deepseek/deepseek-v4.1-flash-1017`,
while the roster id stays `deepseek-v4.1-flash`
(caches,
ledgers,
calibrations and logs keep one name for the seat;
`HyperModelInfo.rosterId` ties the two).
NeuralWatt's `deepseek-v4.1-flash` already served the 1017 weights,
by its listing's `huggingface_id`,
so that side did not change.
Synthetic lists the 1017 weights too,
but this card has no Synthetic side and adding one would be new seating,
so none was added.
Morph's price on the card moved to 0.158 and 0.6 USD per million
(listing of 2026-10-20;
was 0.12 and 0.468).
The five ignored OpenRouter endpoints stay:
DeepInfra and Wafer list the 1017 id,
and OpenInference,
DekaLLM and Sail Research listed no endpoint for it on 2026-10-20,
so those three ignores are inert until one does.
The measured values on the card
(completion cap 13,082,
Hyper `maxOutputLength` 26,214,
`rawCharsPerToken` 228,
the holds and the judge and writer calibrations)
were measured on the pre-1017 ids and carry over as an assumption until the first runs on 1017 measure them.
No cache version moved:
no cache key names a served id.

Hyper-origin identity names and Synthetic-counterpart projections are now explicit:
`HYPER_ORIGIN_ROSTER_IDS`,
`HyperOriginRosterId`,
`NO_SYNTHETIC_COUNTERPART`,
`hyperModelsWithSyntheticCounterparts`
and `hyperModelsWithoutSyntheticCounterparts`.
These do not claim Hyper-exclusive serving;
`reachOf` owns actual provider reach.
Only the type and `NO_SYNTHETIC_COUNTERPART` remain package source:
the two counterpart lists moved to a test fixture on 2026-09-29 (ledger B30),
and `HYPER_ORIGIN_ROSTER_IDS` went the same day (ledger B31),
since only tests read them.

`qwen3.8-max` was removed from roster and Charm Hyper allowlist on 2026-08-28 at owner's instruction.
Its metered cost was disproportionate and exceptionally expensive.
No replacement was selected,
so that cull left nine models.
Dated pricing remains only so historical run artifacts can still be accounted.

Synthetic `hf:zai-org/GLM-5.2` was replaced by `hf:zai-org/GLM-5.3-Flash` on 2026-08-29.
The live endpoint confirmed the successor;
the operational request reported Synthetic's plan to retire the older model.
Synthetic's live model endpoint reported the replacement as always-on beta,
with text and image input,
a 524288-token context,
a 65536-token output ceiling,
and the structured-output features this pipeline requires.
The old `glm-5.2` Charm Hyper route left the active allowlist because it is the superseded roster identity,
not a fallback for GLM-5.3-Flash.
Hyper's live catalog still listed `glm-5.2` but no GLM-5.3-Flash spelling on 2026-08-29.
That replacement left the roster at nine models.

Nemotron-3-Super left every active stage and the callable Synthetic catalog on 2026-08-29 at owner's instruction.
In adjacent required-correction reviews it first proposed concrete wording
and then was sole reviewer rejecting that wording.
The roster now has eight models.
Broad-stage quorum consequently moves from five to four,
which equals entire Synthetic side;
a run with all four Hyper seats dark can reach exact-half quorum on four Synthetic seats.
Both-key startup refusal prevents missing credentials but not provider becoming unavailable later,
so any such run remains degraded evidence rather than readiness proof.
Kimi-K3 takes departed checker seat rather than shrinking below hard floor of three;
it participated in 231-round wide checker arm where added voices changed zero verdicts,
but fresh checker-seat calibration remains required before treating new narrow roster as independently optimal.
GPT-OSS takes departed default restoration-benchmark judge seat because it already checks and judges in production;
benchmark-specific calibration remains open.
Historical artifacts and measured narratives retain departed identity.

GLM-4.7-Flash remains blocked from every active stage,
roster type,
and callable catalog as it has been since 2026-08-24.
Interrupted schema-9 run started before Nemotron removal logged no GLM-4.7-Flash model label,
and no translation-repair process remained alive after run was stopped.
This establishes interrupted pass did not issue GLM-4.7-Flash calls;
it does not identify source or time window of calls visible outside package log.
A widened scan over same-day validation,
review,
replay,
and probe logs found zero GLM-4.7 call labels;
remaining mentions were provider catalog listings or historical stream-parser test names.

Same live catalog read showed `syn:large:text` now points to GLM-5.3-Flash,
while current Synthetic rate-limit documentation names Kimi-K3 as one-request baseline.
Planning denominator now follows Kimi-K3 input price and has GFP coverage;
live `/quotas` remains authoritative.
See `doc/troubleshooting/synthetic-rate-limit-default-drift.md`.

`hf:openai/gpt-oss-120b` left every role on 2026-09-24 at the owner's instruction
("That particular model got cause and effect wrong.
Cull it.",
then "cull it from every role"):
its card carries the hold `owner-culled`,
`RUN_ROSTER` and the readers filter it,
`gemma-4-26b-a4b-it` takes its static checker seat
and `google.gemma-4-e2b` becomes the substitute checker,
both **provisional**.
The card stays for the catalogs and the unit fixture seat `SEAT_SYNTHETIC_TEXT_EVERYWHERE`,
so the blocklist is untouched
(the 2026-09-24 addendum of the seating decision).
The roster is eleven chat seats plus the decision-only seat.

The checker seats were measured on 2026-09-27
(`RUN_CHECKER_ORDER` in `package/translation-repair/run/src/corpus-run/run-config.ts` carries the numbers).
Each candidate answered the resolution sheet for 85 settled fixes
and for the same 85 issues against the unchanged archive text,
where not fixed is certain.
The order is Qwen3.8-27B,
Mimo v2.6 Flash,
Kimi-K3,
`google.gemma-4-31b` and Mercury 2.5,
and a reading seats the first three a wet provider serves.
`gemma-4-26b-a4b-it` left the checker role,
and only that role,
for calling 40 of the 85 unchanged texts fixed;
`google.gemma-4-e2b` left the substitute seat for calling 50 of the 85 real fixes not fixed.
The owner approved Mimo v2.6 Flash,
Mimo v2.6 Pro and Solar Mini 4 on OpenRouter the same day.
Mimo v2.6 Flash joined with a card that holds it out of every judge,
writer and reader seat until measured there.
Mimo v2.6 Pro read below Flash on both halves while sharing 8 of its 9 errors,
and Solar Mini 4 called 78 of the 85 unchanged texts fixed;
neither holds a card.
The roster is twelve chat seats plus the decision-only seat.

`gemma-4-26b-a4b-it` is listed as reading pictures on OpenRouter since 2026-10-21,
behind the card hold `openrouter-pictures-unmeasured`:
the owner said "Yes try gemma" after `XingZ6016` reached the end of its pictures with one reader,
Bedrock's key being never renewed and Bedrock the only route its transcription was measured on (2026-09-08).
The listing reports image input and the zero-data-retention endpoint listing names thirteen endpoints,
every request keeps `provider.zdr: true`,
and the vision reach sends no picture to OpenRouter until the hold is lifted.
The step that lifts it is
`mise run //package/translation-repair/bench:reader-measure -- --trusted <runs dir> --pictures <people dir>`
with `TRANSLATION_REPAIR_OPENROUTER_API_KEY` set:
when it reports both clauses met,
remove the hold from the card,
rebuild,
and run `cache-account-audit` (ledger B721 to B723;
the cache version this step was thought to move did not exist,
see the update in B721).

The hold was lifted on 2026-10-21 (UTC),
after `reader-measure` ran three times through OpenRouter
with zero data retention at `--sample 9`.
The first run (08:44 UTC) and the second (08:48 UTC) named the 20 recent `corpus-pass-*` runs directories,
the second adding the full-corpus pass of 2026-08-19.
Both read 3 trusted records,
matched 3 of the 291 pictures,
produced 3 readings and were not measured:
3 of the 6 readings a verdict needs.
The recent runs directories hold only 3 `picture.<key>.json` files,
all corroborated and all for one entry,
so no sample could have added a picture,
and the report's advice to ask again with a larger sample was the defect of ledger B800.
The third run (09:02 UTC) named 107 older single-entry runs directories,
which hold 450 such files and 192 distinct trusted reading sets across 10 entries
(Mio 106,
hakureico 26,
XingZ60 21,
Toka_ls 12,
gqt 8,
zheermao101 8,
luxuanwen3 3,
Zha_Ke 3,
yuki418330012 3,
Xu_Yushu 2).
It matched 9 pictures and produced 9 readings against a seated band of 43 readings
with a floor of 1.00 corroborated by all and a ceiling of 0.00 corroborated by none,
and both clauses were met:
every one of the 9 readings was corroborated by every seated reading of its picture,
and none was corroborated by none.
The limit of that verdict is its breadth:
the 9 pictures came from two entries (Mio 7,
Toka_ls 2).
The measurement spent 15 calls of the model (3,
3 and 9).
The card no longer carries `openrouter-pictures-unmeasured` (the fix commit `97e17dc6dc`,
red `3a61ff6698`),
and the vision reach of `gemma-4-26b-a4b-it` names Bedrock and OpenRouter.
The hold,
its set and its gate stay in the code for the next candidate;
no card carries it.
No cache version moved (ledger B721,
update of 2026-10-21 (UTC)).

## DeepSeek and Xiaomi reach two seats directly since 2026-10-22

No model is seated or removed.
`deepseek-v4.1-flash` is also served by DeepSeek's own API (wire id `deepseek-flash`),
and `xiaomi/mimo-v2.6-flash` by Xiaomi's (wire id `mimo-v2.6-flash`),
each on a key the owner granted for that one model.
Each card gained a `deepseek` or `xiaomi` side in `shared-contract/src/direct-card-sides.ts`,
the served-id types are single literals,
and every other seat is refused by the client before any request.
Pictures stay off both routes wherever a seat's picture reading is unmeasured.
The record is `doc/decision/translation-repair-direct-providers.md`.
