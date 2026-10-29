/**
 Tests for which logs the cap census counts: a log is a pass run when a line
 among its first two thousand opens `START tip=` followed by a commit id a
 repository could give, and only then are its calls read.

 Every log is written into a scratch directory, from invented, cat-themed
 lines.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MODEL_CARDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import { capCensusPassRunReading, } from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  spendLine,
  streamLine,
} from '@monochromatic-dev/translation-repair-provider-stream/ts/corpus-run/cap-census-log.test-fixture.ts';

/**
 The seat's Hyper id, as its lines name it.
 */
const HYPER_ID = MODEL_CARDS[SEAT_HYPER_OPENROUTER_UNMEASURED].hyper?.id ?? 'no hyper block';

/**
 Lines at the head of a log the census searches for the marker.
 */
const HEAD_LINES = 2_000;

/**
 Bytes the census once read at the head of a log, which the cases write more than before the marker.
 */
const FORMER_HEAD_BYTES = 4_096;

/**
 A commit id as a repository gives one: forty hexadecimal digits that are not one digit repeated.
 */
const REAL_TIP = '2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815';

/**
 The START line of a real pass: its commit id, the digest of the pipeline it ran and the figures a pass prints
 (`corpus-pass-lines.ts`).
 */
const REAL_START = `START tip=${REAL_TIP} `
  + 'pipeline=sha256-tree-v1:5d4265e1c0a37b8f9d2e6a4c1b07f3e58d9a2c4b6e1f0a7d3c5b8e92f4a6d1c0 files=156 pending=2 '
  + 'done=0 soft=259200000ms hard=25200000ms hard-scope=per-attempt call=360000ms claude-call=460000ms '
  + 'at=2026-09-28T09:59:00.000Z';

/**
 The START line a test suite prints from a fixture pass, which no real pass can print: its commit id is one digit
 repeated forty times and its digest covers the one file of a scratch directory (ledger B963).
 */
const FIXTURE_START = `START tip=${'f'.repeat(40,)} `
  + 'pipeline=sha256-tree-v1:9ae510cf8a857e770845eabe23f4b2b24d4a2b36a317fbc6cec8d995a7fb834f files=1 pending=1 '
  + 'done=0 soft=259200000ms hard=25200000ms hard-scope=per-attempt call=360000ms claude-call=460000ms '
  + 'at=2026-09-28T09:59:00.000Z';

/**
 Lines of build output a mise task prints before the pass it runs starts, as a real pass log holds them (the most
 seen before START in a real log is 254).
 */
const BUILD_LINES = Array.from(
  { length: 254, },
  function buildLine(_unused, index,): string {
    return `[//package/translation-repair/corpus-pass:build] chunk ${String(index,)} size: 112.36 kB`;
  },
);

/**
 Bytes of one line the census reads before it gives up on a log as no pass run.
 */
const LINE_BYTES = 65_536;

/**
 Bytes the census reads from a log at a time.
 */
const BLOCK_BYTES = 65_536;

/**
 The two lines of one completed call, five content characters delivered.
 */
const CALL_LINES = [
  streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: HYPER_ID, outcome: 'completed', content: 5, },),
  spendLine({
    stamp: '2026-09-28T10:00:00.020Z',
    tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=13`,
  },),
];

/**
 What reading `CALL_LINES` gives.
 */
const CALL_READING = {
  samples: [{
    provider: 'hyper',
    model: HYPER_ID,
    completion: 13,
    at: Date.parse('2026-09-28T10:00:00.020Z',),
    content: 5,
  },],
  unstampedLines: 0,
};

/**
 Writes a log of the given lines into a scratch directory.

 @param directory - scratch directory
 @param lines - log's lines
 @param name - file name, so logs written side by side never share a file

 @returns Path written

 @example
 ```ts
 const path = await writeLog({ directory: scratch.path, lines: [REAL_START,], },);
 ```
 */
async function writeLog(
  {
    directory,
    lines,
    name = 'nap.log',
  }: {
    readonly directory: string;
    readonly lines: readonly string[];
    readonly name?: string;
  },
): Promise<string> {
  /**
   Where the log goes.
   */
  const path = join(
    directory,
    name,
  );
  await writeFile(
    path,
    lines.join('\n',),
    'utf8',
  );
  return path;
}

await describe({
  name: capCensusPassRunReading.name,
  children: [
    it({
      name: 'READS THE CALLS of a log that opens with the pass-run marker',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'READS A LOG WHOSE MARKER IS NOT ITS FIRST LINE, so long as a line within the head opens with it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              'a banner line',
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'READS A LOG WITH 200 PAGE-ABSENT INCOMPLETE LINES BEFORE ITS MARKER, which a pass prints for the entries '
        + 'whose page is absent at the pin before it prints START, and which hold more bytes than a head counted in '
        + 'bytes allows (ledger B963)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        /**
         One line per entry whose page is absent at the pin, as the pass prints them before START.
         */
        const pageAbsent = Array.from(
          { length: 200, },
          function lineOf(_unused, index,): string {
            return `INCOMPLETE tabby${String(index,)
              .padStart(4, '0',)} at=2026-09-28T09:59:00.000Z: target page absent at the pin (missing-object)`;
          },
        );

        expect(pageAbsent.join('\n',).length,).toBeGreaterThan(FORMER_HEAD_BYTES,);
        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              ...pageAbsent,
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'READS A LOG WHOSE LINE BEFORE THE MARKER IS LONGER THAN A FOUR-KILOBYTE HEAD, in three-byte characters, '
        + 'since the head is counted in lines and no byte count cuts a character or a log',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        /**
         A line of three-byte characters, longer than a head of four kilobytes holds.
         */
        const wide = '猫'.repeat(FORMER_HEAD_BYTES,);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              wide,
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'SAYS A LOG WITH NO MARKER LINE IS NO PASS RUN, however many calls it holds',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: CALL_LINES,
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
    it({
      name: 'SAYS A LOG IS NO PASS RUN where the marker sits mid-line rather than opening one',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              `a suite wrote ${REAL_START} here`,
              ...CALL_LINES,
            ],
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
    it({
      name: 'READS A LOG WHOSE MARKER IS THE LAST LINE OF THE HEAD, with one line fewer than the head holds before it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              ...Array.from(
                { length: HEAD_LINES - 1, },
                function bannerLine(): string {
                  return 'a banner line';
                },
              ),
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'SAYS A LOG IS NO PASS RUN where its marker lies past the first two thousand lines, which bounds the '
        + 'search of a log that holds none',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              ...Array.from(
                { length: HEAD_LINES, },
                function bannerLine(): string {
                  return 'a banner line';
                },
              ),
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
    it({
      name: 'READS A REAL PASS LOG BEHIND 254 BUILD LINES, the most a real log held before START, since the build '
        + 'output a mise task prints first is no reason to count the pass out (ledger B963)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              ...BUILD_LINES,
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'SAYS A TEST-SUITE LOG IS NO PASS RUN where its START line is the one a fixture pass prints, however deep '
        + 'in the head it sits, since a suite log opens that line at line 267 or later and a head counted in lines '
        + 'admitted 515 of them as pass runs (ledger B963)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              ...BUILD_LINES,
              '[info] [2026-10-22T20:02:04.809Z] [lane-contest-driver-test] a fixture lane purrs',
              FIXTURE_START,
              ...CALL_LINES,
            ],
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
    it({
      name: 'SAYS A LOG IS NO PASS RUN where the commit id of its START line is no commit id: one digit repeated, '
        + 'a word, uppercase, or not forty digits long',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        /**
         Commit ids no pass reads from a repository, each as a log's START line would carry it.
         */
        const tips = [
          '0'.repeat(40,),
          'a'.repeat(40,),
          'tabby0001',
          '2A336603BF5D1F0A8C7E94B1D62E0A3F77C9D815',
          '2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d81',
          '2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d8150',
          '',
        ];
        /**
         What the census made of the log opened by each of them.
         */
        const readings = await Promise.all(tips.map(async function readWith(tip, index,): Promise<unknown> {
          return await capCensusPassRunReading({
            path: await writeLog({
              directory: scratch.path,
              name: `tip-${String(index,)}.log`,
              lines: [
                REAL_START.replace(
                  REAL_TIP,
                  tip,
                ),
                ...CALL_LINES,
              ],
            },),
          },);
        },),);

        expect(readings,).toStrictEqual(tips.map(function none(): string {
          return 'not-a-pass-run';
        },),);
      },
    },),
    it({
      name: 'READS A LOG WHOSE START LINE HOLDS EXACTLY THE BYTES A LINE MAY, and says a log whose START line holds '
        + 'one more is no pass run, since the census reads at most that many bytes of a line (ledger B963)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        /**
         A START line padded to the given number of bytes.

         @param bytes - bytes the line holds

         @returns Line
         */
        function startLineOf({ bytes, }: { readonly bytes: number; },): string {
          return `${REAL_START} ${'x'.repeat(bytes - Buffer.byteLength(REAL_START,) - 1,)}`;
        }

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            name: 'at-the-bound.log',
            lines: [
              startLineOf({ bytes: LINE_BYTES, },),
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            name: 'past-the-bound.log',
            lines: [
              startLineOf({ bytes: LINE_BYTES + 1, },),
              ...CALL_LINES,
            ],
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
    it({
      name: 'READS A LOG WHOSE START LINE CROSSES THE END OF A BLOCK OF THE FILE, with the marker cut in two, since the '
        + 'census reads the file in blocks and holds a line whole',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              // The line and its break hold five bytes less than a block, so START begins five bytes before its end.
              'a'.repeat(BLOCK_BYTES - 6,),
              REAL_START,
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'SAYS A LOG OF ONE 13-MEGABYTE LINE IS NO PASS RUN even where the line opens with a START line, since the '
        + 'census reads at most a stated number of bytes of a line and does not buffer a log that has no line break '
        + '(ledger B963)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [`${REAL_START} ${'x'.repeat(13_000_000,)}`,],
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
    it({
      name: 'SAYS AN EMPTY LOG IS NO PASS RUN',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [],
          },),
        },),).toBe('not-a-pass-run',);
      },
    },),
  ],
},);
