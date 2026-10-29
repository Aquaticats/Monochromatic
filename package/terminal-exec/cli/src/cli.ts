/**
 Parses CLI arguments for `xdg-terminal-exec`.
 Supports `--app-id=VALUE`, `--title=VALUE`, `--dir=VALUE`, `--hold`,
 and `--` or `-e` to delimit the command to execute.
 
 @module
 */
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { UserOptions, } from './build-command.ts';

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
  tag: 'cli',
  l: parentLogger,
},);

/**
 Parses process arguments into structured user options.
 Arguments before `--` or `-e` are xdg-terminal-exec options;
 arguments after are the command to execute in the terminal.
 
 @param argv - Raw process arguments (excluding `bun` and script path).
 
 @returns Parsed user options.
 
 @mutates argv - `JSON.stringify` may invoke array accessors or proxy traps.
 
 @example
 ```ts
 parseArgs({ argv: ['--title=My Shell', '--', 'bash', '-l'] })
 // { title: 'My Shell', command: ['bash', '-l'], appId: '', dir: '', hold: false }
 ```
 */
export function parseArgs({ argv, }: { argv: readonly string[]; },): UserOptions {
  /**
   Option accumulator mutated by the parse loop below; let because reassigned conditionally.
   */
  let appId = '';
  /**
   Same shape as appId; the parse loop populates each option in turn.
   */
  let title = '';
  /**
   Same shape as appId; the parse loop populates each option in turn.
   */
  let dir = '';
  /**
   Same shape as appId; the parse loop populates each option in turn.
   */
  let hold = false;
  /**
   Cursor into argv; advanced as options are consumed and used to slice the command remainder.
   */
  let i = 0;

  while (i < argv
    .length) {
    /**
     Current argv slot, scoped to the loop iteration.
     */
    const arg = argv[i];
    if (arg === undefined)
      break;

    if ((arg === '--') || (arg === '-e')) {
      l.debug(`found delimiter '${arg}'`,);
      i++;
      break;
    }

    if (arg.startsWith('--app-id=',)) {
      appId = arg.slice('--app-id='.length,);
      l.debug(`app-id='${appId}'`,);
      i++;
      continue;
    }

    if (arg.startsWith('--title=',)) {
      title = arg.slice('--title='.length,);
      l.debug(`title='${title}'`,);
      i++;
      continue;
    }

    if (arg.startsWith('--dir=',)) {
      dir = arg.slice('--dir='.length,);
      l.debug(`dir='${dir}'`,);
      i++;
      continue;
    }

    if (arg === '--hold') {
      hold = true;
      l.debug('hold=true',);
      i++;
      continue;
    }

    if (arg.startsWith('-',)) {
      l.debug(`ignoring unknown option '${arg}'`,);
      i++;
      continue;
    }

    /**
     First non-option argument starts the command.
     */
    l.debug(`found non-option '${arg}', treating as command start`,);
    break;
  }

  /**
   Remainder after the delimiter is the command to execute.
   */
  const command = argv.slice(i,);

  l.debug(
    `parsed: appId='${appId}', title='${title}', dir='${dir}', hold=${
      String(hold,)
    }, command=${JSON.stringify(command,)}`,
  );
  /**
   Bound to a local before return so the helper-shape allowlist accepts the function-root `let` declarations above.
   */
  const result: UserOptions = {
    appId,
    title,
    dir,
    hold,
    command,
  };
  return result;
}
