/**
 Backend-neutral ephemeral-run helper.
 
 Creates (or clones) a throwaway VM, runs a command in it, then destroys it,
 guaranteeing teardown on SIGINT/SIGTERM and scope exit. Parameterised over a
 backend's create/clone/destroy/exec so both the libvirt `run` and the
 Hetzner backend share one teardown-correct implementation.
 
 @module
 */

import { randomBytes, } from 'node:crypto';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { ExecResult, } from '../exec.ts';
import type { Backend, } from './types.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Number of random bytes used to generate the ephemeral VM name suffix.
 */
const NAME_RANDOM_BYTES = 4;

/**
 Subset of {@link Backend} operations an ephemeral run needs.
 Passing only these avoids a circular import between `run.ts` and the libvirt
 backend object that aggregates `run` itself.
 
 @example
 ```ts
 const ops: EphemeralOps = { clone, create, destroy, exec };
 ```
 */
export type EphemeralOps = Pick<
  Backend,
  'clone' | 'create' | 'destroy' | 'exec'
  >;

/**
 Generates a unique ephemeral VM name using a random hex suffix.
 
 @returns name in the format `ephemeral-<hex>`
 
 @example
 ```ts
 generateEphemeralName(); // => "ephemeral-a1b2c3d4"
 ```
 */
function generateEphemeralName(): string {
  /**
   Hex-encoded random suffix; keeps collision probability negligible across concurrent ephemeral VMs.
   */
  const suffix = randomBytes(NAME_RANDOM_BYTES,)
    .toString('hex',);
  return `ephemeral-${suffix}`;
}

/**
 Creates an ephemeral VM, executes a command inside it, and destroys the VM.
 When `from` is provided, the VM is cloned from that source; otherwise a fresh
 VM is created. Registers SIGINT/SIGTERM handlers and an `await using` guard so
 the VM is destroyed even on interruption or thrown errors.
 
 @param command - shell command to run inside the VM
 
 @param from - source VM to clone from (fresh VM when omitted)
 
 @param ops - backend create/clone/destroy/exec used to provision and run
 
 @returns captured stdout, stderr, and exit code from the command
 
 @throws Error when VM creation, command execution, or cleanup fails
 
 @example
 ```ts
 const result = await ephemeralRun({ command: 'uname -a', ops: libvirtOps });
 ```
 */
export async function ephemeralRun(
  {
    command,
    from,
    ops,
  }: {
    readonly command: string;
    readonly from?: string;
    readonly ops: EphemeralOps;
  },
): Promise<ExecResult> {
  /**
   Logger scoped to this run so VM lifecycle steps log under the right name.
   */
  const rl = tagged({
    tag: ephemeralRun.name,
    l,
  },);
  /**
   Ephemeral VM name; unique per invocation so concurrent runs do not collide.
   */
  const name = generateEphemeralName();

  rl.info(
    `ephemeral VM: ${name}${from !== undefined ? ` (cloned from ${from})` : ' (fresh)'}`,
  );

  /**
   Destroys the ephemeral VM, logging but not re-throwing errors.
   */
  async function cleanup(): Promise<void> {
    rl.info(`destroying ephemeral VM ${name}`,);
    try {
      await ops.destroy({ name, },);
    }
    catch (err: unknown) {
      rl.info(
        `cleanup failed for ${name}: ${
          caughtValueText(err,)
        }`,
      );
    }
  }

  /**
   Signal handler that runs cleanup then re-raises the signal.
   
   @param signal - signal name (`SIGINT` or `SIGTERM`)
   */
  function onSignal(signal: NodeJS.Signals,): void {
    rl.info(`received ${signal}, cleaning up...`,);
    void (async function cleanupAndReraise(): Promise<void> {
      try {
        await cleanup();
      }
      catch (error) {
        if (!(Error.isError(error,)))
          throw error;

        // cleanup() already logs errors internally
      }
      process.kill(
        process.pid,
        signal,
      );
    })();
  }

  process.on(
    'SIGINT',
    onSignal,
  );
  process.on(
    'SIGTERM',
    onSignal,
  );

  /**
   Disposable guard that detaches the signal handlers and destroys the ephemeral VM on scope exit.
   */
  await using _guard = {
    async [Symbol.asyncDispose](): Promise<void> {
      process.removeListener(
        'SIGINT',
        onSignal,
      );
      process.removeListener(
        'SIGTERM',
        onSignal,
      );
      await cleanup();
    },
  };

  await (from !== undefined
    ? ops.clone({
      destination: name,
      source: from,
    },)
    : ops.create({ name, },));

  /**
   Captured stdio and exit code from the guest command; returned to the caller.
   */
  const result = await ops.exec({
    command,
    name,
  },);
  return result;
}
