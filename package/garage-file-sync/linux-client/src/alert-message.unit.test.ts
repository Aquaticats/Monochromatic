import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  causeLines,
  errorLines,
  journalArguments,
  stripAnsi,
  tailLines,
} from './alert-journal.ts';
import {
  composeMessage,
  describeOutcome,
} from './alert-message.ts';
import {
  ALERT_TRIGGERS,
  classifyOutcome,
  classifyRequest,
  parseAlertArguments,
  reportsRecovery,
  resolveAlertRequest,
} from './alert-outcome.ts';
import {
  SYNC_UNIT,
  WATCH_UNIT,
} from './units.ts';

/**
 A failed run as systemd reports it.
 */
const FAILED_RUN = {
  result: 'exit-code',
  exitCode: 'exited',
  exitStatus: '1',
};

/**
 A run that exited with status 0 as systemd reports it.
 */
const CLEAN_EXIT = {
  result: 'success',
  exitCode: 'exited',
  exitStatus: '0',
};

/**
 A run stopped by hand as systemd reports it: a clean stop, not a clean exit.
 */
const STOPPED_BY_HAND = {
  result: 'success',
  exitCode: 'killed',
  exitStatus: 'TERM',
};

/**
 What the entry reads when systemd passes no `MONITOR_*` variables, as it does for a
 handler one unit names in both `OnFailure=` and `OnSuccess=`.
 */
const NO_VARIABLES = {
  result: '',
  exitCode: '',
  exitStatus: '',
};

/**
 Home folder the messages shorten paths against.
 */
const HOME = '/home/example';

await describe({
  name: 'alert messages and their inputs',
  children: [
    //region Outcome classification
    describe({
      name: classifyOutcome.name,
      children: [
        it({
          name: 'calls only a run that exited with status 0 a recovery',
          fn: async function onlyExitedZero() {
            expect(classifyOutcome({ result: 'success', exitCode: 'exited', exitStatus: '0', },),).toBe('recovery',);
            expect(classifyOutcome({ result: 'success', exitCode: 'killed', exitStatus: 'TERM', },),).toBe('stopped',);
            expect(classifyOutcome({ result: 'success', exitCode: 'killed', exitStatus: 'INT', },),).toBe('stopped',);
            expect(classifyOutcome({ result: 'exit-code', exitCode: 'exited', exitStatus: '1', },),).toBe('failure',);
            expect(classifyOutcome({ result: 'timeout', exitCode: 'killed', exitStatus: 'TERM', },),).toBe('failure',);
            expect(classifyOutcome({ result: 'start-limit-hit', exitCode: '', exitStatus: '', },),).toBe('failure',);
            expect(classifyOutcome({ result: '', exitCode: '', exitStatus: '', },),).toBe('failure',);
          },
        },),
      ],
    },),
    describe({
      name: describeOutcome.name,
      children: [
        it({
          name: 'names the result and its exit detail, and says so when systemd passed none',
          fn: async function namesTheResult() {
            expect(describeOutcome(FAILED_RUN,),).toBe('exit-code (exited status 1)',);
            expect(describeOutcome({ result: 'timeout', exitCode: 'killed', exitStatus: 'TERM', },),).toBe('timeout (killed status TERM)',);
            expect(describeOutcome({ result: '', exitCode: '', exitStatus: '', },),).toContain('unknown result',);
          },
        },),
      ],
    },),
    //endregion Outcome classification

    //region Command line
    describe({
      name: parseAlertArguments.name,
      children: [
        it({
          name: 'reads the unit the unit file passes and the trigger a hook names',
          fn: async function readsBoth() {
            expect(parseAlertArguments([],),).toEqual({ trigger: 'unit', },);
            expect(parseAlertArguments([SYNC_UNIT,],),).toEqual({ trigger: 'unit', unit: SYNC_UNIT, },);
            expect(parseAlertArguments([WATCH_UNIT, '--trigger=hook-failed',],),)
              .toEqual({ trigger: 'hook-failed', unit: WATCH_UNIT, },);
            for (const trigger of ALERT_TRIGGERS) {
              expect(parseAlertArguments([SYNC_UNIT, `--trigger=${trigger}`,],),)
                .toEqual({ trigger, unit: SYNC_UNIT, },);
            }
          },
        },),
        it({
          name: 'refuses an unknown trigger, an unknown option and a second unit name',
          fn: async function refusesBadInput() {
            expect(() => parseAlertArguments(['--trigger=later',],),).toThrow('--trigger must be one of',);
            expect(() => parseAlertArguments(['--force',],),).toThrow('unknown option',);
            expect(() => parseAlertArguments(['a.service', 'b.service',],),).toThrow('only one unit name',);
          },
        },),
      ],
    },),
    describe({
      name: classifyRequest.name,
      children: [
        it({
          name: 'a failure handler always reports a failure, even when systemd passed no variables',
          fn: async function failureHandler() {
            expect(classifyRequest({ trigger: 'unit-failed', outcome: FAILED_RUN, },),).toBe('failure',);
            expect(classifyRequest({ trigger: 'unit-failed', outcome: NO_VARIABLES, },),).toBe('failure',);
          },
        },),
        it({
          name: 'a success handler is a recovery only for exit status 0, and a stop otherwise, never a failure',
          fn: async function successHandler() {
            expect(classifyRequest({ trigger: 'unit-succeeded', outcome: CLEAN_EXIT, },),).toBe('recovery',);
            expect(classifyRequest({ trigger: 'unit-succeeded', outcome: STOPPED_BY_HAND, },),).toBe('stopped',);
            expect(classifyRequest({ trigger: 'unit-succeeded', outcome: NO_VARIABLES, },),).toBe('stopped',);
          },
        },),
        it({
          name: 'the hook modes are a failure and a recovery, and a hand-started handler follows the variables',
          fn: async function otherTriggers() {
            expect(classifyRequest({ trigger: 'hook-failed', outcome: NO_VARIABLES, },),).toBe('failure',);
            expect(classifyRequest({ trigger: 'watcher-started', outcome: NO_VARIABLES, },),).toBe('recovery',);
            expect(classifyRequest({ trigger: 'unit', outcome: CLEAN_EXIT, },),).toBe('recovery',);
            expect(classifyRequest({ trigger: 'unit', outcome: NO_VARIABLES, },),).toBe('failure',);
          },
        },),
      ],
    },),
    describe({
      name: reportsRecovery.name,
      children: [
        it({
          name: 'only the watcher starting, a success handler and a hand-started successful run report a recovery',
          fn: async function recoveryWording() {
            expect(reportsRecovery({ trigger: 'watcher-started', outcome: NO_VARIABLES, },),).toBe(true,);
            expect(reportsRecovery({ trigger: 'unit-succeeded', outcome: CLEAN_EXIT, },),).toBe(true,);
            expect(reportsRecovery({ trigger: 'unit', outcome: CLEAN_EXIT, },),).toBe(true,);
            expect(reportsRecovery({ trigger: 'unit', outcome: FAILED_RUN, },),).toBe(false,);
            expect(reportsRecovery({ trigger: 'unit-failed', outcome: CLEAN_EXIT, },),).toBe(false,);
            expect(reportsRecovery({ trigger: 'hook-failed', outcome: NO_VARIABLES, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: resolveAlertRequest.name,
      children: [
        it({
          name: 'takes the unit systemd names before the positional one, and the latest invocation when none is given',
          fn: async function systemdFirst() {
            /**
             Request for a run systemd reported.
             */
            const fromSystemd = resolveAlertRequest({
              argv: [WATCH_UNIT,],
              env: {
                MONITOR_UNIT: SYNC_UNIT,
                MONITOR_INVOCATION_ID: 'abc',
                MONITOR_SERVICE_RESULT: 'exit-code',
                MONITOR_EXIT_CODE: 'exited',
                MONITOR_EXIT_STATUS: '1',
              },
            },);
            expect(fromSystemd.unit,).toBe(SYNC_UNIT,);
            expect(fromSystemd.invocationId,).toBe('abc',);
            expect(fromSystemd.outcome.result,).toBe('exit-code',);
            /**
             Request started by hand.
             */
            const byHand = resolveAlertRequest({ argv: [], env: {}, },);
            expect(byHand.unit,).toBe(SYNC_UNIT,);
            expect(byHand.invocationId,).toBe('',);
            expect(classifyOutcome(byHand.outcome,),).toBe('failure',);
            /**
             watcher hook, which always describes the watcher unit.
             */
            const hook = resolveAlertRequest({ argv: ['--trigger=hook-failed',], env: {}, },);
            expect(hook.unit,).toBe(WATCH_UNIT,);
          },
        },),
      ],
    },),
    //endregion Command line

    //region Journal
    describe({
      name: journalArguments.name,
      children: [
        it({
          name: 'asks for the run\'s own invocation and never for a look-back window',
          fn: async function ownInvocationOnly() {
            /**
             Arguments for a run with a known invocation id.
             */
            const withId = journalArguments({ unit: SYNC_UNIT, invocationId: 'abc123', },);
            expect(withId,).toContain('--invocation=abc123',);
            expect(withId.some((argument,) => argument.startsWith('--since',),),).toBe(false,);
            /**
             Arguments when systemd passed no id: the unit's latest invocation.
             */
            const latest = journalArguments({ unit: SYNC_UNIT, invocationId: '', },);
            expect(latest,).toContain('--invocation=0',);
          },
        },),
      ],
    },),
    describe({
      name: stripAnsi.name,
      children: [
        it({
          name: 'removes colour sequences and keeps every other character',
          fn: async function removesColour() {
            expect(stripAnsi('\u001B[0;32mBisync successful\u001B[0m',),).toBe('Bisync successful',);
            expect(stripAnsi('plain text',),).toBe('plain text',);
            expect(stripAnsi('keep \u001B[brackets',),).toBe('keep \u001B[brackets',);
          },
        },),
      ],
    },),
    describe({
      name: errorLines.name,
      children: [
        it({
          name: 'keeps the text after the ERROR marker only',
          fn: async function onlyErrorLines() {
            expect(errorLines(['ERROR : disk full', 'Bisync successful', 'ERROR: other',],),)
              .toEqual(['disk full', 'other',],);
            expect(errorLines(['ERROR no colon',],),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: causeLines.name,
      children: [
        it({
          name: 'turns a per-file failure into path and reason',
          fn: async function perFileFailure() {
            expect(causeLines({
              errors: ['Plain/main.kdbx: Failed to copy: open: permission denied',],
              localRoot: `${HOME}/Plain/`,
            },),).toEqual(['Plain/main.kdbx: permission denied',],);
          },
        },),
        it({
          name: 'falls back to the critical error with the local root removed, and skips lines naming no cause',
          fn: async function criticalFallback() {
            expect(causeLines({
              errors: [
                'Bisync aborted. Must run --resync to recover.',
                `Bisync critical error: failed to open ${HOME}/Plain/x`,
              ],
              localRoot: `${HOME}/Plain/`,
            },),).toEqual(['failed to open x',],);
            expect(causeLines({
              errors: ['not deleting files as there were IO errors',],
              localRoot: `${HOME}/Plain/`,
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: tailLines.name,
      children: [
        it({
          name: 'keeps the last non-empty lines, oldest first',
          fn: async function lastLines() {
            expect(tailLines({ lines: ['a', '', 'b', 'c',], count: 2, },),).toEqual(['b', 'c',],);
          },
        },),
      ],
    },),
    //endregion Journal

    //region Message text
    describe({
      name: composeMessage.name,
      children: [
        it({
          name: 'a failed sync run names the cause, never an hourly run, and says where the log is',
          fn: async function failedSyncText() {
            /**
             The message under test.
             */
            const message = composeMessage({
              trigger: 'unit',
              unit: SYNC_UNIT,
              outcome: FAILED_RUN,
              journalLines: [
                'ERROR : Plain/main.kdbx: Failed to copy: open: permission denied',
                'ERROR : Bisync aborted.',
              ],
              home: HOME,
            },);
            expect(message.summary,).toBe('Garage sync failed',);
            expect(message.body,).toContain('Plain/main.kdbx: permission denied',);
            expect(message.body,).not.toContain('hourly',);
            expect(message.body,).toContain('the next run should recover by itself',);
            expect(message.body,).toContain(`Log: journalctl --user -u ${SYNC_UNIT} --since today`,);
          },
        },),
        it({
          name: 'a timeout says the run was stopped, even with no error line',
          fn: async function timeoutText() {
            /**
             The message under test.
             */
            const message = composeMessage({
              trigger: 'unit',
              unit: SYNC_UNIT,
              outcome: { result: 'timeout', exitCode: 'killed', exitStatus: 'TERM', },
              journalLines: [],
              home: HOME,
            },);
            expect(message.body,).toContain('hit its time limit',);
            expect(message.body,).not.toContain('with no ERROR line',);
          },
        },),
        it({
          name: 'a watcher failure quotes the run\'s last lines instead of bisync advice',
          fn: async function watcherFailureText() {
            /**
             The message under test.
             */
            const message = composeMessage({
              trigger: 'unit',
              unit: WATCH_UNIT,
              outcome: FAILED_RUN,
              journalLines: ['panic: watch limit reached', 'goroutine 1 [running]:',],
              home: HOME,
            },);
            expect(message.summary,).toBe(`Garage sync failed (${WATCH_UNIT})`,);
            expect(message.body,).toContain('panic: watch limit reached',);
            expect(message.body,).not.toContain('Other files keep syncing',);
          },
        },),
        it({
          name: 'a hook failure and a watcher recovery each say what happened in plain words',
          fn: async function hookAndWatcherText() {
            /**
             Message for a hook that could not start the trigger.
             */
            const hook = composeMessage({
              trigger: 'hook-failed',
              unit: WATCH_UNIT,
              outcome: { result: '', exitCode: '', exitStatus: '', },
              journalLines: [],
              home: HOME,
            },);
            expect(hook.body,).toContain('could not start rclone-bisync-garage-after-current.service',);
            expect(hook.body,).toContain('five-minute timer',);
            /**
             Message for the watcher starting again.
             */
            const watcher = composeMessage({
              trigger: 'watcher-started',
              unit: WATCH_UNIT,
              outcome: { result: '', exitCode: '', exitStatus: '', },
              journalLines: [],
              home: HOME,
            },);
            expect(watcher.summary,).toBe(`Garage sync recovered (${WATCH_UNIT})`,);
            expect(watcher.body,).toContain('watcher is running again',);
            expect(watcher.body,).not.toContain('hourly',);
          },
        },),
        it({
          name: 'a clean sync run is reported as a recovery with the streak closed',
          fn: async function recoveryText() {
            /**
             The message under test.
             */
            const message = composeMessage({
              trigger: 'unit',
              unit: SYNC_UNIT,
              outcome: { result: 'success', exitCode: 'exited', exitStatus: '0', },
              journalLines: [],
              home: HOME,
            },);
            expect(message.summary,).toBe('Garage sync recovered',);
            expect(message.body,).toContain('exit status 0',);
          },
        },),
      ],
    },),
    //endregion Message text
  ],
},);
