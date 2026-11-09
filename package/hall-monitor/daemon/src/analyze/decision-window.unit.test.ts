/**
 The productivity streak policy: the decision window, its streak count and the
 notification trigger.

 The daemon notifies once the last {@link DECISION_WINDOW_SIZE} verdicts are
 all unproductive, then starts a fresh window so the same streak does not
 notify twice. The window shift, the count and the trigger are pure, and an
 off-by-one in either (a window of four, a streak that never resets) changes
 when the user is notified, so each boundary has a case here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  countUnproductive,
  DECISION_WINDOW_SIZE,
  everyDecisionUnproductive,
  initialDecisionWindow,
  shiftDecisionWindow,
} from './decision-window.ts';

import type { Decision, DecisionWindow, } from './decision-window.ts';

/** The verdict that builds a streak. */
const UNPRODUCTIVE: Decision = 'UNPRODUCTIVE';

/** The verdict that breaks one. */
const PRODUCTIVE: Decision = 'PRODUCTIVE';

/**
 One step of the daemon's policy: shift the verdict into the window, count the
 streak, and treat a full unproductive window as one notification followed by
 a reset.
 
 @param window - the window before this verdict
 
 @param verdict - the verdict this step records
 
 @returns the window after the step, its streak count, and whether it notified
 */
function step(
  {
    window,
    verdict,
  }: {
    readonly window: DecisionWindow;
    readonly verdict: Decision;
  },
): {
  readonly window: DecisionWindow;
  readonly streakCount: number;
  readonly notified: boolean;
} {
  /** The window with the verdict shifted in. */
  const shifted = shiftDecisionWindow({
    window,
    verdict,
  },);
  /** Whether this step is the notification trigger. */
  const notified = everyDecisionUnproductive(shifted,);

  return {
    window: notified ? initialDecisionWindow() : shifted,
    streakCount: countUnproductive(shifted,),
    notified,
  };
}

/**
 A window holding a given number of unproductive verdicts at its newest end,
 built with the same shift the daemon uses.
 
 @param unproductiveCount - how many of the window's verdicts are unproductive
 
 @returns the window
 */
function windowWithUnproductive(unproductiveCount: number,): DecisionWindow {
  /** Window being filled, starting from the fresh one. */
  let window: DecisionWindow = initialDecisionWindow();
  for (let index = 0; index < unproductiveCount; index += 1) {
    window = shiftDecisionWindow({
      window,
      verdict: UNPRODUCTIVE,
    },);
  }

  return window;
}

/**
 Runs every verdict through {@link step} from a fresh window.
 
 @param verdicts - verdicts in the order the daemon records them
 
 @returns each step's streak count and notification flag
 */
function runStreak(verdicts: readonly Decision[],): {
  readonly streakCounts: readonly number[];
  readonly notifications: readonly boolean[];
} {
  /** Window threaded through the steps. */
  let window: DecisionWindow = initialDecisionWindow();
  /** Streak count after each step. */
  const streakCounts: number[] = [];
  /** Notification flag of each step. */
  const notifications: boolean[] = [];
  for (const verdict of verdicts) {
    /** The result of recording this verdict. */
    const result = step({
      window,
      verdict,
    },);
    window = result.window;
    streakCounts.push(result.streakCount,);
    notifications.push(result.notified,);
  }

  return {
    streakCounts,
    notifications,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: initialDecisionWindow.name,
      children: [
        it({
          name: `starts a window of ${String(DECISION_WINDOW_SIZE,)} productive verdicts`,
          fn: async () => {
            /** The window a fresh daemon starts from. */
            const window = initialDecisionWindow();

            expect(window,).toHaveLength(DECISION_WINDOW_SIZE,);
            expect(window.every(function isProductive(verdict,) {
              return verdict === PRODUCTIVE;
            },),).toBe(true,);
          },
        },),

        it({
          name: 'does not start as a streak and does not notify',
          fn: async () => {
            /** The window a fresh daemon starts from. */
            const window = initialDecisionWindow();

            expect(countUnproductive(window,),).toBe(0,);
            expect(everyDecisionUnproductive(window,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: shiftDecisionWindow.name,
      children: [
        it({
          name: 'keeps the window at its fixed length',
          fn: async () => {
            /** The window after one more verdict. */
            const shifted = shiftDecisionWindow({
              window: initialDecisionWindow(),
              verdict: UNPRODUCTIVE,
            },);

            expect(shifted,).toHaveLength(DECISION_WINDOW_SIZE,);
          },
        },),

        it({
          name: 'drops the oldest verdict and appends the new one',
          fn: async () => {
            /** A window whose oldest entry is the only unproductive verdict. */
            const window: DecisionWindow = [
              UNPRODUCTIVE,
              PRODUCTIVE,
              PRODUCTIVE,
              PRODUCTIVE,
              PRODUCTIVE,
            ];

            /** The window after one productive verdict. */
            const shifted = shiftDecisionWindow({
              window,
              verdict: PRODUCTIVE,
            },);

            // The unproductive verdict left through the oldest position.
            expect(shifted,).toEqual([
              PRODUCTIVE,
              PRODUCTIVE,
              PRODUCTIVE,
              PRODUCTIVE,
              PRODUCTIVE,
            ],);
          },
        },),

        it({
          name: 'keeps the order of the verdicts it carries over',
          fn: async () => {
            /** A window mixing both verdicts in a known order. */
            const window: DecisionWindow = [
              PRODUCTIVE,
              UNPRODUCTIVE,
              PRODUCTIVE,
              UNPRODUCTIVE,
              PRODUCTIVE,
            ];

            expect(shiftDecisionWindow({
              window,
              verdict: UNPRODUCTIVE,
            },),).toEqual([
              UNPRODUCTIVE,
              PRODUCTIVE,
              UNPRODUCTIVE,
              PRODUCTIVE,
              UNPRODUCTIVE,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: countUnproductive.name,
      children: [
        ...Array.from(
          { length: DECISION_WINDOW_SIZE + 1, },
          function makeCountCase(_unused, unproductiveCount,) {
            return it({
              name: `counts ${String(unproductiveCount,)} unproductive verdicts`,
              fn: async () => {
                expect(countUnproductive(windowWithUnproductive(unproductiveCount,),),).toBe(unproductiveCount,);
              },
            },);
          },
        ),
      ],
    },),

    describe({
      name: everyDecisionUnproductive.name,
      children: [
        it({
          name: 'needs every verdict, not only a majority',
          fn: async () => {
            expect(everyDecisionUnproductive(windowWithUnproductive(DECISION_WINDOW_SIZE - 1,),),).toBe(false,);
            expect(everyDecisionUnproductive(initialDecisionWindow(),),).toBe(false,);
          },
        },),

        it({
          name: 'triggers on a window of unproductive verdicts',
          fn: async () => {
            expect(everyDecisionUnproductive(windowWithUnproductive(DECISION_WINDOW_SIZE,),),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'the notification streak',
      concurrency: 1,
      children: [
        it({
          name: `notifies on the ${String(DECISION_WINDOW_SIZE,)}th consecutive unproductive verdict, not before`,
          fn: async () => {
            /** Verdicts that complete exactly one streak. */
            const verdicts = Array.from(
              { length: DECISION_WINDOW_SIZE, },
              function makeVerdict() {
                return UNPRODUCTIVE;
              },
            );

            /** The streak counts and notifications of the run. */
            const run = runStreak(verdicts,);

            expect(run.streakCounts,).toEqual([1, 2, 3, 4, 5],);
            expect(run.notifications,).toEqual([
              false,
              false,
              false,
              false,
              true,
            ],);
          },
        },),

        it({
          name: 'does not notify when a productive verdict breaks the streak',
          fn: async () => {
            /** One verdict short of a streak, then a productive one, then four more. */
            const verdicts: readonly Decision[] = [
              UNPRODUCTIVE,
              UNPRODUCTIVE,
              UNPRODUCTIVE,
              UNPRODUCTIVE,
              PRODUCTIVE,
              UNPRODUCTIVE,
              UNPRODUCTIVE,
              UNPRODUCTIVE,
              UNPRODUCTIVE,
            ];

            /** The streak counts and notifications of the run. */
            const run = runStreak(verdicts,);

            expect(run.notifications.every(function neverNotified(notified,) {
              return !notified;
            },),).toBe(true,);
            // The productive verdict leaves four unproductive verdicts in the window, so every later step
            // counts four and none of them completes a streak.
            expect(run.streakCounts,).toEqual([
              1,
              2,
              3,
              4,
              4,
              4,
              4,
              4,
              4,
            ],);
          },
        },),

        it({
          name: 'starts a fresh window after notifying, so one streak notifies once',
          fn: async () => {
            /** One full streak, then one more unproductive verdict. */
            const verdicts = Array.from(
              { length: DECISION_WINDOW_SIZE + 1, },
              function makeVerdict() {
                return UNPRODUCTIVE;
              },
            );

            /** The streak counts and notifications of the run. */
            const run = runStreak(verdicts,);

            // The fifth verdict notified and reset; the sixth is a streak of one, not a second notification.
            expect(run.streakCounts,).toEqual([1, 2, 3, 4, 5, 1],);
            expect(run.notifications,).toEqual([
              false,
              false,
              false,
              false,
              true,
              false,
            ],);
          },
        },),

        it({
          name: 'notifies again once a second full streak completes',
          fn: async () => {
            /** Two full streaks back to back. */
            const verdicts = Array.from(
              { length: DECISION_WINDOW_SIZE * 2, },
              function makeVerdict() {
                return UNPRODUCTIVE;
              },
            );

            /** The streak counts and notifications of the run. */
            const run = runStreak(verdicts,);

            expect(run.notifications.filter(function notified(flag,) {
              return flag;
            },),).toHaveLength(2,);
          },
        },),
      ],
    },),
  ],
},);
