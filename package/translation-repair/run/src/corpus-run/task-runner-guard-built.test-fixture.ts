/**
 The suite every package that owns a built command runs to show the start guard at the boundary: a child that holds a
 provider key and does not name the command as the one it means to start refuses, and the same child naming it (as
 the command's task does), or a child that holds no key, goes on to what the command did before the guard.

 TEST SUPPORT, NOT PACKAGE SOURCE. The commands belong to the packages that build them, so each of those packages
 hands its own commands to `guardedSuite` from its own test file, with the entry of its own build; this package holds
 the shared suite and the refusal it expects, since the refusal names the task of the package that owns the command
 through the owner table of this package.

 THE KEY IS INVENTED. `WHISKER_API_KEY` is no provider's variable, so a child holding it passes the guard's key test
 and still builds no client; nothing here can reach a provider. Every child is started by `runBuiltCommand` of the
 shared child fixture, whose `startedBy` says who the marker names, and the cases name a runs directory, a lookup
 cache and a corpus clone under a scratch directory.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  type DescribeChild,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChildRun,
  runBuiltCommand,
  type StartedBy,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { scratchPlaces, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
export const REFUSED_AS_STATED = 6;

/**
 What one command is shown to do: the arguments it is given under a scratch
 directory, and the whole result it leaves when the guard lets it go on.
 */
export type GuardedCommand = {
  /**
   Name of the built command.
   */
  readonly command: string;

  /**
   Directory name, under the family, of the package that owns the command and so its task.
   */
  readonly owner: string;

  /**
   URL of the built index of the package that owns the command, which the command's built file sits beside.
   */
  readonly packageEntry: string;

  /**
   Writes what the command reads under a scratch directory.

   @param directory - scratch directory the case owns

   @returns Command's arguments
   */
  readonly prepare: (directory: string,) => Promise<readonly string[]>;

  /**
   Everything the command leaves once the guard lets it go on.
   */
  readonly before: ChildRun;
};

/**
 The whole stderr of the guard's refusal for a child that holds one key.

 @param command - command the child ran

 @param owner - directory name of the package that owns the command, which names its task

 @returns Line `reportingRefusals` prints, with its newline

 @example
 ```ts
 expect(run.stderr,).toBe(guardRefusal({ command: 'spend-report', owner: 'run', },),);
 ```
 */
export function guardRefusal(
  {
    command,
    owner,
  }: {
    readonly command: string;
    readonly owner: string;
  },
): string {
  return `${command}: will not run in a process that holds provider keys `
    + '(1 variable whose name ends in _API_KEY has a value) unless that process names this command as the one it '
    + 'means to start. '
    + `The command's task does that: mise run //package/translation-repair/${owner}:${command} `
    + '(the task builds the package first). '
    + 'To start the built file without building, as when a pass is running and nothing may rebuild, '
    + `set TRANSLATION_REPAIR_STARTED_BY=${command} for that one start. `
    + 'A process that holds no provider key needs neither.\n';
}

/**
 Runs one command in a child under a scratch directory.

 @param guarded - command and what it reads

 @param holdsKey - whether the child holds the invented key

 @param startedBy - who the child's marker says started it

 @returns What the child wrote and how it exited

 @example
 ```ts
 const run = await runGuarded({ guarded, holdsKey: true, startedBy: 'nothing', },);
 ```
 */
async function runGuarded(
  {
    guarded,
    holdsKey,
    startedBy,
  }: {
    readonly guarded: GuardedCommand;
    readonly holdsKey: boolean;
    readonly startedBy: StartedBy;
  },
): Promise<ChildRun> {
  /**
   Scratch directory the case owns.
   */
  await using scratch = await scratchDir({ prefix: 'task-runner-guard-', },);
  return await runBuiltCommand({
    command: guarded.command,
    packageEntry: guarded.packageEntry,
    args: await guarded.prepare(scratch.path,),
    env: {
      ...scratchPlaces({
        runsDir: scratch.path,
        scratchDir: scratch.path,
      },),
      ...(holdsKey ? { WHISKER_API_KEY: 'purr', } : {}),
    },
    startedBy,
  },);
}

/**
 The three cases that show the guard for one command: refused when the process holds a key and names no command,
 let go on when it names the command, and let go on when it holds no key.

 @param guarded - command and what it reads

 @returns Suite of the three cases

 @example
 ```ts
 await describe({ name: '', concurrency: 1, children: [guardedSuite({ guarded: SPEND_REPORT, },),], },);
 ```
 */
export function guardedSuite({ guarded, }: { readonly guarded: GuardedCommand; },): DescribeChild {
  return describe({
    name: `${guarded.command} guarded as built`,
    concurrency: DEFAULT_CONCURRENCY,
    children: [
      it({
        name: `REFUSES as stated and exits 6 when the process holds a key and names no command, `
          + `before ${guarded.command} prints anything`,
        fn: async function refusesWithoutMarker(): Promise<void> {
          expect(await runGuarded({
            guarded,
            holdsKey: true,
            startedBy: 'nothing',
          },),)
            .toEqual({
              code: REFUSED_AS_STATED,
              stdout: '',
              stderr: guardRefusal({
                command: guarded.command,
                owner: guarded.owner,
              },),
            },);
        },
      },),

      it({
        name: `GOES ON to what ${guarded.command} did before when the process holds a key and names the `
          + 'command, as its own task does',
        fn: async function goesOnWithMarker(): Promise<void> {
          expect(await runGuarded({
            guarded,
            holdsKey: true,
            startedBy: 'its own task',
          },),)
            .toEqual(guarded.before,);
        },
      },),

      it({
        name: `GOES ON to what ${guarded.command} did before when the process holds no key and names no command`,
        fn: async function goesOnWithoutKey(): Promise<void> {
          expect(await runGuarded({
            guarded,
            holdsKey: false,
            startedBy: 'nothing',
          },),)
            .toEqual(guarded.before,);
        },
      },),
    ],
  },);
}
