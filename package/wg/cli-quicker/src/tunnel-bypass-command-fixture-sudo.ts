/**
 Privileged commands of the disposable bypass integration fixture,
 which run through `sudo` and capture their output.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { text, } from 'node:stream/consumers';

import type { FixtureCommandResult, } from './tunnel-bypass-command-fixture-types.ts';

/**
 Runs privileged command and returns stdout.
 
 @param args - Arguments after `sudo`.
 
 @returns Captured UTF-8 stdout.
 
 @example
 ```ts
 await runSudo({ args: ['ip', 'netns', 'list'] });
 ```
 */
export async function runSudo(
  { args, }: { readonly args: readonly string[]; },
): Promise<string> {
  /**
   Fresh argument container sharing only immutable strings.
   */
  const isolatedArgs = args.map(function copyArgument(value,): string {
    return value;
  },);
  /**
   Captured command result checked at privilege boundary.
   */
  const result = await runSudoAllowingFailure({ args: isolatedArgs, },);
  if (result.exitCode !== 0) {
    throw new Error([
      `sudo ${isolatedArgs.join(' ',)} exited ${String(result.exitCode,)}.`,
      result.stdout,
      result.stderr,
    ].filter(Boolean,)
      .join('\n',),);
  }
  return result.stdout;
}

/**
 Runs privileged command while capturing nonzero result.
 
 @param args - Arguments after `sudo`.
 
 @returns Exit code and output streams.
 
 @example
 ```ts
 await runSudoAllowingFailure({ args: ['ip', 'netns', 'delete', 'missing'] });
 ```
 */
export async function runSudoAllowingFailure(
  { args, }: { readonly args: readonly string[]; },
): Promise<FixtureCommandResult> {
  /**
   Fresh argument container sharing only immutable strings.
   */
  const isolatedArgs = args.map(function copyArgument(value,): string {
    return value;
  },);
  /**
   Child with piped output and inherited execution environment.
   */
  const child = spawn(
    'sudo',
    isolatedArgs,
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  if ((child.stdout === null) || (child.stderr === null))
    throw new Error('Fixture command did not expose output streams.',);
  /**
   Concurrent stdout collection preventing pipe backpressure.
   */
  const stdoutPromise = text(child.stdout,);
  /**
   Concurrent stderr collection preventing pipe backpressure.
   */
  const stderrPromise = text(child.stderr,);
  await once(
    child,
    'close',
  );
  /**
   Exit code available after close event.
   */
  const { exitCode, } = child;
  if (exitCode === null)
    throw new Error('Fixture command closed without an exit code.',);
  /**
   Captured streams complete after child closes.
   */
  const [stdout, stderr,] = await Promise.all([
    stdoutPromise,
    stderrPromise,
  ],);
  return {
    exitCode,
    stdout,
    stderr,
  };
}

/**
 Writes root-owned fixture file without shell interpolation.
 
 @param path - Destination path.
 
 @param contents - UTF-8 fixture contents.
 
 @example
 ```ts
 await writeRootFixtureFile({ path: '/tmp/state', contents: '{}' });
 ```
 */
export async function writeRootFixtureFile(
  {
    path,
    contents,
  }: {
    readonly path: string;
    readonly contents: string;
  },
): Promise<void> {
  await runSudo({
    args: [
      process.execPath,
      '--input-type=module',
      '--eval',
      "const { writeFile } = await import('node:fs/promises'); await writeFile(process.argv[1], Buffer.from(process.argv[2], 'base64'));",
      path,
      Buffer.from(
        contents,
        'utf8',
      )
        .toString('base64',),
    ],
  },);
}
