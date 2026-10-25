/**
 Tests for the trials of the editor fidelity probe: one per reviewed reference and damaged twin a deterministic fix
 exists for, each with the accepted issue and the envelope the pipeline would hand an editor.

 Fixtures are cat-themed invention. No corpus content appears here.

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
  editorFidelityTrials,
  editorSheetContextOf,
  FidelityReferenceError,
} from '../dist/final/node/index.mjs';
import {
  REVIEW_DONOR,
  REVIEW_REFERENCE,
} from './fidelity-reference.test-fixture.ts';
import {
  headedReference,
  inventedReference,
} from './editor-fidelity.test-fixture.ts';

/**
 The invented reference's trials of both families the probe builds.
 */
const TRIALS = editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},);

/**
 The insertion trial.
 */
const INSERTION = nonNullishOrThrow(TRIALS.find(function isInsertion(trial,): boolean {
  return trial.damageKind === 'insertion';
},),);

/**
 The alteration trial.
 */
const ALTERATION = nonNullishOrThrow(TRIALS.find(function isAlteration(trial,): boolean {
  return trial.damageKind === 'alteration';
},),);

await describe({
  name: editorFidelityTrials.name,
  children: [
    it({
      name: 'BUILDS ONE TRIAL for each reference and family a deterministic fix exists for, in the manifest\'s order, '
        + 'and none of the four ballot arrangements the judge probe multiplies each twin by, which an editor never sees',
      fn: async () => {
        expect(EDITOR_DAMAGE_KINDS,).toStrictEqual(['insertion', 'alteration',],);
        expect(TRIALS.map(function idOf(trial,): string {
          return trial.trialId;
        },),).toStrictEqual(['invented-reference/insertion', 'invented-reference/alteration',],);
      },
    },),

    it({
      name: 'FINDS THE INSERTED SENTENCE and quotes it WITH THE SENTENCE EITHER SIDE of it on its line, so deleting '
        + 'the quoted region is not the fix, and the accepted issue quotes exactly that text as a target span of the '
        + 'accuracy/addition category, which the preservation gate needs to license the sentence disappearing',
      fn: async () => {
        /**
         The text the issue quotes.
         */
        const quoted = INSERTION.damagedText.slice(
          INSERTION.region.startOffset,
          INSERTION.region.endOffset,
        );
        expect(quoted.startsWith('The well-lit corner',),).toBe(true,);
        expect(quoted.includes(REVIEW_DONOR,),).toBe(true,);
        expect(quoted.endsWith('remembered the patterns each friend liked.',),).toBe(true,);
        expect(quoted.replace(
          INSERTION.damagedDefect,
          '',
        ),).toBe(REVIEW_REFERENCE.slice(
          INSERTION.region.startOffset,
          INSERTION.region.endOffset - INSERTION.damagedDefect.length,
        ),);
        expect(INSERTION.cleanDefect,).toBe('',);
        expect(INSERTION.damagedDefect.trim(),).toBe(REVIEW_DONOR,);
        expect(INSERTION.issues.length,).toBe(1,);
        const [issue,] = INSERTION.issues;
        expect(issue?.status,).toBe('accepted',);
        expect(issue?.claims.length,).toBe(1,);
        const {claim} = nonNullishOrThrow(issue?.claims[0],);
        expect(claim.category,).toBe('accuracy/addition',);
        expect(claim.spans.length,).toBe(1,);
        expect(claim.spans[0],).toMatchObject({
          side: 'target',
          startOffset: INSERTION.region.startOffset,
          endOffset: INSERTION.region.endOffset,
          quotedText: quoted,
        },);
      },
    },),

    it({
      name: 'REFUSES AN INSERTION ALONE ON ITS LINE, whose quoted region would be the sentence by itself: deleting '
        + 'the region would then be the whole fix, and the trial could not tell an editor from a reply that deletes '
        + 'what it is shown',
      fn: async () => {
        /**
         The reference with the inserted sentence moved onto lines of its own.
         */
        const reference = inventedReference();
        const aloneAt = reference.referenceText.indexOf('She carefully kept',);
        expect(function asksLoneInsertion(): void {
          editorFidelityTrials({
            references: [
              {
                ...reference,
                damages: reference.damages.map(function alone(damage,) {
                  return (damage.damageKind === 'insertion')
                    ? {
                      ...damage,
                      damagedText: `${reference.referenceText.slice(
                        0,
                        aloneAt,
                      ).trimEnd()}\n\n${REVIEW_DONOR}\n\n${reference.referenceText.slice(aloneAt,)}`,
                    }
                    : damage;
                },),
              },
            ],
            damageKinds: ['insertion',],
          },);
        },).toThrow(FidelityReferenceError,);
      },
    },),

    it({
      name: 'FINDS THE WHOLE NUMBER that was altered, not the one digit that differs, and takes the sentence it is in '
        + 'as the region, so an editor may rewrite the sentence and the probe can see what else it touched',
      fn: async () => {
        expect(ALTERATION.cleanDefect,).toBe('2023',);
        expect(ALTERATION.damagedDefect.length,).toBe(4,);
        expect(ALTERATION.damagedDefect,).not.toBe('2023',);
        expect(ALTERATION.damagedText.includes(ALTERATION.damagedDefect,),).toBe(true,);
        expect(ALTERATION.damagedText.slice(
          ALTERATION.region.startOffset,
          ALTERATION.region.endOffset,
        ).startsWith('The grey-and-white cat arrived in',),).toBe(true,);
        const {claim} = nonNullishOrThrow(ALTERATION.issues[0]?.claims[0],);
        expect(claim.category,).toBe('accuracy/mistranslation',);
      },
    },),

    it({
      name: 'CUTS THE ENVELOPE with the pipeline\'s own `deriveEditableEnvelopes`: one envelope over exactly the '
        + 'region, whose base text is the region and which serves the accepted issue',
      fn: async () => {
        for (const trial of TRIALS) {
          expect(trial.envelopes.length,).toBe(1,);
          const envelope = nonNullishOrThrow(trial.envelopes[0],);
          expect(envelope.startOffset,).toBe(trial.region.startOffset,);
          expect(envelope.endOffset,).toBe(trial.region.endOffset,);
          expect(envelope.baseText,).toBe(trial.damagedText.slice(
            trial.region.startOffset,
            trial.region.endOffset,
          ),);
          expect(envelope.issueIds,).toStrictEqual([nonNullishOrThrow(trial.issues[0],).issueId,],);
        }
      },
    },),

    it({
      name: 'GIVES EVERY TRIAL THE CONTEXT OF ITS PAGE, prepared from the whole pages of its reference: the same '
        + 'neighbouring sections, declared names and original for both families of one reference',
      fn: async () => {
        /**
         The headed reference, whose passage has a section either side.
         */
        const reference = headedReference();
        /**
         Its trials of both families.
         */
        const headed = editorFidelityTrials({
          references: [reference,],
          damageKinds: EDITOR_DAMAGE_KINDS,
        },);
        expect(headed.length,).toBe(2,);
        for (const trial of headed)
          expect(trial.context,).toStrictEqual(editorSheetContextOf({ reference, },),);
        expect(nonNullishOrThrow(headed[0],).context.neighbouringSourceText.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'WRITES THE ISSUE\'S SUMMARY WITHOUT CORPUS PROSE: it states the kind of defect and nothing the text said',
      fn: async () => {
        for (const trial of TRIALS) {
          const { summary, } = nonNullishOrThrow(trial.issues[0]?.claims[0],).claim;
          expect(summary.length,).toBeGreaterThan(0,);
          expect(REVIEW_REFERENCE.includes(summary,),).toBe(false,);
          expect(summary.includes(REVIEW_DONOR,),).toBe(false,);
          expect(summary.includes('2023',),).toBe(false,);
        }
      },
    },),

    it({
      name: 'REFUSES A DELETION: its fix is a sentence the editor must write, which no deterministic test can score '
        + 'against the archive\'s own wording, so the request is refused rather than scored as a miss',
      fn: async () => {
        expect(function asksDeletion(): void {
          editorFidelityTrials({
            references: [inventedReference(),],
            damageKinds: ['deletion',],
          },);
        },).toThrow(FidelityReferenceError,);
      },
    },),

    it({
      name: 'REFUSES A FAMILY NO REFERENCE CARRIES rather than running the rest, and an empty or repeated family list',
      fn: async () => {
        const reference = inventedReference();
        expect(function asksMissing(): void {
          editorFidelityTrials({
            references: [
              {
                ...reference,
                damages: reference.damages.filter(function notInsertion(damage,): boolean {
                  return damage.damageKind !== 'insertion';
                },),
              },
            ],
            damageKinds: ['insertion',],
          },);
        },).toThrow(FidelityReferenceError,);
        expect(function asksNothing(): void {
          editorFidelityTrials({
            references: [reference,],
            damageKinds: [],
          },);
        },).toThrow(FidelityReferenceError,);
        expect(function asksTwice(): void {
          editorFidelityTrials({
            references: [reference,],
            damageKinds: ['insertion', 'insertion',],
          },);
        },).toThrow(FidelityReferenceError,);
      },
    },),
  ],
},);
