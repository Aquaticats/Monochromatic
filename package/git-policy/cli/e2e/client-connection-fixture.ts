/**
 Observation of a wrapper's connection to the commit service.

 The client prepares its commit before it connects, so an open socket in the wrapper process marks the point at which
 preparation is over and the commit waits for the service. A scenario that froze the service learns from this that a
 client has prepared against the branch as it is, without any program running inside the wrapper.

 @module
 */

import {
  readdir,
  readlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setInterval as ticks, } from 'node:timers/promises';

import type { RunningProcess, } from './process-fixture.ts';

//region Constants

/**
 Longest wait for a client to connect, in milliseconds.
 */
export const CONNECTION_TIMEOUT_MS = 60_000;

/**
 Pause between two polls of the process's descriptors, in milliseconds.
 */
const POLL_MILLISECONDS = 2;

/**
 Prefix of the link text of a socket descriptor.
 */
const SOCKET_LINK_PREFIX = 'socket:';

/**
 Standard descriptors,
 which the harness's own pipes to the wrapper occupy as sockets and which therefore say nothing about the service.
 */
const STANDARD_DESCRIPTORS: ReadonlySet<string> = new Set([
  '0',
  '1',
  '2',
]);

//endregion Constants

//region Observation

/**
 Whether a process holds an open socket other than its standard descriptors.

 @param pid - process ID

 @returns true when one of its descriptors is a socket,
 false when it holds none or has ended

 @example
 ```ts
 await holdsSocket(running.pid);
 ```
 */
export async function holdsSocket(pid: number,): Promise<boolean> {
  /**
   Directory of the process's open descriptors.
   */
  const directory = join(
    '/proc',
    String(pid,),
    'fd',
  );
  try {
    /**
     Link text of every descriptor.
     */
    const links = await Promise.all((await readdir(directory,)).filter(function nonStandard(descriptor,) {
      return !STANDARD_DESCRIPTORS.has(descriptor,);
    },)
      .map(async function target(descriptor,) {
        try {
          return await readlink(join(
            directory,
            descriptor,
          ),);
        }
        catch (error: unknown) {
        // A descriptor closed between the listing and the read.
          if (Error.isError(error,) && ('code' in error)
            && (error.code === 'ENOENT'))
            return '';
          throw error;
        }
      },),);
    return links.some(function isSocket(link,) {
      return link.startsWith(SOCKET_LINK_PREFIX,);
    },);
  }
  catch (error: unknown) {
    // The process ended: it holds nothing.
    if (Error.isError(error,) && ('code' in error)
      && ((error.code === 'ENOENT') || (error.code === 'ESRCH')))
      return false;
    throw error;
  }
}

/**
 Waits until a wrapper has connected to the service or settled.

 @param running - the wrapper process

 @returns `connected` when it holds a socket,
 `settled` when it ended first,
 `timeout` when neither happened in time

 @example
 ```ts
 await waitForConnection(attempt.running);
 ```
 */
export async function waitForConnection(running: RunningProcess,): Promise<'connected' | 'settled' | 'timeout'> {
  /**
   When the wait ends.
   */
  const deadline = Date.now() + CONNECTION_TIMEOUT_MS;
  for await (const _tick of ticks(POLL_MILLISECONDS,)) {
    if (running.isSettled())
      return 'settled';
    if (await holdsSocket(running.pid,))
      return 'connected';
    if (Date.now() >= deadline)
      return 'timeout';
  }
  return 'timeout';
}

//endregion Observation
