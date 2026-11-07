/**
 The recording notification bus for tests that run the real `notify-send`, and the probe that keeps them off the
 owner's bus.

 `package/nested-wayland-session/cli` carries a recording `org.freedesktop.Notifications` service and its own private
 session bus (a `dbus-daemon` with no service directories).
 Its `monochromatic-nested-wayland-session-notification-recorder` runs one command on that bus, with no compositor, and
 writes what the command sent to a log file.
 The recorder gives the command `DBUS_SESSION_BUS_ADDRESS` of the private bus and removes `XDG_RUNTIME_DIR`, so the
 command cannot find the host's bus by its default path either.
 This module only finds the recorder, proves the hosted environment is private before anything runs, and reads the log.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';

/**
 Name of the environment variable that points at the recorder, for a run that wants a specific build.
 */
export const RECORDER_VARIABLE = 'NESTED_WAYLAND_NOTIFICATION_RECORDER';

/**
 File name of the recorder binary.
 */
const RECORDER_NAME = 'monochromatic-nested-wayland-session-notification-recorder';

/**
 Where `mise run //package/nested-wayland-session/cli:build` (release) and the crate's tests (debug) leave it, under the
 repository root, release first.
 */
const RECORDER_BUILD_FOLDERS: readonly string[] = [
  join(
    'package',
    'nested-wayland-session',
    'cli',
    'target',
    'release',
  ),
  join(
    'package',
    'nested-wayland-session',
    'cli',
    'target',
    'debug',
  ),
];

/**
 One notification the recorder logged.
 */
export type RecordedNotification = {
  /**
   Application name the sender gave.
   */
  readonly app: string;
  /**
   Title.
   */
  readonly summary: string;
  /**
   Body text.
   */
  readonly body: string;
};

/**
 Script the probe runs inside the hosted environment, as the argument of `node -e`.
 It exits 3 when a runtime directory is visible, 4 when no session bus address is set or the address is the host's.
 */
const PROBE_SCRIPT = `
if (process.env.XDG_RUNTIME_DIR !== undefined) process.exit(3);
const address = process.env.DBUS_SESSION_BUS_ADDRESS;
if (address === undefined || address === '' || address === process.env.HOST_BUS_ADDRESS) process.exit(4);
`;

/**
 A candidate path of the recorder with whether it is a file.
 */
type Candidate = {
  /**
   The path.
   */
  readonly candidate: string;
  /**
   Whether it is a file.
   */
  readonly isFile: boolean;
};

/**
 Finds the recorder binary.

 @param repository - absolute repository path

 @returns the absolute path of the recorder

 @throws Error when no build of the recorder exists, naming the command that makes one

 @example
 ```ts
 const recorder = await findRecorder('/var/home/user/Monochromatic',);
 ```
 */
export async function findRecorder(repository: string,): Promise<string> {
  /**
   The override, empty when not set.
   */
  const override = process.env[RECORDER_VARIABLE] ?? '';
  /**
   Candidate paths, the override first.
   */
  const candidates = [
    ...override === '' ? [] : [override,],
    ...RECORDER_BUILD_FOLDERS.map(function inFolder(folder,) {
      return join(
        repository,
        folder,
        RECORDER_NAME,
      );
    },),
  ];
  // Looking at one candidate does not depend on looking at another, so they are looked at side by side.
  /**
   Each candidate with whether it is a file.
   */
  const found = await Promise.all(candidates.map(async function exists(candidate,): Promise<Candidate> {
    return {
      candidate,
      isFile: (await stat(
        candidate,
        { throwIfNoEntry: false, },
      ))?.isFile() === true,
    };
  },),);
  /**
   The first candidate that is a file.
   */
  const first = found.find(function isFile(entry,) {
    return entry.isFile;
  },);
  if (first === undefined) {
    throw new Error(
      `the notification recorder is not built (looked for ${candidates.join(', ',)}); build it with `
        + `\`mise run //package/nested-wayland-session/cli:test\` or set ${
         RECORDER_VARIABLE}`,
    );
  }
  return first.candidate;
}

/**
 Runs the probe in the environment a command would get from a launcher, and fails unless that environment is private.

 @param launcher - program and arguments that precede the hosted command: the recorder with its `--log` and `--`, or
   nothing for a run outside the recorder

 @param environment - environment of the launcher

 @throws Error when the hosted environment shows a runtime directory or no private bus address

 @example
 ```ts
 await assertHostedOnPrivateBus({ launcher: [recorder, '--log', log, '--',], environment, },);
 ```
 */
export async function assertHostedOnPrivateBus({
  launcher,
  environment,
}: {
  readonly launcher: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
},): Promise<void> {
  /**
   The program and its arguments: the launcher, then the probe.
   */
  const [program, ...args] = [
    ...launcher,
    process.execPath,
    '-e',
    PROBE_SCRIPT,
  ];
  /**
   The probe process, given the host's own bus address to compare with.
   */
  const child = spawn(
    program,
    args,
    {
      env: {
        ...environment,
        HOST_BUS_ADDRESS: environment.DBUS_SESSION_BUS_ADDRESS ?? '',
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  // Both streams are read to their end while waiting for the exit, so a full pipe cannot stall the probe.
  /**
   What the probe printed on standard error.
   */
  const [stderr,] = await Promise.all([
    text(child.stderr,),
    text(child.stdout,),
    once(
      child,
      'close',
    ),
  ],);
  if (child.exitCode !== 0) {
    throw new Error(
      `the test would not run on a private session bus: the hosted environment failed the probe with status ${child.exitCode} `
        + `(3: a runtime directory is visible, 4: the bus address is missing or the host's) ${stderr}`,
    );
  }
}

/**
 Reads one field of a log record, decoded from the percent form the log uses.

 @param record - one log record

 @param name - field name

 @returns the decoded value, the empty string when the record has no such field

 @example
 ```ts
 fieldOf({ record: 'notify summary=Hi%20there', name: 'summary', },); // 'Hi there'
 ```
 */
export function fieldOf({
  record,
  name,
}: {
  readonly record: string;
  readonly name: string;
}): string {
  /**
   The field, found among the space-separated parts of the record.
   */
  const part = record.split(' ',)
    .find(function isField(candidate,) {
      return candidate.startsWith(`${name}=`,);
    },);
  return part === undefined ? '' : decodeURIComponent(part.slice(name.length + 1,),);
}

/**
 Reads the notification log the recorder wrote.
 The log is a count, then records separated by `; `; text inside a record is percent-encoded, so neither a space nor the
 separator can occur in a value.

 @param log - the log text

 @returns the notifications in the order they were sent

 @example
 ```ts
 parseNotificationLog('1 t=5 notify id=1 app=a summary=s body=b',); // [{ app: 'a', summary: 's', body: 'b', }]
 ```
 */
export function parseNotificationLog(log: string,): readonly RecordedNotification[] {
  /**
   The text after the count.
   */
  const records = log.trim()
    .split(' ',)
    .slice(1,)
    .join(' ',);
  if (records === '')
    return [];
  return records.split('; ',)
    .filter(function isNotify(record,) {
      return record.includes(' notify ',);
    },)
    .map(function toNotification(record,): RecordedNotification {
      return {
        app: fieldOf({
          record,
          name: 'app',
        },),
        summary: fieldOf({
          record,
          name: 'summary',
        },),
        body: fieldOf({
          record,
          name: 'body',
        },),
      };
    },);
}
