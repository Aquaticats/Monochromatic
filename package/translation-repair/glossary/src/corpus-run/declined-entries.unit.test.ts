/**
 Tests for the decline records a pass leaves behind and the next pass reads.

 @module
 */

import {
  mkdir,
  readFile,
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
  declinedEntryIds,
  writeDeclinedEntry,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: writeDeclinedEntry.name,
  children: [
    it({
      name: 'writes one JSON record per declined entry into a directory it creates, and reads the ids back '
        + 'sorted; an absent directory reads as no declines',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'declined-', },);
        /**
         Directory of records, not yet created.
         */
        const declinedDir = join(
          root.path,
          DECLINED_DIR,
        );
        expect(await declinedEntryIds({ declinedDir, },),).toStrictEqual(new Set<string>(),);
        await writeDeclinedEntry({
          declinedDir,
          record: {
            id: 'zeta',
            tip: 'abc',
            pipelineDigest: 'sha256:0',
            corpusSha: 'def',
            timestamp: '2026-09-08T21:00:00.000Z',
            reason: 'archive-original',
            note: '这篇文章的原文即英文',
          },
        },);
        await writeDeclinedEntry({
          declinedDir,
          record: {
            id: 'alpha',
            tip: 'abc',
            pipelineDigest: 'sha256:0',
            corpusSha: 'def',
            timestamp: '2026-09-08T21:00:01.000Z',
            reason: 'archive-original',
            note: '请翻译时不要动本篇',
          },
        },);
        expect([ ...await declinedEntryIds({ declinedDir, },), ],).toStrictEqual([
          'alpha',
          'zeta',
        ],);
        /**
         The record as written.
         */
        const written: unknown = JSON.parse(await readFile(
          join(
            declinedDir,
            'zeta.json',
          ),
          'utf8',
        ),);
        expect(written,).toStrictEqual({
          id: 'zeta',
          tip: 'abc',
          pipelineDigest: 'sha256:0',
          corpusSha: 'def',
          timestamp: '2026-09-08T21:00:00.000Z',
          reason: 'archive-original',
          note: '这篇文章的原文即英文',
        },);
      },
    },),
    it({
      name: 'SKIPS a directory named like a record, as the artifact listing does (ledger A9b)',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'declined-', },);
        /**
         Directory of records.
         */
        const declinedDir = join(
          root.path,
          DECLINED_DIR,
        );
        await mkdir(join(
          declinedDir,
          'Tabby.json',
        ), { recursive: true, },);
        await writeDeclinedEntry({
          declinedDir,
          record: {
            id: 'Calico',
            tip: 'abc',
            pipelineDigest: 'sha256:0',
            corpusSha: 'def',
            timestamp: '2026-09-27T00:00:00.000Z',
            reason: 'archive-original',
            note: 'the cat wrote this page in English herself',
          },
        },);
        expect([ ...await declinedEntryIds({ declinedDir, },), ],).toStrictEqual(['Calico',],);
      },
    },),
    it({
      name: 'REFUSES a decline path it cannot list rather than reading it as no declines (ledger A9b)',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'declined-', },);
        /**
         A regular file standing where the directory belongs.
         */
        const declinedDir = join(
          root.path,
          DECLINED_DIR,
        );
        await writeFile(
          declinedDir,
          'a cat napped here\n',
        );
        await expect(declinedEntryIds({ declinedDir, },),)
          .rejects
          .toThrow('ENOTDIR',);
      },
    },),
  ],
},);
