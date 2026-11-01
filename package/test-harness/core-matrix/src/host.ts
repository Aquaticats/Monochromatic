/**
 Host backend for matrix test execution.
 
 Runs test files directly on the host machine without containers.
 The specified runtime must already be installed and available on PATH.
 No prerequisite installation, no user creation.
 */

import spawn from 'nano-spawn';
import type {
  Combination,
  Runtime,
} from './types.ts';

//region Runtime binary resolution

/**
 Binary names for each runtime as found on PATH.
 Unlike the container backend which uses absolute `$HOME/.../bin/runtime` paths,
 the host backend relies on the runtime being globally installed.
 */
const RUNTIME_HOST_BIN: Record<Runtime, string> = {
  bun: 'bun',
  deno: 'deno',
  node: 'node',
};

/**
 Builds the arguments array for executing a file with the given runtime on the host.

 Node takes the file path directly; bun and deno select their `run` subcommand first.

 @param runtime - Runtime to use

 @param filePath - Absolute path to the file on the host

 @returns arguments array for `nano-spawn`
 */
function runtimeArgs({
  runtime,
  filePath,
}: {
  readonly runtime: Runtime;
  readonly filePath: string;
},): readonly string[] {
  if (runtime === 'node') {
    return [filePath,];
  }

  if (runtime === 'deno') {
    return [
      'run',
      '--allow-all',
      filePath,
    ];
  }

  return [
    'run',
    filePath,
  ];
}

//endregion Runtime binary resolution

/**
 Runs a single combination directly on the host.
 
 Spawns the runtime binary with the arguments built by {@link runtimeArgs}.
 The runtime must be pre-installed and available on PATH.
 The `user` axis is included in the combination for labeling
 but does not affect execution; the process runs as the current user.
 
 @param combination - Fully resolved combination to execute
 
 @returns stdout from the execution
 
 @throws Error when the runtime is not found on PATH or the test file fails
 
 @example
 ```ts
 const output = await runHost({
   combination: {
     file: '/path/to/test.ts',
     os: 'host:',
     user: 'user',
     runtime: 'bun',
   },
 });
 ```
 */
export async function runHost({
  combination,
}: {
  readonly combination: Combination;
},): Promise<string> {
  /**
   Resolved binary name on PATH; the host backend assumes a pre-installed runtime.
   */
  const bin = RUNTIME_HOST_BIN[combination.runtime];
  /**
   Argument array built once so the spawn call below stays declarative.
   */
  const args = runtimeArgs({
    runtime: combination.runtime,
    filePath: combination.file,
  },);

  /**
   Spawn result kept in a binding so stderr can be forwarded before returning stdout.
   */
  const result = await spawn(
    bin,
    [...args,],
  );

  if (result.stderr
    !== '')
    console.error(result.stderr,);

  return result.stdout;
}
