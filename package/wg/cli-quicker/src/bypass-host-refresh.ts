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
import { UPDATE_HOSTS, } from './update-hosts.data.ts';
import {
  POLICY_UPDATE_HOSTS,
  policyNotInForce,
} from './update-policy-notice.ts';

/**
 Module logger for the update-host refresh.
 */
const l = tagged({ tag: 'bypass-host-refresh', },);

/**
 Milliseconds in one second.
 */
const MS_PER_SECOND = 1_000;

/**
 Seconds before retrying after a refresh that failed outright
 or that found the tunnel without a DNS server.
 */
const FAILED_REFRESH_RETRY_SECONDS = 30;

/**
 Reports the outcome of one refresh.

 A refresh with any failure says loudly which policy is not fully in force and what is left unenforced.

 @param report - Outcome of the reconciliation.

 @example
 ```ts
 reportHostSync({ report });
 ```
 */
export function reportHostSync({ report, }: { readonly report: HostSyncReport; },): void {
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
    fl.debug(`update-host rules current: ${String(report.added,)} added, ${String(report.removed,)} removed`,);
    return;
  }
  fl.error(loud,);
}

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
  return policyNotInForce({
    policy: POLICY_UPDATE_HOSTS,
    detail: `${String(report.failures
      .length,)} problem(s); traffic to the affected hosts uses the tunnel until the next refresh `
      + `in ${String(report.nextRefreshSeconds,)} s: ${report.failures
        .join('; ',)}`,
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
  return `the tunnel ${interfaceName} sets no DNS server, so no update-host rule is installed: `
    + 'host rules use only answers from the tunnel\'s own DNS = servers, queried through the tunnel, '
    + 'and the process watch and the toolchain paths still run';
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
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly lookup?: HostLookup;
    readonly tools?: HostRuleTools;
    readonly readDependencies?: TunnelDependencyReader;
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
    hosts: hasDns ? UPDATE_HOSTS : [],
    lookup,
    tools,
    absentCounts,
    protectedKeys: protectedKeys({ dependencies, },),
    preflightFailures: hasDns ? [] : [noDnsFailure({ interfaceName: state.interfaceName, },),],
  },);
  reportHostSync({ report, },);
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
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly lookup?: HostLookup;
    readonly tools?: HostRuleTools;
    readonly readDependencies?: TunnelDependencyReader;
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
    },);
  }
  catch (error) {
    fl.error(
      policyNotInForce({
        policy: POLICY_UPDATE_HOSTS,
        detail: `the refresh failed: ${String(error,)}; no update-host rule was added or changed by this attempt`,
      },),
    );
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
 One round of the refresh loop:
 a guarded refresh, then the wait until the next one.

 @param state - Claimed table and preference.

 @param absentCounts - Per-rule absence counters.

 @param signal - Aborted when the watcher is asked to stop.

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
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  await sleepUnlessAborted({
    seconds: await refreshOrReport({
      state,
      absentCounts,
    },),
    signal,
  },);
}

/**
 Refreshes the update-host rules until the watcher is asked to stop.

 @param state - Claimed table and preference.

 @param signal - Aborted when the watcher is asked to stop.

 @example
 ```ts
 await runHostRefreshLoop({ state, signal: controller.signal });
 ```
 */
export async function runHostRefreshLoop(
  {
    state,
    signal,
  }: {
    readonly state: BypassState;
    readonly signal: AbortSignal;
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
    },);
  }
}
