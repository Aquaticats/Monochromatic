/**
 Tests for reading which decider chose a settled preparation's sections.

 WHAT THESE PIN is the three-way reading, unrecorded against deterministic
 against supplied, and the shapes the reader refuses because this pipeline
 cannot write them. Each refusal mirrors an invariant the section round
 already holds, so a refusal that fired on a shape the producer CAN emit
 would reject valid artifacts, which is the failure these are pointed at.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseSectionPairing, } from '../../dist/final/node/index.mjs';

/**
 Sections a fixture may name, wide enough that no case is bounded by accident.
 */
const ALIGNED_SECTIONS = 4;

/**
 Where every case reports from.
 */
const PATH = 'Whiskers.preparation.sectionPairing';

await describe({
  name: parseSectionPairing.name,
  children: [
    it({
      name:
        'NAMES ABSENCE as unrecorded rather than as the deterministic decider, because a file written '
          + 'before the field cannot say which decider it ran and a rebuild from it is an assumption',
      fn: async () => {
        expect(parseSectionPairing({
          value: undefined,
          alignmentPairCount: ALIGNED_SECTIONS,
          path: PATH,
        },),).toEqual({ kind: 'unrecorded', },);
      },
    },),
    it({
      name:
        'READS the deterministic decider as its own answer, distinct from absence, since that is the '
          + 'ordinary production case and an artifact written after the field says it out loud',
      fn: async () => {
        expect(parseSectionPairing({
          value: { kind: 'deterministic', },
          alignmentPairCount: ALIGNED_SECTIONS,
          path: PATH,
        },),).toEqual({ kind: 'deterministic', },);
      },
    },),
    it({
      name:
        'READS a supplied pairing with its pairs in the order recorded, which is what a rebuild hands '
          + 'back to the aligner in place of its own shape check',
      fn: async () => {
        expect(parseSectionPairing({
          value: {
            kind: 'supplied',
            pairs: [
              {
                source: 0,
                target: 1,
              },
              {
                source: 2,
                target: 2,
              },
            ],
          },
          alignmentPairCount: ALIGNED_SECTIONS,
          path: PATH,
        },),).toEqual({
          kind: 'supplied',
          pairs: [
            {
              source: 0,
              target: 1,
            },
            {
              source: 2,
              target: 2,
            },
          ],
        },);
      },
    },),
    it({
      name:
        'ACCEPTS a supplied pairing that names no pairs, echoing what a direct caller consumed rather '
          + 'than normalising it into the deterministic decider, because the two align differently',
      fn: async () => {
        expect(parseSectionPairing({
          value: {
            kind: 'supplied',
            pairs: [],
          },
          alignmentPairCount: ALIGNED_SECTIONS,
          path: PATH,
        },),).toEqual({
          kind: 'supplied',
          pairs: [],
        },);
      },
    },),
    it({
      name:
        'REFUSES a decider this version does not name, at the kind path, so a later generation cannot be '
          + 'read as one of the two this generation understands',
      fn: async () => {
        /**
         Refusal of an unknown decider.
         */
        const refusalOfUnknownDecider = caught(function unknownDecider() {
          parseSectionPairing({
            value: { kind: 'roster', },
            alignmentPairCount: ALIGNED_SECTIONS,
            path: PATH,
          },);
        },);
        expect(refusalOfUnknownDecider,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfUnknownDecider as Error).message,).toContain(`${PATH}.kind`,);
      },
    },),
    it({
      name:
        'REFUSES pairs beside the deterministic decider and a missing list beside the supplied one, since '
          + 'each is a record no builder writes and the exact-key guard is what keeps the file meaning one thing',
      fn: async () => {
        /**
         Refusal of pairs the deterministic decider cannot carry.
         */
        const refusalOfStrayPairs = caught(function strayPairs() {
          parseSectionPairing({
            value: {
              kind: 'deterministic',
              pairs: [],
            },
            alignmentPairCount: ALIGNED_SECTIONS,
            path: PATH,
          },);
        },);
        expect(refusalOfStrayPairs,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfStrayPairs as Error).message,).toContain('no key here beyond kind',);

        /**
         Refusal of a supplied decider with nothing supplied.
         */
        const refusalOfMissingPairs = caught(function missingPairs() {
          parseSectionPairing({
            value: { kind: 'supplied', },
            alignmentPairCount: ALIGNED_SECTIONS,
            path: PATH,
          },);
        },);
        expect(refusalOfMissingPairs,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfMissingPairs as Error).message,).toContain(`${PATH}.pairs`,);
      },
    },),
    it({
      name:
        'REFUSES a pairing that stands still or runs back on EITHER side, which is stricter than the block '
          + 'pairing check on purpose: a section pairing is one translation section per original, and the '
          + 'agreement step drops any target that fails to advance',
      fn: async () => {
        for (
          const [label, pairs,] of [
            [
              'source stands still',
              [
                {
                  source: 1,
                  target: 0,
                },
                {
                  source: 1,
                  target: 2,
                },
              ],
            ],
            [
              'target stands still',
              [
                {
                  source: 0,
                  target: 1,
                },
                {
                  source: 1,
                  target: 1,
                },
              ],
            ],
            [
              'target runs back',
              [
                {
                  source: 0,
                  target: 2,
                },
                {
                  source: 1,
                  target: 0,
                },
              ],
            ],
          ] as const
        ) {
          /**
           Refusal of the shape this case names.
           */
          const refusalOfShape = caught(function shape() {
            parseSectionPairing({
              value: {
                kind: 'supplied',
                pairs,
              },
              alignmentPairCount: ALIGNED_SECTIONS,
              path: PATH,
            },);
          },);
          expect(refusalOfShape,).toBeInstanceOf(ArtifactParseError,);
          // The label rides inside the checked string, so a failing case
          // names which shape slipped through.
          expect(`${label}: ${(refusalOfShape as Error).message}`,).toContain(`${PATH}.pairs[1]`,);
        }
      },
    },),
    it({
      name:
        'REFUSES more pairs than the preparation aligned, since every supplied pair becomes one aligned '
          + 'section and a longer list describes some other pair of documents',
      fn: async () => {
        /**
         Refusal of a pairing longer than the alignment.
         */
        const refusalOfOverlongPairing = caught(function overlong() {
          parseSectionPairing({
            value: {
              kind: 'supplied',
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
              ],
            },
            alignmentPairCount: 1,
            path: PATH,
          },);
        },);
        expect(refusalOfOverlongPairing,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfOverlongPairing as Error).message,).toContain('at most 1 pair',);
      },
    },),
    it({
      name:
        'REFUSES a pair carrying a key this version does not name, at the pair path, for the reason the '
          + 'block pairing reader gives: an unknown key is a claim this generation cannot interpret',
      fn: async () => {
        /**
         Refusal of a pair with an invented field.
         */
        const refusalOfGrownPair = caught(function grownPair() {
          parseSectionPairing({
            value: {
              kind: 'supplied',
              pairs: [{
                source: 0,
                target: 0,
                confidence: 1,
              },],
            },
            alignmentPairCount: ALIGNED_SECTIONS,
            path: PATH,
          },);
        },);
        expect(refusalOfGrownPair,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfGrownPair as Error).message,).toContain(`${PATH}.pairs[0].confidence`,);
      },
    },),
  ],
},);
