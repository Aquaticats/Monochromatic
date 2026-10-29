/**
 Holds the task runner guard to every built command (the guard is
 `task-runner-guard.ts`): a command refuses to run in a process that holds
 provider keys unless that process names the command as the one it means to
 start, so a task that starts a built command must name it, and an entry file
 must hand the guard the environment it runs in.

 WHAT THE SCAN READS, for each runner entry the `src/build-entries.ts` of every family package lists
 (the library indexes left out):

 - the `mise.toml` of every family package, read as one: every task whose `run` names `dist/final/node/<command>.mjs`
   must set `env.TRANSLATION_REPAIR_STARTED_BY` to that same `<command>`, and
   every command must have such a task;
 - the entry's source: its one `reportingRefusals` call must be handed
   `env: process.env`, since that call is where the guard runs and an entry
   that hands it anything else, or nothing, lets a process that holds keys
   through.

 OUT OF ITS REACH: a task that sets the marker through an inline `env = { ... }`
 table or a file (it reads only the dotted `env.` form the file uses), a
 command started by anything but a task of `mise.toml` (the scan describes
 none), and what a guard does at run time, which the as-built cases of
 `corpus-run/task-runner-guard-built.unit.test.ts` show.

 THE FIXTURE CASE COMES FIRST, so the package case is read against a scan shown
 able to find each failure (ledger M21). Fixtures are cat-themed; the package
 case reads the family's own `mise.toml` files and source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readPackageSource,
  readTaskFiles,
  runnerEntriesOf,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  memberName,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Task runner guard reach scan
// What the tasks of `mise.toml` and the entry files say, read as text and as
// syntax trees, and the findings where they leave a command unguarded.

/**
 Name of the variable a task sets to the command it starts.
 */
const MARKER = 'TRANSLATION_REPAIR_STARTED_BY';

/**
 Function every entry hands its environment to.
 */
const HAND_OFF = 'reportingRefusals';

/**
 One task of `mise.toml` that starts a built command.
 */
type CommandTask = {
  /**
   The task's name.
   */
  readonly task: string;

  /**
   The command its `run` starts.
   */
  readonly command: string;

  /**
   What it sets the marker to, empty where it sets none.
   */
  readonly marker: string;
};

/**
 Header opening of every task table in `mise.toml`.
 */
const TASK_HEADER = '[tasks.';

/**
 Opening of the marker line a task sets, in the dotted form the file uses.
 */
const MARKER_LINE = `env.${MARKER} = "`;

/**
 Path stem a task's `run` names a built command by.
 */
const BUILT_STEM = 'dist/final/node/';

/**
 The tasks of a `mise.toml` that start a built command.

 @param mise - file's text

 @returns Each such task with the command it starts and the marker it sets

 @example
 ```ts
 const tasks = commandTasksOf({ mise, },);
 ```
 */
function commandTasksOf({ mise, }: { readonly mise: string; },): readonly CommandTask[] {
  return mise
    .split(TASK_HEADER,)
    .slice(1,)
    .flatMap(function taskOf(block,): readonly CommandTask[] {
      /**
       The block's lines, the first being the rest of the header.
       */
      const [header = '', ...lines] = block.split('\n',);
      /**
       The line whose run names a built command, absent for a task that starts none.
       */
      const run = lines.find(function startsBuilt(line,): boolean {
        return line.startsWith('run = ',) && line.includes(BUILT_STEM,);
      },);
      if (run === undefined)
        return [];
      /**
       The line that sets the marker, absent for a task that sets none.
       */
      const marker = lines.find(function setsMarker(line,): boolean {
        return line.startsWith(MARKER_LINE,) && line.endsWith('"',);
      },);
      return [
        {
          task: header
            .slice(0, header.indexOf(']',),)
            .replaceAll('"', '',),
          command: run.slice(run.indexOf(BUILT_STEM,) + BUILT_STEM.length, run.indexOf('.mjs',),),
          marker: (marker === undefined) ? '' : marker.slice(MARKER_LINE.length, -1,),
        },
      ];
    },);
}

/**
 Whether an entry's hand-off call is given the process's own environment.

 @param file - entry's source

 @returns Whether its one `reportingRefusals` call has an `env` property that
 reads `env` off `process`

 @example
 ```ts
 const guarded = handsEnvironment({ file, },);
 ```
 */
function handsEnvironment({ file, }: { readonly file: SourceText; },): boolean {
  return nodesUnder({ root: parseSource({ file, },).program, },)
    .filter(function callsHandOff(node,): boolean {
      if (node.type !== 'CallExpression')
        return false;
      return identifierName({ node: node.callee, },) === HAND_OFF;
    },)
    .some(function givesEnvironment(call,): boolean {
      /**
       The object the call is handed, absent when it is handed none.
       */
      const [options,] = call.arguments as readonly unknown[];
      if ((!isTreeNode(options,)) || (options.type !== 'ObjectExpression'))
        return false;
      return (options.properties as readonly TreeNode[])
        .some(function readsProcessEnv(property,): boolean {
          /**
           The property's value without its wrappers.
           */
          const value = unwrapped({ node: property.value, },).inner;
          return (identifierName({ node: property.key, },) === 'env')
            && isTreeNode(value,)
            && (value.type === 'MemberExpression')
            && (identifierName({ node: value.object, },) === 'process')
            && (memberName({ node: value, },) === 'env');
        },);
    },);
}

/**
 Where each command's guard is left open.

 @param mise - text of `mise.toml`

 @param entries - each command and the path of its entry, relative to `src`

 @param files - package's source files

 @returns One finding per command or task that leaves a command unguarded,
 empty when every command is guarded

 @example
 ```ts
 const findings = taskRunnerFindings({ mise, entries: [['purr', 'corpus-run/purr.ts',],], files, },);
 ```
 */
function taskRunnerFindings(
  {
    mise,
    entries,
    files,
  }: {
    readonly mise: string;
    readonly entries: readonly (readonly [string, string,])[];
    readonly files: readonly SourceText[];
  },
): readonly string[] {
  /**
   The tasks that start a built command.
   */
  const tasks = commandTasksOf({ mise, },);
  return [
    ...tasks
      .filter(function leavesMarkerOpen({ command, marker, },): boolean {
        return marker !== command;
      },)
      .map(function markerFinding({ task, command, marker, },): string {
        return `mise.toml: task ${task} starts ${command} and sets ${MARKER} to ${JSON.stringify(marker,)}`;
      },),
    ...entries.flatMap(function entryFindings([command, path,],): readonly string[] {
      /**
       The entry's source, absent where the build lists a file the scan cannot read.
       */
      const file = files.find(function isEntry(candidate,): boolean {
        return candidate.path === path;
      },);
      return [
        ...(tasks.some(function startsIt(task,): boolean {
          return task.command === command;
        },) ? [] : [`mise.toml: no task starts ${command}`,]),
        ...((file === undefined)
          ? [`${path}: the entry of ${command} is not among the source files`,]
          : (handsEnvironment({ file, },) ? [] : [`${path}: ${HAND_OFF} is not handed env: process.env`,])),
      ];
    },),
  ];
}

//endregion Task runner guard reach scan

/**
 A `mise.toml` of three commands: one guarded, one with no marker and one with
 another command's marker.
 */
const FIXTURE_MISE = [
  '[tasks.build]',
  'run = "rolldown"',
  '',
  '[tasks."purr-report"]',
  'env.TRANSLATION_REPAIR_STARTED_BY = "purr-report"',
  'run = "node dist/final/node/purr-report.mjs"',
  '',
  '[tasks."nap-census"]',
  'description = "Count naps"',
  'run = "node dist/final/node/nap-census.mjs"',
  '',
  '[tasks."claw-probe"]',
  'env.TRANSLATION_REPAIR_STARTED_BY = "purr-report"',
  'run = "node dist/final/node/claw-probe.mjs"',
  '',
].join('\n',);

/**
 Entry sources of the fixture: one handing the process's environment, one
 handing none, one handing another object, and one handing it through
 parentheses.
 */
const FIXTURE_FILES: readonly SourceText[] = [
  {
    path: 'corpus-run/purr-report.ts',
    text: 'await reportingRefusals({ what: \'purr-report\', argv: process.argv, env: process.env, run: main, },);',
    isTest: false,
  },
  {
    path: 'corpus-run/nap-census.ts',
    text: 'await reportingRefusals({ what: \'nap-census\', argv: process.argv, run: main, },);',
    isTest: false,
  },
  {
    path: 'corpus-run/claw-probe.ts',
    text: 'await reportingRefusals({ what: \'claw-probe\', argv: process.argv, env: { PATH: \'x\', }, run: main, },);',
    isTest: false,
  },
  {
    path: 'corpus-run/hiss-sample.ts',
    text: 'await reportingRefusals({ what: \'hiss-sample\', argv: process.argv, env: (process.env), run: main, },);',
    isTest: false,
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: taskRunnerFindings.name,
      concurrency: 1,
      children: [
        it({
          name: 'FINDS a task that sets no marker, a task that sets another command\'s, a command with no task and an '
            + 'entry that hands no environment or another object, and leaves a guarded command alone',
          fn: async () => {
            expect(taskRunnerFindings({
              mise: FIXTURE_MISE,
              entries: [
                ['purr-report', 'corpus-run/purr-report.ts',],
                ['nap-census', 'corpus-run/nap-census.ts',],
                ['claw-probe', 'corpus-run/claw-probe.ts',],
                ['hiss-sample', 'corpus-run/hiss-sample.ts',],
                ['yowl-bench', 'corpus-run/yowl-bench.ts',],
              ],
              files: FIXTURE_FILES,
            },),).toEqual([
              'mise.toml: task nap-census starts nap-census and sets TRANSLATION_REPAIR_STARTED_BY to ""',
              'mise.toml: task claw-probe starts claw-probe and sets TRANSLATION_REPAIR_STARTED_BY to "purr-report"',
              'corpus-run/nap-census.ts: reportingRefusals is not handed env: process.env',
              'corpus-run/claw-probe.ts: reportingRefusals is not handed env: process.env',
              'mise.toml: no task starts hiss-sample',
              'mise.toml: no task starts yowl-bench',
              'corpus-run/yowl-bench.ts: the entry of yowl-bench is not among the source files',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'task runner guard reach in this package',
      concurrency: 1,
      children: [
        it({
          name: 'LEAVES NO BUILT COMMAND UNGUARDED: every task that starts one sets the marker to its name, every '
            + 'command has a task, and every entry hands reportingRefusals the process\'s environment',
          fn: async () => {
            /**
             Every file of the family.
             */
            const files = await readPackageSource();
            /**
             Every runner the family builds and the location of its entry.
             */
            const entries = runnerEntriesOf({ files, },).map(function entryOf({ name, path, },): readonly [string, string,] {
              return [name, path,];
            },);
            expectNoFindings({
              findings: taskRunnerFindings({
                mise: (await readTaskFiles()).map(function textOf({ text, },): string {
                  return text;
                },).join('\n',),
                entries,
                files,
              },),
            },);
          },
        },),
      ],
    },),
  ],
},);
