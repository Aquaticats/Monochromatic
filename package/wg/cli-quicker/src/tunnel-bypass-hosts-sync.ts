import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { isResolverTimeout, } from './errors.ts';
import { refreshSeconds, } from './host-address.ts';
import {
  filterHostAnswers,
  type HostAnswer,
  type WantedRule,
} from './host-answer-filter.ts';
import {
  ALL_LOOKUPS_TIMED_OUT_RETRY_SECONDS,
  MAX_CONCURRENT_IP_COMMANDS,
} from './host-rule-limits.data.ts';
import { mapInChunks, } from './map-in-chunks.ts';
import {
  type HostLookup,
  type HostRuleTools,
  type HostSyncReport,
  type OwnedHostRule,
  parseOwnedHostRules,
  planHostRules,
} from './tunnel-bypass-hosts.ts';
import {
  BYPASS_PROTOS,
  type BypassState,
} from './tunnel-bypass-types.ts';

/**
 Module logger for clearnet-host reconciliation.
 */
const l = tagged({ tag: 'tunnel-bypass-hosts-sync', },);

/**
 Lists the rules this lifecycle owns in both families.

 @param state - Claimed table and preference.

 @param tools - Rule side effects.

 @returns Owned rules.

 @example
 ```ts
 await listOwnedRules({ state, tools: IP_HOST_RULE_TOOLS });
 ```
 */
async function listOwnedRules(
  {
    state,
    tools,
  }: {
    readonly state: BypassState;
    readonly tools: HostRuleTools;
  },
): Promise<readonly OwnedHostRule[]> {
  return (await Promise.all(BYPASS_PROTOS.map(async function listFamily(family,): Promise<readonly OwnedHostRule[]> {
    return parseOwnedHostRules({
      text: await tools.list({ family, },),
      family,
      state,
    },);
  },),)).flat();
}

/**
 Asks the resolver about every host,
 turning a throwing resolver into a failure for that host.

 @param hosts - The host lists whose rules this rule set carries.

 @param lookup - Resolver.

 @returns One answer per host.

 @example
 ```ts
 await lookupAll({ hosts: ['crates.io'], lookup });
 ```
 */
function lookupAll(
  {
    hosts,
    lookup,
  }: {
    readonly hosts: readonly string[];
    readonly lookup: HostLookup;
  },
): Promise<readonly HostAnswer[]> {
  return Promise.all(hosts.map(async function lookupOne(host,): Promise<HostAnswer> {
    try {
      return {
        host,
        result: await lookup({ host, },),
      };
    }
    catch (error) {
      return {
        host,
        result: {
          addresses: [],
          failure: `lookup of ${host} threw: ${String(error,)}`,
          ...(isResolverTimeout({ error, },) ? { timedOut: true, } : {}),
        },
      };
    }
  },),);
}

/**
 Applies a plan with a bounded number of `ip` commands at once,
 saying at info level which rule was added or removed and for whom.

 @param state - Claimed table and preference.

 @param tools - Rule side effects.

 @param add - Rules to add.

 @param remove - Rules to remove.

 @param hostsOf - Hosts behind each wanted address,
 for the log.

 @returns Counts of operations that succeeded,
 and the failures of those that did not.

 @example
 ```ts
 await applyPlan({ state, tools, add: [], remove: [], hostsOf: new Map() });
 ```
 */
async function applyPlan(
  {
    state,
    tools,
    add,
    remove,
    hostsOf,
  }: {
    readonly state: BypassState;
    readonly tools: HostRuleTools;
    readonly add: readonly OwnedHostRule[];
    readonly remove: readonly OwnedHostRule[];
    readonly hostsOf: ReadonlyMap<string, readonly string[]>;
  },
): Promise<{
  readonly added: number;
  readonly removed: number;
  readonly failures: readonly string[]
}> {
  /**
   Function-scoped logger for one plan.
   */
  const fl = tagged({
    tag: applyPlan.name,
    l,
  },);
  /**
   Failures collected without stopping.
   */
  const failures: string[] = [];
  /**
   Counts of operations that succeeded.
   */
  const done = {
    added: 0,
    removed: 0,
  };
  await mapInChunks({
    items: add,
    size: MAX_CONCURRENT_IP_COMMANDS,
    task: async function addOne(rule,): Promise<void> {
      try {
        await tools.add({
          rule,
          state,
        },);
        done.added += 1;
        fl.info(`clearnet-host rule added: ${rule.family} ${rule.address} for ${(hostsOf.get(`${rule.family}|${rule.address}`,) ?? []).join(', ',)}`,);
      }
      catch (error) {
        failures.push(`cannot add rule for ${rule.address}: ${String(error,)}`,);
      }
    },
  },);
  await mapInChunks({
    items: remove,
    size: MAX_CONCURRENT_IP_COMMANDS,
    task: async function removeOne(rule,): Promise<void> {
      try {
        await tools.remove({
          rule,
          state,
        },);
        done.removed += 1;
        fl.info(`clearnet-host rule removed: ${rule.family} ${rule.address}`,);
      }
      catch (error) {
        failures.push(`cannot remove rule for ${rule.address}: ${String(error,)}`,);
      }
    },
  },);
  return {
    ...done,
    failures,
  };
}

/**
 Smallest TTL among the wanted rules,
 or zero when there are none.

 @param wanted - Rules wanted now.

 @returns Seconds.

 @example
 ```ts
 smallestTtl({ wanted: [] }); // 0
 ```
 */
function smallestTtl({ wanted, }: { readonly wanted: readonly WantedRule[]; },): number {
  if (wanted.length === 0)
    return 0;
  return Math.min(...wanted.map(function ttlOf(item,): number {
    return item.ttlSeconds;
  },),);
}

/**
 Tells whether the lookup of a host ended because the resolver did not answer in time.

 @param answer - The answer for one host.

 @returns Whether it timed out.

 @example
 ```ts
 didTimeOut({ host: 'crates.io', result: { addresses: [], timedOut: true } }); // true
 ```
 */
function didTimeOut(answer: HostAnswer,): boolean {
  return answer.result
    .timedOut
    === true;
}

/**
 Reconciles the update-host rules with current DNS answers.

 Nothing here throws or invents a fallback:
 every failure is returned in the report so the caller can say which policy is not in force,
 and an address that failed to resolve keeps its existing rules until they expire through the absent-refresh rule.

 @param state - Claimed table and preference.

 @param hosts - The host lists whose rules this rule set carries.

 @param lookup - Resolver.

 @param tools - Rule side effects.

 @param absentCounts - Per-rule absence counters kept between refreshes.

 @param protectedKeys - Keys of addresses the tunnel depends on,
 which never get a rule.

 @param preflightFailures - Problems found before any lookup,
 reported with the rest.

 @returns Counts,
 failures,
 and the delay before the next refresh.

 @example
 ```ts
 await syncUpdateHostRules({ state, hosts, lookup, tools: IP_HOST_RULE_TOOLS, absentCounts: new Map() });
 ```
 */
export async function syncUpdateHostRules(
  {
    state,
    hosts,
    lookup,
    tools,
    absentCounts,
    protectedKeys = new Set(),
    preflightFailures = [],
  }: {
    readonly state: BypassState;
    readonly hosts: readonly string[];
    readonly lookup: HostLookup;
    readonly tools: HostRuleTools;
    readonly absentCounts: Map<string, number>;
    readonly protectedKeys?: ReadonlySet<string>;
    readonly preflightFailures?: readonly string[];
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
   One answer per host.
   */
  const answers = await lookupAll({
    hosts,
    lookup,
  },);
  /**
   Rules wanted now and the problems met while deriving them.
   */
  const filtered = filterHostAnswers({
    answers,
    protectedKeys,
  },);
  /**
   Whether not one lookup was answered in time,
   which is a resolver that is not answering yet and not a problem with a host.
   */
  const everyLookupTimedOut = (answers.length > 0) && answers.every(didTimeOut,);
  /**
   Rules installed now.
   */
  const owned = await listOwnedRules({
    state,
    tools,
  },);
  /**
   Rules to add and remove.
   */
  const plan = planHostRules({
    desired: filtered.wanted
      .map(function ruleOf(item,): OwnedHostRule {
        return item.rule;
      },),
    owned,
    absentCounts,
  },);
  /**
   Hosts behind each wanted address.
   */
  const hostsOf = new Map(filtered.wanted
    .map(function entry(item,): readonly [
      string,
      readonly string[]
    ] {
      return [
        `${item.rule
          .family}|${item.rule
            .address}`,
        item.hosts,
      ];
    },),);
  /**
   Outcome of the operations.
   */
  const applied = await applyPlan({
    state,
    tools,
    add: plan.add,
    remove: plan.remove,
    hostsOf,
  },);
  /**
   Everything that left the policy unenforced.
   */
  const failures = [
    ...preflightFailures,
    ...filtered.failures,
    ...applied.failures,
  ];
  fl.debug(`update-host rules: ${String(applied.added,)} added, ${String(applied.removed,)} removed, ${String(failures.length,)} failures`,);
  return {
    added: applied.added,
    removed: applied.removed,
    failures,
    nextRefreshSeconds: everyLookupTimedOut
      ? ALL_LOOKUPS_TIMED_OUT_RETRY_SECONDS
      : refreshSeconds({ ttlSeconds: smallestTtl({ wanted: filtered.wanted, },), },),
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
  const owned = await listOwnedRules({
    state,
    tools,
  },);
  await mapInChunks({
    items: owned,
    size: MAX_CONCURRENT_IP_COMMANDS,
    task: function removeOne(rule,): Promise<void> {
      return tools.remove({
        rule,
        state,
      },);
    },
  },);
}
