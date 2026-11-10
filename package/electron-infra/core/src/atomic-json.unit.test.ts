/**
 Unit tests for the shared atomic JSON writer, and the check that no Electron main
 process keeps a second definition of that write beside it.

 Issue 1279 found the two app main processes hand-rolling the same temp-file plus
 rename that this package exports, byte-identical to the owner and free to drift
 from it in temp naming or trailing newline while still looking identical. The
 ledger below names every main process that writes boundary-test state: each one
 fails its checks until it calls this writer and holds no write of its own, and a
 new main process that writes state fails until it is listed.

 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  existsSync,
  readFileSync,
} from 'node:fs';
import {
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { writeJsonFileAtomically, } from '../dist/final/node/index.mjs';

/**
 The declaration text the single-definition scan counts.
 */
const WRITER_DECLARATION_TEXT = `export async function ${writeJsonFileAtomically.name}(`;

/**
 The module path every state-writing main process must take the writer from.
 */
const WRITER_MODULE_PATH = '@monochromatic-dev/electron-infra-core/ts/atomic-json';

/**
 Repository root, four folders above this test file.
 */
const repositoryRoot = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Value written by the fixtures below.
 */
const fixtureValue = {
  count: 3,
  ready: true,
};

/**
 The exact file text the writer must produce for `fixtureValue`: two-space
 indentation and one trailing newline, which the app state readers parse.
 */
const fixtureValueFileText = '{\n  "count": 3,\n  "ready": true\n}\n';

/**
 Replacement value written over an existing destination.
 */
const replacementValue = { ready: false, };

/**
 The exact file text the writer must produce for `replacementValue`.
 */
const replacementValueFileText = '{\n  "ready": false\n}\n';

/**
 The Electron main processes that write the boundary-test state file, each with
 the name it reads that file's path from.

 The name marks that the main process still writes state at all, so deleting the
 write cannot pass as deleting a duplicate of it.
 */
const stateWritingMainProcesses: readonly {
  readonly path: string;
  readonly statePathName: string;
}[] = [
  {
    path: 'package/electron-infra/app-counter/src/main.ts',
    statePathName: 'statePathEnvironmentVariable',
  },
  {
    path: 'package/file-manager/variant-electron/src/main.ts',
    statePathName: 'STATE_PATH_ENVIRONMENT_VARIABLE',
  },
];

/**
 Reads a repository file.

 @param path - Repository-relative path.

 @returns The file's text.

 @example
 ```ts
 const text = repositoryFileText({ path: 'package/electron-infra/core/src/index.ts' });
 ```
 */
function repositoryFileText({ path, }: { readonly path: string; },): string {
  return readFileSync(
    join(
      repositoryRoot,
      path,
    ),
    'utf8',
  );
}

/**
 Whether a main process takes the writer from its owner and calls it, while still
 reading the state path its boundary test sets.

 @param text - Text of the main process.

 @param statePathName - Name the main process reads the state path from.

 @returns Whether the main process calls the shared writer.

 @example
 ```ts
 const calls = callsTheSharedWriter({ text: mainProcessText, statePathName: 'STATE_PATH_ENVIRONMENT_VARIABLE' });
 ```
 */
function callsTheSharedWriter(
  {
    statePathName,
    text,
  }: {
    readonly statePathName: string;
    readonly text: string;
  },
): boolean {
  if (!text.includes(statePathName,))
    return false;
  if (!text.includes(WRITER_MODULE_PATH,))
    return false;
  return text.includes(`${writeJsonFileAtomically.name}(`,);
}

/**
 Whether a main process still writes its own temporary file and renames it.

 @param text - Text of the main process.

 @returns Whether the main process holds a write of its own.

 @example
 ```ts
 const own = writesItsOwnTemporaryFile({ text: mainProcessText });
 ```
 */
function writesItsOwnTemporaryFile({ text, }: { readonly text: string; },): boolean {
  if (text.includes('rename',))
    return true;
  return text.includes('.tmp',);
}

/**
 Tracked files whose text defines the writer.

 @returns Repository-relative paths, in `git ls-files` order.

 @example
 ```ts
 const paths = filesDefiningTheWriter();
 ```
 */
function filesDefiningTheWriter(): readonly string[] {
  return execFileSync(
    'git',
    ['ls-files', '-z',],
    {
      cwd: repositoryRoot,
      encoding: 'utf8',
      maxBuffer: 1 << 28,
    },
  )
    .split('\0',)
    .filter((path,) => path.endsWith('.ts',),)
    .filter((path,) => repositoryFileText({ path, },).includes(WRITER_DECLARATION_TEXT,),);
}

/**
 Disposable destination fixture: a directory holding one JSON destination.
 */
type DestinationFixture = {
  /**
   Directory holding the destination, and nothing else once a write completes.
   */
  readonly directory: string;
  /**
   Destination path the writer must create.
   */
  readonly filePath: string;
  /**
   Removes the directory recursively.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a destination fixture in the scratch root.

 @returns Disposable fixture with an absent destination.

 @example
 ```ts
 await using fixture = await makeDestinationFixture();
 ```
 */
async function makeDestinationFixture(): Promise<DestinationFixture> {
  /**
   Directory the destination lives in.
   */
  const directory = await mkdtemp(join(
    scratchRoot(),
    'electron-infra-atomic-json-',
  ),);

  return {
    directory,
    filePath: join(
      directory,
      'state.json',
    ),
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        directory,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

await describe({
  name: writeJsonFileAtomically.name,
  children: [
    it({
      name: 'writes two-space JSON with one trailing newline',
      fn: async () => {
        await using fixture = await makeDestinationFixture();

        await writeJsonFileAtomically({
          filePath: fixture.filePath,
          value: fixtureValue,
        },);

        expect(await readFile(
          fixture.filePath,
          'utf8',
        ),).toBe(fixtureValueFileText,);
      },
    },),
    it({
      name: 'replaces the previous contents of an existing destination',
      fn: async () => {
        await using fixture = await makeDestinationFixture();
        await writeFile(
          fixture.filePath,
          fixtureValueFileText,
          'utf8',
        );

        await writeJsonFileAtomically({
          filePath: fixture.filePath,
          value: replacementValue,
        },);

        expect(await readFile(
          fixture.filePath,
          'utf8',
        ),).toBe(replacementValueFileText,);
      },
    },),
    it({
      name: 'leaves only the destination behind, never its temporary file',
      fn: async () => {
        await using fixture = await makeDestinationFixture();

        await writeJsonFileAtomically({
          filePath: fixture.filePath,
          value: fixtureValue,
        },);

        expect(await readdir(fixture.directory,),).toEqual(['state.json',],);
      },
    },),
  ],
},);

await describe({
  name: 'the single definition of the boundary-test state write',
  children: [
    it({
      name: 'defines the writer in exactly one tracked file',
      fn: async () => {
        expect(filesDefiningTheWriter(),).toEqual(['package/electron-infra/core/src/atomic-json.ts',],);
      },
    },),
    it({
      name: 'names a main process that exists for every ledger entry',
      fn: async () => {
        /**
         Ledger paths that no file holds.
         */
        const missing = stateWritingMainProcesses
          .map((entry,) => entry.path,)
          .filter((path,) => !existsSync(join(
            repositoryRoot,
            path,
          ),),);
        expect(missing,).toEqual([],);
      },
    },),
    it({
      name: 'lists no main process twice',
      fn: async () => {
        expect(
          new Set(stateWritingMainProcesses.map((entry,) => entry.path,),).size,
        ).toBe(stateWritingMainProcesses.length,);
      },
    },),
    it({
      name: 'calls the writer from every main process that writes boundary-test state',
      fn: async () => {
        /**
         Ledger entries whose main process does not call the writer.
         */
        const notCalling = stateWritingMainProcesses
          .filter(({ path, statePathName, },) => !callsTheSharedWriter({
            statePathName,
            text: repositoryFileText({ path, },),
          },),)
          .map((entry,) => entry.path,);
        expect(notCalling,).toEqual([],);
      },
    },),
    it({
      name: 'holds no temporary path or rename of its own in those main processes',
      fn: async () => {
        /**
         Ledger entries whose main process still writes a temporary file itself.
         */
        const stillWriting = stateWritingMainProcesses
          .filter(({ path, },) => writesItsOwnTemporaryFile({ text: repositoryFileText({ path, },), },),)
          .map((entry,) => entry.path,);
        expect(stillWriting,).toEqual([],);
      },
    },),
  ],
},);
