/**
 Turns systemd's notification of a finished run into the alert's inputs.

 systemd sets `MONITOR_SERVICE_RESULT`, `MONITOR_EXIT_CODE`, `MONITOR_EXIT_STATUS`,
 `MONITOR_INVOCATION_ID` and `MONITOR_UNIT` in a handler started through
 `OnFailure=` or `OnSuccess=` (systemd.exec(5), "$MONITOR_* usage", systemd 262).
 Those variables are the only way to tell a run that exited with status 0 from a run
 stopped by hand: systemd reports a hand stop of a running oneshot as `Result=signal`,
 so `OnFailure=` handles it, and rclone that catches SIGTERM exits with status 143.

 @module
 */

import {
  SYNC_UNIT,
  WATCH_UNIT,
} from './units.ts';

/**
 What started the alert entry.

 `unit-failed` is a real run reported by systemd's `OnFailure=` (the template
 `rclone-bisync-garage-alert@.service`), `unit-succeeded` one reported by `OnSuccess=`
 (the template `rclone-bisync-garage-recovered@.service`), and `unit` a handler started by
 hand, classified from the `MONITOR_*` variables alone.
 The two templates are separate because systemd 262 passes no `MONITOR_*` variables to a
 handler that one unit names in both `OnFailure=` and `OnSuccess=`: it logs
 "multiple trigger source candidates for exit status propagation" and skips them
 (measured on this PC's user manager, 2026-11-25), so the trigger is the only reliable
 way to know which dependency started the entry;
 `hook-failed` is the watcher's change hook failing to start the trigger unit;
 `watcher-started` is the watcher's own start hook, which reports a watcher outage
 whose alert streak is open.
 */
export type AlertTrigger =
  | 'unit'
    | 'unit-failed'
    | 'unit-succeeded'
    | 'hook-failed'
    | 'watcher-started';

/**
 Every trigger the command line accepts, in the order the usage text lists them.
 */
export const ALERT_TRIGGERS: readonly AlertTrigger[] = [
  'unit',
  'unit-failed',
  'unit-succeeded',
  'hook-failed',
  'watcher-started',
] as const;

/**
 Tells whether a command-line value names a trigger.

 @param value - text written after `--trigger=`

 @returns true when the value is one of `ALERT_TRIGGERS`
 */
function isAlertTrigger(value: string,): value is AlertTrigger {
  return (ALERT_TRIGGERS as readonly string[]).includes(value,);
}

/**
 Tells whether a trigger reports a run of a unit, so the unit systemd names is the one
 the alert describes.

 @param trigger - what started the entry

 @returns true for `unit`, `unit-failed` and `unit-succeeded`

 @example
 ```ts
 isUnitTrigger('unit-succeeded',); // true
 isUnitTrigger('hook-failed',); // false
 ```
 */
export function isUnitTrigger(trigger: AlertTrigger,): boolean {
  return (trigger === 'unit') || (trigger === 'unit-failed')
    || (trigger === 'unit-succeeded');
}

/**
 How a unit's last run ended, as systemd reports it.
 */
export type UnitOutcome = {
  readonly result: string;
  readonly exitCode: string;
  readonly exitStatus: string;
};

/**
 What the outcome means for the alert.

 `recovery` is a run that exited with status 0; `stopped` is a run a SIGTERM stop ended,
 whether systemd reported the signal itself or rclone's own exit after catching it;
 `failure` is every other run.
 */
export type OutcomeClass =
  | 'failure'
    | 'recovery'
    | 'stopped';

/**
 Exit status rclone reports after its SIGTERM handler ran: 128 + 15.

 `lib/atexit/atexit_unix.go` computes 128 plus the signal number and
 `lib/atexit/atexit.go` exits with it once the registered handlers ran, at rclone
 `v1.76.0`.
 */
const RCLONE_SIGTERM_EXIT_STATUS = '143';

/**
 Tells whether systemd reported a hand stop of a run.

 A hand stop reaches `OnFailure=` in one of two shapes: systemd reports the stopped
 oneshot itself as `Result=signal` with the killed TERM detail (measured on this PC's
 user manager, 2026-11-25), or rclone catches SIGTERM, runs its exit handlers and exits
 143, which systemd reports as `Result=exit-code` with exit status 143.

 @param outcome - result, exit code and exit status systemd reported

 @returns true for exactly those two shapes, never for a timeout or another failure
 */
function isHandStopped(outcome: Readonly<UnitOutcome>,): boolean {
  if (outcome.exitStatus === 'TERM') {
    return (outcome.result === 'signal') && (outcome.exitCode === 'killed');
  }
  return (outcome.result === 'exit-code') && (outcome.exitCode === 'exited')
    && (outcome.exitStatus === RCLONE_SIGTERM_EXIT_STATUS);
}

/**
 Everything the alert entry needs to decide and to write its message.

 `invocationId` is empty when systemd passed none, and the journal read then uses the
 unit's latest invocation.
 */
export type AlertRequest = {
  readonly trigger: AlertTrigger;
  readonly unit: string;
  readonly invocationId: string;
  readonly outcome: UnitOutcome;
};

/**
 A command line the entry cannot use.
 */
export class AlertArgumentError extends Error {
  /**
   Names the argument the entry refused.

   @param message - what was wrong with the command line, shown to the reader
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'AlertArgumentError';
  }
}

/**
 Classifies a finished run from systemd's monitor variables.

 @param outcome - result, exit code and exit status systemd reported

 @returns `recovery` only for a success result with exit status 0, `stopped` for another
   success result, and `failure` for everything else, including a missing result

 @example
 ```ts
 classifyOutcome({ result: 'success', exitCode: 'exited', exitStatus: '0', },); // 'recovery'
 classifyOutcome({ result: 'success', exitCode: 'exited', exitStatus: '1', },); // 'stopped'
 ```
 */
export function classifyOutcome(outcome: Readonly<UnitOutcome>,): OutcomeClass {
  if (outcome.result !== 'success') return 'failure';
  if ((outcome.exitCode === 'exited') && (outcome.exitStatus === '0')) return 'recovery';
  return 'stopped';
}

/**
 Classifies one alert event from what started the entry and what systemd reported.

 `OnFailure=` starts its handler for a failed run and for a run stopped by hand, so
 `unit-failed` is a failure unless systemd reported one of the SIGTERM shapes. `OnSuccess=`
 fires for a success result only, so `unit-succeeded` is a recovery when systemd reports
 exit status 0 and a stop otherwise, including when it passed no variables: a success
 handler never reports a failure.

 @param trigger - what started the entry, which names the dependency that fired

 @param outcome - result, exit code and exit status systemd reported, empty when it passed none

 @returns the class the alert acts on

 @example
 ```ts
 classifyRequest({ trigger: 'unit-failed', outcome: { result: 'signal', exitCode: 'killed', exitStatus: 'TERM', }, },); // 'stopped'
 ```
 */
export function classifyRequest({
  trigger,
  outcome,
}: Readonly<{
  trigger: AlertTrigger;
  outcome: UnitOutcome;
}>): OutcomeClass {
  if (trigger === 'hook-failed') return 'failure';
  if (trigger === 'unit-failed') return isHandStopped(outcome,) ? 'stopped' : 'failure';
  if (trigger === 'watcher-started') return 'recovery';
  if (trigger === 'unit-succeeded') {
    return classifyOutcome(outcome,) === 'recovery' ? 'recovery' : 'stopped';
  }
  return classifyOutcome(outcome,);
}

/**
 Tells whether a message for this event reports a recovery rather than a failure.

 @param trigger - what started the entry

 @param outcome - result systemd reported, which decides a hand-started handler

 @returns true for the watcher starting again, for a success handler, and for a hand-started
   handler whose run systemd reports as a success

 @example
 ```ts
 reportsRecovery({ trigger: 'unit-failed', outcome, },); // false
 ```
 */
export function reportsRecovery({
  trigger,
  outcome,
}: Readonly<{
  trigger: AlertTrigger;
  outcome: UnitOutcome;
}>): boolean {
  return (trigger === 'watcher-started') || (trigger === 'unit-succeeded')
    || ((trigger === 'unit') && (outcome.result === 'success'));
}

/**
 Reads the trigger and unit name from the entry's command line.

 The unit name is positional because the alert unit passes its own instance name
 (`%i`); the trigger is a `--trigger=` option so a hook can name the mode it reports.

 @param argv - arguments after the script name

 @returns the trigger and the unit the command line named, if any

 @throws AlertArgumentError when an argument is not understood

 @example
 ```ts
 parseAlertArguments(['rclone-bisync-garage-watch.service', '--trigger=hook-failed',],);
 // { trigger: 'hook-failed', unit: 'rclone-bisync-garage-watch.service' }
 ```
 */
export function parseAlertArguments(argv: readonly string[],): {
  readonly trigger: AlertTrigger;
  readonly unit?: string;
} {
  /**
   Option prefix that carries the trigger.
   */
  const triggerPrefix = '--trigger=';
  /**
   The parsed command line, filled as arguments are read.
   */
  const parsed: {
    trigger: AlertTrigger;
    unit?: string
  } = { trigger: 'unit', };
  for (const argument of argv) {
    if (argument.startsWith(triggerPrefix,)) {
      /**
       Trigger value, as written after the prefix.
       */
      const value = argument.slice(triggerPrefix.length,);
      if (!isAlertTrigger(value,)) {
        throw new AlertArgumentError(
          `--trigger must be one of ${ALERT_TRIGGERS.join(', ',)}; got "${value}"`,
        );
      }
      parsed.trigger = value;
      continue;
    }
    if (argument.startsWith('-',)) {
      throw new AlertArgumentError(`unknown option "${argument}"`);
    }
    if (parsed.unit !== undefined) {
      throw new AlertArgumentError(`only one unit name is allowed; got "${argument}" as well as "${parsed.unit}"`);
    }
    parsed.unit = argument;
  }
  return parsed;
}

/**
 Builds the request for one alert run.

 The unit name comes from systemd's `MONITOR_UNIT` first, because that is the run the
 alert describes; a positional name is the fallback for a handler started by hand, and
 the sync unit is the last resort. The two hook triggers always describe the watcher
 unit unless a name is given.

 @param argv - arguments after the script name

 @param env - environment systemd started the entry with

 @returns the trigger, unit, invocation id and outcome to act on

 @throws AlertArgumentError when the command line is not understood

 @example
 ```ts
 resolveAlertRequest({
   argv: ['rclone-bisync-garage.service',],
   env: { MONITOR_SERVICE_RESULT: 'exit-code', MONITOR_EXIT_STATUS: '1', },
 },);
 // { trigger: 'unit', unit: 'rclone-bisync-garage.service', invocationId: '', outcome: ... }
 ```
 */
export function resolveAlertRequest({
  argv,
  env,
}: Readonly<{
  argv: readonly string[];
  env: Readonly<NodeJS.ProcessEnv>;
}>): AlertRequest {
  /**
   Command line, already parsed.
   */
  const parsed = parseAlertArguments(argv,);
  /**
   Unit this alert describes.
   */
  const unit = isUnitTrigger(parsed.trigger,)
    ? env.MONITOR_UNIT ?? parsed.unit
      ?? SYNC_UNIT
    : parsed.unit ?? WATCH_UNIT;
  return {
    trigger: parsed.trigger,
    unit,
    invocationId: env.MONITOR_INVOCATION_ID ?? '',
    outcome: {
      result: env.MONITOR_SERVICE_RESULT ?? '',
      exitCode: env.MONITOR_EXIT_CODE ?? '',
      exitStatus: env.MONITOR_EXIT_STATUS ?? '',
    },
  };
}
