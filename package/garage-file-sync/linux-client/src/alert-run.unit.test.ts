import { join, } from 'node:path';
import { mkdtempDisposable, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { JournalRead, } from './alert-journal.ts';
import type { Notification, } from './alert-message.ts';
import type { AlertRequest, UnitOutcome, } from './alert-outcome.ts';
import {
  type AlertDeps,
  runAlert,
} from './alert-run.ts';
import type { NotifyOutcome, } from './alert-notify.ts';
import type { Streak, StreakLookup, } from './alert-streak.ts';
import {
  SYNC_UNIT,
  WATCH_UNIT,
} from './units.ts';

/**
 One hour in milliseconds, the gap after which a still-failing unit alerts again.
 */
const HOUR_MS = 60 * 60 * 1_000;

/**
 A failed run as systemd reports it.
 */
const FAILED_RUN: UnitOutcome = {
  result: 'exit-code',
  exitCode: 'exited',
  exitStatus: '1',
};

/**
 A clean run as systemd reports it.
 */
const CLEAN_RUN: UnitOutcome = {
  result: 'success',
  exitCode: 'exited',
  exitStatus: '0',
};

/**
 A run stopped by hand as systemd reports it: a clean stop, not a clean exit.
 */
const STOPPED_RUN: UnitOutcome = {
  result: 'success',
  exitCode: 'killed',
  exitStatus: 'TERM',
};

/**
 What a fake run recorded, so a test can assert on effects instead of on stdout.
 */
type Recorded = {
  readonly notifications: Notification[];
  readonly stdout: string[];
  readonly stderr: string[];
  readonly journalReads: string[];
  streak?: Streak;
};

/**
 Builds the alert's effects as fakes, recording every call.

 @param options - streak to start with, journal lines to hand back, and whether
   notify-send succeeds

 @returns the deps to run the alert with, and the record of what it did
 */
function fakeDeps(options: Readonly<{
  streak?: Streak;
  journalLines?: readonly string[];
  journalProblem?: string;
  notifySucceeds?: boolean;
  nowMs?: number;
  home?: string;
}> = {},): { deps: AlertDeps; recorded: Recorded; } {
  /**
   Effects recorded for assertions.
   */
  const recorded: Recorded = {
    notifications: [],
    stdout: [],
    stderr: [],
    journalReads: [],
    ...(options.streak === undefined ? {} : { streak: options.streak, }),
  };
  /**
   The effects the alert runner calls.
   */
  const deps: AlertDeps = {
    home: options.home ?? '/home/example',
    readStreak: async function readUnitStreak(): Promise<StreakLookup> {
      return recorded.streak === undefined ? {} : { streak: recorded.streak, };
    },
    writeStreak: async function writeUnitStreak(streak,): Promise<void> {
      recorded.streak = streak;
    },
    removeStreak: async function removeUnitStreak(): Promise<void> {
      delete recorded.streak;
    },
    readJournal: async function readJournal(): Promise<JournalRead> {
      recorded.journalReads.push('read',);
      return options.journalProblem === undefined
        ? { lines: options.journalLines ?? ['ERROR : disk full',], }
        : { lines: [], problem: options.journalProblem, };
    },
    notify: async function notify(notification,): Promise<NotifyOutcome> {
      recorded.notifications.push(notification,);
      return options.notifySucceeds === false
        ? { sent: false, problem: 'notify-send exited 1: no server', }
        : { sent: true, };
    },
    nowMs: function nowMs(): number {
      return options.nowMs ?? (HOUR_MS * 10);
    },
    writeOut: function writeOut(text,): void {
      recorded.stdout.push(text,);
    },
    writeError: function writeError(text,): void {
      recorded.stderr.push(text,);
    },
  };
  return {
    deps,
    recorded,
  };
}

/**
 Builds one alert request.

 @param options - trigger, unit, outcome and invocation id to use

 @returns the request
 */
function request(options: Readonly<{
  trigger?: AlertRequest['trigger'];
  unit?: string;
  outcome?: UnitOutcome;
  invocationId?: string;
}> = {},): AlertRequest {
  return {
    trigger: options.trigger ?? 'unit',
    unit: options.unit ?? SYNC_UNIT,
    invocationId: options.invocationId ?? '',
    outcome: options.outcome ?? FAILED_RUN,
  };
}

await describe({
  name: runAlert.name,
  children: [
    //region Failure alerts
    it({
      name: 'a first failure alerts and records the streak',
      fn: async function firstFailureAlerts() {
        /**
         The run under test.
         */
        const { deps, recorded, } = fakeDeps({ nowMs: 5 * HOUR_MS, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request(), deps, },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.notifications[0]?.summary,).toBe('Garage sync failed',);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: 5 * HOUR_MS, },);
      },
    },),
    it({
      name: 'a failure inside the quiet hour after an alert stays silent and reads no journal',
      fn: async function repeatSuppressed() {
        /**
         The run under test, with an alert one minute ago.
         */
        const { deps, recorded, } = fakeDeps({
          streak: { lastAlertAtMs: HOUR_MS - 60_000, },
          nowMs: HOUR_MS,
        },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request(), deps, },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(0,);
        expect(recorded.journalReads,).toHaveLength(0,);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: HOUR_MS - 60_000, },);
      },
    },),
    it({
      name: 'a failure an hour or more after the last alert alerts again and moves the streak',
      fn: async function repeatAfterAnHour() {
        /**
         The run under test, with an alert exactly an hour ago.
         */
        const { deps, recorded, } = fakeDeps({
          streak: { lastAlertAtMs: HOUR_MS, },
          nowMs: 2 * HOUR_MS,
        },);
        await runAlert({ request: request(), deps, },);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: 2 * HOUR_MS, },);
      },
    },),
    it({
      name: 'a failed notification writes no state and exits non-zero',
      fn: async function notificationFailureKeepsTheStreak() {
        /**
         The run under test, with no notification server.
         */
        const { deps, recorded, } = fakeDeps({ notifySucceeds: false, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request(), deps, },);
        expect(status,).toBe(1,);
        expect(recorded.streak,).toBeUndefined();
        expect(recorded.stderr.join('\n',),).toContain('notify-send exited 1',);
      },
    },),
    //endregion Failure alerts

    //region Recovery alerts
    it({
      name: 'a run that exited with status 0 closes an open streak with one recovery alert',
      fn: async function cleanRunRecovers() {
        /**
         The run under test, with an open streak.
         */
        const { deps, recorded, } = fakeDeps({ streak: { lastAlertAtMs: HOUR_MS, }, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request({ outcome: CLEAN_RUN, }), deps, },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.notifications[0]?.summary,).toBe('Garage sync recovered',);
        expect(recorded.streak,).toBeUndefined();
      },
    },),
    it({
      name: 'a clean run with no open streak shows nothing, so the timer\'s every-five-minute success is free',
      fn: async function cleanRunWithoutStreakIsSilent() {
        /**
         The run under test, with no streak.
         */
        const { deps, recorded, } = fakeDeps({},);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request({ outcome: CLEAN_RUN, }), deps, },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(0,);
        expect(recorded.journalReads,).toHaveLength(0,);
      },
    },),
    it({
      name: 'a run stopped by hand is not a recovery and leaves the streak open',
      fn: async function stoppedRunIsNotRecovery() {
        /**
         The run under test, with an open streak.
         */
        const { deps, recorded, } = fakeDeps({ streak: { lastAlertAtMs: HOUR_MS, }, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request({ outcome: STOPPED_RUN, }), deps, },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(0,);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: HOUR_MS, },);
        expect(recorded.stdout.join('\n',),).toContain('was stopped',);
      },
    },),
    it({
      name: 'a success handler started with no systemd variables stays silent and leaves the streak open',
      fn: async function successHandlerWithoutVariables() {
        /**
         The run under test: the OnSuccess= handler, with an open streak and no MONITOR_* values.
         */
        const { deps, recorded, } = fakeDeps({ streak: { lastAlertAtMs: HOUR_MS, }, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({
          request: request({
            trigger: 'unit-succeeded',
            outcome: { result: '', exitCode: '', exitStatus: '', },
          },),
          deps,
        },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(0,);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: HOUR_MS, },);
      },
    },),
    it({
      name: 'a failure handler alerts even when systemd passed no variables',
      fn: async function failureHandlerWithoutVariables() {
        /**
         The run under test: the OnFailure= handler with no open streak and no MONITOR_* values.
         */
        const { deps, recorded, } = fakeDeps({ nowMs: 5 * HOUR_MS, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({
          request: request({
            trigger: 'unit-failed',
            outcome: { result: '', exitCode: '', exitStatus: '', },
          },),
          deps,
        },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.notifications[0]?.summary,).toBe('Garage sync failed',);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: 5 * HOUR_MS, },);
      },
    },),
    it({
      name: 'a recovery alert that cannot be shown keeps the streak open and exits non-zero',
      fn: async function recoveryNotificationFailure() {
        /**
         The run under test, with no notification server and an open streak.
         */
        const { deps, recorded, } = fakeDeps({
          streak: { lastAlertAtMs: HOUR_MS, },
          notifySucceeds: false,
        },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request({ outcome: CLEAN_RUN, }), deps, },);
        expect(status,).toBe(1,);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: HOUR_MS, },);
      },
    },),
    //endregion Recovery alerts

    //region Watcher modes
    it({
      name: 'a failed watcher hook is a failure of the watcher unit',
      fn: async function hookFailureAlertsForTheWatcher() {
        /**
         The run under test, for the watcher's own failed hook.
         */
        const { deps, recorded, } = fakeDeps({},);
        await runAlert({ request: request({ trigger: 'hook-failed', unit: WATCH_UNIT, outcome: { result: '', exitCode: '', exitStatus: '', }, }), deps, },);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.notifications[0]?.summary,).toBe(`Garage sync failed (${WATCH_UNIT})`,);
        expect(recorded.notifications[0]?.body,).toContain('rclone-bisync-garage-after-current.service',);
        expect(recorded.streak,).toBeDefined();
      },
    },),
    it({
      name: 'the watcher starting again reports its recovery once, and only for an open streak',
      fn: async function watcherStartedRecovers() {
        /**
         The run under test, with the watcher's streak open.
         */
        const openWatcherRun = fakeDeps({ streak: { lastAlertAtMs: 0, }, },);
        await runAlert({ request: request({ trigger: 'watcher-started', unit: WATCH_UNIT, outcome: { result: '', exitCode: '', exitStatus: '', }, }), deps: openWatcherRun.deps, },);
        expect(openWatcherRun.recorded.notifications,).toHaveLength(1,);
        expect(openWatcherRun.recorded.notifications[0]?.summary,).toBe(`Garage sync recovered (${WATCH_UNIT})`,);
        expect(openWatcherRun.recorded.streak,).toBeUndefined();
        /**
         The same start hook with no streak open.
         */
        const closedWatcherRun = fakeDeps({},);
        await runAlert({ request: request({ trigger: 'watcher-started', unit: WATCH_UNIT, outcome: { result: '', exitCode: '', exitStatus: '', }, }), deps: closedWatcherRun.deps, },);
        expect(closedWatcherRun.recorded.notifications,).toHaveLength(0,);
      },
    },),
    //endregion Watcher modes

    //region Journal
    it({
      name: 'the journal read is the failed run\'s own invocation, and a failed read still alerts',
      fn: async function failedJournalReadStillAlerts() {
        /**
         The run under test, whose journal cannot be read.
         */
        const { deps, recorded, } = fakeDeps({ journalProblem: 'journalctl exited 1: no journal', },);
        await runAlert({ request: request({ invocationId: 'abcdef0123456789', }), deps, },);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.stderr.join('\n',),).toContain('could not read the run\'s journal',);
        expect(recorded.notifications[0]?.body,).toContain('with no ERROR line in the log',);
      },
    },),
    //endregion Journal
  ],
},);

await describe({
  name: 'a disposable home',
  children: [
    it({
      name: 'keeps the alert streak in one file per unit under the state folder',
      fn: async function streakFilePerUnit() {
        await using home = await mkdtempDisposable(join(scratchRoot(), 'garage-alert-run-',),);
        /**
         The real streak functions, against the disposable home.
         */
        const streakModule = await import('./alert-streak.ts',);
        await streakModule.writeStreak({
          home: home.path,
          unit: SYNC_UNIT,
          streak: { lastAlertAtMs: HOUR_MS, },
        },);
        expect((await streakModule.readStreak({ home: home.path, unit: SYNC_UNIT, },)).streak,)
          .toEqual({ lastAlertAtMs: HOUR_MS, },);
        expect((await streakModule.readStreak({ home: home.path, unit: WATCH_UNIT, },)).streak,)
          .toBeUndefined();
        await streakModule.removeStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect((await streakModule.readStreak({ home: home.path, unit: SYNC_UNIT, },)).streak,)
          .toBeUndefined();
      },
    },),
  ],
},);
