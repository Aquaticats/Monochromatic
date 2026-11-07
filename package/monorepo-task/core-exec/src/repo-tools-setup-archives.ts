/**
 Fetching one tool's archive for the repository tools action, checking it and unpacking its program.

 The archive of `mpm-shadow` is the row of the `mpm-shadow` tool in `toolchain.lock.json`, checked against the SHA-256
 recorded there; the archives of `file-enforcer` and `cargo-shadow` are release assets of the version the lock names,
 checked against the checksums file of that release. `curl`, `sha256sum` and `tar` run as they did in the shell step, so
 their output and statuses are unchanged. Imports only `node:` modules and sibling files that do, because the action runs on
 a runner that has no dependencies installed.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  jsonAt,
  rawJsonTextOrEmpty,
  requiredEnvironment,
  runProgram,
  runProgramOrEnd,
  sayOnStandardError,
  StepExitError,
} from './github-actions-step-support.ts';

//region The names a runner platform has

/**
 The names a runner platform has in the lock and in release archives.
 */
export type PlatformNames = {
  /**
   Platform id of the artifact rows in `toolchain.lock.json`.
   */
  readonly lockPlatform: string;
  /**
   Rust target triple in the release archive names.
   */
  readonly target: string;
};

/**
 The programs the action installs.
 */
export const KNOWN_TOOLS: ReadonlySet<string> = new Set([
  'mpm-shadow',
  'file-enforcer',
  'cargo-shadow',
  'git-policy-cli',
]);

/**
 The program a tool's archive holds, for the tools whose program is not named like the tool.

 @param tool - A name from {@link KNOWN_TOOLS}

 @returns The name of the program inside the archive, which is also its name on `PATH`

 @example
 ```ts
 const program = programOf('git-policy-cli',); // 'monochromatic-git-policy-cli'
 ```
 */
export function programOf(tool: string,): string {
  return tool === 'git-policy-cli' ? 'monochromatic-git-policy-cli' : tool;
}

//endregion The names a runner platform has

//region Fetching one archive

/**
 The download flags every fetch uses: fail on an HTTP error, print nothing but errors, follow redirects.
 */
const CURL_FLAGS = [
  '--fail',
  '--silent',
  '--show-error',
  '--location',
] as const;

/**
 Downloads a file with `curl`.

 @param url - Address to fetch

 @param output - File to write

 @throws {@link StepExitError} With curl's status when the download fails

 @example
 ```ts
 download({ url: 'https://example.invalid/a.tar.gz', output: '/path/a.tar.gz', },);
 ```
 */
async function download(
  {
    url,
    output,
  }: {
    readonly url: string;
    readonly output: string;
  },
): Promise<void> {
  await runProgramOrEnd({
    file: 'curl',
    args: [
      ...CURL_FLAGS,
      '--output',
      output,
      url,
    ],
    cwd: process.cwd(),
  },);
}

/**
 Checks lines of `<sha256>  <file>` with `sha256sum --check`, which prints one `<file>: OK` line per file.

 @param lines - Lines to check; none gives `sha256sum`'s own "no properly formatted checksum lines" failure

 @param cwd - Directory the file names in the lines are relative to

 @throws {@link StepExitError} With status 1 when `sha256sum` rejects a line

 @example
 ```ts
 checkSha256({ lines: [`${sum}  ${archive}`,], },);
 ```
 */
async function checkSha256(
  {
    lines,
    cwd,
  }: {
    readonly lines: readonly string[];
    readonly cwd: string;
  },
): Promise<void> {
  await runProgramOrEnd({
    file: 'sha256sum',
    args: ['--check',],
    input: lines.map(function lineWithBreak(line: string,) {
      return `${line}\n`;
    },)
      .join('',),
    cwd,
  },);
}

/**
 Fetches the release archive of a crate version and checks it against the checksums file of that release.

 @param lock - Parsed `toolchain.lock.json`

 @param crate - Crate name, whose lock row is `cargo:<crate>`

 @param work - Empty folder to download into

 @param target - Rust target triple of this runner

 @returns Path of the checked archive

 @throws {@link StepExitError} With status 1 when the lock has no row for the crate, or with the status of the failing `curl`
   or `sha256sum`

 @example
 ```ts
 const archive = fetchReleaseArchive({ lock, crate: 'monochromatic-file-enforcer-cli', work, target, },);
 ```
 */
async function fetchReleaseArchive(
  {
    lock,
    crate,
    work,
    target,
  }: {
    readonly lock: unknown;
    readonly crate: string;
    readonly work: string;
    readonly target: string;
  },
): Promise<string> {
  /**
   Version the lock records for the crate.
   */
  const version = rawJsonTextOrEmpty(jsonAt({
    value: lock,
    keys: [
      'tools',
      `cargo:${crate}`,
      'version',
    ],
  },),);
  if (version === '') {
    sayOnStandardError(
      `::error::repo-tools-setup: toolchain.lock.json has no cargo:${crate} row; run 'mpm-shadow update'`,
    );
    throw new StepExitError(1,);
  }
  /**
   File name of the archive in the release.
   */
  const name = `${crate}-${version}-${target}.tar.gz`;
  /**
   File name of the release's checksums file.
   */
  const sums = `${crate}-${version}-sha256sums.txt`;
  /**
   Where the release's files are.
   */
  const base = `https://github.com/${requiredEnvironment('GITHUB_REPOSITORY',)}/releases/download/${crate}-v${version}`;
  /**
   Where the archive lands.
   */
  const archive = join(
    work,
    name,
  );
  await download({
    url: `${base}/${name}`,
    output: archive,
  },);
  await download({
    url: `${base}/${sums}`,
    output: join(
      work,
      sums,
    ),
  },);
  /**
   Lines of the checksums file; the shell step read them with `grep`.
   */
  const matching = await checksumLinesFor({
    path: join(
      work,
      sums,
    ),
    name,
  },);
  await checkSha256({
    lines: matching,
    cwd: work,
  },);
  return archive;
}

/**
 Reads the lines of a checksums file that end with an archive's file name, as `grep " <name>$"` selects them.

 @param path - Checksums file

 @param name - Archive file name

 @returns The matching lines, none when no line names the archive

 @example
 ```ts
 const lines = checksumLinesFor({ path: sums, name: 'a.tar.gz', },);
 ```
 */
async function checksumLinesFor(
  {
    path,
    name,
  }: {
    readonly path: string;
    readonly name: string;
  },
): Promise<readonly string[]> {
  return (await readFile(
    path,
    'utf8',
  ))
    .split('\n',)
    .filter(function namesArchive(line: string,) {
      return line.endsWith(` ${name}`,);
    },);
}

//endregion Fetching one archive

//region Unpacking and fetching a tool

/**
 Extracts the program from a checked archive into the folder that goes on `PATH`, then prints its version.

 @param archive - Checked archive

 @param binDirectory - Folder that receives the program

 @param program - Program name, the last path segment of the archive member to extract

 @param archiveLabel - How the archive is named in a message: the tool it was fetched for

 @throws {@link StepExitError} With status 1 and a message when the archive has no such member, or with the failing `tar` or
   program's own status

 @example
 ```ts
 unpack({ archive, binDirectory, program: 'file-enforcer', archiveLabel: 'file-enforcer', },);
 ```
 */
export async function unpack(
  {
    archive,
    binDirectory,
    program,
    archiveLabel,
  }: {
    readonly archive: string;
    readonly binDirectory: string;
    readonly program: string;
    readonly archiveLabel: string;
  },
): Promise<void> {
  /**
   The archive's member names, one per line.
   */
  const listing = await runProgram({
    file: 'tar',
    args: [
      '--list',
      '--gzip',
      '--file',
      archive,
    ],
    captureStdout: true,
    cwd: process.cwd(),
  },);
  /**
   The first member whose last path segment is the program, as `grep "/<tool>$" | head --lines=1` finds it.
   */
  const member = listing.stdout
    .split('\n',)
    .find(function namesTool(line: string,) {
      return line.endsWith(`/${program}`,);
    },);
  if (member === undefined) {
    sayOnStandardError(
      `::error::repo-tools-setup: the archive of ${archiveLabel} holds no file named ${program}, so nothing was installed. Either the archive could not be read (tar says why above) or the release lays its files out differently from what this action expects; open the release named in toolchain.lock.json and check the file names in it.`,
    );
    throw new StepExitError(1,);
  }
  if (listing.status !== 0)
    throw new StepExitError(listing.status,);
  await runProgramOrEnd({
    file: 'tar',
    args: [
      '--extract',
      '--gzip',
      '--file',
      archive,
      '--strip-components=1',
      '--directory',
      binDirectory,
      member,
    ],
    cwd: process.cwd(),
  },);
  await runProgramOrEnd({
    file: join(
      binDirectory,
      program,
    ),
    args: ['--version',],
    cwd: process.cwd(),
  },);
}

/**
 Fetches the archive a lock row names for this platform, and checks it against the SHA-256 recorded beside it.

 @param lock - Parsed `toolchain.lock.json`

 @param rowId - Id of the tool's row in the lock

 @param tool - Name of the tool, for the file the archive is saved as

 @param platform - Names this runner platform has in the lock

 @param work - Empty folder to download into

 @returns Path of the checked archive

 @throws {@link StepExitError} With status 1 and a message when the row has no address or SHA-256 for the platform, or with
   the status of the failing download or check

 @example
 ```ts
 const archive = await fetchLockedArchive({ lock, rowId: 'mpm-shadow', tool: 'mpm-shadow', platform, work, },);
 ```
 */
async function fetchLockedArchive(
  {
    lock,
    rowId,
    tool,
    platform,
    work,
  }: {
    readonly lock: unknown;
    readonly rowId: string;
    readonly tool: string;
    readonly platform: PlatformNames;
    readonly work: string;
  },
): Promise<string> {
  /**
   Where the lock says to download it.
   */
  const url = rawJsonTextOrEmpty(jsonAt({
    value: lock,
    keys: [
      'tools',
      rowId,
      'artifacts',
      platform.lockPlatform,
      'url',
    ],
  },),);
  /**
   The SHA-256 the lock records for it.
   */
  const sum = rawJsonTextOrEmpty(jsonAt({
    value: lock,
    keys: [
      'tools',
      rowId,
      'artifacts',
      platform.lockPlatform,
      'sha256',
    ],
  },),);
  if ((url === '') || (sum === '')) {
    sayOnStandardError(
      `::error::repo-tools-setup: toolchain.lock.json has no ${rowId} archive row for ${platform.lockPlatform}; run 'mpm-shadow update'`,
    );
    throw new StepExitError(1,);
  }
  /**
   Where the archive lands.
   */
  const archive = join(
    work,
    `${tool}.tar.gz`,
  );
  await download({
    url,
    output: archive,
  },);
  await checkSha256({
    lines: [`${sum}  ${archive}`,],
    cwd: process.cwd(),
  },);
  return archive;
}

/**
 Fetches and checks the archive of one tool.

 @param lock - Parsed `toolchain.lock.json`

 @param tool - One of `mpm-shadow`, `file-enforcer`, `cargo-shadow`, `git-policy-cli`

 @param platform - Names this runner platform has in the lock and in archives

 @param work - Empty folder to download into

 @returns Path of the checked archive

 @throws {@link StepExitError} With status 1 for a lock without the tool's row, or with the status of the failing download or
   check

 @example
 ```ts
 const archive = fetchTool({ lock, tool: 'mpm-shadow', platform, work, },);
 ```
 */
export async function fetchTool(
  {
    lock,
    tool,
    platform,
    work,
  }: {
    readonly lock: unknown;
    readonly tool: string;
    readonly platform: PlatformNames;
    readonly work: string;
  },
): Promise<string> {
  if (tool === 'mpm-shadow')
    return await fetchLockedArchive({
      lock,
      rowId: 'mpm-shadow',
      tool,
      platform,
      work,
    },);
  if (tool === 'git-policy-cli')
    return await fetchLockedArchive({
      lock,
      rowId: 'cargo:monochromatic-git-policy-cli',
      tool,
      platform,
      work,
    },);
  if (tool === 'file-enforcer')
    return await fetchReleaseArchive({
      lock,
      crate: 'monochromatic-file-enforcer-cli',
      work,
      target: platform.target,
    },);
  if (tool === 'cargo-shadow')
    return await fetchReleaseArchive({
      lock,
      crate: 'monochromatic-cargo-shadow',
      work,
      target: platform.target,
    },);
  throw new Error(`repo-tools-setup: tool '${tool}' has no download and was not rejected as unknown`,);
}
