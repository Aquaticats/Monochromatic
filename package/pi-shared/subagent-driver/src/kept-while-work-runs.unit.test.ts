/**
 What the driver does with a read of the work a run left running, and with a message it sends about that work: a read whose
 failure keeps the run, a read whose failure is news again after a good one, and a wake whose send failed. Nothing here starts a
 process, a unit or a Pi.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CommandAnswer,
  UNREADABLE_WORK_MILLISECONDS,
  type WatchRequest,
  type WorkWaitedFor,
  askAboutItsWorkOrReportAFailure,
  answeredItsWake,
} from '../dist/final/node/index.mjs';

/**
 A watch holding only what the calls in this file read: the work the run has, the messages that go to Pi, and what the driver notes.

 Everything else a watch holds belongs to the watch loop, which no test here runs, so the object is handed over as a watch rather
 than pretending to be one.

 @param read - how the run's work is read

 @param send - how a message is sent to Pi

 @param noted - where a feed line the driver writes is kept

 @param problems - where the summary's work problem is kept

 @param feedFailed - whether the driver's own feed has already failed

 @returns a watch these calls read

 @example
 ```ts
 const watch = aWatchWith({ read: async () => [], send: async () => ({ ending: 'silence', problem: 'no answer' }), },);
 ```
 */
function aWatchWith({
  read,
  send,
  noted = [],
  problems = [],
  feedFailed = false,
}: {
  readonly read: () => Promise<readonly string[]>;
  readonly send: (request: {
    readonly command: {
      readonly type: string;
      readonly message: string;
    };
    readonly deadline: number;
  },) => Promise<CommandAnswer>;
  readonly noted?: Record<string, unknown>[];
  readonly problems?: string[];
  readonly feedFailed?: boolean;
},): WatchRequest {
  return {
    workLeftRunning: read,
    deadline: Date.now() + 60_000,
    connection: {
      send,
    },
    record: {
      settledCount: function settledCount() {
        return 0;
      },
      note: function note(entry: Record<string, unknown>,) {
        noted.push(entry,);
      },
      noteWorkProblem: function noteWorkProblem(problem: string,) {
        problems.push(problem,);
      },
      feedProblem: function feedProblem() {
        return { failed: feedFailed, };
      },
    },
  } as unknown as WatchRequest;
}

/**
 The failure a read of the process table gives in this file, which is no process having gone.

 @returns the failure

 @example
 ```ts
 throw anUnreadableProcessTable();
 ```
 */
function anUnreadableProcessTable(): Error {
  return new Error('EIO while reading the process table',);
}

await describe({
  name: '',
  children: [
    describe({
      name: askAboutItsWorkOrReportAFailure.name,
      children: [
        it({
          name: 'sends a wake once when its send throws, keeps the run for a reply that may yet come, and notes it as a wake rather than a read',
          fn: async () => {
            /**
             The messages that went to Pi.
             */
            const sent: string[] = [];
            /**
             The feed lines the driver wrote.
             */
            const noted: Record<string, unknown>[] = [];
            /**
             The work problems the driver reported, of which a failed send is none.
             */
            const problems: string[] = [];
            /**
             How many times the work was read.
             */
            let reads = 0;
            /**
             A watch whose work runs once and is gone after that, and whose first message is accepted by Pi before the answer path
             fails.
             */
            const watch = aWatchWith({
              read: async function read() {
                reads += 1;
                return (reads === 1) ? ['4242 build',] : [];
              },
              send: async function send({ command, },) {
                sent.push(command.message,);
                throw new Error('the answer path failed after Pi took the prompt',);
              },
              noted,
              problems,
            },);
            /**
             Where the driver stands with the work.
             */
            const waited: WorkWaitedFor = {};
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('kept',);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('resumed',);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('nothing',);
            // The wake went out once and is never sent again, because its send failed after Pi may have taken it.
            expect(sent.length,).toBe(1,);
            expect(problems,).toEqual([],);
            /**
             The feed's line for that wake.
             */
            const wake = noted.find(function isTheWake(entry,) {
              return entry.kind === 'wake';
            },);
            expect(wake?.acceptance,).toBe('unknown',);
            expect(String(wake?.error,),).toContain('the answer path failed after Pi took the prompt',);
            // A reply to that wake is still owed, so the run is not free to end as one that came back.
            expect(waited.wokenAtSettle,).toBe(0,);
            expect(waited.work,).toBeUndefined();
          },
        },),
        it({
          name: 'sends a wake once when its answer never came, and keeps the run for a turn that may still settle',
          fn: async () => {
            /**
             The messages that went to Pi.
             */
            const sent: string[] = [];
            /**
             The feed lines the driver wrote.
             */
            const noted: Record<string, unknown>[] = [];
            /**
             How many times the work was read.
             */
            let reads = 0;
            /**
             A watch whose work ran once, and whose wake Pi accepted while its answer was lost: the connection answers as it does
             when no answer arrives, which is a resolved value and not a throw.
             */
            const watch = aWatchWith({
              read: async function read() {
                reads += 1;
                return (reads === 1) ? ['4242 build',] : [];
              },
              send: async function send({ command, },) {
                sent.push(command.message,);
                return {
                  ending: 'silence',
                  problem: 'no answer in 30000 ms',
                };
              },
              noted,
            },);
            /**
             Where the driver stands with the work.
             */
            const waited: WorkWaitedFor = {};
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('kept',);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('resumed',);
            // The wake went out once, and no turn has settled since, so the run is not free to end as one that came back.
            expect(sent.length,).toBe(1,);
            expect(waited.wokenAtSettle,).toBe(0,);
            expect(answeredItsWake({
              ...((waited.wokenAtSettle === undefined) ? {} : { wokenAtSettle: waited.wokenAtSettle, }),
              settledNow: 0,
            },),).toBe(false,);
            /**
             The feed's line for that wake, which says the acceptance is unknown rather than that Pi declined it.
             */
            const wake = noted.find(function isTheWake(entry,) {
              return entry.kind === 'wake';
            },);
            expect(wake?.acceptance,).toBe('unknown',);
            expect(String(wake?.error,),).toContain('no answer in 30000 ms',);
          },
        },),
        it({
          name: 'ends the run as one whose leftover work could not be read after the span of failed reads, and keeps it before that',
          fn: async () => {
            /**
             The instant the test's clock stands at.
             */
            let clock = 1_000_000;
            /**
             A watch whose read of the run's work always fails.
             */
            const watch = aWatchWith({
              read: async function read() {
                throw anUnreadableProcessTable();
              },
              send: async function send() {
                throw new Error('no message is sent in this test',);
              },
            },);
            /**
             Where the driver stands with the work.
             */
            const waited: WorkWaitedFor = {};
            /**
             Where the test's clock is read from.
             */
            function now(): number {
              return clock;
            }
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
              now,
            },),).toBe('kept',);
            clock += UNREADABLE_WORK_MILLISECONDS - 1;
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
              now,
            },),).toBe('kept',);
            clock += 1;
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
              now,
            },),).toBe('work-unreadable',);
          },
        },),
        it({
          name: 'keeps the run when a failed read recovers before the span, and counts the span again from the next failure',
          fn: async () => {
            /**
             The instant the test's clock stands at.
             */
            let clock = 1_000_000;
            /**
             How many times the work was read, of which the second answered.
             */
            let reads = 0;
            /**
             A watch whose read fails, answers, and fails again.
             */
            const watch = aWatchWith({
              read: async function read() {
                reads += 1;
                if (reads === 2) {
                  return [];
                }
                throw anUnreadableProcessTable();
              },
              send: async function send() {
                throw new Error('no message is sent in this test',);
              },
            },);
            /**
             Where the driver stands with the work.
             */
            const waited: WorkWaitedFor = {};
            /**
             Where the test's clock is read from.
             */
            function now(): number {
              return clock;
            }
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
              now,
            },),).toBe('kept',);
            clock += UNREADABLE_WORK_MILLISECONDS * 2;
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
              now,
            },),).toBe('nothing',);
            // The good read cleared the span, so this failure is counted from its own instant rather than from the one before it.
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
              now,
            },),).toBe('kept',);
            expect(waited.unreadableSince,).toBe(clock,);
          },
        },),
        it({
          name: 'notes a read that fails, once while it lasts, and again when it comes back after a good read',
          fn: async () => {
            /**
             The work problems the driver reported.
             */
            const problems: string[] = [];
            /**
             How many times the work was read, of which the second answered.
             */
            let reads = 0;
            /**
             A watch whose read fails, answers, fails again, and fails once more with the same failure.
             */
            const watch = aWatchWith({
              read: async function read() {
                reads += 1;
                if (reads === 2) {
                  return [];
                }
                throw anUnreadableProcessTable();
              },
              send: async function send() {
                throw new Error('no message is sent in this test',);
              },
              problems,
            },);
            /**
             Where the driver stands with the work.
             */
            const waited: WorkWaitedFor = {};
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('kept',);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('nothing',);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('kept',);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited,
            },),).toBe('kept',);
            // One note for the failure before the good read and one for the failure after it, and none for the repeat of that one.
            /**
             The text of the failure, which is what the driver keeps and reports.
             */
            const failure = String(anUnreadableProcessTable(),);
            expect(problems,).toEqual([failure, failure,],);
            expect(waited.unreadable,).toBe(failure,);
          },
        },),
        it({
          name: 'reads as nothing when its read fails and the driver\'s own feed has already failed',
          fn: async () => {
            /**
             A watch whose read fails and whose own record of the run has failed.
             */
            const watch = aWatchWith({
              read: async function read() {
                throw anUnreadableProcessTable();
              },
              send: async function send() {
                throw new Error('no message is sent in this test',);
              },
              feedFailed: true,
            },);
            expect(await askAboutItsWorkOrReportAFailure({
              watch,
              waited: {},
            },),).toBe('nothing',);
          },
        },),
      ],
    },),
  ],
},);
