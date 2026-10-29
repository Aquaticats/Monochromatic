#!/usr/bin/env node

/**
 CLI entry point for bringing a WireGuard interface up or down without the
 quadratic bash config parsing that makes `wg-quick` hang on a large
 `AllowedIPs` value.
 
 @example
 ```sh
 wg-quicker up mx-que-mx1
 wg-quicker down mx-que-mx1
 ```
 
 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { resolveApplicationExemptionCommand, } from './application-exemption-command.ts';
import { loadConfig, } from './config.ts';
import { CliUsageError, } from './errors.ts';
import { RECOMMENDED_EXEMPT_MARK, } from './exempt-mark-reservation.ts';
import { restorePrivilegeContext, } from './privilege-context.ts';
import { relaunchWithRootIfNeeded, } from './privilege.ts';
import {
  down,
  up,
} from './tunnel.ts';

/**
 Module logger for command lifecycle.
 */
const l = tagged({ tag: 'wg-quicker', },);

/**
 Recognized subcommands.
 */
type Subcommand = 'up' | 'down';

/**
 Parsed command-line contract.
 */
type CliArgs = {
  readonly subcommand: Subcommand;
  readonly target: string;
};

/**
 Parses the two-positional-argument contract: `<up|down> <interface|path>`.
 
 @param argv - Arguments after runtime and script path.
 
 @returns Validated subcommand and target.
 
 @throws {@link CliUsageError} when arguments are missing or the subcommand is unknown.
 
 @example
 ```ts
 parseCliArgs({ argv: ['up', 'mx-que-mx1'] });
 ```
 */
function parseCliArgs({ argv, }: { readonly argv: readonly string[]; },): CliArgs {
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
  return {
    subcommand,
    target,
  };
}

/**
 Validates invocation,
 crosses privilege boundary,
 then runs requested tunnel operation.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Arguments after runtime and script path.
   */
  const processArguments = await restorePrivilegeContext();
  /**
   Parsed subcommand and config target.
   */
  const {
    subcommand,
    target,
  } = parseCliArgs({ argv: processArguments, },);
  l.debug(`${subcommand} ${target}`,);
  /**
   Whether non-root process handed entire operation to sudo child.
   */
  const delegated = await relaunchWithRootIfNeeded();
  if (delegated)
    return;
  /**
   Parsed config for requested interface.
   */
  const config = await loadConfig({
    arg: target,
    expandAllowedIps: subcommand === 'up',
  },);
  if (subcommand === 'up') {
    if (config.exemptMark === undefined) {
      l.warn(
        `Config for ${target} has no ExemptMark; Ghostty, Steam, Helium, Pale Moon, Firefox Nightly, ChatGPT, Interpreter, and Qure will use the tunnel. Add \`ExemptMark = ${String(RECOMMENDED_EXEMPT_MARK,)}\` under \`[Interface]\`, then bring this interface down and up again so application exemptions attach.`,
      );
    } else {
      /**
       Exact companion path resolved before first network mutation.
       */
      const command = await resolveApplicationExemptionCommand();
      /**
       Process environment updated with exact preflighted path for later watcher calls.
       */
      const environment = process.env;
      environment.WG_QUICKER_EXEMPT_COMMAND = command;
    }
  }
  await (subcommand === 'up' ? up({ config, },) : down({ config, },));
}

await main();
