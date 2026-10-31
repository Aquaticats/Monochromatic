/**
 Tests for the role each asking of a bench names in this package (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench: the consolidation writers are the writer role, and the gate
 and the polish gate are asked of the late judges.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  roleNamedInLiteral,
  rolesNamedAtCalls,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Consolidate role tests

await describe({
  name: 'the role the consolidation names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE WRITER ROLE where the writers are asked for a consolidation',
      fn: async () => {
        const source = await readFile(new URL('consolidate-produce.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['writer',],);
      },
    },),

    it({
      name: 'NAMES THE LATE-JUDGE ROLE where the gate is asked',
      fn: async () => {
        const source = await readFile(new URL('consolidate-gate-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runWindowedRounds', },),).toEqual(['late-judge',],);
      },
    },),

    it({
      name: 'NAMES THE LATE-JUDGE ROLE where the polish gate is asked',
      fn: async () => {
        const source = await readFile(new URL('consolidation-polish-gate-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runWindowedRounds', },),).toEqual(['late-judge',],);
      },
    },),

    it({
      name: 'ASKS THE AUTHORS ABOUT THEIR FINDINGS IN THE WRITER ROLE and shares with the gather that wrote the proposals '
        + 'the one record of the models asked',
      fn: async () => {
        const source = await readFile(new URL('consolidate-produce.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'repairInvalidCandidates', },),).toEqual(['writer',],);
        expect(rolesNamedAtCalls({ source, callee: 'repairInvalidCandidates', key: 'reaskClaims', },),).toEqual([
          'reaskClaims',
        ],);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', key: 'reaskClaims', },),).toEqual(['reaskClaims',],);
      },
    },),

    it({
      name: 'NAMES THE LATE-JUDGE ROLE where the consolidation slate is judged, in the argument object both asks share',
      fn: async () => {
        const source = await readFile(new URL('consolidate-settle-judge.ts', import.meta.url,), 'utf8',);
        expect(roleNamedInLiteral({ source, name: 'judging', },),).toBe('late-judge',);
      },
    },),

    it({
      name: 'NAMES THE ROLE OF THE BENCH THE POLISH IS HANDED, the select judges, which are the wide judges and not the '
        + 'late judges',
      fn: async () => {
        const source = await readFile(new URL('consolidation-polish-round.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runRefineStage', key: 'judgeRole', },),).toEqual(['wide-judge',],);
      },
    },),
  ],
},);

//endregion Consolidate role tests
