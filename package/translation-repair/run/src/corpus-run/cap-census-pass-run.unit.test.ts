/**
 Tests for which logs the cap census counts: a log is a pass run when a line
 among its first two thousand opens `START tip=`, and only then are its calls
 read.

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
} from '@monochromatic-dev/module-test/ts';

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
 @param lines - the log's lines

 @returns Path written

 @example
 ```ts
 const path = await writeLog({ directory: scratch.path, lines: ['START tip=tabby',], },);
 ```
 */
async function writeLog(
  {
    directory,
    lines,
  }: {
    readonly directory: string;
    readonly lines: readonly string[];
  },
): Promise<string> {
  /**
   Where the log goes.
   */
  const path = join(
    directory,
    'nap.log',
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
              'START tip=tabby0001',
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
              'START tip=tabby0001',
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
              'START tip=tabby0001',
              ...CALL_LINES,
            ],
          },),
        },),).toEqual(CALL_READING,);
      },
    },),
    it({
      name: 'READS A LOG WHOSE LINE BEFORE THE MARKER IS LONGER THAN THE FORMER BYTE HEAD, in three-byte characters, '
        + 'since the head is counted in lines and no byte count cuts a character or a log',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-pass-run-', },);

        /**
         A line of three-byte characters, longer than the former head held.
         */
        const wide = '猫'.repeat(FORMER_HEAD_BYTES,);

        expect(await capCensusPassRunReading({
          path: await writeLog({
            directory: scratch.path,
            lines: [
              wide,
              'START tip=tabby0001',
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
              'a suite wrote START tip=tabby0001 here',
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
              'START tip=tabby0001',
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
              'START tip=tabby0001',
              ...CALL_LINES,
            ],
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
