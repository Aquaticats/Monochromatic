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

import { parseCliArgs, } from './cli-args.ts';
import { loadConfig, } from './config.ts';
import { runOpenSnitchRulesCommand, } from './opensnitch-rules-command.ts';
import { restorePrivilegeContext, } from './privilege-context.ts';
import { relaunchWithRootIfNeeded, } from './privilege.ts';
import { runSubcommand, } from './run-subcommand.ts';
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
}
else {
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
  if (!delegated) {
    /**
     Parsed config for requested interface.
     */
    const config = await loadConfig({
      arg: target,
      expandAllowedIps: subcommand === 'up',
    },);
    await runSubcommand({
      subcommand,
      target,
      config,
      steps: {
        up,
        down,
        warn: function warnNotice(notice,): void {
          l.warn(notice,);
        },
      },
    },);
  }
}
