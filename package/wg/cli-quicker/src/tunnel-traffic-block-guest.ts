/**
 A guest network namespace inside the disposable namespace of a scenario,
 for scenarios that need a packet to arrive from elsewhere and be forwarded.

 @module
 */

import { spawn, } from 'node:child_process';
import { readlink, } from 'node:fs/promises';
import { setTimeout as wait, } from 'node:timers/promises';

import { run, } from '../dist/final/node/runner.mjs';

/**
 Seconds the guest process lives at most,
 so that no run can leave it behind.
 */
const GUEST_LIFETIME_SECONDS = '120';

/**
 Time between looks at whether the guest has its own network namespace.
 */
const GUEST_POLL_MS = 50;

/**
 How many looks are taken before the guest is given up on.
 */
const GUEST_POLL_COUNT = 100;

/**
 A process in a network namespace of its own,
 stopped when the scope ends.
 */
export type Guest = {
  /**
   Process ID that `nsenter` targets.
   */
  readonly pid: number;

  /**
   Stops the process,
   which removes its namespace.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Starts a process in a new network namespace of this user namespace and waits until the namespace exists.

 @returns The guest,
 disposed by `await using`.

 @example
 ```ts
 await using guest = await startGuest();
 ```
 */
export async function startGuest(): Promise<Guest> {
  /**
   Own namespace to tell the guest's from.
   */
  const own = await readlink('/proc/self/ns/net',);
  /**
   The guest process.
   */
  const child = spawn(
    'unshare',
    [
      '--net',
      'sleep',
      GUEST_LIFETIME_SECONDS,
    ],
    { stdio: 'ignore', },
  );
  /**
   Process ID of the guest,
   which `spawn` reports once the process started.
   */
  const {pid} = child;
  if (pid === undefined)
    throw new Error('the guest process did not start',);
  for (let look = 0; look < GUEST_POLL_COUNT; look += 1) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- The namespace appears some time after the process starts.
    if (await readlink(`/proc/${String(pid,)}/ns/net`,) !== own)
      break;
    // oxlint-disable-next-line eslint/no-await-in-loop -- The namespace appears some time after the process starts.
    await wait(GUEST_POLL_MS,);
  }
  return {
    pid,
    [Symbol.asyncDispose]: function stopGuest(): Promise<void> {
      child.kill('SIGKILL',);
      return Promise.resolve();
    },
  };
}

/**
 Runs a command in the guest's network namespace.

 @param guest - The guest.

 @param command - Executable.

 @param args - Arguments.

 @returns What `run` reports.

 @example
 ```ts
 await inGuest({ guest, command: 'ip', args: ['link'] });
 ```
 */
export function inGuest(
  {
    guest,
    command,
    args,
  }: {
    readonly guest: Guest;
    readonly command: string;
    readonly args: readonly string[]
  },
): ReturnType<typeof run> {
  return run({
    command: 'nsenter',
    args: [
      '--target',
      String(guest.pid,),
      '--net',
      command,
      ...args,
    ],
  },);
}

/**
 Runs `ip` in this namespace.

 @param args - Arguments of `ip`.

 @example
 ```ts
 await ip(['link', 'set', 'lo', 'up']);
 ```
 */
export async function ip(args: readonly string[],): Promise<void> {
  await run({
    command: 'ip',
    args,
  },);
}

/**
 Reads the hardware address of an interface of this namespace,
 which `/sys` does not show for a namespace other than the host's.

 @param name - Interface name.

 @returns The hardware address.

 @throws {@link Error} when `ip` does not report an address.

 @example
 ```ts
 await linkAddress('veth0');
 ```
 */
export async function linkAddress(name: string,): Promise<string> {
  /**
   JSON listing of the interface.
   */
  const listed: unknown = JSON.parse((await run({
    command: 'ip',
    args: [
      '-j',
      'link',
      'show',
      'dev',
      name,
    ],
  },)).stdout,);
  /**
   First interface of the listing.
   */
  const first: unknown = Array.isArray(listed,) ? listed[0] : undefined;
  if (((typeof first) === 'object') && (first !== null)
    && ('address' in first)
    && ((typeof first.address) === 'string'))
    return first.address;
  throw new Error(`ip did not report a hardware address for ${name}`,);
}
