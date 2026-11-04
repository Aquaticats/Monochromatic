/**
 Waiting for the commit service to finish the after-commit work of a repository.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setInterval as ticks, } from 'node:timers/promises';

import { isAbsent, } from './service-connection-fixture.ts';

//region Constants

/**
 Pause between two polls, in milliseconds.
 */
const POLL_MILLISECONDS = 10;

/**
 How long the observation waits for the service to finish a repository's after-commit work, in milliseconds.
 */
const SETTLE_DEADLINE_MILLISECONDS = 30_000;

//endregion Constants

//region Settlement

/**
 Names in a directory, none when the directory does not exist.

 @param directory - directory to list

 @returns the entry names

 @throws the read's failure when it is not a missing directory
 */
async function namesIn(directory: string,): Promise<readonly string[]> {
  try {
    return await readdir(directory,);
  }
  catch (error) {
    // A repository the service never saw has no work directory; any other failure is real.
    if (isAbsent(error,))
      return [];
    throw error;
  }
}

/**
 Waits until the service has finished the after-commit work of one repository.

 A commit the service lands returns before its push: the wrapper writes a push record and a maintenance record, and the
 service runs them on its own thread. The remote therefore reaches the landed commit some time after the exit, so the
 observation that compares the remote with the local branch waits here first. A record that failed counts as finished,
 so the remote check reports the missing push instead of this wait hiding it.

 @param gitDir - the worktree's own Git directory, which holds its work records

 @returns the records still pending, claimed or being recovered after the deadline, as `<state>/<name>`;
 empty when the service finished everything.
 A claim a killed service held is taken back by a drain once its lease is over, which is longer than this wait,
 so a scenario that kills the service while it holds a claim waits with {@link waitForServiceWork} first.

 @example
 ```ts
 const unsettled = await settleServiceWork(repository.gitDir);
 ```
 */
export async function settleServiceWork(gitDir: string,): Promise<readonly string[]> {
  return await waitForServiceWork({
    gitDir,
    deadlineMilliseconds: SETTLE_DEADLINE_MILLISECONDS,
  },);
}

/**
 Waits until the service has finished the after-commit work of one repository, for as long as the caller allows.

 @param gitDir - the worktree's own Git directory, which holds its work records

 @param deadlineMilliseconds - how long to wait for the records to leave `pending`, `claimed` and `recovering`

 @returns the records still pending, claimed or being recovered after the deadline, as `<state>/<name>`;
 empty when the service finished everything

 @example
 ```ts
 const unsettled = await waitForServiceWork({ gitDir: repository.gitDir, deadlineMilliseconds: budget });
 ```
 */
export async function waitForServiceWork({
  gitDir,
  deadlineMilliseconds,
}: Readonly<{
  gitDir: string;
  deadlineMilliseconds: number;
}>,): Promise<readonly string[]> {
  /**
   Directory of the repository's after-commit records.
   */
  const workDirectory = join(
    gitDir,
    'cli-git',
    'work',
  );
  /**
   When the wait for the records ends.
   */
  const deadline = Date.now() + deadlineMilliseconds;
  for await (const _tick of ticks(POLL_MILLISECONDS,)) {
    /**
     Records waiting for the service and records it is working on.
     */
    const [
      pending,
      claimed,
      recovering,
    ] = await Promise.all([
      namesIn(join(
        workDirectory,
        'pending',
      ),),
      namesIn(join(
        workDirectory,
        'claimed',
      ),),
      namesIn(join(
        workDirectory,
        'recovering',
      ),),
    ],);
    /**
     Whether no item of the work directory is still pending, claimed or being recovered.
     */
    const nothingOutstanding = [
      pending,
      claimed,
      recovering,
    ].every(function isEmpty(names,) {
      return names.length === 0;
    },);
    if (nothingOutstanding)
      return [];
    if (Date.now() >= deadline)
      return [
        ...pending.map(function pendingName(name,) {
          return `pending/${name}`;
        },),
        ...claimed.map(function claimedName(name,) {
          return `claimed/${name}`;
        },),
        ...recovering.map(function recoveringName(name,) {
          return `recovering/${name}`;
        },),
      ];
  }
  return [];
}

//endregion Settlement
