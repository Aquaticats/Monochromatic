/**
 Tests for declining an entry whose archive note says the page is the
 author's own English: the archive ships, so no page may stand for it (ledger
 A16c, following from the owner's rule that a run always ships and the
 archive-note rule of 2026-09-08).

 A page can stand there only when a crash lost its artifact between the page
 write and the artifact write, and a later pass then declined the entry.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup';
import {
  assertPipelineDigest,
  type PipelineDigest,
} from '@monochromatic-dev/translation-repair-glossary';
import { inEntryLogContext, } from '@monochromatic-dev/translation-repair-typography';
import {
  entryArchiveOriginalOf,
  recordEntryDecline,
  removeDeclinedPages,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { entryLinesWritten, } from '@monochromatic-dev/translation-repair-test-fixture/ts/entry-log-lines.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Hex digits in a tree digest.
 */
const DIGEST_HEX_LENGTH = 64;

/**
 Digest the declining build claims, before it is checked for shape.
 */
const DIGEST_TEXT = `sha256-tree-v1:${'d'.repeat(DIGEST_HEX_LENGTH,)}`;
assertPipelineDigest(DIGEST_TEXT,);

/**
 Digest the declining build claims.
 */
const DIGEST: PipelineDigest = DIGEST_TEXT;

/**
 Corpus commit every case's pass read the entry at, invented.
 */
const DECLINE_COMMIT = 'feedface'.repeat(5,);

/**
 The entry every case declines.
 */
const ENTRY = {
  id: 'SelfWrittenCat',
  sourceText: '猫猫自己写的。\n',
  targetText: 'The cat wrote this herself.\n',
};

/**
 A throwaway runs directory's decline and page roots.

 @returns Both roots and where the entry's page would stand, removed when its
 `await using` scope ends

 @example
 ```ts
 await using run = await runsDirectory();
 ```
 */
async function runsDirectory(): Promise<{
  readonly declinedDir: string;
  readonly publishDir: string;
  readonly pageDir: string;
} & AsyncDisposable> {
  // Runs directory this case owns.
  return await scratchDirWith({
    prefix: 'pass-decline-',
    setup: async function named({ path: runsDir, },): Promise<{
      readonly declinedDir: string;
      readonly publishDir: string;
      readonly pageDir: string;
    }> {
      /**
       Root of the mirrored tree.
       */
      const publishDir = join(
        runsDir,
        'fixed',
      );
      return {
        declinedDir: join(
          runsDir,
          'declined',
        ),
        publishDir,
        pageDir: join(
          publishDir,
          'people',
          ENTRY.id,
        ),
      };
    },
  },);
}

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Attempt the decline is printed for.
 */
const THIRD_ATTEMPT = 3;

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 await recordEntryDecline({ ..., attempt: THIRD_ATTEMPT, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

/**
 Declines the entry in one runs directory.

 @param run - directories to write

 @example
 ```ts
 await decline({ run, },);
 ```
 */
async function decline(
  { run, }: { readonly run: Awaited<ReturnType<typeof runsDirectory>>; },
): Promise<void> {
  await recordEntryDecline({
    entry: ENTRY,
    declinedDir: run.declinedDir,
    publishDir: run.publishDir,
    tip: 'a'.repeat(DIGEST_HEX_LENGTH,),
    pipelineDigest: DIGEST,
    corpusSha: DECLINE_COMMIT,
    note: '原文即英文',
    startedAt: monotonicMs(),
    attempt: THIRD_ATTEMPT,
    wallClock: fixedWallClock,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: entryArchiveOriginalOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A WHOLE-PAGE NOTE as the page being the author\'s own English, and logs the note with its reading',
          fn: async () => {
            /**
             Lines the reading logged.
             */
            const messages: string[] = [];

            expect(entryArchiveOriginalOf({
              entry: {
                ...ENTRY,
                targetText: '<!-- 这只猫的原文即英文 -->\n\nThe cat wrote this herself.\n',
              },
              l: capturingLogger({ messages, },),
            },),).toEqual({
              kind: 'whole-page',
              note: '这只猫的原文即英文',
            },);
            expect(messages,).toEqual([
              `[${entryArchiveOriginalOf.name}] ARCHIVE NOTE entry=${ENTRY.id} reading=whole-page: 这只猫的原文即英文`,
            ],);
          },
        },),
        it({
          name: 'SEALS NOTHING for a note that speaks of an English original in a wording no mark reads, and WARNS so '
            + 'a new wording is seen rather than silently unsealed (ledger E12)',
          fn: async () => {
            /**
             Lines the reading logged.
             */
            const messages: string[] = [];

            expect(entryArchiveOriginalOf({
              entry: {
                ...ENTRY,
                targetText: '<!-- The cat wrote the original in English, mostly. -->\n\nThe cat wrote this herself.\n',
              },
              l: capturingLogger({ messages, },),
            },),).toEqual({ kind: 'none', },);
            expect(messages,).toEqual([
              `[${entryArchiveOriginalOf.name}] ARCHIVE NOTE entry=${ENTRY.id} reading=unmarked-original-claim: the note `
                + 'speaks of an English original in a wording no mark reads, so nothing is sealed; add its mark if it '
                + 'seals (The cat wrote the original in English, mostly.)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: recordEntryDecline.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS the decline where no page stands, the control the removal rests on',
          fn: async () => {
            await using run = await runsDirectory();

            await decline({ run, },);

            expect(await readdir(run.declinedDir,),).toStrictEqual([`${ENTRY.id}.json`,],);
          },
        },),
        it({
          name: 'PRINTS the DECLINED tally with the clock time it was printed at and the attempt it belongs to, '
            + 'straight after the status, so a reader windowing the log by time can place the decline',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using run = await runsDirectory();

            await decline({ run, },);

            expect(printed.lines.length,).toBe(1,);

            /**
             Text the line opens with, up to the duration.
             */
            const opening = `TALLY ${ENTRY.id} status=DECLINED at=2026-10-22T08:10:11.123Z attempt=3 `
              + 'reason=archive-original ms=';
            expect(printed.lines[0]?.startsWith(opening,),).toBe(true,);

            /**
             Text after the opening, which is the duration in milliseconds.
             */
            const duration = Number(printed.lines[0]?.slice(opening.length,),);
            expect(Number.isInteger(duration,),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES THE CORPUS COMMIT IT IS HANDED on the record, never one of its own',
          fn: async () => {
            await using run = await runsDirectory();

            await decline({ run, },);

            /**
             The record as written.
             */
            const recordText = await readFile(
              join(
                run.declinedDir,
                `${ENTRY.id}.json`,
              ),
              'utf8',
            );
            expect(JSON.parse(recordText,),).toHaveProperty(
              'corpusSha',
              DECLINE_COMMIT,
            );
          },
        },),
        it({
          name: 'REMOVES a page an earlier crash left for the entry, so the archive ships as its note says',
          fn: async () => {
            await using run = await runsDirectory();
            await mkdir(
              run.pageDir,
              { recursive: true, },
            );
            await writeFile(
              join(
                run.pageDir,
                'page.en.md',
              ),
              'A page the run settled before a crash lost its artifact.\n',
            );

            await decline({ run, },);

            expect(await readdir(run.pageDir,),).toStrictEqual([],);
            expect(await readdir(run.declinedDir,),).toStrictEqual([`${ENTRY.id}.json`,],);
          },
        },),
        it({
          name: 'RETHROWS a removal failure other than a missing page and records nothing, so a page it could not '
            + 'remove never stands behind a recorded decline',
          fn: async () => {
            await using run = await runsDirectory();
            // A FILE WHERE THE ENTRY'S PAGE DIRECTORY BELONGS: removing the page
            // under it fails with ENOTDIR, a failure that is not the page's absence.
            await mkdir(
              dirname(run.pageDir,),
              { recursive: true, },
            );
            await writeFile(
              run.pageDir,
              'A file where the cat\'s page directory belongs.\n',
            );

            await expect(decline({ run, },),).rejects.toHaveProperty(
              'code',
              'ENOTDIR',
            );
            /**
             What the runs directory holds after the refusal: the mirrored tree,
             and no decline record's directory.
             */
            const written = await readdir(dirname(run.declinedDir,),);
            expect(written,).toStrictEqual(['fixed',],);
          },
        },),
      ],
    },),

    describe({
      name: `${recordEntryDecline.name} inside an attempt (ledger B963)`,
      // SEQUENTIAL: the case diverts the one global `console.info`, where the tagged logger's `info` sink resolves.
      concurrency: 1,
      children: [
        it({
          name: 'NAMES THE ATTEMPT on the ARCHIVE ORIGINAL line it logs for the entry, since the line is the entry '
            + 'logger\'s and a reader windowing the log by attempt would otherwise place it nowhere',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using run = await runsDirectory();

            /**
             Lines the entry logger wrote while the decline was recorded inside the attempt.
             */
            const entryLines = await entryLinesWritten({
              sinon: ctx.sinon,
              entryId: ENTRY.id,
              run: async () => {
                await inEntryLogContext({
                  entry: ENTRY.id,
                  generation: DIGEST_TEXT,
                  attempt: THIRD_ATTEMPT,
                  run: async () => {
                    await decline({ run, },);
                  },
                },);
              },
            },);

            expect(printed.lines.length,).toBe(1,);
            /**
             The lines among them that are the archive line.
             */
            const archiveLines = entryLines.filter(function isArchiveLine(line,): boolean {
              return line.includes('ARCHIVE ORIGINAL',);
            },);
            expect(archiveLines.length,).toBe(1,);
            expect(archiveLines[0],).toContain(`[${ENTRY.id}] [attempt 3] ARCHIVE ORIGINAL entry=${ENTRY.id}:`,);
          },
        },),
      ],
    },),

    describe({
      name: removeDeclinedPages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REMOVES the page standing for an entry declined earlier, since no later pass visits it',
          fn: async () => {
            await using run = await runsDirectory();
            await decline({ run, },);
            await mkdir(
              run.pageDir,
              { recursive: true, },
            );
            await writeFile(
              join(
                run.pageDir,
                'page.en.md',
              ),
              'A page standing beside a decline record.\n',
            );

            expect(await removeDeclinedPages({
              declinedDir: run.declinedDir,
              publishDir: run.publishDir,
            },),).toStrictEqual([ENTRY.id,],);
            expect(await readdir(run.pageDir,),).toStrictEqual([],);
          },
        },),
        it({
          name: 'REMOVES NOTHING where no page stands, the control the removal rests on',
          fn: async () => {
            await using run = await runsDirectory();
            await decline({ run, },);

            expect(await removeDeclinedPages({
              declinedDir: run.declinedDir,
              publishDir: run.publishDir,
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
