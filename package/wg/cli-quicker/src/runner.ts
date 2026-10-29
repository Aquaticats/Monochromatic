import {
  spawn as spawnChild,
} from 'node:child_process';
import { once, } from 'node:events';
import { text, } from 'node:stream/consumers';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { CommandError, } from './errors.ts';

/**
 Module logger for external command execution.
 */
const l = tagged({ tag: 'runner', },);

/**
 Result of a successfully executed external command.
 */
export type CommandResult = {
  /**
   Captured standard output decoded as UTF-8.
   */
  readonly stdout: string;

  /**
   Captured standard error decoded as UTF-8.
   */
  readonly stderr: string;
};

/**
 Renders an argument list for logging without reading a caller-owned array.
 
 @param command - Executable name.
 
 @param args - Arguments passed to the executable.
 
 @returns Single-line rendering of the invocation.
 
 @example
 ```ts
 renderInvocation({ command: 'ip', args: ['link', 'show'] });
 ```
 */
function renderInvocation(
  {
    command,
    args,
  }: {
    readonly command: string;
    readonly args: readonly string[];
  },
): string {
  /**
   Fresh copy so joining never touches a caller-owned array.
   */
  const rendered: readonly string[] = [
    ...args,
  ];
  return `${command} ${rendered.join(' ',)}`;
}

/**
 Runs one external command, capturing output and throwing on non-zero exit.
 
 Input can be piped to standard in, which is how the raw peer config reaches
 `wg addconf` without an intermediate file.
 
 @param command - Executable name resolved through `PATH`.
 
 @param args - Arguments passed to the command.
 
 @param input - Optional text written to the command's standard in.
 
 @returns Captured stdout and stderr.
 
 @throws {@link CommandError} when the command exits non-zero or closes without a code.
 
 @example
 ```ts
 await run({ command: 'ip', args: ['link', 'show', 'dev', 'wg0'] });
 ```
 */
export async function run(
  {
    command,
    args,
    input,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly input?: string;
  },
): Promise<CommandResult> {
  /**
   Function-scoped logger for one command invocation.
   */
  const fl = tagged({
    tag: run.name,
    l,
  },);
  fl.debug(`exec ${renderInvocation({
    command,
    args,
  },)}`,);
  /**
   Spawned child with piped streams for capture.
   */
  const child = spawnChild(
    command,
    [
      ...args,
    ],
    {
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Swallows `EPIPE` from a child that closes standard in early (for example a
   rejection), so the real error surfaces via the child's exit code instead of
   an unhandled stream error.
   
   @param error - Stream error from the standard-in pipe.
   */
  function onStdinError(error: Readonly<Error & { readonly code?: string; }>,): void {
    if (error.code !== 'EPIPE')
      fl.error(`stdin error for ${command}: ${error.message}`,);
  }
  child.stdin
    .on(
    'error',
    onStdinError,
    );
  if (input !== undefined)
    child.stdin
      .write(input,);
  child.stdin
    .end();
  /**
   Captured stdout, stderr, and close event awaited concurrently.
   */
  const [stdout, stderr,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(
      child,
      'close',
    ),
  ],);
  /**
   Numeric exit code established by the close event.
   */
  const { exitCode, } = child;
  if (exitCode !== 0) {
    fl.error(`${command} exited ${String(exitCode ?? (-1),)}: ${stderr}`,);
    throw new CommandError({
      command,
      args,
      exitCode: exitCode ?? (-1),
      stderr,
    },);
  }
  return {
    stdout,
    stderr,
  };
}

/**
 Runs a command that is allowed to fail, returning its exit code instead of throwing.
 
 Used for idempotent teardown and for probes where absence is expected.
 
 @param command - Executable name resolved through `PATH`.
 
 @param args - Arguments passed to the command.
 
 @returns Exit code and captured output.
 
 @example
 ```ts
 await runAllowingFailure({ command: 'ip', args: ['link', 'delete', 'dev', 'wg0'] });
 ```
 */
export async function runAllowingFailure(
  {
    command,
    args,
  }: {
    readonly command: string;
    readonly args: readonly string[];
  },
): Promise<CommandResult & { readonly exitCode: number; }> {
  /**
   Function-scoped logger for one tolerated invocation.
   */
  const fl = tagged({
    tag: runAllowingFailure.name,
    l,
  },);
  fl.debug(`exec (allowed to fail) ${renderInvocation({
    command,
    args,
  },)}`,);
  /**
   Spawned child with standard in closed and output captured.
   */
  const child = spawnChild(
    command,
    [
      ...args,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Captured stdout and stderr awaited alongside the close event.
   */
  const [stdout, stderr,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(
      child,
      'close',
    ),
  ],);
  return {
    stdout,
    stderr,
    exitCode: child.exitCode ?? (-1),
  };
}
