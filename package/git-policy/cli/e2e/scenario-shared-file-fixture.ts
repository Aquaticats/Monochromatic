/**
 Two agents editing one file:
 far-apart hunks must both land;
 overlapping hunks may fail only as a reported replay conflict.

 @module
 */

import {
  holdAt,
  reached,
  releaseAt,
  settleWithin,
} from './barrier-fixture.ts';
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
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  readWorktree,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Window the second agent gets to start and capture before the first is released.
 */
const CAPTURE_WINDOW_MS = 1_500;

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
 The first agent is held in `pre-commit` after capturing,
 the second edits and starts committing,
 then the first is released.

 @param context - scenario context

 @param secondFrom - first line the second agent edits

 @returns both finished attempts

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
}>,): Promise<readonly AttemptRecord[]> {
  await editSharedLines({
    context,
    from: context.random
      .integer(FIRST_HUNK,),
    label: 'first',
  },);
  /**
   First agent.
   */
  const first = await startAttempt({
    ...context,
    label: 'first',
    paths: ['shared.txt',],
    mode: 'explicit',
  },);
  await holdAt({
    repository: context.repository,
    token: first.token,
    event: 'pre-commit',
  },);
  await reached({
    repository: context.repository,
    token: first.token,
    running: first.running,
    event: 'pre-commit',
  },);
  await editSharedLines({
    context,
    from: secondFrom,
    label: 'second',
  },);
  /**
   Second agent.
   */
  const second = await startAttempt({
    ...context,
    label: 'second',
    paths: ['shared.txt',],
    mode: 'explicit',
  },);
  await settleWithin({
    running: second.running,
    windowMs: CAPTURE_WINDOW_MS,
  },);
  await releaseAt({
    repository: context.repository,
    token: first.token,
    event: 'pre-commit',
  },);
  return await finishAll([
    first,
    second,
  ],);
}

/**
 Shared-file repository setup.

 @param random - seeded source

 @returns options with a hooked `pre-commit` and the shared file

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
    hooks: 'config',
    hookEvents: ['pre-commit',],
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
  summary: 'two agents commit the same file with far-apart hunks while the first is still preparing',
  // The first agent is held in a config `pre-commit` hook (Git 2.54.0).
  minimumGit: '2.54.0',
  repository: sharedRepository,
  async run(context,) {
    /**
     Finished attempts.
     */
    const attempts = await runSharedFilePair({
      context,
      secondFrom: context.random
        .integer(FAR_HUNK,),
    },);
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts,
    },),];
  },
};

/**
 Overlapping hunks in one file.
 */
const sharedOverlapping: ScenarioDefinition = {
  name: 'shared-file-overlapping-hunks',
  group: 'concurrency',
  summary: 'two agents rewrite the same lines; one may fail only as a reported replay conflict',
  // The first agent is held in a config `pre-commit` hook (Git 2.54.0).
  minimumGit: '2.54.0',
  repository: sharedRepository,
  async run(context,) {
    /**
     Finished attempts.
     */
    const attempts = await runSharedFilePair({
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
      {
        name: 'one-lands',
        holds: failed.length < attempts.length,
        detail: 'no attempt landed',
      },
      {
        name: 'conflict-reported',
        holds: failed.every(function reported(attempt,) {
          return (attempt.outcome
            .exitCode
            === 1)
            && attempt.outcome
              .stderr
              .includes('"type":"core-finding"',);
        },),
        detail: failed.map(function describe(attempt,) {
          return `${attempt.label} exited ${String(attempt.outcome
            .exitCode,)} without a core-finding: ${attempt.outcome
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
