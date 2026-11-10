/**
 Journal sink tests for the hall-monitor daemon.

 The sink is the daemon's only logging channel, so these cases prove the parts an
 operator relies on: every level reaches its own syslog priority, a message that
 looks like an option stays a message, one record becomes exactly one journal
 invocation, and a journal utility that fails moves the record to stderr instead
 of dropping it.

 The journal utility is injected, so no case writes to the host journal.

 @module
 */

import type {
  Level,
  LogRecord,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
  type TestContext,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createJournalSink,
  journalArgumentsFor,
  reportExpectedFailure,
} from './syslog.ts';
import { recordingLogger, } from './test-fixtures.ts';

//region Fixtures

/**
 Journal tag every argument list must carry, so an operator finds the daemon with
 `journalctl -t hall-monitor`.
 */
const EXPECTED_JOURNAL_TAG = 'hall-monitor';

/**
 Separator that keeps a message from being read as an option of the journal
 utility.
 */
const ARGUMENT_SEPARATOR = '--';

/**
 Priority each level must be filed at, including the levels syslog cannot
 represent exactly.
 */
const EXPECTED_PRIORITY_BY_LEVEL: Record<Level, string> = {
  debug: 'debug',
  error: 'err',
  fatal: 'crit',
  info: 'info',
  trace: 'debug',
  warn: 'warning',
};

/**
 Every level the logger can emit, so the mapping has no untested branch.
 */
const LOG_LEVELS: readonly Level[] = [
  'debug',
  'error',
  'fatal',
  'info',
  'trace',
  'warn',
];

/**
 Record the daemon would file during a capture cycle.
 */
const RECORD: LogRecord = {
  level: 'info',
  message: 'capture complete',
  timestamp: 0,
};

/**
 Sentinel {@link thrownBy} returns when the call it ran threw nothing.
 */
const NO_THROW: unique symbol = Symbol('hall-monitor/test/no-throw',);

/**
 Runs one call and returns the value it threw.

 A case needs to assert on a rethrown value that is deliberately not an `Error`,
 and holding it in a function-root `let` would leak that binding to every later
 statement of the case.

 @param run - call expected to throw

 @returns the thrown value, or {@link NO_THROW} when the call returned

 @example
 ```ts
 expect(thrownBy({ run: () => reportExpectedFailure({ error: 1, l, operation: 'x' }) })).toBe(1);
 ```
 */
function thrownBy({ run, }: Readonly<{ run: () => void; }>,): unknown {
  try {
    run();
  }
  catch (error: unknown) {
    return error;
  }
  return NO_THROW;
}

//endregion Fixtures

await describe({
  name: 'journal sink',
  children: [
    describe({
      name: journalArgumentsFor.name,
      children: [
        it({
          name: 'files every level at its own syslog priority',
          fn: async function testPriorities(): Promise<void> {
            for (const level of LOG_LEVELS) {
              const args = journalArgumentsFor({
                ...RECORD,
                level,
              },);

              expect(args,).toEqual([
                '-t',
                EXPECTED_JOURNAL_TAG,
                '-p',
                `user.${EXPECTED_PRIORITY_BY_LEVEL[level]}`,
                ARGUMENT_SEPARATOR,
                RECORD.message,
              ],);
            }
          },
        },),
        it({
          name: 'keeps a message that starts with an option after the separator',
          fn: async function testOptionLikeMessage(): Promise<void> {
            /**
             Message a log line could plausibly carry, and which the journal
             utility would otherwise read as options.
             */
            const message = '-n 5 --priority debug';
            const args = journalArgumentsFor({
              ...RECORD,
              message,
            },);

            expect(args.indexOf(ARGUMENT_SEPARATOR,),).toBe(args.length - 2,);
            expect(args.at(-1,),).toBe(message,);
          },
        },),
      ],
    },),
    describe({
      name: reportExpectedFailure.name,
      children: [
        it({
          name: 'files the caught value of a failure the call site expected',
          fn: async function testExpectedFailureIsFiled(): Promise<void> {
            /**
             Recording logger the failure is reported through, asserted on below.
             */
            const { logger, records, } = recordingLogger();

            reportExpectedFailure({
              error: new Error('pkill exited 1',),
              l: logger,
              operation: 'pkill',
            },);

            expect(records,).toEqual([
              {
                level: 'debug',
                message: 'pkill failed: pkill exited 1',
              },
            ],);
          },
        },),
        it({
          name: 'rethrows a caught value that is not an Error, so an unplanned throw stays fatal',
          fn: async function testNonErrorIsRethrown(): Promise<void> {
            /**
             Recording logger the failure would be reported through; the case proves nothing reached it.
             */
            const { logger, records, } = recordingLogger();

            /**
             Value the call rethrows; captured rather than asserted with `toThrow`, because the
             rethrown value is not an Error.
             */
            const rethrown = thrownBy({
              run: function reportUnexpected(): void {
                reportExpectedFailure({
                  error: 'not an Error',
                  l: logger,
                  operation: 'pkill',
                },);
              },
            },);

            expect(rethrown,).toBe('not an Error',);
            expect(records,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: createJournalSink.name,
      children: [
        it({
          name: 'files one invocation per record',
          fn: async function testOneInvocationPerRecord(): Promise<void> {
            /**
             Argument lists the sink handed the injected utility.
             */
            const invocations: (readonly string[])[] = [];
            const sink = createJournalSink({
              run: async function record(args: readonly string[],): Promise<void> {
                invocations.push(args,);
              },
            },);
            /**
             Second record, so the case proves the sink files every write rather
             than only the first.
             */
            const secondRecord: LogRecord = {
              level: 'error',
              message: 'capture failed',
              timestamp: 1,
            };

            await sink.write(RECORD,);
            await sink.write(secondRecord,);

            expect(invocations,).toEqual([
              journalArgumentsFor(RECORD,),
              journalArgumentsFor(secondRecord,),
            ],);
          },
        },),
        it({
          name: 'reports a failing journal utility on stderr instead of losing the record',
          fn: async function testFailureFallback({
            sinon,
          }: TestContext,): Promise<void> {
            /**
             Stubbed stderr write, standing in for the journal utility that
             refused the record.
             */
            const stderrStub = sinon.stub(
              process.stderr,
              'write',
            );
            const sink = createJournalSink({
              run: async function refuse(): Promise<void> {
                throw new Error('journal socket refused the record',);
              },
            },);

            await sink.write(RECORD,);

            expect(stderrStub.callCount,).toBe(1,);
            /**
             Line the fallback wrote.
             */
            const written: unknown = stderrStub.firstCall?.args[0];
            expect(typeof written,).toBe('string',);
            if ((typeof written) !== 'string')
              throw new Error('stderr fallback wrote no string',);
            expect(written,).toContain(EXPECTED_JOURNAL_TAG,);
            expect(written,).toContain(RECORD.message,);
            expect(written,).toContain('journal socket refused the record',);
          },
        },),
        it({
          name: 'verifies as available, because a missing utility still reaches stderr',
          fn: async function testVerify(): Promise<void> {
            const sink = createJournalSink();

            expect(await sink.verify(),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
