/**
 Turns systemd's notification of a finished run into the alert's inputs.

 systemd sets `MONITOR_SERVICE_RESULT`, `MONITOR_EXIT_CODE`, `MONITOR_EXIT_STATUS`,
 `MONITOR_INVOCATION_ID` and `MONITOR_UNIT` in a handler started through
 `OnFailure=` or `OnSuccess=` (systemd.exec(5), "$MONITOR_* usage", systemd 262).
 Those variables are the only way to tell a run that exited with status 0 from a run
 that was stopped by hand, which also ends in the unit's "success" result.

 @module
 */

import {
  SYNC_UNIT,
  WATCH_UNIT,
} from './units.ts';

/**
 What started the alert entry.

 `unit` is a real run of a unit, reported by systemd's `OnFailure=` or `OnSuccess=`;
 `hook-failed` is the watcher's change hook failing to start the trigger unit;
 `watcher-started` is the watcher's own start hook, which reports a watcher outage
 whose alert streak is open.
 */
export type AlertTrigger =
  | 'unit'
    | 'hook-failed'
    | 'watcher-started';

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

 `recovery` is a run that exited with status 0; `stopped` is a run that ended in the
 unit's "success" result by any other means, which includes a run stopped by hand
 (systemd counts SIGTERM as a clean stop).
 */
export type OutcomeClass =
  | 'failure'
    | 'recovery'
    | 'stopped';

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

 @returns `recovery` only for a run that exited with status 0, `stopped` for another
   clean stop, and `failure` for everything else, including a missing result

 @example
 ```ts
 classifyOutcome({ result: 'success', exitCode: 'exited', exitStatus: '0', },); // 'recovery'
 classifyOutcome({ result: 'success', exitCode: 'killed', exitStatus: 'TERM', },); // 'stopped'
 ```
 */
export function classifyOutcome(outcome: Readonly<UnitOutcome>,): OutcomeClass {
  if (outcome.result !== 'success') return 'failure';
  if ((outcome.exitCode === 'exited') && (outcome.exitStatus === '0')) return 'recovery';
  return 'stopped';
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
      if ((value !== 'unit') && (value !== 'hook-failed')
        && (value !== 'watcher-started')) {
        throw new AlertArgumentError(
          `--trigger must be one of unit, hook-failed, watcher-started; got "${value}"`,
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
  const unit = parsed.trigger === 'unit'
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
