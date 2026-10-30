# Audit ledger: reasoning runaways and the re-ask of another model

A reply that ran away reasoning,
re-asked of the same model on another route and running away again (B1072),
and the two re-asks of a length stop with no content that still ask the same model (B1073).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1072: a reply that ran away reasoning was re-asked of the same model on another route, and ran away again

Status:
fixed at 00:03 UTC on 2026-10-30 in `4f20f05f15` (provider),
the cases committed red first in `0b52a103b7` at 23:56 UTC on 2026-10-29,
and the logger of the loop tagged in `3b4c280885` after the red `21f3195739`.
The lead's decision of 2026-10-29 (open to the owner's veto):
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
- A runaway is re-asked,
  nudged,
  of the roster's other models in turn
  (`provider/src/reask-alternate-models.ts`,
  roster order,
  without the model that ran away,
  without decision-only seats and without the Claude route),
  until one has a provider with budget:
  a `NoProviderForModelError` for a model moves on to the next.
- Where none has,
  the model that ran away is the only one left to ask,
  and the re-ask is what it was before,
  on another provider,
  with a warning naming the model and how many were tried.
- The info line names the model,
  the reason and the model chosen
  (`deepseek-v4.1-flash: reasoning runaway on neuralwatt (a length stop with no content);
  re-asking <model> instead,
  nudged`),
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

Choices the lead owns,
each open to the owner's veto:

- The order is the roster's,
  and the first model that is not the runaway comes first.
  For `deepseek-v4.1-flash` that is `hf:zai-org/GLM-5.3-Flash`,
  the seat with the highest share of calls at the cap in the census of 2026-10-29 (B1070):
  Synthetic 100 of 2,062 since the caps (4.85 percent),
  OpenRouter 239 of 21,498 (1.11 percent),
  NeuralWatt 19 of 1,758 (1.08 percent),
  Hyper 10 of 2,095 (0.48 percent).
  An order by the lowest share at the cap would put it last.
  No card records which model suits a question,
  so the order is a placeholder until the lead says.
- The answer is recorded as the voice of the seat that ran away,
  and the model that gave it may sit on the same bench.
  `roster-id.ts` says why one model in two seats overstates agreement.
  The provider package does not know the bench,
  so it cannot leave such a model out.
  A stage that knows its bench could pass the models to leave out,
  and none does yet.
- The Claude route is left out because its key is metered ("be careful what you use").
  The other metered routes are not,
  since the router already spends on them for a first ask.
- The nudge reads "Your previous reply could not be used",
  which the other model never wrote.
  Its prompt is a digest of its own under that model's name,
  and a seat that already answered the question gets a fresh sample.
  A wording that does not claim a previous reply would be a new ruling of the kind P9 records.

Cache versions:
none moves.
The pass opens every slice-cache namespace under the digest of the built pipeline,
which hashes the first-party bytes under `dist/final/node`,
so a build holding this change discards the namespaces an older build wrote before it reads them
(`slice-reading/src/pairing-cache-version.ts`,
the account of 2026-10-18,
ledger B670).
`cache-account-audit`,
run at 00:03 UTC on 2026-10-30,
read 13,711 slice-cache records under 441 runs directories
(four directories could not be listed).
The newest is from 19:14 UTC on 2026-10-22 and seven of the eight cache versions were set after it,
so no record stands under those seven numbers and this change rides inside them.
The eighth,
the pairing version 3 set on 2026-09-28,
holds records,
and this change rides inside it on the account of 2026-10-18 and not on an empty directory,
as the "Rides inside 3 too" paragraphs of its TSDoc do.
The stored model replies of the prompt payload store are keyed by model and prompt and stay valid as replies.
The re-ask of another model is a prompt of its own,
and a stored re-ask of the same model is never asked for.

Tests:
`nudged-reask.unit.test.ts` (nine cases added,
seven of them red at `0b52a103b7`:
the re-ask to the first other roster model with no hint and the nudge last,
the choice for the first,
second and Claude seats,
the same-model re-ask kept for a content-holding mismatch,
a content-holding cut,
a refusal and an unparseable empty reply,
the move to the next model,
the fall back to the runaway with its warning,
the info lines,
the cause that names the model asked,
and the dry refusal;
the ninth,
the same-model line unchanged,
passes before the fix),
`provider-router.unit.test.ts` (the router's own `chatJson` re-asks a Qwen3.8-27B runaway on Synthetic of GLM-5.3-Flash,
which it routes to Hyper),
and `prompt-uniqueness-client.unit.test.ts` (the claim path asks another model with no hint,
a digest of its own,
and replays both on a later ask).
`reask-alternate-models.unit.test.ts` holds the list itself.

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

## B1073: two re-asks of a length stop with no content still ask the same model

Status:
open,
for the lead's decision.
Found on 2026-10-29 while fixing B1072.

The lead's decision of 2026-10-29 named the nudged re-ask (`nudged-reask.ts`,
`provider-router-reask.ts`).
Two other places ask the same model again after a length stop with no content:

- The stage's recovery round (`stage/src/stage-quorum.ts`,
  `recovery-nudge.ts`)
  re-asks every seat whose last answer was unreadable,
  with the `cut-short` nudge,
  through the same client,
  so it asks the seat's own model a second time.
  `XingZ6020` line 29966 is a direct DeepSeek call whose lost-voice line (29967) is tagged `[runRecoveryRound]`,
  and the round says what it re-asks
  (`recovery round for 1 unreadable answer (1 cut-short)`).
  It is the call the lead's brief pairs with the NeuralWatt runaway at line 26575,
  and the lines between the two,
  30 minutes apart,
  were not read to say which ask of the round each was.
  After B1072 the re-ask of the round's own ask goes to another model,
  but the round's ask of the seat's model is still made.
- `reaskCut` in `slice-reading/src/image-reading-past-refusal.ts` re-asks a picture reading
  whose reply the length limit cut empty
  of the same model on another provider (`otherThan`),
  and on the same provider where none serves it;
  `image-reading-cut-empty.unit.test.ts` pins both.
  A picture reader joins a route only after a keyed measurement of that route
  (`design-commitments-providers-and-seats.md`,
  the rule of 2026-09-08),
  so an unmeasured model cannot simply stand in for it.

Neither is changed.
