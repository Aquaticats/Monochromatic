import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { CliUsageError, } from './errors.ts';
import {
  assertInterfaceName,
  interfaceNameOfArgument,
} from './interface-name.ts';

/**
 Module logger for command-line parsing.
 */
const l = tagged({ tag: 'cli-args', },);

/**
 Recognized tunnel subcommands.
 */
export type Subcommand = 'up' | 'down';

/**
 Parsed command-line contract.
 */
export type CliArgs = {
  readonly subcommand: Subcommand;
  readonly target: string;
};

/**
 Parses the two-positional-argument contract: `<up|down> <interface|path>`.
 The interface name the target stands for is checked here,
 before the process asks for privilege and before any command, file name or nft table is built from it.

 @param argv - Arguments after runtime and script path.

 @returns Validated subcommand and target.

 @throws {@link CliUsageError} when arguments are missing or the subcommand is unknown.

 @throws {@link ConfigError} when the target stands for a name that cannot be an interface name.

 @example
 ```ts
 parseCliArgs({ argv: ['up', 'mx-que-mx1'] });
 ```
 */
export function parseCliArgs({ argv, }: { readonly argv: readonly string[]; },): CliArgs {
  /**
   Positional subcommand and config target extracted from argv.
   */
  const [subcommand, target,] = argv;
  if ((subcommand === undefined) || ((subcommand !== 'up') && (subcommand !== 'down'))) {
    l.error('first argument must be up or down',);
    throw new CliUsageError('Usage: wg-quicker <up|down> <interface|config-path>',);
  }
  if (target === undefined) {
    l.error('missing interface or config path',);
    throw new CliUsageError('Usage: wg-quicker <up|down> <interface|config-path>',);
  }
  assertInterfaceName({ interfaceName: interfaceNameOfArgument({ arg: target, },), },);
  return {
    subcommand,
    target,
  };
}
