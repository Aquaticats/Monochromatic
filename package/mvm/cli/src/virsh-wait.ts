/**
 VM lifecycle polling operations.
 
 Provides guest agent readiness checks and graceful shutdown
 waiting logic for libvirt VMs.
 
 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { MS_PER_SECOND, } from '@monochromatic-dev/module-const/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { VM_PREFIX, } from './config.ts';
import { virsh, } from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Milliseconds between guest agent ping attempts.
 */
const AGENT_POLL_INTERVAL_MS = MS_PER_SECOND;

/**
 Default maximum milliseconds to wait for guest agent before giving up.
 Five minutes: on 2026-10-05 a fresh Alpine VM's agent first answered 45 seconds
 after start on a busy host, and a Windows VM's agent about 80 seconds after a
 reboot, so the earlier 15 seconds failed `mvm create` although the VM was fine.
 The wait ends as soon as the agent answers.
 */
const DEFAULT_AGENT_TIMEOUT_MS = 300_000;

/**
 Milliseconds between VM state polls when waiting for shutdown.
 */
const SHUTDOWN_POLL_INTERVAL_MS = MS_PER_SECOND;

/**
 Maximum milliseconds to wait for graceful shutdown.
 */
const SHUTDOWN_TIMEOUT_MS = 120_000;

/**
 Polls the QEMU guest agent until it responds, indicating the VM has fully booted
 and is ready to accept commands.
 
 @param name - VM name without the mvm- prefix
 
 @param timeoutMs - Maximum milliseconds to wait (defaults to five minutes)
 
 @throws Error when the guest agent does not respond within the timeout
 
 @example
 ```ts
 await waitForGuestAgent({ name: 'dev-01' });
 await waitForGuestAgent({ name: 'template', timeoutMs: 120_000 });
 ```
 */
export async function waitForGuestAgent({
  name,
  timeoutMs = DEFAULT_AGENT_TIMEOUT_MS,
}: {
  readonly name: string;
  readonly timeoutMs?: number;
},): Promise<void> {
  /**
   Logger scoped to this call so retry logs are namespaced.
   */
  const rl = tagged({
    tag: waitForGuestAgent.name,
    l,
  },);
  /**
   Prefixed libvirt domain name; what {@link virsh} expects on the wire.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   Pre-serialised `guest-ping` payload; reused on every poll.
   */
  const pingPayload = JSON.stringify({ execute: 'guest-ping', },);
  /**
   Wall-clock reference for the timeout check; epoch ms at entry.
   */
  const startTime = Date.now();

  rl.info(`waiting for guest agent on ${name}...`,);

  /**
   Retry state; a failed attempt below the deadline explicitly requests the next poll.
   */
  const polling = { retryPending: true, };
  while (polling.retryPending) {
    polling.retryPending = false;
    try {
      // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling attempt
      await virsh({ args: [
        'qemu-agent-command',
        fullName,
        pingPayload,
      ], },);
      rl.info(`guest agent on ${name} is ready`,);
      return;
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      /**
       Milliseconds since polling began; compared against `timeoutMs` to give up.
       */
      const elapsed = Date.now()
        - startTime;
      if (elapsed >= timeoutMs) {
        throw new Error(
          `guest agent on ${name} did not respond within ${
            String(timeoutMs / MS_PER_SECOND,)
          }s`,
          { cause: error, },
        );
      }
      rl.debug(
        `guest agent not ready yet (${
          String(Math.round(elapsed / MS_PER_SECOND,),)
        }s elapsed), retrying...`,
      );
      // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling delay
      await wait(AGENT_POLL_INTERVAL_MS,);
      polling.retryPending = true;
    }
  }
}

/**
 Sends a graceful shutdown request to a VM via the guest agent.
 
 @param name - VM name without the mvm- prefix
 
 @example
 ```ts
 await shutdownVm({ name: 'dev-01' });
 ```
 */
export async function shutdownVm({ name, }: { readonly name: string; },): Promise<void> {
  /**
   Logger scoped to this shutdown call so the diagnostic catch is namespaced.
   */
  const rl = tagged({
    tag: shutdownVm.name,
    l,
  },);
  /**
   Prefixed libvirt domain name; what {@link virsh} expects on the wire.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   Serialised `guest-shutdown` request; the response is ignored because the agent dies mid-shutdown.
   */
  const payload = JSON.stringify({ execute: 'guest-shutdown', },);
  try {
    await virsh({ args: [
      'qemu-agent-command',
      fullName,
      payload,
    ], },);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // Guest agent often disconnects before sending a response during shutdown.
    // This is expected behavior: the VM is shutting down.
    rl.debug('guest agent disconnected during shutdown (expected)',);
  }
}

/**
 Polls VM state until it reaches "shut off", indicating graceful shutdown completed.
 
 @param name - VM name without the mvm- prefix
 
 @throws Error when the VM does not shut down within the timeout
 
 @example
 ```ts
 await waitForShutdown({ name: 'dev-01' });
 ```
 */
export async function waitForShutdown({ name, }: { readonly name: string; },): Promise<void> {
  /**
   Logger scoped to this call so polling logs are namespaced.
   */
  const rl = tagged({
    tag: waitForShutdown.name,
    l,
  },);
  /**
   Prefixed libvirt domain name; what `virsh domstate` expects on the wire.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   Wall-clock reference for the timeout check; epoch ms at entry.
   */
  const startTime = Date.now();

  rl.info(`waiting for VM ${name} to shut down...`,);

  /**
   Latest libvirt domain state; polling continues until it reaches `shut off`.
   */
  const polling = {
    state: await virsh({ args: [
      'domstate',
      fullName,
    ], },),
  };
  while (polling.state !== 'shut off') {
    /**
     Milliseconds since polling began; compared against the shutdown timeout to give up.
     */
    const elapsed = Date.now()
      - startTime;
    if (elapsed >= SHUTDOWN_TIMEOUT_MS) {
      throw new Error(
        `VM ${name} did not shut down within ${
          String(SHUTDOWN_TIMEOUT_MS / MS_PER_SECOND,)
        }s`,
      );
    }

    rl.debug(
      `VM state: ${polling.state} (${String(Math.round(elapsed / MS_PER_SECOND,),)}s elapsed)`,
    );
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling delay
    await wait(SHUTDOWN_POLL_INTERVAL_MS,);
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial state polling
    polling.state = await virsh({ args: [
      'domstate',
      fullName,
    ], },);
  }
  rl.info(`VM ${name} has shut down`,);
}
