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
  incompleteMessage,
  leftoverMessage,
  reason,
  refusedMessage,
} from './tunnel-teardown-messages.ts';
import {
  blockTraffic,
  removeTrafficBlock,
  trafficBlockExists,
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
   Blocks all traffic that does not stay on the machine,
   except the clearnet traffic the routing sends around the tunnel by design (the exempt mark and the update hosts).
   It runs before any routing is removed,
   so a `down` that is killed part way leaves traffic blocked and not outside the tunnel.
   It fails, with the tunnel still up, when it cannot read what to let through.
   */
  readonly blockTraffic: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Removes the block.
   It runs once the interface is verified gone.
   */
  readonly removeTrafficBlock: (call: { readonly config: WireguardConfig; },) => Promise<void>;

  /**
   Tells whether the block of an earlier `down` is in place.
   */
  readonly trafficBlockPresent: (call: { readonly config: WireguardConfig; },) => Promise<boolean>;
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
  blockTraffic: async function blockTrafficStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    await blockTraffic({ interfaceName: config.interfaceName, },);
  },
  removeTrafficBlock: function removeTrafficBlockStep({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
    return removeTrafficBlock({ interfaceName: config.interfaceName, },);
  },
  trafficBlockPresent: function trafficBlockPresentStep({ config, }: { readonly config: WireguardConfig; },): Promise<boolean> {
    return trafficBlockExists({ interfaceName: config.interfaceName, },);
  },
};

/**
 Message for a refusal before any routing is removed by this attempt,
 truthful about an earlier `down` that did remove routing.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @param what - The step that failed.

 @param why - Why it failed.

 @param state - What this attempt did and did not change.

 @param fix - What to do about it.

 @returns The loud message.

 @example
 ```ts
 await refusal({ config, steps, what: 'stop the route watcher', why: 'x', state: 'nothing was removed', fix: 'y' });
 ```
 */
async function refusal(
  {
    config,
    steps,
    what,
    why,
    state,
    fix,
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
    readonly what: string;
    readonly why: string;
    readonly state: string;
    readonly fix: string;
  },
): Promise<string> {
  return refusedMessage({
    interfaceName: config.interfaceName,
    what,
    why,
    state,
    fix,
    earlierRemoval: await steps.trafficBlockPresent({ config, },),
  },);
}

/**
 Starts the route watcher again after a later step could not be done,
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
    return 'the route watcher was started again';
  }
  catch (restartError) {
    return `the route watcher was already stopped and could not be started again (${reason({ error: restartError, },)}), `
      + 'so the bypass table no longer follows the default route';
  }
}

/**
 Stops the application watcher and fails when it could not be stopped.
 A missing `wg-quicker-exempt` program is such a failure:
 a watcher started earlier may still run,
 and a tunnel is not taken down over a watcher that was not stopped.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @throws {@link Error} carrying the report when the program was not found.

 @example
 ```ts
 await stopApplicationWatcher({ config, steps });
 ```
 */
async function stopApplicationWatcher(
  {
    config,
    steps,
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
  },
): Promise<void> {
  /**
   What the stop led to.
   */
  const outcome = await steps.stopApplicationExemptions({ config, },);
  if (!outcome.ok)
    throw new Error(outcome.report,);
}

/**
 Lifts the traffic block once the interface is gone.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @returns Why the block could not be lifted,
 absent when it was lifted.

 @example
 ```ts
 await liftBlock({ config, steps });
 ```
 */
async function liftBlock(
  {
    config,
    steps,
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
  },
): Promise<{ readonly blockFailure?: string; }> {
  try {
    await steps.removeTrafficBlock({ config, },);
    return {};
  }
  catch (error) {
    l.error(`down ${config.interfaceName}: could not lift the traffic block: ${reason({ error, },)}`,);
    return { blockFailure: reason({ error, },), };
  }
}

/**
 Runs the steps after the interface is gone and collects their failures.
 They cannot expose anything:
 the failures are reported and the other steps still run.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @returns Words for each failure.

 @example
 ```ts
 await finishLeftovers({ config, steps });
 ```
 */
async function finishLeftovers(
  {
    config,
    steps,
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
  },
): Promise<readonly string[]> {
  /**
   Failures of the steps.
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
    ] as const
  ) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Each leftover step runs after the previous one finished.
      await step({ config, },);
    }
    catch (error) {
      l.error(`down ${config.interfaceName}: could not ${what}: ${reason({ error, },)}`,);
      leftovers.push(`could not ${what} (${reason({ error, },)})`,);
    }
  }
  return leftovers;
}

/**
 Tears the interface down in an order that keeps the tunnel up when a step that can fail fails,
 and blocks traffic from before the first removal until the interface is verified gone.

 The steps that can fail without having changed routing run first:
 the route watcher is stopped,
 then the application watcher (a missing `wg-quicker-exempt` program counts as a failure),
 then the traffic block is installed.
 If any of them fails,
 nothing has been removed (a route watcher that was already stopped is started again),
 the tunnel is still fully up,
 and the error says what to do.
 Only then is routing removed,
 the policy rules first.
 The block is already in place,
 so a step that fails,
 or a `down` that is killed or crashes,
 leaves traffic blocked and not outside the tunnel,
 and the error says how to finish.
 When the interface is verified gone the block is lifted.
 The steps after the interface is gone cannot expose anything;
 their failures are reported without a block,
 except that a block that cannot be lifted is reported as the first problem,
 because it stops the network.

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
      await refusal({
        config,
        steps,
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
    await stopApplicationWatcher({
      config,
      steps,
    },);
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
      await refusal({
        config,
        steps,
        what: 'stop the application watcher',
        why: reason({ error, },),
        state: `no routing was removed; ${restored}, so only the application watcher state is uncertain`,
        fix: `run \`sudo wg-quicker-exempt watch-stop ${iface}\` yourself and read its error; `
          + 'if the wg-quicker-exempt program is missing, put it back where wg-quicker looks for it (the error names the places)',
      },),
      { cause: error, },
    );
  }
  try {
    await steps.blockTraffic({ config, },);
  }
  catch (error) {
    l.error(`down ${iface}: the traffic block that protects the removal could not be installed: ${reason({ error, },)}`,);
    /**
     Whether the route watcher that was just stopped runs again.
     */
    const restored = await restoreRouteWatcher({
      config,
      steps,
    },);
    throw new TeardownRefusedError(
      await refusal({
        config,
        steps,
        what: 'install the traffic block that protects the removal of the routing',
        why: reason({ error, },),
        state: `no routing was removed; ${restored}; the application watcher stays stopped, so application bypass is not enforced until down is run again or the tunnel is brought up again`,
        fix: 'make sure `nft` works (for example `sudo nft list ruleset`) and that the application-bypass state and `ip rule show` can be read; '
          + 'the block lets through the traffic the routing sends around the tunnel, so it needs them',
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
    l.error(`down ${iface}: could not ${current.what}: ${reason({ error, },)}; traffic stays blocked`,);
    throw new TeardownIncompleteError(
      incompleteMessage({
        interfaceName: iface,
        what: current.what,
        why: reason({ error, },),
      },),
      { cause: error, },
    );
  }
  /**
   Why the block could not be lifted,
   absent when it was.
   */
  const lifted = await liftBlock({
    config,
    steps,
  },);
  /**
   Failures of the other steps after the interface is gone.
   */
  const leftovers = await finishLeftovers({
    config,
    steps,
  },);
  if ((lifted.blockFailure !== undefined) || (leftovers.length > 0))
    throw new TeardownIncompleteError(
      leftoverMessage({
        interfaceName: iface,
        leftovers,
        ...lifted,
      },),
    );
}
