/**
 Process and artifact boundary helpers: subprocess runs, SHA-256 checked
 downloads, archive extraction, and atomic links.
 */

import { createHash, } from 'node:crypto';
import {
  chmodSync,
  createWriteStream,
  linkSync,
  mkdirSync,
  renameSync,
  rmSync,
  symlinkSync,
  unlinkSync,
} from 'node:fs';
import {
  readFile,
  rename,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import { pipeline, } from 'node:stream/promises';
import spawn from 'nano-spawn';

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
 Runs one command, capturing output and exit code without throwing on
 non-zero exits.
 
 @param options - Command, arguments, environment and working directory
 
 @returns Captured result
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
    /**
     Spawn error carrying the captured output and exit code.
     */
    const failure = error as {
      readonly exitCode?: number;
      readonly stdout?: string;
      readonly stderr?: string;
      readonly message?: string;
    };
    return {
      stdout: failure.stdout ?? '',
      stderr: failure.stderr ?? failure.message
        ?? String(error,),
      code: failure.exitCode ?? 1,
    };
  }
}

//endregion Subprocess

//region Downloads and extraction

/**
 Computes the SHA-256 hex digest of one file.
 
 @param path - Absolute file path
 
 @returns Lowercase hex digest
 */
export async function sha256File(path: string,): Promise<string> {
  return createHash('sha256',)
    .update(await readFile(path,),)
    .digest('hex',);
}

/**
 Downloads one URL to a target file, verifying its SHA-256 before the file
 is left in place.
 
 @param options - URL, target path, and expected hex digest
 
 @returns The verified target path
 
 @throws {Error} when the download fails or the digest does not match
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
  mkdirSync(
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
    rmSync(
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
 
 @param options - Archive path and destination directory
 
 @returns The destination directory
 
 @throws {Error} when neither extractor succeeds
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
  mkdirSync(
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
 
 @param options - Link location and link target
 
 @returns The link path
 */
export function atomicSymlink(
  {
    linkPath,
    target,
  }: {
    readonly linkPath: string;
    readonly target: string;
  },
): string {
  mkdirSync(
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
    unlinkSync(temp);
  } catch {
    // No stale temporary link to clear.
  }
  symlinkOrCopy(
    {
      linkPath: temp,
      target,
    },
  );
  renameSync(
    temp,
    linkPath,
  );
  return linkPath;
}

/**
 Creates a symlink, falling back to a hard link when symlinks are
 unavailable.
 
 @param options - Link location and link target
 */
export function symlinkOrCopy(
  {
    linkPath,
    target,
  }: {
    readonly linkPath: string;
    readonly target: string;
  },
): void {
  try {
    symlinkSync(
      target,
      linkPath,
    );
  } catch {
    linkSync(
      target,
      linkPath,
    );
  }
}

/**
 Marks one file executable.
 
 @param path - Absolute file path
 */
export function makeExecutable(path: string,): void {
  chmodSync(
    path,
    0o755,
  );
}

//endregion Atomic links
