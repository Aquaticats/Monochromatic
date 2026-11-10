/**
 Unit tests running the catalog-tighten entry point as a real process.

 These cases pin the user boundary for the startup failures: the message a
 workspace without a catalog produces, how many times it appears, and the exit
 code. A doubled message is invisible to any test that only greps for the text,
 so the count is what is asserted here.

 @module
 */

import {
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import {
  fileURLToPath,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  scratchRoot,
} from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import spawn, {
  SubprocessError,
} from 'nano-spawn';

//region Constants

/**
 Entry point under test, run the way the repository runs it (`node <path>`).
 */
const CLI_PATH = fileURLToPath(new URL('index.ts', import.meta.url,),);

/**
 Message the command reports when the workspace declares no catalog entries.
 */
const NO_CATALOG_MESSAGE = 'No catalog found in pnpm-workspace.yaml';

/**
 Workspace file content that parses but carries no `catalog:` block.
 */
const WORKSPACE_WITHOUT_CATALOG = "packages:\n  - 'package/*'\n";

/**
 Exit code a thrown startup error produces.
 */
const ERROR_EXIT_CODE = 1;

//endregion Constants

//region Helpers

/**
 Counts the lines that report the missing catalog, in either the bare form the
 command prints itself or the `Error:` form an uncaught throw adds. The thrown
 error's quoted source line mentions the message too, so only whole lines at
 the start of a report count.
 
 @param output - combined process output
 
 @returns number of lines reporting the missing catalog
 
 @example
 ```ts
 countMessageLines('Error: No catalog found in pnpm-workspace.yaml\n') // 1
 ```
 */
function countMessageLines(output: string,): number {
  return output.split('\n',)
    .filter(function isCatalogMessageLine(line,): boolean {
      return (line === NO_CATALOG_MESSAGE) || (line === `Error: ${NO_CATALOG_MESSAGE}`);
    },)
    .length;
}

/**
 Runs the entry point in a disposable workspace that declares no catalog, and
 keeps whatever ended the run.
 
 @param directory - disposable working directory and home
 
 @returns thrown subprocess failure, or the settled result when the run exited 0
 
 @example
 ```ts
 await caughtRun('/tmp/workspace-without-catalog');
 ```
 */
async function caughtRun(directory: string,): Promise<unknown> {
  try {
    return await spawn(
      process.execPath,
      [CLI_PATH,],
      {
        cwd: directory,
        env: { HOME: directory, },
        stdin: 'ignore',
      },
    );
  }
  catch (error: unknown) {
    return error;
  }
}

//endregion Helpers

await describe({
  name: 'catalog-tighten entry point',
  concurrency: 1,
  children: [
    it({
      name: 'reports a missing catalog once and exits 1',
      fn: async () => {
        /**
         Disposable workspace and home, so the run cannot read real config.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'catalog-tighten-cli-',),);
        await writeFile(
          join(
            directory.path,
            'pnpm-workspace.yaml',
          ),
          WORKSPACE_WITHOUT_CATALOG,
        );
        /**
         Whatever ended the run, expected to be the thrown startup error.
         */
        const caught = await caughtRun(directory.path,);
        expect(caught,).toBeInstanceOf(SubprocessError,);
        /**
         Failed run whose streams carry the reported message.
         */
        const failure = caught as SubprocessError;
        expect(failure.exitCode,).toBe(ERROR_EXIT_CODE,);
        expect(countMessageLines(failure.stderr,),).toBe(1,);
        expect(failure.stdout,).toBe('',);
      },
    },),
  ],
},);
