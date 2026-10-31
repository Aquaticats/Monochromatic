/**
 Tests for the role each asking of a bench names in this package (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. The translators are the translator role; the judges of a
 slate stand in the role their caller seats them in, which the select functions take from it.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Translate stage role tests

await describe({
  name: 'the role the translate stage names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE TRANSLATOR ROLE where the translators are asked to write',
      fn: async () => {
        const source = await readFile(new URL('translate-produce.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['translator',],);
      },
    },),

    it({
      name: 'ASKS THE JUDGES OF A SLATE IN THE ROLE THEIR CALLER SEATS THEM IN, not in one fixed here: the translate '
        + 'lane seats the wide judges and the consolidation slate the late judges',
      fn: async () => {
        const select = await readFile(new URL('candidate-select.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source: select, callee: 'gatherStageVoices', },),).toEqual(['role',],);
        const judge = await readFile(new URL('translate-judge.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source: judge, callee: 'selectBestCandidate', },),).toEqual(['role',],);
      },
    },),

    it({
      name: 'ASKS THE AUTHORS ABOUT THEIR FINDINGS IN THE ROLE OF THE TRANSLATORS and shares with the gather that wrote the '
        + 'candidates the one record of the models asked, so a model that took a runaway seat there gives no second voice '
        + 'in the repair',
      fn: async () => {
        const source = await readFile(new URL('translate-produce.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'repairInvalidCandidates', },),).toEqual(['translator',],);
        expect(rolesNamedAtCalls({ source, callee: 'repairInvalidCandidates', key: 'reaskClaims', },),).toEqual([
          'reaskClaims',
        ],);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', key: 'reaskClaims', },),).toEqual(['reaskClaims',],);
        expect(source.split('createReaskClaims(',).length - 1,).toBe(1,);
      },
    },),

    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the slate of the translate lane is judged, at the call that hands the judges',
      fn: async () => {
        const source = await readFile(new URL('translate-stage-repair.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'judgeSlateWithRetry', key: 'judging.role', },),).toEqual(['select-judge',],);
      },
    },),
  ],
},);

//endregion Translate stage role tests
