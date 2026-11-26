/**
 The alert entry: the file systemd starts as `rclone-bisync-garage-alert@<unit>.service`.

 What it does for each event is in `alert-run.ts`; this file only supplies the real
 effects: the state folder and the journal under the user's home, notify-send, and the
 program's own output.

 Its stdout is the notification text, which journald records for the alert unit, and
 its stderr carries diagnostics that must not reach the notification. Both are exact
 terminal output, so they use `console` rather than the repository's tagged logger,
 whose sinks would also write a log file for a program that runs for a second.

 @module
 */

import { homedir, } from 'node:os';
import { readRunJournal, } from './alert-journal.ts';
import { sendNotification, } from './alert-notify.ts';
import { backupListingsExist, } from './bisync-backup-listings.ts';
import {
  AlertArgumentError,
  type AlertRequest,
  resolveAlertRequest,
} from './alert-outcome.ts';
import {
  type AlertDeps,
  runAlert,
} from './alert-run.ts';
import {
  readStreak,
  removeStreak,
  writeStreak,
} from './alert-streak.ts';

/**
 Exit status of a command line the entry cannot use.
 */
const USAGE_EXIT_STATUS = 2;

/**
 Writes the entry's own output, which journald records for the alert unit.

 @param text - the notification text that went out, or the decision that made it silent
 */
function writeOut(text: string,): void {
  console.log(text,);
}

/**
 Writes a diagnostic that must not reach the notification.

 @param text - what went wrong
 */
function writeError(text: string,): void {
  console.error(text,);
}

/**
 Builds the real effects for one alert.

 @param home - home folder of the user the alert describes

 @param request - the unit and invocation this alert is about

 @returns every effect the alert runner calls, bound to that unit

 @example
 ```ts
 const deps = alertDeps({ home: '/home/example', request, },);
 ```
 */
function alertDeps({
  home,
  request,
}: Readonly<{
  home: string;
  request: AlertRequest;
}>): AlertDeps {
  return {
    home,
    readStreak: function readUnitStreak(unit,) {
      return readStreak({
        home,
        unit,
      },);
    },
    writeStreak: function recordUnitStreak(streak,) {
      return writeStreak({
        home,
        unit: request.unit,
        streak,
      },);
    },
    removeStreak: function removeUnitStreak() {
      return removeStreak({
        home,
        unit: request.unit,
      },);
    },
    readJournal: function readThisRunJournal() {
      return readRunJournal({
        unit: request.unit,
        invocationId: request.invocationId,
      },);
    },
    backupListingsExist: function readBackupListings() {
      return backupListingsExist({ home, },);
    },
    notify: sendNotification,
    nowMs: Date.now,
    writeOut,
    writeError,
  };
}

try {
  /**
   The event this entry was started for.
   */
  const request = resolveAlertRequest({
    argv: process.argv
      .slice(2,),
    env: process.env,
  },);
  process.exitCode = await runAlert({
    request,
    deps: alertDeps({
      home: homedir(),
      request,
    },),
  },);
} catch (error) {
  if (Error.isError(error,) && (error instanceof AlertArgumentError)) {
    console.error(`${error.message}
usage: alert.ts [<unit>] [--trigger=unit|hook-failed|watcher-started]`,);
    process.exitCode = USAGE_EXIT_STATUS;
  } else {
    console.error(`the alert entry could not finish: ${String(error,)}`,);
    process.exitCode = 1;
  }
}
