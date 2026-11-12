/**
 A batch started as its own systemd user unit: the unit's name, the command that starts it, the file its output goes to, and the
 line a reader gets.

 The program is run for real against a stand-in `systemd-run` on a `PATH` of the test's own, so no test starts a unit.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { delimiter, dirname, join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  BATCH_UNIT_CPU_PERCENT,
  BATCH_UNIT_IO_WEIGHT,
  BATCH_UNIT_MEMORY,
  BATCH_UNIT_SECONDS,
  BATCH_UNIT_TASK_COUNT,
  checkoutFolderOf,
  batchOutputFileOf,
  batchUnitCommandOf,
  batchUnitNameOf,
  startedLineOf,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 A folder a test throws away, holding a stand-in `systemd-run` and `mise`.

 @returns the folder and where its programs live

 @example
 ```ts
 await using folder = await makeFolderWithPrograms();
 ```
 */
async function makeFolderWithPrograms(): Promise<{
  readonly path: string;
  readonly bins: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-batch-unit-',),);
  /**
   Where the stand-in programs live.
   */
  const bins = join(path, 'bin',);
  await writeFile(join(path, 'list.json',), '[]\n',);
  await mkdir(bins, { recursive: true, },);
  return {
    path,
    bins,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one stand-in program.

 @param folder - the folder the programs live in

 @param name - the program's name

 @param body - what it does, as a bash body

 @example
 ```ts
 await writeProgram({ folder: bins, name: 'systemd-run', body: 'exit 0', });
 ```
 */
async function writeProgram({
  folder,
  name,
  body,
}: {
  readonly folder: string;
  readonly name: string;
  readonly body: string;
}): Promise<void> {
  /**
   The program's file.
   */
  const file = join(folder, name,);
  await writeFile(file, `#!/usr/bin/env bash\n${body}\n`,);
  await chmod(file, 0o700,);
}

/**
 Runs the batch-unit command as a program, with the test's programs ahead of node.

 @param request - the folder whose programs it uses, its words, and the log its stand-ins write to

 @returns the status and both streams

 @example
 ```ts
 const ran = await runTheCommand({ folder, words: ['--batch-file=/runs/b.json'], });
 ```
 */
async function runTheCommand({
  folder,
  words,
  log,
}: {
  readonly folder: {
    readonly path: string;
    readonly bins: string;
  };
  readonly words: readonly string[];
  readonly log: string;
}): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   The programs the command may run, then the node that runs it.
   */
  const path = [
    folder.bins,
    dirname(process.execPath,),
    '/usr/bin',
  ].join(delimiter,);
  try {
    /**
     What the command printed.
     */
    const done = await execFileAsync(process.execPath, [
      new URL('start-a-batch-in-its-own-unit.ts', import.meta.url,).pathname,
      ...words,
    ], {
      env: {
        PATH: path,
        HOME: folder.path,
        FAKE_LOG: log,
      },
      encoding: 'utf8',
      timeout: 60_000,
    },);
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    /**
     What the failure carries.
     */
    const failure = caught as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : 1,
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: batchUnitNameOf.name,
      children: [
        it({
          name: 'names the unit after the list, never with the pi- prefix the slot count reads as a run',
          fn: async () => {
            expect(batchUnitNameOf({ batchFile: '/runs/agents-batch-round-231.json', },),).toBe('launch-agents-batch-round-231',);
            expect(batchUnitNameOf({ batchFile: '/runs/agents-batch-round-231.json', },).startsWith('pi-',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: batchOutputFileOf.name,
      children: [
        it({
          name: 'writes the output beside the list, under the list name and out',
          fn: async () => {
            expect(batchOutputFileOf({ batchFile: '/runs/agents-batch-round-231.json', },),).toBe('/runs/agents-batch-round-231.out',);
          },
        },),
      ],
    },),
    describe({
      name: batchUnitCommandOf.name,
      children: [
        it({
          name: 'starts the batch command as a capped unit whose both streams go to the output file, with the PATH and home it needs',
          fn: async () => {
            /**
             What the unit runs.
             */
            const command = batchUnitCommandOf({
              batchFile: '/runs/agents-batch-round-231.json',
              deadlineSeconds: 7_200,
              outputFile: '/runs/agents-batch-round-231.out',
              pathOfMiseEnvironment: '/bin',
              home: '/home/a',
              node: '/bin/node',
              batchEntry: '/pkg/src/start-pi-runs-from-a-batch-list.ts',
              checkoutFolder: '/repo',
            },);
            expect(command.command,).toBe('systemd-run',);
            expect(command.words,).toEqual([
              '--user',
              '--unit=launch-agents-batch-round-231',
              '--collect',
              `--property=RuntimeMaxSec=${String(BATCH_UNIT_SECONDS,)}`,
              `--property=MemoryMax=${BATCH_UNIT_MEMORY}`,
              `--property=CPUQuota=${String(BATCH_UNIT_CPU_PERCENT,)}%`,
              `--property=TasksMax=${String(BATCH_UNIT_TASK_COUNT,)}`,
              `--property=IOWeight=${String(BATCH_UNIT_IO_WEIGHT,)}`,
              '--property=StandardOutput=append:/runs/agents-batch-round-231.out',
              '--property=StandardError=append:/runs/agents-batch-round-231.out',
              '--setenv=PATH=/bin',
              '--setenv=HOME=/home/a',
              '--working-directory=/repo',
              '/bin/node',
              '/pkg/src/start-pi-runs-from-a-batch-list.ts',
              '--batch-file=/runs/agents-batch-round-231.json',
              '--deadline-seconds=7200',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: startedLineOf.name,
      children: [
        it({
          name: 'says which unit started and where its output is, and says what systemd-run said when it started nothing',
          fn: async () => {
            expect(startedLineOf({
              unit: 'launch-b',
              outputFile: '/runs/b.out',
              status: 0,
              problem: '',
            },),).toBe('started launch-b.service; output in /runs/b.out',);
            expect(startedLineOf({
              unit: 'launch-b',
              outputFile: '/runs/b.out',
              status: 1,
              problem: 'Unit launch-b.service already exists.',
            },),).toBe('could not start launch-b: Unit launch-b.service already exists.',);
          },
        },),
      ],
    },),
    describe({
      name: 'start-a-batch-in-its-own-unit',
      children: [
        it({
          name: 'starts the unit through systemd-run and says where its output goes',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            /**
             What the stand-in programs were asked.
             */
            const log = join(folder.path, 'asked.log',);
            await writeProgram({
              folder: folder.bins,
              name: 'systemd-run',
              body: `printf '%s\\n' "systemd-run $*" >> "$FAKE_LOG"`,
            },);
            await writeProgram({
              folder: folder.bins,
              name: 'mise',
              body: `printf '%s\\n' '/bin:/usr/bin'`,
            },);
            /**
             What the command printed.
             */
            const ran = await runTheCommand({
              folder,
              words: [`--batch-file=${join(folder.path, 'list.json',)}`,],
              log,
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toBe(`started launch-list.service; output in ${join(folder.path, 'list.out',)}`,);
            /**
             What systemd-run was asked.
             */
            const asked = await readFile(log, 'utf8',);
            expect(asked,).toContain('--unit=launch-list',);
            expect(asked,).toContain(`--batch-file=${join(folder.path, 'list.json',)}`,);
            // The unit works in the checkout the command was run in, never in the folder the list lives in: a launcher started
            // outside the checkout cannot read where a run's worktree belongs.
            expect(asked,).toContain(`--working-directory=${await checkoutFolderOf({ from: process.cwd(), },)}`,);
          },
        },),
        it({
          name: 'refuses a line that names no list',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            /**
             What the command answered with no list named.
             */
            const refused = await runTheCommand({
              folder,
              words: [],
              log: join(folder.path, 'asked.log',),
            },);
            expect(refused.status,).not.toBe(0,);
            expect(refused.stderr,).toContain('--batch-file is required',);
          },
        },),
      ],
    },),
  ],
},);
