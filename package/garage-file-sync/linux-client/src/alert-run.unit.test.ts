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
import {
  readStreak,
  removeStreak,
  type Streak,
  type StreakLookup,
  writeStreak,
} from './alert-streak.ts';
import type { BackupListings, } from './bisync-backup-listings.ts';
import {
  SYNC_UNIT,
  WATCH_UNIT,
} from './units.ts';

/**
 Milliseconds in one minute.
 */
const MINUTE_MS = 60 * 1_000;

/**
 One hour in milliseconds, the gap after which a still-failing unit alerts again.
 */
const HOUR_MS = 60 * MINUTE_MS;

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
 Hand stops as systemd reports them to the failure handler.

 The first is systemd's own report of the stopped oneshot (`Result=signal`, measured on
 this PC's user manager at 2026-11-25); the second is rclone after it caught SIGTERM,
 ran its exit handlers and exited with 128 + 15.
 */
const HAND_STOP_RUNS: readonly UnitOutcome[] = [
  { result: 'signal', exitCode: 'killed', exitStatus: 'TERM', },
  { result: 'exit-code', exitCode: 'exited', exitStatus: '143', },
];

/**
 What a fake run recorded, so a test can assert on effects instead of on stdout.
 */
type Recorded = {
  readonly notifications: Notification[];
  readonly stdout: string[];
  readonly stderr: string[];
  readonly journalReads: string[];
  readonly listingReads: string[];
  streak?: Streak;
};

/**
 Builds the alert's effects as fakes, recording every call.

 @param options - streak to start with, the reason its file could not be used, journal
   lines to hand back, and whether notify-send succeeds

 @returns the deps to run the alert with, and the record of what it did
 */
function fakeDeps(options: Readonly<{
  streak?: Streak;
  streakProblem?: string;
  journalLines?: readonly string[];
  journalProblem?: string;
  notifySucceeds?: boolean;
  nowMs?: number;
  home?: string;
  backupListingsExist?: boolean;
}> = {},): { deps: AlertDeps; recorded: Recorded; } {
  /**
   Effects recorded for assertions.
   */
  const recorded: Recorded = {
    notifications: [],
    stdout: [],
    stderr: [],
    journalReads: [],
    listingReads: [],
    ...(options.streak === undefined ? {} : { streak: options.streak, }),
  };
  /**
   The effects the alert runner calls.
   */
  const deps: AlertDeps = {
    home: options.home ?? '/home/example',
    readStreak: async function readUnitStreak(): Promise<StreakLookup> {
      if (options.streakProblem !== undefined) return { problem: options.streakProblem, };
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
    backupListingsExist: async function backupListingsExist(): Promise<BackupListings> {
      recorded.listingReads.push('read',);
      return { exist: options.backupListingsExist ?? true, };
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
          streak: { lastAlertAtMs: HOUR_MS - MINUTE_MS, },
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

    //region A state file that could not be used
    it({
      name: 'a state file that could not be used still alerts and logs the reason',
      fn: async function unusableStreakStillAlerts() {
        /**
         The reason the state file could not be used.
         */
        const reason = 'could not read /home/example/.local/state/rclone-bisync-garage-alert/rclone-bisync-garage.service.json: EACCES: permission denied';
        /**
         The run under test, whose streak file could not be used.
         */
        const { deps, recorded, } = fakeDeps({ streakProblem: reason, nowMs: 5 * HOUR_MS, },);
        /**
         Exit status of the entry.
         */
        const status = await runAlert({ request: request(), deps, },);
        expect(status,).toBe(0,);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.notifications[0]?.summary,).toBe('Garage sync failed',);
        expect(recorded.streak,).toEqual({ lastAlertAtMs: 5 * HOUR_MS, },);
        expect(recorded.stderr.join('\n',),).toContain(reason,);
        expect(recorded.stderr.join('\n',),).toContain('treating it as no open streak',);
      },
    },),
    //endregion A state file that could not be used

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
    ...HAND_STOP_RUNS.map(function handStopRun(outcome,) {
      return it({
        name: `a hand stop systemd reports as ${outcome.result} (${outcome.exitCode} status ${outcome.exitStatus}) stays silent and leaves the streak open`,
        fn: async function handStopIsNotAFailure() {
          /**
           The run under test, with an open streak and the hand stop systemd reported.
           */
          const { deps, recorded, } = fakeDeps({ streak: { lastAlertAtMs: HOUR_MS, }, },);
          /**
           Exit status of the entry.
           */
          const status = await runAlert({
            request: request({
              trigger: 'unit-failed',
              outcome,
            },),
            deps,
          },);
          expect(status,).toBe(0,);
          expect(recorded.notifications,).toHaveLength(0,);
          expect(recorded.streak,).toEqual({ lastAlertAtMs: HOUR_MS, },);
          expect(recorded.stdout.join('\n',),).toContain('was stopped',);
        },
      },);
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
      name: 'a failed watcher hook is a failure of the watcher unit, and reads no journal or listings',
      fn: async function hookFailureAlertsForTheWatcher() {
        /**
         The run under test, for the watcher's own failed hook.
         */
        const { deps, recorded, } = fakeDeps({},);
        await runAlert({ request: request({ trigger: 'hook-failed', unit: WATCH_UNIT, outcome: { result: '', exitCode: '', exitStatus: '', }, }), deps, },);
        expect(recorded.journalReads,).toHaveLength(0,);
        expect(recorded.listingReads,).toHaveLength(0,);
        expect(recorded.notifications,).toHaveLength(1,);
        expect(recorded.notifications[0]?.kind,).toBe('failure',);
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
        expect(openWatcherRun.recorded.notifications[0]?.kind,).toBe('recovery',);
        expect(openWatcherRun.recorded.streak,).toEqual({ lastAlertAtMs: 0, recovered: true, },);
        /**
         The same start hook with no streak open.
         */
        const closedWatcherRun = fakeDeps({},);
        await runAlert({ request: request({ trigger: 'watcher-started', unit: WATCH_UNIT, outcome: { result: '', exitCode: '', exitStatus: '', }, }), deps: closedWatcherRun.deps, },);
        expect(closedWatcherRun.recorded.notifications,).toHaveLength(0,);
        expect(closedWatcherRun.recorded.streak,).toBeUndefined();
      },
    },),
    it({
      name: 'a watcher crash loop sends one failure and one recovery inside the hour, then one more of each past it',
      fn: async function watcherCrashLoopKeepsTheHour() {
        await using home = await mkdtempDisposable(join(scratchRoot(), 'garage-alert-watch-loop-',),);
        /**
         Every notification the entry sent, in order.
         */
        const notifications: Notification[] = [];
        /**
         Clock the scenario moves between events; an object rather than a local, so no
         binding changes value.
         */
        const clock = { nowMs: 0, };
        /**
         The watcher's streak file in the disposable home.
         */
        async function watchStreak(): Promise<StreakLookup> {
          return await readStreak({
            home: home.path,
            unit: WATCH_UNIT,
          },);
        }
        /**
         The real effects, minus the notification server and the journal.
         */
        const deps: AlertDeps = {
          home: home.path,
          readStreak: watchStreak,
          writeStreak: async function writeWatchStreak(streak,) {
            await writeStreak({
              home: home.path,
              unit: WATCH_UNIT,
              streak,
            },);
          },
          removeStreak: async function removeWatchStreak() {
            await removeStreak({
              home: home.path,
              unit: WATCH_UNIT,
            },);
          },
          readJournal: async function readNoJournal() {
            return { lines: [], };
          },
          backupListingsExist: async function noListings() {
            return { exist: false, };
          },
          notify: async function collect(notification,) {
            notifications.push(notification,);
            return { sent: true, };
          },
          nowMs: function now() {
            return clock.nowMs;
          },
          writeOut: function writeNothing() {},
          writeError: function refuseDiagnostic(text,) {
            throw new Error(`the entry wrote a diagnostic the scenario did not expect: ${text}`,);
          },
        };
        /**
         Runs one watcher event to its end.
         */
        async function runWatcherEvent(trigger: AlertRequest['trigger'],): Promise<void> {
          await runAlert({
            request: request({
              trigger,
              unit: WATCH_UNIT,
              outcome: { result: '', exitCode: '', exitStatus: '', },
            },),
            deps,
          },);
        }
        //region The crash loop inside the hour
        clock.nowMs = 0;
        await runWatcherEvent('unit-failed',);
        expect(notifications.map(function kind(notification,) {
          return notification.kind;
        },),).toEqual(['failure',],);
        expect((await watchStreak()).streak,).toEqual({ lastAlertAtMs: 0, },);
        clock.nowMs = MINUTE_MS;
        await runWatcherEvent('watcher-started',);
        expect(notifications.map(function kind(notification,) {
          return notification.kind;
        },),).toEqual(['failure', 'recovery',],);
        expect((await watchStreak()).streak,).toEqual({ lastAlertAtMs: 0, recovered: true, },);
        clock.nowMs = 2 * MINUTE_MS;
        await runWatcherEvent('unit-failed',);
        clock.nowMs = 3 * MINUTE_MS;
        await runWatcherEvent('watcher-started',);
        clock.nowMs = 4 * MINUTE_MS;
        await runWatcherEvent('unit-failed',);
        expect(notifications.map(function kind(notification,) {
          return notification.kind;
        },),).toEqual(['failure', 'recovery',],);
        expect((await watchStreak()).streak,).toEqual({ lastAlertAtMs: 0, recovered: true, },);
        //endregion The crash loop inside the hour

        //region A failure past the hour
        clock.nowMs = HOUR_MS + MINUTE_MS;
        await runWatcherEvent('unit-failed',);
        expect(notifications.map(function kind(notification,) {
          return notification.kind;
        },),).toEqual(['failure', 'recovery', 'failure',],);
        expect((await watchStreak()).streak,).toEqual({ lastAlertAtMs: HOUR_MS + MINUTE_MS, },);
        clock.nowMs = HOUR_MS + (2 * MINUTE_MS);
        await runWatcherEvent('watcher-started',);
        expect(notifications.map(function kind(notification,) {
          return notification.kind;
        },),).toEqual(['failure', 'recovery', 'failure', 'recovery',],);
        expect((await watchStreak()).streak,).toEqual({
          lastAlertAtMs: HOUR_MS + MINUTE_MS,
          recovered: true,
        },);
        //endregion A failure past the hour
      },
      timeout: 30_000,
    },),
    //endregion Watcher modes

    //region Journal
    it({
      name: 'a failed journal read still alerts, names the failure, and logs the reason',
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
