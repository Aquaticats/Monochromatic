/**
 A stand-in control group for the check scope accounting tests: the files a kernel writes there, and the ways one goes.

 The kernel makes a control group with its files already there and takes them away one at a time before the directory, so
 a fixture writes a folder and renames it into the group's name, and takes it away under another name before deleting it.
 A named pipe is the one seam here that no kernel writes: it holds one read of a watch open at a chosen point, so a test
 knows where a reading stood when the group went instead of hoping a loaded host picks that point.

 Nothing in this package reads this module: it is the fixture two test files share (`AGENTS.md`, rule `UTL`).
 */
import {
  mkdir,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { runGateCommand } from './native-gate-run.ts';

/**
 Milliseconds `mkfifo` is given to make one named pipe, which is far more than the call needs.
 */
const namedPipeDeadlineMilliseconds = 5_000;

/**
 Bytes a fixture's `memory.peak` holds, which is the peak a check at its largest was measured at.
 */
export const standInMemoryPeakBytes = 1_953_531_904;

/**
 Bytes a fixture's `memory.max` holds, which is the 4 GiB bound every recorded check ran under.
 */
export const standInMemoryMaxBytes = 4_294_967_296;

/**
 The figures a control group holds after a loaded check.

 These are a stand-in: the shapes of the files are the kernel's (`bin/native-gate-container-cgroup-memory.ts` reads the
 same names), and the numbers are the ones a round measured for `//package/git-policy/cli:test:unit` at 800% on this host
 on 2026-11-27, with the refused forks added because no unbound run of the suite met its task bound.
 */
export const standInCgroupFigures: Readonly<Record<string, string>> = {
  'pids.current': '30',
  'pids.peak': '434',
  'pids.max': '1024',
  'pids.events': 'max 9',
  'pids.events.local': 'max 6',
  'memory.current': '893263872',
  'memory.peak': String(standInMemoryPeakBytes,),
  'memory.max': String(standInMemoryMaxBytes,),
  'memory.events': 'low 0\nhigh 0\nmax 1\noom 5\noom_kill 5\noom_group_kill 0',
  'memory.events.local': 'low 0\nhigh 0\nmax 0\noom 2\noom_kill 2\noom_group_kill 0',
  'cpu.max': '800000 100000',
  'cpu.stat': 'usage_usec 76928181\nuser_usec 54543767\nsystem_usec 22384414\nnr_periods 160\nnr_throttled 68\nthrottled_usec 97294323',
};

/**
 Write one control group's files into a fixture tree, as the kernel would.

 @param cgroupPath - the fixture's control group

 @param figures - every file to write, by name, each holding one line per entry

 @example
 ```ts
 await writeStandInCgroup({ cgroupPath, figures: standInCgroupFigures });
 ```
 */
export async function writeStandInCgroup({
  cgroupPath,
  figures,
}: {
  readonly cgroupPath: string;
  readonly figures: Readonly<Record<string, string>>;
},): Promise<void> {
  // The kernel makes a control group with its files already there, so the fixture does too: a watch that read a half
  // written fixture would read a control group no kernel ever made.
  /**
   The folder this fixture writes before its control group appears.
   */
  const written = `${cgroupPath}.written`;
  await mkdir(
    written,
    { recursive: true, },
  );
  /**
   One write per file of the group, which are made together.
   */
  const writes: Promise<void>[] = [];
  for (const name of Object.keys(figures,)) {
    /**
     What that file holds.
     */
    const text = figures[name];
    /**
     Where that file goes.
     */
    const file = join(
      written,
      name,
    );
    if (text !== undefined)
      writes.push(writeFile(
        file,
        `${text}\n`,
      ),);
  }
  await Promise.all(writes,);
  await rename(
    written,
    cgroupPath,
  );
}

/**
 End one fixture control group, which is what a scope that ends does to its own.

 A scope's control group goes as one path, so the fixture takes its folder away under another name before it deletes it:
 a reader then sees a scope that is gone rather than a control group being deleted file by file.

 @param cgroupPath - the fixture's control group

 @example
 ```ts
 await removeStandInCgroup({ cgroupPath });
 ```
 */
export async function removeStandInCgroup({ cgroupPath }: { readonly cgroupPath: string; },): Promise<void> {
  await rename(
    cgroupPath,
    `${cgroupPath}.gone`,
  );
  await rm(
    `${cgroupPath}.gone`,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Make a named pipe where one of a control group's files is, which holds one read open at a chosen point.

 The reader of a named pipe waits inside its open until something opens the other end, and then reads until that end
 closes, so a test that opens the other end knows the reading has reached this file.

 @param path - where the pipe goes

 @throws Error when `mkfifo` did not make the pipe

 @example
 ```ts
 makeStandInNamedPipe({ path: join(cgroupPath, 'pids.events.local') });
 ```
 */
export async function makeStandInNamedPipe({ path }: { readonly path: string; },): Promise<void> {
  /**
   What `mkfifo` answered.
   */
  const made = await runGateCommand({
    program: 'mkfifo',
    args: [path,],
    deadlineMilliseconds: namedPipeDeadlineMilliseconds,
  },);
  if (made.status !== 0)
    throw new Error(`mkfifo could not make ${path}: ${made.problem ?? made.stderr}`,);
}
