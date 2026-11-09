/**
 The one wait of the watch: it ends at a terminal state, prints only the findings, and never waits past its deadline. The clock
 and the wait are the test's, so nothing here sleeps.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type FinishedRunEvidence,
  type OverdueLook,
  watchForOverdueRuns,
} from '../dist/final/node/index.mjs';

/**
 The finished runs a look here compares against: two of them, and calls that ended.
 */
const EVIDENCE: FinishedRunEvidence = {
  finishedRuns: 2,
  overdueMilliseconds: 90 * 60_000,
  toolCalls: 4,
  openCallMilliseconds: 10 * 60_000,
};

/**
 A finished runs folder with nothing in it, which says no limit at all.
 */
const NO_EVIDENCE: FinishedRunEvidence = {
  finishedRuns: 0,
  toolCalls: 0,
};

/**
 Builds what one look answers with.

 @param lines - the lines the look found

 @param anyRunning - whether any run is going

 @param evidence - what the look compared against

 @returns what a look answers with

 @example
 ```ts
 const found = lookFound({ lines: ['OVERDUE pi-a.service',], },);
 ```
 */
function lookFound({
  lines = [],
  anyRunning = true,
  evidence = EVIDENCE,
}: {
  readonly lines?: readonly string[];
  readonly anyRunning?: boolean;
  readonly evidence?: FinishedRunEvidence;
},): OverdueLook {
  return {
    lines,
    anyRunning,
    reported: {},
    evidence,
  };
}

/**
 A clock and a wait the test owns, so a wait costs nothing and is counted.

 @param start - the moment the clock starts at

 @returns the clock, the waits it recorded, and how many looks were taken

 @example
 ```ts
 const clock = standInClock({ start: 0, },);
 ```
 */
function standInClock({ start, }: { readonly start: number; },) {
  return {
    at: start,
    waits: [] as number[],
    looks: 0,
    now: function now() {
      return this.at;
    },
    wait: async function wait(milliseconds: number) {
      this.waits.push(milliseconds,);
      this.at += milliseconds;
    },
  };
}

await describe({
  name: watchForOverdueRuns.name,
  children: [
    it({
      name: 'answers with the findings of the first look that found one, without waiting',
      fn: async () => {
        /**
         The clock the wait reads.
         */
        const clock = standInClock({ start: 0, },);
        /**
         The lines the wait answers with.
         */
        const lines = await watchForOverdueRuns({
          look: async function found() {
            clock.looks += 1;
            return lookFound({ lines: ['OVERDUE pi-alpha.service: running long',], },);
          },
          now: clock.now.bind(clock,),
          wait: clock.wait.bind(clock,),
          deadline: 1_000,
          intervalMilliseconds: 600,
          once: false,
        },);
        expect(lines,).toEqual(['OVERDUE pi-alpha.service: running long',],);
        expect(clock.waits,).toEqual([],);
      },
    },),
    it({
      name: 'ends at once when no Pi run is going, so a caller that starts one later starts the watch again',
      fn: async () => {
        /**
         The clock the wait reads.
         */
        const clock = standInClock({ start: 0, },);
        expect(await watchForOverdueRuns({
          look: async function none() {
            return lookFound({ anyRunning: false, },);
          },
          now: clock.now.bind(clock,),
          wait: clock.wait.bind(clock,),
          deadline: 1_000,
          intervalMilliseconds: 600,
          once: false,
        },),).toEqual(['no Pi agent is running',],);
      },
    },),
    it({
      name: 'looks once when asked to, and says what it compared against',
      fn: async () => {
        /**
         The clock the wait reads.
         */
        const clock = standInClock({ start: 0, },);
        /**
         The line the wait answers with.
         */
        const lines = await watchForOverdueRuns({
          look: async function none() {
            clock.looks += 1;
            return lookFound({},);
          },
          now: clock.now.bind(clock,),
          wait: clock.wait.bind(clock,),
          deadline: 1_000,
          intervalMilliseconds: 600,
          once: true,
        },);
        expect(clock.looks,).toBe(1,);
        expect(lines,).toHaveLength(1,);
        expect(lines[0],).toContain('no Pi agent is past a limit',);
        expect(lines[0],).toContain('90% of 2 finished runs were done within 90 min',);
        expect(clock.waits,).toEqual([],);
      },
    },),
    it({
      name: 'waits no longer than the deadline leaves, and ends at the deadline',
      fn: async () => {
        /**
         The clock the wait reads.
         */
        const clock = standInClock({ start: 0, },);
        /**
         The line the wait answers with.
         */
        const lines = await watchForOverdueRuns({
          look: async function none() {
            clock.looks += 1;
            return lookFound({},);
          },
          now: clock.now.bind(clock,),
          wait: clock.wait.bind(clock,),
          deadline: 1_000,
          intervalMilliseconds: 600,
          once: false,
        },);
        expect(clock.waits,).toEqual([600, 400,],);
        expect(clock.at,).toBe(1_000,);
        expect(clock.looks,).toBe(3,);
        expect(lines,).toHaveLength(1,);
        expect(lines[0],).toContain('the watch reached its deadline with no Pi agent past a limit',);
      },
    },),
    it({
      name: 'says at its deadline that no finished run was there to compare against',
      fn: async () => {
        /**
         The clock the wait reads.
         */
        const clock = standInClock({ start: 0, },);
        /**
         The line the wait answers with.
         */
        const lines = await watchForOverdueRuns({
          look: async function none() {
            return lookFound({ evidence: NO_EVIDENCE, },);
          },
          now: clock.now.bind(clock,),
          wait: clock.wait.bind(clock,),
          deadline: 0,
          intervalMilliseconds: 600,
          once: false,
        },);
        expect(lines,).toHaveLength(1,);
        expect(lines[0],).toContain('no finished run with a summary is in the runs folder',);
      },
    },),
  ],
},);
