/**
 Tests for the block and section pairing caches, which store one shape and
 share one validator (audit area six, ledger B18): a record round-trips
 through each, a record whose findings are not all text, or a bare list,
 is never resumed, and a record filled by another built pipeline is never
 resumed by this one, which is what keeps a pairing from outliving the code
 that settled it (the pairing cache version does not carry that). Fixtures are cat-themed invention written into throwaway
 directories.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  openPairingCache,
  openSectionPairingCache,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Built pipeline the fixtures are filled under.
 */
const TEST_GENERATION = `sha256-tree-v1:${'b'.repeat(64,)}`;

/**
 Both pairing caches, named for the test output.
 */
const OPENERS = [
  {
    name: 'block',
    open: openPairingCache,
  },
  {
    name: 'section',
    open: openSectionPairingCache,
  },
] as const;

/**
 Keys each cache resumes after one value is persisted under `paws` and the
 cache is opened again.

 @param dir - entry cache directory

 @param serialized - stored value

 @returns Resumed keys, per cache

 @example
 ```ts
 const resumed = await resumedAfter({ dir, serialized: '[]', },);
 ```
 */
async function resumedAfter(
  {
    dir,
    serialized,
  }: {
    readonly dir: string;
    readonly serialized: string;
  },
): Promise<readonly (readonly string[])[]> {
  return await Promise.all(OPENERS.map(async function roundTrip({ name, open, },): Promise<readonly string[]> {
    /**
     Directory of this cache's own case.
     */
    const caseDir = join(
      dir,
      name,
    );
    await (await open({
      dir: caseDir,
      generation: TEST_GENERATION,
    },)).persist({
      key: 'paws',
      serialized,
    },);
    return [...(await open({
      dir: caseDir,
      generation: TEST_GENERATION,
    },)).resumed.keys(),];
  },),);
}

await describe({
  name: 'pairing caches',
  children: [
    it({
      name: 'ROUND-TRIPS a pairing record through the block and the section cache',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-pairing-cache-', },);
        expect(await resumedAfter({
          dir: scratch.path,
          serialized: JSON.stringify({
            pairs: [{
              source: 0,
              target: 1,
            },],
            findings: ['the cat napped',],
          },),
        },),).toEqual([['paws',], ['paws',],],);
      },
    },),
    it({
      name: 'NEVER RESUMES a record whose findings are not all text, which both record types promise, nor one '
        + 'whose pairs are not index pairs, nor a bare list, in either cache (ledger B18: the copies checked only '
        + 'that findings were a list)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-pairing-cache-', },);
        expect(await resumedAfter({
          dir: join(
            scratch.path,
            'numbers',
          ),
          serialized: JSON.stringify({
            pairs: [],
            findings: [7,],
          },),
        },),).toEqual([[], [],],);
        expect(await resumedAfter({
          dir: join(
            scratch.path,
            'pairs',
          ),
          serialized: JSON.stringify({
            pairs: [{
              source: 'the first block',
              target: 1,
            },],
            findings: [],
          },),
        },),).toEqual([[], [],],);
        expect(await resumedAfter({
          dir: join(
            scratch.path,
            'bare',
          ),
          serialized: JSON.stringify([],),
        },),).toEqual([[], [],],);
      },
    },),
    it({
      name: 'NEVER RESUMES A RECORD FILLED BY ANOTHER BUILT PIPELINE, in either cache, and the build that filled it finds it gone '
        + 'once the other has opened the directory: the pairing cache version is not what retires a stored pairing when the '
        + 'code that settled it changes',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-pairing-cache-', },);
        /**
         A build other than the one that fills the fixtures.
         */
        const otherGeneration = `sha256-tree-v1:${'c'.repeat(64,)}`;
        /**
         What each cache resumes for the build that filled it, for another build, and for the first again.
         */
        const resumed = await Promise.all(OPENERS.map(async function openedByAnotherBuild({ name, open, },): Promise<unknown> {
          /**
           Directory of this cache's own case.
           */
          const caseDir = join(
            scratch.path,
            name,
          );
          await (await open({
            dir: caseDir,
            generation: TEST_GENERATION,
          },)).persist({
            key: 'paws',
            serialized: JSON.stringify({
              pairs: [],
              findings: [],
            },),
          },);
          /**
           What the build that fills the directory resumes while it is the only one to have opened it.
           */
          const before = [
            ...(await open({
              dir: caseDir,
              generation: TEST_GENERATION,
            },)).resumed.keys(),
          ];
          /**
           What another build resumes from it.
           */
          const other = [
            ...(await open({
              dir: caseDir,
              generation: otherGeneration,
            },)).resumed.keys(),
          ];
          /**
           What the first build resumes once the other has opened the directory and discarded what it held.
           */
          const after = [
            ...(await open({
              dir: caseDir,
              generation: TEST_GENERATION,
            },)).resumed.keys(),
          ];
          return {
            before,
            other,
            after,
          };
        },),);
        expect(resumed,).toEqual([
          {
            before: ['paws',],
            other: [],
            after: [],
          },
          {
            before: ['paws',],
            other: [],
            after: [],
          },
        ],);
      },
    },),
  ],
},);
