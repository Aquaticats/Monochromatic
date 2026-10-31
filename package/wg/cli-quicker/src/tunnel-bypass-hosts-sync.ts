import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  isPublicAddress,
  normalizeAddress,
  refreshSeconds,
} from './host-address.ts';
import {
  type HostAddress,
  type HostLookup,
  type HostLookupResult,
  type HostRuleTools,
  type HostSyncReport,
  type OwnedHostRule,
  parseOwnedHostRules,
  planHostRules,
  ruleKey,
} from './tunnel-bypass-hosts.ts';
import {
  BYPASS_PROTOS,
  type BypassState,
} from './tunnel-bypass-types.ts';

/**
 Module logger for update-host reconciliation.
 */
const l = tagged({ tag: 'tunnel-bypass-hosts-sync', },);

/**
 Reconciles the update-host rules with current DNS answers.

 Nothing here throws or invents a fallback:
 every failure is returned in the report so the caller can say which policy is not in force,
 and an address that failed to resolve keeps its existing rules until they expire through the absent-refresh rule.

 @param state - Claimed table and preference.

 @param hosts - The update-host list.

 @param lookup - Resolver.

 @param tools - Rule side effects.

 @param absentCounts - Per-rule absence counters kept between refreshes.

 @returns Counts,
 failures,
 and the delay before the next refresh.

 @example
 ```ts
 await syncUpdateHostRules({ state, hosts, lookup: lookupHost, tools: IP_HOST_RULE_TOOLS, absentCounts: new Map() });
 ```
 */
export async function syncUpdateHostRules(
  {
    state,
    hosts,
    lookup,
    tools,
    absentCounts,
  }: {
    readonly state: BypassState;
    readonly hosts: readonly string[];
    readonly lookup: HostLookup;
    readonly tools: HostRuleTools;
    readonly absentCounts: Map<string, number>;
  },
): Promise<HostSyncReport> {
  /**
   Function-scoped logger for one reconciliation.
   */
  const fl = tagged({
    tag: syncUpdateHostRules.name,
    l,
  },);
  /**
   Answers per host.
   */
  const answers = await Promise.all(hosts.map(async function lookupOne(host,): Promise<HostLookupResult> {
    try {
      return await lookup({ host, },);
    }
    catch (error) {
      return {
        addresses: [],
        failure: `lookup of ${host} threw: ${String(error,)}`,
      };
    }
  },),);
  /**
   Failures collected without stopping.
   */
  const failures: string[] = answers.flatMap(function failureOf(answer,): readonly string[] {
    return answer.failure === undefined ? [] : [answer.failure,];
  },);
  /**
   Answers that are public unicast addresses.
   */
  const usable = answers.flatMap(function addressesOf(answer,): readonly HostAddress[] {
    return answer.addresses;
  },)
    .flatMap(function normalized(item,): readonly (HostAddress & { readonly rule: OwnedHostRule; })[] {
    /**
     Normalized address text.
     */
      const address = normalizeAddress({
        family: item.family,
        text: item.address,
      },);
      if (address === '') {
        failures.push(`ignored malformed ${item.family} answer ${item.address}`,);
        return [];
      }
      if (!isPublicAddress({
        family: item.family,
        address,
      },)) {
        failures.push(`refused non-public answer ${address}`,);
        return [];
      }
      return [{
        ...item,
        address,
        rule: {
          family: item.family,
          address,
        },
      },];
    },);
  /**
   Rules wanted now,
   without duplicates.
   */
  const desired = [
    ...new Map(usable.map(function entry(item,): readonly [
      string,
      OwnedHostRule
    ] {
      return [
        ruleKey({ rule: item.rule, },),
        item.rule,
      ];
    },),)
      .values(),
  ];
  /**
   Rules installed now.
   */
  const owned = (await Promise.all(BYPASS_PROTOS.map(async function listFamily(family,): Promise<readonly OwnedHostRule[]> {
    return parseOwnedHostRules({
      text: await tools.list({ family, },),
      family,
      state,
    },);
  },),)).flat();
  /**
   Rules to add and remove.
   */
  const plan = planHostRules({
    desired,
    owned,
    absentCounts,
  },);
  /**
   Counts of operations that succeeded.
   */
  const done = {
    added: 0,
    removed: 0,
  };
  await Promise.all([
    ...plan.add
      .map(async function addOne(rule,): Promise<void> {
        try {
          await tools.add({
            rule,
            state,
          },);
          done.added += 1;
        }
        catch (error) {
          failures.push(`cannot add rule for ${rule.address}: ${String(error,)}`,);
        }
      },),
    ...plan.remove
      .map(async function removeOne(rule,): Promise<void> {
        try {
          await tools.remove({
            rule,
            state,
          },);
          done.removed += 1;
        }
        catch (error) {
          failures.push(`cannot remove rule for ${rule.address}: ${String(error,)}`,);
        }
      },),
  ],);
  fl.debug(`update-host rules: ${String(done.added,)} added, ${String(done.removed,)} removed, ${String(failures.length,)} failures`,);
  return {
    added: done.added,
    removed: done.removed,
    failures,
    nextRefreshSeconds: refreshSeconds({
      ttlSeconds: usable.length === 0
        ? 0
        : Math.min(...usable.map(function ttlOf(item,): number {
          return item.ttlSeconds;
        },),),
    }),
  };
}

/**
 Removes every update-host rule the lifecycle owns.

 @param state - Claimed table and preference.

 @param tools - Rule side effects.

 @example
 ```ts
 await removeUpdateHostRules({ state, tools: IP_HOST_RULE_TOOLS });
 ```
 */
export async function removeUpdateHostRules(
  {
    state,
    tools,
  }: {
    readonly state: BypassState;
    readonly tools: HostRuleTools;
  },
): Promise<void> {
  /**
   Rules installed now.
   */
  const owned = (await Promise.all(BYPASS_PROTOS.map(async function listFamily(family,): Promise<readonly OwnedHostRule[]> {
    return parseOwnedHostRules({
      text: await tools.list({ family, },),
      family,
      state,
    },);
  },),)).flat();
  await Promise.all(owned.map(function removeOne(rule,): Promise<void> {
    return tools.remove({
      rule,
      state,
    },);
  },),);
}
