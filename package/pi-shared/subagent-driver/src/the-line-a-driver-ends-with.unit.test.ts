/**
 The line the driver prints as a run that did not end on its own terms ends: whether it names what a run ended `work-unreadable` kept
 its worktree for.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  endingLineOf,
  type RunSummary,
  UNREADABLE_WORK_MILLISECONDS,
} from '../dist/final/node/index.mjs';

/**
 One run's summary, holding only what the line reads, with every other field a run that did nothing.

 @param fields - what to override

 @returns the summary

 @example
 ```ts
 const summary = aSummary({ endReason: 'work-unreadable', },);
 ```
 */
function aSummary(fields: Readonly<Record<string, unknown>> = {},): RunSummary {
  return {
    endReason: 'idle',
    totalMilliseconds: 1_000,
    turns: 0,
    settled: 1,
    toolHistogram: {},
    toolErrors: [],
    dialogs: [],
    sessionStats: null,
    lastAssistantText: '',
    retries: 0,
    rateLimitErrors: 0,
    piStartAttempts: 1,
    ...fields,
  } as unknown as RunSummary;
}

await describe({
  name: endingLineOf.name,
  children: [
    it({
      name: 'names the work a run kept its worktree for, and says the worktree is kept and that work may still run',
      fn: async () => {
        /**
         The line a run ended on work it could not read ends with.
         */
        const line = endingLineOf({
          summary: aSummary({
            endReason: 'work-unreadable',
            workProblem: 'EIO while reading the process table',
            workNotRead: ['proc_build 4242 cargo build',],
          },),
          runFolder: '/runs/a-1',
        },);
        expect(line,).toContain('"work-unreadable"',);
        expect(line,).toContain(String(UNREADABLE_WORK_MILLISECONDS,),);
        expect(line,).toContain('proc_build 4242 cargo build',);
        expect(line,).toContain('may still run',);
        expect(line,).toContain('worktree is kept',);
        expect(line,).toContain('/runs/a-1/summary.json',);
      },
    },),
    it({
      name: 'says the driver knew of no work when a run it could not read had none to name',
      fn: async () => {
        /**
         The line of a run whose reads failed before anything was seen.
         */
        const line = endingLineOf({
          summary: aSummary({
            endReason: 'work-unreadable',
            workProblem: 'EIO while reading the process table',
          },),
          runFolder: '/runs/a-1',
        },);
        expect(line,).toContain('it knew of no work',);
        expect(line,).toContain('worktree is kept',);
      },
    },),
    it({
      name: 'says nothing about work it did not read when the run ended for another reason',
      fn: async () => {
        expect(endingLineOf({
          summary: aSummary({ endReason: 'wall-clock', },),
          runFolder: '/runs/a-1',
        },),).not.toContain('worktree is kept',);
      },
    },),
  ],
},);
