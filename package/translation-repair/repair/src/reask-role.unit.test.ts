/**
 Tests for the role each asking of a bench names in this package (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench: the critics, the restoration judges and the slates of the
 repair lane are the wide judges, the checkers the checker role, and the editors and the refiners the roster role.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Repair role tests

/**
 Reads the role each real call of a function names in one source file of this package.

 @param file - source file beside this test

 @param callee - function called

 @returns Role named at each call, in source order

 @example
 ```ts
 const roles = await rolesIn({ file: './repair-stages.ts', callee: 'gatherStageVoices', },);
 ```
 */
async function rolesIn(
  {
    file,
    callee,
  }: {
    readonly file: string;
    readonly callee: string;
  },
): Promise<readonly string[]> {
  return rolesNamedAtCalls({
    source: await readFile(new URL(file, import.meta.url,), 'utf8',),
    callee,
  },);
}

await describe({
  name: 'the role the repair lane names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE EDITOR ROLE where the editors are asked, since their pool is the writers the editor calibration '
        + 'seats and not the whole roster',
      fn: async () => {
        expect(await rolesIn({ file: './editor-gather.ts', callee: 'gatherStageVoices', },),).toEqual(['editor',],);
      },
    },),

    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the critics are asked',
      fn: async () => {
        expect(await rolesIn({ file: './repair-stages.ts', callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
      },
    },),

    it({
      name: 'NAMES THE CHECKER ROLE where the checkers are asked',
      fn: async () => {
        expect(await rolesIn({ file: './repair-edit-stages.ts', callee: 'gatherStageVoices', },),).toEqual(['checker',],);
      },
    },),

    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the restoration judges are asked',
      fn: async () => {
        expect(await rolesIn({ file: './restoration-judge.ts', callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
      },
    },),

    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the derivability probe is asked',
      fn: async () => {
        expect(await rolesIn({ file: './derivability-probe.ts', callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
      },
    },),

    it({
      name: 'NAMES THE REFINER ROLE where the refiners are asked, and takes the role of the judges of their slate from '
        + 'the caller, since the repair lane seats the wide judges and the consolidation polish the late judges',
      fn: async () => {
        expect(await rolesIn({ file: './refine-stage.ts', callee: 'gatherStageVoices', },),).toEqual(['refiner',],);
        expect(await rolesIn({ file: './refine-stage.ts', callee: 'selectBestCandidate', },),).toEqual(['judgeRole',],);
      },
    },),

    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the slate of the editors is judged, at both of its asks',
      fn: async () => {
        expect(await rolesIn({ file: './editor-ensemble.ts', callee: 'selectBestCandidate', },),).toEqual([
          'select-judge',
          'select-judge',
        ],);
      },
    },),

    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the slate of the refiners is judged in the repair lane',
      fn: async () => {
        expect(await rolesIn({ file: './refine-slice-settle.ts', callee: 'runRefineStage', },),).toEqual(['none',],);
        const source = await readFile(new URL('refine-slice-settle.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runRefineStage', key: 'judgeRole', },),).toEqual(['select-judge',],);
      },
    },),
  ],
},);

//endregion Repair role tests
