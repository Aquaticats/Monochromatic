/**
 The external worktrees: plain folders (under `~/worktrees`) that `btrfs send` cannot move. Each gets a new subvolume on
 the 860 and a root `rsync` into it in two passes.

 Pass two copies again without deleting anything, then lists with a dry-run `rsync --delete` every file that was
 deleted at the source after pass one. The list is a report for the owner; the copy keeps those files.

 @module
 */

import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  createRecords,
  pathExists,
  writeTextAtomically,
} from './completion-records.ts';
import {
  EXTERNAL_RSYNC_ARGUMENTS,
  STATE_FOLDER_MODE,
} from './constants.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';
import { isPlainName, } from './text-checks.ts';

/**
 Marker of a file that only the destination has, in `rsync --itemize-changes` output.
 */
const DELETING_MARKER = '*deleting';

/**
 The plain folder names directly below the external source.

 @param source - the external source folder
 
 @returns the sorted names
 
 @throws MoveStopError when a name has a character this tool does not quote into commands
 */
async function externalNames(source: string,): Promise<readonly string[]> {
  /**
   The entries of this step.
   */
  const entries = await readdir(
    source,
    { withFileTypes: true, },
  );
  return entries.filter(function isFolder(entry,) {
    return entry.isDirectory();
  },)
    .map(function nameOf(entry,) {
      return entry.name;
    },)
    .toSorted()
    .map(function requirePlain(name,) {
      if (!isPlainName(name,)) {
        throw new MoveStopError(`external worktree folder name has a character this tool does not handle: ${name}`,);
      }
      return name;
    },);
}

/**
 Runs one `rsync` of an external worktree.

 @param options - the options
 
 @param runner - the runner
 
 @param name - the worktree folder name
 
 @param extra - arguments after the shared ones
 
 @returns what `rsync` printed
 */
async function rsyncCopy(
  {
    options,
    runner,
    name,
    extra,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
    readonly name: string;
    readonly extra: readonly string[];
  },
): Promise<string> {
  return await runner.exec({
    command: 'rsync',
    args: [
      ...EXTERNAL_RSYNC_ARGUMENTS,
      ...extra,
      `${join(
        options.externalSource,
        name,
      )}/`,
      `${join(
        options.externalFinal,
        name,
      )}/`,
    ],
    transfer: true,
    deadlineSeconds: options.externalDeadlineSeconds,
  },);
}

/**
 Reports the files deleted at the source after pass one, from a dry-run `rsync --delete`, and keeps the copy as it is.

 @param options - the options
 
 @param runner - the runner
 
 @param name - the worktree folder name
 */
async function reportDeletions(
  {
    options,
    runner,
    name,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
    readonly name: string;
  },
): Promise<void> {
  /**
   The itemized of this step.
   */
  const itemized = await rsyncCopy({
    options,
    runner,
    name,
    extra: [
      '--dry-run',
      '--delete',
      '--itemize-changes',
    ],
  },);
  /**
   The deleted of this step.
   */
  const deleted = itemized.split('\n',)
    .filter(function isDeletion(line,) {
      return line.startsWith(DELETING_MARKER,);
    },)
    .map(function pathOf(line,) {
      return line.slice(DELETING_MARKER.length,)
        .trim();
    },);
  if (options.run) {
    await writeTextAtomically({
      path: join(
        options.stateFolder,
        `external-${name}.deleted.txt`,
      ),
      text: deleted.map(function withNewline(path,) {
        return `${path}\n`;
      },)
        .join('',),
    },);
  }
  runner.log(
    `deleted at the source after pass one: ${
      String(deleted.length,)
    } in ${name}; the copy keeps them, nothing was deleted`,
  );
}

/**
 Runs pass one or pass two over the external worktrees.

 @param options - the options; `pass` is `external-first` or `external-final`
 
 @param runner - the runner
 
 @throws MoveStopError when a source or destination folder is missing, a name is unsafe, or a command fails

 @example
 ```ts
 import { runExternalPass, } from './external-pass.ts';
 
 await runExternalPass({ options, runner, },);
 ```
 */
export async function runExternalPass(
  {
    options,
    runner,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
  },
): Promise<void> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    externalSource: source,
    externalFinal: destination,
  } = options;
  if ((source === '') || (destination === '')) {
    throw new MoveStopError('the external passes need --external-source= and --external-final=',);
  }
  if ((!await pathExists(source,)) || (options.run && (!await pathExists(destination,)))) {
    throw new MoveStopError(`${await pathExists(source,) ? destination : source} does not exist`,);
  }
  /**
   The run's record store.
   */
  const records = createRecords({
    stateFolder: options.stateFolder,
    writable: options.run,
  },);
  /**
   The step of this step.
   */
  const step = options.pass === 'external-first' ? 'first' : 'final';
  if (options.run) {
    await mkdir(
      options.stateFolder,
      {
        recursive: true,
        mode: STATE_FOLDER_MODE,
      },
    );
  }
  for (const name of await externalNames(source,)) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- external folders are copied one at a time, each recorded before the next, under one total deadline; see doc/lint/sequential-awaits.md
    if (await records.has({
      name: `external-${name}`,
      step,
    },)) {
      continue;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- external folders are copied one at a time, each recorded before the next, under one total deadline; see doc/lint/sequential-awaits.md
    if (!await pathExists(join(
      destination,
      name,
    ),)) {
      if (step === 'final') {
        runner.log(`${name} has no first-pass copy: it is created now and copied in full`,);
      }
      // oxlint-disable-next-line eslint/no-await-in-loop -- external folders are copied one at a time, each recorded before the next, under one total deadline; see doc/lint/sequential-awaits.md
      await runner.exec({
        command: 'btrfs',
        args: [
          'subvolume',
          'create',
          join(
            destination,
            name,
          ),
        ],
      },);
    }
    runner.log(`external ${name}`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- external folders are copied one at a time, each recorded before the next, under one total deadline; see doc/lint/sequential-awaits.md
    await rsyncCopy({
      options,
      runner,
      name,
      extra: [],
    },);
    if (step === 'final') {
      // oxlint-disable-next-line eslint/no-await-in-loop -- external folders are copied one at a time, each recorded before the next, under one total deadline; see doc/lint/sequential-awaits.md
      await reportDeletions({
        options,
        runner,
        name,
      },);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- external folders are copied one at a time, each recorded before the next, under one total deadline; see doc/lint/sequential-awaits.md
    await records.write({
      name: `external-${name}`,
      step,
      data: { name, },
    },);
  }
  runner.log(options.run ? `pass ${options.pass} finished` : 'print mode: nothing was run',);
}
