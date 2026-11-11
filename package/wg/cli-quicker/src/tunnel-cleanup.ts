import {
  stopApplicationExemptions,
  type ExemptionOutcome,
} from './application-exemption.ts';
import type { WireguardConfig, } from './config.ts';
import { runAllowingFailure, } from './runner.ts';
import { removeExemptRule, } from './tunnel-bypass.ts';
import { removeKillSwitch, } from './tunnel-firewall.ts';
import { linkExists, } from './tunnel-link.ts';
import { removeOpenSnitchEndpointAllowance, } from './opensnitch.ts';
import { removePolicyRules, } from './tunnel-route.ts';

/**
 Removes application watcher, routes, rules, firewall, link, and DNS without hooks.
 
 Shared by failed-up rollback and hook-wrapped down path.

 Only the two steps that read the link need it to be there:
 the policy rules are read from the interface's own fwmark,
 and the interface is what the last step deletes.
 Everything else outlives the link.
 Another tool can delete a link while the application bypass is active,
 and the bypass rules,
 the copied routes,
 the ownership state,
 the route watcher and the kill switch are all still there;
 a cleanup that stopped at the missing link left them behind and kept host rules
 selecting the physical bypass table.

 Stops the application watcher and reports a failure instead of ending the cleanup there:
 a watcher that cannot be stopped still leaves routing and state that must be removed,
 and the caller says so in the words of the report.

 @param config - Interface lifecycle configuration.

 @returns What stopping the application watcher led to.

 @example
 ```ts
 await cleanup({ config });
 ```
 */
export async function cleanup(
  { config, }: { readonly config: WireguardConfig; },
): Promise<ExemptionOutcome> {
  /**
   Interface whose state is removed.
   */
  const iface = config.interfaceName;
  await removeOpenSnitchEndpointAllowance({ interfaceName: iface, },);
  /**
   What stopping the application watcher led to,
   which the caller reports when it is a failure.
   */
  const exemptions = await stopExemptionsOrReport({ config, },);
  /**
   Whether the link is still there,
   which decides the two steps that need it.
   */
  const present = await linkExists({ interfaceName: iface, },);
  if (present && (config.table !== 'off'))
    await removePolicyRules({ interfaceName: iface, },);
  await removeExemptRule({ interfaceName: iface, });
  await removeKillSwitch({ interfaceName: iface, },);
  if (present)
    await runAllowingFailure({
      command: 'ip',
      args: [
        'link',
        'delete',
        'dev',
        iface,
      ],
    },);
  if (config.dns
    .length
    > 0)
    await runAllowingFailure({
      command: 'resolvectl',
      args: [
        'revert',
        iface,
      ],
    },);
  return exemptions;
}

/**
 Stops the application watcher of the interface,
 and turns a failure into the report the caller shows.

 @param config - Interface lifecycle configuration.

 @returns What the stop led to.

 @example
 ```ts
 await stopExemptionsOrReport({ config });
 ```
 */
async function stopExemptionsOrReport(
  { config, }: { readonly config: WireguardConfig; },
): Promise<ExemptionOutcome> {
  try {
    return await stopApplicationExemptions({
      interfaceName: config.interfaceName,
      configured: config.exemptMark !== undefined,
    },);
  }
  catch (error: unknown) {
    return {
      ok: false,
      report: `the application watcher of ${config.interfaceName} could not be stopped: ${String(error,)}`,
    };
  }
}
