/**
 Subsumption replay:
 a second agent edits the shared file right next to a first agent's edit while the first is still in flight,
 so its captured bytes already hold the first edit;
 when the first lands,
 the second replays with its captured bytes as they are
 (`doc/decision/cli-git-concurrent-commits.md` "Serial landing").

 Both agents pause after preparation through the test-only `preparation-done` phase marker,
 so the second captures after the first and the first lands first,
 without hooks and without the hook lock.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { BARRIER_TIMEOUT_MS, } from './barrier-fixture.ts';
import {
  joinLines,
  splitLines,
} from './content-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { waitForMarker, } from './process-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import { repositoryOptions, } from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  readWorktree,
  type StartedAttempt,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Lines in the shared file.
 */
const SHARED_LINES = 60;

/**
 Range of the first agent's edited line.
 */
const FIRST_LINE = {
  min: 10,
  max: 50,
} as const;

/**
 Where the second agent's edit sits relative to the first:
 the line after it,
 the line before it,
 or a new line inserted directly after it.
 */
const PLACEMENTS = [
  'after',
  'before',
  'insert',
] as const;

/**
 Largest seed-line suffix.
 */
const LINE_SUFFIX_MAX = 9_999;

//endregion Constants

//region Workload

/**
 Rewrites the shared file through one edit.

 @param context - scenario context

 @param edit - line edit

 @example
 ```ts
 await editShared({ context, edit: function first(lines) { return lines; } });
 ```
 */
async function editShared({
  context,
  edit,
}: Readonly<{
  context: ScenarioContext;
  edit: (lines: readonly string[],) => readonly string[];
}>,): Promise<void> {
  await writeWorktree({
    ...context,
    path: 'shared.txt',
    bytes: joinLines(
      edit(splitLines((await readWorktree({
        repository: context.repository,
        path: 'shared.txt',
      },)).bytes ?? Buffer.from('\n',),),),
    ),
  },);
}

/**
 Starts an agent that pauses after preparation, and waits until it is there.

 @param context - scenario context

 @param label - agent label

 @returns started agent, its phase directory, and whether it reached the pause

 @example
 ```ts
 await startPaused({ context, label: 'first' });
 ```
 */
async function startPaused({
  context,
  label,
}: Readonly<{
  context: ScenarioContext;
  label: string;
}>,): Promise<Readonly<{
  attempt: StartedAttempt;
  phases: string;
  paused: boolean;
}>> {
  /**
   Phase directory of this agent.
   */
  const phases = join(
    context.repository
      .markerDir,
    `${label}-phases`,
  );
  await mkdir(
    phases,
    { recursive: true, },
  );
  /**
   Started agent.
   */
  const attempt = await startAttempt({
    ...context,
    label,
    paths: ['shared.txt',],
    mode: 'explicit',
    env: { CLI_GIT_TEST_ONLY_PHASE_SIGNAL: `preparation-done:pause:${phases}`, },
  },);
  return {
    attempt,
    phases,
    paused: (await waitForMarker({
      path: join(
        phases,
        'preparation-done.reached',
      ),
      timeoutMs: BARRIER_TIMEOUT_MS,
      isSettled: attempt.running
        .isSettled,
    },)) === 'marker',
  };
}

/**
 Releases a paused agent and waits for it.

 @param agent - paused agent

 @returns finished attempt
 */
async function release(agent: Awaited<ReturnType<typeof startPaused>>,): Promise<AttemptRecord> {
  await writeFile(
    join(
      agent.phases,
      'preparation-done.release',
    ),
    '',
  );
  return await agent.attempt
    .finished;
}

//endregion Workload

//region Scenario

/**
 Adjacent edits in one shared file.
 */
const sharedAdjacent: ScenarioDefinition = {
  name: 'shared-file-adjacent-edits',
  group: 'concurrency',
  summary: 'a second agent edits right next to an in-flight first edit; both land, the second with its captured bytes through subsumption',
  repository(random,) {
    return repositoryOptions({ seedFiles: [{
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
    },], },);
  },
  async run(context,) {
    /**
     First agent's line.
     */
    const at = context.random
      .integer(FIRST_LINE,);
    /**
     Second agent's placement.
     */
    const placement = PLACEMENTS[context.random
      .integer({
        min: 0,
        max: PLACEMENTS.length - 1,
      },)] ?? 'after';
    await editShared({
      context,
      edit: function firstEdit(lines,) {
        return lines.with(
          at,
          `first edit ${String(at,)}`,
        );
      },
    },);
    /**
     First agent, paused after capturing and preparing.
     */
    const first = await startPaused({
      context,
      label: 'first',
    },);
    await editShared({
      context,
      edit: function secondEdit(lines,) {
        if (placement === 'insert')
          return lines.toSpliced(
            at + 1,
            0,
            `second insert ${String(at + 1,)}`,
          );
        /**
         Line the second agent rewrites.
         */
        const target = placement === 'after' ? at + 1 : at - 1;
        return lines.with(
          target,
          `second edit ${String(target,)}`,
        );
      },
    },);
    /**
     Bytes the second agent captures.
     */
    const captured = (await readWorktree({
      repository: context.repository,
      path: 'shared.txt',
    },)).bytes ?? Buffer.from('',);
    /**
     Second agent, paused after capturing both edits.
     */
    const second = await startPaused({
      context,
      label: 'second',
    },);
    /**
     First agent, landed.
     */
    const firstRecord = await release(first,);
    /**
     Second agent, replayed.
     */
    const secondRecord = await release(second,);
    /**
     Shared file at the tip.
     */
    const landed = await realGit({
      repository: context.repository,
      args: [
        'cat-file',
        'blob',
        'HEAD:shared.txt',
      ],
    },);
    return [
      allSucceeded({
        name: 'all-commits-succeed',
        attempts: [
          firstRecord,
          secondRecord,
        ],
      },),
      {
        name: 'agents-paused-in-order',
        holds: first.paused && second.paused,
        detail: `first paused ${String(first.paused,)}, second paused ${String(second.paused,)} (${placement})`,
      },
      {
        name: 'second-replayed-as-captured',
        holds: secondRecord.outcome
          .stderr
          .includes('"type":"commit-replayed"',)
          && (landed === captured.toString('utf8',)),
        detail: `the second agent (${placement} line ${String(at,)}) did not replay onto the first with its captured bytes`,
      },
    ];
  },
};

/**
 Subsumption scenarios in report order.
 */
export const SUBSUMPTION_SCENARIOS: readonly ScenarioDefinition[] = [sharedAdjacent,];

//endregion Scenario
