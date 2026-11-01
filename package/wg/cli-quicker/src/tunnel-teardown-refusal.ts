import type { WireguardConfig, } from './config.ts';
import {
  reason,
  refusedMessage,
} from './tunnel-teardown-messages.ts';
import type { TeardownSteps, } from './tunnel-teardown.ts';

/**
 Message for a refusal before any routing is removed by this attempt,
 truthful about an earlier `down` that did remove routing.

 @param config - Interface lifecycle configuration.

 @param steps - Steps.

 @param what - The step that failed.

 @param why - Why it failed.

 @param state - What this attempt did and did not change.

 @param fix - What to do about it.

 @param manualSteps - Lines of exact commands to put under the message,
 for a person to finish without wg-quicker.

 @returns The loud message.

 @example
 ```ts
 await refusal({ config, steps, what: 'stop the route watcher', why: 'x', state: 'nothing was removed', fix: 'y' });
 ```
 */
export async function refusal(
  {
    config,
    steps,
    what,
    why,
    state,
    fix,
    manualSteps = [],
  }: {
    readonly config: WireguardConfig;
    readonly steps: TeardownSteps;
    readonly what: string;
    readonly why: string;
    readonly state: string;
    readonly fix: string;
    readonly manualSteps?: readonly string[];
  },
): Promise<string> {
  return refusedMessage({
    interfaceName: config.interfaceName,
    what,
    why,
    state,
    fix,
    manualSteps,
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
export async function restoreRouteWatcher(
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
