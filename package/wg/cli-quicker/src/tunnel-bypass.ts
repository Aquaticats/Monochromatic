import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { BypassRouteError, } from './errors.ts';
import { runIpDelete, } from './ip-delete.ts';
import { run, } from './runner.ts';
import {
  BYPASS_STATE_ABSENT,
  claimBypassState,
  persistBypassState,
  readBypassState,
  releaseBypassState,
} from './tunnel-bypass-state.ts';
import {
  claimBypassAllocationOperation as claimBypassAllocationOperationInternal,
  claimBypassInterfaceOperation as claimBypassInterfaceOperationInternal,
} from './tunnel-bypass-operation-lock.ts';
import {
  readPhysicalDefaults,
  removeOwnedBypassRoutes,
  synchronizeBypassRoutes as synchronizeBypassRoutesInternal,
} from './tunnel-bypass-route.ts';
import {
  BYPASS_PROTOS,
  BYPASS_ROUTE_PROTOCOL,
  type BypassState,
} from './tunnel-bypass-types.ts';
import {
  startBypassWatcher,
  stopBypassWatcher,
} from './tunnel-bypass-watch-service.ts';

export { processArgumentsMatch, } from './linux-process-identity.ts';
export {
  claimBypassAllocationOperation,
  claimBypassInterfaceOperation,
} from './tunnel-bypass-operation-lock.ts';
export { synchronizeBypassRoutes, } from './tunnel-bypass-route.ts';
export { readBypassStatePath, } from './tunnel-bypass-state.ts';

/**
 Module logger for bypass ownership lifecycle.
 */
const l = tagged({ tag: 'tunnel-bypass', },);

/**
 Adds exact owned mark rule for each address family.
 
 @param state - Persisted mark,
 table,
 preference,
 and protocol ownership.
 
 @example
 ```ts
 await addBypassRules({ state });
 ```
 */
async function addBypassRules(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  /**
   Primitive rule fields copied from caller state.
   */
  const {
    mark,
    table,
    preference,
  } = state;
  await Promise.all(BYPASS_PROTOS.map(function addRule(proto,): Promise<unknown> {
    return run({
      command: 'ip',
      args: [
        proto,
        'rule',
        'add',
        'fwmark',
        String(mark,),
        'table',
        String(table,),
        'pref',
        String(preference,),
        'protocol',
        String(BYPASS_ROUTE_PROTOCOL,),
      ],
    },);
  },),);
}

/**
 Removes exact owned mark rules idempotently.
 
 @param state - Persisted rule identity.
 
 @example
 ```ts
 await removeBypassRules({ state });
 ```
 */
async function removeBypassRules(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  /**
   Primitive rule fields copied from caller state.
   */
  const {
    mark,
    table,
    preference,
  } = state;
  await Promise.all(BYPASS_PROTOS.map(function removeRule(proto,): Promise<unknown> {
    return runIpDelete({
      args: [
        proto,
        'rule',
        'delete',
        'fwmark',
        String(mark,),
        'table',
        String(table,),
        'pref',
        String(preference,),
        'protocol',
        String(BYPASS_ROUTE_PROTOCOL,),
      ],
    },);
  },),);
}

/**
 Removes watcher,
 exact rules,
 protocol-tagged defaults,
 state,
 and cooperative locks.
 
 @param state - Persisted ownership state.
 
 @example
 ```ts
 await cleanupBypassState({ state });
 ```
 */
async function cleanupBypassState(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  /**
   Watcher stop attempted before route removal to avoid resynchronization race.
   */
  const watcherResults = await Promise.allSettled([
    stopBypassWatcher({ state, },),
  ],);
  /**
   Network cleanup attempted even when sidecar validation blocks watcher signaling.
   */
  const networkResults = await Promise.allSettled([
    removeBypassRules({ state, },),
    removeOwnedBypassRoutes({ state, },),
  ],);
  /**
   All rejected cleanup operations whose ownership must remain persisted.
   */
  const failures = [
    ...watcherResults,
    ...networkResults,
  ].flatMap(function rejectionReason(
    result: Readonly<PromiseSettledResult<void>>,
  ): readonly unknown[] {
    if (result.status === 'fulfilled')
      return [];
    return [result.reason as unknown,];
  },);
  if (failures.length > 0) {
    throw new AggregateError(
      failures,
      `Application-bypass cleanup failed for ${state.interfaceName}; ownership state retained.`,
    );
  }
  await releaseBypassState({ state, },);
}

/**
 Installs collision-safe policy route for exempt-marked traffic.
 
 Physical defaults are copied into dynamically claimed table.
 Missing family gets unreachable default,
 preventing marked traffic from falling through to VPN policy.
 Supervised watcher resynchronizes after DHCP,
 router advertisement,
 and roaming changes.
 
 @param interfaceName - Tunnel interface owning bypass state.
 
 @param mark - Socket mark identifying exempt traffic.
 
 @param watchRouteChanges - Whether to start detached watcher
 in caller's privilege and network namespace.
 
 @example
 ```ts
 await addExemptRule({ interfaceName: 'wg0', mark: 100, watchRouteChanges: true });
 ```
 */
export async function addExemptRule(
  {
    interfaceName,
    mark,
    watchRouteChanges,
  }: {
    readonly interfaceName: string;
    readonly mark: number;
    readonly watchRouteChanges: boolean;
  },
): Promise<void> {
  /**
   Function-scoped logger for setup lifecycle.
   */
  const fl = tagged({
    tag: addExemptRule.name,
    l,
  },);
  /**
   Kernel lock serializing same-interface up,
   down,
   stale cleanup,
   and persistence.
   */
  await using interfaceOperation = await claimBypassInterfaceOperationInternal({ interfaceName, },);
  /**
   Physical defaults confirmed before replacing any working prior lifecycle.
   */
  const physical = await readPhysicalDefaults();
  if (physical.length === 0) {
    throw new BypassRouteError(
      `Cannot exempt applications for ${interfaceName}: no IPv4 or IPv6 physical default route exists.`,
    );
  }
  /**
   Existing state from interrupted prior lifecycle.
   */
  const existing = await readBypassState({ interfaceName, },);
  if (existing !== BYPASS_STATE_ABSENT) {
    fl.warn(`cleaning stale bypass state for ${interfaceName}`,);
    await cleanupBypassState({ state: existing, },);
  }
  /**
   Global kernel lock spans resource scan through kernel-visible installation.
   */
  await using allocationOperation = await claimBypassAllocationOperationInternal();
  /**
   Collision-safe resources covered by rollback before persistence.
   */
  const state = await claimBypassState({
    interfaceName,
    mark,
  },);
  try {
    await persistBypassState({ state, },);
    /**
     Physical defaults observed during initial synchronization.
     */
    const synchronized = await synchronizeBypassRoutesInternal({ state, },);
    if (synchronized === 0) {
      throw new BypassRouteError(
        `Physical defaults disappeared while configuring ${interfaceName}.`,
      );
    }
    await addBypassRules({ state, },);
    if (watchRouteChanges)
      await startBypassWatcher({ state, },);
    fl.debug(
      `installed mark ${String(mark,)} table ${String(state.table,)} preference ${String(state.preference,)}`,
    );
  }
  catch (error) {
    fl.error(`failed to install bypass state for ${interfaceName}: ${String(error,)}`,);
    /**
     Latest transition fingerprints ensure rollback removes every installed route form.
     */
    const persisted = await readBypassState({ interfaceName, },);
    await cleanupBypassState({
      state: persisted === BYPASS_STATE_ABSENT
        ? state
        : persisted,
    },);
    throw error;
  }
}

/**
 Removes persisted application-bypass state for interface.
 
 Teardown discovers mark,
 table,
 and preference from root-owned state rather than current config values.
 Absence is idempotent.
 
 @param interfaceName - Tunnel interface whose state is removed.
 
 @example
 ```ts
 await removeExemptRule({ interfaceName: 'wg0' });
 ```
 */
export async function removeExemptRule(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<void> {
  /**
   Kernel lock preventing concurrent replacement or duplicate teardown.
   */
  await using interfaceOperation = await claimBypassInterfaceOperationInternal({ interfaceName, },);
  /**
   Persisted state when bypass setup completed or partially completed.
   */
  const state = await readBypassState({ interfaceName, },);
  if (state === BYPASS_STATE_ABSENT) {
    l.debug(`no bypass state for ${interfaceName}`,);
    return;
  }
  await cleanupBypassState({ state, },);
  l.debug(`removed bypass state for ${interfaceName}`,);
}
