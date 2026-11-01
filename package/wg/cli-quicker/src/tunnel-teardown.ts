import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  stopApplicationExemptions,
  type ExemptionOutcome,
} from './application-exemption.ts';
import type { WireguardConfig, } from './config.ts';
import {
  TeardownIncompleteError,
  TeardownRefusedError,
} from './errors.ts';
import { removeOpenSnitchEndpointAllowance, } from './opensnitch.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { removeExemptRule, } from './tunnel-bypass.ts';
import {
  restartRouteWatcher,
  stopRouteWatcher,
} from './tunnel-bypass-watcher-control.ts';
import { removeKillSwitch, } from './tunnel-firewall.ts';
import { linkExists, } from './tunnel-link.ts';
import { removePolicyRules, } from './tunnel-route.ts';
import {
  installTrafficBlock,
  removeTrafficBlock,
  trafficBlockTable,
} from './tunnel-traffic-block.ts';

/**
 Module logger for the teardown order.
 */
const l = tagged({ tag: 'tunnel-teardown', },);

/**
 The steps of a teardown,
 replaceable in tests.
 Each takes the interface lifecycle configuration.
 */
export type TeardownSteps = {
  /**
   Stops the route watcher and nothing else.
   */
  readonly stopRouteWatcher: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Starts the route watcher again.
   */
  readonly restartRouteWatcher: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Stops the application watcher of `wg-quicker-exempt`.
   */
  readonly stopApplicationExemptions: (call: { readonly config: WireguardConfig; },) => Promise<ExemptionOutcome>;

  /**
   Removes the policy routing rules that send traffic into the tunnel.
   */
  readonly removePolicyRules: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Removes the application-bypass routes,
   rules and persisted state.
   */
  readonly removeBypassState: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Removes the firewall kill switch of the tunnel.
   */
  readonly removeKillSwitch: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Deletes the interface and fails when it is still there.
   */
  readonly deleteLink: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Reverts the DNS settings of the interface.
   */
  readonly revertDns: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Removes the OpenSnitch allowance for the tunnel's endpoint.
   */
  readonly removeOpenSnitchAllowance: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Blocks all traffic that does not stay on the machine.
   */
  readonly blockTraffic: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Removes the block.
   */
  readonly removeTrafficBlock: (call: { readonly config: WireguardConfig; },) => Promise<void>;
};

/**
 Deletes the interface,
 and fails when it is still there afterwards:
 a delete that silently failed left a tunnel that the person believes is gone.

 @param config - Interface lifecycle configuration.

 @throws {@link Error} when the interface still exists after `ip link delete`.

 @example
 ```ts
 await deleteLinkVerified({ config });
 ```
 */
export async function deleteLinkVerified({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
  /**
   What `ip link delete` said.
   */
  const deleted = await runAllowingFailure({
    command: 'ip',
    args: [
      'link',
      'delete',
      'dev',
      config.interfaceName,
    ],
  },);
  if (await linkExists({ interfaceName: config.interfaceName, },))
    throw new Error(`the interface ${config.interfaceName} still exists after \`ip link delete\` (exit code ${String(deleted.exitCode,)}: ${deleted.stderr
      .trim()})`,);
}

/**
 The real steps.
 */
const PRODUCTION_STEPS: TeardownSteps = {
  stopRouteWatcher: function stopRouteWatcherStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return stopRouteWatcher({ interfaceName: config.interfaceName, },);
  },
  restartRouteWatcher: function restartRouteWatcherStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return restartRouteWatcher({ interfaceName: config.interfaceName, },);
  },
  stopApplicationExemptions: function stopApplicationExemptionsStep({ config, }: { readonly config: WireguardConfig; },): Promise<ExemptionOutcome> {
    return stopApplicationExemptions({
      interfaceName: config.interfaceName,
      configured: config.exemptMark !== undefined,
    },);
  },
  removePolicyRules: function removePolicyRulesStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return removePolicyRules({ interfaceName: config.interfaceName, },);
  },
  removeBypassState: function removeBypassStateStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return removeExemptRule({ interfaceName: config.interfaceName, },);
  },
  removeKillSwitch: function removeKillSwitchStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return removeKillSwitch({ interfaceName: config.interfaceName, },);
  },
  deleteLink: deleteLinkVerified,
  revertDns: async function revertDnsStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    if (config.dns
      .length
      === 0)
      return;
    await runAllowingFailure({
      command: 'resolvectl',
      args: [
        'revert',
        config.interfaceName,
      ],
    },);
  },
  removeOpenSnitchAllowance: function removeOpenSnitchAllowanceStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return removeOpenSnitchEndpointAllowance({ interfaceName: config.interfaceName, },);
  },
  blockTraffic: function blockTrafficStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return installTrafficBlock({ interfaceName: config.interfaceName, },);
  },
  removeTrafficBlock: function removeTrafficBlockStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return removeTrafficBlock({ interfaceName: config.interfaceName, },);
  },
};

/**
 Words for a caught value in a message.

 @param error - Caught value.

 @returns Its text.

 @example
 ```ts
 reason({ error: new Error('x') }); // 'Error: x'
 ```
 */
function reason({ error, }: { readonly error: unknown; },): string {
  return String(error,);
}

/**
 Message for a `down` that stopped before it changed anything.

 @param interfaceName - Tunnel interface.

 @param what - The step that failed.

 @param why - Why it failed.

 @param state - What was and was not changed.

 @param fix - What to do about it.

 @returns The loud message.

 @example
 ```ts
 refusedMessage({ interfaceName: 'wg0', what: 'stop the route watcher', why: 'x', state: 'nothing was removed', fix: 'y' });
 ```
 */
function refusedMessage(
  {
    interfaceName,
    what,
    why,
    state,
    fix,
  }: {
    readonly interfaceName: string;
    readonly what: string;
    readonly why: string;
    readonly state: string;
    readonly fix: string;
  },
): string {
  return `error: could not ${what} (${why})\n`
    + `the tunnel ${interfaceName} is still fully up; ${state}\n`
    + `fix: ${fix}, then run \`wg-quicker down ${interfaceName}\` again`;
}

/**
 Message for a `down` that failed after it started removing routing.

 @param interfaceName - Tunnel interface.

 @param what - The step that failed.

 @param why - Why it failed.

 @param blockFailure - Why traffic could not be blocked,
 absent when it is blocked.

 @returns The loud message.

 @example
 ```ts
 incompleteMessage({ interfaceName: 'wg0', what: 'delete the interface', why: 'x' });
 ```
 */
function incompleteMessage(
  {
    interfaceName,
    what,
    why,
    blockFailure,
  }: {
    readonly interfaceName: string;
    readonly what: string;
    readonly why: string;
    readonly blockFailure?: string;
  },
): string {
  /**
   Table that holds the block.
   */
  const table = trafficBlockTable({ interfaceName, },);
  /**
   What happens to traffic now.
   */
  const traffic = blockFailure === undefined
    ? `traffic is BLOCKED (nft table inet ${table} drops everything that leaves the machine), `
      + 'so nothing goes outside the tunnel unprotected, and the network will not work until down is finished'
    : `traffic could NOT be blocked (${blockFailure}), so it leaves OUTSIDE the tunnel now`;
  return `error: down ${interfaceName} failed part way: could not ${what} (${why})\n`
    + `${traffic}\n`
    + `to finish: run \`wg-quicker down ${interfaceName}\` again, with this build or with the build that started the tunnel; a down that succeeds removes the block\n`
    + `to lift the block by hand: sudo nft delete table inet ${table}`;
}

/**
 Starts the route watcher again after the application watcher could not be stopped,
 and says in words what state that leaves.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @returns Words for the message.

 @example
 ```ts
 await restoreRouteWatcher({ config, steps });
 ```
 */
async function restoreRouteWatcher(
  {
    config,
    steps,
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
  },
): Promise<string> {
  try {
    await steps.restartRouteWatcher({ config, },);
    return 'the route watcher was started again, so only the application watcher state is uncertain';
  }
  catch (restartError) {
    return `the route watcher was already stopped and could not be started again (${reason({ error: restartError, },)}), `
      + 'so the bypass table no longer follows the default route';
  }
}

/**
 Blocks traffic and says whether that worked.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @returns Why the block failed,
 absent when traffic is blocked.

 @example
 ```ts
 await tryBlockTraffic({ config, steps });
 ```
 */
async function tryBlockTraffic(
  {
    config,
    steps,
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
  },
): Promise<{ readonly blockFailure?: string; }> {
  try {
    await steps.blockTraffic({ config, },);
    return {};
  }
  catch (blockError) {
    l.error(`down ${config.interfaceName}: traffic could not be blocked: ${reason({ error: blockError, },)}`,);
    return { blockFailure: reason({ error: blockError, },), };
  }
}

/**
 Tears the interface down in an order that keeps the tunnel up when a step that can fail fails.

 The steps that can fail without having changed routing run first:
 the route watcher is stopped,
 then the application watcher.
 If either fails,
 nothing has been removed (a route watcher that was already stopped is started again),
 the tunnel is still fully up,
 and the error says what to do.
 Only then is routing removed,
 the policy rules first.
 If a step fails after that point traffic would go outside the tunnel from a tunnel the person believes is up,
 so traffic is blocked instead and the error says how to finish.
 The steps after the interface is gone cannot expose anything;
 their failures are reported without a block.

 @param config - Interface lifecycle configuration.

 @param steps - Steps,
 replaceable in tests.

 @throws {@link TeardownRefusedError} when a step before the routing removal failed.

 @throws {@link TeardownIncompleteError} when a step after it failed.

 @example
 ```ts
 await teardownKeepingTunnelOnFailure({ config });
 ```
 */
export async function teardownKeepingTunnelOnFailure(
  {
    config,
    steps = PRODUCTION_STEPS,
  }: {
    readonly config: WireguardConfig;
    readonly steps?: TeardownSteps;
  },
): Promise<void> {
  /**
   Interface torn down by this sequence.
   */
  const iface = config.interfaceName;
  try {
    await steps.stopRouteWatcher({ config, },);
  }
  catch (error) {
    l.error(`down ${iface}: the route watcher could not be stopped: ${reason({ error, },)}`,);
    throw new TeardownRefusedError(
      refusedMessage({
        interfaceName: iface,
        what: 'stop the route watcher',
        why: reason({ error, },),
        state: 'nothing was removed',
        fix: 'make sure the route watcher is the process wg-quicker started (its pid is in the .watcher.json record next to the state in /run/wg-quicker); '
          + `if it still fails, run \`wg-quicker down ${iface}\` with the build that started the tunnel`,
      },),
      { cause: error, },
    );
  }
  try {
    await steps.stopApplicationExemptions({ config, },);
  }
  catch (error) {
    l.error(`down ${iface}: the application watcher could not be stopped: ${reason({ error, },)}`,);
    /**
     Whether the route watcher that was just stopped runs again.
     */
    const restored = await restoreRouteWatcher({
      config,
      steps,
    },);
    throw new TeardownRefusedError(
      refusedMessage({
        interfaceName: iface,
        what: 'stop the application watcher',
        why: reason({ error, },),
        state: `no routing was removed; ${restored}`,
        fix: `run \`sudo wg-quicker-exempt watch-stop ${iface}\` yourself and read its error`,
      },),
      { cause: error, },
    );
  }
  /**
   The step being run,
   for the message when it fails.
   */
  const current = { what: 'remove the policy routing rules', };
  try {
    if (config.table !== 'off')
      await steps.removePolicyRules({ config, },);
    current.what = 'remove the application-bypass routes and rules';
    await steps.removeBypassState({ config, },);
    current.what = 'remove the firewall kill switch';
    await steps.removeKillSwitch({ config, },);
    current.what = 'delete the interface';
    await steps.deleteLink({ config, },);
  }
  catch (error) {
    l.error(`down ${iface}: could not ${current.what}: ${reason({ error, },)}; blocking traffic`,);
    /**
     Why traffic could not be blocked,
     absent when it is blocked.
     */
    const blocked = await tryBlockTraffic({
      config,
      steps,
    },);
    throw new TeardownIncompleteError(
      incompleteMessage({
        interfaceName: iface,
        what: current.what,
        why: reason({ error, },),
        ...blocked,
      },),
      { cause: error, },
    );
  }
  /**
   Failures of the steps after the interface is gone.
   */
  const leftovers: string[] = [];
  for (
    const [what, step,] of [
      [
        'revert the DNS settings',
        steps.revertDns,
      ],
      [
        'remove the OpenSnitch allowance',
        steps.removeOpenSnitchAllowance,
      ],
      [
        'remove the traffic block',
        steps.removeTrafficBlock,
      ],
    ] as const
  ) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Each leftover step runs after the previous one finished.
      await step({ config, },);
    }
    catch (error) {
      l.error(`down ${iface}: could not ${what}: ${reason({ error, },)}`,);
      leftovers.push(`could not ${what} (${reason({ error, },)})`,);
    }
  }
  if (leftovers.length > 0)
    throw new TeardownIncompleteError(
      `error: down ${iface} removed the tunnel but ${leftovers.join('; ',)}\n`
        + 'nothing is blocked and no routing is left over; finish these by hand',
    );
}
