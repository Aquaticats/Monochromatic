# Audit ledger: reasoning runaways and the re-ask of another model

A reply that ran away reasoning,
re-asked of the same model on another route and running away again (B1072),
the two re-asks of a length stop with no content that still asked the same model (B1073),
and the owner's answers of 2026-10-30 that settled both:
every re-ask goes to another model and honours the role of its seat (B1074),
in the order of the census's shares of runaways (B1075),
without a model on the bench (B1076),
with the plain question (B1077),
the pass logs the census reads that are gone (B1078),
and the callers that name no role yet (B1079).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1072: a reply that ran away reasoning was re-asked of the same model on another route, and ran away again

Status:
fixed at 00:03 UTC on 2026-10-30 in `4f20f05f15` (provider),
the cases committed red first in `0b52a103b7` at 23:56 UTC on 2026-10-29,
and the logger of the loop tagged in `3b4c280885` after the red `21f3195739`.
The choices it left to the lead were answered on 2026-10-30,
and B1074 to B1077 carry the answers out.
The lead's decision of 2026-10-29 (confirmed by the owner's answer "Everywhere (Recommended)" on 2026-10-30):
"After an answer from model M ends in a length stop with no content (a reasoning runaway),
a re-ask goes to a different model,
never M again on another route.
Other re-ask reasons (schema mismatch with content,
HTTP errors) are unchanged."
Where M is the only model left the lead asked for today's behaviour,
logged;
and "Never disable thinking anywhere and send no new thinking or effort parameter" holds:
no request gained a field.

Found on 2026-10-29 reading the DeepSeek calls that ran to the cap (B1070).
Seven of the 63 direct DeepSeek calls ran to 13,082 tokens with `finish_reason=length` and no content,
in 526 seconds of streaming and 0.131 USD at the logged costs:
`corpus-pass-XingZ6019.log:407`,
`corpus-pass-XingZ6020.log:1657`,
`:10238`,
`:19841`,
`:26556`
and `:29966`,
and `corpus-pass-XingZ6021.log:470`.
Those logs are no longer under the agent root (B1078).
Six of them came after another provider's answer of the same model
(`deepseek-v4.1-flash` on NeuralWatt)
failed the same way:
`XingZ6021` lines 456 then 470,
`XingZ6020` lines 1626 then 1657,
10171 then 10238,
19794 then 19841,
26490 then 26556,
and 26575 then 29966.
Each earlier answer was 13,082 tokens with no content
(`XingZ6021` line 456:
56,609 ms and 54,607 reasoning characters),
and the lost-voice lines that follow five of the second calls say `raw=""` for both
(`XingZ6020` lines 1659,
10240,
19843,
26558 and 29967).
So six of the seven times a direct DeepSeek call ran to the cap,
the re-ask of the same weights on another route repeated a runaway.
How many NeuralWatt runaways were re-asked on DeepSeek and answered usably was not counted
(the re-ask lines repeat when a claim is replayed from memory,
`XingZ6020` lines 29895 to 29899),
so the share of such re-asks that repeat the runaway is not known.
The nudge the re-ask carried (`CROSS_PROVIDER_NUDGE`) asks for the JSON object only and says nothing of the length.

The fix:

- A reply is a reasoning runaway when its reading is a schema mismatch with reason `truncated-completion`
  and its raw text shows a reader nothing (`rendersAsNothing`).
  The `raw=""` of the lines in the account of the evidence is that raw text.
  A cut answer that holds content,
  a refusal,
  and an unparseable reply with no content and no length stop are not.
- A runaway is re-asked of the roster's other models in turn
  (`provider/src/reask-alternate-models.ts`,
  without the model that ran away,
  without decision-only seats and without the Claude route),
  until one has a provider with budget:
  a `NoProviderForModelError` for a model moves on to the next.
  The order,
  the nudge and the models left out by role and bench are the answers of B1074 to B1077;
  as w382 wrote it the order was the roster's and every ask carried the nudge.
- Where none has,
  the model that ran away is the only one left to ask,
  and the re-ask is what it was before,
  on another provider,
  with a warning naming the model and how many were tried.
- The info line names the model,
  the reason and the model chosen,
  under the tag `askOtherModels` beneath `reaskElsewhereNudged`.
  The line of the same-model re-ask (`re-asking another provider, nudged`) is unchanged.
- The router's own `chatJson` routes an unhinted re-ask as it routes a first ask.
  The uniqueness wrapper's claim path needed no change:
  the re-ask is a prompt under another model's name,
  so it is claimed and stored as its own digest.
- A re-ask of another model that fails too says so in the cause of the first answer's `reaskFailure`
  (`; asked of <model>`),
  so the lost-voice line does not read as the runaway answering twice.
- `EveryProviderDryError` is rethrown by the re-ask,
  where before it was kept as a warning.
  The re-ask of another model is routed like a first ask,
  so it can meet that refusal,
  which ends the run;
  the old re-ask,
  hinted away from one provider,
  could not.

Choices the lead owned,
and how the owner's answers of 2026-10-30 settled each:

- The order was the roster's,
  a placeholder until the lead said.
  The answer "Lowest runaway share (Recommended)" replaced it by the census order (B1075).
- The answer was recorded as the voice of the seat that ran away,
  and the model that gave it could sit on the same bench.
  The lead's adoption of 2026-10-30 leaves a bench member out (B1076).
- The Claude route stays left out because its key is metered ("be careful what you use").
  The lead's adoption of 2026-10-30 changes nothing here:
  the other metered routes are asked,
  since the router already spends on them for a first ask.
- The list read no hold of a card.
  The answer "Honour holds (Recommended)" makes it read the holds of the seat's role (B1074).
- The nudge read "Your previous reply could not be used",
  which the other model never wrote.
  The answer "Plain question (Recommended)" removes it from every ask but the fall back to the model that ran away (B1077).
- Where most providers read dry,
  one runaway costs up to eleven attempts one after another,
  each a routed call that reads the budgets before it refuses,
  and then the fall back to the runaway.
  Unchanged in kind,
  and now eight attempts at most before the fall back for the roster-wide role (B1075).

Cache versions:
none moves,
and B1074 gives the account for every cache this round touches.

Tests:
`nudged-reask.unit.test.ts` (nine cases added by w382,
seven of them red at `0b52a103b7`),
`provider-router.unit.test.ts` (the router's own `chatJson` re-asks a Qwen3.8-27B runaway on Synthetic of another model),
and `prompt-uniqueness-client.unit.test.ts` (the claim path asks another model with no hint,
and replays both on a later ask).
`reask-alternate-models.unit.test.ts` holds the list itself.
B1074 to B1077 name the cases they changed and added.

Review:
the post-commit standards review of `4f20f05f15` (`doc/audit/standards-review-gate-shadow.md`)
found the logger of the loop untagged,
fixed in `3b4c280885` after the red `21f3195739`,
and two articles at the head of `@returns` lines.
It also named the capital letters of the module notes,
the package-root imports and the thrown `Error` of the test stubs,
and the inline reason beside `no-await-in-loop`,
all of which follow the code around them and stay.

Recurrence:
`mistake-prevention.md`,
"Cached decisions" for the cache account,
and a re-ask is chosen by why the first answer failed and not only that it failed.

## B1073: two re-asks of a length stop with no content still asked the same model

Status:
fixed between 01:50 and 02:40 UTC on 2026-10-31,
the stage in `640dac1856` after the red `f3168bee5c`
and the picture re-ask in `37e025db0a` after the red `e020231119`.
The owner answered "Everywhere (Recommended)" on 2026-10-30:
after a reasoning runaway of model M (a length stop with no content),
every re-ask goes to a different model,
the provider re-ask (B1072),
the stage recovery round and the picture re-ask (`reaskCut`).
Other re-ask reasons stay as they are.
B1074 holds the account of what changed.

Found on 2026-10-29 while fixing B1072.
The lead's decision of 2026-10-29 named the nudged re-ask (`nudged-reask.ts`,
`provider-router-reask.ts`).
Two other places asked the same model again after a length stop with no content:

- The stage's recovery round (`stage/src/stage-quorum.ts`,
  `recovery-nudge.ts`)
  re-asked every seat whose last answer was unreadable,
  with the `cut-short` nudge,
  through the same client,
  so it asked the seat's own model a second time.
  The log of `XingZ6020` shows it twice for slice 92,
  at 02:14:33 UTC on 2026-10-23 (line 26560) and at 02:44:04 UTC (line 29907),
  each saying `recovery round for 1 unreadable answer (1 cut-short)`.
  By its timing the first round's ask is the NeuralWatt call at line 26575
  (13,082 tokens,
  no content,
  78.7 seconds long and ending 78.7 seconds after the round began),
  so the round asked the model that had already run away twice
  (lines 26490 and 26556) a third time and it ran away again.
  Line 29966 is the nudged re-ask of the second round:
  a direct DeepSeek call that ran to the cap,
  followed by the lost-voice line (29967) tagged `[runRecoveryRound]`
  and by `deepseek answered schema-mismatch too` (line 29968).
  That is four runaways of one model for one question
  (lines 26490,
  26556,
  26575 and 29966),
  and the third was the round's own ask.
- `reaskCut` in `slice-reading/src/image-reading-past-refusal.ts` re-asked a picture reading
  whose reply the length limit cut empty
  of the same model on another provider (`otherThan`),
  and on the same provider where none served it;
  `image-reading-cut-empty.unit.test.ts` pinned both.
  A picture reader joins a route only after a keyed measurement of that route
  (`design-commitments-providers-and-seats.md`,
  the rule of 2026-09-08),
  so an unmeasured model cannot simply stand in for it.
  The reader role's hold (`reader-unmeasured`) is how a card records that measurement (B1074).

## B1074: after a reasoning runaway the stage recovery round and the picture re-ask asked the same model, and no re-ask read the role of its seat

Status:
fixed between 01:50 and 02:40 UTC on 2026-10-31.
The cases were committed red first:
`d82d79a4b7` (shared contract),
`eba153f361` (provider),
`f3168bee5c` (stage) and `e020231119` (slice reading);
the fixes are `b40433e0bd`,
`35cd20d2a7`,
`640dac1856` and `37e025db0a`.
Two fixtures and one case that scripted an empty length stop as "unreadable" now script a cut that holds content
(`121719e570` in corpus-pass,
`cf93bef02a` in archive),
because an empty length stop is a runaway that the recovery round asks of another model.
The owner's answers of 2026-10-30:
"Everywhere (Recommended)" (every re-ask after a runaway goes to a different model)
and "Honour holds (Recommended)" (leave out any model whose card holds it out of the role being asked).
The lead's adoption of 2026-10-30 keeps the Claude route left out,
and no other metered route is.

What the evidence was.
Slice 92 of `XingZ6020` ran away four times in one model through the recovery round (B1073).
No request carried the role of its seat,
so the list of B1072 could not read a card's holds;
it listed `xiaomi/mimo-v2.6-flash` and `google.gemma-4-31b` for any seat,
and `hf:openai/gpt-oss-120b`,
which the owner culled from every role on 2026-09-24.

What it does now:

- The role.
  `ChatTextRequest.reaskSeat` (`{ role, bench }`) carries the role of the seat and the models seated on the question.
  It is no part of the prompt identity and no client puts it on the wire,
  so no request gained a thinking or effort parameter.
  The shared contract names seven roles (`seat-role.ts`):
  `roster`,
  `translator`,
  `writer`,
  `wide-judge`,
  `late-judge`,
  `reader` and `checker`,
  and the holds each honours,
  read off the bench derivation of `run-config.ts` on 2026-10-30.
  `roster` and the four seats built on it honour `owner-culled` and `judge-unmeasured`;
  `translator` adds `writer-unmeasured` and `translator-dropped`;
  `writer` adds `writer-unmeasured`;
  `wide-judge` adds `wide-seat-dropped`;
  `late-judge` adds `late-judge-dropped`;
  `reader` honours `owner-culled` and `reader-unmeasured` only,
  since `RUN_READER_MODELS` lets a model the judge probe withheld read;
  and `checker` honours the cull alone.
  Holds of a route (`openrouter-dropped`,
  `openrouter-withheld`,
  `synthetic-withheld`,
  `hyper-slow-select`) are the router's to read at the call.
- The provider re-ask reads `request.reaskSeat`
  and leaves out each model the role holds out,
  naming it and the hold in the log.
- The stage names the bench and the role on every ask of a gather
  (`gatherStageVoices`,
  parameter `role`,
  default `roster`),
  and the call returns a length stop with no content as the cause `runaway`,
  apart from `cut-short`,
  carrying the model an earlier re-ask went to (`askedInstead`).
  The recovery round asks a `runaway` seat of the next model that the role,
  the bench and that earlier ask leave,
  with the plain question,
  and records the answer as the seat's voice.
  Where no model is left it asks the seat's own model under the `cut-short` nudge,
  as before,
  on a warning.
- The picture re-ask (`reaskCut`) asks the models that read pictures,
  that the `reader` role does not hold out and that are not readers of the same picture,
  with the first reading responsibility.
  `readImagePair` passes the readers of the picture as the bench.
  Models that read no picture are named in the log as left out for it.
  Where none can take it the reader itself is asked as before.
- The log lines name the model,
  the reason,
  the model chosen and each model left out with why:
  `deepseek-v4.1-flash: reasoning runaway on neuralwatt (a length stop with no content);
  re-asking gemma-4-26b-a4b-it instead, plain question; left out: ...`
  (provider),
  the same under `[recoverRunaway]` (stage),
  and `... was cut by the completion limit on <picture> (finish reason length on <provider>);
  re-asking <model> instead, plain question; left out: ...` (picture).
  A fall back to the model that ran away is a warning:
  `no other model could take the re-ask ... (N tried)`.
  The model that ran away,
  the decision-only seat and the Claude route are always left out and are not named.

Tests
(case names are the `it` names of the files):
`seat-role.unit.test.ts` (six cases,
a new file),
`reask-alternate-models.unit.test.ts` (ten),
`nudged-reask.unit.test.ts` (six changed and three added:
the plain question,
the choice of the lowest share,
the move to the next model,
the fall back with its nudge on that ask alone,
the log,
the cause that names the model asked,
the role skip,
the bench skip and the empty list),
`stage-recovery-runaway.unit.test.ts` (seven),
`stage-quorum-runaway.unit.test.ts` (two),
`stage-call-runaway.unit.test.ts` (three),
`recovery-nudge.unit.test.ts` (two added),
`image-reading-cut-empty.unit.test.ts` (nine,
five red),
`provider-router.unit.test.ts` (one changed)
and `prompt-uniqueness-client.unit.test.ts` (one changed).
Two cases of `stage-quorum.unit.test.ts` scripted an empty length stop as an answer that stays unreadable,
and now script a cut that holds content.

Cache versions:
none moves.
The eight slice-cache versions
(pairing 3,
consolidate 32,
lane contest 16,
translate slice 25,
repair slice 43,
refine 14,
consistency 7
and page title 6)
key what a question was and who answered it,
and this change alters whose voice a runaway seat contributes to a gather,
not what any stored record means.
A record stored by an older build is not served to this one:
every namespace opens under the digest of the built pipeline,
which hashes the first-party bytes under `dist/final/node`,
and a cache filled by a different pipeline is discarded before it is read
(`slice-cache/src/corpus-run/slice-cache-namespace.ts`,
the account of 2026-10-18,
ledger B670,
as B1072 applied it).
The account of B1072 stands:
`cache-account-audit`,
run at 00:03 UTC on 2026-10-30,
read 13,711 records under 441 runs directories,
the newest from 19:14 UTC on 2026-10-22,
and seven of the eight versions were set after it.
This round did not run the audit again (a built runner),
and no pass has run since:
the newest run directory,
`corpus-pass-XingZ6022`,
was last written at 11:44 UTC on 2026-10-29,
and its `slice-cache` directory is empty.
The stored model replies of the prompt payload store are keyed by model and prompt digest and stay valid as replies;
what the new asks do to those digests is B1077.

Open to the owner's veto:
the table of roles and holds is a reading of the bench derivation and not a decision of its own.

Recurrence:
a hold names a role,
and a re-ask of a question asks a model of the role of the seat it stands in
(`mistake-prevention.md`,
"Cached decisions").

## B1075: the order of the models a runaway is re-asked of was the roster's, with no measure behind it

Status:
fixed between 01:50 and 02:40 UTC on 2026-10-31 in `35cd20d2a7`,
with the cases red in `eba153f361`;
the input of the order is open (B1078).
The owner's answer of 2026-10-30:
"Lowest runaway share (Recommended)":
order the other models by the measured share of their calls that ran away
(length stop with no content,
since the caps,
pooled over providers),
lowest first.

The reading.
The order rests on the cap census of 00:18 UTC on 2026-10-30,
run from this tree with the pairing of B1070 and the seats of B1071
(447 pass-run logs,
582,710 completed calls).
"Ran away" is what the census counts as at the cap with none:
a completed call that reached the cap its card sends and streamed no content,
among the calls since the caps went on the wire,
pooled over the providers that served the model.
A call the census could not classify (4 ambiguous,
none unpaired) is among the calls and in neither count.
The table
(calls since the caps,
runaways,
share):

- `gemma-4-26b-a4b-it`:
  70,847 calls,
  0,
  0 percent.
- `google.gemma-4-e2b`:
  67,481,
  0,
  0 percent.
- `google.gemma-4-31b`:
  214,
  0,
  0 percent.
- `inception/mercury-2.5`:
  59,027,
  0,
  0 percent.
- `hf:moonshotai/Kimi-K3`:
  15,202,
  3,
  0.02 percent.
- `xiaomi/mimo-v2.6-flash`:
  1,829,
  1,
  0.05 percent.
- `hf:openai/gpt-oss-120b`:
  40,825,
  30,
  0.07 percent.
- `minimax-m3`:
  73,956,
  253,
  0.34 percent.
- `deepseek-v4.1-flash`:
  78,100,
  438,
  0.56 percent.
- `glm-5.3`:
  3,202,
  23,
  0.72 percent.
- `hf:Qwen/Qwen3.8-27B`:
  16,340,
  194,
  1.19 percent.
- `hf:zai-org/GLM-5.3-Flash`:
  27,413,
  349,
  1.27 percent.

The four models at zero keep roster order among themselves.
The Claude cards are not in the table:
the Claude route is never a re-ask target.

The constant is `RUNAWAY_READINGS` in `provider/src/reask-runaway-readings.ts`,
and its TSDoc cites the census date,
each model's runaways and calls,
and the limits of the reading.
`alternateModelsFor` sorts by the share,
then by roster position.
For the roster-wide role and `deepseek-v4.1-flash` as the runaway the list is
`gemma-4-26b-a4b-it`,
`google.gemma-4-e2b`,
`inception/mercury-2.5`,
`hf:moonshotai/Kimi-K3`,
`minimax-m3`,
`glm-5.3`,
`hf:Qwen/Qwen3.8-27B` and `hf:zai-org/GLM-5.3-Flash`,
eight models,
so a runaway costs at most eight routed attempts before the fall back,
where B1072 counted eleven.

How a model with too few calls is placed.
A model with fewer than 100 calls since the caps,
or no row,
is placed after every measured model,
in roster order.
The 100 is the completion-cap rule's own floor (`completion-cap.ts`).
A share read from fewer calls says less than any measured share,
so it does not outrank one,
and no measured model waits behind a model whose runaways are not known.
No model in the table is under the floor:
`google.gemma-4-31b` has 214 calls,
the thinnest.

Limits:

- The share is of calls that reached the cap with no content,
  which is what the census counts and not the finish reason of each call.
- The calls of a model on the ids of the 1017 weights and before them are pooled for the DeepSeek seat (B1071).
- A model's share on a provider that serves only part of its calls may differ from its pooled share.
  GLM-5.3-Flash reads 4.61 percent on Synthetic (95 of 2,062) and 0.48 percent on Hyper (10 of 2,095).
- The census had just been fixed (B1070),
  and a pass that ran under heavier load would read other numbers.

Tests:
`reask-alternate-models.unit.test.ts` holds the order,
the tie rule and the placement of an unmeasured model,
and `reask-runaway-readings.unit.test.ts` (three cases,
added with the source scans of the round) holds the share and the floor.

Open to the owner's veto:
the order itself,
and the floor of 100 calls.

## B1076: a model seated on the bench of the question was a candidate for the voice of another seat

Status:
fixed between 01:50 and 02:40 UTC on 2026-10-31 in `35cd20d2a7` (provider) and `640dac1856` (stage),
and `37e025db0a` (pictures),
with the cases red first.
The lead's adoption of 2026-10-30,
following from recorded rules:
a model already seated on the bench of the question is left out of the alternates,
since its answer recorded as M's voice would count one model twice
("one judge alone never decides",
2026-10-25;
"Two judges must name it",
2026-10-25;
`roster-id.ts` on one model in two seats).
Open to the owner's veto.

What it does:

- The stage passes `modelIds`,
  the whole roster the gather names,
  withheld seats included,
  as the bench of every ask.
- The picture stage passes the readers of the picture.
- `alternateModelsFor` takes the bench and names each model it leaves out for it
  (`seated on the bench of this question`).
- When no model is left the fall back to M applies,
  with the warning.
- The recovery round also leaves out the model an earlier re-ask for the same seat went to,
  so it asks the next one:
  the uniqueness wrapper would replay the first model's claimed reply as a second voice.

A limit this round did not remove.
Two seats of one question that both ran away are each re-asked of the first model left in the order,
and the second's plain question is the first's digest,
so one claimed reply is recorded as two voices of two seats.
The provider re-ask of the first round has the same shape.
A claim shared across the seats of one gather would remove it
(each seat taking its model before it asks,
which one process can do without a lock),
and it was not built.
Whether to build it is the lead's decision.

## B1077: the plain question for another model, and what it does to the prompt digests

Status:
fixed between 01:50 and 02:40 UTC on 2026-10-31 in `35cd20d2a7` (provider) and `640dac1856` (stage),
with the cases red first.
The owner's answer of 2026-10-30:
"Plain question (Recommended)":
a different model gets the original question with no nudge;
the nudge stays only on the fall back to M.

What it does to the digests the payload store keys on
(`modelPromptDigest`,
the model id and the ordered messages):

- The ask of another model X is the digest of X and the original messages,
  which is the digest of X's own first ask of that question.
  A payload stored for that ask,
  by this run or by an earlier one,
  is replayed and X is not sampled again.
  Under B1072 the ask carried the nudge and had a digest of its own,
  so it was always a fresh sample.
- The fall back to M keeps its digests:
  the provider re-ask the digest of M and the question with `CROSS_PROVIDER_NUDGE`,
  and the recovery round the digest of M and the question with the `cut-short` nudge,
  so a payload stored for either still replays.
- No payload stored under the digests of B1072's nudged ask of another model exists:
  the build holding `4f20f05f15` was committed at 00:03 UTC on 2026-10-30,
  after the newest pass directory was written (11:44 UTC on 2026-10-29).
- The cases:
  `prompt-uniqueness-client.unit.test.ts` pins that the ask has no nudge and the digest of the other model's
  own first ask,
  and `nudged-reask.unit.test.ts` pins the nudge on the fall back alone.

## B1078: the pass logs the cap census reads are gone, so the order cannot be measured again

Status:
open,
for the lead.
Found at 01:41 UTC on 2026-10-31 running the census this round was asked to run
(`mise run //package/translation-repair/run:cap-census -- /var/home/user/temp/agent`).

It read 106,377 logs,
40 pass-run logs and 0 completed calls,
where the run of 00:18 UTC on 2026-10-30 read 113,645 logs,
447 pass-run logs and 582,710 completed calls.
Which 40 logs those are was not listed.
The directories of the runs from `XingZ6016` to `XingZ6022` are under the agent root,
and the `corpus-pass-<name>.log` file beside each is not,
nor is any file of that name under `/var/home/user` or `/var/tmp`.
So the lines this ledger cites for B1070,
B1072 and B1073 (`corpus-pass-XingZ6020.log:26575` and the rest) can no longer be read.
Why they went is not known.
The saved output of the earlier run,
kept in the scratch directory of the w382 fixer as `census-after-2.out`,
is what B1075's table is read from,
and it is the only copy.

Not done:
a count of runaways from the stored replies of the run directories
(`prompt-payloads` of each),
which survived.
That would be another measure than the census's and reads raw provider traffic;
the lead may choose it.

The next step:
a pass of the corpus,
or the logs restored from wherever they went,
then the census,
and the table replaced if its shares differ.
The census output should be saved beside the ledger entry that cites it.

## B1079: the callers of the gather name no role yet

Status:
open,
for the lead.

`gatherStageVoices` takes a `role`,
and no caller passes one,
so every gather asks the re-ask of a runaway as the roster-wide role:
it honours the cull and `judge-unmeasured`,
and not the holds of the wide judges,
the late judges,
the translators or the writers.
A model dropped from the wide seats can therefore be asked for the voice of a runaway critic,
panelist or select judge,
though the order puts the three such models (`glm-5.3`,
`hf:Qwen/Qwen3.8-27B`,
`hf:zai-org/GLM-5.3-Flash`) last,
so they are asked only when every model before them has no provider with budget.

The callers in the production code,
and the role the bench constants of `run-config.ts` suggest for each
(read from the constants and not from each caller):

- `translate-stage/src/translate-produce.ts`:
  `translator`.
- `consolidate/src/consolidate-produce.ts`:
  `writer`.
- `translate-stage/src/candidate-select.ts`,
  `adjudication/src/panel-stage.ts`,
  `adjudication/src/adjudicate-wire.ts`,
  `adjudication/src/resolution-wire.ts` and the critic stage
  (whose caller was not located this round):
  `wide-judge`.
- The lane contest,
  the consolidation slate and gate
  (their callers were not located this round):
  `late-judge`.
- The editors,
  refiners,
  restoration judge,
  coverage,
  pairing and admission rounds:
  `roster`,
  which is the default.
- The checkers:
  `checker`.

Each package that wires a role needs its own red case and its own gates,
and this round was not asked to touch them.
The picture path names its role (`reader`) in `reaskCut` itself.
