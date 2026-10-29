import {
  BAD_AFTER_SAMPLES,
  CLEAR_AFTER_SAMPLES,
} from './constants.ts';
import type {
  Metric,
  Streak,
  StreakEvent,
} from './types.ts';

/**
 Streak of a metric that has never been bad.
 */
export const QUIET: Streak = {
  bad: 0,
  good: 0,
  alerting: false,
};

/**
 Every metric starting quiet.
 */
export const INITIAL_STREAKS: Readonly<Record<Metric, Streak>> = {
  'cpu-pressure-some-avg10': QUIET,
  'memory-pressure-some-avg10': QUIET,
  'cpu-thermal-throttle-mhz': QUIET,
  'scheduler-lag-ms': QUIET,
};

/**
 Advances a metric's streak by one sample: an alert needs {@link BAD_AFTER_SAMPLES} bad samples in a row,
 and clearing needs {@link CLEAR_AFTER_SAMPLES} good ones, so a single spike or dip changes nothing.

 
 @param streak - the metric's streak before this sample
 
 @param isBad - whether this sample is at or above the limit

 @returns the new streak and the event this sample caused

 @example
 ```ts
 advanceStreak({ streak: QUIET, isBad: true, },).event; // 'none' until the third bad sample
 ```
 */
export function advanceStreak({
  streak,
  isBad,
}: Readonly<{
  streak: Streak;
  isBad: boolean;
}>,): Readonly<{
  streak: Streak;
  event: StreakEvent
}> {
  /**
   Consecutive bad samples including this one, 0 after a good sample.
   */
  const bad = isBad ? streak.bad + 1 : 0;
  /**
   Consecutive good samples including this one, 0 after a bad sample.
   */
  const good = isBad ? 0 : streak.good + 1;
  if ((!streak.alerting) && (bad >= BAD_AFTER_SAMPLES)) {
    return {
      streak: {
        bad,
        good,
        alerting: true,
      },
      event: 'alert',
    };
  }
  if (streak.alerting && (good >= CLEAR_AFTER_SAMPLES)) {
    return {
      streak: {
        bad,
        good,
        alerting: false,
      },
      event: 'clear',
    };
  }
  return {
    streak: {
      bad,
      good,
      alerting: streak.alerting,
    },
    event: 'none',
  };
}
