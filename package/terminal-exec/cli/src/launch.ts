/**
 Library entry point for launching a terminal from other packages.
 
 Unlike the CLI entry which replaces the current process via {@link execvp},
 {@link launchTerminal} spawns the terminal as a detached child and returns
 once it has started.
 
 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { buildCommand, } from './build-command.ts';
import { resolveTerminal, } from './resolve.ts';
import { NO_TERMINAL, } from './validate.ts';

export { buildCommand, } from './build-command.ts';
export { expandEscapes, } from './desktop-entry-types.ts';
export { stripTrailingSlashes, } from './xdg-paths.ts';

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
  tag: 'launch',
  l: parentLogger,
},);

/**
 Resolves the preferred terminal emulator via {@link resolveTerminal} and
 launches it as a detached process, building the final argv through
 {@link buildCommand}.
 
 @param dir - directory to open the terminal in
 
 @param command - optional command and arguments to execute in the terminal
 
 @param title - optional window title
 
 @throws when {@link resolveTerminal} returns {@link NO_TERMINAL} or the process fails to spawn
 
 @example
 ```ts
 await launchTerminal({ dir: '/home/user/project' });
 await launchTerminal({ dir: '/tmp', command: ['bash', '-l'] });
 ```
 */
export async function launchTerminal({
  dir,
  command = [],
  title = '',
}: {
  readonly dir: string;
  readonly command?: readonly string[];
  readonly title?: string;
},): Promise<void> {
  /**
   Platform-specific resolution; the NO_TERMINAL path raises a user-facing error below.
   */
  const terminal = await resolveTerminal();

  if (terminal === NO_TERMINAL) {
    throw new Error(
      process.platform
        === 'win32'
        ? 'No terminal emulator found.'
        : 'No terminal emulator found. Install a terminal emulator or configure one in ~/.config/xdg-terminals.list',
    );
  }

  l.info(`resolved terminal: ${terminal.entryId}`,);

  /**
   Final command array fed to spawn; built from the terminal entry and user options.
   */
  const argv = buildCommand({
    terminal,
    options: {
      appId: '',
      title,
      dir,
      hold: false,
      command,
    },
  },);

  if (argv.length
    === 0)
    throw new Error('launchTerminal: buildCommand returned empty argv',);

  /**
   Splits argv to feed spawn's separate executable/args parameters.
   */
  const [executable, ...args] = argv;

  l.info(`launching: ${String(executable,)} ${args.join(' ',)}`,);

  /**
   Detached child reference; the Node event helper rejects if spawn fails.
   */
  const child = spawn(
    String(executable,),
    args,
    {
      cwd: dir,
      detached: true,
      stdio: 'ignore',
    },
  );
  await once(
    child,
    'spawn',
  );
  child.unref();
}
