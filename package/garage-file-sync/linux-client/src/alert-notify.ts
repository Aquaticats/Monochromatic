/**
 Sends one desktop notification with notify-send.

 notify-send talks to the graphical session's notification server, so a failure while
 logged out is not shown. The alert entry does not treat that as success: the state
 file is written only after notify-send exits 0, so the next failed run tries again.

 @module
 */

import type { Notification, } from './alert-message.ts';
import { runCommand, } from './run-command.ts';

/**
 How long notify-send may wait for the notification server, in milliseconds.

 GLib waits up to 25 s for the server's reply, so a shorter timeout would report a
 failure for an alert that is on screen.
 */
export const NOTIFY_TIMEOUT_MS = 30_000;

/**
 What one notify-send call produced.

 `sent` is true only when notify-send exited 0.
 */
export type NotifyOutcome = {
  readonly sent: boolean;
  readonly problem?: string;
};

/**
 Shows one notification.

 @param notification - summary and body to show

 @returns whether the notification server accepted it, or the reason it did not

 @example
 ```ts
 const { sent } = await sendNotification({ summary: 'Garage sync failed', body: 'disk full', },);
 ```
 */
export async function sendNotification(notification: Readonly<Notification>,): Promise<NotifyOutcome> {
  /**
   Finished notify-send run.
   */
  const result = await runCommand({
    command: 'notify-send',
    args: [
      '--urgency=critical',
      '--app-name=Garage sync',
      '--icon=dialog-error',
      notification.summary,
      notification.body,
    ],
    timeoutMs: NOTIFY_TIMEOUT_MS,
  },);
  if (result.problem !== undefined) return {
    sent: false,
    problem: result.problem,
  };
  if (result.exitCode !== 0) {
    return {
      sent: false,
      problem: `notify-send exited ${String(result.exitCode,)}: ${result.stderr
        .trim()}`,
    };
  }
  return { sent: true, };
}
