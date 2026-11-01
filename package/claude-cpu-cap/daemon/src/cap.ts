import { writeFile, } from 'node:fs/promises';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  ABSENT,
  DEFAULT_THREADS,
  PERIOD_USEC,
} from './constants.ts';
import { readTextOrEmpty, } from './files.ts';

/**
 Result of one cap reconciliation.
 */
export type CapResult = Readonly<{
  /**
   Thread count the cap file asks for (or the default).
   */
  threads: number;
  /**
   Whether the cgroup `cpu.max` was rewritten in this pass.
   */
  applied: boolean;
}>;

/**
 Reads the wanted thread count; an absent, empty or invalid file yields the default.

 @param capFile - file holding one integer
 
 @param totalThreads - hardware thread count, the upper bound
 
 @param l - parent logger

 @returns the thread count

 @example
 ```ts
 await desiredThreads({ capFile, totalThreads: 32, l, },); // 28 when the file holds "28"
 ```
 */
export async function desiredThreads({
  capFile,
  totalThreads,
  l,
}: Readonly<{
  capFile: string;
  totalThreads: number;
  l: Logger;
}>,): Promise<number> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: desiredThreads.name,
    l,
  },);
  /**
   Contents of the cap file, empty when it does not exist.
   */
  const text = (await readTextOrEmpty(capFile,)).trim();
  if (text === ABSENT) {
    rl.debug(`cap file ${capFile} absent or empty; using ${DEFAULT_THREADS}`,);
    return DEFAULT_THREADS;
  }
  /**
   The file's integer, NaN when it holds anything else.
   */
  const parsed = Number(text,);
  if ((!Number.isInteger(parsed,)) || (parsed < 1)
    || (parsed > totalThreads)) {
    rl.warn(`CAP-FILE-INVALID ${capFile} must hold an integer from 1 to ${totalThreads}; using ${DEFAULT_THREADS}`,);
    return DEFAULT_THREADS;
  }
  return parsed;
}

/**
 Writes the quota for the wanted thread count into the cgroup when it differs.
 Claude Code recreates the cgroup in every boot and has no CPU limit setting, so this runs on every pass.

 @param capFile - file holding the wanted thread count
 
 @param cgroupFile - the cgroup's `cpu.max`
 
 @param totalThreads - hardware thread count
 
 @param l - parent logger

 @returns the wanted thread count and whether `cpu.max` was rewritten

 @example
 ```ts
 await reconcileCap({ capFile, cgroupFile, totalThreads: 32, l, },); // { threads: 28, applied: true, }
 ```
 */
export async function reconcileCap({
  capFile,
  cgroupFile,
  totalThreads,
  l,
}: Readonly<{
  capFile: string;
  cgroupFile: string;
  totalThreads: number;
  l: Logger;
}>,): Promise<CapResult> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: reconcileCap.name,
    l,
  },);
  /**
   Thread count the cap file asks for.
   */
  const threads = await desiredThreads({
    capFile,
    totalThreads,
    l: rl,
  },);
  /**
   Quota text for `cpu.max`: quota then period in microseconds.
   */
  const wanted = `${threads * PERIOD_USEC} ${PERIOD_USEC}`;
  /**
   Quota text now in the cgroup, empty when the cgroup does not exist yet.
   */
  const current = (await readTextOrEmpty(cgroupFile,)).trim();
  if (current === ABSENT) {
    rl.debug(`cgroup ${cgroupFile} absent; nothing to cap yet`,);
    return {
      threads,
      applied: false,
    };
  }
  if (current === wanted) return {
    threads,
    applied: false,
  };
  await writeFile(
    cgroupFile,
    wanted,
  );
  rl.info(`CAP-APPLIED claude-code-bash cpu.max "${current}" -> "${wanted}" (${threads} of ${totalThreads} threads)`,);
  return {
    threads,
    applied: true,
  };
}
