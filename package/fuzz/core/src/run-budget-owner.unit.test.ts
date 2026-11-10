/**
 The ledger of every fuzz sidecar that reads a fast-check run budget, and the check that the budget has one owner.

 `package/fuzz/core/src/run-budget.ts` decides the default run count and what an override that does not parse
 means. Issue 1419 found that the ten sidecars listed below each carried a character-identical copy of that module, so
 the default and the policy were decided in ten places with no owner; they took their count from the shared module
 instead, naming only their own environment variable.

 {@link SIDECARS_READING_A_RUN_BUDGET} is the ledger: a sidecar that stops binding its count fails the first test, and
 a file that declares a run count without being listed fails the last one, so a new copy cannot appear unnoticed. The
 middle two tests hold the shape the copies had: only the shared module may declare a default run count or parse an
 override, and every listed sidecar imports the parse rather than writing it.

 @module
 */

import { execFileSync, } from 'node:child_process';
import { readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Repository scan

/**
 The worktree root, four directories above this file (`src` -> `coverage` -> `fuzz` -> `package`).
 */
const repositoryRoot = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 The one file allowed to declare a default run count or parse an override.
 */
const OWNER_PATH = 'package/fuzz/core/src/run-budget.ts';

/**
 The module specifier every sidecar imports the shared parse through.
 */
const SHARED_SPECIFIER = '@monochromatic-dev/fuzz-core/ts';

/**
 The fuzz sidecars that read a per-property run count, each with the file that binds it for the sidecar's properties.
 */
const SIDECARS_READING_A_RUN_BUDGET: readonly string[] = [
  'package/css/core-edit.fuzz/src/fuzz-budget.ts',
  'package/jsonc-edit/core-ts.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/conf.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/make-asynchronous.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/matcher.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/p-limit.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/p-map.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/p-memoize.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/pify.fuzz/src/fuzz-budget.ts',
  'package/upstream-fork/quick-lru.fuzz/src/fuzz-budget.ts',
];

/**
 Folders the scan skips: documents, paused or retired packages, and installed dependencies.
 */
const SKIPPED_FOLDERS: readonly string[] = [
  'doc/',
  'package-deprecated/',
  'package-paused/',
  'node_modules/',
];

/**
 A file's text, read once per run.
 */
const textsRead = new Map<string, string>();

/**
 The text of one tracked file.

 @param path - repository-relative path

 @returns the file's text

 @example
 ```ts
 const text = textOf(OWNER_PATH);
 ```
 */
function textOf(path: string,): string {
  /**
   Text read earlier in this run, when the file has been read already.
   */
  const earlier = textsRead.get(path,);
  if (earlier !== undefined)
    return earlier;
  /**
   The file's text, kept for the reads that follow.
   */
  const text = readFileSync(
    join(
      repositoryRoot,
      path,
    ),
    'utf8',
  );
  textsRead.set(path, text,);
  return text;
}

/**
 Tracked TypeScript files of the packages, which is where a run budget can be copied into.

 Newly written files count too, so a copy is caught before it is committed rather than after. Suites are left out: this
 file names the declarations it guards, and a suite that asserts on one is not a copy of it.

 @returns repository-relative paths

 @example
 ```ts
 const files = trackedPackageTypeScriptPaths();
 ```
 */
function trackedPackageTypeScriptPaths(): readonly string[] {
  return execFileSync(
    'git',
    ['ls-files', '--cached', '--others', '--exclude-standard', '-z', 'package',],
    {
      cwd: repositoryRoot,
      encoding: 'utf8',
      maxBuffer: 1 << 28,
    },
  )
    .split('\0',)
    .filter((path,) => path.endsWith('.ts',),)
    .filter((path,) => !path.includes('.test.ts',),)
    .filter((path,) => !SKIPPED_FOLDERS.some((folder,) => path.includes(folder,),),);
}

/**
 Files whose text shows they declare a run count instead of importing one.

 @param marker - text a file declares with

 @returns repository-relative paths

 @example
 ```ts
 const declaring = pathsDeclaring('export const fuzzRuns');
 ```
 */
function pathsDeclaring(marker: string,): readonly string[] {
  return trackedPackageTypeScriptPaths().filter((path,) => textOf(path,).includes(marker,),);
}

//endregion Repository scan

//region Ledger

await describe({
  name: 'fuzz run-budget ledger',
  children: [
    it({
      name: 'lists a sidecar file that still exports its run count',
      fn: async () => {
        expect(
          SIDECARS_READING_A_RUN_BUDGET.filter((path,) => !textOf(path,).includes('export const fuzzRuns',),),
        ).toEqual([],);
      },
    },),
    it({
      name: 'takes every sidecar run count from the shared parse, naming only its environment variable',
      fn: async () => {
        /**
         Sidecars that write their own parse or import the shared one differently.
         */
        const ownParses = SIDECARS_READING_A_RUN_BUDGET.filter((path,) => {
          /**
           The sidecar's run-budget file.
           */
          const text = textOf(path,);
          return !text.includes(SHARED_SPECIFIER,)
            || !text.includes('fuzzRunsFromEnvironment(',)
            || text.includes('Number.',);
        },);
        expect(ownParses,).toEqual([],);
      },
    },),
    it({
      name: 'declares the default run count in the shared module alone',
      fn: async () => {
        expect(pathsDeclaring('const DEFAULT_FUZZ_RUNS',),).toEqual([OWNER_PATH,],);
      },
    },),
    it({
      name: 'lists every file that declares a run count, so a new sidecar cannot skip the ledger',
      fn: async () => {
        /**
         Files that declare the per-sidecar value; the shared module's own example shows the declaration a sidecar
         writes, so it names the shape without being a sidecar.
         */
        const declaring = pathsDeclaring('export const fuzzRuns',)
          .filter((path,) => path !== OWNER_PATH,);
        expect(declaring,).toEqual(SIDECARS_READING_A_RUN_BUDGET,);
      },
    },),
  ],
},);

//endregion Ledger
