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
  joinLines,
  splitLines,
} from './content-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import {
  finishAll,
  repositoryOptions,
} from './scenario-helper-fixture.ts';
import {
  queuedExpectation,
  startQueued,
} from './scenario-queue-fixture.ts';
import {
  allSucceeded,
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

//endregion Workload

//region Scenario

/**
 Adjacent edits in one shared file.
 */
const sharedAdjacent: ScenarioDefinition = {
  name: 'shared-file-adjacent-edits',
  group: 'concurrency',
  summary: 'a second agent edits right next to a queued first edit while the service is frozen; both land, the second with its captured bytes',
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
    context.repository
      .service
      .stall();
    /**
     First agent, prepared and queued behind the frozen service.
     */
    const first = await startQueued({
      context,
      label: 'first',
      paths: ['shared.txt',],
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
     Second agent, prepared with both edits and queued behind the first.
     */
    const second = await startQueued({
      context,
      label: 'second',
      paths: ['shared.txt',],
    },);
    context.repository
      .service
      .resume();
    /**
     Both agents, landed.
     */
    const [firstRecord, secondRecord,] = await finishAll([
      first.attempt,
      second.attempt,
    ],);
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
      queuedExpectation([
        first,
        second,
      ],),
      allSucceeded({
        name: 'all-commits-succeed',
        attempts: [
          firstRecord,
          secondRecord,
        ].filter(function present(record,) {
          return record !== undefined;
        },),
      },),
      {
        name: 'second-landed-as-captured',
        holds: landed === captured.toString('utf8',),
        detail: `the tip's shared file is not the bytes the second agent captured (${placement} line ${String(at,)})`,
      },
    ];
  },
};

/**
 Subsumption scenarios in report order.
 */
export const SUBSUMPTION_SCENARIOS: readonly ScenarioDefinition[] = [sharedAdjacent,];

//endregion Scenario
