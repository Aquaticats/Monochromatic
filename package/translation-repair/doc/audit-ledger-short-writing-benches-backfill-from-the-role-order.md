# Audit ledger: short writing benches backfill from the role order

Corpus pass `XingZ6024` (2026-11-01,
build `e9b966af55`) stopped both attempts with
`JUDGE SEATS phase=lanes writing bench unreachable: refiners 1 of 3 reachable, floor 2`
and shipped only an interim page (88 archive slices,
30 source,
0 settled).
Round w403 (2026-11-01 UTC) answers it with the owner's decision of that day:
"Backfill,
and add sonnet".

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

The owner's rules that decide every item here:
the writing-bench floor stays two reachable seats (2026-09-08,
`doc/decision/translation-repair-writing-bench-floor.md`);
"Never wait,
relaunch" (2026-10-29);
"no matter what,
we must ship";
alternates come only from the role's measured run list,
a withheld model is never that role's stand-in and every hold is honoured (2026-10-30);
the role order is "Order by role standing (Recommended)" (2026-10-31);
Sonnet runs on the owner's Claude key with no effort sent,
as the editor seat does (the owner declined to pick one:
"Why don't you bench?");
Qwen3.8-27B was removed from the wide seats only (2026-10-29),
so it may write.

## B1280: the editor and refiner benches are derived from the reading

On 2026-11-01 the checker bench was derived from the reading
(the best-ranked checkers a wet provider serves,
padded to the width,
ledger B711),
and the editor and refiner benches were the static lists in `run-config.ts`.
`XingZ6024` read NeuralWatt,
Hyper,
OpenRouter and Bedrock dry:
GLM-5.3-Flash (NeuralWatt,
Hyper,
OpenRouter) and minimax-m3 (Hyper,
OpenRouter) had no wet provider,
DeepSeek V4.1 Flash alone was reachable,
and Kimi-K3 and Qwen3.8-27B,
both measured refiners and both served by the wet Synthetic,
sat unused.

The fix is `writingBenchFor` in `run/src/corpus-run/run-writing-backfill.ts`,
called twice in `judgeSeatsFor`:
the static seats a wet provider serves,
then the next models of the role order that a wet provider serves and the run does not withhold,
until the bench reaches its static width,
then the static seats nobody serves.
The result replaces the static lists in `JudgeSeats.repairModels`,
which is the one place the floor (`benchesOf`),
the unmet-floor readings,
the stage rosters (`pass-lanes.ts`),
the consolidation polish (`configuredConsolidationPolish`) and both re-seating hooks take their benches from,
and in the `assertCheckerIndependence` call of the derivation.
The floor stays two and counts the derived bench.

Red on `e9b966af55` (commit `0fcd60be04`):
6 of the 7 `judgeSeatsFor` cases,
both `readPhaseSeats` cases and both re-seating cases of `pass-reseat-writing-bench.unit.test.ts` fail,
for the reason named in each name
(`refiners 1 of 3 reachable, floor 2` for the XingZ6024 reading).
The expectation is worked out by `writing-bench-expectation.test-fixture.ts` from the static bench,
the role order and who serves what,
and a case compares it with the bench the run derives in each of the 256 readings of the eight meters.
Green at `0529595fed`.

## B1281: the role order is one list, not a copy

The brief asked for the order the owner chose on 2026-10-31
(editors Qwen3.8-27B,
Kimi-K3,
minimax-m3,
gemma-4-26b,
gemma-4-e2b,
Mercury;
refiners `glm-5.3`,
Kimi-K3,
Qwen3.8-27B,
gemma-4-26b,
gemma-4-e2b,
Mercury),
which is each role's pool less the static bench,
in the standing of `ROLE_STANDINGS` and then in the order of the share of runaways.
`standingOrderOf` returns the measured models alone,
and the tail comes from `alternateOrderOf` in `provider/src/reask-alternate-models.ts`,
which is private.
`alternateModelsFor` is the public function that applies both,
and it also leaves out the Claude route (metered on the owner's key),
the decision-only seats and every model a card holds out of the role,
so `backfillOrderOf` calls it with the first static seat named as the model that ran away
(it is on the bench and is left out either way) and the static bench as the bench of the question.
The order is computed once at load (`EDITOR_BACKFILL_ORDER`,
`REFINER_BACKFILL_ORDER`),
and a case in `run-writing-backfill.unit.test.ts` holds that its measured part is `standingOrderOf` of the pool.
No list of model names is copied.

## B1282: a candidate that also checks

`assertCheckerIndependence` refuses a checker that also edits or refines unless the run permits it,
and the run does (`checkerSelfCertificationPermitted: true`,
the owner's decision of 2026-08-24:
"enable the discount and let every model do both").
What the contract does not say is where the discount lands when a checker's answer is recorded under another seat.
A checker's ballot is weighed by the seat that cast it (`tallyResolutionChecks` keys `wroteTextForIssue` by the ballot's
checker id),
and the re-ask of a seat that ran away is recorded under that seat,
so a model of the measured checker order that wrote the text,
sits off the checker bench and answers a re-ask for a seat it is not,
weighs in full for text it wrote.
`seat-role-pool.unit.test.ts` already pins that no model of the checker order is a static editor or refiner,
for that reason.

The rule chosen:
a candidate of the measured checker order is seated only when the checker bench of the same reading holds it,
where its own ballot is the one the discount halves,
and only while the run permits the overlap;
otherwise it is skipped and the next candidate of the order takes the place.
In the XingZ6024 reading Kimi-K3 sits on both benches by this rule.
Both branches are held by `run-writing-backfill.unit.test.ts`
(the run's own flag is true,
so the branch where the contract refuses is reached only through the function).
The other choice,
skipping every model of the checker order,
would have cost Kimi-K3 the second refiner place in every reading that serves it,
for an overlap the owner permitted by name.

## B1283: a withheld candidate

The run withholds `hyper-slow` models from the seats it derives from the roster while Hyper would serve them,
and `ROUTE_TIME_HOLDS_OF_ROLE` leaves the editors and refiners out of that,
because they were hand-picked benches.
A candidate is seated only when `seated` in `judgeSeatsFor` says so,
the answer every other derived bench takes,
so Qwen3.8-27B is skipped for the next model of the order while Hyper would serve it and seated while Synthetic does
(the owner removed it from the wide seats only).
Models held out of the role by a card (`owner-culled`,
`judge-unmeasured`,
`writer-unmeasured`) are not in the pool.

Open:
the re-ask pool of the editor and refiner roles still holds Qwen3.8-27B whatever serves it,
so a re-ask can ask it where the backfill would not seat it.
Adding `hyper-slow` to those two roles in `ROUTE_TIME_HOLDS_OF_ROLE` would make the two agree and take the top editor
stand-in of the owner's 2026-10-31 order out of every re-ask,
a change of an answer the owner gave,
so it is the owner's to say.

## B1284: Sonnet as a refiner

`claude-sonnet-5-5` is the second static refiner,
as it is the second static editor,
with the owner's words in the comment and the note that it is unmeasured in the role.
`EDITOR_LANE_REFINERS` takes the run's refiners by name,
so the next `editor-calibrate` seats it in its refiner chair without an edit.

A fourth refiner moves nothing in the arithmetic the lane lives on:
the quorum of a bench of four is two,
as of three (`rosterQuorumSize`),
the last-seat grace needs a quorum greater than two (`reachableQuorum`),
the first round asks the quorum and the spare in both cases (`firstRoundWindow`),
and Sonnet holds `judge-unmeasured`,
so it judges no slate and the half-weight rule for a writer judging its own text has no case to apply to.
Cost lines count Sonnet's calls as they count the editor seat's.

## B1285: what the owner's answer changed for the readings that stop

Bedrock alone,
the reading that founded the floor (the eighth hakureico pass),
stops the lanes no longer:
Bedrock serves `gemma-4-26b-a4b-it` and `google.gemma-4-e2b`,
both in the pool of both roles,
so it reaches a pair of editors and a pair of refiners from the role order.
The editor calibration recorded `gemma-4-26b-a4b-it` last of its six standings in both roles
(7.0 percent as an editor,
3.4 as a refiner),
and `google.gemma-4-e2b` unmeasured in both.
That is a consequence of "Backfill",
recorded in the decision for the owner to veto.

Over all 256 readings of the eight meters,
8 now stop the lanes on a writing bench,
and the translate lane stops in the same 8:
the readings in which the wet providers are each subset of DeepSeek,
Xiaomi and Anthropic,
the empty one included.
In 6 of them the editors and refiners are under the floor,
and in 2 (DeepSeek with Anthropic,
and the three together) only the translators are.
No reading leaves the editors and refiners short with the translators and select judges at quorum,
which the case "reads each lane under its own phase" of `pass-reseat.unit.test.ts` relied on;
it now reads DeepSeek alone on one rig for each lane.
Seven test files of `run` read Bedrock alone,
Synthetic alone or the NeuralWatt shape as a stop;
they read DeepSeek alone now,
with a `deepseekWet` switch on `clientBefore` for the cases that need NeuralWatt dry and a writing provider wet.

## B1286: the lines

The `JUDGE SEATS` line counts the editors and refiners
(`editors=<n> refiners=<n>`)
and names the seats taken from the role order
(`backfilled-by-role-order=none`,
or `editors:<ids>|refiners:<ids>` for the benches that took seats).
The stop line,
the line of a phase that goes on without its writers and the line after a wait say
`backfilled by role order: <words>` between the clauses and the verdict.
The chunk re-seating line names the editors,
the refiners and the same words.
No provider text is in any of them.

## B1287: the readers of the editor and refiner lists

`rg 'RUN_MODELS\.(editor|refiner)ModelIds'` over the family finds these readers of the static lists,
each kept or changed on purpose:

- `run-config.ts`:
  the declaration,
  the load-time independence check,
  the one-wire-model rule of `RUN_BENCHES` and `RUN_WRITING_PROVIDERS`.
  The last now also counts the providers of both roles' pools,
  so a provider that serves only a model the role order may seat counts as a writing provider
  (no change today:
  every provider but Xiaomi).
- `run-seats.ts`:
  the derivation reads them as the width and the first choice,
  and the independence check reads the derived benches.
- The calibrations and probes that measure the static list on purpose
  (`editor-calibrate-lane.ts`,
  `editor-width-probe-run.ts`,
  `editor-width-slice.ts`,
  `translate-probe.ts`,
  `sentinel-probe.ts`):
  unchanged.
- Test files:
  `seat-role-pool.unit.test.ts` and `owner-cull.unit.test.ts` read the static lists as the declaration,
  and their cases that assumed the derived bench equalled the static one were changed (B1285).

Every other reader takes `JudgeSeats.repairModels`.

## B1288: the cache versions

The rule is step 4 of `mistake-prevention.md` (M25,
M28).
The audit
(`slice-cache:cache-account-audit -- --runs-under` the agent folder,
run on the tree of this round after its code commits)
lists the eight constants at the values of B1263
(consolidate 44,
consistency 10,
lane contest 19,
refine 20,
repair slice 51,
page title 9,
pairing 3
and translate slice 37)
and 14,064 slice-cache records under 435 runs directories,
the newest 2026-11-01T07:42Z (`XingZ6024`),
with 0 of 8 versions set after it.
Three directories could not be listed (`EACCES`,
all `pgvector` scratch folders of 2026-09-03),
which leaves their runs out of the count.

- No version moves.
  The change decides which seats a run asks,
  and the keys already hash them:
  `repairRunShape` (repair slice 51) hashes the editors,
  the refiners and the checkers in order,
  `refineRunShape` (refine 20) the refiners,
  and the consolidation key (consolidate 44) the polish refiners.
  A derived bench keys apart from any other list,
  and a list that equals an older key's asks the same question as before,
  so no stored decision is read under a question it was not made for.
  Adding Sonnet to the static refiners lengthens the list from three to four,
  which keys apart from every key made before.
  No sheet,
  floor,
  threshold or settlement rule of a cached stage changes.
- Lane contest 19,
  translate slice 37,
  consistency 10,
  page title 9 and pairing 3 read neither list.
- The audit lists the code commits of this round as source commits no account names,
  with every version listed as one they ride inside,
  and the version of each was set before the newest record,
  so a change that did reach a key would not ride inside them.

## B1289: found by the package suites, not by the red tests

The red commit's stop-line case named only the editors and refiners and failed on a stop line that also names the
translators (DeepSeek alone reaches one translator);
`0529595fed` reads every writing bench of the phase.
The full `test:unit` of `run` found 7 test files that read a stop where the lanes now go on,
and the full `test:unit` of `corpus-pass` found 11 cases in 5 files for the same reason
and the new chunk-line words.
`bench` passed unchanged (143 pass lines and no failure line).
The oxlint run of `run` found 19 warnings and 4 errors in the new code
(`cc1783a0e8`),
and that of `corpus-pass` 4 warnings (`e5b7538512`).
The whole source-scan suite found 4 scans' findings on the docs commit (fixed in `b48353727a`):
an import nothing read,
an exported type no other file imports,
7 TSDoc descriptions opening with an article,
and a poll bound in a test that the real-clock list does not hold.
Each was fixed in code,
with nothing added to a list.

## B1290: a static list beside a derived seating

The family is recorded in `mistake-prevention-shared-code-loops-and-keyed-tables.md`
under "A static list beside a derived seating":
the checker bench had followed the reading since 2026-09-27 and the two benches under the same floor had not.

## B1291: the slips of the round and what stays open

Slips:

- The red commit message calls the stop case green on `e9b966af55` ("guarded");
  it is red there (the stop line has no backfill words,
  and the bench order differs).
  It was not amended.
- The test commit and the fix commit were made before the linter ran on the new files,
  and the first lint run found 19 warnings and 4 errors (`cc1783a0e8`).
- A `for` loop with an empty body was typed into one command,
  a few `sed` edits ran without the keyless wrapper,
  and `tail` read the end of a type-check log.
  No key was read or printed.
- The whole source-scan suite ran twice,
  not once:
  the first run found the 4 scans' findings named in B1289,
  and the second ran on the commit that fixed them.
- The message of `b48353727a` says 5 scans where the log shows 4 failing scans (8 failure lines,
  2 for each).
  It was not amended.
- A test run was started on a hook that never ends
  (a hold that did not end,
  so the reading waited again and again)
  and was stopped by its task id;
  the case now ends the hold at the first dryness read.

Open:

- Bedrock alone now seats the two Gemma sizes as the editors and refiners (B1285).
  Whether that is wanted,
  or the two should stay off the writing benches,
  is the owner's to say.
- The re-ask pool of the editor and refiner roles and the backfill disagree about Qwen3.8-27B under Hyper (B1283).
- Sonnet 5.5 is unmeasured as a refiner,
  and the Claude key is metered:
  a run that backfills nothing still asks it as the second refiner in every reading in which the key is wet.
- `doc/audit/standards-review-gate-shadow.md` is modified in this tree by a hook that is not part of this round,
  and is not committed.
