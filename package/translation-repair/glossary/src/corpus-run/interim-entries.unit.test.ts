/**
 Tests for the interim records a pass leaves for an entry it shipped a page for and kept open, and the readers of them.

 Fixtures are cat-themed invention. No corpus content appears here.

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
  INTERIM_DIR,
  type InterimEntryRecord,
  interimPageDigest,
  readInterimEntries,
  removeInterimEntry,
  writeInterimEntry,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 A record for an entry, with the counts a case chooses.

 @param id - entry the record is for

 @returns Record

 @example
 ```ts
 const record = recordFor({ id: 'Tabby', },);
 ```
 */
function recordFor({ id, }: { readonly id: string; },): InterimEntryRecord {
  return {
    id,
    tip: 'abc',
    pipelineDigest: 'sha256-tree-v1:0',
    corpusSha: 'def',
    timestamp: '2026-10-29T21:00:00.000Z',
    reason: 'dry-stop',
    settledSlices: 2,
    archiveSlices: 1,
    sourceSlices: 0,
    pageSha256: interimPageDigest({ pageText: 'The cat waits.\n', },),
  };
}

await describe({
  name: writeInterimEntry.name,
  children: [
    it({
      name: 'WRITES one record per entry into a directory it creates and reads the records back by id in code-point '
        + 'order; an absent directory reads as no interim pages',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-', },);
        /**
         Directory of records, not yet created.
         */
        const interimDir = join(
          root.path,
          INTERIM_DIR,
        );
        expect([...await readInterimEntries({ interimDir, },),],).toStrictEqual([],);
        await writeInterimEntry({
          interimDir,
          record: recordFor({ id: 'Zeta', },),
        },);
        await writeInterimEntry({
          interimDir,
          record: recordFor({ id: 'Alpha', },),
        },);
        expect([...await readInterimEntries({ interimDir, },),],).toStrictEqual([
          ['Alpha', recordFor({ id: 'Alpha', },),],
          ['Zeta', recordFor({ id: 'Zeta', },),],
        ],);
      },
    },),

    it({
      name: 'DIGESTS a page by its bytes, so the same text gives the same digest and a changed page gives another',
      fn: async () => {
        expect(interimPageDigest({ pageText: 'The cat waits.\n', },),).toBe(
          interimPageDigest({ pageText: 'The cat waits.\n', },),
        );
        expect(interimPageDigest({ pageText: 'The cat waits.\n', },),).not.toBe(
          interimPageDigest({ pageText: 'The cat waits. \n', },),
        );
        expect(interimPageDigest({ pageText: '', },),).toBe(
          'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
        );
      },
    },),

    it({
      name: 'REPLACES the record of an entry written twice, so a later stop of the same entry is the one read',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-', },);
        const interimDir = join(
          root.path,
          INTERIM_DIR,
        );
        await writeInterimEntry({
          interimDir,
          record: recordFor({ id: 'Tabby', },),
        },);
        await writeInterimEntry({
          interimDir,
          record: {
            ...recordFor({ id: 'Tabby', },),
            settledSlices: 5,
          },
        },);
        expect((await readInterimEntries({ interimDir, },)).get('Tabby',)?.settledSlices,).toBe(5,);
      },
    },),

    it({
      name: 'REMOVES a record, and reads the removal of an absent one as done',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-', },);
        const interimDir = join(
          root.path,
          INTERIM_DIR,
        );
        await writeInterimEntry({
          interimDir,
          record: recordFor({ id: 'Tabby', },),
        },);
        await removeInterimEntry({
          interimDir,
          entryId: 'Tabby',
        },);
        await removeInterimEntry({
          interimDir,
          entryId: 'Tabby',
        },);
        await removeInterimEntry({
          interimDir: join(
            root.path,
            'never-made',
          ),
          entryId: 'Tabby',
        },);
        expect([...await readInterimEntries({ interimDir, },),],).toStrictEqual([],);
      },
    },),

    it({
      name: 'LEAVES OUT a file that is not the record its name says, a directory named like one, and a file that is no '
        + 'JSON, without costing the records beside them',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-', },);
        const interimDir = join(
          root.path,
          INTERIM_DIR,
        );
        await writeInterimEntry({
          interimDir,
          record: recordFor({ id: 'Calico', },),
        },);
        await mkdir(
          join(
            interimDir,
            'Dir.json',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            interimDir,
            'Garbled.json',
          ),
          '{not json',
        );
        await writeFile(
          join(
            interimDir,
            'Other.json',
          ),
          `${JSON.stringify(recordFor({ id: 'Somebody', },),)}\n`,
        );
        await writeFile(
          join(
            interimDir,
            'Short.json',
          ),
          '{"id":"Short"}',
        );
        expect([...(await readInterimEntries({ interimDir, },)).keys(),],).toStrictEqual(['Calico',],);
      },
    },),
  ],
},);
