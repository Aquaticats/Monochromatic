/**
 Tests for the store that resumes ballots an earlier run bought.

 WHY THE SHAPE IS CHECKED DOWN TO THE BALLOT rather than to the outcome, which
 is the claim these cases exist to prove. The artifact reader refuses a ballot
 it cannot read, so a store that resumed one would let a corrupted cache file
 settle an entry into an artifact no reader will take: the pass would spend a
 whole document and then write a file that fails to parse. Refusing it here
 costs one re-asked slice.

 Fixtures are cat-themed invention written into throwaway directories.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { openLaneContestCache, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Built pipeline the fixtures are filled under.
 */
const TEST_GENERATION = `sha256-tree-v1:${'a'.repeat(64,)}`;

/**
 Key every case writes and reads under.
 */
const CAT_KEY = 'b'.repeat(64,);

/**
 One ballot, carrying every field the loader checks.
 */
const CAT_BALLOT = {
  choice: 'translate',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'the original supports it',
};

/**
 Writes one outcome and reads the directory back through a fresh store.

 @param outcome - value to persist, valid or not

 @returns Whether a second store resumed it

 @example
 ```ts
 const resumed = await roundTrip({ outcome, },);
 ```
 */
async function roundTrip(
  { outcome, }: { readonly outcome: unknown; },
): Promise<boolean> {
  await using scratch = await scratchDir({ prefix: 'whiskers-lane-contest-', },);

  /**
   Store this run persists through.
   */
  const writing = await openLaneContestCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },);
  await writing.persist({
    key: CAT_KEY,
    serialized: JSON.stringify(outcome,),
  },);

  /**
   Store a later run would resume through.
   */
  const reading = await openLaneContestCache({
    dir: scratch.path,
    generation: TEST_GENERATION,
  },);
  return reading.resumed
    .has(CAT_KEY,);
}

await describe({
  name: openLaneContestCache.name,
  children: [
    it({
      name:
        'ROUND-TRIPS a settled contest, which is the half that fails silently: persist and resume '
          + 'disagreeing about a file name costs a re-bought contest per slice per run and errors nowhere',
      fn: async () => {
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [
              CAT_BALLOT,
              CAT_BALLOT,
            ],
            usable: 2,
            findings: [],
          },
        },),).toBe(true,);
      },
    },),
    it({
      name:
        'REFUSES a file whose BALLOT is missing a field, rather than resuming an outcome the artifact '
          + 'reader will refuse after a whole document has been paid for',
      fn: async () => {
        /**
         Ballot without the raw findings the reader requires.
         */
        const { droppedRaw: _dropped, ...partial } = CAT_BALLOT;
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [partial,],
            usable: 1,
            findings: [],
          },
        },),).toBe(false,);
      },
    },),
    it({
      name:
        'REFUSES a file whose usable count disagrees with its own ballots, which every relation '
          + 'downstream reads as the number the quorum was measured against',
      fn: async () => {
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [CAT_BALLOT,],
            usable: 2,
            findings: [],
          },
        },),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a file naming a candidate that is neither lane nor the refusal',
      fn: async () => {
        expect(await roundTrip({
          outcome: {
            choice: 'archive',
            ballots: [
              CAT_BALLOT,
              CAT_BALLOT,
            ],
            usable: 2,
            findings: [],
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'READS an archive verdict on a BALLOT that is absent or names either verdict, and REFUSES a '
        + 'ballot naming something else and a value that is no record at all',
      fn: async () => {
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [CAT_BALLOT,],
            usable: 1,
            findings: [],
          },
        },),).toBe(true,);
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [{ ...CAT_BALLOT, archive: 'publishable', },],
            usable: 1,
            findings: [],
          },
        },),).toBe(true,);
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [{ ...CAT_BALLOT, archive: 'flawed', },],
            usable: 1,
            findings: [],
          },
        },),).toBe(true,);
        expect(await roundTrip({
          outcome: {
            choice: 'translate',
            ballots: [{ ...CAT_BALLOT, archive: 'nonsense', },],
            usable: 1,
            findings: [],
          },
        },),).toBe(false,);
        expect(await roundTrip({ outcome: 5, },),).toBe(false,);
      },
    },),
  ],
},);
