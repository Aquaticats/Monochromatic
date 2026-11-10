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

import type { Level, LogRecord, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
  type TestContext,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createJournalSink,
  journalArgumentsFor,
} from './syslog.ts';

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
              message: '[error] capture failed',
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
