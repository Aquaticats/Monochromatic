/**
 Running a candidate `mpm-shadow`, downloading the lock's archive, checking it and unpacking it.

 Imports only `node:` modules, like the rest of the `mpm-shadow` bootstrap, so that it runs from source on a checkout that
 has no `node_modules` yet (see `mpm-shadow-lock-logic.ts`).

 @module
 */

import { spawn, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import { once, } from 'node:events';
import {
  chmod,
  mkdir,
  rename,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  type ArchiveRow,
  BOOTSTRAP_CACHE_DIRECTORY,
  MpmShadowLockError,
  parseVersionOutput,
} from './mpm-shadow-lock-read.ts';

//region Running a candidate: which version is it

/**
 What `<binary> --version` told.

 @example
 ```ts
 const answer: VersionAnswer = { usable: true, version: '0.2.1', };
 ```
 */
export type VersionAnswer = {
  readonly usable: true;
  readonly version: string
} | {
  readonly usable: false;
  readonly reason: string;
};

/**
 Runs `<binary> --version` and reports the version, or why the file is not a usable `mpm-shadow`.

 @param binary - Path of the candidate, followed through links

 @returns The version it prints, or the reason it printed none (a missing file, a broken link, a non-zero exit)

 @example
 ```ts
 const answer = await versionOf(join(root, TOOLCHAIN_BIN_LINK,),);
 ```
 */
export async function versionOf(binary: string,): Promise<VersionAnswer> {
  /**
   The candidate, started with its output piped.
   */
  const child = spawn(
    binary,
    ['--version',],
    { stdio: [
      'ignore',
      'pipe',
      'pipe',
    ], },
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
 File mode of an unpacked program: read, write and execute for the owner, read and execute for others.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Downloads a URL into memory.

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
 Unpacks the `mpm-shadow` member of a release archive (one top folder, then `mpm-shadow`) into a directory.

 @param archive - Path of the `.tar.gz`

 @param directory - Existing directory that receives `mpm-shadow`

 @throws MpmShadowLockError when `tar` cannot start or exits non-zero

 @example
 ```ts
 await extractMpmShadow({ archive: '/x/a.tar.gz', directory: '/x/out', },);
 ```
 */
export async function extractMpmShadow(
  {
    archive,
    directory,
  }: {
    readonly archive: string;
    readonly directory: string;
  },
): Promise<void> {
  /**
   The same `tar` call as the repository `Dockerfile` and the README bootstrap use.
   */
  const child = spawn(
    'tar',
    [
      '--extract',
      '--gzip',
      '--file',
      archive,
      '--wildcards',
      '--strip-components=1',
      '--directory',
      directory,
      '*/mpm-shadow',
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
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT'))
      return false;
    throw error;
  }
}

/**
 Fetches the lock's archive into the bootstrap cache.

 The archive is unpacked into a folder of its own and moved into place with one rename, so a task that runs at the same
 time never sees a half-written copy; the copy must print the locked version before it is kept.

 @param root - Repository root

 @param version - The version the lock names, which the unpacked program must print

 @param row - The archive row to fetch

 @param download - Fetches an archive's bytes

 @param log - Prints one note for the person running the task

 @returns Absolute path of the new copy

 @throws MpmShadowLockError when the download fails, the SHA-256 differs from the lock's, or the copy prints another version

 @example
 ```ts
 const binary = await fetchLockedMpmShadow({ root, version: '0.2.1', row, download, log, },);
 ```
 */
export async function fetchLockedMpmShadow(
  {
    root,
    version,
    row,
    download,
    log,
  }: {
    readonly root: string;
    readonly version: string;
    readonly row: ArchiveRow;
    readonly download: (url: string,) => Promise<Uint8Array>;
    readonly log: (line: string,) => void;
  },
): Promise<string> {
  /**
   The final folder of this version.
   */
  const final = join(
    root,
    BOOTSTRAP_CACHE_DIRECTORY,
    version,
  );
  /**
   A folder of this process alone, renamed to the final one when the copy is checked.
   */
  const partial = `${final}.partial-${String(process.pid,)}`;
  /**
   The downloaded bytes.
   */
  const bytes = await download(row.url,);
  /**
   Their digest.
   */
  const actual = sha256Hex(bytes,);
  if (actual !== row.sha256) {
    throw new MpmShadowLockError(
      `the archive ${row.url} has SHA-256 ${actual}, but toolchain.lock.json records ${row.sha256}; nothing was installed. `
        + 'A changed archive under the same version is refused; run `mpm-shadow update mpm-shadow` to record the new one after checking where it came from.',
    );
  }
  await rm(
    partial,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    partial,
    { recursive: true, },
  );
  await writeFile(
    join(
      partial,
      'archive.tar.gz',
    ),
    bytes,
  );
  await extractMpmShadow({
    archive: join(
      partial,
      'archive.tar.gz',
    ),
    directory: partial,
  },);
  await rm(
    join(
      partial,
      'archive.tar.gz',
    ),
    { force: true, },
  );
  /**
   The unpacked program inside the partial folder.
   */
  const candidate = join(
    partial,
    'mpm-shadow',
  );
  await chmod(
    candidate,
    EXECUTABLE_MODE,
  );
  /**
   What the unpacked program says it is.
   */
  const answer = await versionOf(candidate,);
  if ((!answer.usable) || (answer.version !== version)) {
    await rm(
      partial,
      {
        recursive: true,
        force: true,
      },
    );
    throw new MpmShadowLockError(
      `the archive ${row.url} unpacked to a program that ${
        answer.usable ? `prints version ${answer.version}` : `does not run (${answer.reason})`
      }, but toolchain.lock.json names ${version}`,
    );
  }
  await mkdir(
    dirname(final,),
    { recursive: true, },
  );
  try {
    await rename(
      partial,
      final,
    );
  }
  catch (error) {
    // Another task finished first and its folder is now `final`; that copy was checked by the task that put it there.
    await rm(
      partial,
      {
        recursive: true,
        force: true,
      },
    );
    if (!(await pathExists(join(
      final,
      'mpm-shadow',
    ),)))
      throw error;
    log(`another task installed mpm-shadow ${version} first (${String(error,)}); using its copy`,);
  }
  return join(
    final,
    'mpm-shadow',
  );
}

//endregion Fetching: download, check, unpack
