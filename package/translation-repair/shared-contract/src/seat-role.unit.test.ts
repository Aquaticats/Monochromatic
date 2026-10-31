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
  holdsKeepingOutOf,
  ROSTER_MODEL_IDS,
  SEAT_ROLE_HOLDS,
  SEAT_ROLES,
} from '../dist/final/node/index.mjs';

//region Seat role tests

await describe({
  name: holdsKeepingOutOf.name,
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
          expect(holdsKeepingOutOf({ modelId: 'hf:openai/gpt-oss-120b', role, },),).toContain('owner-culled',);
        }
      },
    },),

    it({
      name: 'READS ONLY THE HOLDS OF THE ROLE ASKED: Qwen3.8-27B is held off the wide judges and the late judges but '
        + 'not off the translator, writer or reader seats it keeps',
      fn: async () => {
        expect(holdsKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'wide-judge', },),).toEqual([
          'wide-seat-dropped',
        ],);
        expect(holdsKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'late-judge', },),).toEqual([
          'late-judge-dropped',
        ],);
        for (const role of ['translator', 'writer', 'reader', 'checker', 'roster',] as const) {
          expect(holdsKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role, },),).toEqual([],);
        }
      },
    },),

    it({
      name: 'HOLDS THE DROPPED TRANSLATOR OUT OF THE TRANSLATOR SEAT ONLY, and the model no judge probe seated out of '
        + 'both writing seats',
      fn: async () => {
        expect(holdsKeepingOutOf({ modelId: 'inception/mercury-2.5', role: 'translator', },),).toEqual([
          'translator-dropped',
        ],);
        expect(holdsKeepingOutOf({ modelId: 'inception/mercury-2.5', role: 'writer', },),).toEqual([],);
        expect(holdsKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'writer', },),).toEqual([
          'judge-unmeasured',
          'writer-unmeasured',
        ],);
        expect(holdsKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'translator', },),).toEqual([
          'judge-unmeasured',
          'writer-unmeasured',
        ],);
      },
    },),

    it({
      name: 'LETS THE CHECKER AND THE PICTURE READER KEEP WHAT THE JUDGE PROBE WITHHELD: google.gemma-4-31b reads and '
        + 'xiaomi/mimo-v2.6-flash checks, though neither is a measured judge; the unmeasured reader is held off the '
        + 'reader role',
      fn: async () => {
        expect(holdsKeepingOutOf({ modelId: 'google.gemma-4-31b', role: 'reader', },),).toEqual([],);
        expect(holdsKeepingOutOf({ modelId: 'google.gemma-4-31b', role: 'checker', },),).toEqual([],);
        expect(holdsKeepingOutOf({ modelId: 'google.gemma-4-31b', role: 'roster', },),).toEqual([
          'judge-unmeasured',
        ],);
        expect(holdsKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'checker', },),).toEqual([],);
        expect(holdsKeepingOutOf({ modelId: 'xiaomi/mimo-v2.6-flash', role: 'reader', },),).toEqual([
          'reader-unmeasured',
        ],);
        expect(holdsKeepingOutOf({ modelId: 'deepseek-v4.1-flash', role: 'reader', },),).toEqual([
          'reader-unmeasured',
        ],);
      },
    },),

    it({
      name: 'ANSWERS FOR EVERY ROSTER MODEL AND EVERY ROLE with holds the role honours, never a hold the role does not '
        + 'honour',
      fn: async () => {
        for (const modelId of ROSTER_MODEL_IDS) {
          for (const role of SEAT_ROLES) {
            for (const hold of holdsKeepingOutOf({ modelId, role, },)) {
              expect(SEAT_ROLE_HOLDS[role].includes(hold,),).toBe(true,);
            }
          }
        }
      },
    },),
  ],
},);

//endregion Seat role tests
