/**
 A runs folder holding the runs a test describes, in a folder the test throws away. No test reads the real runs folder, and no
 test needs a unit: a look is handed the units it judges.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 One entry a test puts in its runs folder.
 */
export type RunFixture = {
  /**
   The entry's name.
   */
  readonly name: string;

  /**
   The summary to write, as the driver would write it; absent for a run that has not finished.
   */
  readonly summary?: Readonly<Record<string, unknown>>;

  /**
   The feed lines to write, each as one JSON line; absent for a run that wrote no feed.
   */
  readonly feedLines?: readonly Readonly<Record<string, unknown>>[];

  /**
   The feed to write as it is, for lines no JSON writer produces, such as one cut in half.
   */
  readonly feedText?: string;

  /**
   Whether the entry is a plain file instead of a folder, which a runs folder may hold by mistake.
   */
  readonly isAPlainFile?: boolean;
};

/**
 A runs folder a test describes its runs in.
 */
export type DisposableRunsFolder = {
  /**
   The folder.
   */
  readonly path: string;

  /**
   The path of one entry in it.
   */
  readonly entryPathOf: (name: string) => string;

  /**
   Removes the folder and everything in it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Writes one entry of a runs folder.

 @param runsFolder - the folder the entry belongs in

 @param run - the entry to write

 @example
 ```ts
 await writeRun({ runsFolder: '/runs', run: { name: 'a-1', summary: { totalMilliseconds: 5, }, }, });
 ```
 */
async function writeRun({
  runsFolder,
  run,
}: {
  readonly runsFolder: string;
  readonly run: RunFixture;
},): Promise<void> {
  /**
   Where the entry goes.
   */
  const path = join(
    runsFolder,
    run.name,
  );
  if (run.isAPlainFile === true) {
    await writeFile(
      path,
      'not a run',
      { encoding: 'utf8', },
    );
    return;
  }
  await mkdir(
    path,
    { recursive: true, },
  );
  if (run.summary !== undefined) {
    await writeFile(
      join(
        path,
        'summary.json',
      ),
      JSON.stringify(run.summary,),
      { encoding: 'utf8', },
    );
  }
  if (run.feedLines !== undefined) {
    await writeFile(
      join(
        path,
        'feed.jsonl',
      ),
      run.feedLines.map(function asLine(line,) {
        return `${JSON.stringify(line,)}\n`;
      },)
        .join('',),
      { encoding: 'utf8', },
    );
  }
  if (run.feedText !== undefined) {
    await writeFile(
      join(
        path,
        'feed.jsonl',
      ),
      run.feedText,
      { encoding: 'utf8', },
    );
  }
}

/**
 Writes a runs folder with the runs a test describes.

 @param runs - the entries to write

 @returns the folder, and how to remove it

 @example
 ```ts
 await using runs = await standInRunsFolder({ runs: [{ name: 'a-1', summary: { totalMilliseconds: 5, }, },], },);
 ```
 */
export async function standInRunsFolder({
  runs,
}: {
  readonly runs: readonly RunFixture[];
},): Promise<DisposableRunsFolder> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-overdue-runs-',),);
  await Promise.all(runs.map(async function writeOne(run,) {
    await writeRun({
      runsFolder: path,
      run,
    },);
  },));
  return {
    path,
    entryPathOf: function entryPathOf(name,) {
      return join(
        path,
        name,
      );
    },
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}
