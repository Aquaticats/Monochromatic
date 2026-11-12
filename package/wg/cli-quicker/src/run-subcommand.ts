import type { WireguardConfig, } from './config-types.ts';
import { RECOMMENDED_EXEMPT_MARK, } from './exempt-mark-reservation.ts';
import { PROVIDER_HOSTS, } from './provider-hosts.data.ts';
import { UPDATE_HOSTS, } from './update-hosts.data.ts';
import { startNotices, } from './update-policy-notice.ts';

/**
 What a tunnel subcommand does besides printing,
 replaceable in tests.
 */
export type SubcommandSteps = {
  /**
   Brings the interface up.
   */
  readonly up: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Takes the interface down.
   */
  readonly down: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Prints one start notice or loud report where the person will see it.
   */
  readonly printNotice: (notice: string,) => void;
};

/**
 Runs the tunnel subcommand for a parsed config.

 A start prints every notice first and then brings the interface up,
 whatever the notices say:
 a config without `ExemptMark` and a config with `Table = off` are warned about and still come up,
 which is the owner's decision.
 A stop prints no start notice.

 @param subcommand - `up` or `down`.

 @param target - Interface name or config path the user gave.

 @param config - Parsed config.

 @param steps - Start,
 stop and print steps.

 @example
 ```ts
 await runSubcommand({ subcommand: 'up', target: 'wg0', config, steps });
 ```
 */
export async function runSubcommand(
  {
    subcommand,
    target,
    config,
    steps,
  }: {
    readonly subcommand: 'up' | 'down';
    readonly target: string;
    readonly config: WireguardConfig;
    readonly steps: SubcommandSteps;
  },
): Promise<void> {
  if (subcommand === 'down') {
    await steps.down({ config, },);
    return;
  }
  for (const notice of startNotices({
    target,
    ...(config.exemptMark === undefined ? {} : { exemptMark: config.exemptMark, }),
    tableOff: config.table === 'off',
    dnsServers: config.dns,
    hosts: UPDATE_HOSTS,
    providerHosts: PROVIDER_HOSTS,
    recommendedMark: RECOMMENDED_EXEMPT_MARK,
  },))
    steps.printNotice(notice,);
  await steps.up({ config, },);
}
