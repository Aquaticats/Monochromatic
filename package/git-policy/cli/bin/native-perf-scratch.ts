/**
 Removal of the latency benchmark's scratch tree without racing a process that still writes in it.

 Git detaches background work from the command that started it: `git commit` runs `git maintenance run --auto --quiet
 --detach`, and the detached run keeps the repository's directories open after the commit returned and the harness saw
 the process close. A recursive removal that starts while such a process is alive can list a directory, delete what it
 listed, and find the directory not empty again (`ENOTEMPTY`) because the process wrote more in between.
 */
import {
  readdir,
  readFile,
  readlink,
  realpath,
  rm,
} from 'node:fs/promises';
import {
  join,
  sep,
} from 'node:path';
import { setTimeout as pause } from 'node:timers/promises';

/**
 How long cleanup waits for processes that hold the scratch tree before it fails, in milliseconds.
 */
export const DEFAULT_HOLDER_DEADLINE_MILLISECONDS = 30_000;

/**
 Pause between two scans of the process table, in milliseconds.
 */
const SCAN_INTERVAL_MILLISECONDS = 50;

/**
 One process that has the scratch tree as its working directory, its executable, or an open file.
 */
export type Holder = {
  readonly pid: number;
  readonly command: string;
  readonly via: string;
};

/**
 Whether a link target lies in the tree, counting the tree itself.

 @param target - what a `/proc` link points at

 @param root - the scratch tree

 @returns true when the target is the tree or below it
 */
function isInside({
  target,
  root,
}: {
  readonly target: string;
  readonly root: string;
}): boolean {
  return (target === root) || target.startsWith(`${root}${sep}`);
}

/**
 Whether a failure to read a `/proc` entry is the ordinary case of a process that ended or is not ours.

 @param error - what the read threw

 @returns true for the codes a vanished or foreign process gives

 @example
 ```ts
 isOrdinaryProcessFailure(new Error('x'));
 ```
 */
function isOrdinaryProcessFailure(error: unknown): boolean {
  return Error.isError(error) && ('code' in error)
    && [
      'ENOENT',
      'ESRCH',
      'EACCES',
      'EPERM'
    ].includes(String(error.code));
}

/**
 Read a `/proc` link, treating a vanished process or an unreadable link as no target.

 @param path - the link

 @returns the target as the only element, or no element when it cannot be read

 @example
 ```ts
 await targetOf('/proc/1/cwd');
 ```
 */
async function targetOf(path: string): Promise<readonly string[]> {
  try {
    return [await readlink(path)];
  }
  catch (error) {
    if (!isOrdinaryProcessFailure(error))
      console.error(`Cannot read the link ${path}: ${String(error)}`);
    return [];
  }
}

/**
 Read the descriptor names of a process; an unreadable table means a process that is gone or not ours.

 @param base - the process's directory in the process table

 @returns the names, or none

 @example
 ```ts
 await descriptorsOf('/proc/1');
 ```
 */
async function descriptorsOf(base: string): Promise<readonly string[]> {
  try {
    return await readdir(join(
      base,
      'fd',
    ));
  }
  catch (error) {
    if (!isOrdinaryProcessFailure(error))
      console.error(`Cannot list the descriptors of ${base}: ${String(error)}`);
    return [];
  }
}

/**
 The first place where one process touches the tree, if it does.

 @param pid - process to inspect

 @param root - the scratch tree

 @returns a short description of the touch as the only element, or no element when the process does not touch the tree

 @example
 ```ts
 await touchOf({ pid: 1, root: '/scratch' });
 ```
 */
async function touchOf({
  pid,
  root,
}: {
  readonly pid: number;
  readonly root: string;
}): Promise<readonly string[]> {
  /**
   Directory of the process in the process table.
   */
  const base = join(
    '/proc',
    String(pid),
  );
  /**
   The three links that place a process in the tree, with their targets inside it.
   */
  const links = (await Promise.all([
    'cwd',
    'exe',
    'root'
  ].map(async function inTree(link) {
    return (await targetOf(join(
      base,
      link,
    ))).filter(function isUnderRoot(target) {
      return isInside({
        target,
        root,
      });
    })
      .map(function describe(target) {
        return `${link} ${target}`;
      });
  }))).flat();
  if (links.length > 0)
    return links.slice(
      0,
      1,
    );
  /**
   Descriptor names of the process.
   */
  const descriptors = await descriptorsOf(base);
  return (await Promise.all(descriptors.map(async function inTree(descriptor) {
    return (await targetOf(join(
      base,
      'fd',
      descriptor,
    ))).filter(function isUnderRoot(target) {
      return isInside({
        target,
        root,
      });
    })
      .map(function describe(target) {
        return `fd ${descriptor} ${target}`;
      });
  }))).flat()
    .slice(
    0,
    1,
    );
}

/**
 Read the command line of a process with its separators made visible.

 @param pid - the process

 @returns the command line, or the process number when the process just ended

 @example
 ```ts
 await commandOf(1);
 ```
 */
async function commandOf(pid: number): Promise<string> {
  try {
    return (await readFile(
      join(
        '/proc',
        String(pid),
        'cmdline',
      ),
      'utf8',
    )).replaceAll(
      '\0',
      ' ',
    )
      .trim();
  }
  catch (error) {
    // The process ended between the scan and the read; its number still names it.
    if (!isOrdinaryProcessFailure(error))
      console.error(`Cannot read the command line of ${String(pid)}: ${String(error)}`);
    return String(pid);
  }
}

/**
 Every process other than this one that touches the tree.

 @param root - the scratch tree, as a canonical absolute path

 @returns the holders, in process-number order

 @example
 ```ts
 await findHolders('/home/user/temp/agent/scratch');
 ```
 */
export async function findHolders(root: string): Promise<readonly Holder[]> {
  /**
   Names in the process table that are process numbers.
   */
  const pids = (await readdir('/proc'))
    .filter(function isNumber(name) {
      return (Number(name) > 0) && (String(Number(name)) === name);
    })
    .map(Number)
    .filter(function isOther(pid) {
      return pid !== process.pid;
    })
    .toSorted(function ascending(
      left,
      right
    ) {
      return left - right;
    });
  return (await Promise.all(pids.map(async function holderOf(pid): Promise<readonly Holder[]> {
    /**
     Where this process touches the tree, as the only element, or none.
     */
    const [via] = await touchOf({
      pid,
      root,
    });
    return via === undefined ? [] : [{
      pid,
      command: await commandOf(pid),
      via,
    }];
  }))).flat();
}

/**
 Scan the process table until no process touches the tree or the deadline passes.

 @param root - the scratch tree, as a canonical absolute path

 @param deadline - time after which the wait ends, in milliseconds since the epoch

 @returns the holders at the last scan

 @example
 ```ts
 await waitForNoHolders({ root: '/scratch', deadline: Date.now() + 1000 });
 ```
 */
async function waitForNoHolders({
  root,
  deadline,
}: {
  readonly root: string;
  readonly deadline: number;
}): Promise<readonly Holder[]> {
  /**
   Holders at this scan.
   */
  const holders = await findHolders(root);
  if ((holders.length === 0) || (Date.now() >= deadline))
    return holders;
  await pause(SCAN_INTERVAL_MILLISECONDS);
  return await waitForNoHolders({
    root,
    deadline,
  });
}

/**
 Remove the scratch tree once no other process touches it.

 @param root - the scratch tree, as a canonical absolute path

 @param deadlineMilliseconds - how long to wait for holders before failing

 @throws Error when a process still touches the tree at the deadline, naming each process, its command and what it holds,
   or when the removal itself fails; neither is ignored, because a tree left behind or half removed is a defect to see

 @example
 ```ts
 await removeScratchRoot({ root: '/home/user/temp/agent/scratch' });
 ```
 */
export async function removeScratchRoot({
  root,
  deadlineMilliseconds = DEFAULT_HOLDER_DEADLINE_MILLISECONDS,
}: {
  readonly root: string;
  readonly deadlineMilliseconds?: number;
}): Promise<void> {
  /**
   The tree as the process table spells it: links there hold canonical paths, and the home directory may be a link.
   */
  const canonicalRoot = await realpath(root);
  /**
   Holders when the wait ended.
   */
  const holders = await waitForNoHolders({
    root: canonicalRoot,
    deadline: Date.now() + deadlineMilliseconds,
  });
  if (holders.length > 0) {
    throw new Error(
      `${String(holders.length)} process(es) still touch ${root} after ${String(deadlineMilliseconds)} ms, so it was not removed: ${
        holders.map(function describe(holder) {
          return `${String(holder.pid)} (${holder.command}) holds ${holder.via}`;
        })
          .join('; ')
      }`,
    );
  }
  await rm(
    root,
    {
      recursive: true,
      force: true,
    },
  );
}
