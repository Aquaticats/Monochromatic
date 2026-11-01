#!/usr/bin/env node

/**
 Runs `mpm-shadow` at least at the version `toolchain.lock.json` names, whatever `mpm-shadow` the machine's `PATH` offers.

 Every root `toolchain:*` task of `mise.toml` runs through this entry. It finds the repository root (the folder with
 `toolchain.lock.json`, searched upward from the current directory), finds or fetches a copy at or above the lock's version
 (see {@link resolveLockedMpmShadow}), and runs it with the arguments it was given, passing its exit status on. Notes about
 which copy runs go to standard error, so standard output stays the program's own (`bin --json` is read by other programs).

 It needs Node 24 or newer (`import.meta.main`, `Error.isError`) with unflagged TypeScript stripping when it runs from source;
 the Node that `toolchain.lock.json` names is newer than that. A failure the entry itself reports (an unreadable lock, a
 download or checksum failure, a copy that does not run) prints one short line and exits 1, without a stack trace.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/mpm-shadow.ts install --lockfile-only --frozen-lockfile
 ```
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  dirname,
  join,
} from 'node:path';

import {
  downloadBytes,
  pathExists,
} from './mpm-shadow-lock-fetch.ts';
import { resolveLockedMpmShadow, } from './mpm-shadow-lock-logic.ts';
import {
  hostLockPlatformId,
  MpmShadowLockError,
  toolchainRootOf,
  toolchainRootsOf,
} from './mpm-shadow-lock-read.ts';

/**
 The repository root: the nearest folder at or above a directory that holds `toolchain.lock.json`.

 @param directory - Directory to look in first

 @returns Absolute path of the folder

 @throws MpmShadowLockError when no folder up to the file system root holds the lock

 @example
 ```ts
 const root = await findRoot(process.cwd(),);
 ```
 */
async function findRoot(directory: string,): Promise<string> {
  if (await pathExists(join(
    directory,
    'toolchain.lock.json',
  ),))
    return directory;
  /**
   The folder above it.
   */
  const parent = dirname(directory,);
  if (parent === directory)
    throw new MpmShadowLockError('no toolchain.lock.json at or above the current directory',);
  return await findRoot(parent,);
}

/**
 Runs a program with the terminal's input and output and resolves with its exit status.

 @param binary - Program to run

 @param args - Its arguments

 @returns The exit status; 1 when it ended on a signal

 @throws MpmShadowLockError when the program cannot be started

 @example
 ```ts
 const status = await runInherited({ binary: '/repo/.cache/toolchain-bin/mpm-shadow', args: ['list',], },);
 ```
 */
async function runInherited(
  {
    binary,
    args,
  }: {
    readonly binary: string;
    readonly args: readonly string[];
  },
): Promise<number> {
  /**
   The program, started with the terminal's streams.
   */
  const child = spawn(
    binary,
    [...args,],
    { stdio: 'inherit', },
  );
  try {
    // `once` rejects when the program could not start (a missing file, no execute permission).
    await once(
      child,
      'close',
    );
  }
  catch (error) {
    throw new MpmShadowLockError(
      `could not run ${binary} (${String(error,)})`,
      { cause: error, },
    );
  }
  if (child.signalCode !== null)
    console.error(`[toolchain] ${binary} ended on ${child.signalCode}`,);
  return child.exitCode ?? 1;
}

/**
 Resolves the copy to run, runs it, and returns its exit status.

 @param args - Arguments for the program

 @returns The program's exit status

 @throws MpmShadowLockError for any failure the entry reports itself

 @example
 ```ts
 const status = await main(['install',],);
 ```
 */
async function main(args: readonly string[],): Promise<number> {
  /**
   The copy to run.
   */
  const copy = await resolveLockedMpmShadow({
    root: await findRoot(process.cwd(),),
    toolchainRoot: toolchainRootOf(process.env,),
    scannedRoots: toolchainRootsOf(process.env,),
    platformId: await hostLockPlatformId(),
    download: downloadBytes,
    log(line,) {
      console.error(`[toolchain] ${line}`,);
    },
  },);
  return await runInherited({
    binary: copy.binary,
    args,
  },);
}

if (import.meta.main) {
  try {
    /**
     The program's exit status.
     */
    const status = await main(process.argv
      .slice(2,),);
    // The task's status is the program's own, so a non-zero status other than 1 is passed on unchanged.
    if (status !== 0)
      process.exitCode = status;
  }
  catch (error) {
    // A failure this entry reports itself is one line, not a stack; anything else is a defect and keeps its stack.
    if (!(error instanceof MpmShadowLockError))
      throw error;
    console.error(`[toolchain] ${error.message}`,);
    process.exitCode = 1;
  }
}
