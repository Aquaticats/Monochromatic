/**
 The daily update's real effects over git, files and the desktop.

 @module
 */

import spawn from 'nano-spawn';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';

import { runCommandBounded, } from './daily-update-command.ts';
import type { DailyUpdateEffects, } from './daily-update-run.ts';

/**
 Length of the two status letters and the space before the path in a `git status --porcelain=v1 -z` entry.
 */
export const PORCELAIN_PREFIX_LENGTH = 3;

/**
 Runs `git` in the repository.

 @param repository - absolute repository path

 @param args - arguments after `git -C <repository>`

 @returns standard output text

 @throws SubprocessError when git exits nonzero

 @example
 ```ts
 const names = await git({ repository: '/r', args: ['ls-files', '-z',], },);
 ```
 */
export async function git({
  repository,
  args,
}: {
  readonly repository: string;
  readonly args: readonly string[];
},): Promise<string> {
  /**
   What git printed.
   */
  const result = await spawn(
    'git',
    [
      '-C',
      repository,
      ...args,
    ],
  );
  return result.stdout;
}

/**
 Splits NUL-separated git output.

 @param output - text from a `-z` command

 @returns the non-empty entries

 @example
 ```ts
 splitNul('a\0b\0',); // ['a', 'b']
 ```
 */
export function splitNul(output: string,): readonly string[] {
  return output.split('\0',)
    .filter(function isEntry(entry,) {
      return entry !== '';
    },);
}

/**
 The environment the desktop notifier runs with: the caller's, with the session bus address exactly as the caller has
 it. An address the caller does not have is never invented, so a process that was given no bus (a test, a fixture) can
 never reach the owner's desktop by a guessed path; the timer's unit sets the address itself.

 @param environment - the caller's environment

 @returns a copy to give the notifier

 @example
 ```ts
 notifierEnvironment({ environment: { DBUS_SESSION_BUS_ADDRESS: 'unix:path=/run/user/1000/bus', }, },);
 ```
 */
export function notifierEnvironment({
  environment,
}: {
  readonly environment: NodeJS.ProcessEnv;
},): NodeJS.ProcessEnv {
  return { ...environment, };
}

/**
 Sends a desktop notification and writes the same text to the journal.

 @param title - notification title

 @param body - notification text

 @example
 ```ts
 await notifyOwner({ title: 'Daily toolchain update needs a look', body: 'nothing landed', },);
 ```
 */
export async function notifyOwner({
  title,
  body,
}: {
  readonly title: string;
  readonly body: string;
},): Promise<void> {
  process.stderr
    .write(`${title}\n${body}\n`,);
  /**
   Maximum characters of the body shown on the desktop; the journal has all of it.
   */
  const shown = body.slice(
    0,
    NOTIFICATION_BODY_CHARACTERS,
  );
  try {
    await spawn(
      'notify-send',
      [
        '--app-name',
        'toolchain-daily-update',
        '--icon',
        'dialog-warning',
        title,
        shown,
      ],
      {
        env: notifierEnvironment({ environment: process.env, },),
        timeout: NOTIFY_TIMEOUT_MS,
      },
    );
  }
  catch (error) {
    // A missing desktop must not turn a reported problem into a crash; the journal already has the text.
    process.stderr
      .write(
      `daily-update: no desktop notification was shown: ${String(error,)} ${
          (Error.isError(error,)) && ('stderr' in error) ? String(error.stderr,) : ''
        }\n`,
      );
  }
}

/**
 Longest `notify-send` may take, in milliseconds.
 GLib's D-Bus calls wait 25 seconds for an answer at most, which is the longest `notify-send` can wait on a bus whose
 service does not answer; measured 2026-11-07: 0.6 seconds against a private bus with no notification service.
 */
export const NOTIFY_TIMEOUT_MS = 30_000;

/**
 Characters of a notification body that are sent to the desktop.
 */
export const NOTIFICATION_BODY_CHARACTERS = 1_000;

/**
 Reads a file's text.

 @param absolutePath - absolute path

 @returns the text

 @example
 ```ts
 await readUtf8('/etc/hostname',);
 ```
 */
export function readUtf8(absolutePath: string,): Promise<string> {
  return readFile(
    absolutePath,
    'utf8',
  );
}

/**
 Replaces a file's text.

 @param absolutePath - absolute path of the file

 @param text - new contents

 @example
 ```ts
 await writeUtf8({ absolutePath: '/r/a', text: 'x', },);
 ```
 */
export function writeUtf8({
  absolutePath,
  text,
}: {
  readonly absolutePath: string;
  readonly text: string;
},): Promise<void> {
  return writeFile(
    absolutePath,
    text,
  );
}

/**
 Writes one line to the journal.

 @param line - text without a newline

 @example
 ```ts
 logLine('running: mise run toolchain:update',);
 ```
 */
export function logLine(line: string,): void {
  process.stderr
    .write(`daily-update: ${line}\n`,);
}

/**
 Builds the real effects for a repository.

 @param repository - absolute repository path

 @returns the effects

 @example
 ```ts
 const effects = realEffects('/var/home/user/Monochromatic',);
 ```
 */
export function realEffects(repository: string,): DailyUpdateEffects {
  return {
    repository,
    runCommand: function runInRepository(command,) {
      return runCommandBounded({
        repository,
        command,
      },);
    },
    dirtyPaths: async function dirtyPaths() {
      return splitNul(await git({
        repository,
        args: [
          'status',
          '--porcelain=v1',
          '-z',
        ],
      },),)
        .map(
        function pathOf(entry,) {
          return entry.slice(PORCELAIN_PREFIX_LENGTH,);
        },
        );
    },
    trackedPaths: async function trackedPaths() {
      return splitNul(await git({
        repository,
        args: [
          'ls-files',
          '-z',
        ],
      },),);
    },
    readText: readUtf8,
    writeText: writeUtf8,
    commit: async function commit({
      message,
      paths,
    },) {
      await git({
        repository,
        args: [
          'commit',
          '--message',
          message,
          '--',
          ...paths,
        ],
      },);
    },
    headCommit: async function headCommit() {
      return (await git({
        repository,
        args: [
          'rev-parse',
          'HEAD',
        ],
      },)).trim();
    },
    notify: notifyOwner,
    log: logLine,
  };
}
