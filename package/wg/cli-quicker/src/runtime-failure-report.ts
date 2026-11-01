import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { text, } from 'node:stream/consumers';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  escapeMarkup,
  sanitizeText,
} from './escape-text.ts';
import {
  createRepeatLimit,
  withHeldRepeats,
} from './failure-repeat-limit.ts';

/**
 Module logger for failure delivery.
 */
const l = tagged({ tag: 'runtime-failure-report', },);

/**
 One notification per failure kind per this many milliseconds.
 */
export const FAILURE_NOTIFICATION_INTERVAL_MS = 600_000;

/**
 Milliseconds a notification stays on the desktop,
 as long as the interval after which the same kind is sent again.
 */
export const NOTIFICATION_EXPIRY_MS = 600_000;

/**
 One journal line per failure kind per this many milliseconds,
 so a failure that repeats many times a second cannot flood the journal.
 */
export const FAILURE_JOURNAL_INTERVAL_MS = 60_000;

/**
 Notification ids remembered for replacement before the memory is cleared.
 */
const MAX_SHOWN_KINDS = 64;

/**
 Milliseconds a delivery command may run before it is killed.
 */
const COMMAND_TIMEOUT_MS = 3_000;

/**
 Identifier written before each journal line,
 shown by `journalctl --identifier=wg-quicker`.
 */
export const JOURNAL_IDENTIFIER = 'wg-quicker';

/**
 Journal priority of every failure:
 error level,
 as the owner chose for failures after startup.
 */
export const JOURNAL_PRIORITY = 'err';

/**
 Search path of every delivery command and of the notification client,
 a cleared environment otherwise.
 */
const SECURE_PATH = '/usr/bin:/bin';

/**
 Summary line of every desktop notification.
 */
export const NOTIFICATION_SUMMARY = 'wg-quicker: update traffic policy not in force';

/**
 One command run for delivery.
 */
export type DeliveryCommand = {
  /**
   Executable name found on the secure path.
   */
  readonly command: string;

  /**
   Arguments.
   */
  readonly args: readonly string[];

  /**
   Text written to standard in.
   */
  readonly input?: string;
};

/**
 Runs one delivery command,
 rejecting when it fails or runs too long,
 and resolves to what it printed.
 */
export type DeliveryRunner = (call: DeliveryCommand,) => Promise<string>;

/**
 The user whose desktop receives notifications.
 */
export type NotificationUser = {
  /**
   Numeric user id.
   */
  readonly uid: number;

  /**
   Numeric primary group id used for the privilege drop.
   */
  readonly gid: number;
};

/**
 The user to notify,
 or why none can be named.
 */
export type NotificationUserLookup =
  | {
    readonly found: true;
    readonly user: NotificationUser
  }
    | {
      readonly found: false;
      readonly reason: string
    };

/**
 The environment values that name the user who started the tunnel through `sudo`.
 */
export type SudoEnvironment = {
  /**
   Numeric user id of the caller.
   */
  readonly SUDO_UID?: string;

  /**
   Numeric primary group id of the caller.
   */
  readonly SUDO_GID?: string;
};

/**
 Delivers one loud failure line to a person.
 The kind, not the text, decides what counts as a repeat,
 because the text embeds names a user chose and distinct names would defeat a text key.
 */
export type FailureDelivery = (call: {
  readonly kind: string;
  readonly line: string
},) => Promise<void>;

/**
 Reports whether text holds a character that is not an ASCII decimal digit.

 @param value - Text to check.

 @returns Whether any character is not a digit.

 @example
 ```ts
 hasNonDigit({ value: '12a' }); // true
 ```
 */
function hasNonDigit({ value, }: { readonly value: string; },): boolean {
  for (const character of value)
    if ((character < '0') || (character > '9'))
      return true;
  return false;
}

/**
 Reads one decimal id out of the environment of a sudo-started process.

 @param name - Variable name,
 for the message.

 @param value - Variable value.

 @returns The id,
 or the reason it is unusable.

 @example
 ```ts
 readSudoId({ name: 'SUDO_UID', value: '1000' }); // { ok: true, id: 1000 }
 ```
 */
function readSudoId(
  {
    name,
    value,
  }: {
    readonly name: string;
    readonly value?: string;
  },
): {
  readonly ok: true;
  readonly id: number
} | {
  readonly ok: false;
  readonly reason: string
} {
  if (value === undefined)
    return {
      ok: false,
      reason: `${name} is not set`,
    };
  /**
   Numeric value after the syntax check.
   */
  const id = Math.trunc(Number(value,),);
  if ((value === '') || hasNonDigit({ value, },)
    || (!Number.isSafeInteger(id,)))
    return {
      ok: false,
      reason: `${name} is not a decimal id: ${value}`,
    };
  return {
    ok: true,
    id,
  };
}

/**
 Names the user who receives desktop notifications:
 the sudo caller,
 the same user the process watch follows.

 @param environment - Environment of the watcher,
 where sudo recorded the caller.

 @returns The user,
 or the reason no user can be named.

 @example
 ```ts
 resolveNotificationUser({ environment: { SUDO_UID: '1000', SUDO_GID: '1000' } });
 ```
 */
export function resolveNotificationUser(
  { environment, }: { readonly environment: SudoEnvironment; },
): NotificationUserLookup {
  /**
   Caller user id.
   */
  const uid = readSudoId({
    name: 'SUDO_UID',
    ...(environment.SUDO_UID === undefined ? {} : { value: environment.SUDO_UID, }),
  },);
  if (!uid.ok)
    return {
      found: false,
      reason: `no user to notify: ${uid.reason}`,
    };
  /**
   Caller group id.
   */
  const gid = readSudoId({
    name: 'SUDO_GID',
    ...(environment.SUDO_GID === undefined ? {} : { value: environment.SUDO_GID, }),
  },);
  if (!gid.ok)
    return {
      found: false,
      reason: `no user to notify: ${gid.reason}`,
    };
  return {
    found: true,
    user: {
      uid: uid.id,
      gid: gid.id,
    },
  };
}

/**
 Builds the command that writes one line to the journal at error level.

 @param line - The failure line.

 @returns Command with the line on standard in.

 @example
 ```ts
 journalCommand({ line: 'POLICY NOT IN FORCE: x' });
 ```
 */
export function journalCommand({ line, }: { readonly line: string; },): DeliveryCommand {
  return {
    command: 'systemd-cat',
    args: [
      `--identifier=${JOURNAL_IDENTIFIER}`,
      `--priority=${JOURNAL_PRIORITY}`,
    ],
    input: `${sanitizeText({ text: line, },)}\n`,
  };
}

/**
 Builds the `busctl` arguments for a desktop notification
 through the freedesktop notification service.

 @param summary - Notification title.

 @param body - Notification text,
 escaped for the markup of the notification service.

 @param replacesId - Id of the notification this one replaces,
 zero for none.

 @param expiryMs - Milliseconds the notification stays.

 @returns Arguments after `busctl`.

 @example
 ```ts
 notificationArguments({ summary: 'title', body: 'text', replacesId: 0, expiryMs: 600000 });
 ```
 */
export function notificationArguments(
  {
    summary,
    body,
    replacesId,
    expiryMs,
  }: {
    readonly summary: string;
    readonly body: string;
    readonly replacesId: number;
    readonly expiryMs: number;
  },
): readonly string[] {
  return [
    '--user',
    'call',
    'org.freedesktop.Notifications',
    '/org/freedesktop/Notifications',
    'org.freedesktop.Notifications',
    'Notify',
    'susssasa{sv}i',
    'wg-quicker',
    String(replacesId,),
    'dialog-warning',
    summary,
    escapeMarkup({ text: sanitizeText({ text: body, },), },),
    '0',
    '0',
    String(expiryMs,),
  ];
}

/**
 Builds the command that shows a desktop notification to the user.

 A root watcher drops to the user's real and effective ids and clears supplementary groups through `setpriv`,
 so the bus call is made by the user and not by root.
 The client runs with a cleared environment that holds only the secure path and the user's runtime directory.
 A watcher that already runs as the user calls the client directly.

 @param user - The user to notify.

 @param line - The failure line shown as the text.

 @param asRoot - Whether the watcher runs as root.

 @param replacesId - Id of the notification this one replaces.

 @returns The command.

 @example
 ```ts
 notificationCommand({ user: { uid: 1000, gid: 1000 }, line: 'text', asRoot: true });
 ```
 */
export function notificationCommand(
  {
    user,
    line,
    asRoot,
    replacesId = 0,
  }: {
    readonly user: NotificationUser;
    readonly line: string;
    readonly asRoot: boolean;
    readonly replacesId?: number;
  },
): DeliveryCommand {
  /**
   The client invocation behind the environment reset.
   */
  const client = [
    'env',
    '--ignore-environment',
    `PATH=${SECURE_PATH}`,
    `XDG_RUNTIME_DIR=/run/user/${String(user.uid,)}`,
    'busctl',
    ...notificationArguments({
      summary: NOTIFICATION_SUMMARY,
      body: line,
      replacesId,
      expiryMs: NOTIFICATION_EXPIRY_MS,
    },),
  ];
  if (!asRoot)
    return {
      command: client[0] ?? 'env',
      args: client.slice(1,),
    };
  return {
    command: 'setpriv',
    args: [
      '--reuid',
      String(user.uid,),
      '--regid',
      String(user.gid,),
      '--clear-groups',
      '--no-new-privs',
      ...client,
    ],
  };
}

/**
 Runs one delivery command with a cleared environment and a timeout.

 @param call - The command.

 @param timeoutMs - Milliseconds before the command is killed.

 @returns What the command printed.

 @throws When the command cannot start,
 exits nonzero,
 or runs past the timeout and is killed.

 @example
 ```ts
 await runDeliveryCommandWithin({ call: { command: 'true', args: [] }, timeoutMs: 3000 });
 ```
 */
export async function runDeliveryCommandWithin(
  {
    call,
    timeoutMs,
  }: {
    readonly call: DeliveryCommand;
    readonly timeoutMs: number;
  },
): Promise<string> {
  /**
   Running command with a cleared environment,
   killed when it outlives the timeout.
   */
  const child = spawn(
    call.command,
    [...call.args,],
    {
      env: { PATH: SECURE_PATH, },
      killSignal: 'SIGKILL',
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
      timeout: timeoutMs,
    },
  );
  child.stdin
    .on(
    'error',
    function ignoreBrokenPipe(): void {
      // A child that exits before reading its input reports through its exit code.
    },
    );
  child.stdin
    .end(call.input ?? '',);
  /**
   Diagnostics of the child,
   read while waiting for it to close.
   */
  const [diagnostic, printed,] = await Promise.all([
    text(child.stderr,),
    text(child.stdout,),
    once(
      child,
      'close',
    ),
  ],);
  if (child.exitCode !== 0)
    throw new Error(
      child.signalCode === null
        ? `${call.command} exited ${String(child.exitCode,)}: ${diagnostic.trim()}`
        : `${call.command} was killed by ${child.signalCode} after ${String(timeoutMs,)} ms: ${diagnostic.trim()}`,
    );
  return printed;
}

/**
 Runs one delivery command with the production timeout.

 @param call - The command.

 @returns What the command printed.

 @throws When the command cannot start,
 exits nonzero,
 or runs past the timeout and is killed.

 @example
 ```ts
 await runDeliveryCommand({ command: 'true', args: [] });
 ```
 */
export function runDeliveryCommand(call: DeliveryCommand,): Promise<string> {
  return runDeliveryCommandWithin({
    call,
    timeoutMs: COMMAND_TIMEOUT_MS,
  },);
}

/**
 Reads the notification id out of what `busctl` printed for `Notify`,
 which looks like `u 42`.

 @param output - What the client printed.

 @returns The id,
 or zero when the output has none.

 @example
 ```ts
 parseNotificationId({ output: 'u 42\n' }); // 42
 ```
 */
export function parseNotificationId({ output, }: { readonly output: string; },): number {
  /**
   Words of the output.
   */
  const [type, value,] = output.trim()
    .split(' ',);
  if ((type !== 'u') || (value === undefined)
    || hasNonDigit({ value, },)
    || (value === ''))
    return 0;
  return Math.trunc(Number(value,),);
}

/**
 Delivers failures after startup to a person:
 to the journal at error level once per distinct failure per minute,
 and to the user's desktop as a notification once per distinct failure per ten minutes.

 A channel that fails is named in the log and never changes routing.

 @param lookup - The user to notify.

 @param run - Command runner,
 replaceable in tests.

 @param now - Clock in milliseconds,
 replaceable in tests.

 @param asRoot - Whether this process runs as root.

 @param reportChannelFailure - Receives the words naming a channel that failed,
 by default the error log.

 @returns The delivery function.

 @example
 ```ts
 const deliver = createFailureDelivery({ lookup: resolveNotificationUser({ environment: process.env }) });
 ```
 */
export function createFailureDelivery(
  {
    lookup,
    run = runDeliveryCommand,
    now = Date.now,
    asRoot = process.geteuid?.() === 0,
    reportChannelFailure,
  }: {
    readonly lookup: NotificationUserLookup;
    readonly run?: DeliveryRunner;
    readonly now?: () => number;
    readonly asRoot?: boolean;
    readonly reportChannelFailure?: (message: string,) => void;
  },
): FailureDelivery {
  /**
   Repeat limit of the journal.
   */
  const journalLimit = createRepeatLimit({ intervalMs: FAILURE_JOURNAL_INTERVAL_MS, },);
  /**
   Repeat limit of the desktop.
   */
  const desktopLimit = createRepeatLimit({ intervalMs: FAILURE_NOTIFICATION_INTERVAL_MS, },);
  /**
   Function-scoped logger for the delivery.
   */
  const fl = tagged({
    tag: createFailureDelivery.name,
    l,
  },);
  /**
   Id of the notification last shown for each kind,
   so the next one of the kind replaces it.
   */
  const shown = new Map<string, number>();
  return async function deliver({
    kind,
    line,
  },): Promise<void> {
    /**
     Channels that failed for this line.
     */
    const failed: string[] = [];
    /**
     Current time.
     */
    const current = now();
    /**
     Decision for the journal.
     */
    const journal = journalLimit({
      key: kind,
      now: current,
    },);
    if (journal.deliver)
      try {
        await run(journalCommand({ line: withHeldRepeats({
          line,
          held: journal.held,
        },), },),);
      }
      catch (error) {
        failed.push(`journal: ${String(error,)}`,);
      }
    /**
     Decision for the desktop.
     */
    const desktop = desktopLimit({
      key: kind,
      now: current,
    },);
    if (desktop.deliver) {
      if (lookup.found)
        try {
          /**
           What the notification client printed:
           the id of the notification now on the desktop.
           */
          const printed = await run(notificationCommand({
            user: lookup.user,
            line: withHeldRepeats({
              line,
              held: desktop.held,
            },),
            asRoot,
            replacesId: shown.get(kind,) ?? 0,
          },),);
          if (shown.size >= MAX_SHOWN_KINDS)
            shown.clear();
          shown.set(
            kind,
            parseNotificationId({ output: printed, },),
          );
        }
        catch (error) {
          failed.push(`desktop notification: ${String(error,)}`,);
        }
      else
        failed.push(`desktop notification: ${lookup.reason}`,);
    }
    for (const problem of failed) {
      /**
       Words naming the channel that failed and the report it could not carry.
       */
      const message = sanitizeText({ text: `a failure report channel failed, so a person may not see the report: ${problem}; report was: ${line}`, },);
      fl.error(message,);
      reportChannelFailure?.(message,);
    }
  };
}
