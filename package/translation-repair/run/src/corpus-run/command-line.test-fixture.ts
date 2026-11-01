/**
 A runner's command line read the way `reportingRefusals` reads it, for the
 tests of the readers that take one (ledger B75). Each case names the runner
 whose line it reads, so it exercises that runner's real declaration rather
 than one written for the test.

 @module
 */

import {
  COMMAND_LINES,
  type CommandLineOf,
  type CommandName,
  readCommandLine,
} from '../../dist/final/node/index.mjs';

/**
 Runtime path every built command line starts with.
 */
const RUNTIME = '/usr/bin/node';

/**
 Reads what a person typed after a runner, against that runner's declaration.

 @param command - runner whose declaration the line is read against

 @param typed - arguments after the script path

 @returns The line its body would receive

 @throws StatedRefusalError when the runner would refuse the line

 @example
 ```ts
 const line = lineOf({ command: 'corpus-pass', typed: ['--only', 'Tabby_01',], },);
 ```
 */
export function lineOf<const Command extends CommandName>(
  {
    command,
    typed,
  }: {
    readonly command: Command;
    readonly typed: readonly string[];
  },
): CommandLineOf<Command> {
  return readCommandLine({
    command,
    spec: COMMAND_LINES[command],
    argv: [
      RUNTIME,
      `dist/final/node/${command}.mjs`,
      ...typed,
    ],
  },);
}
