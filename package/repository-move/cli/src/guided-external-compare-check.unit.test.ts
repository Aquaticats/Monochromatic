/**
 Test of the cut-over check that reads the external worktrees' compare file: the check passes when rsync reported
 nothing, or only files deleted at the source after pass one, and fails on any other difference.

 The runner's shell is `bash --noprofile --norc -e -o pipefail -c`, so a `grep --invert-match` that selects no lines
 exits 1, which is exactly what an empty compare and a compare of only `*deleting` lines give: the first draft of this
 check failed on both, and failed for the right reason only on a real difference. This test holds all three, and the
 reader that stops early must not turn the stage's report into a status 141.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  createGuidedConfig,
  cutoverStages,
  runShell,
  type Stage,
} from '../dist/final/node/index.mjs';

/**
 Identifier of the stage that compares the external worktrees against their copies.
 */
const EXTERNAL_COMPARE_STAGE_ID = 'c17-externals';

/**
 A line rsync prints for a file deleted at the source after pass one, which the copy keeps on purpose.
 */
const DELETING_LINE = '*deleting   doc/removed-after-pass-one.md';

/**
 A line rsync prints for any other difference, which fails the stage.
 */
const DIFFERENCE_LINE = '>f+++++++++ doc/changed-after-pass-one.md';

/**
 How many difference lines the stage's report shows, the `head --lines` of its check.
 */
const DIFFERENCES_SHOWN = 20;

/**
 Exit status of a check that passed.
 */
const CHECK_PASSED = 0;

/**
 Exit status of a check that failed on its own terms, never of a shell ended by a signal.
 */
const CHECK_FAILED = 1;

/**
 Exit status a pipeline reports when its writer was ended by SIGPIPE.
 */
const KILLED_BY_PIPE = 141;

/**
 Deadline of a shell run, in seconds: the check only reads small files.
 */
const SHELL_DEADLINE_SECONDS = 30;

/**
 Seconds between terminate and kill for a shell run.
 */
const SHELL_KILL_GRACE_SECONDS = 5;

/**
 A disposable home folder with the move's state folder, removed when the scope ends.
 */
type HomeFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a disposable home folder holding the runner's state folder.

 @returns the folder, to hold with `await using`

 @example
 ```ts
 await using home = await makeHome();
 ```
 */
async function makeHome(): Promise<HomeFolder> {
  /**
   Path of the new folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'guided-external-compare-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The stage, its configuration and the file its check reads, built from a disposable home.

 @returns the check text, the compare file's path and the differences file's path

 @example
 ```ts
 const { check, compareFile, } = await externalCompare();
 ```
 */
async function externalCompare(): Promise<{
  readonly home: HomeFolder;
  readonly check: string;
  readonly compareFile: string;
  readonly differencesFile: string;
}> {
  /**
   The disposable home this invocation reads.
   */
  const home = await makeHome();
  /**
   The configuration the stages read.
   */
  const config = await createGuidedConfig({
    env: {
      HOME: home.path,
      MOVE_DRIVE: join(home.path, 'drive',),
      MOVE_DESTINATION_ROOT: join(home.path, 'drive', 'repository',),
    },
    nodePath: process.execPath,
  },);
  await mkdir(config.wiz, { recursive: true, },);
  /**
   The cut-over stages.
   */
  const stages = await cutoverStages({ c: config, stamp: 'S', },);
  /**
   The stage that compares the external worktrees.
   */
  const stage = stages.find((candidate: Stage,) => candidate.id === EXTERNAL_COMPARE_STAGE_ID,) as Stage;
  if ((typeof stage.check) !== 'string')
    throw new Error(`stage ${EXTERNAL_COMPARE_STAGE_ID} has no plain check text`,);
  return {
    home,
    check: stage.check,
    compareFile: join(config.wiz, 'external-compare.txt',),
    differencesFile: join(config.wiz, 'external-differences.txt',),
  };
}

/**
 Runs the stage's check against a compare file holding the given lines.

 @param lines - the compare file's lines

 @returns the check's exit status and everything it printed

 @example
 ```ts
 const { status, printed, } = await check(withLines([DELETING_LINE]));
 ```
 */
async function check(
  {
    lines,
  }: {
    readonly lines: readonly string[];
  },
): Promise<{
  readonly home: HomeFolder;
  readonly status: number;
  readonly printed: string;
  readonly differences: string;
}> {
  /**
   The stage, its check and the files it reads.
   */
  const compare = await externalCompare();
  await writeFile(compare.compareFile, lines.length === 0 ? '' : `${lines.join('\n',)}\n`,);
  /**
   What the check printed.
   */
  const printed: string[] = [];
  /**
   The exit status of the check.
   */
  const status = await runShell({
    command: compare.check,
    outputFile: join(compare.home.path, 'output',),
    seconds: SHELL_DEADLINE_SECONDS,
    killGrace: SHELL_KILL_GRACE_SECONDS,
    outputFileEnv: join(compare.home.path, 'output',),
    out: (text: string,) => {
      printed.push(text,);
    },
  },);
  /**
   The lines the check kept as the differences it reports.
   */
  const differences = await readIfThere(compare.differencesFile,);
  return {
    home: compare.home,
    status,
    printed: printed.join('',),
    differences,
  };
}

/**
 Reads a file the check may not have written.

 @param path - the file

 @returns the text, or an empty string when the file does not exist

 @example
 ```ts
 const text = await readIfThere(path);
 ```
 */
async function readIfThere(path: string,): Promise<string> {
  try {
    return await readFile(path, 'utf8',);
  }
  catch {
    return '';
  }
}

await describe({
  name: 'the external-worktree check of the cut-over',
  children: [
    it({
      name: 'passes when the compare file is empty',
      fn: async () => {
        const result = await check({ lines: [], },);
        await result.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_PASSED,);
        expect(result.printed,).toContain('kept files deleted at the source after pass one: 0',);
        expect(result.differences,).toBe('',);
      },
    },),
    it({
      name: 'passes when the compare file holds only files deleted at the source',
      fn: async () => {
        const result = await check({ lines: [DELETING_LINE, DELETING_LINE,], },);
        await result.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_PASSED,);
        expect(result.printed,).toContain('kept files deleted at the source after pass one: 2',);
        expect(result.differences,).toBe('',);
      },
    },),
    it({
      name: 'fails on any other difference and reports it',
      fn: async () => {
        const result = await check({ lines: [DELETING_LINE, DIFFERENCE_LINE,], },);
        await result.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_FAILED,);
        expect(result.differences,).toBe(`${DIFFERENCE_LINE}\n`,);
        expect(result.printed,).toContain(DIFFERENCE_LINE,);
      },
    },),
    it({
      name: 'shows only the first differences it is set to show, without ending in a pipe signal',
      fn: async () => {
        /**
         More differences than the report shows, so the reader of the report stops before its writer does.
         */
        const differences = Array.from({ length: DIFFERENCES_SHOWN * 2, }, (_, index) => `${DIFFERENCE_LINE} ${String(index,)}`,);
        const result = await check({ lines: [DELETING_LINE, ...differences,], },);
        await result.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_FAILED,);
        expect(result.status,).not.toBe(KILLED_BY_PIPE,);
        expect(result.differences.split('\n',).filter(function isLine(line,) {
          return line !== '';
        },).length,).toBe(DIFFERENCES_SHOWN,);
      },
    },),
  ],
},);
