/**
 The productivity streak policy: the decision window, its streak count and the
 notification trigger.

 The daemon notifies once the last {@link DECISION_WINDOW_SIZE} verdicts are
 all unproductive, then starts a fresh window so the same streak does not
 notify twice. The window shift, the count and the trigger are pure, and an
 off-by-one in either (a window of four, a streak that never resets) changes
 when the user is interrupted, so each boundary has a case here.

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
  recordVerdict,
  shiftDecisionWindow,
  type Decision,
  type DecisionWindow,
} from '../../dist/final/node/testing.mjs';

/** The verdict that builds a streak. */
const UNPRODUCTIVE: Decision = 'UNPRODUCTIVE';

/** The verdict that breaks one. */
const PRODUCTIVE: Decision = 'PRODUCTIVE';

/**
 What a run of verdicts did, step by step.
 */
type StreakRun = {
  /**
   The window after the last verdict.
   */
  readonly window: DecisionWindow;
  /**
   Unproductive verdicts in the window after each step, before any reset.
   */
  readonly streakCounts: readonly number[];
  /**
   Whether each step notified.
   */
  readonly notifications: readonly boolean[];
};

/**
 Runs every verdict through {@link recordVerdict} and the reset `cycle()` performs
 after it has notified.
 
 The reset is the caller's step in production, so this helper states the order
 the daemon relies on: shift, notify a human, then start a fresh window.
 
 @param verdicts - verdicts in the order the daemon records them
 
 @returns each step's streak count and notification flag
 */
function runStreak(verdicts: readonly Decision[],): StreakRun {
  return verdicts.reduce<StreakRun>(
    function recordStep(
      run,
      verdict,
    ) {
      /** What this verdict did to the window and the streak. */
      const {
        window: filledWindow,
        streakCount,
        notified,
      } = recordVerdict({
        window: run.window,
        verdict,
      },);

      return {
        window: notified ? initialDecisionWindow() : filledWindow,
        streakCounts: [
          ...run.streakCounts,
          streakCount,
        ],
        notifications: [
          ...run.notifications,
          notified,
        ],
      };
    },
    {
      window: initialDecisionWindow(),
      streakCounts: [],
      notifications: [],
    },
  );
}

/**
 A window holding a given number of unproductive verdicts at its newest end,
 built with the same shift the daemon uses.
 
 @param unproductiveCount - how many of the window's verdicts are unproductive
 
 @returns the window
 */
function windowWithUnproductive(unproductiveCount: number,): DecisionWindow {
  /** Placeholder entries, only their count matters. */
  const placeholders = Array.from(
    { length: unproductiveCount, },
    function makePlaceholder(_unused, index,) {
      return index;
    },
  );

  return placeholders.reduce<DecisionWindow>(
    function shiftIn(window,) {
      return shiftDecisionWindow({
        window,
        verdict: UNPRODUCTIVE,
      },);
    },
    initialDecisionWindow(),
  );
}

/**
 The parameterized streak-count cases: a window with zero up to all of its
 verdicts unproductive.
 */
const countCases = Array.from(
  { length: DECISION_WINDOW_SIZE + 1, },
  function makeCountCase(_unused, unproductiveCount,) {
    return it({
      name: `counts ${String(unproductiveCount,)} unproductive verdicts`,
      fn: async () => {
        /** The window under test. */
        const window = windowWithUnproductive(unproductiveCount,);

        expect(countUnproductive(window,),).toBe(unproductiveCount,);
      },
    },);
  },
);

/**
 A run of verdicts that is exactly one full streak.
 */
const ONE_STREAK: readonly Decision[] = Array.from(
  { length: DECISION_WINDOW_SIZE, },
  function makeVerdict() {
    return UNPRODUCTIVE;
  },
);

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
            expect(shifted,)
              .toEqual([
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
            /** The window after one more unproductive verdict. */
            const shifted = shiftDecisionWindow({
              window,
              verdict: UNPRODUCTIVE,
            },);

            expect(shifted,)
              .toEqual([
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
      children: countCases,
    },),

    describe({
      name: everyDecisionUnproductive.name,
      children: [
        it({
          name: 'needs every verdict, not only a majority',
          fn: async () => {
            /** A window one verdict short of a full streak. */
            const almost = windowWithUnproductive(DECISION_WINDOW_SIZE - 1,);
            /** A window with no unproductive verdict at all. */
            const none = windowWithUnproductive(0,);

            expect(everyDecisionUnproductive(almost,),).toBe(false,);
            expect(everyDecisionUnproductive(none,),).toBe(false,);
          },
        },),

        it({
          name: 'triggers on a window of unproductive verdicts',
          fn: async () => {
            /** A full unproductive window. */
            const streak = windowWithUnproductive(DECISION_WINDOW_SIZE,);

            expect(everyDecisionUnproductive(streak,),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: recordVerdict.name,
      children: [
        it({
          name: 'leaves the completed window in place, so the caller can reset it after notifying',
          fn: async () => {
            /** A window one unproductive verdict short of a streak. */
            const before = windowWithUnproductive(DECISION_WINDOW_SIZE - 1,);
            /** What recording the last unproductive verdict reports. */
            const step = recordVerdict({
              window: before,
              verdict: UNPRODUCTIVE,
            },);

            expect(step.notified,).toBe(true,);
            expect(step.streakCount,).toBe(DECISION_WINDOW_SIZE,);
            // Not reset here: `cycle()` resets only after the notification has been sent.
            expect(everyDecisionUnproductive(step.window,),).toBe(true,);
          },
        },),

        it({
          name: 'reports the unproductive count without resetting a window that is not a streak',
          fn: async () => {
            /** A window with two unproductive verdicts at its newest end. */
            const before = windowWithUnproductive(2,);
            /** What recording one more unproductive verdict reports. */
            const step = recordVerdict({
              window: before,
              verdict: UNPRODUCTIVE,
            },);

            expect(step.notified,).toBe(false,);
            expect(step.streakCount,).toBe(3,);
            expect(step.window,).toEqual(windowWithUnproductive(3,),);
          },
        },),
      ],
    },),

    describe({
      name: 'the notification streak',
      children: [
        it({
          name: `notifies on the ${String(DECISION_WINDOW_SIZE,)}th consecutive unproductive verdict, not before`,
          fn: async () => {
            /** The streak counts and notifications of one full streak. */
            const run = runStreak(ONE_STREAK,);

            expect(run.streakCounts,)
              .toEqual([
                1,
                2,
                3,
                4,
                5,
              ],);
            expect(run.notifications,)
              .toEqual([
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
            /** Whether any step notified. */
            const notifiedAny = run
              .notifications
              .some(function notified(flag,) {
                return flag;
              },);

            // The productive verdict leaves four unproductive verdicts in the window, so no later step
            // completes a streak.
            expect(notifiedAny,).toBe(false,);
            expect(run.streakCounts,)
              .toEqual([
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
            const verdicts: readonly Decision[] = [
              ...ONE_STREAK,
              UNPRODUCTIVE,
            ];
            /** The streak counts and notifications of the run. */
            const run = runStreak(verdicts,);

            // The fifth verdict notified and reset; the sixth is a streak of one, not a second notification.
            expect(run.streakCounts,)
              .toEqual([
                1,
                2,
                3,
                4,
                5,
                1,
              ],);
            expect(run.notifications,)
              .toEqual([
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
            const verdicts: readonly Decision[] = [
              ...ONE_STREAK,
              ...ONE_STREAK,
            ];
            /** The streak counts and notifications of the run. */
            const run = runStreak(verdicts,);
            /** The steps that notified. */
            const notifyingSteps = run
              .notifications
              .filter(function notified(flag,) {
                return flag;
              },);

            expect(notifyingSteps,).toHaveLength(2,);
          },
        },),
      ],
    },),
  ],
},);
