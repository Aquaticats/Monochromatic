/**
 Replaces the current process with the resolved terminal command.
 Uses `node:child_process` `spawn` with inherited stdio since Node lacks a native `execvp`.
 
 @module
 */

import { spawn, } from 'node:child_process';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'exec',
  l: parentLogger,
},);

/**
 Exit code for command-not-found errors.
 */
const EXIT_NOT_FOUND = 127;

/**
 Spawns the terminal command, inheriting all stdio, and exits with its exit code.
 This replaces the shell script's `exec "$\@"` pattern.
 
 @param command - Complete command array where `command[0]` is the executable.
 
 @throws Error when the command array is empty.
 
 @example
 ```ts
 execvp(\{ command: ['/usr/bin/ghostty', '--gtk-single-instance=true', '-e', 'bash'] \})
 ```
 */
export function execvp({ command, }: { readonly command: readonly string[]; },): void {
  if (command.length
    === 0)
    throw new Error('execvp: empty command array',);

  /**
   First token separated so the args slice below can pass the rest to `spawn`.
   */
  const [executable,] = command;
  if (executable === undefined)
    throw new Error('execvp: unreachable (length checked above)',);
  /**
   Arguments without the executable, ready to feed spawn's separate argv parameter.
   */
  const args = command.slice(1,);

  l.debug(`exec: ${executable} ${args.join(' ',)}`,);

  /**
   Owned spawn options isolated from command provenance.
   */
  const spawnOptions = { stdio: 'inherit', } as const;
  /**
   Spawned-process handle; its lifecycle events propagate the exit code.
   */
  const proc = spawn(
    executable,
    args,
    spawnOptions,
  );

  proc.on(
    'exit',
    function onExit(code,) {
      // A null code means the child was terminated by a signal; map that to the failure code.
      process.exitCode = code ?? EXIT_NOT_FOUND;
    },
  );
  proc.on(
    'error',
    /**
     Reports a process-spawn failure.
     
     @param err - Failure emitted by Node.
     
     @mutates err - `caughtValueText` may invoke string-conversion hooks.
     */
    function onError(err: unknown,): void {
      /**
       Failure detail preserving arbitrary thrown-value text.
       */
      const detail = caughtValueText(err,);
      console.error(`terminal-exec: failed to execute '${executable}': ${detail}`,);
      process.exitCode = EXIT_NOT_FOUND;
    },
  );
}
