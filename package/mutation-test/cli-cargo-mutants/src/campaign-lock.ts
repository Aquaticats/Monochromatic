import {
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { CargoMutantsToolingError } from './error.ts';
import { isMissing } from './fs-guards.ts';

/**
 Name of the lock file in a campaign's evidence directory.
 */
const lockFileName = 'campaign.lock';

/**
 Field of `/proc/<pid>/stat` that holds the process start time in clock ticks since boot, counted from 1.
 */
const startTimeField = 22;

/**
 Fields of `/proc/<pid>/stat` that precede the one after the command name, counted from 1: the process ID and the name.
 */
const fieldsBeforeState = 2;

/**
 Name a process is known by for as long as it runs, or nothing when it is gone.

 The boot ID and the start time together tell a process from a later one that reuses its number, also across a reboot.

 @param pid - process number

 @returns `<boot ID>:<start ticks>`, or the empty text when no such process runs

 @example
 ```ts
 await processIdentity(process.pid);
 ```
 */
async function processIdentity(pid: number): Promise<string> {
  try {
    /**
     Text of the process's stat line.
     */
    const stat = await readFile(
      `/proc/${String(pid)}/stat`,
      'utf8',
    );
    /**
     Fields after the command name, which may itself hold spaces and parentheses.
     */
    const afterName = stat.slice(stat.lastIndexOf(')') + 2)
      .split(' ');
    return `${(await readFile(
      '/proc/sys/kernel/random/boot_id',
      'utf8',
    )).trim()}:${afterName[startTimeField - fieldsBeforeState
      - 1] ?? ''}`;
  }
  catch (error) {
    // The process has exited (or this is not Linux, where no campaign can run).
    console.error(`No identity for process ${String(pid)}: ${String(error)}`);
    return '';
  }
}

/**
 Check whether the process that wrote a lock file still runs.

 @param lockPath - lock file path

 @returns true when the recorded process runs with the recorded identity

 @example
 ```ts
 await lockHolderRuns('/e/campaign.lock');
 ```
 */
async function lockHolderRuns(lockPath: string): Promise<boolean> {
  try {
    /**
     Recorded process number and identity.
     */
    const [pidText, identity] = (await readFile(
      lockPath,
      'utf8',
    )).trim()
      .split(' ');
    return (identity !== undefined) && (identity !== '')
      && (await processIdentity(Number(pidText)) === identity);
  }
  catch (error) {
    // An absent or unreadable lock file names no live holder; only an unreadable one is worth a line.
    if (!isMissing(error))
      console.error(`Unreadable campaign lock ${lockPath}: ${String(error)}`);
    return false;
  }
}

/**
 Take the right to run one campaign, so that no second run writes the same evidence directory.

 A lock left by a killed run, or by a run from before a reboot, names a process that no longer exists and is taken
 over.

 @param evidence - the campaign's evidence directory

 @returns the held lock; declare it with `await using` so it is released when the campaign ends

 @throws CargoMutantsToolingError when another run of the same campaign is alive

 @example
 ```ts
 await using lock = await acquireCampaignLock('/e/native-mutation-0123456789ab');
 ```
 */
export async function acquireCampaignLock(evidence: string): Promise<AsyncDisposable> {
  /**
   Lock file path.
   */
  const lockPath = join(
    evidence,
    lockFileName,
  );
  if (await lockHolderRuns(lockPath))
    throw new CargoMutantsToolingError(`Another run of this campaign is alive (${lockPath} names its process); wait for it or stop it, then run this again.`);
  /**
   Record of this process.
   */
  const record = `${String(process.pid)} ${await processIdentity(process.pid)}\n`;
  await writeFile(
    lockPath,
    record,
  );
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        lockPath,
        { force: true },
      );
    },
  };
}
