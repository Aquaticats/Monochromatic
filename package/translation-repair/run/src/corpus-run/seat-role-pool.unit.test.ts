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
  rolePoolOf,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { readsImages, } from '@monochromatic-dev/translation-repair-provider';
import {
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
