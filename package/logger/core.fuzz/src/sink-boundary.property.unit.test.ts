/**
 Sink boundary properties under Node. Every message crosses a syntax
 boundary on its way out of a sink: JSON for the file and sessionStorage
 sinks, a terminal for the console sink. Each property feeds adversarial
 records through the built sink and reparses or inspects what came out.

 - The console neutralizer agrees with an independent reference and never
   leaves a forbidden control in its output.
 - The console sink's emitted texts are exactly the reference prediction:
   grouped by level, formatted, neutralized, nothing added or lost.
 - The sessionStorage sink's persisted batches reparse to the exact records.
 - The file sink's appended lines reparse to the exact records.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _neutralizeControlCharacters as neutralizeControlCharacters,
  type LogRecord,
  sinks,
} from '@monochromatic-dev/logger-core';
import { createFileSink, } from '@monochromatic-dev/logger-core/node';
import {
  assert,
  asyncProperty,
} from 'fast-check';

import {
  adversarialMessage,
  logRecords,
} from './adversarial-message.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  enterThrowawayPackage,
  hasForbiddenConsoleUnit,
  readFileSinkLines,
  readSessionStorageLines,
  referenceConsoleRuns,
  referenceNeutralize,
  settleTrackedRuns,
  trackRun,
  yieldToEventLoop,
} from './sink-boundary-harness.ts';

/**
 Run plan resolved once for every property in this file.
 */
const run = fuzzRunPlan();

/**
 Console methods the sink can call for non-debug levels; debug goes to
 process stderr under Node.
 */
const CONSOLE_METHODS = [
  'error',
  'info',
  'trace',
  'warn',
] as const;

/**
 Makes the console sink verbose (so debug and trace records are emitted)
 and un-suppresses warn for the duration of a test; disposing restores
 both environment variables.

 @returns Disposable restoring the previous environment.

 @example
 ```ts
 using verbose = verboseConsole();
 ```
 */
function verboseConsole(): Disposable {
  /**
   Previous values, absent when the variable was unset.
   */
  const previous: {
    verbose?: string;
    warn?: string;
  } = {};
  if (process.env.MONOCHROMATIC_VERBOSE !== undefined)
    previous.verbose = process.env.MONOCHROMATIC_VERBOSE;
  if (process.env.MONOCHROMATIC_WARN !== undefined)
    previous.warn = process.env.MONOCHROMATIC_WARN;
  process.env.MONOCHROMATIC_VERBOSE = 'true';
  delete process.env.MONOCHROMATIC_WARN;
  return {
    [Symbol.dispose](): void {
      if (previous.verbose === undefined)
        delete process.env.MONOCHROMATIC_VERBOSE;
      else
        process.env.MONOCHROMATIC_VERBOSE = previous.verbose;
      if (previous.warn !== undefined)
        process.env.MONOCHROMATIC_WARN = previous.warn;
    },
  };
}

/**
 Writes records through a fresh file sink inside a throwaway package and
 checks the appended JSONL reparses to exactly those records. The sink keeps
 no verify probe record of its own.

 @param records - Records to append.

 @example
 ```ts
 await fileRoundTrip([{ level: 'info', message: 'a', timestamp: 0 }]);
 ```
 */
async function fileRoundTrip(records: readonly LogRecord[],): Promise<void> {
  await yieldToEventLoop();
  await using pkg = await enterThrowawayPackage();
  /**
   Sink under test, resolving its log file under the throwaway package.
   */
  const sink = createFileSink();
  expect(await sink.verify(),)
    .toBe(true,);
  for (const record of records)
    // oxlint-disable-next-line no-await-in-loop -- Sequential appends keep file order deterministic; the property compares order.
    await sink.write(record,);
  /**
   Reparsed lines. The sink keeps no verify probe record, so the whole file
   reparses to exactly the appended records.
   */
  const lines = await readFileSinkLines({ dir: pkg.dir, },);
  expect(lines,).toEqual(records,);
}

await describe({
  name: 'sink boundaries under Node',
  // The console property stubs console methods and the environment, the
  // sessionStorage property clears the process-global store, and the file
  // property changes the working directory; they run one at a time.
  concurrency: 1,
  children: [
    it({
      name: 'the console neutralizer agrees with an independent reference and leaves no forbidden control',
      timeout: run.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            adversarialMessage(),
            async function agrees(message,) {
              await yieldToEventLoop();
              /**
               Output of the neutralizer under test.
               */
              const actual = neutralizeControlCharacters(message,);
              expect(actual,)
                .toBe(referenceNeutralize(message,),);
              expect(hasForbiddenConsoleUnit(actual,),)
                .toBe(false,);
            },
          ),
          run.params,
        );
      },
    },),

    it({
      name: 'the console sink emits exactly the reference texts, grouped by level, with every message neutralized',
      timeout: run.timeout,
      fn: async ({ sinon, },) => {
        using _verbose = verboseConsole();
        /**
         Every text the sink handed to a console method or to process
         stderr, in emission order.
         */
        const texts: string[] = [];
        /**
         Records one emitted text.

         @param text - Text the sink emitted.
         */
        function keep(text: unknown,): void {
          texts.push(String(text,),);
        }
        for (const method of CONSOLE_METHODS)
          sinon.stub(
            console,
            method,
          )
            .callsFake(keep,);
        sinon.stub(
          process.stderr,
          'write',
        )
          .callsFake(function keepStderr(chunk: unknown,): boolean {
            keep(chunk,);
            return true;
          },);
        await assert(
          asyncProperty(
            logRecords(),
            async function emits(records,) {
              await yieldToEventLoop();
              texts.length = 0;
              /**
               Sink under test.
               */
              const sink = sinks.createConsoleSink();
              expect(await sink.verify(),)
                .toBe(true,);
              // One synchronous frame of writes, so contiguous same-level
              // records collapse into one emitted text each.
              await Promise.all(records.map(function writeOne(record,): Promise<void> {
                return sink.write(record,);
              },),);
              if (sink.flush !== undefined)
                await sink.flush();
              expect(texts,)
                .toEqual(referenceConsoleRuns(records,),);
              for (const text of texts)
                expect(hasForbiddenConsoleUnit(text,),)
                  .toBe(false,);
            },
          ),
          run.params,
        );
      },
    },),

    it({
      name: 'the sessionStorage sink persists batches that reparse to the exact records',
      timeout: run.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            logRecords(),
            async function roundTrips(records,) {
              await yieldToEventLoop();
              globalThis.sessionStorage.clear();
              /**
               Sink under test, writing into Node's in-memory sessionStorage.
               */
              const sink = sinks.createSessionStorageSink();
              expect(await sink.verify(),)
                .toBe(true,);
              for (const record of records)
                // oxlint-disable-next-line no-await-in-loop -- Sequential writes keep batch order deterministic; the property compares order.
                await sink.write(record,);
              if (sink.flush !== undefined)
                await sink.flush();
              expect(readSessionStorageLines(),)
                .toEqual(records,);
            },
          ),
          run.params,
        );
      },
    },),

    it({
      name: 'the file sink appends JSONL lines that reparse to the exact records',
      timeout: run.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            logRecords(),
            function roundTrips(records,): Promise<void> {
              return trackRun(fileRoundTrip(records,),);
            },
          ),
          run.params,
        );
        // An interrupted campaign abandons the run in flight; wait for it so
        // the working directory is restored before the process moves on.
        await settleTrackedRuns();
      },
    },),
  ],
},);
