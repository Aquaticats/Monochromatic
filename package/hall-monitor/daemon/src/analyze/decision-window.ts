/**
 The productivity streak policy: which recent verdicts the daemon remembers,
 how many of them are unproductive, and when the user is notified.

 A sliding window of the newest verdicts decides the notification: once every
 verdict in it is unproductive the user is told, and the caller starts the
 window over so the same streak cannot notify twice. An off-by-one in the window length, in
 the shift, or in the reset changes when a user is interrupted, so the policy
 lives here as pure functions that a test can drive a whole streak through,
 apart from the capture, model and notification around it.

 @module
 */

//region Window shape

/**
 Possible productivity verdict values.
 */
export type Decision = 'PRODUCTIVE' | 'UNPRODUCTIVE';

/**
 Fixed-size tuple tracking the most recent verdicts, oldest first.

 The tuple and not an array, so the size is checked by the compiler rather
 than by a comment.
 */
export type DecisionWindow = readonly [
  Decision,
  Decision,
  Decision,
  Decision,
  Decision,
];

/**
 The window a fresh daemon starts from: every remembered cycle productive, so
 a run that begins unproductive still needs a full streak before it notifies.
 */
const INITIAL_WINDOW: DecisionWindow = [
  'PRODUCTIVE',
  'PRODUCTIVE',
  'PRODUCTIVE',
  'PRODUCTIVE',
  'PRODUCTIVE',
];

/**
 How many cycles the window covers, derived from the window itself so the
 count and the tuple cannot disagree.
 */
export const DECISION_WINDOW_SIZE: number = INITIAL_WINDOW.length;

/**
 Checks whether a decision is unproductive.
 
 @param decision - verdict to check
 
 @returns true when the verdict is UNPRODUCTIVE
 */
function isUnproductive(decision: Decision,): boolean {
  return decision === 'UNPRODUCTIVE';
}

/**
 The window a fresh daemon starts from, and the one a notification resets to.
 
 @returns a window whose every verdict is productive
 
 @example
 ```ts
 const window = initialDecisionWindow();
 ```
 */
export function initialDecisionWindow(): DecisionWindow {
  return INITIAL_WINDOW;
}

/**
 Shifts one verdict into the window, dropping the oldest.
 
 @param window - window before this cycle
 
 @param verdict - verdict this cycle recorded
 
 @returns the window holding the newest verdicts, oldest first
 
 @example
 ```ts
 shiftDecisionWindow({ window: initialDecisionWindow(), verdict: 'UNPRODUCTIVE' });
 // ['PRODUCTIVE', 'PRODUCTIVE', 'PRODUCTIVE', 'PRODUCTIVE', 'UNPRODUCTIVE']
 ```
 */
export function shiftDecisionWindow(
  {
    window,
    verdict,
  }: {
    readonly window: DecisionWindow;
    readonly verdict: Decision;
  },
): DecisionWindow {
  /**
   The four newest verdicts of the window, oldest first; the leading hole is
   the verdict that falls out.
   */
  const [
    ,
    second,
    third,
    fourth,
    fifth,
  ] = window;
  return [
    second,
    third,
    fourth,
    fifth,
    verdict,
  ];
}

//endregion Window shape

//region Recording a verdict

/**
 What one recorded verdict does to the window and the streak.
 */
export type StreakStep = {
  /**
   The window with the verdict shifted in, before any post-notification reset.
   */
  readonly window: DecisionWindow;
  /**
   How many verdicts in that window are unproductive.
   */
  readonly streakCount: number;
  /**
   Whether this verdict completed a window of uninterrupted unproductive
   cycles, which is when the user is notified.
   */
  readonly notified: boolean;
};

/**
 Records one verdict: shifts it into the window, counts the unproductive
 verdicts in it, and reports whether that completes the notification streak.
 
 The reset after a notification is the caller's step, on purpose: a caller
 that notifies a human keeps the window it just acted on until the
 notification has actually happened, so a failed notification can still retire
 the streak instead of losing it.
 
 @param window - window before this cycle
 
 @param verdict - verdict this cycle recorded
 
 @returns the shifted window, its unproductive count, and whether to notify
 
 @example
 ```ts
 const { window: nextWindow, streakCount, notified } = recordVerdict({ window, verdict });
 if (notified) { await sendNotification(text); }
 ```
 */
export function recordVerdict(
  {
    window,
    verdict,
  }: {
    readonly window: DecisionWindow;
    readonly verdict: Decision;
  },
): StreakStep {
  /**
   The window with this verdict shifted in.
   */
  const shifted = shiftDecisionWindow({
    window,
    verdict,
  },);
  /**
   Whether this cycle completes the streak.
   */
  const notified = everyDecisionUnproductive(shifted,);

  return {
    window: shifted,
    streakCount: countUnproductive(shifted,),
    notified,
  };
}

//endregion Recording a verdict

//region Streak inspection

/**
 Counts the unproductive verdicts in a window, which is the streak the daemon
 logs.
 
 @param window - window to count
 
 @returns how many of its verdicts are unproductive
 
 @example
 ```ts
 countUnproductive(initialDecisionWindow()); // 0
 ```
 */
export function countUnproductive(window: DecisionWindow,): number {
  return window
    .filter(function checkUnproductive(decision,) {
      return isUnproductive(decision,);
    },)
    .length;
}

/**
 Checks whether the whole window is unproductive, which is the notification
 trigger.
 
 @param window - window to check
 
 @returns true when every verdict in the window is unproductive
 
 @example
 ```ts
 everyDecisionUnproductive(window); // true once five cycles in a row were unproductive
 ```
 */
export function everyDecisionUnproductive(window: DecisionWindow,): boolean {
  return window
    .every(function checkUnproductive(decision,) {
      return isUnproductive(decision,);
    },);
}

//endregion Streak inspection
