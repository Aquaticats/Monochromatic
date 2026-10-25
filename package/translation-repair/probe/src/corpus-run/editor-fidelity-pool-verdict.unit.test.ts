/**
 Tests for the scoreability check a drawn trial must pass before it joins the pool: the instrument must read the damaged
 text as not fixed, a scripted ideal editor must fix the defect cleanly through the real stage
 path, and no cheap reply may read as a fix. Fixtures are cat-themed invention.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  EDITOR_DAMAGE_KINDS,
  type EditorFidelityTrial,
  editorFidelityTrials,
  scoreableVerdict,
} from '../../dist/final/node/index.mjs';
import { inventedReference, } from '../editor-fidelity.test-fixture.ts';

/**
 The invented reference's trials.
 */
const TRIALS = editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},);

/**
 The insertion trial.
 */
const INSERTION: EditorFidelityTrial = nonNullishOrThrow(TRIALS.find(function isInsertion(trial,): boolean {
  return trial.damageKind === 'insertion';
},),);

/**
 The alteration trial.
 */
const ALTERATION: EditorFidelityTrial = nonNullishOrThrow(TRIALS.find(function isAlteration(trial,): boolean {
  return trial.damageKind === 'alteration';
},),);

await describe({
  name: scoreableVerdict.name,
  children: [
    it({
      name: 'PASSES A TRIAL THE INSTRUMENT READS AS IT SHOULD, with the empty verdict, for both kinds of defect',
      fn: async () => {
        expect(await scoreableVerdict({ trial: INSERTION, },),).toBe('',);
        expect(await scoreableVerdict({ trial: ALTERATION, },),).toBe('',);
      },
    },),

    it({
      name: 'FAILS A TRIAL WHOSE QUOTED REGION IS THE INSERTED SENTENCE ALONE, because deleting that region would read '
        + 'as a fix: the cheap reply the widened region exists to refuse',
      fn: async () => {
        /**
         The insertion trial with its region cut back to the inserted sentence.
         */
        const narrow: EditorFidelityTrial = {
          ...INSERTION,
          region: {
            startOffset: INSERTION.defectStart,
            endOffset: INSERTION.defectStart + INSERTION.damagedDefect.trim().length,
          },
        };
        expect(await scoreableVerdict({ trial: narrow, },),).toBe('deleting the quoted region reads as a fix',);
      },
    },),

    it({
      name: 'FAILS A TRIAL WHOSE DAMAGED TEXT ALREADY READS AS FIXED, since doing nothing would then pass',
      fn: async () => {
        expect(await scoreableVerdict({
          trial: {
            ...ALTERATION,
            damagedText: ALTERATION.cleanText,
          },
        },),).toBe('damaged text scores fixed',);
      },
    },),
  ],
},);
