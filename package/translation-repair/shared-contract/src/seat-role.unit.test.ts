/**
 Tests for the roles a card can hold a model out of, as a re-ask reads them (ledger B1074).

 A HOLD NAMES A ROLE, NOT A MODEL: `wide-seat-dropped` keeps a model off the nine-wide benches and leaves its
 translator seat alone. A re-ask asks a model of the same role the seat it stands in belongs to, so it reads the holds
 of that role and no others.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  holdKeepingOutOf,
  ROSTER_MODEL_IDS,
  SEAT_ROLE_HOLDS,
  SEAT_ROLES,
} from '../dist/final/node/index.mjs';

//region Seat role tests

await describe({
  name: holdKeepingOutOf.name,
  children: [
    it({
      name: 'NAMES SEVEN ROLES, each with the holds it honours, and the roster-wide role honours the two that empty a '
        + 'bench for every role but the checker and the reader',
      fn: async () => {
        expect([...SEAT_ROLES,],).toEqual([
          'roster',
          'translator',
          'writer',
          'wide-judge',
          'late-judge',
          'reader',
          'checker',
        ],);
        expect(SEAT_ROLE_HOLDS.roster,).toEqual(['owner-culled', 'judge-unmeasured',],);
        expect(SEAT_ROLE_HOLDS.checker,).toEqual(['owner-culled',],);
        expect(SEAT_ROLE_HOLDS.reader,).toEqual(['owner-culled', 'reader-unmeasured',],);
      },
    },),

    it({
      name: 'HOLDS THE CULLED MODEL OUT OF EVERY ROLE, since the owner\'s cull empties every bench',
      fn: async () => {
        for (const role of SEAT_ROLES) {
          expect(holdKeepingOutOf({ modelId: 'hf:openai/gpt-oss-120b', role, },),).toBe('owner-culled',);
        }
      },
    },),

    it({
      name: 'READS ONLY THE HOLDS OF THE ROLE ASKED: Qwen3.8-27B is held off the wide judges and the late judges but '
        + 'not off the translator, writer or reader seats it keeps',
      fn: async () => {
        expect(holdKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'wide-judge', },),).toBe('wide-seat-dropped',);
        expect(holdKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'late-judge', },),).toBe('late-judge-dropped',);
        for (const role of ['translator', 'writer', 'reader', 'checker', 'roster',] as const) {
          expect(holdKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role, },),).toBeUndefined();
        }
      },
    },),

    it({
      name: 'HOLDS THE DROPPED TRANSLATOR OUT OF THE TRANSLATOR SEAT ONLY, and the unmeasured writer out of both writing '
        + 'seats',
      fn: async () => {
        expect(holdKeepingOutOf({ modelId: 'inception/mercury-2.5', role: 'translator', },),).toBe('translator-dropped',);
        expect(holdKeepingOutOf({ modelId: 'inception/mercury-2.5', role: 'writer', },),).toBeUndefined();
        expect(holdKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'writer', },),).toBe('judge-unmeasured',);
        expect(holdKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'translator', },),).toBe('judge-unmeasured',);
      },
    },),

    it({
      name: 'LETS THE CHECKER AND THE PICTURE READER KEEP WHAT THE JUDGE PROBE WITHHELD: google.gemma-4-31b reads and '
        + 'xiaomi/mimo-v2.6-flash checks, though neither is a measured judge; the unmeasured reader is held off the reader role',
      fn: async () => {
        expect(holdKeepingOutOf({ modelId: 'google.gemma-4-31b', role: 'reader', },),).toBeUndefined();
        expect(holdKeepingOutOf({ modelId: 'google.gemma-4-31b', role: 'checker', },),).toBeUndefined();
        expect(holdKeepingOutOf({ modelId: 'google.gemma-4-31b', role: 'roster', },),).toBe('judge-unmeasured',);
        expect(holdKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'checker', },),).toBeUndefined();
        expect(holdKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'reader', },),).toBe('reader-unmeasured',);
        expect(holdKeepingOutOf({ modelId: 'deepseek-v4.1-flash', role: 'reader', },),).toBe('reader-unmeasured',);
      },
    },),

    it({
      name: 'ANSWERS FOR EVERY ROSTER MODEL AND EVERY ROLE with a hold the role honours or nothing, never a hold the '
        + 'role does not honour',
      fn: async () => {
        for (const modelId of ROSTER_MODEL_IDS) {
          for (const role of SEAT_ROLES) {
            /**
             What the card keeps this model out of this role by.
             */
            const hold = holdKeepingOutOf({ modelId, role, },);
            expect(hold === undefined || SEAT_ROLE_HOLDS[role].includes(hold,),).toBe(true,);
          }
        }
      },
    },),
  ],
},);

//endregion Seat role tests
