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
 Read a `/proc` link, treating a vanished process or an unreadable link as no target.

 @param path - the link

 @returns the target, or undefined when it cannot be read
 */
async function targetOf(path: string): Promise<string | undefined> {
  try {
    return await readlink(path);
  }
  catch {
    return undefined;
  }
}

/**
 The first place where one process touches the tree, if it does.

 @param pid - process to inspect

 @param root - the scratch tree

 @returns a short description of the touch, or undefined when the process does not touch the tree
 */
async function touchOf({
  pid,
  root,
}: {
  readonly pid: number;
  readonly root: string;
}): Promise<string | undefined> {
  /**
   Directory of the process in the process table.
   */
  const base = join(
    '/proc',
    String(pid),
  );
  for (const link of [
    'cwd',
    'exe',
    'root',
  ]) {
    /**
     Where the link points.
     */
    const target = await targetOf(join(
      base,
      link,
    ));
    if (target !== undefined && isInside({
      target,
      root,
    }))
      return `${link} ${target}`;
  }
  /**
   Descriptor names of the process; an unreadable table means a process that is gone or not ours.
   */
  let descriptors: string[];
  try {
    descriptors = await readdir(join(
      base,
      'fd',
    ));
  }
  catch {
    return undefined;
  }
  for (const descriptor of descriptors) {
    /**
     Where the descriptor points.
     */
    const target = await targetOf(join(
      base,
      'fd',
      descriptor,
    ));
    if (target !== undefined && isInside({
      target,
      root,
    }))
      return `fd ${descriptor} ${target}`;
  }
  return undefined;
}

/**
 Every process other than this one that touches the tree.

 @param root - the scratch tree, as a canonical absolute path

 @returns the holders, in process-number order
 */
export async function findHolders(root: string): Promise<readonly Holder[]> {
  /**
   Names in the process table that are process numbers.
   */
  const pids = (await readdir('/proc'))
    .filter(function isNumber(name) {
      return /^[0-9]+$/u.test(name);
    })
    .map(Number)
    .filter(function isOther(pid) {
      return pid !== process.pid;
    })
    .toSorted(function ascending(left, right) {
      return left - right;
    });
  /**
   Holders found so far.
   */
  const holders: Holder[] = [];
  for (const pid of pids) {
    /**
     Where this process touches the tree.
     */
    const via = await touchOf({
      pid,
      root,
    });
    if (via === undefined)
      continue;
    /**
     Command line with its separators made visible, or the process number when the process just ended.
     */
    let command = String(pid);
    try {
      command = (await readFile(
        join(
          '/proc',
          String(pid),
          'cmdline',
        ),
        'utf8',
      )).replaceAll(
        '\0',
        ' ',
      ).trim();
    }
    catch {
      // The process ended between the scan and the read; its number still names it.
    }
    holders.push({
      pid,
      command,
      via,
    });
  }
  return holders;
}

/**
 Remove the scratch tree once no other process touches it.

 @param root - the scratch tree, as a canonical absolute path

 @param deadlineMilliseconds - how long to wait for holders before failing

 @throws Error when a process still touches the tree at the deadline, naming each process, its command and what it holds,
   or when the removal itself fails; neither is ignored, because a tree left behind or half removed is a defect to see
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
   Time after which the wait ends.
   */
  const deadline = Date.now() + deadlineMilliseconds;
  /**
   Holders at the latest scan.
   */
  let holders = await findHolders(canonicalRoot);
  while (holders.length > 0 && Date.now() < deadline) {
    await pause(SCAN_INTERVAL_MILLISECONDS);
    holders = await findHolders(canonicalRoot);
  }
  if (holders.length > 0) {
    throw new Error(
      `${String(holders.length)} process(es) still touch ${root} after ${String(deadlineMilliseconds)} ms, so it was not removed: ${
        holders.map(function describe(holder) {
          return `${String(holder.pid)} (${holder.command}) holds ${holder.via}`;
        }).join('; ')
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
