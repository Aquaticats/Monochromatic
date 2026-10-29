/**
 Tests for the context a line carries: which entry, lane and slice it belongs
 to (ledger A11).

 Cat-themed invention throughout.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contextRoot,
  currentLogContext,
  inEntryLogContext,
  inSliceLogContext,
  sliceTagged,
} from '../dist/final/node/index.mjs';

/**
 Collects what `console.info`, where the tagged logger's `info` sink resolves,
 receives while one piece of work runs.

 @param run - work that logs

 @returns Lines printed

 @example
 ```ts
 const lines = await infoLinesOf({ run: async () => { l.info('purr',); }, },);
 ```
 */
async function infoLinesOf(
  { run, }: { readonly run: () => Promise<void>; },
): Promise<readonly string[]> {
  /**
   Lines printed.
   */
  const lines: string[] = [];
  /**
   `console.info` as it was, put back once the work returns.
   */
  const informed = console.info;
  console.info = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  await using restore = {
    [Symbol.asyncDispose]: async () => {
      console.info = informed;
    },
  };
  await run();
  return lines;
}

/**
 Writes one line through a lane logger and one through a stage logger inside attempt 2, slice 3 of the repair lane of
 an entry, and collects what `console.info` received.

 @param laneLogger - logger that writes `kneads`

 @param stageLogger - logger that writes `stretches`

 @returns Printed lines one by one, since the logger hands a run of lines of one level to the console as one call

 @example
 ```ts
 const printed = await printedInOneSlice({ laneLogger, stageLogger, },);
 ```
 */
async function printedInOneSlice(
  {
    laneLogger,
    stageLogger,
  }: {
    readonly laneLogger: Pick<Logger, 'info'>;
    readonly stageLogger: Pick<Logger, 'info'>;
  },
): Promise<readonly string[]> {
  /**
   Lines printed inside one slice.
   */
  const lines = await infoLinesOf({
    run: async () => {
      await inEntryLogContext({
        entry: 'Tabby',
        generation: 'nap-3',
        attempt: 2,
        run: async () => {
          await inSliceLogContext({
            lane: 'repair',
            sliceIndex: 3,
            run: async () => {
              laneLogger.info('kneads',);
              stageLogger.info('stretches',);
            },
          },);
        },
      },);
    },
  },);
  return lines.flatMap(function split(text,): readonly string[] {
    return text.split('\n',);
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: inSliceLogContext.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS empty outside any entry, the entry and its attempt inside one, and entry, attempt, lane and slice '
            + 'inside a slice (ledger B960)',
          fn: async () => {
            expect(currentLogContext(),).toStrictEqual({
              entry: '',
              generation: '',
              attempt: '',
              lane: '',
              slice: '',
            },);
            await inEntryLogContext({
              entry: 'Tabby',
              generation: 'nap-3',
              attempt: 2,
              run: async () => {
                expect(currentLogContext().entry,).toBe('Tabby',);
                expect(currentLogContext().attempt,).toBe('2',);
                await inSliceLogContext({
                  lane: 'repair',
                  sliceIndex: 3,
                  run: async () => {
                    expect(currentLogContext(),).toStrictEqual({
                      entry: 'Tabby',
                      generation: 'nap-3',
                      attempt: '2',
                      lane: 'repair',
                      slice: '3',
                    },);
                  },
                },);
                expect(currentLogContext().slice,).toBe('',);
                expect(currentLogContext().attempt,).toBe('2',);
              },
            },);
          },
        },),
        it({
          name: 'KEEPS an entry\'s attempt apart from the attempt of the same entry run after it, since a second '
            + 'attempt of one entry begins under a fresh context',
          fn: async () => {
            /**
             Attempt each of two runs of one entry saw.
             */
            const seen: string[] = [];
            for (
              const attempt of [
                1,
                2,
              ]
            ) {
              /* oxlint-disable no-await-in-loop -- the attempts of an entry run one after the other */
              await inEntryLogContext({
                entry: 'Tabby',
                generation: 'nap-3',
                attempt,
                run: async () => {
                  seen.push(currentLogContext().attempt,);
                },
              },);
              /* oxlint-enable no-await-in-loop */
            }
            expect(seen,).toStrictEqual([
              '1',
              '2',
            ],);
          },
        },),
        it({
          name: 'KEEPS each concurrent slice in its own context across awaits',
          fn: async () => {
            /**
             Slice each concurrent run saw after yielding.
             */
            const seen = await inEntryLogContext({
              entry: 'Calico',
              generation: 'nap-3',
              attempt: 1,
              run: async () => await Promise.all([
                1,
                2,
              ].map(async function settleOne(sliceIndex,): Promise<string> {
                return await inSliceLogContext({
                  lane: 'translate',
                  sliceIndex,
                  run: async () => {
                    // A timer turn, so the two slices interleave.
                    await wait(0,);
                    return currentLogContext().slice;
                  },
                },);
              },),),
            },);
            expect(seen,).toStrictEqual([
              '1',
              '2',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: contextRoot.name,
      // SEQUENTIAL: each case diverts the one global `console.info` across an await.
      concurrency: 1,
      children: [
        it({
          name: 'NAMES the entry, its attempt, lane and slice after the root tag, and nothing outside an entry',
          fn: async () => {
            /**
             Root logger under test.
             */
            const l = contextRoot({ tag: 'whiskers', },);
            /**
             Lines printed inside a slice and outside any entry.
             */
            const lines = await infoLinesOf({
              run: async () => {
                await inEntryLogContext({
                  entry: 'Tabby',
                  generation: 'nap-3',
                  attempt: 2,
                  run: async () => {
                    l.info('purr in the entry',);
                    await inSliceLogContext({
                      lane: 'repair',
                      sliceIndex: 3,
                      run: async () => {
                        l.info('purr in the slice',);
                      },
                    },);
                  },
                },);
                l.info('purr outside',);
              },
            },);
            expect(lines.some(function tagsSlice(line,): boolean {
              return line.includes('[whiskers] [Tabby] [attempt 2] [repair slice 3] purr in the slice',);
            },),).toBe(true,);
            expect(lines.some(function tagsEntry(line,): boolean {
              return line.includes('[whiskers] [Tabby] [attempt 2] purr in the entry',);
            },),).toBe(true,);
            expect(lines.some(function bareOutside(line,): boolean {
              return line.includes('[whiskers] purr outside',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ADDS only the attempt, lane and slice to a logger already tagged with the entry',
          fn: async () => {
            /**
             Entry logger under test, tagged with the entry the way the pass
             tags it.
             */
            const l = sliceTagged({ l: tagged({ tag: 'Tabby', },), },);
            /**
             Lines printed inside one slice.
             */
            const lines = await infoLinesOf({
              run: async () => {
                await inEntryLogContext({
                  entry: 'Tabby',
                  generation: 'nap-3',
                  attempt: 4,
                  run: async () => {
                    await inSliceLogContext({
                      lane: 'consolidation',
                      sliceIndex: 7,
                      run: async () => {
                        l.info('kneads',);
                      },
                    },);
                  },
                },);
              },
            },);
            expect(lines.some(function tagsSliceOnce(line,): boolean {
              return line.includes('[Tabby] [attempt 4] [consolidation slice 7] kneads',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'WRITES the attempt, lane and slice ONCE where a lane slice-tags its lines beneath an entry logger that '
            + 'is slice-tagged too, and keeps the order of the tags a lane has always written (ledger B963)',
          fn: async () => {
            /**
             Entry logger, slice-tagged as the pass builds it.
             */
            const entryLogger = sliceTagged({ l: tagged({ tag: 'Tabby', },), },);
            /**
             Lane logger over it, tagged with the lane's function and slice-tagged again, as `repairPreparedDocument` does.
             */
            const laneLogger = sliceTagged({
              l: tagged({
                tag: 'repairPreparedDocument',
                l: entryLogger,
              },),
            },);
            /**
             A stage logger over the entry logger, tagged with a function only.
             */
            const stageLogger = tagged({
              tag: 'editor',
              l: entryLogger,
            },);
            /**
             The printed lines one by one.
             */
            const printed = await printedInOneSlice({
              laneLogger,
              stageLogger,
            },);
            expect(printed.filter(function isLane(line,): boolean {
              return line.includes('[Tabby] [repairPreparedDocument] [attempt 2] [repair slice 3] kneads',);
            },).length,).toBe(1,);
            expect(printed.filter(function isStage(line,): boolean {
              return line.includes('[Tabby] [attempt 2] [repair slice 3] [editor] stretches',);
            },).length,).toBe(1,);
            expect(printed.map(function attemptTags(line,): number {
              return line.split('[attempt 2]',).length - 1;
            },),).toStrictEqual([
              1,
              1,
            ],);
          },
        },),
        it({
          name: 'WRITES the attempt, lane and slice ONCE where a lane slice-tags its lines beneath a module root logger '
            + 'instead of an entry logger, which is what `repairPreparedDocument` does when its caller hands it no '
            + 'parent logger, and keeps the root\'s own order for a stage logger tagged over it (ledger B963)',
          fn: async () => {
            /**
             Module root logger, as `repairPreparedDocument` defaults its parent logger to.
             */
            const rootLogger = contextRoot({ tag: 'translation-repair-pipeline', },);
            /**
             Lane logger over it, tagged with the lane's function and slice-tagged, as `repairPreparedDocument` builds it.
             */
            const laneLogger = sliceTagged({
              l: tagged({
                tag: 'repairPreparedDocument',
                l: rootLogger,
              },),
            },);
            /**
             A stage logger over the root, tagged with a function only.
             */
            const stageLogger = tagged({
              tag: 'editor',
              l: rootLogger,
            },);
            /**
             The printed lines one by one.
             */
            const printed = await printedInOneSlice({
              laneLogger,
              stageLogger,
            },);
            expect(printed.filter(function isLane(line,): boolean {
              return line.includes(
                '[translation-repair-pipeline] [Tabby] [repairPreparedDocument] [attempt 2] [repair slice 3] kneads',
              );
            },).length,).toBe(1,);
            expect(printed.filter(function isStage(line,): boolean {
              return line.includes(
                '[translation-repair-pipeline] [Tabby] [attempt 2] [repair slice 3] [editor] stretches',
              );
            },).length,).toBe(1,);
            expect(printed.map(function attemptTags(line,): number {
              return line.split('[attempt 2]',).length - 1;
            },),).toStrictEqual([
              1,
              1,
            ],);
          },
        },),
        it({
          name: 'PREFIXES EVERY LEVEL, not only info, and forwards a flush (ledger T8)',
          fn: async () => {
            /**
             What the wrapped logger received, as `level message`.
             */
            const received: string[] = [];
            /**
             A level's method that records instead of printing.

             @param level - level name recorded beside the message

             @returns Recording method
             */
            function recording(level: string,): (message: string,) => void {
              return function record(message: string,): void {
                received.push(`${level} ${message}`,);
              };
            }
            /**
             Slice logger over the recording one.
             */
            const l = sliceTagged({
              l: {
                debug: recording('debug',),
                error: recording('error',),
                fatal: recording('fatal',),
                flush: async function flush(): Promise<void> {
                  received.push('flush',);
                },
                info: recording('info',),
                trace: recording('trace',),
                warn: recording('warn',),
              },
            },);
            await inSliceLogContext({
              lane: 'repair',
              sliceIndex: 2,
              run: async () => {
                l.debug('stretches',);
                l.error('knocks a cup over',);
                l.fatal('naps forever',);
                l.info('purrs',);
                l.trace('tracks a moth',);
                l.warn('hisses',);
                await l.flush();
              },
            },);
            expect(received,).toStrictEqual([
              'debug [repair slice 2] stretches',
              'error [repair slice 2] knocks a cup over',
              'fatal [repair slice 2] naps forever',
              'info [repair slice 2] purrs',
              'trace [repair slice 2] tracks a moth',
              'warn [repair slice 2] hisses',
              'flush',
            ],);
          },
        },),
      ],
    },),
  ],
},);
