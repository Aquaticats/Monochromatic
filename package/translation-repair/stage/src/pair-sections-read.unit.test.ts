/**
 Tests for the section-pairing reader: what a model may return and what it may
 not.

 STRICTER THAN THE BLOCK READER ON EXACTLY ONE POINT, and these cases pin it.
 The block reader permits a repeat on either side, because a translation
 splitting or merging paragraphs is a correspondence the slice machinery can
 carry. A `ChunkPair` carries ONE section on each side, so a repeat here has
 nowhere to go and would silently drop whichever section lost the race.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isSectionPairingWire,
  readSectionPairing,
  SectionPairingError,
} from '../dist/final/node/index.mjs';

/**
 Sections on the original side, as a count the reader bounds indices against.
 */
const SOURCE_COUNT = 3;

/**
 Sections on the translation side.
 */
const TARGET_COUNT = 4;

/**
 Reads a pairing against the fixture's counts.

 @param value - what a model returned

 @returns Pairs the reader accepted

 @throws SectionPairingError when it refuses

 @example
 ```ts
 const pairs = read({ pairs: [], },);
 ```
 */
function read(value: unknown,) {
  return readSectionPairing({
    value,
    sourceCount: SOURCE_COUNT,
    targetCount: TARGET_COUNT,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readSectionPairing.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a strictly increasing pairing that leaves sections out on both sides, since an '
            + 'omitted section is the correct answer for one with no counterpart',
          fn: async () => {
            expect(read({ pairs: [
              {
                source: 0,
                target: 1,
              },
              {
                source: 2,
                target: 3,
              },
            ], },).length,).toBe(2,);
          },
        },),

        it({
          name: 'ACCEPTS an empty pairing, which is a roster saying it could match nothing rather than '
            + 'a reply that failed',
          fn: async () => {
            expect(read({ pairs: [], },).length,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES a reply that is not a pairing at all',
          fn: async () => {
            expect(function readsRubbish() {
              read({ correspondences: [], },);
            },).toThrow(SectionPairingError,);
            expect(function readsNull() {
              read(null,);
            },).toThrow(SectionPairingError,);
            expect(function readsNonInteger() {
              read({ pairs: [{
                source: 0.5,
                target: 1,
              },], },);
            },).toThrow(SectionPairingError,);
          },
        },),

        it({
          name: 'REFUSES an original section the document does not have, rather than clamping it, since '
            + 'a model that invented an index did not read the sheet it was sent',
          fn: async () => {
            expect(function namesMissingSource() {
              read({ pairs: [{
                source: SOURCE_COUNT,
                target: 0,
              },], },);
            },).toThrow(SectionPairingError,);
            expect(function namesNegativeSource() {
              read({ pairs: [{
                source: -1,
                target: 0,
              },], },);
            },).toThrow(SectionPairingError,);
          },
        },),

        it({
          name: 'REFUSES a translation section the document does not have',
          fn: async () => {
            expect(function namesMissingTarget() {
              read({ pairs: [{
                source: 0,
                target: TARGET_COUNT,
              },], },);
            },).toThrow(SectionPairingError,);
          },
        },),

        it({
          name: 'REFUSES a pairing that moves backwards on either side, because both documents say '
            + 'things in the same order and a reply that does not was not reading them as documents',
          fn: async () => {
            expect(function goesBackOnSource() {
              read({ pairs: [
                {
                  source: 2,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
              ], },);
            },).toThrow(SectionPairingError,);
            expect(function goesBackOnTarget() {
              read({ pairs: [
                {
                  source: 0,
                  target: 2,
                },
                {
                  source: 1,
                  target: 1,
                },
              ], },);
            },).toThrow(SectionPairingError,);
          },
        },),

        it({
          name: 'REFUSES one original claimed by two translations, which the block reader ALLOWS as a '
            + 'split: a section pair carries one section on each side, so the second claim would be '
            + 'dropped without anything saying so',
          fn: async () => {
            expect(function splitsASection() {
              read({ pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 0,
                  target: 1,
                },
              ], },);
            },).toThrow(SectionPairingError,);
          },
        },),

        it({
          name: 'REFUSES one translation claimed by two originals, which the block reader ALLOWS as a '
            + 'merge, for the same reason',
          fn: async () => {
            expect(function mergesTwoSections() {
              read({ pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 0,
                },
              ], },);
            },).toThrow(SectionPairingError,);
          },
        },),
      ],
    },),

    describe({
      name: isSectionPairingWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS the shape and says nothing about whether the indices exist, which is the '
            + 'reader’s question because only it has the counts',
          fn: async () => {
            expect(isSectionPairingWire({ pairs: [{
              source: 99,
              target: 99,
            },], },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a value carrying no pair list, a list that is not an array, and an entry '
            + 'missing either side',
          fn: async () => {
            expect(isSectionPairingWire({},),).toBe(false,);
            expect(isSectionPairingWire({ pairs: 'none', },),).toBe(false,);
            expect(isSectionPairingWire({ pairs: [{ source: 0, },], },),).toBe(false,);
            expect(isSectionPairingWire({ pairs: [{ target: 0, },], },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
