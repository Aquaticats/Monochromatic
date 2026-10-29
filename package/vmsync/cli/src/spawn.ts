/**
 Subprocess execution helper wrapping nano-spawn with logging.
 
 @module
 */

import nanoSpawn from 'nano-spawn';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

/**
 Spawns a command and returns its trimmed stdout.
 Logs the command at debug level before execution.
 
 @param args - Arguments array for the command
 
 @param command - Command name or path to execute
 
 @returns Trimmed stdout output from the command
 
 @throws Error when command exits with non-zero code, including stderr
 
 @example
 ```ts
 const output = await spawn({ command: 'qemu-img', args: ['info', 'disk.qcow2'] });
 ```
 */
export async function spawn(
  {
    args,
    command,
  }: {
    readonly args: readonly string[];
    readonly command: string;
  },
): Promise<string> {
  /**
   Tagged logger so spawn entries are scoped to `spawn` in the output.
   */
  const rl = tagged({
    tag: spawn.name,
    l,
  },);
  rl.debug(`${command} ${args.join(' ',)}`,);

  /**
   Combined stdout from the child process; trimmed and returned by the caller.
   */
  const { stdout, } = await nanoSpawn(
    command,
    [...args,],
  );

  return stdout.trim();
}
