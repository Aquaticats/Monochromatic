import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { advanceStreak, QUIET, } from './streak.ts';

await describe({
  name: advanceStreak.name,
  children: [
    it({
      name: 'raises an alert only on the third bad sample in a row',
      fn: async function alertsOnThird() {
        /**
         Result after the first bad sample.
         */
        const first = advanceStreak({ streak: QUIET, isBad: true, },);
        /**
         Result after the second bad sample.
         */
        const second = advanceStreak({ streak: first.streak, isBad: true, },);
        /**
         Result after the third bad sample.
         */
        const third = advanceStreak({ streak: second.streak, isBad: true, },);
        expect([first.event, second.event, third.event,],).toEqual(['none', 'none', 'alert',],);
        expect(third.streak.alerting,).toBe(true,);
      },
    },),
    it({
      name: 'a good sample between bad ones restarts the count',
      fn: async function goodResets() {
        /**
         Streak with two bad samples.
         */
        const twoBad = advanceStreak({ streak: advanceStreak({ streak: QUIET, isBad: true, },).streak, isBad: true, },).streak;
        /**
         Streak after a good sample interrupts.
         */
        const interrupted = advanceStreak({ streak: twoBad, isBad: false, },);
        expect(interrupted.streak.bad,).toBe(0,);
        expect(advanceStreak({ streak: interrupted.streak, isBad: true, },).event,).toBe('none',);
      },
    },),
    it({
      name: 'clears only after five good samples and keeps alerting until then',
      fn: async function clearsAfterFive() {
        /**
         Streak that is alerting with no good samples yet.
         */
        const alerting = { bad: 3, good: 0, alerting: true, };
        /**
         Events of five successive good samples.
         */
        const events = [1, 2, 3, 4, 5,].reduce(function step(
          acc: Readonly<{ streak: typeof alerting; events: readonly string[]; }>,
        ) {
          /**
           Result of one more good sample.
           */
          const next = advanceStreak({ streak: acc.streak, isBad: false, },);
          return { streak: next.streak, events: [...acc.events, next.event,], };
        }, { streak: alerting, events: [], },);
        expect(events.events,).toEqual(['none', 'none', 'none', 'none', 'clear',],);
        expect(events.streak.alerting,).toBe(false,);
      },
    },),
    it({
      name: 'a bad sample while alerting does not raise a second alert',
      fn: async function noSecondAlert() {
        expect(advanceStreak({ streak: { bad: 3, good: 0, alerting: true, }, isBad: true, },).event,).toBe('none',);
      },
    },),
  ],
},);
