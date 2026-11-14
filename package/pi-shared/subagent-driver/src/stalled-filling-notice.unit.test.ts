/**
 The notice a filling owes when it has started nothing for longer than a bound while entries stand and places are free: one at the
 bound and not one a round, and none at all while no entry stands or no place is free. Also the desktop notification itself, which is
 `notify-send` on the caller's own bus address, bounded, and never thrown.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type DesktopNotice,
  DESKTOP_NOTICE_APP_NAME,
  type DesktopNoticeOutcome,
  noticeIfTheFillingStalls,
  noticeOnTheDesktop,
  type StalledFillingLook,
  stalledFillingNoticeOf,
  STALLED_FILLING_BOUND_MILLISECONDS,
} from '../dist/final/node/index.mjs';

/**
 A look at a filling that has started nothing for the length given, with the entries and places given.

 @param request - how long nothing has started, and what stands and is free

 @returns the look, its notices and the lines it reported
 */
function aLookAt({
  idleMilliseconds,
  entriesStanding = 7,
  placesFree = 3,
  lastNoticedAt,
  outcome = { shown: true, },
}: {
  readonly idleMilliseconds: number;
  readonly entriesStanding?: number;
  readonly placesFree?: number;
  readonly lastNoticedAt?: number;
  readonly outcome?: DesktopNoticeOutcome;
},): StalledFillingLook & {
  readonly notified: DesktopNotice[];
  readonly said: string[];
} {
  /**
   The notices the look showed.
   */
  const notified: DesktopNotice[] = [];
  /**
   The lines the look reported.
   */
  const said: string[] = [];
  /**
   When the look reads the clock: the bound after the last start, plus the length given.
   */
  const now = 1_000_000_000 + STALLED_FILLING_BOUND_MILLISECONDS + idleMilliseconds;
  return {
    lastStartedAt: 1_000_000_000,
    now,
    entriesStanding,
    placesFree,
    queueFile: '/runs/queue.json',
    stoppedBecause: 'no entry of the 7 left may start now: every pool one of them asks for is full',
    ...(lastNoticedAt === undefined ? {} : { lastNoticedAt, }),
    notify: function notify(notice,) {
      notified.push(notice,);
      return Promise.resolve(outcome,);
    },
    report: function report(line,) {
      said.push(line,);
    },
    notified,
    said,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: stalledFillingNoticeOf.name,
      children: [
        it({
          name: 'names how many entries stand, how many places are free, when something last started and why the last pass stopped',
          fn: async () => {
            /**
             What the notice says.
             */
            const notice = stalledFillingNoticeOf({ look: aLookAt({ idleMilliseconds: 0, },), },);
            expect(notice.title,).toBe('Pi runs are not being started',);
            expect(notice.body,).toContain('7 entries stand and 3 places are free',);
            expect(notice.body,).toContain('/runs/queue.json',);
            expect(notice.body,).toContain('every pool one of them asks for is full',);
          },
        },),
      ],
    },),
    describe({
      name: noticeIfTheFillingStalls.name,
      children: [
        it({
          name: 'says nothing before the bound, and one notice at it',
          fn: async () => {
            /**
             The look one millisecond before the bound.
             */
            const early = aLookAt({ idleMilliseconds: -1, },);
            expect((await noticeIfTheFillingStalls({ look: early, },)).noticed,).toBe(false,);
            expect(early.notified,).toEqual([],);
            /**
             The look at the bound.
             */
            const atTheBound = aLookAt({ idleMilliseconds: 0, },);
            expect((await noticeIfTheFillingStalls({ look: atTheBound, },)).noticed,).toBe(true,);
            expect(atTheBound.notified,).toHaveLength(1,);
            expect(atTheBound.said.join('',),).toContain('a desktop notification was shown',);
          },
        },),
        it({
          name: 'says nothing a round after the notice, and says it again a bound after the last one',
          fn: async () => {
            /**
             What the first notice answered.
             */
            const first = await noticeIfTheFillingStalls({ look: aLookAt({ idleMilliseconds: 0, },), },);
            expect(first.noticed,).toBe(true,);
            /**
             The look a round later.
             */
            const aRoundLater = aLookAt({
              idleMilliseconds: 150_000,
              ...(first.lastNoticedAt === undefined ? {} : { lastNoticedAt: first.lastNoticedAt, }),
            },);
            expect((await noticeIfTheFillingStalls({ look: aRoundLater, },)).noticed,).toBe(false,);
            expect(aRoundLater.notified,).toEqual([],);
            /**
             The look a bound after the last notice.
             */
            const aBoundLater = aLookAt({
              idleMilliseconds: STALLED_FILLING_BOUND_MILLISECONDS,
              ...(first.lastNoticedAt === undefined ? {} : { lastNoticedAt: first.lastNoticedAt, }),
            },);
            expect((await noticeIfTheFillingStalls({ look: aBoundLater, },)).noticed,).toBe(true,);
            expect(aBoundLater.notified,).toHaveLength(1,);
          },
        },),
        it({
          name: 'says nothing while no entry stands or no place is free, which is not a stall',
          fn: async () => {
            expect((await noticeIfTheFillingStalls({ look: aLookAt({
              idleMilliseconds: 0,
              entriesStanding: 0,
            },), },)).noticed,).toBe(false,);
            expect((await noticeIfTheFillingStalls({ look: aLookAt({
              idleMilliseconds: 0,
              placesFree: 0,
            },), },)).noticed,).toBe(false,);
          },
        },),
        it({
          name: 'still reports the stall when the desktop refused the notice',
          fn: async () => {
            /**
             The look whose desktop refused.
             */
            const refused = aLookAt({
              idleMilliseconds: 0,
              outcome: {
                shown: false,
                problem: 'no notification service',
              },
            },);
            expect((await noticeIfTheFillingStalls({ look: refused, },)).noticed,).toBe(true,);
            expect(refused.said.join('',),).toContain('the desktop notification was not shown: no notification service',);
          },
        },),
      ],
    },),
    describe({
      name: noticeOnTheDesktop.name,
      children: [
        it({
          name: 'asks notify-send for the notice on the caller\'s own bus, with the app name this filling shows',
          fn: async () => {
            /**
             What the command was asked.
             */
            const asked: { command: string; words: readonly string[] }[] = [];
            expect(await noticeOnTheDesktop({
              notice: {
                title: 'Pi runs are not being started',
                body: '7 entries stand',
              },
              run: function run({ command, words, },) {
                asked.push({
                  command,
                  words,
                },);
                return Promise.resolve({
                  status: 0,
                  stdout: '',
                  stderr: '',
                },);
              },
            },),).toEqual({ shown: true, },);
            expect(asked[0]?.command,).toBe('notify-send',);
            expect(asked[0]?.words,).toEqual([
              `--app-name=${DESKTOP_NOTICE_APP_NAME}`,
              '--urgency=critical',
              'Pi runs are not being started',
              '7 entries stand',
            ],);
          },
        },),
        it({
          name: 'answers what the executable said when the desktop refused, and never throws',
          fn: async () => {
            expect(await noticeOnTheDesktop({
              notice: {
                title: 'x',
                body: 'y',
              },
              run: function run() {
                return Promise.resolve({
                  status: 1,
                  stdout: '',
                  stderr: 'Failed to show notification: no service\n',
                },);
              },
            },),).toEqual({
              shown: false,
              problem: 'Failed to show notification: no service',
            },);
          },
        },),
      ],
    },),
  ],
},);
