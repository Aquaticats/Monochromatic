/**
 One whole run of the package's stand-in Pi, started in a worktree of a throwaway repository, making one `edit` or `write` call:
 what the guard refuses, what it allows, and what the run's record keeps of the call.

 The fault these tests answer is the one of 2026-11-27 (issue 6636): a run told to read a file "on `main`" opened the main
 checkout's copy, and its edit landed there. So a run started in a worktree must not be able to change the checkout it was made
 from, and a run started in the checkout itself must behave exactly as it did before this guard existed.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  FEED_PREVIEW_CHARACTERS,
  isRecord,
  type RunSummary,
} from '../dist/final/node/index.mjs';
import { EDITS_A_PATH_SCENARIO, } from './stand-in-pi-edits-a-path-plays-fixture.ts';
import {
  A_FILE_NAME,
  MAIN_CHECKOUT_TEXT,
  makeThrowawayRepository,
  type ThrowawayRepository,
  WORKTREE_TEXT,
} from './throwaway-repository-fixture.ts';

/**
 The driver's own command, run from this package rather than its built copy: the bin imports the built library the way a user's
 install does.
 */
const DRIVER = new URL('drive-pi-subagent.ts', import.meta.url,).pathname;

/**
 The stand-in Pi this package's tests start instead of a real one, which calls no model.
 */
const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 Seconds a run may last: the wall clock is a backstop for a run whose own events end it, so it sits far above them.
 */
const RUN_WALL_SECONDS = 120;

/**
 Seconds the run waits for a steering message after Pi settles, which is short enough for a test to sit through and long enough
 that the settled event is the run's own end.
 */
const IDLE_SECONDS = 0.2;

/**
 Turns a run may start, which is one tool call and the turn around it.
 */
const TURN_CAP = 3;

/**
 Seconds a test allows a driver beyond the run's own wall limit, so a stalled driver fails rather than hangs it.
 */
const TEST_TEARDOWN_ALLOWANCE_SECONDS = 120;

/**
 Milliseconds in a second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 The text a run's call writes, which is what a landed call leaves in the file.
 */
const WRITTEN_TEXT = 'the line the run wrote\n';

/**
 One sentence of the text a test makes longer than a feed line keeps, whose length is what sizes that text.
 */
const ONE_SENTENCE = 'a long line the run wrote. ';

/**
 A run finished, and what a test reads of it.
 */
type FinishedRun = {
  readonly status: number;
  readonly standardError: string;
  readonly summary: RunSummary;
  readonly feed: readonly Record<string, unknown>[];
};

/**
 What one file holds, or that nothing is there.
 */
type TextOfAFile =
  | { readonly text: string }
    | { readonly absent: true };

/**
 Reads one file's text, or says that nothing is there.

 @param path - the file

 @returns its text, or that nothing is there
 */
async function textOfAFile(path: string,): Promise<TextOfAFile> {
  try {
    return { text: await readFile(
      path,
      'utf8',
    ), };
  }
  catch (error) {
    // A file that is not there is exactly what a test about a refused write asks about.
    void error;
    return { absent: true, };
  }
}

/**
 Installs the stand-in as the `pi` a run starts, in a folder of the test's own.

 @param folder - the folder the launcher is written into
 */
async function installStandIn(folder: string,): Promise<void> {
  /**
   The launcher, which runs this process's own Node on the stand-in.
   */
  const launcher = join(
    folder,
    'pi',
  );
  await writeFile(
    launcher,
    `#!/usr/bin/env bash\nexec '${process.execPath}' '${STAND_IN}' "$@"\n`,
  );
  await chmod(
    launcher,
    0o700,
  );
}

/**
 Runs the driver once, in a folder of the throwaway repository, whose stand-in makes one `edit` or `write` call.

 @param repository - the throwaway repository

 @param workingFolder - the folder the run works in: its worktree, or the main checkout for a run with none

 @param toolName - the tool the call is

 @param path - the path the call names

 @param text - the text the call would write

 @param pathLast - whether the path comes last in the call's arguments, as a model that wrote a long text first leaves it

 @returns what the run ended with, its summary and its feed

 @example
 ```ts
 const run = await runOneCall({ repository, workingFolder: repository.worktree, toolName: 'edit', path });
 ```
 */
async function runOneCall({
  repository,
  workingFolder,
  toolName,
  path,
  text = WRITTEN_TEXT,
  pathLast = false,
}: {
  readonly repository: ThrowawayRepository;
  readonly workingFolder: string;
  readonly toolName: string;
  readonly path: string;
  readonly text?: string;
  readonly pathLast?: boolean;
},): Promise<FinishedRun> {
  /**
   This run's own folder, which holds the feed and the summary.
   */
  const runFolder = join(
    repository.home,
    'runs',
    'a-1',
  );
  /**
   The prompt the run is given, which no test leaves empty.
   */
  const promptFile = join(
    repository.home,
    'prompt.md',
  );
  await writeFile(
    promptFile,
    'read the file on main and change it',
  );
  /**
   The folder holding the stand-in this run starts as `pi`.
   */
  const standInFolder = join(
    repository.home,
    'stand-in',
  );
  await mkdir(
    standInFolder,
    { recursive: true, },
  );
  await installStandIn(standInFolder,);
  /**
   What the driver printed on its standard error, which a run that refused a call still says any problem on.
   */
  const standardError = await driverStandardError({
    arguments: [
      DRIVER,
      `--run-folder=${runFolder}`,
      `--working-folder=${workingFolder}`,
      `--prompt-file=${promptFile}`,
      '--provider=stand-in',
      '--model=stand-in-model',
      `--wall-seconds=${String(RUN_WALL_SECONDS,)}`,
      `--idle-seconds=${String(IDLE_SECONDS,)}`,
      `--turn-cap=${String(TURN_CAP,)}`,
    ],
    repository,
    standInFolder,
    toolName,
    path,
    text,
    pathLast,
  },);
  return {
    status: 0,
    standardError,
    summary: JSON.parse(await readFile(
      join(
        runFolder,
        'summary.json',
      ),
      'utf8',
    ),) as RunSummary,
    feed: (await readFile(
      join(
        runFolder,
        'feed.jsonl',
      ),
      'utf8',
    )).split('\n',)
      .filter(function isALine(line,) {
        return line !== '';
      },)
      .map(function readOneLine(line,) {
        return JSON.parse(line,) as Record<string, unknown>;
      },),
  };
}

/**
 Runs the driver as a child process and reads what it printed, failing with those words when it ended non-zero.

 @param request - the driver's words, and what the stand-in is told

 @returns what the driver printed on its standard error

 @throws Error naming the standard error when the driver ended non-zero, which no run of these tests may
 */
async function driverStandardError({
  arguments: driverArguments,
  repository,
  standInFolder,
  toolName,
  path,
  text,
  pathLast,
}: {
  readonly arguments: readonly string[];
  readonly repository: ThrowawayRepository;
  readonly standInFolder: string;
  readonly toolName: string;
  readonly path: string;
  readonly text: string;
  readonly pathLast: boolean;
},): Promise<string> {
  try {
    /**
     What the driver answered.
     */
    const ended = await execFileAsync(process.execPath, [...driverArguments,], {
      env: {
        PATH: [
          standInFolder,
          dirname(process.execPath,),
          '/usr/bin',
        ].join(delimiter,),
        HOME: repository.home,
        STAND_IN_PI_SCENARIO: EDITS_A_PATH_SCENARIO,
        STAND_IN_PI_EDIT_TOOL: toolName,
        STAND_IN_PI_EDIT_PATH: path,
        STAND_IN_PI_EDIT_TEXT: text,
        ...(pathLast ? { STAND_IN_PI_EDIT_PATH_LAST: 'yes', } : {}),
        // The process table is not this machine's: a run reads the folder a test names, which holds nothing.
        PI_RUNS_PROCESS_FOLDER: repository.processFolder,
      },
      encoding: 'utf8',
      timeout: (RUN_WALL_SECONDS + TEST_TEARDOWN_ALLOWANCE_SECONDS) * MILLISECONDS_PER_SECOND,
      killSignal: 'SIGKILL',
    },);
    return ended.stderr;
  }
  catch (error) {
    throw new Error(
      `the run's driver ended non-zero, which no run of these tests may: ${standardErrorOf({ error, },)}`,
      { cause: error, },
    );
  }
}

/**
 Reads the standard error a child process left, when the operating system's answer carries one.

 @param error - what `execFile` rejected with

 @returns its standard error, or empty text when it carries none
 */
function standardErrorOf({ error, }: { readonly error: unknown },): string {
  if (!isRecord(error,)) {
    return '';
  }
  return ((typeof error.stderr) === 'string') ? error.stderr : '';
}

/**
 Reads the text a file holds, which a test's own run always left there.

 @param path - the file

 @returns its text

 @throws Error when nothing is there, which is a run that wrote no file at all
 */
async function textThatIsThere(path: string,): Promise<string> {
  /**
   What the file holds.
   */
  const held = await textOfAFile(path,);
  if ('absent' in held) {
    throw new Error(`no file is at ${path}, which this test's run wrote`,);
  }
  return held.text;
}

/**
 Reads the one tool call's start off a run's feed.

 @param feed - the run's feed lines

 @param kind - the kind of line to read

 @returns the line, which a test's own run always wrote

 @throws Error when the run wrote no such line, which is a run that took no call at all
 */
function oneToolLine({
  feed,
  kind,
}: {
  readonly feed: readonly Record<string, unknown>[];
  readonly kind: string;
},): Record<string, unknown> {
  /**
   The line of that kind.
   */
  const line = feed.find(function isOfTheKind(one,) {
    return (one.kind === kind) && ((one.tool === 'edit') || (one.tool === 'write'));
  },);
  if (line === undefined) {
    throw new Error(`the run's feed holds no ${kind} line: ${feed.map(function asText(one,) {
      return String(one.kind,);
    },)
      .join(', ',)}`,);
  }
  return line;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a run started in a worktree',
      children: [
        it({
          name: "refuses an edit of the main checkout's copy, and both copies keep what they held",
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The main checkout's copy of the file, which the run may not change.
             */
            const mainCopy = join(
              repository.repository,
              'package',
              A_FILE_NAME,
            );
            /**
             The worktree's copy, which the refused call leaves as it held it.
             */
            const worktreeCopy = join(
              repository.worktree,
              'package',
              A_FILE_NAME,
            );
            /**
             What the run ended with.
             */
            const run = await runOneCall({
              repository,
              workingFolder: repository.worktree,
              toolName: 'edit',
              path: mainCopy,
            },);
            /**
             The line the feed keeps of the call's end, which is where Pi's refusal reaches a reader.
             */
            const ended = oneToolLine({
              feed: run.feed,
              kind: 'tool_end',
            },);

            expect(await textThatIsThere(mainCopy,),).toBe(MAIN_CHECKOUT_TEXT,);
            expect(await textThatIsThere(worktreeCopy,),).toBe(WORKTREE_TEXT,);
            expect(run.status,).toBe(0,);
            expect(run.summary.endReason,).toBe('idle',);
            expect(ended.isError,).toBe(true,);
            expect(String(ended.result,),).toContain('may not change the main checkout',);
            expect(String(ended.result,),).toContain(mainCopy,);
          },
        }),
        it({
          name: "allows an edit of the worktree's copy of the same path",
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The worktree's copy of the file, which is the run's to change.
             */
            const worktreeCopy = join(
              repository.worktree,
              'package',
              A_FILE_NAME,
            );
            /**
             What the run ended with.
             */
            const run = await runOneCall({
              repository,
              workingFolder: repository.worktree,
              toolName: 'edit',
              path: worktreeCopy,
            },);

            expect(await textThatIsThere(worktreeCopy,),).toBe(WRITTEN_TEXT,);
            expect(oneToolLine({
              feed: run.feed,
              kind: 'tool_end',
            },).isError,).toBe(false,);
          },
        }),
        it({
          name: "allows a write to the run's scratch folder",
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The file the run writes in its own scratch folder, outside the repository.
             */
            const inTheScratchFolder = join(
              repository.scratchFolder,
              'notes.md',
            );
            /**
             What the run ended with.
             */
            const run = await runOneCall({
              repository,
              workingFolder: repository.worktree,
              toolName: 'write',
              path: inTheScratchFolder,
            },);

            expect(await textThatIsThere(inTheScratchFolder,),).toBe(WRITTEN_TEXT,);
            expect(oneToolLine({
              feed: run.feed,
              kind: 'tool_end',
            },).isError,).toBe(false,);
          },
        }),
        it({
          name: 'keeps the path of the call whole when its other arguments are longer than the cut',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The worktree's copy, so the call lands and the record holds a whole call.
             */
            const worktreeCopy = join(
              repository.worktree,
              'package',
              A_FILE_NAME,
            );
            /**
             A text longer than a feed line keeps of a tool call's arguments, which is what cuts the path off when the model
             writes the path after the text.
             */
            const longText = ONE_SENTENCE.repeat(
              Math.ceil((FEED_PREVIEW_CHARACTERS.toolArguments * 2) / ONE_SENTENCE.length,),
            );
            /**
             What the run ended with.
             */
            const run = await runOneCall({
              repository,
              workingFolder: repository.worktree,
              toolName: 'write',
              path: worktreeCopy,
              text: longText,
              pathLast: true,
            },);
            /**
             The line the feed keeps of the call's start.
             */
            const started = oneToolLine({
              feed: run.feed,
              kind: 'tool_start',
            },);

            // The preview alone lost the path: that is the fault the whole path answers.
            expect(String(started.args,),).not.toContain(worktreeCopy,);
            expect(started.path,).toBe(worktreeCopy,);
          },
        }),
      ],
    }),
    describe({
      name: 'a run started in the main checkout itself',
      children: [
        it({
          name: 'refuses nothing, as a session of a person that is no run in a worktree',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The main checkout's copy, which a run started there may change as any session may.
             */
            const mainCopy = join(
              repository.repository,
              'package',
              A_FILE_NAME,
            );
            /**
             What the run ended with.
             */
            const run = await runOneCall({
              repository,
              workingFolder: repository.repository,
              toolName: 'edit',
              path: mainCopy,
            },);

            expect(await textThatIsThere(mainCopy,),).toBe(WRITTEN_TEXT,);
            expect(oneToolLine({
              feed: run.feed,
              kind: 'tool_end',
            },).isError,).toBe(false,);
          },
        }),
      ],
    }),
  ],
},);
