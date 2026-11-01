/**
 Tests for what an entry does with the error that ended it (B1220, B1247): it ships what it has, tells the stop it is
 done, and files the stop on its tally.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readInterimEntries, } from '@monochromatic-dev/translation-repair-glossary';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  createPassStop,
  settleCaughtEntry,
} from '../../dist/final/node/index.mjs';
import {
  CORPUS_SHA,
  dirsOf,
  DIGEST,
  ENTRY_ID,
  pageOf,
  slicedProgress,
  SOURCE_DOC,
  TARGET_DOC,
  TIP,
} from './pass-entry-stop.test-fixture.ts';

/**
 Instant the lines of these cases are printed at.
 */
const PRINTED_AT = new Date('2026-11-01T06:00:00.000Z',);

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the lines are printed at

 @example
 ```ts
 await settleCaughtEntry({ ...input, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

await describe({
  name: settleCaughtEntry.name,
  concurrency: 1,
  children: [
    it({
      name: 'SHIPS THE INTERIM PAGE AND ITS RECORD, THEN FILES THE STOP: the page stands, the record names the class, and '
        + 'the tally of the stop carries the same class and the disposition the pass schedules by',
      fn: async (ctx,) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'pass-entry-caught-', },);
        const dirs = dirsOf({ root: root.path, },);
        /**
         What filing the stop answered.
         */
        const outcome = await settleCaughtEntry({
          error: new RangeError('scripted transport failure',),
          progress: slicedProgress(),
          entry: {
            id: ENTRY_ID,
            sourceText: SOURCE_DOC,
            targetText: TARGET_DOC,
          },
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          tip: TIP,
          pipelineDigest: DIGEST,
          corpusSha: CORPUS_SHA,
          attempt: 2,
          wallClock: fixedWallClock,
          durationMs: 5,
          aborted: false,
          l: capturingLogger({ messages: [], },),
        },);
        expect(outcome,).toEqual({ kind: 'resumable-failure', errorClass: 'RangeError', },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,)?.errorClass,)
          .toBe('RangeError',);
        expect(printed.lines.some(function tally(line,): boolean {
          return line.startsWith(`TALLY ${ENTRY_ID} status=ERROR `,) && line.includes(' class=RangeError ',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'TELLS THE STOP ITS FLIGHT IS SHIPPED, and the record names the stop signal when the stop was requested',
      fn: async (ctx,) => {
        using _printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'pass-entry-caught-', },);
        const dirs = dirsOf({ root: root.path, },);
        const stop = createPassStop();
        stop.request();
        /**
         Whether the shipper told the flight.
         */
        const told: { current: boolean; } = { current: false, };
        await settleCaughtEntry({
          error: stop.signal.reason,
          progress: slicedProgress(),
          entry: {
            id: ENTRY_ID,
            sourceText: SOURCE_DOC,
            targetText: TARGET_DOC,
          },
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          tip: TIP,
          pipelineDigest: DIGEST,
          corpusSha: CORPUS_SHA,
          attempt: 1,
          wallClock: fixedWallClock,
          stop,
          flight: {
            shipped: function shipped(): void {
              told.current = true;
            },
            [Symbol.dispose](): void {},
          },
          durationMs: 5,
          aborted: true,
          l: capturingLogger({ messages: [], },),
        },);
        expect(told.current,).toBe(true,);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,)?.reason,)
          .toBe('stop-signal',);
      },
    },),
  ],
},);
