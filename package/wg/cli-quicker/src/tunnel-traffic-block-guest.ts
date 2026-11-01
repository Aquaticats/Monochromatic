/**
 Network namespaces inside the disposable namespace of a scenario,
 for scenarios that need a packet to arrive from elsewhere and be forwarded,
 and a listener elsewhere that answers it.

 @module
 */

import { spawn, } from 'node:child_process';
import { readlink, } from 'node:fs/promises';
import { setTimeout as wait, } from 'node:timers/promises';

import {
  run,
  runAllowingFailure,
} from '../dist/final/node/runner.mjs';

/**
 Seconds a namespace's process lives at most,
 so that no run can leave it behind.
 */
const GUEST_LIFETIME_SECONDS = '120';

/**
 Time between looks at whether a namespace or a listener is ready.
 */
const GUEST_POLL_MS = 50;

/**
 How many looks are taken before giving up.
 */
const GUEST_POLL_COUNT = 100;

/**
 Python that listens on a TCP port on every address of the namespace and closes each connection it accepts.
 */
const LISTENER_PROGRAM = [
  'import socket, sys',
  's = socket.socket()',
  's.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)',
  's.bind(("", int(sys.argv[1])))',
  's.listen(8)',
  'while True:',
  '    connection, _ = s.accept()',
  '    connection.close()',
].join('\n',);

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
  const { pid, } = child;
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
 Reads the hardware address of an interface,
 in this namespace or in a guest's,
 which `/sys` does not show for a namespace other than the host's.

 @param name - Interface name.

 @param guest - Guest whose namespace the interface is in,
 or nothing for this namespace.

 @returns The hardware address.

 @throws {@link Error} when `ip` does not report an address.

 @example
 ```ts
 await linkAddress({ name: 'veth0' });
 ```
 */
export async function linkAddress({
  name,
  guest,
}: {
  readonly name: string;
  readonly guest?: Guest
},): Promise<string> {
  /**
   Arguments of `ip` that list the interface as JSON.
   */
  const args = [
    '-j',
    'link',
    'show',
    'dev',
    name,
  ];
  /**
   JSON listing of the interface.
   */
  const listed: unknown = JSON.parse((guest === undefined
    ? await run({
      command: 'ip',
      args,
    },)
    : await inGuest({
      guest,
      command: 'ip',
      args,
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

/**
 A listening process in a guest's namespace,
 stopped when the scope ends.
 */
export type Listener = {
  /**
   TCP port the listener listens on.
   */
  readonly port: number;

  /**
   Stops the listener.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Starts a TCP listener in a guest's namespace and waits until it listens.
 It accepts a connection and closes it at once,
 which is all a connect probe needs.

 @param guest - Guest whose namespace the listener runs in.

 @param port - TCP port to listen on.

 @returns The listener,
 disposed by `await using`.

 @throws {@link Error} when the listener does not listen in time.

 @example
 ```ts
 await using listener = await startListener({ guest, port: 8080 });
 ```
 */
export async function startListener({
  guest,
  port,
}: {
  readonly guest: Guest;
  readonly port: number
},): Promise<Listener> {
  /**
   The listener process.
   */
  const child = spawn(
    'nsenter',
    [
      '--target',
      String(guest.pid,),
      '--net',
      'python3',
      '-I',
      '-c',
      LISTENER_PROGRAM,
      String(port,),
    ],
    { stdio: 'ignore', },
  );
  for (let look = 0; look < GUEST_POLL_COUNT; look += 1) {
    /**
     The listening sockets of the namespace.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- The listener starts listening some time after the process starts.
    const sockets = await runAllowingFailure({
      command: 'nsenter',
      args: [
        '--target',
        String(guest.pid,),
        '--net',
        'ss',
        '--listening',
        '--tcp',
        '--numeric',
        '--no-header',
      ],
    },);
    if (sockets.stdout
      .includes(`:${String(port,)} `,))
      return {
        port,
        [Symbol.asyncDispose]: function stopListener(): Promise<void> {
          child.kill('SIGKILL',);
          return Promise.resolve();
        },
      };
    // oxlint-disable-next-line eslint/no-await-in-loop -- The listener starts listening some time after the process starts.
    await wait(GUEST_POLL_MS,);
  }
  child.kill('SIGKILL',);
  throw new Error(`the listener did not listen on port ${String(port,)}`,);
}
