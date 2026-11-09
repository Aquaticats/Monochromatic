import {
  resolveNotificationUser,
  type SudoEnvironment,
} from './notification-user.ts';
import {
  createFailureDelivery,
  type DeliveryRunner,
  type FailureDelivery,
} from './runtime-failure-report.ts';
import {
  POLICY_BYPASS_TABLE,
  POLICY_CLEARNET_HOSTS,
  policyNotInForce,
} from './update-policy-notice.ts';

/**
 Builds the loud report for a route monitor that failed and will be replaced.

 @param error - What the monitor session threw.

 @returns Report naming the policy that is not in force until the replacement runs.

 @example
 ```ts
 monitorFailureReport({ error: new Error('ip exited') });
 ```
 */
export function monitorFailureReport({ error, }: { readonly error: unknown; },): string {
  return policyNotInForce({
    policy: POLICY_BYPASS_TABLE,
    detail: `the route monitor failed (${String(error,)}) and will be restarted; `
      + 'until it runs again a change of the physical default route is not copied into the bypass table, '
      + 'so marked traffic can follow a route that no longer exists',
  },);
}

/**
 Builds the loud report for a watcher that stopped for good.

 @param error - What ended the watcher.

 @returns Report naming both policies the watcher keeps in force.

 @example
 ```ts
 watcherStoppedReport({ error: new Error('state vanished') });
 ```
 */
export function watcherStoppedReport({ error, }: { readonly error: unknown; },): string {
  return `${policyNotInForce({
    policy: POLICY_BYPASS_TABLE,
    detail: `the route watcher stopped (${String(error,)}) and the bypass table no longer follows the physical default route`,
  },)}; ${policyNotInForce({
    policy: POLICY_CLEARNET_HOSTS,
    detail: 'the same watcher stopped, so the host rules of the update hosts and the model provider hosts are no longer refreshed from DNS and stay as they are '
      + 'until `wg-quicker down` removes them; run `wg-quicker up` again after fixing the cause',
  },)}`;
}

/**
 Creates the delivery the detached watcher uses,
 because its own standard output and error are discarded.

 Failures go to the journal at error level and to the desktop of the user who started the tunnel through `sudo`,
 limited to one notification per distinct failure per ten minutes.

 @param environment - Environment of the watcher,
 where `sudo` recorded the user.

 @param run - Command runner,
 replaceable in tests.

 @returns The delivery function.

 @example
 ```ts
 const deliver = createWatcherDelivery({ environment: process.env });
 ```
 */
export function createWatcherDelivery(
  {
    environment,
    run,
  }: {
    readonly environment: SudoEnvironment;
    readonly run?: DeliveryRunner;
  },
): FailureDelivery {
  return createFailureDelivery({
    lookup: resolveNotificationUser({ environment, },),
    ...(run === undefined ? {} : { run, }),
  },);
}

/**
 Builds the loud report for a route watcher that died without being stopped.

 @param pid - Process identifier of the dead watcher.

 @returns Report naming both policies the watcher enforced.

 @example
 ```ts
 watcherDiedReport({ pid: 4242 });
 ```
 */
export function watcherDiedReport({ pid, }: { readonly pid: number; },): string {
  return `${policyNotInForce({
    policy: POLICY_BYPASS_TABLE,
    detail: `the route watcher (pid ${String(pid,)}) is no longer running and wg-quicker did not stop it (it was killed, ran out of memory or crashed), `
      + 'so the bypass table no longer follows the physical default route',
  },)}; ${policyNotInForce({
    policy: POLICY_CLEARNET_HOSTS,
    detail: 'the same watcher died, so the host rules of the update hosts and the model provider hosts are no longer refreshed from DNS and stay as they are '
      + 'until `wg-quicker down` removes them; run `wg-quicker up` again',
  },)}`;
}

/**
 Builds the loud report for a route watcher that ended while its identity record cannot be read.

 @param pid - Process identifier of the watcher that ended.

 @param reason - Why the record cannot be read.

 @returns Report saying it cannot be told whether wg-quicker asked for the end.

 @example
 ```ts
 watcherEndUnknownReport({ pid: 4242, reason: 'EACCES' });
 ```
 */
export function watcherEndUnknownReport(
  {
    pid,
    reason,
  }: {
    readonly pid: number;
    readonly reason: string;
  },
): string {
  return policyNotInForce({
    policy: POLICY_BYPASS_TABLE,
    detail: `the route watcher (pid ${String(pid,)}) is no longer running and its identity record cannot be read (${reason}), `
      + 'so it cannot be told whether wg-quicker asked for that; if not, the bypass table no longer follows the physical default route '
      + 'and the host rules of the update hosts and the model provider hosts are no longer refreshed',
  },);
}
