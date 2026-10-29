/**
 Process and artifact boundary helpers: subprocess runs, SHA-256 checked
 downloads, archive extraction, and atomic links.
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { createHash, } from 'node:crypto';
import { createWriteStream, } from 'node:fs';
import {
  chmod,
  link,
  mkdir,
  readFile,
  rename,
  rm,
  symlink,
  unlink,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import { pipeline, } from 'node:stream/promises';
import spawn, { SubprocessError, } from 'nano-spawn';
import { l, } from '../logger.ts';

//region Subprocess

/**
 Result of one subprocess run.
 */
export type RunResult = {
  readonly stdout: string;
  readonly stderr: string;
  readonly code: number;
};

/**
 Exit code reported when a failure carries no numeric exit code of its own
 (a command that could not start, or one killed by a signal).
 */
const GENERIC_FAILURE_EXIT_CODE = 1;

/**
 Runs one command, capturing output and exit code without throwing on
 non-zero exits.

 @param command - Executable to start

 @param args - Arguments passed to the executable

 @param env - Environment of the subprocess; absent means the parent's

 @param cwd - Working directory of the subprocess; absent means the parent's

 @returns Captured result

 @example
 ```ts
 const result = await run({ command: 'git', args: ['--version',], },);
 ```
 */
export async function run(
  {
    command,
    args,
    env,
    cwd,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly env?: Readonly<Record<string, string>>;
    readonly cwd?: string;
  },
): Promise<RunResult> {
  try {
    /**
     Successful spawn result.
     */
    const result = await spawn(
      command,
      [...args],
      {
        ...(env === undefined
          ? {}
          : { env }),
        ...(cwd === undefined
          ? {}
          : { cwd }),
      },
    );
    return {
      stdout: result.stdout,
      stderr: result.stderr,
      code: 0,
    };
  } catch (error) {
    if (error instanceof SubprocessError) {
      return {
        stdout: error.stdout,
        stderr: error.stderr,
        code: error.exitCode ?? GENERIC_FAILURE_EXIT_CODE,
      };
    }
    return {
      stdout: '',
      stderr: caughtValueText(error,),
      code: GENERIC_FAILURE_EXIT_CODE,
    };
  }
}

//endregion Subprocess

//region Downloads and extraction

/**
 Computes the SHA-256 hex digest of one file.

 @param path - Absolute file path

 @returns Lowercase hex digest

 @example
 ```ts
 const digest = await sha256File('/tmp/archive.tar.gz',);
 ```
 */
export async function sha256File(path: string,): Promise<string> {
  return createHash('sha256',)
    .update(await readFile(path,),)
    .digest('hex',);
}

/**
 Downloads one URL to a target file, verifying its SHA-256 before the file
 is left in place.

 @param url - Address to download

 @param target - Absolute path the verified file is moved to

 @param sha256 - Expected lowercase hex digest of the downloaded bytes

 @returns The verified target path

 @throws Error when the download fails or the digest does not match

 @example
 ```ts
 await downloadVerified({ url: 'https://example.test/tool.tar.gz', target: '/tmp/tool.tar.gz', sha256: 'abc123', },);
 ```
 */
export async function downloadVerified(
  {
    url,
    target,
    sha256,
  }: {
    readonly url: string;
    readonly target: string;
    readonly sha256: string;
  },
): Promise<string> {
  await mkdir(
    dirname(target,),
    { recursive: true },
  );
  /**
   Temporary download path beside the target.
   */
  const temp = `${target}.download`;
  /**
   Fetched response body.
   */
  const response = await fetch(url,);
  if ((!response.ok) || (response.body === null)) {
    throw new Error(`download failed for ${url}: HTTP ${response.status}`,);
  }
  await pipeline(
    response.body,
    createWriteStream(temp,),
  );
  /**
   Observed digest of the downloaded bytes.
   */
  const observed = await sha256File(temp,);
  if (observed !== sha256) {
    await rm(
      temp,
      { force: true },
    );
    throw new Error(
      `SHA-256 mismatch for ${url}: expected ${sha256}, observed ${observed}`,
    );
  }
  await rename(
    temp,
    target,
  );
  return target;
}

/**
 Extracts one archive into a directory, using `tar` for tar families and
 `unzip` for zip payloads.

 @param archive - Absolute path of the archive to extract

 @param into - Destination directory, created when absent

 @returns The destination directory

 @throws Error when neither extractor succeeds

 @example
 ```ts
 await extractArchive({ archive: '/tmp/tool.tar.gz', into: '/tmp/tool', },);
 ```
 */
export async function extractArchive(
  {
    archive,
    into,
  }: {
    readonly archive: string;
    readonly into: string;
  },
): Promise<string> {
  await mkdir(
    into,
    { recursive: true },
  );
  if (archive.endsWith('.zip',)) {
    /**
     `unzip` result for zip payloads.
     */
    const unzipped = await run({
      command: 'unzip',
      args: [
        '-q',
        '-o',
        archive,
        '-d',
        into,
      ],
    },);
    if (unzipped.code !== 0)
      throw new Error(`unzip failed for ${archive}: ${unzipped.stderr}`,);
    return into;
  }
  /**
   `tar` result for tar families.
   */
  const untarred = await run({
    command: 'tar',
    args: [
      '--extract',
      '--file',
      archive,
      '--directory',
      into,
    ],
  },);
  if (untarred.code !== 0)
    throw new Error(`tar failed for ${archive}: ${untarred.stderr}`,);
  return into;
}

//endregion Downloads and extraction

//region Atomic links

/**
 Creates or replaces one symlink atomically:
 a temporary name plus a rename over the final name, so a concurrent reader
 never sees a missing file (the `cli-git-bin` postinstall pattern).

 @param linkPath - Final link location

 @param target - Path the link resolves to

 @returns The link path

 @example
 ```ts
 await atomicSymlink({ linkPath: '/tmp/bin/git', target: '/tmp/tools/git', },);
 ```
 */
export async function atomicSymlink(
  {
    linkPath,
    target,
  }: {
    readonly linkPath: string;
    readonly target: string;
  },
): Promise<string> {
  /**
   Logger tagged for this link replacement.
   */
  const log = tagged({
    tag: atomicSymlink.name,
    l,
  },);
  await mkdir(
    dirname(linkPath,),
    { recursive: true },
  );
  /**
   Temporary link name beside the final one.
   */
  const temp = join(
    dirname(linkPath,),
    `.${
      basename(linkPath,)
    }.${
      process.pid
    }.tmp`,
  );
  try {
    await unlink(temp,);
  } catch (error) {
    log.debug(`no stale temporary link at ${temp} to clear: ${caughtValueText(error,)}`,);
  }
  await symlinkOrCopy(
    {
      linkPath: temp,
      target,
    },
  );
  await rename(
    temp,
    linkPath,
  );
  return linkPath;
}

/**
 Creates a symlink, falling back to a hard link when symlinks are
 unavailable.

 @param linkPath - Location of the new link

 @param target - Path the link resolves to

 @example
 ```ts
 await symlinkOrCopy({ linkPath: '/tmp/bin/git', target: '/tmp/tools/git', },);
 ```
 */
export async function symlinkOrCopy(
  {
    linkPath,
    target,
  }: {
    readonly linkPath: string;
    readonly target: string;
  },
): Promise<void> {
  /**
   Logger tagged for this link creation.
   */
  const log = tagged({
    tag: symlinkOrCopy.name,
    l,
  },);
  try {
    await symlink(
      target,
      linkPath,
    );
  } catch (error) {
    log.debug(`symlink to ${target} unavailable, using a hard link: ${caughtValueText(error,)}`,);
    await link(
      target,
      linkPath,
    );
  }
}

/**
 Permission bits of an executable file: owner read, write, execute; group
 and others read and execute.
 */
const EXECUTABLE_FILE_MODE = 0o755;

/**
 Marks one file executable.

 @param path - Absolute file path

 @example
 ```ts
 await makeExecutable('/tmp/tools/git',);
 ```
 */
export async function makeExecutable(path: string,): Promise<void> {
  await chmod(
    path,
    EXECUTABLE_FILE_MODE,
  );
}

//endregion Atomic links
