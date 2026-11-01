#!/usr/bin/env node

/**
 Runs `mpm-shadow` at the version `toolchain.lock.json` names, whatever `mpm-shadow` the machine's `PATH` offers.

 Every root `toolchain:*` task of `mise.toml` runs through this entry. It finds the repository root (the folder with
 `toolchain.lock.json`, searched upward from the current directory), finds or fetches the locked copy
 (see {@link resolveLockedMpmShadow}), and runs it with the arguments it was given, passing its exit status on.
 Notes about which copy runs go to standard error, so standard output stays the program's own (`bin --json` is read
 by other programs).

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

if (import.meta.main) {
  /**
   Where the lock and the cache are.
   */
  const root = await findRoot(process.cwd(),);
  /**
   The copy to run.
   */
  const copy = await resolveLockedMpmShadow({
    root,
    platformId: await hostLockPlatformId(),
    download: downloadBytes,
    log(line,) {
      console.error(`[toolchain] ${line}`,);
    },
  },);
  /**
   The program's exit status.
   */
  const status = await runInherited({
    binary: copy.binary,
    args: process.argv
      .slice(2,),
  },);
  // The task's status is the program's own, so a non-zero status other than 1 is passed on unchanged.
  if (status !== 0)
    process.exitCode = status;
}
