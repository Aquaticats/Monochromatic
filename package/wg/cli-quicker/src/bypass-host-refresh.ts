import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type HostLookup,
  type HostRuleTools,
  type HostSyncReport,
  IP_HOST_RULE_TOOLS,
  lookupHost,
} from './tunnel-bypass-hosts.ts';
import { syncUpdateHostRules, } from './tunnel-bypass-hosts-sync.ts';
import type { BypassState, } from './tunnel-bypass-types.ts';
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
 Granularity at which a pending sleep notices a stop request,
 in milliseconds.
 */
const STOP_POLL_MS = 1_000;

/**
 Milliseconds in one second.
 */
const MS_PER_SECOND = 1_000;

/**
 Seconds before retrying after a refresh that failed outright.
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
   Loud report text when something is not in force.
   */
  const loud = hostSyncFailureText({ report, },);
  if (loud === '') {
    l.debug(`update-host rules current: ${String(report.added,)} added, ${String(report.removed,)} removed`,);
    return;
  }
  l.error(loud,);
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
 Runs one reconciliation with the production resolver and `ip`,
 reporting its outcome.

 @param state - Claimed table and preference.

 @param absentCounts - Per-rule absence counters.

 @param lookup - Resolver,
 replaceable in tests.

 @param tools - Rule side effects,
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
    lookup = lookupHost,
    tools = IP_HOST_RULE_TOOLS,
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly lookup?: HostLookup;
    readonly tools?: HostRuleTools;
  },
): Promise<number> {
  /**
   Reconciliation outcome.
   */
  const report = await syncUpdateHostRules({
    state,
    hosts: UPDATE_HOSTS,
    lookup,
    tools,
    absentCounts,
  },);
  reportHostSync({ report, },);
  return report.nextRefreshSeconds;
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
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly lookup?: HostLookup;
    readonly tools?: HostRuleTools;
  },
): Promise<number> {
  try {
    return await refreshUpdateHostRules({
      state,
      absentCounts,
      ...(lookup === undefined ? {} : { lookup, }),
      ...(tools === undefined ? {} : { tools, }),
    },);
  }
  catch (error) {
    l.error(
      policyNotInForce({
        policy: POLICY_UPDATE_HOSTS,
        detail: `the refresh failed: ${String(error,)}; no rule was changed by this attempt`,
      },),
    );
    return FAILED_REFRESH_RETRY_SECONDS;
  }
}

/**
 Refreshes the update-host rules until the watcher is asked to stop.

 @param state - Claimed table and preference.

 @param isStopped - Whether the watcher has been asked to stop.

 @example
 ```ts
 await runHostRefreshLoop({ state, isStopped: () => lifecycle.requested });
 ```
 */
export async function runHostRefreshLoop(
  {
    state,
    isStopped,
  }: {
    readonly state: BypassState;
    readonly isStopped: () => boolean;
  },
): Promise<void> {
  /**
   Absence counters kept between refreshes.
   */
  const absentCounts = new Map<string, number>();
  while (!isStopped()) {
    /**
     Seconds to wait before the next refresh.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- Each refresh depends on the rules the previous one left.
    const seconds = await refreshOrReport({
      state,
      absentCounts,
    },);
    for (let waited = 0; (waited < (seconds
      * MS_PER_SECOND)) && (!isStopped()); waited += STOP_POLL_MS) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- The sleep is sliced so a stop request is noticed within a second.
      await wait(STOP_POLL_MS,);
    }
  }
}
