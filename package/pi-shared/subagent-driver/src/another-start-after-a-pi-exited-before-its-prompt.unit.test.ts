/**
 The decision to start Pi again after a start that ended before its Pi took the prompt: which endings it answers, the bounds that
 stop it, and the pause it waits. Every case hands it a clock of its own, so no case waits on real time.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AnotherStart,
  anotherStartOf,
  START_ATTEMPTS,
  startThatTookNoPromptOf,
  waitForAnotherStart,
} from '../dist/final/node/index.mjs';

/**
 A clock a case reads and moves by hand, which records how long each wait asked for.

 @param startsAt - the time the clock is set to first

 @returns the clock
 */
function clockAt(startsAt: number,): {
  readonly now: () => number;
  readonly waits: readonly number[];
  readonly wait: (milliseconds: number) => Promise<void>
} {
  /**
   What changes as the case runs: the time, and every wait asked of it.
   */
  const state: { at: number; waits: number[] } = {
    at: startsAt,
    waits: [],
  };
  return {
    now: function now() {
      return state.at;
    },
    waits: state.waits,
    wait: async function wait(milliseconds,) {
      state.waits.push(milliseconds,);
      state.at += milliseconds;
    },
  };
}

/**
 A run's bounds as a case sets them: the wall clock far away, and the sequence of starts far enough for one more start.

 @param now - when the case starts

 @returns the bounds
 */
function roomForAnotherStart(now: number,): {
  readonly wallDeadline: number;
  readonly startSequenceDeadline: number
} {
  return {
    wallDeadline: now + 600_000,
    startSequenceDeadline: now + 120_000,
  };
}

await describe({
  name: startThatTookNoPromptOf.name,
  children: [
    it({
      name: 'answers a Pi that exited with a status before it took the prompt',
      fn: async () => {
        expect(startThatTookNoPromptOf({
          endReason: 'pi-exited',
          tookPrompt: false,
          turnsStarted: 0,
          exit: { code: 1, },
        },),)
          .toEqual({
            tookNoPrompt: true,
            exit: { code: 1, },
          },);
      },
    },),
    it({
      name: 'answers no Pi that took the prompt or started a turn, because work may have begun',
      fn: async () => {
        expect(startThatTookNoPromptOf({
          endReason: 'pi-exited',
          tookPrompt: true,
          turnsStarted: 0,
          exit: { code: 1, },
        },),)
          .toEqual({ tookNoPrompt: false, },);
        expect(startThatTookNoPromptOf({
          endReason: 'pi-exited',
          tookPrompt: false,
          turnsStarted: 1,
          exit: { code: 1, },
        },),)
          .toEqual({ tookNoPrompt: false, },);
      },
    },),
    it({
      name: 'answers no Pi the operating system refused to start, and no exit that names neither a status nor a signal',
      fn: async () => {
        // Nothing about the machine changes between two such tries, so a pause would only make the run end later.
        expect(startThatTookNoPromptOf({
          endReason: 'pi-exited',
          tookPrompt: false,
          turnsStarted: 0,
          exit: { processError: 'spawn pi ENOENT', },
        },),)
          .toEqual({ tookNoPrompt: false, },);
        expect(startThatTookNoPromptOf({
          endReason: 'pi-exited',
          tookPrompt: false,
          turnsStarted: 0,
          exit: {},
        },),)
          .toEqual({ tookNoPrompt: false, },);
      },
    },),
    it({
      name: 'answers no run that ended for another reason, whatever Pi did',
      fn: async () => {
        for (const endReason of ['wall-clock', 'prompt-rejected', 'driver-failed', 'stop-file', 'idle'] as const) {
          expect(startThatTookNoPromptOf({
            endReason,
            tookPrompt: false,
            turnsStarted: 0,
            exit: { code: 1, },
          },),)
            .toEqual({ tookNoPrompt: false, },);
        }
      },
    },),
  ],
},);

await describe({
  name: anotherStartOf.name,
  children: [
    it({
      name: 'starts Pi again after the pause when the starts and the time one start-up takes both leave room for it',
      fn: async () => {
        /**
         The clock, which this case only reads.
         */
        const clock = clockAt(1_000,);
        /**
         What the driver decides.
         */
        const another: AnotherStart = anotherStartOf({
          startsTaken: 1,
          pauseMilliseconds: 10_000,
          now: clock.now(),
          ...roomForAnotherStart(clock.now(),),
        },);
        expect(another,).toEqual({
          startAgain: true,
          pauseMilliseconds: 10_000,
        },);
      },
    },),
    it({
      name: 'stops once the starts a run allows are spent',
      fn: async () => {
        /**
         The clock, which this case only reads.
         */
        const clock = clockAt(1_000,);
        expect(anotherStartOf({
          startsTaken: START_ATTEMPTS,
          pauseMilliseconds: 10_000,
          now: clock.now(),
          ...roomForAnotherStart(clock.now(),),
        },),)
          .toEqual({
            startAgain: false,
            why: 'tries-spent',
          },);
      },
    },),
    it({
      name: 'stops when the time one start-up takes is spent, so a slow failure is not paid for three times',
      fn: async () => {
        /**
         The clock, of a start that began one start-up bound ago.
         */
        const clock = clockAt(120_000,);
        expect(anotherStartOf({
          startsTaken: 1,
          pauseMilliseconds: 10_000,
          now: clock.now(),
          wallDeadline: clock.now() + 600_000,
          startSequenceDeadline: clock.now(),
        },),)
          .toEqual({
            startAgain: false,
            why: 'start-sequence-spent',
          },);
      },
    },),
    it({
      name: 'stops when the run has less wall clock left than the pause needs, and says that bound rather than the other',
      fn: async () => {
        /**
         The clock of a run whose wall clock ends before its sequence of starts would.
         */
        const clock = clockAt(50_000,);
        expect(anotherStartOf({
          startsTaken: 1,
          pauseMilliseconds: 10_000,
          now: clock.now(),
          wallDeadline: clock.now() + 5_000,
          startSequenceDeadline: clock.now() + 120_000,
        },),)
          .toEqual({
            startAgain: false,
            why: 'wall-clock-too-near',
          },);
      },
    },),
  ],
},);

await describe({
  name: waitForAnotherStart.name,
  children: [
    it({
      name: 'waits the whole pause while the bound leaves room after it, and answers that a start may be made',
      fn: async () => {
        /**
         The clock, which moves by what the wait asks of it.
         */
        const clock = clockAt(1_000,);
        expect(await waitForAnotherStart({
          pauseMilliseconds: 10_000,
          deadline: clock.now() + 120_000,
          now: clock.now,
          wait: clock.wait,
        },),)
          .toBe(true,);
        expect(clock.waits,).toEqual([10_000,],);
      },
    },),
    it({
      name: 'waits only what the bound leaves, and answers that no start may follow a pause that crossed it',
      fn: async () => {
        /**
         The clock, which moves by what the wait asks of it.
         */
        const clock = clockAt(1_000,);
        expect(await waitForAnotherStart({
          pauseMilliseconds: 10_000,
          deadline: clock.now() + 4_000,
          now: clock.now,
          wait: clock.wait,
        },),)
          .toBe(false,);
        expect(clock.waits,).toEqual([4_000,],);
      },
    },),
    it({
      name: 'waits nothing when the bound has passed, so no start is made into a run that is already over',
      fn: async () => {
        /**
         The clock of a run whose bound is behind it.
         */
        const clock = clockAt(1_000,);
        expect(await waitForAnotherStart({
          pauseMilliseconds: 10_000,
          deadline: clock.now(),
          now: clock.now,
          wait: clock.wait,
        },),)
          .toBe(false,);
        expect(clock.waits,).toEqual([],);
      },
    },),
  ],
},);
