import { setTimeout as sleep, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type HostLookup,
  type HostRuleTools,
  type HostSyncReport,
  IP_HOST_RULE_TOOLS,
} from './tunnel-bypass-hosts.ts';
import { syncUpdateHostRules, } from './tunnel-bypass-hosts-sync.ts';
import type { BypassState, } from './tunnel-bypass-types.ts';
import {
  protectedKeys,
  readTunnelDependencies,
  type TunnelDependencyReader,
} from './tunnel-dependencies.ts';
import { createTunnelHostLookup, } from './tunnel-dns-lookup.ts';
import type { FailureDelivery, } from './runtime-failure-report.ts';
import { PROVIDER_HOSTS, } from './provider-hosts.data.ts';
import { UPDATE_HOSTS, } from './update-hosts.data.ts';
import {
  KIND_UPDATE_HOST_PROBLEMS,
  KIND_UPDATE_HOST_REFRESH_FAILED,
  POLICY_CLEARNET_HOSTS,
  policyNotInForce,
} from './update-policy-notice.ts';

/**
 Module logger for the clearnet-host refresh.
 */
const l = tagged({ tag: 'bypass-host-refresh', },);

/**
 Every host whose traffic the wg tools keep outside the tunnel for every program:
 the update hosts and the model provider hosts Pi reaches.
 They share one rule set,
 because a refresh that installed only its own list would remove the other list's
 rules as unwanted,
 and each list keeps its own reason in the start notice.
 */
export const CLEARNET_HOSTS: readonly string[] = [
  ...UPDATE_HOSTS,
  ...PROVIDER_HOSTS,
];

/**
 Milliseconds in one second.
 */
const MS_PER_SECOND = 1_000;

/**
 Seconds before retrying after a refresh that failed outright
 or that found the tunnel without a DNS server.
 */
export const FAILED_REFRESH_RETRY_SECONDS = 30;

/**
 Reports the outcome of one refresh.

 A refresh with any failure says loudly which policy is not fully in force and what is left unenforced.

 @param report - Outcome of the reconciliation.

 @param deliver - Delivery of a loud report to a person,
 absent while `up` runs, where the log is the terminal.

 @example
 ```ts
 await reportHostSync({ report });
 ```
 */
export async function reportHostSync(
  {
    report,
    deliver,
  }: {
    readonly report: HostSyncReport;
    readonly deliver?: FailureDelivery;
  },
): Promise<void> {
  /**
   Function-scoped logger for one report.
   */
  const fl = tagged({
    tag: reportHostSync.name,
    l,
  },);
  /**
   Loud report text when something is not in force.
   */
  const loud = hostSyncFailureText({ report, },);
  if (loud === '') {
    fl.debug(`clearnet-host rules current: ${String(report.added,)} added, ${String(report.removed,)} removed`,);
    return;
  }
  fl.error(loud,);
  await deliver?.({
    kind: KIND_UPDATE_HOST_PROBLEMS,
    line: loud,
  },);
}

/**
 Most problems named in the text of one report.
 */
export const MAX_LISTED_FAILURES = 10;

/**
 Builds the loud report for a refresh with problems.

 @param report - Outcome of the reconciliation.

 @returns Report naming the policy,
 or an empty string when the refresh was clean.

 @example
 ```ts
 hostSyncFailureText({ report });
 ```
 */
export function hostSyncFailureText({ report, }: { readonly report: HostSyncReport; },): string {
  if (report.failures
    .length
    === 0)
    return '';
  /**
   Problems named in the report.
   The rest are counted,
   because an answer with thousands of refused addresses must not make a line of thousands of problems.
   */
  const listed = report.failures
    .slice(
    0,
    MAX_LISTED_FAILURES,
    );
  /**
   Problems left out of the list.
   */
  const unlisted = report.failures
    .length
    - listed.length;
  return policyNotInForce({
    policy: POLICY_CLEARNET_HOSTS,
    detail: `${String(report.failures
      .length,)} problem(s); traffic to the affected hosts uses the tunnel until the next refresh `
      + `in ${String(report.nextRefreshSeconds,)} s: ${listed
        .join('; ',)}${unlisted > 0 ? `; and ${String(unlisted,)} more` : ''}`,
  },);
}

/**
 Words for a tunnel link that has no DNS server.

 @param interfaceName - Tunnel interface name.

 @returns One failure sentence for the report.

 @example
 ```ts
 noDnsFailure({ interfaceName: 'wg0' });
 ```
 */
export function noDnsFailure({ interfaceName, }: { readonly interfaceName: string; },): string {
  return `the tunnel ${interfaceName} sets no DNS server, so no host rule is installed: `
    + 'host rules use only answers from the tunnel\'s own DNS = servers, queried through the tunnel, '
    + 'and the process watch, the toolchain paths and the Pi coverage still run';
}

/**
 Runs one reconciliation with the tunnel's own resolver and `ip`,
 reporting its outcome.

 The refresh first reads the DNS servers and peer endpoints of the tunnel.
 A tunnel with no DNS server gets no host rules,
 and a refresh that cannot read them changes nothing,
 and both say so loudly.

 @param state - Claimed table and preference.

 @param absentCounts - Per-rule absence counters.

 @param lookup - Resolver,
 replaceable in tests.

 @param tools - Rule side effects,
 replaceable in tests.

 @param readDependencies - Reader of the tunnel's DNS servers and peer endpoints,
 replaceable in tests.

 @param deliver - Delivery of a loud report to a person.

 @returns Seconds until the next refresh.

 @example
 ```ts
 await refreshUpdateHostRules({ state, absentCounts: new Map() });
 ```
 */
export async function refreshUpdateHostRules(
  {
    state,
    absentCounts,
    lookup = createTunnelHostLookup({ interfaceName: state.interfaceName, },),
    tools = IP_HOST_RULE_TOOLS,
    readDependencies = readTunnelDependencies,
    deliver,
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly lookup?: HostLookup;
    readonly tools?: HostRuleTools;
    readonly readDependencies?: TunnelDependencyReader;
    readonly deliver?: FailureDelivery;
  },
): Promise<number> {
  /**
   Function-scoped logger for one refresh.
   */
  const fl = tagged({
    tag: refreshUpdateHostRules.name,
    l,
  },);
  /**
   DNS servers and peer endpoints of the tunnel.
   */
  const dependencies = await readDependencies({ interfaceName: state.interfaceName, },);
  /**
   Whether any DNS server is set on the tunnel link.
   */
  const hasDns = dependencies.dnsServers
    .length
    > 0;
  /**
   Reconciliation outcome;
   without a DNS server the host list is empty and only the loud report remains.
   */
  const report = await syncUpdateHostRules({
    state,
    hosts: hasDns ? CLEARNET_HOSTS : [],
    lookup,
    tools,
    absentCounts,
    protectedKeys: protectedKeys({ dependencies, },),
    preflightFailures: hasDns ? [] : [noDnsFailure({ interfaceName: state.interfaceName, },),],
  },);
  await reportHostSync({
    report,
    ...(deliver === undefined ? {} : { deliver, }),
  },);
  fl.debug(`next refresh in ${String(report.nextRefreshSeconds,)} s`,);
  return hasDns ? report.nextRefreshSeconds : FAILED_REFRESH_RETRY_SECONDS;
}

/**
 Refreshes once and turns an unexpected error into the loud policy report,
 so the loop keeps trying instead of ending the watcher.

 @param state - Claimed table and preference.

 @param absentCounts - Per-rule absence counters.

 @param lookup - Resolver,
 replaceable in tests.

 @param tools - Rule side effects,
 replaceable in tests.

 @param readDependencies - Reader of the tunnel's dependencies,
 replaceable in tests.

 @param deliver - Delivery of a loud report to a person.

 @returns Seconds until the next refresh.

 @example
 ```ts
 await refreshOrReport({ state, absentCounts: new Map() });
 ```
 */
export async function refreshOrReport(
  {
    state,
    absentCounts,
    lookup,
    tools,
    readDependencies,
    deliver,
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly lookup?: HostLookup;
    readonly tools?: HostRuleTools;
    readonly readDependencies?: TunnelDependencyReader;
    readonly deliver?: FailureDelivery;
  },
): Promise<number> {
  /**
   Function-scoped logger for one guarded refresh.
   */
  const fl = tagged({
    tag: refreshOrReport.name,
    l,
  },);
  try {
    return await refreshUpdateHostRules({
      state,
      absentCounts,
      ...(lookup === undefined ? {} : { lookup, }),
      ...(tools === undefined ? {} : { tools, }),
      ...(readDependencies === undefined ? {} : { readDependencies, }),
      ...(deliver === undefined ? {} : { deliver, }),
    },);
  }
  catch (error) {
    /**
     Loud report of the failed refresh.
     */
    const loud = policyNotInForce({
      policy: POLICY_CLEARNET_HOSTS,
      detail: `the refresh failed: ${String(error,)}; no host rule was added or changed by this attempt`,
    },);
    fl.error(loud,);
    await deliver?.({
      kind: KIND_UPDATE_HOST_REFRESH_FAILED,
      line: loud,
    },);
    return FAILED_REFRESH_RETRY_SECONDS;
  }
}

/**
 Waits the given seconds,
 ending early without an error when the signal aborts.

 @param seconds - Seconds to wait.

 @param signal - Aborted when the watcher is asked to stop.

 @example
 ```ts
 await sleepUnlessAborted({ seconds: 30, signal });
 ```
 */
export async function sleepUnlessAborted(
  {
    seconds,
    signal,
  }: {
    readonly seconds: number;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  /**
   Function-scoped logger for one wait.
   */
  const fl = tagged({
    tag: sleepUnlessAborted.name,
    l,
  },);
  try {
    await sleep(
      seconds * MS_PER_SECOND,
      undefined,
      { signal, },
    );
  }
  catch (error) {
    if (!signal.aborted)
      throw error;
    fl.debug('wait ended early: the watcher was asked to stop',);
  }
}

/**
 The guarded refresh the loop runs,
 replaceable in tests.
 */
export type GuardedRefresh = typeof refreshOrReport;

/**
 One round of the refresh loop:
 a guarded refresh, then the wait until the next one.

 @param state - Claimed table and preference.

 @param absentCounts - Per-rule absence counters.

 @param signal - Aborted when the watcher is asked to stop.

 @param deliver - Delivery of a loud report to a person.

 @param refresh - The guarded refresh,
 replaceable in tests.

 @example
 ```ts
 await refreshThenWait({ state, absentCounts: new Map(), signal });
 ```
 */
export async function refreshThenWait(
  {
    state,
    absentCounts,
    signal,
    deliver,
    refresh = refreshOrReport,
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly signal: AbortSignal;
    readonly deliver?: FailureDelivery;
    readonly refresh?: GuardedRefresh;
  },
): Promise<void> {
  await sleepUnlessAborted({
    seconds: await refresh({
      state,
      absentCounts,
      ...(deliver === undefined ? {} : { deliver, }),
    },),
    signal,
  },);
}

/**
 Refreshes the clearnet-host rules, of the update hosts and the model provider hosts, until the watcher is asked to stop.

 @param state - Claimed table and preference.

 @param signal - Aborted when the watcher is asked to stop.

 @param deliver - Delivery of a loud report to a person,
 which the detached watcher needs because its own output is discarded.

 @param refresh - The guarded refresh,
 replaceable in tests.

 @example
 ```ts
 await runHostRefreshLoop({ state, signal: controller.signal });
 ```
 */
export async function runHostRefreshLoop(
  {
    state,
    signal,
    deliver,
    refresh,
  }: {
    readonly state: BypassState;
    readonly signal: AbortSignal;
    readonly deliver?: FailureDelivery;
    readonly refresh?: GuardedRefresh;
  },
): Promise<void> {
  /**
   Absence counters kept between refreshes.
   */
  const absentCounts = new Map<string, number>();
  while (!signal.aborted) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- See doc/lint/no-await-in-loop-host-refresh.md: each refresh reads the rules the previous one left.
    await refreshThenWait({
      state,
      absentCounts,
      signal,
      ...(deliver === undefined ? {} : { deliver, }),
      ...(refresh === undefined ? {} : { refresh, }),
    },);
  }
}
