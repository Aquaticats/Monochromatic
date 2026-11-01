/**
 Guards ledger A9: the DONE line counts every entry a run finished, a decline
 as much as an artifact, and never subtracts declines already on disk.

 Cat-themed invention throughout.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DECLINED_DIR,
  INTERIM_DIR,
  writeDeclinedEntry,
  writeInterimEntry,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  entriesFinishedThisRun,
  finishedEntryIds,
} from '../../dist/final/node/index.mjs';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 A throwaway runs dir, removed when the case ends.

 @returns Artifact and decline directories with the disposer

 @example
 ```ts
 await using runs = await throwawayRuns();
 ```
 */
async function throwawayRuns(): Promise<{
  readonly artifactsDir: string;
  readonly declinedDir: string;
} & AsyncDisposable> {
  // Root nothing outside this case writes into.
  return await scratchDirWith({
    prefix: 'pass-finished-',
    setup: async function seeded({ path: root, },): Promise<{
      readonly artifactsDir: string;
      readonly declinedDir: string;
    }> {
      /**
       Artifact directory, created empty.
       */
      const artifactsDir = join(
        root,
        'artifacts',
      );
      await mkdir(artifactsDir,);
      return {
        artifactsDir,
        declinedDir: join(
          root,
          DECLINED_DIR,
        ),
      };
    },
  },);
}

/**
 Records one cat's decline.

 @param declinedDir - directory of decline records

 @param id - cat whose page was declined

 @example
 ```ts
 await decline({ declinedDir, id: 'Tabby', },);
 ```
 */
async function decline(
  {
    declinedDir,
    id,
  }: {
    readonly declinedDir: string;
    readonly id: string;
  },
): Promise<void> {
  await writeDeclinedEntry({
    declinedDir,
    record: {
      id,
      tip: 'abc',
      pipelineDigest: 'sha256:0',
      corpusSha: 'feedfac',
      timestamp: '2026-09-27T00:00:00.000Z',
      reason: 'archive-original',
      note: 'the cat wrote this page in English herself',
    },
  },);
}

await describe({
  name: entriesFinishedThisRun.name,
  children: [
    it({
      name: 'LEAVES AN ENTRY WITH AN INTERIM RECORD AND NO ARTIFACT OUT OF THE FINISHED, so the next pass attempts it '
        + 'again and replaces the interim page (B1225): only an artifact or a decline record says an entry is done',
      fn: async () => {
        await using runs = await throwawayRuns();
        await writeInterimEntry({
          interimDir: join(
            runs.artifactsDir,
            '..',
            INTERIM_DIR,
          ),
          record: {
            id: 'Tabby',
            tip: 'abc',
            pipelineDigest: 'sha256-tree-v1:0',
            corpusSha: 'def',
            timestamp: '2026-11-01T00:00:00.000Z',
            reason: 'entry-error',
            errorClass: 'SliceSpliceError',
            settledSlices: 0,
            archiveSlices: 2,
            sourceSlices: 0,
            pageSha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
          },
        },);
        expect(await finishedEntryIds(runs,),).toStrictEqual(new Set(),);
      },
    },),
    it({
      name: 'COUNTS a new artifact and a new decline, and never subtracts a decline from before the run',
      fn: async () => {
        await using runs = await throwawayRuns();
        await decline({
          declinedDir: runs.declinedDir,
          id: 'Tabby',
        },);
        /**
         Finished ids when the run started: Tabby's decline alone.
         */
        const before = await finishedEntryIds(runs,);
        expect(before,).toStrictEqual(new Set(['Tabby',],),);
        await writeFile(
          join(
            runs.artifactsDir,
            'Calico.json',
          ),
          '{}\n',
        );
        await decline({
          declinedDir: runs.declinedDir,
          id: 'Siamese',
        },);
        expect(await entriesFinishedThisRun({
          before,
          ...runs,
        },),).toBe(2,);
      },
    },),
    it({
      name: 'COUNTS nothing for a run that finished nothing on a dir holding a decline',
      fn: async () => {
        await using runs = await throwawayRuns();
        await decline({
          declinedDir: runs.declinedDir,
          id: 'Tabby',
        },);
        expect(await entriesFinishedThisRun({
          before: await finishedEntryIds(runs,),
          ...runs,
        },),).toBe(0,);
      },
    },),
  ],
},);
