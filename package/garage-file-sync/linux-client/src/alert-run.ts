/**
 Runs one alert: decides whether it is due, shows it, and records the streak.

 Every effect arrives through `AlertDeps`, so the tests drive the whole decision on a
 disposable home with no journal, no notification server and no systemd.

 @module
 */

import type { JournalRead, } from './alert-journal.ts';
import {
  composeMessage,
  type Notification,
} from './alert-message.ts';
import type { NotifyOutcome, } from './alert-notify.ts';
import {
  classifyRequest,
  type AlertRequest,
} from './alert-outcome.ts';
import {
  decideFailure,
  decideRecovery,
  type Streak,
  type StreakLookup,
} from './alert-streak.ts';

/**
 The alert entry's effects, injected so tests replace them.

 `writeOut` prints what the entry decided to its own stdout, which systemd records in
 the journal; `writeError` prints a diagnostic to stderr without changing the outcome.
 */
export type AlertDeps = {
  readonly home: string;
  readonly readStreak: (unit: string,) => Promise<StreakLookup>;
  readonly writeStreak: (streak: Streak,) => Promise<void>;
  readonly removeStreak: () => Promise<void>;
  readonly readJournal: () => Promise<JournalRead>;
  readonly notify: (notification: Notification,) => Promise<NotifyOutcome>;
  readonly nowMs: () => number;
  readonly writeOut: (text: string,) => void;
  readonly writeError: (text: string,) => void;
};

/**
 Reports the streak and writes the recovery or failure alert.

 @param request - the trigger, unit, invocation id and outcome to act on

 @param deps - effects the alert needs

 @returns the exit status: 0 when the entry did what the event asked for, 1 when the
   notification or the state write failed

 @example
 ```ts
 const status = await runAlert({ request, deps, },);
 ```
 */
export async function runAlert({
  request,
  deps,
}: Readonly<{
  request: AlertRequest;
  deps: AlertDeps;
}>): Promise<number> {
  /**
   What this event means: a failure, a recovery, or neither.
   */
  const outcomeClass = classifyRequest(request,);
  /**
   The unit's open streak, and the reason its file could not be used.
   */
  const lookup = await deps.readStreak(request.unit,);
  if (lookup.problem !== undefined) {
    deps.writeError(`${request.unit}: ${lookup.problem}; treating it as no open streak`,);
  }
  if (outcomeClass === 'stopped') {
    deps.writeOut(
      `${request.unit}: the run was stopped, not failed and not finished with status 0; no alert.`
        + ' An open streak, if any, stays open.',
    );
    return 0;
  }
  if (outcomeClass === 'failure') {
    /**
     Whether this failure is due an alert.
     */
    const decision = decideFailure({
      lookup,
      nowMs: deps.nowMs(),
    },);
    if (decision === 'suppressed') {
      deps.writeOut(`${request.unit}: still failing, and the last alert was less than an hour ago; no alert.`,);
      return 0;
    }
    return alertFailure({
      request,
      deps,
    },);
  }
  /**
   Whether this clean run closes an open streak.
   */
  const recovery = decideRecovery({ lookup, },);
  if (recovery === 'nothing') {
    deps.writeOut(`${request.unit}: finished cleanly with no open streak; no alert.`,);
    return 0;
  }
  return alertRecovery({
    request,
    deps,
  },);
}

/**
 Shows and records a failure alert.

 @param request - the trigger, unit, invocation id and outcome to act on

 @param deps - effects the alert needs

 @returns 0 when the alert was shown and recorded, 1 when either failed
 */
async function alertFailure({
  request,
  deps,
}: Readonly<{
  request: AlertRequest;
  deps: AlertDeps;
}>): Promise<number> {
  /**
   The failed run's own journal.
   */
  const journal = await deps.readJournal();
  if (journal.problem !== undefined) {
    deps.writeError(`${request.unit}: could not read the run's journal: ${journal.problem}`,);
  }
  /**
   The notification to show.
   */
  const notification = composeMessage({
    trigger: request.trigger,
    unit: request.unit,
    outcome: request.outcome,
    journalLines: journal.lines,
    home: deps.home,
  },);
  /**
   What notify-send reported.
   */
  const notified = await deps.notify(notification,);
  if (!notified.sent) {
    deps.writeError(
      `${request.unit}: the alert was not shown, so the streak is unchanged: ${notified.problem ?? 'unknown reason'}`,
    );
    return 1;
  }
  try {
    await deps.writeStreak({ lastAlertAtMs: deps.nowMs(), },);
  } catch (error) {
    deps.writeError(`${request.unit}: the alert was shown but its streak could not be recorded: ${String(error,)}`,);
    return 1;
  }
  deps.writeOut(renderNotification(notification,),);
  return 0;
}

/**
 Shows a recovery alert and closes the streak.

 @param request - the trigger, unit, invocation id and outcome to act on

 @param deps - effects the alert needs

 @returns 0 when the alert was shown and the streak removed, 1 when either failed
 */
async function alertRecovery({
  request,
  deps,
}: Readonly<{
  request: AlertRequest;
  deps: AlertDeps;
}>): Promise<number> {
  /**
   The notification to show.
   */
  const notification = composeMessage({
    trigger: request.trigger,
    unit: request.unit,
    outcome: request.outcome,
    journalLines: [],
    home: deps.home,
  },);
  /**
   What notify-send reported.
   */
  const notified = await deps.notify(notification,);
  if (!notified.sent) {
    deps.writeError(
      `${request.unit}: the recovery alert was not shown, so the streak stays open: ${notified.problem ?? 'unknown reason'}`,
    );
    return 1;
  }
  try {
    await deps.removeStreak();
  } catch (error) {
    deps.writeError(`${request.unit}: the recovery alert was shown but its streak could not be removed: ${String(error,)}`,);
    return 1;
  }
  deps.writeOut(renderNotification(notification,),);
  return 0;
}

/**
 Renders a notification as the entry's stdout.

 @param notification - the notification that went out

 @returns summary and body, one blank line apart
 */
function renderNotification(notification: Readonly<Notification>,): string {
  return `${notification.summary}\n${notification.body}`;
}
