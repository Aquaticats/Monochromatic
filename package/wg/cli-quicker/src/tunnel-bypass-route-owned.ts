import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { BypassRouteError, } from './errors.ts';
import { runIpDelete, } from './ip-delete.ts';
import { run, } from './runner.ts';
import {
  assertRecordedDefaults,
  currentOwnedState,
  exactRouteKey,
  hasBypassProtocol,
  ownedRouteTokens,
  readAllDefaultRoutes,
  readOwnedRoutes,
  routeKey,
  stateWithRoutes,
  unionRoutes,
} from './tunnel-bypass-route-owned-helper.ts';
import {
  desiredFamilyRoutes,
  readPhysicalDefaults,
  type FamilyRoute,
} from './tunnel-bypass-route-physical.ts';
import { persistBypassState, } from './tunnel-bypass-state.ts';
import {
  BYPASS_ROUTE_PROTOCOL,
  type BypassState,
} from './tunnel-bypass-types.ts';

/**
 Module logger for bypass route ownership synchronization.
 */
const l = tagged({ tag: 'tunnel-bypass-route-owned', },);

/**
 Synchronizes claimed bypass table to current physical defaults.
 
 Transition state persists previous and intended fingerprints before mutation.
 This makes interrupted setup and watcher retries exactly cleanable.
 Missing family receives unreachable default so marked traffic cannot fall through to VPN policy.
 
 @param state - Persisted table ownership identity.
 
 @returns Number of physical defaults observed.
 
 @example
 ```ts
 await synchronizeBypassRoutes({ state });
 ```
 */
export async function synchronizeBypassRoutes(
  { state, }: { readonly state: BypassState; },
): Promise<number> {
  /**
   Function-scoped logger for one synchronization.
   */
  const fl = tagged({
    tag: synchronizeBypassRoutes.name,
    l,
  },);
  /**
   Latest fingerprints from previous complete or interrupted synchronization.
   */
  const current = await currentOwnedState({ requested: state, },);
  await assertRecordedDefaults({ state: current, },);
  /**
   Current physical defaults before owned table mutation.
   */
  const physical = await readPhysicalDefaults();
  /**
   Desired routes for both families.
   */
  const desired = desiredFamilyRoutes({ physical, },);
  /**
   Intended command forms included in transition cleanup coverage.
   */
  const intended = desired.map(function intendedOwnedRoute(route,): FamilyRoute {
    return {
      proto: route.proto,
      tokens: ownedRouteTokens({
        route,
        table: current.table,
      },),
    };
  },);
  /**
   Transition ownership covering old and new forms before first mutation.
   */
  const transition = stateWithRoutes({
    state: current,
    routes: unionRoutes({
      first: current.routes,
      second: intended,
    },),
  },);
  await persistBypassState({ state: transition, },);
  for (const route of desired) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Route replacement order preserves connectivity before stale deletion.
    await run({
      command: 'ip',
      args: [
        route.proto,
        'route',
        'replace',
        ...ownedRouteTokens({
          route,
          table: current.table,
        },),
      ],
    },);
  }
  /**
   Desired normalized identities used to identify stale prior routes.
   */
  const desiredKeys = new Set(desired.map(function desiredIdentity(route,): string {
    /**
     Fresh token copy before joining through default-library boundary.
     */
    const tokens = [...route.tokens,];
    return routeKey({
      proto: route.proto,
      text: tokens.join(' ',),
    },);
  },),);
  for (const route of current.routes) {
    /**
     Fresh token copy before joining through default-library boundary.
     */
    const tokens = [...route.tokens,];
    if (desiredKeys.has(routeKey({
      proto: route.proto,
      text: tokens.join(' ',),
    },)))
      continue;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Exact stale route deletions are sequenced after replacement.
    await runIpDelete({
      args: [
        route.proto,
        'route',
        'delete',
        ...route.tokens,
      ],
    },);
  }
  /**
   Kernel-rendered owned routes become final exact fingerprints.
   */
  const installed = await readOwnedRoutes({ table: current.table, },);
  /**
   All defaults checked against protocol-filtered owned rendering after mutation.
   */
  const allDefaults = await readAllDefaultRoutes({ table: current.table, },);
  /**
   Exact canonical routes carrying owner protocol after synchronization.
   */
  const installedKeys = new Set(installed.map(function installedIdentity(route,): string {
    /**
     Fresh token copy before semantic identity rendering.
     */
    const tokens = [...route.tokens,];
    return routeKey({
      proto: route.proto,
      text: tokens.join(' ',),
    },);
  },),);
  if (allDefaults.some(function unownedDefault(route,): boolean {
    if (!hasBypassProtocol({ route, }))
      return true;
    /**
     Fresh token copy before semantic identity rendering.
     */
    const tokens = [...route.tokens,];
    return !installedKeys.has(routeKey({
      proto: route.proto,
      text: tokens.join(' ',),
    },),);
  },)) {
    throw new BypassRouteError(
      `Bypass table ${String(current.table,)} gained unowned default during synchronization.`,
    );
  }
  if (installed.length
    > transition.routes
      .length) {
    throw new BypassRouteError(
      `Bypass table ${String(current.table,)} gained unrecorded protocol ${String(BYPASS_ROUTE_PROTOCOL,)} route.`,
    );
  }
  await persistBypassState({
    state: stateWithRoutes({
      state: current,
      routes: installed,
    },),
  },);
  fl.debug(`synchronized ${String(physical.length,)} physical default route(s)`,);
  return physical.length;
}

/**
 Deletes only exact route fingerprints persisted by owner.
 
 Unrelated routes and table contents are never flushed,
 including routes that independently use protocol `201`.
 
 @param state - Persisted table ownership.
 
 @example
 ```ts
 await removeOwnedBypassRoutes({ state });
 ```
 */
export async function removeOwnedBypassRoutes(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  /**
   Pending exact route deletions.
   */
  const removals: Promise<void>[] = [];
  for (const route of state.routes) {
    removals.push(runIpDelete({
      args: [
        route.proto,
        'route',
        'delete',
        ...route.tokens,
      ],
    },),);
  }
  await Promise.all(removals,);
}
