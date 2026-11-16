/**
 Tests of where mvm writes its images and disks.

 A host whose `~/.local/share/mvm` cannot hold them has no other way to give mvm
 somewhere else, so the data directory has to follow the environment, and the
 default has to stay what it was when the environment says nothing.

 The value is computed once, when the library loads, so each case reads it from
 a process of its own rather than from this one.

 @module
 */

import { execFile, } from 'node:child_process';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { DATA_DIR_ENVIRONMENT, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Absolute path of the built library.
 */
const LIBRARY_PATH = fileURLToPath(new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
),);

/**
 What the variable holds in the child: a directory, or nothing at all.

 Absent is its own case rather than an empty text, because the two mean
 different things to mvm.
 */
type EnvironmentValue = {
  /**
   Discriminant separating a named directory from an unset variable.
   */
  readonly kind: 'named';

  /**
   The directory the variable names.
   */
  readonly directory: string;
} | {
  /**
   Discriminant separating an unset variable from a named directory.
   */
  readonly kind: 'unset';
};

/**
 Runs a program and answers its standard output.

 @param options.arguments - arguments after the program name

 @param options.environment - environment of the child

 @param options.program - program to run

 @returns the child's standard output with the program's own prefix removed
 */
async function runProgram(
  {
    arguments: programArguments,
    environment,
    program,
  }: {
    readonly arguments: readonly string[];
    readonly environment: NodeJS.ProcessEnv;
    readonly program: string;
  },
): Promise<string> {
  return await new Promise<string>(function collect(
    resolve,
    reject,
  ): void {
    execFile(
      program,
      [...programArguments,],
      { env: environment, },
      function report(
        error,
        stdout,
      ): void {
        if (error !== null) {
          reject(Error.isError(error) ? error : new Error(String(error,),),);
          return;
        }
        resolve(stdout,);
      },
    );
  },);
}

/**
 Reads the data directory a fresh process computes.

 @param value - what the data-directory variable holds in that process

 @returns the data directory that process reported
 */
async function dataDirOfFreshProcess(value: EnvironmentValue,): Promise<string> {
  /**
   Environment of the child: this process's own, with the variable set as asked.
   A copy that leaves the variable out is how the unset case is expressed, so
   nothing here removes a key.
   */
  const environment: NodeJS.ProcessEnv = {};
  for (const [name, text,] of Object.entries(process.env,)) {
    if ((value.kind === 'unset') && (name === DATA_DIR_ENVIRONMENT))
      continue;
    environment[name] = text;
  }
  if (value.kind === 'named') {
    environment[DATA_DIR_ENVIRONMENT] = value.directory;
  }
  /**
   The one value the child under test prints.
   */
  const printed = await runProgram({
    arguments: [
      '--input-type=module',
      '--eval',
      `const { DATA_DIR } = await import(${JSON.stringify(LIBRARY_PATH,)}); console.log(DATA_DIR);`,
    ],
    environment,
    program: process.execPath,
  },);
  return printed.trim();
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: DATA_DIR_ENVIRONMENT,
      children: [
        //region The environment names the data directory

        it({
          name: 'writes below the directory the environment names',
          fn: async () => {
            /**
             Path this test gives mvm.
             */
            const named = join(
              scratchRoot(),
              'mvm-config-unit-named',
            );
            expect(await dataDirOfFreshProcess({
              kind: 'named',
              directory: named,
            },),).toBe(named,);
          },
        },),

        //endregion The environment names the data directory

        //region Without it the user's data directory is used

        it({
          name: 'writes below the user data directory when the environment says nothing',
          fn: async () => {
            expect(await dataDirOfFreshProcess({ kind: 'unset', },),).toBe(
              join(
                homedir(),
                '.local',
                'share',
                'mvm',
              ),
            );
          },
        },),

        //endregion Without it the user's data directory is used
      ],
    },),
  ],
},);
