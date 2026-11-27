/**
 One arm's add of the 2026-11-21 identical-fence control, and the walk that reads what a destination holds.

 Both arms run the same binary through a shim and differ in one thing only: whether the size read starts beside the source
 check. What the add left behind is read here so the control compares two destinations it made itself.
 */
import {
  chmod,
  cp,
  mkdir,
  readFile,
  readdir,
  rm,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { runCommand, } from '../bin/native-verification-process.ts';
import { lastAddTimingRecordOf, } from '../bin/fence-size-read-beside-the-check-2026-11-21-add-timing-record.ts';
import { stderrTailInFailureMessage, } from './fence-size-read-beside-the-check-2026-11-21-commands.ts';
import type {
  FenceReading,
  FenceTreeEntry,
} from '../bin/fence-size-read-beside-the-check-2026-11-21-identical-fence-report.ts';
import { fixtureEnvironment, } from './fence-size-read-beside-the-check-2026-11-21-identical-fence-source.ts';
import {
  BRANCH,
  executableMode,
  RECORD,
  ROOT,
  SOURCE,
  SWITCH,
  verboseFenceLines,
} from './fence-size-read-beside-the-check-2026-11-21-identical-fence-paths.ts';

/**
 What one arm's add left behind, and where it wrote it.
 */
export type ArmAdd = FenceReading & {
  /**
   The destination this arm's add wrote, which the two trees are read from.
   */
  readonly destination: string;
};

/**
 Every regular file under a directory, as path and bytes, for a byte-for-byte comparison.

 One entry per name, and no entry reads another, so a directory's entries are read at the same time; the paths are ordered
 afterwards, which is what makes the two destinations comparable.

 @param directory - the directory to walk

 @param prefix - the path this directory has inside the destination, empty at the destination itself

 @returns one entry per regular file, ordered by path

 @example
 ```ts
 await tree({ directory: serial.destination, prefix: '' });
 ```
 */
export async function tree({
  directory,
  prefix,
}: {
  readonly directory: string;
  readonly prefix: string;
}): Promise<readonly FenceTreeEntry[]> {
  /**
   What this directory holds, and the files of the directories inside it.
   */
  const entries = await Promise.all((await readdir(directory,))
    .filter(function isNotTheGitDirectory(name) {
      return name !== '.git';
    },)
    .map(async function entryOf(name): Promise<readonly FenceTreeEntry[]> {
      /**
       The entry's path on this host.
       */
      const full = join(
        directory,
        name,
      );
      /**
       The entry's path inside the destination.
       */
      const relative = prefix === '' ? name : `${prefix}/${name}`;
      if ((await stat(full,)).isDirectory()) {
        return tree({
          directory: full,
          prefix: relative,
        },);
      }
      return [[
        relative,
        (await readFile(full,)).toString('base64',),
      ],];
    },));
  return entries.flat()
    .toSorted(function byPath(
      left,
      right
    ) {
      return left[0]
        .localeCompare(right[0],);
    },);
}

/**
 Makes one arm's add: a shim of the same binary, the switch set or unset, and one worktree add of the fixture source.

 @param overlap - whether this arm starts the size read beside the source check

 @returns what the add printed, what its destination holds, and the record it appended

 @throws Error when the add did not run, naming what it printed

 @example
 ```ts
 await addWith({ overlap: false });
 ```
 */
export async function addWith({ overlap, }: { readonly overlap: boolean; }): Promise<ArmAdd> {
  /**
   The directory this arm's shim of the binary is put in.
   */
  const shim = join(
    ROOT,
    overlap ? 'shim-overlap' : 'shim-serial',
  );
  await rm(
    shim,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    shim,
    { recursive: true, },
  );
  await cp(
    BRANCH,
    join(
      shim,
      'git',
    ),
  );
  await chmod(
    join(
      shim,
      'git',
    ),
    executableMode,
  );
  /**
   The destination this arm's add writes.
   */
  const destination = join(
    ROOT,
    overlap ? 'dest-overlap' : 'dest-serial',
  );
  await rm(
    destination,
    {
      recursive: true,
      force: true,
    },
  );
  /**
   The environment this arm's add runs with.
   */
  const env: NodeJS.ProcessEnv = {
    ...fixtureEnvironment(),
    PATH: `${shim}:/usr/bin:/bin:/usr/sbin:/sbin`,
    MONOCHROMATIC_VERBOSE: 'true',
    ...(overlap ? { [SWITCH]: '1', } : {}),
  };
  /**
   What the add answered.
   */
  const added = await runCommand({
    command: 'git',
    args: [
      'worktree',
      'add',
      '--quiet',
      '-b',
      `wt-${overlap ? 'overlap' : 'serial'}`,
      destination,
    ],
    cwd: SOURCE,
    env,
    capture: true,
    allowFailure: true,
  },);
  if ((added.status ?? 1) !== 0) throw new Error(added.stderr
    .slice(-stderrTailInFailureMessage,));
  /**
   The line the add printed its fence decision on, or the words saying it printed none.
   */
  const fence = added.stderr
    .split('\n',)
    .find(function carriesTheFence(line) {
      return line.includes('the fence carries',);
    },)
    ?? 'no fence line';
  /**
   What the destination's own status says, which is the worktree the add wrote.
   */
  const status = (await runCommand({
    command: 'git',
    args: [
      'status',
      '--porcelain',
    ],
    cwd: destination,
    env,
    capture: true,
    allowFailure: true,
  },)).stdout;
  /**
   The record the add appended for itself.
   */
  const record = lastAddTimingRecordOf({ text: await readFile(
    RECORD,
    'utf8',
  ), },);
  return {
    fence,
    status,
    destination,
    ...(record.notes
      .route
      === undefined ? {} : { route: record.notes
        .route, }),
    carried: record.steps,
    verbose: added.stderr
      .split('\n',)
      .filter(function namesTheFence(line) {
        return line.includes('fence',);
      },)
      .slice(
      0,
      verboseFenceLines,
      ),
  };
}
