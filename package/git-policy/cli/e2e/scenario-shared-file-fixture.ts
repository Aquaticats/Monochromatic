/**
 Two agents editing one file:
 far-apart hunks must both land;
 overlapping hunks may fail only as a reported replay conflict.

 @module
 */

import {
  joinLines,
  splitLines,
} from './content-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import {
  finishAll,
  repositoryOptions,
} from './scenario-helper-fixture.ts';
import {
  type QueuedAttempt,
  queuedExpectation,
  startQueued,
} from './scenario-queue-fixture.ts';
import {
  allSucceeded,
  isReportedConflict,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  readWorktree,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Lines in the shared file.
 */
const SHARED_LINES = 60;

/**
 Lines each agent rewrites.
 */
const HUNK_LINES = 3;

/**
 Range of the first agent's hunk start.
 */
const FIRST_HUNK = {
  min: 2,
  max: 8,
} as const;

/**
 Range of the second agent's far-apart hunk start.
 */
const FAR_HUNK = {
  min: 40,
  max: 50,
} as const;

/**
 Second agent's overlapping hunk start, inside every first hunk.
 */
const OVERLAPPING_HUNK = 4;

/**
 Largest seed-line suffix.
 */
const LINE_SUFFIX_MAX = 9_999;

//endregion Constants

//region Workload

/**
 Rewrites a block of lines of the shared file.

 @param context - scenario context

 @param from - first line index

 @param label - text marking whose edit it is

 @example
 ```ts
 await editSharedLines({ context, from: 2, label: 'first' });
 ```
 */
async function editSharedLines({
  context,
  from,
  label,
}: Readonly<{
  context: ScenarioContext;
  from: number;
  label: string;
}>,): Promise<void> {
  /**
   Current shared lines.
   */
  const lines = splitLines((await readWorktree({
    repository: context.repository,
    path: 'shared.txt',
  },)).bytes ?? Buffer.from('\n',),);
  await writeWorktree({
    ...context,
    path: 'shared.txt',
    bytes: joinLines(lines.map(function edit(
      line,
      index,
    ) {
      return (index >= from) && (index < (from
        + HUNK_LINES)) ? `${label} edit ${String(index,)}` : line;
    },),),
  },);
}

/**
 Both agents prepare against the branch as it is, while the service is frozen:
 the first agent has edited its hunk and is queued,
 the second then edits its hunk and is queued behind it,
 and the service runs again.
 The first lands, and the second, which saw the branch before the first landed, has its captured bytes merged onto it.

 @param context - scenario context

 @param secondFrom - first line the second agent edits

 @returns both finished attempts and the queue facts

 @example
 ```ts
 await runSharedFilePair({ context, secondFrom: 40 });
 ```
 */
async function runSharedFilePair({
  context,
  secondFrom,
}: Readonly<{
  context: ScenarioContext;
  secondFrom: number;
}>,): Promise<Readonly<{
  attempts: readonly AttemptRecord[];
  queued: readonly QueuedAttempt[];
}>> {
  await editSharedLines({
    context,
    from: context.random
      .integer(FIRST_HUNK,),
    label: 'first',
  },);
  context.repository
    .service
    .stall();
  /**
   First agent, prepared and queued.
   */
  const first = await startQueued({
    context,
    label: 'first',
    paths: ['shared.txt',],
  },);
  await editSharedLines({
    context,
    from: secondFrom,
    label: 'second',
  },);
  /**
   Second agent, prepared and queued behind the first.
   */
  const second = await startQueued({
    context,
    label: 'second',
    paths: ['shared.txt',],
  },);
  context.repository
    .service
    .resume();
  return {
    attempts: await finishAll([
      first.attempt,
      second.attempt,
    ],),
    queued: [
      first,
      second,
    ],
  };
}

/**
 Shared-file repository setup.

 @param random - seeded source

 @returns options with the shared file

 @example
 ```ts
 sharedRepository(random);
 ```
 */
function sharedRepository(random: Parameters<ScenarioDefinition['repository']>[0],): ReturnType<ScenarioDefinition['repository']> {
  return repositoryOptions({
    seedFiles: [{
      path: 'shared.txt',
      bytes: joinLines(Array.from(
        { length: SHARED_LINES, },
        function line(
          _unused,
          index,
        ) {
          return `shared base line ${String(index,)} ${String(random.integer({
          min: 0,
          max: LINE_SUFFIX_MAX,
        },),)}`;
        },
      ),),
    },],
  },);
}

//endregion Workload

//region Scenarios

/**
 Non-overlapping hunks in one file.
 */
const sharedNonOverlapping: ScenarioDefinition = {
  name: 'shared-file-non-overlapping-hunks',
  group: 'concurrency',
  summary: 'two agents commit the same file with far-apart hunks while the service is frozen, so both prepared before either landed',
  repository: sharedRepository,
  async run(context,) {
    /**
     Finished attempts and queue facts.
     */
    const {
      attempts,
      queued,
    } = await runSharedFilePair({
      context,
      secondFrom: context.random
        .integer(FAR_HUNK,),
    },);
    return [
      queuedExpectation(queued,),
      allSucceeded({
        name: 'all-commits-succeed',
        attempts,
      },),
    ];
  },
};

/**
 Overlapping hunks in one file.
 */
const sharedOverlapping: ScenarioDefinition = {
  name: 'shared-file-overlapping-hunks',
  group: 'concurrency',
  summary: 'two agents rewrite the same lines while the service is frozen; one may fail only as a reported conflict',
  repository: sharedRepository,
  async run(context,) {
    /**
     Finished attempts and queue facts.
     */
    const {
      attempts,
      queued,
    } = await runSharedFilePair({
      context,
      secondFrom: OVERLAPPING_HUNK,
    },);
    /**
     Attempts that failed.
     */
    const failed = attempts.filter(function nonZero(attempt,) {
      return attempt.outcome
        .exitCode
        !== 0;
    },);
    return [
      queuedExpectation(queued,),
      {
        name: 'one-lands',
        holds: failed.length < attempts.length,
        detail: 'no attempt landed',
      },
      {
        name: 'conflict-reported',
        holds: failed.every(function reportsConflict(attempt,) {
          return isReportedConflict(attempt,);
        },),
        detail: failed.map(function describe(attempt,) {
          return `${attempt.label} exited ${String(attempt.outcome
            .exitCode,)} without a replay-conflict finding naming its path: ${attempt.outcome
              .stderr
              .trim()
              .split('\n',)
              .at(-1,)
              ?? ''}`;
        },)
          .join('; ',),
      },
    ];
  },
};

/**
 Shared-file scenarios in report order.
 */
export const SHARED_FILE_SCENARIOS: readonly ScenarioDefinition[] = [
  sharedNonOverlapping,
  sharedOverlapping,
];

//endregion Scenarios
