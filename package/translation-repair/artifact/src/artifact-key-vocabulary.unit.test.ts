/**
 Tests for which spelling of the renamed keys each artifact generation used.

 THE POSITIVE CONTROL COMES FIRST here, because every other case in this file
 is a lookup and a lookup table that returned the same row for everything
 would satisfy all of them. The first case pins that the two named tables
 actually disagree, on every field, so the dispatch has something to decide.

 GENERATION 3 IS THE INTERESTING ONE, and it is why a table exists at all
 rather than a boolean. It spells the change-set keys the new way and the
 slice index the old way, because the array rename forced a wire change on
 artifacts whose lane result is passed through whole while the index rename
 did not. A reader holding one flag would read every generation 3 artifact's
 slice index as ABSENT.

 Fixtures are version numbers and key names. There is no passage here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ArtifactKeyVocabulary,
  CHUNK_SPELLED_KEYS,
  keyVocabularyOf,
  KNOWN_ARTIFACT_SCHEMA_VERSIONS,
  SLICE_SPELLED_KEYS,
  UnknownArtifactGenerationError,
} from '../dist/final/node/index.mjs';

/**
 Fields every generation spells, in one place so a fifth cannot be added to
 the type and quietly go unchecked here.
 */
const FIELDS = [
  'changedSliceIndices',
  'withdrawnSliceIndices',
  'sliceCritics',
  'sliceIndex',
] as const;

/**
 Generation that kept these records at the artifact root.
 */
const GENERATION_ONE = 1;

/**
 Generation that moved them into the lanes and kept the older spelling.
 */
const GENERATION_TWO = 2;

/**
 Generation that renamed the change-set arrays and left the index alone.
 */
const GENERATION_THREE = 3;

/**
 Generation that finished key renames.
 */
const GENERATION_FOUR = 4;

/**
 Generation current pass writes, retaining same key spelling.
 */
const GENERATION_FIVE = 5;

/**
 Generation adding auditable final body polish under same key spelling.
 */
const GENERATION_SIX = 6;

/**
 Generation adding contest eligibility under same key spelling.
 */
const GENERATION_SEVEN = 7;

/**
 Generation adding absolute naturalness review under same key spelling.
 */
const GENERATION_EIGHT = 8;

/**
 Generation adding bounded correction digest chain under same key spelling.
 */
const GENERATION_NINE = 9;

/**
 Generation no table covers, one past newest: read off the known list, so a
 new generation does not turn this into one the table spells.
 */
const GENERATION_UNKNOWN = Math.max(...KNOWN_ARTIFACT_SCHEMA_VERSIONS,) + 1;

await describe({
  name: keyVocabularyOf.name,
  children: [
    it({
      name: 'POSITIVE CONTROL: the two named tables disagree on every field, so selecting between them '
        + 'can change what a reader asks the artifact for. A table that returned one row for both '
        + 'generations would pass every other case in this file',
      fn: async () => {
        for (const field of FIELDS) {
          expect(CHUNK_SPELLED_KEYS[field],).not.toBe(SLICE_SPELLED_KEYS[field],);
        }
      },
    },),

    it({
      name: 'spells the current table with its own field names, which is what makes the internal '
        + 'vocabulary readable: a reader meeting `changedSliceIndices` in the code and in the file is '
        + 'looking at one name rather than two that happen to line up',
      fn: async () => {
        for (const field of FIELDS) {
          expect(SLICE_SPELLED_KEYS[field],).toBe(field,);
        }
      },
    },),

    it({
      name: 'spells the older table with the names generations 1 and 2 actually wrote, which is the '
        + 'whole reason it exists: those files are on disk and cannot be re-spelled',
      fn: async () => {
        expect(CHUNK_SPELLED_KEYS,).toStrictEqual({
          changedSliceIndices: 'shippedChunkIndices',
          withdrawnSliceIndices: 'withdrawnChunkIndices',
          sliceCritics: 'chunkCritics',
          sliceIndex: 'chunkIndex',
        },);
      },
    },),

    it({
      name: 'gives generation 3 a MIXTURE of the two, which is the case a boolean cannot express: the '
        + 'change-set arrays carry the current names and the slice index still carries the old one',
      fn: async () => {
        expect(keyVocabularyOf({ version: GENERATION_THREE, },),).toStrictEqual({
          changedSliceIndices: 'changedSliceIndices',
          withdrawnSliceIndices: 'withdrawnSliceIndices',
          sliceCritics: 'sliceCritics',
          sliceIndex: 'chunkIndex',
        },);
      },
    },),

    it({
      name: 'hands generation 3 a row that is neither named table, so a reader cannot satisfy it by '
        + 'picking whichever of the two is closer',
      fn: async () => {
        /**
         Row the mixture generation dispatches to.
         */
        const mixed = keyVocabularyOf({ version: GENERATION_THREE, },);

        expect(mixed,).not.toStrictEqual(CHUNK_SPELLED_KEYS,);
        expect(mixed,).not.toStrictEqual(SLICE_SPELLED_KEYS,);
      },
    },),

    it({
      name: 'gives generations 1 and 2 older spelling and generations 4 through 9 current one, which is '
        + 'the whole dispatch',
      fn: async () => {
        expect(keyVocabularyOf({ version: GENERATION_ONE, },),).toBe(CHUNK_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_TWO, },),).toBe(CHUNK_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_FOUR, },),).toBe(SLICE_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_FIVE, },),).toBe(SLICE_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_SIX, },),).toBe(SLICE_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_SEVEN, },),).toBe(SLICE_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_EIGHT, },),).toBe(SLICE_SPELLED_KEYS,);
        expect(keyVocabularyOf({ version: GENERATION_NINE, },),).toBe(SLICE_SPELLED_KEYS,);
      },
    },),

    it({
      name: 'REFUSES a generation it has no spelling for, rather than falling back to either table. A '
        + 'fallback would read one generation under another generation\'s names and report the renamed '
        + 'keys as ABSENT, which is the one wrong answer here that looks like an ordinary older artifact',
      fn: async () => {
        /**
         What the selector did when asked for a generation past the newest.
         */
        const refusal = caught(function selectsUnknownGeneration() {
          keyVocabularyOf({ version: GENERATION_UNKNOWN, },);
        },);

        expect(refusal,).toBeInstanceOf(UnknownArtifactGenerationError,);
        expect((refusal as Error).message,).toContain(String(GENERATION_UNKNOWN,),);
        expect((refusal as Error).message,).toContain('absent rather than as unread',);
      },
    },),

    it({
      name: 'ACCEPTS either table where the vocabulary type is asked for, so a reader threading one '
        + 'down does not have to know which generation it came from',
      fn: async () => {
        /**
         Both tables under the one type every reader takes.
         */
        const both: readonly ArtifactKeyVocabulary[] = [
          CHUNK_SPELLED_KEYS,
          SLICE_SPELLED_KEYS,
        ];

        expect(both.map(function criticsKeyOf(keys,): string {
          return keys.sliceCritics;
        },),).toStrictEqual([
          'chunkCritics',
          'sliceCritics',
        ],);
      },
    },),
  ],
},);
