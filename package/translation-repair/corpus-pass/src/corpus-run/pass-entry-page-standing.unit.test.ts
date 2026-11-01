/**
 Tests for how the pass reads the page that stands where a stopped entry is about to ship its interim page (B1242,
 B1243): no page, a page an interim record describes, a page no record describes, and a page or a record that cannot be
 read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { mkdir, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  interimPageDigest,
  writeInterimEntry,
} from '@monochromatic-dev/translation-repair-glossary';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { pageStandingOf, } from '../../dist/final/node/index.mjs';
import {
  CORPUS_SHA,
  dirsOf,
  DIGEST,
  ENTRY_ID,
  standingPage,
  TIP,
} from './pass-entry-stop.test-fixture.ts';

/**
 Page an interim record describes in these cases.
 */
const INTERIM_PAGE = 'The cat naps.\n';

/**
 Writes the interim record of the case's entry for a page.

 @param interimDir - directory of interim records

 @param pageText - page the record's digest is taken over

 @param settledSlices - slices the record says are settled

 @example
 ```ts
 await recordFor({ interimDir, pageText: INTERIM_PAGE, settledSlices: 3, },);
 ```
 */
async function recordFor(
  {
    interimDir,
    pageText,
    settledSlices,
  }: {
    readonly interimDir: string;
    readonly pageText: string;
    readonly settledSlices: number;
  },
): Promise<void> {
  await writeInterimEntry({
    interimDir,
    record: {
      id: ENTRY_ID,
      tip: TIP,
      pipelineDigest: DIGEST,
      corpusSha: CORPUS_SHA,
      timestamp: '2026-11-01T00:00:00.000Z',
      reason: 'entry-error',
      settledSlices,
      archiveSlices: 1,
      sourceSlices: 0,
      pageSha256: interimPageDigest({ pageText, },),
    },
  },);
}

await describe({
  name: pageStandingOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS NO PAGE AS NONE',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-page-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        expect(await pageStandingOf({
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          entryId: ENTRY_ID,
          l: capturingLogger({ messages: [], },),
        },),).toEqual({ kind: 'none', },);
      },
    },),

    it({
      name: 'READS A PAGE THE ENTRY\'S RECORD DESCRIBES AS AN INTERIM PAGE with the record\'s settled slices',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-page-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await standingPage({
          dirs,
          text: INTERIM_PAGE,
        },);
        await recordFor({
          interimDir: dirs.interimDir,
          pageText: INTERIM_PAGE,
          settledSlices: 3,
        },);
        expect(await pageStandingOf({
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          entryId: ENTRY_ID,
          l: capturingLogger({ messages: [], },),
        },),).toEqual({
          kind: 'interim',
          settledSlices: 3,
        },);
      },
    },),

    it({
      name: 'READS A PAGE NO RECORD DESCRIBES AS A SETTLED PAGE, whether the entry has no record or its record names '
        + 'other bytes',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-page-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await standingPage({
          dirs,
          text: 'the settled page\n',
        },);
        /**
         What the page reads as with no record at all.
         */
        const withoutRecord = await pageStandingOf({
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          entryId: ENTRY_ID,
          l: capturingLogger({ messages: [], },),
        },);
        await recordFor({
          interimDir: dirs.interimDir,
          pageText: INTERIM_PAGE,
          settledSlices: 3,
        },);
        /**
         What it reads as beside a record of another page.
         */
        const besideOtherRecord = await pageStandingOf({
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          entryId: ENTRY_ID,
          l: capturingLogger({ messages: [], },),
        },);
        expect([withoutRecord, besideOtherRecord,],).toEqual([{ kind: 'settled', }, { kind: 'settled', },],);
      },
    },),

    it({
      name: 'READS NONE AND SAYS SO WHEN THE PAGE CANNOT BE READ, since a read fault must not keep the entry from '
        + 'shipping: a directory stands where the page file should',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-page-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await mkdir(
          `${dirs.publishDir}/people/${ENTRY_ID}/page.en.md`,
          { recursive: true, },
        );
        /**
         Log lines of the read.
         */
        const messages: string[] = [];
        expect(await pageStandingOf({
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          entryId: ENTRY_ID,
          l: capturingLogger({ messages, },),
        },),).toEqual({ kind: 'none', },);
        expect(messages.some(function namesTheFault(message,): boolean {
          return message.includes('could not be read',) && message.includes('class=',);
        },),).toBe(true,);
      },
    },),
  ],
},);
