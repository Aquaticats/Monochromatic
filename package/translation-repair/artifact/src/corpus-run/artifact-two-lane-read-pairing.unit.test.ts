/**
 Tests for reading the pairing a settled preparation was built on.

 WHAT THESE PIN is the difference between an artifact that records no pairing
 and one that records an empty one, and the four shapes the reader refuses
 because this pipeline cannot write them. Each refusal mirrors an invariant
 `readBlockPairing` already holds over a roster reply, so a refusal that fired
 on a shape the producer CAN emit would reject valid artifacts, which is the
 failure these are pointed at rather than laxness.

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
import { parseBlockPairing, } from '../../dist/final/node/index.mjs';

/**
 Sections a fixture may name, wide enough that no case is bounded by accident.
 */
const ALIGNED_SECTIONS = 4;

await describe({
  name: parseBlockPairing.name,
  children: [
    it({
      name:
        'NAMES ABSENCE rather than returning an empty list, because a file written before the field and '
          + 'a roster asked about every section that agreed nothing are different facts about a run',
      fn: async () => {
        expect(parseBlockPairing({
          value: undefined,
          alignmentPairCount: ALIGNED_SECTIONS,
          path: 'Whiskers.preparation.blockPairing',
        },),).toEqual({ kind: 'unrecorded', },);
      },
    },),
    it({
      name:
        'ACCEPTS a recorded pairing that names no sections, which is the answer when the roster was asked '
          + 'and committed to nothing anywhere, and is NOT the same reading as an absent field',
      fn: async () => {
        expect(parseBlockPairing({
          value: [],
          alignmentPairCount: ALIGNED_SECTIONS,
          path: 'Whiskers.preparation.blockPairing',
        },),).toEqual({
          kind: 'stored',
          sections: [],
        },);
      },
    },),
    it({
      name:
        'ACCEPTS a split and a merge, which are the shapes this corpus actually carries: one original '
          + 'rendered as two blocks repeats the source, and two originals rendered as one repeat the target',
      fn: async () => {
        expect(parseBlockPairing({
          value: [
            {
              sectionIndex: 0,
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 0,
                  target: 1,
                },
              ],
            },
            {
              sectionIndex: 2,
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 0,
                },
              ],
            },
          ],
          alignmentPairCount: ALIGNED_SECTIONS,
          path: 'Whiskers.preparation.blockPairing',
        },).kind,).toBe('stored',);
      },
    },),
    it({
      name:
        'REFUSES a section index the preparation beside it never aligned, the way each lane index set is '
          + 'bounded by the slice count: a pairing filed under a section nobody aligned describes another document',
      fn: async () => {
        /**
         What pastTheEnd raised, read for its class as well as its wording.
         */
        const refusalOfPastTheEnd = caught(function pastTheEnd() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: ALIGNED_SECTIONS,
                pairs: [],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfPastTheEnd,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfPastTheEnd as Error).message,).toContain('[0].sectionIndex',);
      },
    },),
    it({
      name:
        'REFUSES a section named twice, since the writer builds the list from a map keyed by section and '
          + 'cannot emit one, so a repeat means two runs merged into a file neither wrote',
      fn: async () => {
        /**
         What twice raised, read for its class as well as its wording.
         */
        const refusalOfTwice = caught(function twice() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 1,
                pairs: [],
              },
              {
                sectionIndex: 1,
                pairs: [],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfTwice,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfTwice as Error).message,).toContain('rather than 1 again',);
      },
    },),
    it({
      name:
        'REFUSES sections out of recorded order rather than sorting them, because the writer sorts and '
          + 'quietly reordering would accept the file while hiding which run produced it',
      fn: async () => {
        /**
         What backwards raised, read for its class as well as its wording.
         */
        const refusalOfBackwards = caught(function backwards() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 2,
                pairs: [],
              },
              {
                sectionIndex: 1,
                pairs: [],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfBackwards,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfBackwards as Error).message,).toContain('a section above 2',);
      },
    },),
    it({
      name:
        'REFUSES a pairing that walks backwards on either side, which the roster reply reader already '
          + 'refuses: both documents say things in the same order',
      fn: async () => {
        /**
         What backOnTheOriginal raised, read for its class as well as its wording.
         */
        const refusalOfBackOnTheOriginal = caught(function backOnTheOriginal() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 0,
                pairs: [
                  {
                    source: 2,
                    target: 0,
                  },
                  {
                    source: 1,
                    target: 1,
                  },
                ],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfBackOnTheOriginal,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfBackOnTheOriginal as Error).message,).toContain('on both sides',);
        /**
         What backOnTheTranslation raised, read for its class as well as its wording.
         */
        const refusalOfBackOnTheTranslation = caught(function backOnTheTranslation() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 0,
                pairs: [
                  {
                    source: 0,
                    target: 2,
                  },
                  {
                    source: 1,
                    target: 1,
                  },
                ],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfBackOnTheTranslation,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfBackOnTheTranslation as Error).message,).toContain('on both sides',);
      },
    },),
    it({
      name:
        'REFUSES a correspondence made twice in one section, which repeats an answer rather than giving '
          + 'a new one, and which the roster reply reader refuses for the same reason',
      fn: async () => {
        /**
         What repeated raised, read for its class as well as its wording.
         */
        const refusalOfRepeated = caught(function repeated() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 0,
                pairs: [
                  {
                    source: 1,
                    target: 1,
                  },
                  {
                    source: 1,
                    target: 1,
                  },
                ],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfRepeated,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfRepeated as Error).message,).toContain('does not already make',);
      },
    },),
    it({
      name:
        'REFUSES a key this version does not name, on the section and on the pair alike, so a field from '
          + 'a later generation cannot be read as one this reader understands',
      fn: async () => {
        /**
         What extraOnSection raised, read for its class as well as its wording.
         */
        const refusalOfExtraOnSection = caught(function extraOnSection() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 0,
                pairs: [],
                whiskers: 3,
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfExtraOnSection,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfExtraOnSection as Error).message,).toContain('[0].whiskers',);
        /**
         What extraOnPair raised, read for its class as well as its wording.
         */
        const refusalOfExtraOnPair = caught(function extraOnPair() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 0,
                pairs: [
                  {
                    source: 0,
                    target: 0,
                    confidence: 1,
                  },
                ],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfExtraOnPair,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfExtraOnPair as Error).message,).toContain('[0].pairs[0].confidence',);
      },
    },),
    it({
      name:
        'REFUSES a fractional or negative index on either side, since every index here counts blocks and '
          + 'a fraction means the writer and this reader disagree about what the field holds',
      fn: async () => {
        /**
         What fractional raised, read for its class as well as its wording.
         */
        const refusalOfFractional = caught(function fractional() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: 0,
                pairs: [
                  {
                    source: 1.5,
                    target: 0,
                  },
                ],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfFractional,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfFractional as Error).message,).toContain('[0].pairs[0].source',);
        /**
         What negative raised, read for its class as well as its wording.
         */
        const refusalOfNegative = caught(function negative() {
          parseBlockPairing({
            value: [
              {
                sectionIndex: -1,
                pairs: [],
              },
            ],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfNegative,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNegative as Error).message,).toContain('[0].sectionIndex',);
      },
    },),
    it({
      name:
        'REFUSES anything that is not a list of records, so a pairing written as an object keyed by '
          + 'section reaches no reader as an empty one',
      fn: async () => {
        /**
         What notAList raised, read for its class as well as its wording.
         */
        const refusalOfNotAList = caught(function notAList() {
          parseBlockPairing({
            value: { 0: [], },
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfNotAList,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNotAList as Error).message,).toContain('Whiskers.preparation.blockPairing',);
        /**
         What notARecord raised, read for its class as well as its wording.
         */
        const refusalOfNotARecord = caught(function notARecord() {
          parseBlockPairing({
            value: ['paired',],
            alignmentPairCount: ALIGNED_SECTIONS,
            path: 'Whiskers.preparation.blockPairing',
          },);
        },);

        expect(refusalOfNotARecord,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNotARecord as Error).message,).toContain('[0]',);
      },
    },),
  ],
},);
