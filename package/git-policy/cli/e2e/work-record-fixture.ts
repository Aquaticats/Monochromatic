/**
 Observation of the after-commit work records a landed commit leaves for the service.

 The wrapper writes a `push-<oid>` record under `cli-git/work/pending` once a commit has landed and before it wakes the
 service, so the record is the earliest durable trace of a landing that a harness can watch from outside.
 It moves through `claimed` to `done` or `failed` as the service works.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setInterval as ticks, } from 'node:timers/promises';

import type { RunningProcess, } from './process-fixture.ts';
import type { ScenarioRepository, } from './repository-fixture.ts';

//region Constants

/**
 Longest wait for a record, in milliseconds.
 */
export const RECORD_TIMEOUT_MS = 60_000;

/**
 Pause between two polls, in milliseconds.
 */
const POLL_MILLISECONDS = 1;

/**
 Every state a record passes through, in order.
 */
const STATES: readonly string[] = [
  'pending',
  'claimed',
  'done',
  'failed',
];

//endregion Constants

//region Observation

/**
 Lists the push records in the given states.

 @param repository - scenario repository

 @param states - work directories to list, every state by default

 @returns record names,
 empty before any commit landed

 @example
 ```ts
 await pushRecordNames({ repository });
 ```
 */
export async function pushRecordNames({
  repository,
  states = STATES,
}: Readonly<{
  repository: ScenarioRepository;
  states?: readonly string[];
}>,): Promise<readonly string[]> {
  return (await Promise.all(states.map(async function listState(state,) {
    try {
      return await readdir(join(
        repository.gitDir,
        'cli-git',
        'work',
        state,
      ),);
    }
    catch (error: unknown) {
      // The work directory appears with the first landing.
      if (Error.isError(error,) && ('code' in error)
        && (error.code === 'ENOENT'))
        return [];
      throw error;
    }
  },),)).flat()
    .filter(function isPush(name,) {
      return name.startsWith('push-',);
    },);
}

/**
 Waits until a commit's push record exists or the commit's wrapper settled.

 @param repository - scenario repository

 @param running - the wrapper process

 @param states - work directories the record may be in, every state by default

 @returns `record` when one appeared,
 `settled` when the wrapper ended first,
 `timeout` otherwise

 @example
 ```ts
 await waitForPushRecord({ repository, running: victim.running });
 ```
 */
export async function waitForPushRecord({
  repository,
  running,
  states = STATES,
}: Readonly<{
  repository: ScenarioRepository;
  running: Pick<RunningProcess, 'isSettled'>;
  states?: readonly string[];
}>,): Promise<'record' | 'settled' | 'timeout'> {
  /**
   When the wait ends.
   */
  const deadline = Date.now() + RECORD_TIMEOUT_MS;
  for await (const _tick of ticks(POLL_MILLISECONDS,)) {
    if ((await pushRecordNames({
      repository,
      states,
    },)).length > 0)
      return 'record';
    if (running.isSettled())
      return 'settled';
    if (Date.now() >= deadline)
      return 'timeout';
  }
  return 'timeout';
}

/**
 Reads the tip of the scenario's branch from its loose reference file.
 The service moves the branch with `update-ref`, which writes the loose file,
 so a read of the file is far quicker than starting Git and sees the move as soon as it happens.

 @param repository - scenario repository

 @returns the commit ID the branch names

 @example
 ```ts
 await readBranchTip(repository);
 ```
 */
export async function readBranchTip(repository: ScenarioRepository,): Promise<string> {
  return (await readFile(
    join(
      repository.commonDir,
      'refs',
      'heads',
      repository.branch,
    ),
    'utf8',
  )).trim();
}

/**
 Waits until the branch moved off a commit or the wrapper settled.

 @param repository - scenario repository

 @param from - the tip before the commit started

 @param running - the wrapper process

 @returns `moved` when the branch names another commit,
 `settled` when the wrapper ended first,
 `timeout` otherwise

 @example
 ```ts
 await waitForBranchMove({ repository, from: head, running: victim.running });
 ```
 */
export async function waitForBranchMove({
  repository,
  from,
  running,
}: Readonly<{
  repository: ScenarioRepository;
  from: string;
  running: Pick<RunningProcess, 'isSettled'>;
}>,): Promise<'moved' | 'settled' | 'timeout'> {
  /**
   When the wait ends.
   */
  const deadline = Date.now() + RECORD_TIMEOUT_MS;
  for await (const _tick of ticks(POLL_MILLISECONDS,)) {
    if (await readBranchTip(repository,) !== from)
      return 'moved';
    if (running.isSettled())
      return 'settled';
    if (Date.now() >= deadline)
      return 'timeout';
  }
  return 'timeout';
}

//endregion Observation
