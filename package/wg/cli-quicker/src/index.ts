#!/usr/bin/env node

/**
 CLI entry point for bringing a WireGuard interface up or down without the
 quadratic bash config parsing that makes `wg-quick` hang on a large
 `AllowedIPs` value.
 
 @example
 ```sh
 wg-quicker up mx-que-mx1
 wg-quicker down mx-que-mx1
 wg-quicker opensnitch-rules apply
 ```

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { resolveApplicationExemptionCommand, } from './application-exemption-command.ts';
import { loadConfig, } from './config.ts';
import { CliUsageError, } from './errors.ts';
import { RECOMMENDED_EXEMPT_MARK, } from './exempt-mark-reservation.ts';
import { runOpenSnitchRulesCommand, } from './opensnitch-rules-command.ts';
import { UPDATE_HOSTS, } from './update-hosts.data.ts';
import { startNotices, } from './update-policy-notice.ts';
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
 Subcommand maintaining the daemon rules that let toolchain update traffic through OpenSnitch.
 */
const OPENSNITCH_RULES_SUBCOMMAND = 'opensnitch-rules';

/**
 Recognized tunnel subcommands.
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
  if (processArguments[0] === OPENSNITCH_RULES_SUBCOMMAND) {
    /**
     Whether non-root process handed the rule maintenance to a sudo child.
     */
    const rulesDelegated = await relaunchWithRootIfNeeded();
    if (!rulesDelegated) {
      await runOpenSnitchRulesCommand({ argv: processArguments.slice(1,), },);
    }
    return;
  }
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
    for (const notice of startNotices({
      target,
      ...(config.exemptMark === undefined ? {} : { exemptMark: config.exemptMark, }),
      tableOff: config.table === 'off',
      dnsServers: config.dns,
      hosts: UPDATE_HOSTS,
      recommendedMark: RECOMMENDED_EXEMPT_MARK,
    },))
      l.warn(notice,);
    if (config.exemptMark !== undefined) {
      /*
       The companion must exist before the first network mutation,
       so a missing watcher stops `up` instead of leaving the process policy unenforced.
       */
      await resolveApplicationExemptionCommand();
    }
  }
  await (subcommand === 'up' ? up({ config, },) : down({ config, },));
}

await main();
