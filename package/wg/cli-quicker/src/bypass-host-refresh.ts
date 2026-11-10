import { setTimeout as sleep, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type HostListRead,
  type HostListReader,
  createInstalledClearnetHostsReader,
} from './installed-clearnet-hosts.ts';
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

 This is the list this build compiles in.
 A running watcher reads the same lists from the data file beside its bundle at every refresh,
 so a newer build's lists reach a tunnel without a restart,
 and this list is what a watcher falls back to when that file cannot be read.
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

 @param hosts - Every host this refresh installs a rule set for,
 read from the data file beside the running build.

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
 await refreshUpdateHostRules({ state, absentCounts: new Map(), hosts: CLEARNET_HOSTS });
 ```
 */
export async function refreshUpdateHostRules(
  {
    state,
    absentCounts,
    hosts,
    lookup = createTunnelHostLookup({ interfaceName: state.interfaceName, },),
    tools = IP_HOST_RULE_TOOLS,
    readDependencies = readTunnelDependencies,
    deliver,
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly hosts: readonly string[];
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
    hosts: hasDns ? hosts : [],
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

 @param hosts - Every host this refresh installs a rule set for.

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
 await refreshOrReport({ state, absentCounts: new Map(), hosts: CLEARNET_HOSTS });
 ```
 */
export async function refreshOrReport(
  {
    state,
    absentCounts,
    hosts,
    lookup,
    tools,
    readDependencies,
    deliver,
  }: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly hosts: readonly string[];
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
      hosts,
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
 What the refresh loop carries from one round to the next.
 */
type HostRefreshCarried = {
  /**
   Absence counters of the rules the kernel lists,
   kept across rounds so a rotating address is not removed at once.
   */
  readonly absentCounts: Map<string, number>;

  /**
   Hosts of the last data file this watcher read.
   */
  hosts: readonly string[];
};

/**
 The guarded refresh the loop runs,
 replaceable in tests.
 */
export type GuardedRefresh = (
  call: {
    readonly state: BypassState;
    readonly absentCounts: Map<string, number>;
    readonly hosts: readonly string[];
    readonly deliver?: FailureDelivery;
  },
) => Promise<number>;

/**
 Waits out the time until the next refresh,
 replaceable in tests.

 @param call - Seconds asked for and the signal that ends the wait.

 @returns Nothing once the wait is over.

 @example
 ```ts
 await pauseUntilNextRefresh({ seconds: 30, signal });
 ```
 */
export type RefreshPause = (call: {
  readonly seconds: number;
  readonly signal: AbortSignal
},) => Promise<void>;

/**
 Builds the loud report of a host list that cannot be read.

 The rules already installed stay in force,
 because a list that cannot be read says nothing about which hosts belong outside the tunnel,
 and nothing is removed because of it.

 @param failure - What is wrong with the data file,
 as the reader names it.

 @param retrySeconds - Seconds until the next attempt.

 @returns Report naming the policy,
 the file,
 and what stays in force.

 @example
 ```ts
 hostListReadFailureText({ failure: 'x is not JSON', path: 'data.json', retrySeconds: 30 });
 ```
 */
export function hostListReadFailureText(
  {
    failure,
    retrySeconds,
  }: {
    readonly failure: string;
    readonly retrySeconds: number;
  },
): string {
  return policyNotInForce({
    policy: POLICY_CLEARNET_HOSTS,
    detail: `the host list installed beside the route watcher cannot be read: ${failure}; `
      + `the list this watcher read last stays in force, `
      + `so no rule is added or removed for the change and no routing is removed; `
      + `retrying in ${String(retrySeconds,)} s`,
  },);
}

/**
 Reads the clearnet host list for a start of the tunnel.

 A start installs the list compiled into its own build when the data file cannot be read,
 because the file the running build carries is the list that build is,
 and a start that installed nothing would leave every update and provider request in the tunnel.

 @param read - Reader of the data file,
 the installed copy beside the bundle by default.

 @returns Every host to install rules for.

 @example
 ```ts
 const hosts = await readClearnetHostsForStart({});
 ```
 */
export async function readClearnetHostsForStart(
  { read = createInstalledClearnetHostsReader({},), }: {
    readonly read?: HostListReader;
  } = {},
): Promise<readonly string[]> {
  /**
   Reading of the data file besides this build.
   */
  const result: HostListRead = await read();
  if (result.kind === 'read')
    return result.hosts;
  /**
   Loud report of a build whose data file cannot be read.
   */
  const loud = policyNotInForce({
    policy: POLICY_CLEARNET_HOSTS,
    detail: `the host list installed beside this build cannot be read: ${result.failure}; `
      + `this start installs the ${String(CLEARNET_HOSTS.length,)} hosts compiled into the build instead`,
  },);
  l.error(loud,);
  return CLEARNET_HOSTS;
}

/**
 One round of the refresh loop:
 read the list the installed build carries now, reconcile with it, then wait for the next round.

 A file that cannot be read leaves the list this watcher read last in force,
 is reported where a person sees it,
 and is read again after the short retry delay instead of the refresh interval.

 @param carried - Hosts and absence counters the loop keeps between rounds.

 @param state - Claimed table and preference.

 @param signal - Aborted when the watcher is asked to stop.

 @param deliver - Delivery of a loud report to a person.

 @param refresh - The guarded refresh,
 replaceable in tests.

 @param readHosts - Reader of the installed host list,
 replaceable in tests.

 @param pause - Wait until the next refresh,
 replaceable in tests.

 @example
 ```ts
 await runHostRound({ carried, state, signal: controller.signal });
 ```
 */
async function runHostRound(
  {
    carried,
    state,
    signal,
    deliver,
    refresh,
    readHosts,
    pause,
  }: {
    readonly carried: HostRefreshCarried;
    readonly state: BypassState;
    readonly signal: AbortSignal;
    readonly deliver?: FailureDelivery;
    readonly refresh: GuardedRefresh;
    readonly readHosts: HostListReader;
    readonly pause: RefreshPause;
  },
): Promise<void> {
  /**
   This round's reading of the data file the installed build carries.
   */
  const read: HostListRead = await readHosts();
  /**
   Whether this round follows a data file that cannot be read.
   */
  const unreadable = read.kind === 'failed';
  if (read.kind === 'read') {
    carried.hosts = read.hosts;
  }
  else {
    /**
     Loud report of a list that cannot be read,
     which leaves the list read last in force.
     */
    const loud = hostListReadFailureText({
      failure: read.failure,
      retrySeconds: FAILED_REFRESH_RETRY_SECONDS,
    },);
    l.error(loud,);
    await deliver?.({
      kind: KIND_UPDATE_HOST_REFRESH_FAILED,
      line: loud,
    },);
  }
  /**
   Seconds the refresh asks to wait before the next one.
   */
  const seconds = await refresh({
    state,
    absentCounts: carried.absentCounts,
    hosts: carried.hosts,
    ...(deliver === undefined ? {} : { deliver, }),
  },);
  await pause({
    seconds: unreadable ? FAILED_REFRESH_RETRY_SECONDS : seconds,
    signal,
  },);
}

/**
 Refreshes the clearnet-host rules, of the update hosts and the model provider hosts, until the watcher is asked to stop.

 Each round reads the data file beside the running build first,
 so a list a newer build installs there takes effect in this watcher without a restart,
 between two rules that both lists name nothing is removed,
 and a file that cannot be read leaves the list read last in force with a loud report.

 @param state - Claimed table and preference.

 @param signal - Aborted when the watcher is asked to stop.

 @param deliver - Delivery of a loud report to a person,
 which the detached watcher needs because its own output is discarded.

 @param refresh - The guarded refresh,
 replaceable in tests.

 @param readHosts - Reader of the installed host list,
 replaceable in tests.

 @param pause - Wait until the next refresh,
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
    refresh = refreshOrReport,
    readHosts = createInstalledClearnetHostsReader({},),
    pause = sleepUnlessAborted,
  }: {
    readonly state: BypassState;
    readonly signal: AbortSignal;
    readonly deliver?: FailureDelivery;
    readonly refresh?: GuardedRefresh;
    readonly readHosts?: HostListReader;
    readonly pause?: RefreshPause;
  },
): Promise<void> {
  /**
   What the loop carries from one round to the next:
   the absence counters of the rules the kernel lists,
   and the hosts of the last data file this watcher read,
   the list compiled into this build before the first read succeeds.
   */
  const carried: HostRefreshCarried = {
    absentCounts: new Map<string, number>(),
    hosts: CLEARNET_HOSTS,
  };
  while (!signal.aborted) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- See doc/lint/no-await-in-loop-host-refresh.md: each round reads the list the installed build carries now and waits for the next one.
    await runHostRound({
      carried,
      state,
      signal,
      ...(deliver === undefined ? {} : { deliver, }),
      refresh,
      readHosts,
      pause,
    },);
  }
}
