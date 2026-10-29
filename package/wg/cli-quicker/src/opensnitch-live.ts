import { readlink, } from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/module-async-time/ts';

import {
  OpenSnitchConfigError,
  OpenSnitchLiveReloadError,
} from './errors.ts';
import { currentNetworkNamespace, } from './network-namespace.ts';
import { runAllowingFailure, } from './runner.ts';

/**
 Delay between live-reload verification probes.
 */
const RELOAD_PROBE_INTERVAL_MS = 100;

/**
 Bounded number of probes allowing OpenSnitch file watcher to rebuild nftables rules.
 */
const RELOAD_PROBE_ATTEMPTS = 40;

/**
 Consecutive healthy observations required after file watcher reload.
 */
const RELOAD_STABLE_PROBES = 3;

/**
 Deep-readonly projection of one settled namespace probe.
 */
type NamespaceProbeResult = {
  readonly status: 'fulfilled';
  readonly value: string;
} | {
  readonly status: 'rejected';
  readonly reason: unknown;
};

/**
 Reports whether OpenSnitch daemon is active in current network namespace.
 
 Exit status one from `pgrep` proves process absence.
 Namespace comparison excludes daemons attached to unrelated disposable fixtures;
 other probe failures are operational errors rather than absence.
 
 @returns Whether exact daemon process name is active.
 
 @throws {@link OpenSnitchConfigError} when process probe itself fails.
 
 @example
 ```ts
 await isOpenSnitchDaemonActive();
 ```
 */
async function isOpenSnitchDaemonActive(): Promise<boolean> {
  /**
   Exact process-name probe returning candidate process IDs.
   */
  const result = await runAllowingFailure({
    command: 'pgrep',
    args: [
      '--exact',
      'opensnitchd',
    ],
  },);
  if (result.exitCode === 1)
    return false;
  if (result.exitCode !== 0) {
    throw new OpenSnitchConfigError(
      `Cannot determine whether OpenSnitch daemon is active: ${result.stderr}`,
    );
  }
  /**
   Current network-namespace identity.
   */
  const { identity: currentNamespace, } = await currentNetworkNamespace();
  /**
   Candidate daemon process identifiers from pgrep.
   */
  const processIds = result
    .stdout
    .split('\n',)
    .filter(function nonempty(value,): boolean {
      return value !== '';
    },);
  /**
   Namespace probes tolerate candidates exiting between pgrep and readlink.
   */
  const namespaces = await Promise.allSettled(processIds.map(async function processNamespace(
    processId,
  ): Promise<string> {
    return await readlink(`/proc/${processId}/ns/net`,);
  },),) as readonly NamespaceProbeResult[];
  return namespaces.some(function sameNamespace(candidate,): boolean {
    return (candidate.status === 'fulfilled') && (candidate.value === currentNamespace);
  },);
}

/**
 Reports whether live output chain contains correctly ordered allowances.
 
 @param output - Numeric nft chain listing.
 
 @param requiredPorts - UDP destination ports that must precede NFQUEUE.
 
 @param forbiddenPorts - Formerly owned ports that must be absent.
 
 @returns Whether live reload reached usable strict-deny state.
 
 @example
 ```ts
 isLiveRuleSetReady({
   output: 'udp dport 51820 accept\nqueue',
   requiredPorts: [51820],
   forbiddenPorts: [],
 });
 ```
 */
export function isLiveRuleSetReady(
  {
    output,
    requiredPorts,
    forbiddenPorts,
  }: {
    readonly output: string;
    readonly requiredPorts: readonly number[];
    readonly forbiddenPorts: readonly number[];
  },
): boolean {
  /**
   First application-filter queue position establishing ordering boundary.
   */
  const queueIndex = output.indexOf('queue',);
  if (queueIndex === (-1))
    return false;
  /**
   Resolves exact generated accept-rule position.
   
   @param port - Exact UDP destination port.
   
   @returns Character offset or negative absence sentinel.
   */
  function portIndex(port: number,): number {
    return output.indexOf(`udp dport ${String(port,)} accept`,);
  }
  if (!requiredPorts.every(function precedesQueue(port,): boolean {
    /**
     Exact generated rule offset.
     */
    const index = portIndex(port,);
    return (index >= 0) && (index < queueIndex);
  },))
    return false;
  return forbiddenPorts.every(function excludesPort(port,): boolean {
    return portIndex(port,) < 0;
  },);
}

/**
 Reports whether processless OpenSnitch chain is safe for requested transition.
 
 Without NFQUEUE,
 positive allowances are unnecessary until daemon reloads persisted config.
 Forbidden exact ports must never remain.
 
 @param output - Numeric stale chain listing.
 
 @param requiredPorts - Ports needed when stale NFQUEUE remains.
 
 @param forbiddenPorts - Removed exact ports required absent.
 
 @returns Whether processless chain cannot preserve or cause policy failure.
 
 @example
 ```ts
 isInactiveRuleSetSafe({ output: '', requiredPorts: [51820], forbiddenPorts: [] });
 ```
 */
export function isInactiveRuleSetSafe(
  {
    output,
    requiredPorts,
    forbiddenPorts,
  }: {
    readonly output: string;
    readonly requiredPorts: readonly number[];
    readonly forbiddenPorts: readonly number[];
  },
): boolean {
  /**
   Reports exact generated allowance presence.
   
   @param port - Exact UDP destination port.
   
   @returns Whether stale chain contains generated accept rule.
   */
  function containsPort(port: number,): boolean {
    return output.includes(`udp dport ${String(port,)} accept`,);
  }
  if (forbiddenPorts.some(containsPort,))
    return false;
  if (!output.includes('queue',))
    return true;
  return isLiveRuleSetReady({
    output,
    requiredPorts,
    forbiddenPorts,
  },);
}

/**
 Verifies stale kernel state when no OpenSnitch daemon process exists.
 
 Absent table is safe because next daemon start reads persisted config.
 Existing table must not retain forbidden ports;
 an existing NFQUEUE path must already carry required ordered allowances.
 
 @param path - Config path used in failure diagnostic.
 
 @param requiredPorts - Managed endpoint ports needed when stale NFQUEUE remains.
 
 @param forbiddenPorts - Removed exact ports required absent from stale chain.
 
 @throws {@link OpenSnitchConfigError} when nftables inspection fails.
 
 @throws {@link OpenSnitchLiveReloadError} when stale kernel state is unsafe.
 
 @example
 ```ts
 await verifyInactiveOpenSnitchKernelState({
   path,
   requiredPorts: [51820],
   forbiddenPorts: [],
 });
 ```
 */
async function verifyInactiveOpenSnitchKernelState(
  {
    path,
    requiredPorts,
    forbiddenPorts,
  }: {
    readonly path: string;
    readonly requiredPorts: readonly number[];
    readonly forbiddenPorts: readonly number[];
  },
): Promise<void> {
  /**
   Full table listing distinguishes absent table from failed chain probe.
   */
  const tables = await runAllowingFailure({
    command: 'nft',
    args: [
      '--numeric',
      'list',
      'tables',
    ],
  },);
  if (tables.exitCode !== 0) {
    throw new OpenSnitchConfigError(
      `Cannot inspect nftables tables while OpenSnitch daemon is stopped: ${tables.stderr}`,
    );
  }
  if (!tables
    .stdout
    .includes('table inet opensnitch',))
    return;
  /**
   Existing stale OpenSnitch output chain.
   */
  const chain = await runAllowingFailure({
    command: 'nft',
    args: [
      '--numeric',
      'list',
      'chain',
      'inet',
      'opensnitch',
      'mangle_output',
    ],
  },);
  if (chain.exitCode !== 0) {
    throw new OpenSnitchConfigError(
      `Cannot inspect stale OpenSnitch output chain while daemon is stopped: ${chain.stderr}`,
    );
  }
  if (!isInactiveRuleSetSafe({
    output: chain.stdout,
    requiredPorts,
    forbiddenPorts,
  },)) {
    throw new OpenSnitchLiveReloadError(
      `Stopped OpenSnitch left stale or incomplete wg-quicker endpoint rules after reading ${path}.`,
    );
  }
}

/**
 Waits for active OpenSnitch daemon to apply config through file watcher.
 
 Confirmed daemon absence still inspects any surviving nftables table.
 
 @param path - Config path used in failure diagnostic.
 
 @param requiredPorts - Managed endpoint ports expected before live NFQUEUE rules.
 
 @param forbiddenPorts - Removed exact ports expected absent from live chain.
 
 @throws {@link OpenSnitchConfigError} when active daemon does not converge.
 
 @example
 ```ts
 await verifyOpenSnitchLiveReload({
   path,
   requiredPorts: [51820],
   forbiddenPorts: [],
 });
 ```
 */
export async function verifyOpenSnitchLiveReload(
  {
    path,
    requiredPorts,
    forbiddenPorts,
  }: {
    readonly path: string;
    readonly requiredPorts: readonly number[];
    readonly forbiddenPorts: readonly number[];
  },
): Promise<void> {
  if (!(await isOpenSnitchDaemonActive())) {
    await verifyInactiveOpenSnitchKernelState({
      path,
      requiredPorts,
      forbiddenPorts,
    },);
    return;
  }
  /**
   Mutable bounded cursor contained in object rather than function-root binding.
   */
  const cursor = {
    attempt: 0,
    consecutiveReady: 0,
  };
  /* oxlint-disable eslint/no-await-in-loop -- Each probe depends on prior external state and delay. */
  while (cursor.attempt < RELOAD_PROBE_ATTEMPTS) {
    await wait(RELOAD_PROBE_INTERVAL_MS,);
    /**
     Current numeric output-chain listing after one reload interval.
     */
    const result = await runAllowingFailure({
      command: 'nft',
      args: [
        '--numeric',
        'list',
        'chain',
        'inet',
        'opensnitch',
        'mangle_output',
      ],
    },);
    if ((result.exitCode === 0) && isLiveRuleSetReady({
      output: result.stdout,
      requiredPorts,
      forbiddenPorts,
    },))
      cursor.consecutiveReady += 1;
    else
      cursor.consecutiveReady = 0;
    if (cursor.consecutiveReady >= RELOAD_STABLE_PROBES)
      return;
    cursor.attempt += 1;
  }
  /* oxlint-enable eslint/no-await-in-loop */
  throw new OpenSnitchLiveReloadError(
    `OpenSnitch did not converge after reading ${path}; its live nftables chain has missing, stale, or misordered rules.`,
  );
}
