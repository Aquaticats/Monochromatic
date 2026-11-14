/**
 One desktop notification, on the path this repository's other tools use: `notify-send`, which asks the session bus's
 `org.freedesktop.Notifications` service to show it.

 The bus address is the caller's own, never one invented here: a filling started by a unit has the address the unit was given, and a
 process given no bus cannot reach the owner's desktop by a guessed path. The call is bounded, and a desktop that refuses or is not
 there is reported and never thrown: a filling that cannot reach the desktop must go on filling.

 @module
 */

import {
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';

/**
 How long `notify-send` may take. Its own D-Bus call waits 25 seconds for a service that never answers, and the settled-agent
 notification measured twenty deliveries at load average 81 to 96 between 20 ms and 3 060 ms, so this covers the slowest with room
 and stays well inside the wait GLib would make.
 */
export const DESKTOP_NOTICE_TIMEOUT_MILLISECONDS = 15_000;

/**
 The name the desktop shows as the notification's sender.
 */
export const DESKTOP_NOTICE_APP_NAME = 'pi-queue-filling';

/**
 One notification to show.
 */
export type DesktopNotice = {
  /**
   The notification's title.
   */
  readonly title: string;

  /**
   The notification's body.
   */
  readonly body: string;
};

/**
 What showing one notification answered.
 */
export type DesktopNoticeOutcome = {
  /**
   Whether the desktop accepted it.
   */
  readonly shown: boolean;

  /**
   What went wrong when it did not, and empty text when it did.
   */
  readonly problem?: string;
};

/**
 Shows one desktop notification.

 @param notice - the title and body to show

 @param run - how the command is run, which a test replaces

 @param timeoutMilliseconds - how long the command may take

 @returns whether the desktop accepted it, and why not when it did not

 @example
 ```ts
 const shown = await noticeOnTheDesktop({ notice: { title: 'Pi runs are not being started', body: '7 entries stand' } });
 ```
 */
export async function noticeOnTheDesktop({
  notice,
  run = runCommandWithChildProcess,
  timeoutMilliseconds = DESKTOP_NOTICE_TIMEOUT_MILLISECONDS,
}: {
  readonly notice: DesktopNotice;
  readonly run?: RunCommand;
  readonly timeoutMilliseconds?: number;
},): Promise<DesktopNoticeOutcome> {
  /**
   What `notify-send` ended with.
   */
  const ended = await run({
    command: 'notify-send',
    words: [
      `--app-name=${DESKTOP_NOTICE_APP_NAME}`,
      '--urgency=critical',
      notice.title,
      notice.body,
    ],
    timeoutMilliseconds,
  },);
  if (ended.status === 0) {
    return { shown: true, };
  }
  return {
    shown: false,
    problem: ended.problem
      ?? ended.stderr
        .trim(),
  };
}
