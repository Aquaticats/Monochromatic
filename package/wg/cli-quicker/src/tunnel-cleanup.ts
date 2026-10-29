import { stopApplicationExemptions, } from './application-exemption.ts';
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
 
 @param config - Interface lifecycle configuration.
 
 @example
 ```ts
 await cleanup({ config });
 ```
 */
export async function cleanup(
  { config, }: { readonly config: WireguardConfig; },
): Promise<void> {
  /**
   Interface whose state is removed.
   */
  const iface = config.interfaceName;
  await removeOpenSnitchEndpointAllowance({ interfaceName: iface, },);
  await stopApplicationExemptions({
    interfaceName: iface,
    configured: config.exemptMark !== undefined,
  },);
  if (!(await linkExists({ interfaceName: iface, },)))
    return;
  if (config.table !== 'off')
    await removePolicyRules({ interfaceName: iface, },);
  await removeExemptRule({ interfaceName: iface, });
  await removeKillSwitch({ interfaceName: iface, },);
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
}
