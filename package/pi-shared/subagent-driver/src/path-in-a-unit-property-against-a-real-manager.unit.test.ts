/**
 The paths a unit's `ReadOnlyPaths=` and `ReadWritePaths=` are given, tried against this machine's own manager.

 The encoding in `path-in-a-unit-property.ts` decides whether a start happens at all when a checkout or a worktree path carries an
 awkward character, and only a real unit can answer that: this test asks for one capped transient unit per awkward name, with the
 property words this package builds, and checks that the writable folder inside the read-only root is writable and the root's own
 file is not. A machine with no user manager to answer is skipped with the reason it gave, so the test says something there rather
 than failing.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  fileSystemOfARun,
  fileSystemPropertiesOf,
  makeTheFoldersAToolMakesIn,
} from '../dist/final/node/index.mjs';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 Seconds one question to the manager may take, which is a bound rather than a measurement: a manager that cannot answer in this
 long is not answering.
 */
const MANAGER_PATIENCE_SECONDS = 60;

/**
 The awkward names a checkout folder, a worktree or a run folder may carry, each of which systemd's path lists would split or
 break on without the quoting.
 */
const AWKWARD_NAMES: readonly (readonly [string, string])[] = [
  ['space', 'inside with space',],
  ['single-quote', "inside'single",],
  ['double-quote', 'inside"double',],
  ['percent', 'inside%100',],
  ['specifier-percent', 'inside%h',],
  ['backslash', String.raw`inside\backslash`,],
  ['colon', 'inside:colon',],
  ['leading-dash', '-inside',],
  ['line-break', 'inside\nline-break',],
];

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, removed when the test ends.

 @returns the folder

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-run-encoding-manager-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 What one question to this machine's manager answered.

 @param request - the unit's name, its properties and the command it runs

 @param request.unit - the unit's name

 @param request.properties - the property words

 @param request.words - the command

 @returns the status, and what the manager and the command printed

 @example
 ```ts
 const ended = await askTheManager({ unit: 'probe-x', properties: [], words: ['/bin/true'], });
 ```
 */
async function askTheManager({
  unit,
  properties,
  words,
}: {
  readonly unit: string;
  readonly properties: readonly string[];
  readonly words: readonly string[];
},): Promise<{ readonly status: number; readonly stdout: string; readonly stderr: string }> {
  try {
    /**
     What the manager and the command answered.
     */
    const answered = await execFileAsync('systemd-run', [
      '--user',
      '--wait',
      '--collect',
      '--quiet',
      '--pipe',
      `--unit=${unit}`,
      '--property=RuntimeMaxSec=30',
      '--property=CPUQuota=100%',
      '--property=MemoryMax=256M',
      '--property=TasksMax=64',
      ...properties,
      ...words,
    ], {
      encoding: 'utf8',
      timeout: MANAGER_PATIENCE_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    return {
      status: 0,
      stdout: answered.stdout,
      stderr: answered.stderr,
    };
  }
  catch (error) {
    /**
     What a manager that answered with a status reported, read field by field because the status is the answer.
     */
    const failure = error as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
      readonly message?: unknown;
    };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : (-1),
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: [
        ((typeof failure.stderr) === 'string') ? failure.stderr : '',
        ((typeof failure.message) === 'string') ? failure.message : '',
      ].join('\n',)
        .trim(),
    };
  }
}

/**
 How many units this test has asked for, so each unit name is its own.
 */
const asked: { count: number } = { count: 0, };

/**
 Reads the next unit name and counts it.

 @returns the unit's name

 @example
 ```ts
 const unit = nextUnitName();
 ```
 */
function nextUnitName(): string {
  asked.count += 1;
  return `probe-encoding-${String(asked.count,)}`;
}

/**
 Asks this machine's manager whether it answers at all, which is what a test here needs before it asks for a unit.

 @returns what it said when it does not answer, or nothing when it does

 @example
 ```ts
 const { reason, } = await noManagerReason();
 ```
 */
async function noManagerReason(): Promise<{ readonly reason?: string }> {
  /**
   What the manager answered to a unit that does nothing.
   */
  const manager = await askTheManager({
    unit: nextUnitName(),
    properties: [],
    words: ['/bin/true',],
  },);
  if (manager.status === 0) {
    return {};
  }
  return { reason: `no user manager answered ${JSON.stringify(manager.stderr,)}, so no unit can be asked for here`, };
}

/**
 Asks for one capped unit on a read-only root holding a writable folder whose name carries an awkward character, and checks that
 the unit wrote inside that folder and could not change the root's own file.

 @param request - the folder this case makes its roots in, the awkward character's name and the writable folder's name

 @param request.folder - the folder the case's root goes in

 @param request.caseName - the awkward character's name, which names the root

 @param request.insideName - the writable folder's name

 @example
 ```ts
 await askOneAwkwardPath({ folder: '/w', caseName: 'space', insideName: 'inside with space', });
 ```
 */
async function askOneAwkwardPath({
  folder,
  caseName,
  insideName,
}: {
  readonly folder: string;
  readonly caseName: string;
  readonly insideName: string;
},): Promise<void> {
  /**
   The folder the checkout stands for, which the unit reads only.
   */
  const root = join(
    folder,
    `${caseName}-root`,
  );
  /**
   The folder inside it the unit must be able to write.
   */
  const inside = join(
    root,
    insideName,
  );
  await mkdir(inside, { recursive: true, },);
  /**
   A file of the read-only root that the unit must not be able to change.
   */
  const protectedFile = join(root, 'protected.txt',);
  await writeFile(protectedFile, 'seed\n',);
  /**
   What the unit writes when its writable folder is the one it was named.
   */
  const written = join(inside, 'written',);
  /**
   What the manager answered for this name.
   */
  const ended = await askTheManager({
    unit: nextUnitName(),
    properties: fileSystemPropertiesOf({
      foldersAUnitCannotWrite: [root,],
      foldersAUnitMustWrite: [inside,],
    },),
    words: [
      '/bin/sh',
      '-c',
      'echo written > "$1" || exit 9; echo changed >> "$2" && exit 8; exit 0',
      'sh',
      written,
      protectedFile,
    ],
  },);
  expect(ended.status,).toBe(0,);
  expect(await readFile(written, 'utf8',),).toBe('written\n',);
  expect(await readFile(protectedFile, 'utf8',),).toBe('seed\n',);
}

/**
 Why every test here is skipped when this machine's manager does not answer, which is asked once for the whole file.
 */
const { reason: NO_MANAGER_REASON, } = await noManagerReason();

await describe({
  name: '',
  children: [
    describe({
      name: 'the paths a unit is given on this machine',
      children: [
        ...AWKWARD_NAMES.map(function aTestForOneName([caseName, insideName,],) {
          return it({
            name: `starts one capped unit for a checkout path whose writable folder carries a ${caseName}, and the unit writes there and not in the checkout`,
            fn: async () => {
              if (NO_MANAGER_REASON !== undefined) {
                console.log(`[the paths a unit is given on this machine] ${caseName}: skipped: ${NO_MANAGER_REASON}`,);
                return;
              }
              await using folder = await makeFolder();
              await askOneAwkwardPath({
                folder: folder.path,
                caseName,
                insideName,
              },);
            },
          },);
        },),
        it({
          name: 'starts a unit for a checkout whose cache folder a tool makes, which the launcher makes first so the unit can write in it',
          fn: async () => {
            if (NO_MANAGER_REASON !== undefined) {
              console.log(`[the paths a unit is given on this machine] absent-cache: skipped: ${NO_MANAGER_REASON}`,);
              return;
            }
            await using folder = await makeFolder();
            /**
             The checkout the run works from, which has no cache folder yet.
             */
            const checkout = join(
              folder.path,
              'checkout',
            );
            /**
             Where the run works, which is a worktree of that checkout.
             */
            const worktree = join(
              checkout,
              '.claude',
              'worktrees',
              'pi-a',
            );
            /**
             The run's own folder.
             */
            const runFolder = join(
              folder.path,
              'runs',
              'a-1',
            );
            await mkdir(join(checkout, '.git',), { recursive: true, },);
            await mkdir(worktree, { recursive: true, },);
            await mkdir(runFolder, { recursive: true, },);
            /**
             A file of the checkout that the unit must not be able to change.
             */
            const protectedFile = join(checkout, 'protected.txt',);
            await writeFile(protectedFile, 'seed\n',);
            // What a start does before it asks for its unit: the folder a run's tools install themselves into is made, so a tool
            // that makes it under the read-only checkout is not refused.
            await makeTheFoldersAToolMakesIn({ checkoutFolder: checkout, },);
            /**
             The folder the unit must be able to make its own files in.
             */
            const insideTheCache = join(
              checkout,
              '.cache',
              'toolchain-bin',
            );
            /**
             What the manager answered for this checkout.
             */
            const ended = await askTheManager({
              unit: nextUnitName(),
              properties: fileSystemPropertiesOf(await fileSystemOfARun({
                git: async function answers(request,) {
                  /**
                   The two directories this fixture's Git would name, which are the checkout's own `.git` folder.
                   */
                  const gitDirectory = join(checkout, '.git',);
                  const [first, second, third,] = request.words.slice(0, 4,);
                  return ((first === 'rev-parse') && (second === '--path-format=absolute')
                    && ((third === '--absolute-git-dir') || (third === '--git-common-dir')))
                    ? {
                      status: 0,
                      stdout: `${gitDirectory}\n`,
                      stderr: '',
                    }
                    : {
                      status: 1,
                      stdout: '',
                      stderr: 'a test s git answers nothing else',
                    };
                },
                checkoutFolder: checkout,
                worktree,
                runFolder,
                worksInTheCheckoutItself: false,
              },),),
              words: [
                '/bin/sh',
                '-c',
                'mkdir --parents "$1" || exit 9; echo installed > "$1/tool" || exit 8; echo changed >> "$2" && exit 7; exit 0',
                'sh',
                insideTheCache,
                protectedFile,
              ],
            },);
            expect(ended.status,).toBe(0,);
            expect(await readFile(join(insideTheCache, 'tool',), 'utf8',),).toBe('installed\n',);
            expect(await readFile(protectedFile, 'utf8',),).toBe('seed\n',);
          },
        },),
      ],
    },),
  ],
},);
