/**
 Payload of the Freedesktop desktop notification a settled Pi agent sends.

 Two properties of the payload are load-bearing, and both are the notification
 server's rules rather than this package's preferences:

 - The body names the settled agent's working folder, because many agents run at
   once and a notification that says only "an agent finished" says nothing about
   which one.
 - The body therefore differs between agents, which keeps consecutive
   settlements far apart in the one comparison Plasma's notification server makes
   before it refuses a request: two notifications whose application name,
   summary, body, desktop entry, event id, action names, icon and URLs all match,
   sent less than 1000 ms apart, are refused with
   `org.freedesktop.Notifications.Error.ExcessNotificationGeneration`
   (`libnotificationmanager/server_p.cpp:220-231` in plasma-workspace 6.8.2).

 @module
 */

import type { NotificationInvocation, } from './notification-process.ts';

//region Constants

/**
 Executable implementing the local Freedesktop desktop-notification boundary.
 */
const NOTIFICATION_COMMAND = 'notify-send';

/**
 Application name the desktop attributes the notification to.
 */
const NOTIFICATION_APP_NAME = 'Pi';

/**
 Title every settlement shares, so a popup is recognizable before it is read.
 */
const NOTIFICATION_TITLE = 'Pi agent finished';

/**
 Body used when Pi reports no working folder for the settled agent.

 Pi's extension context always carries one (`ExtensionContext.cwd`), so this text
 covers a host that stops supplying it rather than a state this package chooses.
 */
const NOTIFICATION_BODY_WITHOUT_FOLDER = 'Agent is idle and ready for input.';

//endregion Constants

//region Body

/**
 Renders the notification body that names which agent finished.

 @param folder - working folder Pi reported for the settled agent, already trimmed

 @returns body naming that folder

 @example
 ```ts
 notificationBody({ folder: '/work/agent-7', }); // 'Idle in /work/agent-7'
 ```
 */
function notificationBody(
  {
    folder,
  }: {
    readonly folder: string;
  },
): string {
  return `Idle in ${folder}`;
}

/**
 Builds the argument list handed to the desktop-notification executable.

 Every value stays a separate argument, so no part of the payload is ever read as
 shell syntax.

 @param body - notification body, which names the settled agent

 @returns application name, title and body, in the order the executable expects

 @example
 ```ts
 notificationArguments({ body: 'Idle in /work/agent-7', });
 // ['--app-name=Pi', 'Pi agent finished', 'Idle in /work/agent-7']
 ```
 */
function notificationArguments(
  {
    body,
  }: {
    readonly body: string;
  },
): readonly string[] {
  return [
    `--app-name=${NOTIFICATION_APP_NAME}`,
    NOTIFICATION_TITLE,
    body,
  ];
}

//endregion Body

//region Invocation

/**
 Builds the command and arguments that deliver one settled-agent notification.

 @param folder - working folder Pi reported for the settled agent, when it reported one

 @returns executable plus the argument list that names this agent

 @example
 ```ts
 notificationInvocation({ folder: '/work/agent-7', });
 // { command: 'notify-send', args: ['--app-name=Pi', 'Pi agent finished', 'Idle in /work/agent-7'] }
 ```
 */
function notificationInvocation(
  {
    folder,
  }: {
    readonly folder?: string;
  },
): NotificationInvocation {
  /**
   Folder with surrounding whitespace removed, so a blank value reads as absent.
   */
  const namedFolder = folder?.trim() ?? '';
  return {
    command: NOTIFICATION_COMMAND,
    args: notificationArguments({
      body: namedFolder.length === 0
        ? NOTIFICATION_BODY_WITHOUT_FOLDER
        : notificationBody({ folder: namedFolder, },),
    },),
  };
}

//endregion Invocation

export {
  NOTIFICATION_APP_NAME,
  NOTIFICATION_BODY_WITHOUT_FOLDER,
  NOTIFICATION_COMMAND,
  NOTIFICATION_TITLE,
  notificationArguments,
  notificationBody,
  notificationInvocation,
};
