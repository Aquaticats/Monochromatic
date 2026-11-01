# Translation repair handover 2026-09-06: NeuralWatt as a provider, 2026-10-20

NeuralWatt joined the pipeline on 2026-10-20 as its first provider:
the owner's terms,
the meter and the stop that was built and removed,
the day boundary and account-wide count the owner then ruled,
the move to the DeepSeek V4.1 Flash 1017 weights,
and the first run on it,
`zheermao19`.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## The files that continue this record

This file holds the owner's terms and the record up to the first run on NeuralWatt,
`zheermao19`.
The current state and the next steps are in
[the landings, the zheermao22 restart and the state after K4](translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-landings-the-zheermao22-restart-and-the-state-after-k4.md).
The rest of the record is filed by span,
oldest first:

- [The NeuralWatt rotation reads and the first fix rounds, 2026-10-21](translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md):
  the reads of `zheermao20` and `CuspariaKLSY20`,
  the seating investigation,
  the w253 and w254 fixes,
  the archive block review round,
  and the reads of `zheermao21` and `CuspariaKLSY21`.
  It opens with the "Next" section this file once ended on.
- [The reboot, Gemma measured and the katex shim, 2026-10-21](translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-reboot-gemma-measured-and-the-katex-shim.md):
  the reboot,
  the owner's rulings on the picture bench and on request parameters,
  the Gemma measurement,
  and the katex shim.
- [The landings, the zheermao22 restart and the state after K4, 2026-10-21](translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-landings-the-zheermao22-restart-and-the-state-after-k4.md):
  the landings of w262 and w260,
  the runs `zheermao22`,
  `CuspariaKLSY22`
  and `XingZ6017`,
  the classes K1 to K4,
  the queue after them,
  and the state after K4 landed.

## The owner's terms

The owner added `TRANSLATION_REPAIR_NEURALWATT_API_KEY` on 2026-10-20 with
"550Wh of energy on DeepSeek V4.1 Flash and GLM 5.3 Flash per day"
and "we need to meter (using estimations) ourselves because overage eats into my normal plan".
The 550 Wh is the Max plan's 500 Wh plus the 50 Wh for product-news email
(`https://docs.neuralwatt.com/billing/daily-free-energy`).
That page says it resets at 00:00 UTC;
for this account the day starts at the quota's anchor instead
(the section "The day boundary,
the account-wide count and the key allowance").
It does not carry over,
covers each covered model's standard and flex lanes and not `-speed`,
and once spent requests bill the plan.

Served ids `deepseek-v4.1-flash` and `glm-5.3-flash`,
standard lane only;
first in the provider order,
by the rule that expiring money is spent first
and by the owner's ruling of 2026-10-20:
"Use the neuralwatt free energy front and center."
Every seat of the two models goes to NeuralWatt first in every role it already holds;
no new roles,
and the owner's 2026-09-02 unseating of GLM-5.3-Flash as a judge stands.
The meter's margin is kept as small as can be justified,
so free energy is not left unspent.
The encrypted store `.env.local.json` went to the branch worktree on 2026-10-20 (main's copy,
a strict superset of the old one by key names);
the scratch gate worktrees stay keyless.

## The meter as first built

No endpoint was then known to report what is left of the free energy
(`/v1/quota` reports the plan's `kwh_used`,
`/v1/usage/energy` the account's days),
so the pipeline keeps its own meter,
as the Bedrock ledger does:
reserve an estimate per call from the energy matrix's prompt-size buckets,
settle with the stream's `: energy` comment,
and read `/v1/quota` at start and end of a run.

It was built in keyless worktree w242
(base `7900a9a639`,
16 commits `8fca5003b7` to `ce1b710326`,
then docs and test commits to `0a4570d563`);
handback `/var/home/user/temp/agent/x15-fix/neuralwatt/handback.md`;
decision record `doc/decision/translation-repair-neuralwatt-provider.md`
(no cache version moves,
veto-open);
ledger P15 in `package/translation-repair/doc/audit-ledger-providers-routing-and-seating.md`.
Gate g129 on `ce1b710326`:
suite 2630 pass and 31 fail,
all 31 in `source-scan`'s scans finding the new code
(fixed in w242);
oxlint red in 21 packages on `prefer-readonly-parameter-type` alone
(105 errors),
tuple-destructured callbacks in files the branch never touched.
Main's TypeScript 7.1.0-beta move (`27b1cf2eec`) landed after the green gate g128,
and baseline gate g130 on main `a4987be78a` is red the same way:
the same 105 errors in the same 21 packages,
suite 2608 pass and 0 fail.
`doc/troubleshooting/typescript-is-tuple-type-tuple-reference.md` explains them:
7.1 hands the plugin the tuple reference,
so mutable tuples from `Object.entries` and `Map.entries` callbacks now surface,
and the offers are sound.
Landed as `1dde8c5cd6` after gate g131
(suite 2631 pass and 0 fail,
lint red only on the inherited 105).

## The first keyed call and the stop that was removed

`zheermao18` launched at 22:22 UTC on that build;
its first NeuralWatt call used 0.0167 Wh,
a tiny share of the day's 550 Wh,
yet reported a `request_cost_usd` of 0.000107.
The meter as built never looked at the energy here:
it took any cost above zero as proof that the free energy was spent and the plan was being charged,
so after that one call it marked NeuralWatt out of energy until 00:00 UTC.
That rule was the defect.
The run went on spending Hyper until it was stopped at 22:30 UTC after 21 Hyper calls.
The owner read the dashboard
("Your call only used 60.3J,
which isn't even 0.0%")
and ruled:
"From here on just use NeuralWatt normally,
I can tolerate some potential losses.
If it works wrongly I'll tell you."
The stop was removed
(red `265cb0d224`,
fix `05330e45c4`,
docs `1c4c10b4d1`;
legacy ledger lines that carry `billedUsd` read as plain energy lines),
landed as `5b5dca49a9` after gate g132
(suite 2648 pass and 0 fail,
lint red only on the inherited 105);
record and ledger P16 landed in `745b51e38d`.
The branch worktree's build digest matched w243's:
`sha256-tree-v1:f706c65f04541d1d043d5d39c4597ef034b537c8a73e827ef8c937da7c6d4ee9`.

The 105 `prefer-readonly-parameter-type` findings were fixed type-only in keyless worktree w246
(explicit deeply readonly entry types at the package's own boundaries,
never by loosening the rule;
the build digest unchanged),
folded into one `ReadonlyEntry` in `shared-contract` (`d390fd0f86`),
and landed as `506972d0e4` after gate g133
(2631 pass and 0 fail,
every lint step exit 0).
Four of its commit bodies claimed more than ran;
corrective comments 205886719,
205886729,
205886731 and 205886736 say what did.

## The day boundary, the account-wide count and the key allowance

The owner then ruled
(about 23:15 UTC):
`The stop logic should obviously count total free energy used per day. Every neuralwatt day starts at 15:15`.
Read-only calls confirmed both
(`/v1/quota` `current_period_start` `2026-10-20T19:15:08Z`,
which is 15:15 EDT;
`/v1/usage/by-model` account-wide covered energy 4.095 Wh against the ledger's 4.094 Wh;
record section "The day boundary and the account-wide count",
ledger P17).
The fix is being built in keyless worktree w247:
the day anchored at the quota's time of day,
spent energy the account-wide covered total since the anchor,
never below the ledger,
plus calls in flight.
`zheermao19` kept the old build;
its 00:00 UTC ledger reset would hand back about 20 Wh NeuralWatt still counts,
far inside the 550 Wh,
so it was not stopped.

The owner then set a 1 kWh daily limit on this key
("I've also just setup a 1kWh daily limit on this specific api key");
`/v1/quota` still reported `key.allowance: null` at 23:15 and 23:20 UTC,
and `/v1/usage/by-key` `budget_limit_usd: null` on both keys,
so w247 also reads the documented USD allowance (`blocked` makes NeuralWatt dry until `reset_at`)
and prints it on the `METERS` line.

## The DeepSeek V4.1 Flash 1017 weights

The owner asked:
"Migrate to deepseek v4 flash 1017 where possible."
Catalogs read at about 23:30 UTC:
NeuralWatt's `deepseek-v4.1-flash` already serves `deepseek-ai/DeepSeek-V4.1-Flash-1017`;
Hyper lists `deepseek-v4.1-flash-1017`,
OpenRouter `deepseek/deepseek-v4.1-flash-1017`
(Morph serves it at 0.158 and 0.6),
Synthetic `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`
(the card has no Synthetic side;
none is added).
Keyless worktree w248 moved the card's Hyper and OpenRouter served ids,
keeping the roster id `deepseek-v4.1-flash` as the card's name
(red `ad616b6a84`,
fix `dadeb42ba6`,
docs `1d610cd05e`),
landed as `d8cf1933f4` after gate g134 on the merge with main
(2648 pass and 0 fail,
every step exit 0).
`hyperIdFor` had matched on the Hyper id and would have read the seat as unserved,
so `HyperModelInfo` now carries `rosterId`;
`hyper-price.ts` gained a `deepseek-v4.1-flash-1017` row,
since the `SPEND` label is the wire id.

No cache version moves.
No cache key holds a served id,
and the weights behind the name changed,
but the slice cache lives inside each run's own directory
(`package/translation-repair/run/src/corpus-run/runs-layout.ts`),
so a new run starts with none of the old weights' decisions.
NeuralWatt's side served the 1017 weights before the migration too.

## The first run on NeuralWatt: zheermao19

`zheermao19` (entry zheermao101) ran from 23:00 UTC on 2026-10-20 on the digest of `5b5dca49a9`
and settled at 23:35 UTC in 35.6 min
(`TALLY zheermao101 status=SETTLED ... repairAccepted=13 ... pageChanged=9`);
its read is in the pass log of 2026-10-20,
the full draft at `/var/home/user/temp/agent/x15-fix/reads/zheermao19-read.md`.
NeuralWatt served 197 calls for 28.7 Wh measured,
every estimate running 2 to 6 times its measured figure;
the run-end `NeuralWatt PLAN ENERGY SPENT: 345.2 Wh` line is the owner's other key
(`mimo-v2.6-pro` rose 335 Wh between 23:15 and 23:36 UTC by `/v1/usage/by-model`).
Hyper reached `hyperBalance=0` and Synthetic `syntheticWeekly=0%` during the run;
Hyper is not recharged.

The Bedrock key has expired
(HTTP 401 "The security token included in the request has expired.",
already on `zheermao17`'s three Gemma seats),
and the owner ruled on 2026-10-20 to leave Bedrock off rather than issue a new key.
The run then withheld each Gemma seat for the whole run after 4 refused asks,
though `gemma-4-26b-a4b-it` and `google.gemma-4-31b` have OpenRouter sides with no holds
and OpenRouter was wet:
a credential refusal belongs to the provider,
not the seat.
Keyless worktree w249 makes it hold the provider out for the run and move each seat to its next provider
(red `5e07a8bb6d`,
fix `786013e4b7`,
guards `de44d19ea7`,
docs `7863b003b1`).
Withholding the seat was a default and not a ruling:
ledger B685 says "there is no rule for a 401".
A second round closed its four gaps
(red `987b4ef553`,
fix `e9d0c5cd6f`,
docs `d3c8727aad` and `f1f7c15d94`):
a 401 on the cross-provider re-ask holds the provider too,
the warn names only providers that can take the call,
reach and quorum readers read a held provider as dry,
and `corpus-pass` builds one router per run,
so its holds are shared across the run.
Landed as `624c5ff583` after gate g135
(2657 pass and 0 fail,
every step exit 0);
ledger P18 landed in `803c85d603`.
`google.gemma-4-e2b` is Bedrock-only and stays out.
The rotation waits for the combined build,
since a run now would lose those seats again and spend OpenRouter credit on a build about to change.

The account-wide count in w247 also had to take in tracking aliases:
a call to `nw-flash`
(which points at `deepseek-v4.1-flash`)
is listed in `products[]` under `requested_model` `nw-flash`
(`https://docs.neuralwatt.com/guides/tracking-aliases`),
and the first w247 filter counted only the served ids and their `-flex` twins.
The w247 agent was asked at 00:40 UTC on 2026-10-21 to count alias rows whose alias points at a covered model,
read from `GET /v1/models`.
