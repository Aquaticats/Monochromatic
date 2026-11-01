/**
 Running a candidate `mpm-shadow` for its version, downloading an archive, checking its SHA-256 and unpacking it.

 Imports only `node:` modules, like the rest of the `mpm-shadow` bootstrap, so that it runs from source on a checkout that
 has no `node_modules` yet (see `mpm-shadow-lock-logic.ts`).

 @module
 */

import { spawn, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import { once, } from 'node:events';
import { stat, } from 'node:fs/promises';

import {
  MpmShadowLockError,
  parseVersionOutput,
} from './mpm-shadow-lock-read.ts';

//region Running a candidate: which version is it

/**
 How long `<binary> --version` may take before the candidate counts as not working, in milliseconds. A program that prints its
 version at once needs a few milliseconds; a link to a file on a stalled mount or a program waiting on input needs more than
 this and would otherwise stop every `toolchain:*` task.
 */
export const VERSION_TIMEOUT_MS = 10_000;

/**
 Milliseconds in a second, for the note that names a timeout.
 */
const MS_PER_SECOND = 1_000;

/**
 What `<binary> --version` told.

 @example
 ```ts
 const answer: VersionAnswer = { usable: true, version: '0.2.1', };
 ```
 */
export type VersionAnswer = {
  readonly usable: true;
  readonly version: string;
} | {
  readonly usable: false;
  readonly reason: string;
};

/**
 Runs `<binary> --version` and reports the version, or why the file is not a usable `mpm-shadow`.

 @param binary - Path of the candidate, followed through links

 @param timeoutMs - How long to wait for the answer before ending the candidate

 @returns The version it prints, or the reason it printed none (a missing file, a broken link, a non-zero exit, no answer in time)

 @example
 ```ts
 const answer = await versionOf({ binary: join(root, TOOLCHAIN_BIN_LINK,), timeoutMs: VERSION_TIMEOUT_MS, },);
 ```
 */
export async function versionOf(
  {
    binary,
    timeoutMs,
  }: {
    readonly binary: string;
    readonly timeoutMs: number;
  },
): Promise<VersionAnswer> {
  /**
   The candidate, started with its output piped and ended after the timeout.
   */
  const child = spawn(
    binary,
    ['--version',],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      timeout: timeoutMs,
    },
  );
  /**
   Pieces of standard output.
   */
  const stdout: string[] = [];
  /**
   Pieces of standard error.
   */
  const stderr: string[] = [];
  child.stdout
    .on(
    'data',
    function collectStdout(chunk: unknown,) {
      stdout.push(String(chunk,),);
    },
    );
  child.stderr
    .on(
    'data',
    function collectStderr(chunk: unknown,) {
      stderr.push(String(chunk,),);
    },
    );
  try {
    // `once` rejects when the process could not start (a missing file, a broken link, no execute permission).
    await once(
      child,
      'close',
    );
  }
  catch (error) {
    return {
      usable: false,
      reason: String(error,),
    };
  }
  if (child.signalCode === 'SIGTERM')
    return {
      usable: false,
      reason: `\`--version\` gave no answer within ${String(timeoutMs / MS_PER_SECOND,)} seconds`,
    };
  /**
   The version in the output, when there is one.
   */
  const version = parseVersionOutput(stdout.join('',),);
  if ((child.exitCode === 0) && (version !== ''))
    return {
      usable: true,
      version,
    };
  return {
    usable: false,
    reason: `\`--version\` exited ${String(child.exitCode,)} and printed "${stdout.join('',)
      .trim()}${stderr.join('',)
        .trim()}"`,
  };
}

//endregion Running a candidate: which version is it

//region Fetching: download, check, unpack

/**
 How long a download may take before it is given up, in milliseconds.
 */
const DOWNLOAD_TIMEOUT_MS = 300_000;

/**
 Downloads a URL into memory.

 `fetch` in Node does not read `HTTP_PROXY` and `HTTPS_PROXY` unless `NODE_USE_ENV_PROXY=1` is set (measured with Node
 26.13.0: with it a request goes to the proxy address, without it straight to the host), so a machine behind a proxy sets
 that variable for the task.

 @param url - Address of the archive

 @returns The bytes

 @throws MpmShadowLockError when the request fails or answers a non-2xx status

 @example
 ```ts
 const bytes = await downloadBytes('https://example.invalid/a.tar.gz',);
 ```
 */
export async function downloadBytes(url: string,): Promise<Uint8Array> {
  try {
    /**
     The response; redirects are followed because release assets are served from another host.
     */
    const response = await fetch(
      url,
      {
        redirect: 'follow',
        signal: AbortSignal.timeout(DOWNLOAD_TIMEOUT_MS,),
      },
    );
    if (!response.ok)
      throw new MpmShadowLockError(`downloading ${url} answered HTTP ${String(response.status,)}`,);
    return new Uint8Array(await response.arrayBuffer(),);
  }
  catch (error) {
    if (error instanceof MpmShadowLockError)
      throw error;
    throw new MpmShadowLockError(
      `downloading ${url} failed (${String(error,)})`,
      { cause: error, },
    );
  }
}

/**
 Unpacks a release archive into a directory with `tar`, whole and without stripping a folder, as `mpm-shadow install` does
 for an archive-route tool (`install_support.rs`, `extract_archive`).

 @param archive - Path of the `.tar.gz`

 @param directory - Existing directory that receives the contents

 @throws MpmShadowLockError when `tar` cannot start or exits non-zero

 @example
 ```ts
 await extractArchive({ archive: '/x/a.tar.gz', directory: '/x/out', },);
 ```
 */
export async function extractArchive(
  {
    archive,
    directory,
  }: {
    readonly archive: string;
    readonly directory: string;
  },
): Promise<void> {
  /**
   The `tar` run.
   */
  const child = spawn(
    'tar',
    [
      '--extract',
      '--file',
      archive,
      '--directory',
      directory,
    ],
    { stdio: [
      'ignore',
      'ignore',
      'pipe',
    ], },
  );
  /**
   Pieces of what tar said on standard error.
   */
  const stderr: string[] = [];
  child.stderr
    .on(
    'data',
    function collectStderr(chunk: unknown,) {
      stderr.push(String(chunk,),);
    },
    );
  try {
    await once(
      child,
      'close',
    );
  }
  catch (error) {
    throw new MpmShadowLockError(
      `could not run tar to unpack ${archive} (${String(error,)})`,
      { cause: error, },
    );
  }
  if (child.exitCode !== 0) {
    throw new MpmShadowLockError(
      `tar could not unpack ${archive} (exit ${String(child.exitCode,)}): ${stderr.join('',)
        .trim()}`,
    );
  }
}

/**
 Hex SHA-256 of bytes.

 @param bytes - Content to hash

 @returns Lowercase hex digest

 @example
 ```ts
 sha256Hex(new Uint8Array(),); // 'e3b0c442...'
 ```
 */
export function sha256Hex(bytes: Uint8Array,): string {
  return createHash('sha256',)
    .update(bytes,)
    .digest('hex',);
}

//endregion Fetching: download, check, unpack

//region Paths: whether something is there

/**
 Whether an error says a path does not exist.

 @param error - A caught value

 @returns True for an error whose code is `ENOENT` (nothing there) or `ENOTDIR` (a parent is not a folder)

 @example
 ```ts
 isNotFound(error,);
 ```
 */
export function isNotFound(error: unknown,): boolean {
  return Error.isError(error,) && ('code' in error)
    && ((error.code === 'ENOENT') || (error.code === 'ENOTDIR'));
}

/**
 Whether a path exists, following links.

 @param path - Path to test

 @returns True when something is there

 @throws when the check fails for a reason other than the path being absent (for example a permission problem)

 @example
 ```ts
 await pathExists('/repo/toolchain.lock.json',); // true
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch (error) {
    // Only "nothing there" means false; a permission problem is a real failure.
    if (isNotFound(error,))
      return false;
    throw error;
  }
}

//endregion Paths: whether something is there
