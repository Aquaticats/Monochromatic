/**
 Tests that the pool CENSUSES THE CALLER'S OWN LISTING when it was given one.

 WHY THE LISTING IS PASSED AT ALL. A reader that lists the artifacts directory
 and then lets the census list it again is looking at two views of a directory
 an accumulation may still be writing into, so an entry can be in the reader's
 listing and absent from the census, or the other way round. The caller's
 listing is forwarded precisely so both halves classify the same files.

 WHAT WAS MEASURED. On 2026-08-25, inverting the condition that forwards that
 listing, so the names are dropped exactly when a caller supplied them, failed
 no test in this package. Nothing throws: the census silently widens to
 whatever is on disk, and a rate is then reported over a population its own
 reader never saw.

 READ OFF THE POOL, not off a call count, because the pooled ids are what
 every downstream rate divides by.

 NO NETWORK AND NO REAL ARTIFACTS. Two throwaway files are written to a
 temporary directory, both tagged with the same commit and the same pipeline
 digest so they form ONE generation and no ambiguity refusal can fire. The
 pool is then asked for one of them by name.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { resolvePool, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from './pool-policy.test-fixture.ts';

//region Fixtures

/**
 Commit both fixtures record, shaped as a full object id because anything
 shorter is refused as unplaceable.
 */
const TIP = 'a1b2c3d4e5f60718293a4b5c6d7e8f9012345678';

/**
 Built pipeline both fixtures record, so the two of them are ONE generation
 and the pool has nothing ambiguous to refuse.
 */
const DIGEST = `sha256-tree-v1:${'0123456789abcdef'.repeat(4,)}`;

/**
 Entry the caller asks for.
 */
const ASKED = 'mittens';

/**
 Entry sitting beside it that the caller did not ask for.
 */
const UNASKED = 'whiskers';

/**
 Writes one settled-looking artifact carrying the provenance the pool reads.

 @param artifactsDir - directory to write into

 @param entryId - id the file is named for and claims inside

 @returns Nothing; the file is the effect

 @example
 ```ts
 await placeArtifact({ artifactsDir, entryId: 'mittens', },);
 ```
 */
async function placeArtifact(
  {
    artifactsDir,
    entryId,
  }: {
    readonly artifactsDir: string;
    readonly entryId: string;
  },
): Promise<void> {
  await writeFile(
    `${artifactsDir}/${entryId}.json`,
    JSON.stringify({
      id: entryId,
      tip: TIP,
      pipelineDigest: DIGEST,
    },),
    'utf8',
  );
}

//endregion Fixtures

await describe({
  name: resolvePool.name,
  concurrency: 1,
  children: [
    it({
      name: 'POOLS ONLY THE NAMES IT WAS HANDED, so a reader that listed the directory itself and '
        + 'the census that classifies those files are looking at one view of it rather than two, '
        + 'which is the whole reason the listing travels with the call',
      fn: async () => {
        /**
         Throwaway artifacts directory, so nothing real is read or written.
         */
        await using scratch = await scratchDir({ prefix: 'pool-names-', },);
        const artifactsDir = scratch.path;

        await placeArtifact({ artifactsDir, entryId: ASKED, },);
        await placeArtifact({ artifactsDir, entryId: UNASKED, },);

        /**
         Pool resolved against a listing naming ONE of the two files present.
         */
        const eligible = await resolvePool({
          artifactsDir,
          names: [`${ASKED}.json`,],
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
        },);

        // The unasked entry is on disk and would be pooled by a census that
        // listed the directory for itself. Its absence here is the forwarding.
        expect(eligible.entryIds,).toStrictEqual([ASKED,],);
        expect(eligible.malformedIds,).toStrictEqual([],);
      },
    },),
  ],
},);
