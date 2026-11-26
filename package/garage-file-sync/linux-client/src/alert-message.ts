/**
 Builds the desktop notification for one alert.

 The text names what happened, what to do next, and where the log is, in plain words.
 It never assumes an hourly run: the five-minute timer and the change trigger make runs
 frequent, so guidance says "the next run" and names the run's own log command.

 @module
 */

import { join, } from 'node:path';
import {
  causeLines,
  errorLines,
  tailLines,
} from './alert-journal.ts';
import {
  type AlertTrigger,
  reportsRecovery,
  type UnitOutcome,
} from './alert-outcome.ts';
import {
  SYNC_UNIT,
  TRIGGER_UNIT,
} from './units.ts';

/**
 Rclone's name for a refused file name, which a ZFS dataset refuses once decomposed.
 */
const NAME_REFUSED_MARKERS: readonly string[] = [
  'operation not supported',
  'file name too long',
];

/**
 Marker that bisync needs a resync before it can continue.
 */
const RESYNC_MARKERS: readonly string[] = [
  'Must run --resync to recover',
  'cannot find prior Path1 or Path2 listings',
];

/**
 Journal line that says bisync aborted the run.
 */
const ABORT_MARKER = 'Bisync aborted';

/**
 Recovery steps for a stopped bisync, in the repository's troubleshooting docs.
 */
const RECOVERY_DOC = '~/Monochromatic/doc/troubleshooting/rclone-bisync-aborts-on-a-name-the-local-disk-refuses.md';

/**
 How much of a non-bisync unit's journal an alert quotes.
 */
const WATCHER_TAIL_LINES = 3;

/**
 A notification as notify-send receives it.
 */
export type Notification = {
  readonly summary: string;
  readonly body: string;
};

/**
 Everything an alert message is built from.
 */
export type MessageInput = {
  readonly trigger: AlertTrigger;
  readonly unit: string;
  readonly outcome: UnitOutcome;
  readonly journalLines: readonly string[];
  readonly home: string;
};

/**
 Describes a run's outcome in words a reader can act on.

 @param outcome - result, exit code and exit status systemd reported

 @returns the result with its exit detail, or that the result is unknown

 @example
 ```ts
 describeOutcome({ result: 'exit-code', exitCode: 'exited', exitStatus: '1', },);
 // 'exit-code (exited status 1)'
 ```
 */
export function describeOutcome(outcome: Readonly<UnitOutcome>,): string {
  if (outcome.result === '') return 'an unknown result, because systemd set no service result';
  if ((outcome.exitCode === '') && (outcome.exitStatus === '')) return outcome.result;
  return `${outcome.result} (${outcome.exitCode} status ${outcome.exitStatus})`;
}

/**
 Whether a message says bisync needs a resync.

 @param errors - error messages of the run

 @returns true when one of them carries a resync marker
 */
function needsResync(errors: readonly string[],): boolean {
  return errors.some(function mentionsResync(error,) {
    return RESYNC_MARKERS.some(function matches(marker,) {
      return error.includes(marker,);
    },);
  },);
}

/**
 The address and title of the notification.

 @param trigger - what started the entry

 @param unit - unit the alert describes

 @param outcome - result, exit code and exit status systemd reported

 @returns summary line notify-send shows as the notification title

 @example
 ```ts
 summaryFor({ trigger: 'hook-failed', unit: 'rclone-bisync-garage-watch.service', ... },);
 // 'Garage sync failed (rclone-bisync-garage-watch.service)'
 ```
 */
export function summaryFor({
  trigger,
  unit,
  outcome,
}: Readonly<{
  trigger: AlertTrigger;
  unit: string;
  outcome: UnitOutcome;
}>): string {
  /**
   Whether this message reports a recovery rather than a failure.
   */
  const recovered = reportsRecovery({
    trigger,
    outcome,
  },);
  if (recovered) {
    return unit === SYNC_UNIT ? 'Garage sync recovered' : `Garage sync recovered (${unit})`;
  }
  return unit === SYNC_UNIT ? 'Garage sync failed' : `Garage sync failed (${unit})`;
}

/**
 Removes a leading path from a journal message.

 @param line - the message

 @param localRoot - the local bisync root with a trailing separator

 @returns the message without the local root
 */
function shorten({
  line,
  localRoot,
}: Readonly<{
  line: string;
  localRoot: string;
}>): string {
  return line.replaceAll(
    localRoot,
    '',
  );
}

/**
 Body of an alert about a failed bisync run.

 @param input - the failed unit, its outcome and its journal lines

 @returns body lines, the log command last
 */
function syncFailureBody(input: Readonly<MessageInput>,): readonly string[] {
  /**
   Body lines, filled in reading order.
   */
  const body: string[] = [];
  /**
   Error messages of this run.
   */
  const errors = errorLines(input.journalLines,);
  /**
   Causes worth quoting.
   */
  const causes = causeLines({
    errors,
    localRoot: `${join(
      input.home,
      'Plain',
    )}/`,
  },);
  if (input.outcome
    .result
    === 'timeout') {
    body.push('The run hit its time limit and was stopped.',);
  }
  if (causes.length > 0) {
    body.push(...causes,);
  } else if (errors.length > 0) {
    body.push(`rclone: ${shorten({
      line: errors[0] ?? '',
      localRoot: `${join(
        input.home,
        'Plain',
      )}/`,
    },)}`,);
  } else if (input.outcome
    .result
    !== 'timeout') {
    body.push(`Exited with ${describeOutcome(input.outcome,)}, with no ERROR line in the log.`,);
  }
  if (causes.some(function refusedName(cause,) {
    /**
     Cause in lower case, because rclone's wording varies in case.
     */
    const lowerCased = cause.toLowerCase();
    return NAME_REFUSED_MARKERS.some(function matches(marker,) {
      return lowerCased.includes(marker,);
    },);
  },)) {
    body.push('That name is too long for this disk once decomposed (ZFS formD). Rename it shorter on Garage.',);
  }
  /**
   Whether any error message says bisync stopped itself.
   */
  const aborted = errors.some(function mentionsAbort(error,) {
    return error.includes(ABORT_MARKER,) || needsResync([error,],);
  },);
  if (needsResync(errors,)) {
    body.push(
      `bisync stopped syncing until a --resync. Steps: ${RECOVERY_DOC}`,
    );
  } else if (aborted) {
    body.push(
      'Other files keep syncing. Once the cause is fixed, the next run should recover by itself'
        + ' (--recover uses the backup listings; measured on local test folders), despite rclone\'s "Must run --resync".'
        + ` Watch for .conflict copies afterwards. Details: ${RECOVERY_DOC}`,
    );
  }
  return body;
}

/**
 Body of an alert about a unit that is not the bisync unit.

 @param input - the failed unit, its outcome and its journal lines

 @returns body lines, the log command last
 */
function otherUnitFailureBody(input: Readonly<MessageInput>,): readonly string[] {
  /**
   Last lines of the run, which for a crashed watcher carry its own error message.
   */
  const tail = tailLines({
    lines: input.journalLines,
    count: WATCHER_TAIL_LINES,
  },);
  if (tail.length > 0) {
    return [
      'Last lines of the failed run:',
      ...tail,
    ];
  }
  return [`Exited with ${describeOutcome(input.outcome,)}, and the run wrote nothing to its own log.`,];
}

/**
 Body of the alert that reports a watcher hook that could not start a sync.

 @returns body lines, the log command last
 */
function hookFailureBody(): readonly string[] {
  return [
    `A change in ~/Plain could not start ${TRIGGER_UNIT}.`,
    'Syncs still run on the five-minute timer, and the watcher keeps watching:',
    'the next change tries again, and the recovery notice comes when the watcher starts again.',
  ];
}

/**
 Body of the recovery alert for a watcher that started again.

 @returns body lines, the log command last
 */
function watcherRecoveryBody(): readonly string[] {
  return [
    'The watcher is running again, so a change in ~/Plain starts a sync run once more.',
    'The five-minute timer kept syncing while it was away.',
  ];
}

/**
 Body of every other recovery alert.

 @returns body lines, the log command last
 */
function runRecoveryBody(): readonly string[] {
  return [
    'The run finished with exit status 0, so the alert streak is closed.',
    'The next failed run alerts again.',
  ];
}

/**
 Builds the notification for one alert.

 @param input - the trigger, unit, outcome, journal lines and home folder

 @returns the summary and body notify-send receives

 @example
 ```ts
 const { summary, body } = composeMessage({ trigger: 'unit', unit: SYNC_UNIT, outcome, journalLines, home, },);
 ```
 */
export function composeMessage(input: Readonly<MessageInput>,): Notification {
  /**
   Whether this message reports a recovery.
   */
  const recovered = reportsRecovery(input,);
  /**
   Body lines before the log command.
   */
  const body: string[] = [];
  if (recovered) {
    if (input.trigger === 'watcher-started') {
      body.push(...watcherRecoveryBody(),);
    } else {
      body.push(...runRecoveryBody(),);
    }
  } else if (input.trigger === 'hook-failed') {
    body.push(...hookFailureBody(),);
  } else if (input.unit === SYNC_UNIT) {
    body.push(...syncFailureBody(input,),);
  } else {
    body.push(...otherUnitFailureBody(input,),);
  }
  body.push(`Log: journalctl --user -u ${input.unit} --since today`,);
  return {
    summary: summaryFor(input,),
    body: body.join('\n',),
  };
}
