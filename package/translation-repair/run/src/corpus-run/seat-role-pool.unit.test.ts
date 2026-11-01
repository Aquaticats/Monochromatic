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
import {
  type BudgetView,
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
          // THE EDITORS AND THE REFINERS (ledger B1165) are seated from the run's static lists in every reading, so the run
          // withholds nothing from them by the meters. The two equalities guard that a derivation changing the list in one
          // of these readings fails here and asks for an entry in the list of benches; a derivation that only filters the
          // static list by who is seated (`filter(seated)`) passes unchanged and is not guarded (ledger B1206).
          expect(seats.repairModels.editorModelIds,).toEqual([...RUN_MODELS.editorModelIds,],);
          expect(seats.repairModels.refinerModelIds,).toEqual([...RUN_MODELS.refinerModelIds,],);
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
      name: 'NO MODEL OF THE CHECKER ORDER EDITS OR REFINES, so a checker\'s re-ask is never answered by a model that wrote '
        + 'the text it checks, and an answer recorded as a checker\'s voice never weighs in full for text its answerer wrote',
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
