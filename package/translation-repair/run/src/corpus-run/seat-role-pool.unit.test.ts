/**
 Holds the pool a re-ask of each role draws from to the bench derivation the run seats that role from (ledger B1100).

 A RE-ASK ASKS A MODEL ITS ROLE'S LIST NAMES, and the lists are read in the shared contract (`rolePoolOf`: the roster
 less the models the role's holds keep out, and the measured checker order for the checker role). The run derives its
 benches in `run-config.ts` by its own filters over the same cards. The two are written in different packages, so
 each case here fails when a role's pool and its bench derivation drift apart: a hold added to one and not the other, a
 role whose bench the run changes, a checker order the run reads from another list.

 NOTHING HERE REACHES A PROVIDER.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isDecisionSeat,
  MEASURED_CHECKER_ORDER,
  MODEL_CARDS,
  reaskPoolOf,
  rolePoolOf,
  type RosterModelId,
  type SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  alternateModelsFor,
  type BudgetView,
  isWithheldFromRole,
  NO_PROVIDER,
  providerServing,
  reachOf,
  readsImages,
} from '@monochromatic-dev/translation-repair-provider';
// READ FROM THE BUILD, AS THE LINT RULE `test-import/require-eventual-artifact` REQUIRES (ledger B1151): `test:unit` does not
// build first, so this case reads the benches of the last build; `buildAndTest` builds and then tests.
import {
  judgeSeatsFor,
  RUN_CHECKER_ORDER,
  RUN_LATE_JUDGES,
  RUN_MODELS,
  RUN_READER_MODELS,
  RUN_ROSTER,
  RUN_TRANSLATORS,
  RUN_WIDE_SEATS,
  RUN_WRITERS,
} from '../../dist/final/node/index.mjs';

//region Seat role pool tests

/**
 Models of a pool a re-ask can ask: chat models, which is what every run bench holds.

 @param pool - pool of a role

 @returns Pool without a decision-only seat

 @example
 ```ts
 const chat = chatModelsOf({ pool: rolePoolOf({ role: 'roster', },), },);
 ```
 */
function chatModelsOf({ pool, }: { readonly pool: readonly RosterModelId[]; },): readonly RosterModelId[] {
  return pool.filter(function isChat(modelId,): boolean {
    return !isDecisionSeat({ modelId, },);
  },);
}

/**
 Models that are not on the Claude route.

 @param modelIds - models to filter

 @returns Models whose card names no Anthropic route

 @example
 ```ts
 const others = withoutClaude({ modelIds: RUN_MODELS.editorModelIds, },);
 ```
 */
function withoutClaude({ modelIds, }: { readonly modelIds: readonly RosterModelId[]; },): readonly RosterModelId[] {
  return modelIds.filter(function isNotClaude(modelId,): boolean {
    return MODEL_CARDS[modelId].anthropic === undefined;
  },);
}

/**
 The measured checkers a reading may have withheld: those of the order ranked above the lowest-ranked checker on the bench
 that a wet provider serves, or the whole order where the bench is padded with seats no provider serves (then every served
 checker of the order is on it already). A served checker below the bench is only beyond its width, not withheld.

 @param seats - benches the run derives in the reading

 @param dry - reading of the meters

 @returns Checkers of the order the reading could have withheld, in rank order

 @example
 ```ts
 const listed = checkersRankedAboveBench({ seats: judgeSeatsFor({ dry, },), dry, },);
 ```
 */
function checkersRankedAboveBench(
  {
    seats,
    dry,
  }: {
    readonly seats: ReturnType<typeof judgeSeatsFor>;
    readonly dry: BudgetView;
  },
): readonly RosterModelId[] {
  /**
   Whether a wet provider serves a model in this reading.

   @param modelId - model under question

   @returns True when the router would send its calls somewhere
   */
  function served(modelId: RosterModelId,): boolean {
    return providerServing({ reach: reachOf({ modelId, },), dry, },) !== NO_PROVIDER;
  }
  if (!seats.checkers.every(served,))
    return RUN_CHECKER_ORDER;
  /**
   Rank of the lowest-ranked checker on the bench.
   */
  const lowest = Math.max(...seats.checkers.map(function rankOf(modelId,): number {
    return RUN_CHECKER_ORDER.indexOf(modelId,);
  },),);
  return RUN_CHECKER_ORDER.slice(0, lowest + 1,);
}

await describe({
  name: rolePoolOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'THE POOL OF EACH ROLE THE CARDS DECIDE IS THE BENCH THE RUN DERIVES FOR IT: the roster, the translators, the '
        + 'writers, the wide seats, the late judges and the readers, in the same order',
      fn: async () => {
        expect(chatModelsOf({ pool: rolePoolOf({ role: 'roster', },), },),).toEqual([...RUN_ROSTER,],);
        expect(chatModelsOf({ pool: rolePoolOf({ role: 'translator', },), },),).toEqual([...RUN_TRANSLATORS,],);
        expect(chatModelsOf({ pool: rolePoolOf({ role: 'writer', },), },),).toEqual([...RUN_WRITERS,],);
        expect(chatModelsOf({ pool: rolePoolOf({ role: 'wide-judge', },), },),).toEqual([...RUN_WIDE_SEATS,],);
        expect(chatModelsOf({ pool: rolePoolOf({ role: 'late-judge', },), },),).toEqual([...RUN_LATE_JUDGES,],);
        expect(chatModelsOf({ pool: rolePoolOf({ role: 'reader', },), },).filter(function reads(modelId,): boolean {
          return readsImages({ modelId, },);
        },),).toEqual([...RUN_READER_MODELS,],);
      },
    },),

    it({
      name: 'THE CHECKER POOL IS THE CHECKER ORDER THE RUN SEATS ITS BENCH FROM, by identity of the list and not by a '
        + 'copy, and the bench is the first of it',
      fn: async () => {
        expect(RUN_CHECKER_ORDER === MEASURED_CHECKER_ORDER,).toBe(true,);
        expect(rolePoolOf({ role: 'checker', },),).toEqual([...RUN_CHECKER_ORDER,],);
        expect(RUN_MODELS.checkerModelIds.every(function inPool(modelId,): boolean {
          return rolePoolOf({ role: 'checker', },).includes(modelId,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'THE EDITOR AND REFINER POOLS ARE THE WRITERS THE EDITOR CALIBRATION SEATS IN THOSE CHAIRS, and the run\'s own '
        + 'editors and refiners, but for the Claude route, are in them',
      fn: async () => {
        for (const role of ['editor', 'refiner',] as const) {
          expect(chatModelsOf({ pool: rolePoolOf({ role, },), },),).toEqual([...RUN_WRITERS,],);
        }
        for (const modelId of withoutClaude({ modelIds: RUN_MODELS.editorModelIds, },)) {
          expect(rolePoolOf({ role: 'editor', },).includes(modelId,),).toBe(true,);
        }
        for (const modelId of withoutClaude({ modelIds: RUN_MODELS.refinerModelIds, },)) {
          expect(rolePoolOf({ role: 'refiner', },).includes(modelId,),).toBe(true,);
        }
      },
    },),

    it({
      name: 'NEVER ASKS A MODEL THE RUN WITHHELD FROM A ROLE\'S BENCH, in any reading of the meters (ledger B1143): a model the '
        + 'run keeps off the seats of a role while Hyper would serve it (Qwen3.8-27B, and Kimi-K3 on the select and slate '
        + 'seats) is in no re-ask pool of that role, unless no provider would serve it',
      fn: async () => {
        /**
         Every reading of the three meters the holds depend on, the other providers wet but for NeuralWatt, which follows
         Synthetic as it does in a run.
         */
        const readings: readonly BudgetView[] = [false, true,].flatMap(function bySynthetic(synthetic,) {
          return [false, true,].flatMap(function byHyper(hyper,) {
            return [false, true,].map(function byOpenRouter(openrouter,): BudgetView {
              return {
                neuralwatt: synthetic,
                synthetic,
                hyper,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter,
              };
            },);
          },);
        },);
        const findings: string[] = [];
        for (const dry of readings) {
          const seats = judgeSeatsFor({ dry, },);
          /**
           Each role beside the bench the run lists for it and the bench it derives in this reading.
           */
          const benches: readonly (readonly [SeatRole, readonly RosterModelId[], readonly RosterModelId[]])[] = [
            ['roster', RUN_ROSTER, seats.roster,],
            ['translator', RUN_TRANSLATORS, seats.translators,],
            ['writer', RUN_WRITERS, seats.writers,],
            ['reader', RUN_READER_MODELS, seats.readers,],
            ['wide-judge', RUN_WIDE_SEATS, seats.wideSeats,],
            ['late-judge', RUN_LATE_JUDGES, seats.lateJudges,],
            ['select-judge', RUN_WIDE_SEATS, seats.selectJudges,],
            ['slate-judge', RUN_LATE_JUDGES, seats.slateJudges,],
            // THE CHECKERS (ledger B1165): the bench is the first of the measured order that a wet provider serves, so a
            // served model ranked above the lowest served checker and off the bench was withheld by the reading.
            ['checker', checkersRankedAboveBench({ seats, dry, },), seats.checkers,],
          ];
          // THE EDITORS AND THE REFINERS (ledger B1165) were seated from the run's static lists in every reading, and the
          // run withheld nothing from them by the meters. Since the owner's answer of 2026-11-01 ("Backfill, and add
          // sonnet", ledger B1280) they are derived: the static seats a wet provider serves, then the models of the role
          // order that one serves and the run does not withhold, so the bench changes with the reading. What stays true
          // is the pool: every seat of a derived bench is a static seat or a model of the role's pool (no hold of a
          // card keeps it out), and a static seat is only ever moved behind the backfill, never dropped while served.
          //
          // AND THE RE-ASK (ledger B1330): a model the reading withholds from the role is on no derived bench and is no
          // alternate of the role in that reading, a model it does not withhold and that is on no bench is an alternate
          // (so Qwen3.8-27B is not dropped from every re-ask), and the function that says which is the one the roster
          // bench of the same reading agrees with.
          for (const [role, listed, derived,] of [
            ['editor', RUN_MODELS.editorModelIds, seats.repairModels.editorModelIds,],
            ['refiner', RUN_MODELS.refinerModelIds, seats.repairModels.refinerModelIds,],
          ] as const) {
            for (const modelId of derived)
              expect(listed.includes(modelId,) || rolePoolOf({ role, },).includes(modelId,),).toBe(true,);
            for (const modelId of listed.filter(function served(listedId,): boolean {
              return providerServing({ reach: reachOf({ modelId: listedId, },), dry, },) !== NO_PROVIDER;
            },))
              expect(derived,).toContain(modelId,);
            /**
             Models a re-ask of a seat of the role may ask in this reading.
             */
            const { alternates, } = alternateModelsFor({
              modelId: nonNullishOrThrow(listed[0],),
              role,
              bench: derived,
              routeTimeHolds: { applied: 'in-reading', dry, },
            },);
            for (const modelId of rolePoolOf({ role, },).filter(function offBench(poolModel,): boolean {
              return (!derived.includes(poolModel,))
                && (poolModel !== listed[0])
                && (MODEL_CARDS[poolModel].anthropic === undefined)
                && (!isDecisionSeat({ modelId: poolModel, },));
            },)) {
              /**
               Whether the reading withholds the model from the role.
               */
              const withheld = isWithheldFromRole({
                modelId,
                role,
                dry,
              },);
              /**
               The reading the finding names.
               */
              const readingText = JSON.stringify(dry,);
              /**
               Names the model, the role and the reading in a finding.
               */
              const where = `${role}: ${modelId}`;
              if (withheld && alternates.includes(modelId,))
                findings.push(`${where} is withheld by the reading but a re-ask may ask it (dry ${readingText})`,);
              if ((!withheld) && (!alternates.includes(modelId,)))
                findings.push(`${where} is not withheld by the reading but no re-ask asks it (dry ${readingText})`,);
              if (withheld && derived.includes(modelId,))
                findings.push(`${where} is withheld by the reading but the bench seats it (dry ${readingText})`,);
            }
            // NOT KNOWING THE READING WITHHOLDS EVERY MODEL THAT CARRIES A HOLD APPLIED AT SEATING TIME.
            for (const modelId of reaskPoolOf({ role, },))
              expect(MODEL_CARDS[modelId].holds.includes('hyper-slow',),).toBe(false,);
          }
          // THE ONE FUNCTION AGREES WITH THE ROSTER BENCH: a model a wet provider serves is on the roster bench of the
          // reading exactly when the function does not withhold it from the roster role.
          for (const modelId of rolePoolOf({ role: 'roster', },).filter(function served(poolModel,): boolean {
            return (providerServing({ reach: reachOf({ modelId: poolModel, },), dry, },) !== NO_PROVIDER)
              && (!isDecisionSeat({ modelId: poolModel, },));
          },)) {
            if (seats.roster.includes(modelId,) === isWithheldFromRole({ modelId, role: 'roster', dry, },))
              findings.push(
                `roster: ${modelId} is on the bench exactly when the function withholds it `
                  + `(dry ${JSON.stringify(dry,)})`,
              );
          }
          for (const [role, listed, derived,] of benches) {
            for (const modelId of listed.filter(function withheld(listedId,): boolean {
              return !derived.includes(listedId,);
            },)) {
              if (reaskPoolOf({ role, },).includes(modelId,)
                && (providerServing({ reach: reachOf({ modelId, },), dry, },) !== NO_PROVIDER)) {
                findings.push(`${role}: ${modelId} is withheld from the bench but in the re-ask pool (dry ${
                  JSON.stringify(dry,)
                })`,);
              }
            }
          }
        }
        expect(findings,).toEqual([],);
      },
    },),

    it({
      name: 'THE STATIC EDITORS AND REFINERS ARE NOT IN THE CHECKER ORDER, so no static writer is also a checker; the '
        + 'candidates that fill a dark place of those benches are held to the contract\'s overlap rule instead (ledger '
        + 'B1333, which records that the answerer\'s own ballot is discounted whether or not the lists overlap)',
      fn: async () => {
        for (const modelId of RUN_CHECKER_ORDER) {
          expect(RUN_MODELS.editorModelIds.includes(modelId,),).toBe(false,);
          expect(RUN_MODELS.refinerModelIds.includes(modelId,),).toBe(false,);
        }
      },
    },),
  ],
},);

//endregion Seat role pool tests
