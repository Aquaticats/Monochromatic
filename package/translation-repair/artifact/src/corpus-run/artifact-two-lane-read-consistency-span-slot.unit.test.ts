/**
 Tests for the slot a recorded place of a frame is expected to carry: read with the definitions the place records from
 generation eighteen, over the span alone before, none for a run, and a refusal of definitions that move no slot.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyVoice, } from '@monochromatic-dev/translation-repair-consistency';
import {
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  expectedSlotOf,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 Three ballots that all gave the slot `b` beside the span `a[b]c`.
 */
const VOICES: readonly ConsistencyVoice[] = ['a', 'b', 'c',].map(function ballotFor(seat,): ConsistencyVoice {
  return {
    seat,
    reason: 'It words the first place best.',
    quotes: [{ label: 'OCCURRENCE 1', quote: 'a[b]c', slot: 'b', },],
    choice: 'OCCURRENCE 1',
    keep: [],
  };
},);

/**
 The slot expected for the span `a[b]c` under the given definitions and generation.

 @param definitions - definitions the place records, none where it records none

 @param refrain - whether the group repeats a refrain

 @param generation - generation the section was written under

 @returns Expected slot as a list of zero or one

 @example
 ```ts
 const expected = slotFor({ refrain: true, generation: 18, },);
 ```
 */
function slotFor(
  {
    definitions,
    refrain,
    generation,
  }: {
    readonly definitions?: string;
    readonly refrain: boolean;
    readonly generation: typeof ARTIFACT_SCHEMA_VERSION_V17 | typeof ARTIFACT_SCHEMA_VERSION_V18;
  },
): unknown {
  return expectedSlotOf({
    voices: VOICES,
    label: 'OCCURRENCE 1',
    agreed: 'a[b]c',
    span: 'a[b]c',
    refrain,
    ...((definitions === undefined) ? {} : { definitions, }),
    generation,
    path: 'cat.rendering',
  },);
}

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: expectedSlotOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE SLOT OVER THE SPAN ALONE WHERE THE PLACE RECORDS NO DEFINITIONS, and gives none for a run, which has no slot',
          fn: async () => {
            expect([
              slotFor({ refrain: true, generation: ARTIFACT_SCHEMA_VERSION_V18, },),
              slotFor({ refrain: true, generation: ARTIFACT_SCHEMA_VERSION_V17, },),
              slotFor({ refrain: false, generation: ARTIFACT_SCHEMA_VERSION_V18, },),
            ],).toEqual([
              [{ slot: 'b', slotStart: 2, },],
              [{ slot: 'b', slotStart: 2, },],
              [],
            ],);
          },
        },),
        it({
          name: 'READS NO SLOT WHERE THE DEFINITIONS THE PLACE RECORDS MAKE THE SPAN ONE WORD, and refuses recorded definitions that '
            + 'change nothing about the slot, at the path of the definitions',
          fn: async () => {
            expect(slotFor({ definitions: '[b]: https://x.example/b', refrain: true, generation: ARTIFACT_SCHEMA_VERSION_V18, },),)
              .toEqual([],);
            expect(refusalTextOf({
              read: function movesNothing(): unknown {
                return slotFor({ definitions: '[z]: https://x.example/z', refrain: true, generation: ARTIFACT_SCHEMA_VERSION_V18, },);
              },
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.rendering.definitions: expected the definitions that change the slot the '
                + 'usable ballots give inside those words, and none where they change none.',
            );
          },
        },),
      ],
    },),
  ],
},);
