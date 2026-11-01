import {
  readdir,
  readFile,
  readlink,
} from 'node:fs/promises';
import {
  createServer,
  type Server,
} from 'node:net';

import { log, } from './syslog.ts';

/**
 Abstract Unix socket name used for single-instance enforcement.
 */
const SOCKET_NAME = '\0hall-monitor';

/**
 Index of the inode field in /proc/net/unix output lines.
 */
const INODE_FIELD_INDEX = 6;

/**
 Milliseconds to wait between lock acquisition retries.
 */
const RETRY_DELAY_MS = 300;

/**
 Maximum SIGTERM retry attempts before escalating to SIGKILL.
 */
const SIGTERM_RETRIES = 10;

/**
 Maximum SIGKILL retry attempts before giving up.
 */
const SIGKILL_RETRIES = 7;

/**
 Module-singleton mutable state for the lock-server handle; wrapped so it satisfies no-module-root-let.
 */
const state: { lockServer?: Server; } = {};

/**
 Closes the held lock socket, if any.
 Used during shutdown; a no-op when no lock is currently held.
 
 @example
 ```ts
 closeLock();
 ```
 */
export function closeLock(): void {
  /**
   Current lock-server handle, if any; closed to release the held abstract socket.
   */
  const { lockServer, } = state;
  if (lockServer)
    lockServer.close();
}

/**
 Attempts to acquire the single-instance lock by listening on an abstract Unix socket.
 Returns `true` if the lock was acquired, `false` if another instance holds it.
 
 @returns whether the lock was successfully acquired
 
 @example
 ```ts
 if (!(await acquireLock())) {
   log.error("Another instance is already running.");
 }
 ```
 */
export function acquireLock(): Promise<boolean> {
  // oxlint-disable-next-line promise/avoid-new -- wrapping Node.js callback-based Server API
  return new Promise(function tryListen(resolve,) {
    /**
     Local handle to the freshly created server so listeners can be attached before assigning into `state`.
     */
    const server = createServer();
    state.lockServer = server;
    server.on(
      'error',
      function handleSocketError(err: Readonly<NodeJS.ErrnoException>,) {
        if (err.code
          === 'EADDRINUSE')
          resolve(false,);
        else {
          log.error(`[lock] Socket error: ${err.message}`,);
          resolve(false,);
        }
      },
    );
    server.listen(
      SOCKET_NAME,
      function onListening() {
        resolve(true,);
      },
    );
  },);
}

/**
 Returns true when every character in `s` is an ASCII digit `0`-`9`.
 Empty string returns false so PID-shape gates do not accept it.
 
 @param s - candidate string
 
 @returns whether `s` is one or more digits
 */
function isAllDigits(s: string,): boolean {
  if (s.length
    === 0)
    return false;
  for (const c of s) {
    if ((c < '0') || (c > '9'))
      return false;
  }
  return true;
}

/**
 Splits `s` on runs of horizontal whitespace, dropping empty leading or
 trailing tokens; mirrors `.split(/\s+/)` semantics for a trimmed input.
 
 Single linear pass: a cursor records each token's start index and slices
 once at the closing whitespace (or end of input). Build is O(n) time and
 O(1) stack, allocating only the output array plus one slice per token,
 versus the prior recursive walker which was O(n) stack and O(n^2) time
 from per-token `[...acc, token]` array spreads. Exported for unit tests.
 
 @param s - text to split
 
 @returns array of non-empty whitespace-separated tokens
 
 @example
 ```ts
 splitOnWhitespace('  a  b\tc '); // ['a', 'b', 'c']
 ```
 */
export function splitOnWhitespace(s: string,): string[] {
  return (function build(): string[] {
    /**
     Completed tokens in order; one push and one slice per token keeps the build O(n).
     */
    const tokens: string[] = [];
    /**
     Start index of the in-progress token, or `(-1)` while between tokens.
     */
    let tokenStart = -1;
    /**
     Scan cursor advanced one character at a time across `s`.
     */
    let idx = 0;
    while (idx < s
      .length) {
      /**
       Current char; whitespace closes the in-progress token.
       */
      const c = s.charAt(idx,);
      if (
        (c === ' ')
          || (c === '\t')
          || (c === '\n')
          || (c === '\r')
          || (c === '\f')
          || (c === '\v')
      ) {
        if (tokenStart !== (-1)) {
          tokens.push(s.slice(
            tokenStart,
            idx,
          ),);
          tokenStart = -1;
        }
      }
      else if (tokenStart === (-1))
        tokenStart = idx;
      idx += 1;
    }
    if (tokenStart !== (-1))
      tokens.push(s.slice(tokenStart,),);
    return tokens;
  })();
}

/**
 Finds the PID of the process holding the hall-monitor abstract socket
 by cross-referencing `/proc/net/unix` inodes with `/proc/{pid}/fd` symlinks.
 
 @returns PID of the socket owner
 
 @throws when no process is found holding the hall-monitor socket
 */
async function findSocketOwnerPid(): Promise<number> {
  /**
   Snapshot of `/proc/net/unix` listing every Unix socket on the system.
   */
  const unix = await readFile(
    '/proc/net/unix',
    'utf8',
  );
  /**
   First `/proc/net/unix` row whose path matches the hall-monitor abstract socket.
   */
  const line = unix.split('\n',)
    .find(function matchHallMonitor(l,) {
      return l.includes('@hall-monitor',);
    },);
  if (line === undefined)
    throw new Error('Socket in use but could not find owner PID.',);
  /**
   Whitespace-separated `/proc/net/unix` columns used to extract the inode.
   */
  const fields = splitOnWhitespace(line.trim(),);
  /**
   Inode number that links the socket row to a `/proc/{pid}/fd` symlink target.
   */
  const inode = fields[INODE_FIELD_INDEX];

  for (const pid of await readdir('/proc',)) {
    if (!isAllDigits(pid,))
      continue;
    try {
      /**
       Open file descriptors of the candidate process; scanned for a matching socket inode.
       */
      // oxlint-disable-next-line no-await-in-loop -- sequential /proc traversal; parallel reads would race with process exits
      const fds = await readdir(`/proc/${pid}/fd`,);
      for (const fd of fds) {
        /**
         Resolved fd symlink target; equals `socket:[<inode>]` for socket descriptors.
         */
        // oxlint-disable-next-line no-await-in-loop -- sequential readlink for each fd
        const link = await readlink(`/proc/${pid}/fd/${fd}`,);
        if (link === `socket:[${inode}]`) {
          return Math.trunc(Number(pid,));
        }
      }
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      // permission denied or process exited between readdir and readlink
    }
  }
  throw new Error('Socket in use but could not find owner PID.',);
}

/**
 Kills the existing hall-monitor instance, found via
 {@link findSocketOwnerPid}, and acquires the lock via {@link acquireLock}.
 Sends SIGTERM first, escalates to SIGKILL if the process does not exit
 within the retry span.
 
 @throws when the lock cannot be acquired after killing the existing instance
 
 @example
 ```ts
 await killExisting();
 // lock is now held by this process
 ```
 */
export async function killExisting(): Promise<void> {
  /**
   PID of the existing hall-monitor instance that must be terminated before the lock is free.
   */
  const pid = await findSocketOwnerPid();

  log.debug(`[lock] Sending SIGTERM to existing instance (PID ${pid})...`,);
  process.kill(
    pid,
    'SIGTERM',
  );

  for (let loopIndex = 0; loopIndex < SIGTERM_RETRIES; loopIndex++) {
    // oxlint-disable-next-line no-await-in-loop, promise/avoid-new -- sequential retry loop with delay
    await new Promise(function retryDelay(resolve,) {
      setTimeout(
        resolve,
        RETRY_DELAY_MS,
      );
    },);
    // oxlint-disable-next-line no-await-in-loop -- sequential retry loop
    if (await acquireLock())
      return;
  }

  log.debug(`[lock] Sending SIGKILL to PID ${pid}...`,);
  try {
    process.kill(
      pid,
      'SIGKILL',
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // process may already be gone
  }

  for (let loopIndex = 0; loopIndex < SIGKILL_RETRIES; loopIndex++) {
    // oxlint-disable-next-line no-await-in-loop, promise/avoid-new -- sequential retry loop with delay
    await new Promise(function retryDelay(resolve,) {
      setTimeout(
        resolve,
        RETRY_DELAY_MS,
      );
    },);
    // oxlint-disable-next-line no-await-in-loop -- sequential retry loop
    if (await acquireLock())
      return;
  }

  throw new Error('Failed to take over lock after killing existing instance.',);
}
