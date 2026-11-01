/**
 Installing the lock's `mpm-shadow` archive into the toolchain root, the way `mpm-shadow install` installs it.

 The layout is the one `mpm-shadow` keeps for an archive-route tool that this repository publishes
 (`archive_staging.rs`, `published_archive.rs`): one folder per version, `<toolchain root>/mpm-shadow/<version>/`, holding the
 unpacked archive and the completion marker written last, and a `current` link in `<toolchain root>/mpm-shadow/` that names the
 newest finished version. A copy fetched here is therefore the copy `mpm-shadow install` would have put there: there is one
 place for it, and one rule that removes the older ones ("No older repo-maintained tools should ever exist when newer is
 present"), which this module applies as well. Imports only `node:` modules, so it runs from source on a checkout without
 `node_modules`.

 @module
 */

import {
  chmod,
  mkdir,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  extractArchive,
  sha256Hex,
  VERSION_TIMEOUT_MS,
  versionOf,
} from './mpm-shadow-lock-fetch.ts';
import {
  findProgramUnder,
  removeStaleHidden,
  SEARCH_DEPTH,
} from './mpm-shadow-lock-folder.ts';
import {
  finishLayout,
  publishStaged,
} from './mpm-shadow-lock-publish.ts';
import {
  type ArchiveRow,
  checkVersionName,
  COMPLETE_MARKER,
  MPM_SHADOW_TOOL_ID,
  MpmShadowLockError,
  TOOL_DIRECTORY_NAME,
} from './mpm-shadow-lock-read.ts';

/**
 File mode of the unpacked program: read, write and execute for the owner, read and execute for others.
 */
const EXECUTABLE_MODE = 0o755;


//region Installing

/**
 Installs the lock's `mpm-shadow` archive for this platform into the toolchain root, as `mpm-shadow install` would: checks the
 SHA-256 against the lock, unpacks into a hidden staging folder, requires a program that prints the locked version, writes the
 completion marker last, renames the folder to the version's name, points `current` at the newest finished version, and removes
 the older version folders.

 @param toolchainRoot - `<data home>/monochromatic-dev/toolchain`

 @param version - The version the lock names

 @param row - The archive row to fetch

 @param download - Fetches an archive's bytes

 @param log - Prints one note for the person running the task

 @returns Absolute path of the installed program

 @throws MpmShadowLockError when the version is not a plain name, the download fails, its SHA-256 differs from the lock's, the
 archive holds no program, or the program prints another version; nothing is left at the version's name in those cases

 @example
 ```ts
 const program = await installLockedArchive({ toolchainRoot, version: '0.2.1', row, download, log, },);
 ```
 */
export async function installLockedArchive(
  {
    toolchainRoot,
    version,
    row,
    download,
    log,
  }: {
    readonly toolchainRoot: string;
    readonly version: string;
    readonly row: ArchiveRow;
    readonly download: (url: string,) => Promise<Uint8Array>;
    readonly log: (line: string,) => void;
  },
): Promise<string> {
  checkVersionName(version,);
  /**
   The tool folder, and the paths of this task's hidden staging, download and moved-aside copy.
   */
  const toolDirectory = join(
    toolchainRoot,
    TOOL_DIRECTORY_NAME,
  );
  /**
   The version folder the install ends in.
   */
  const final = join(
    toolDirectory,
    version,
  );
  /**
   This task's process id, which ends the names of its hidden files so that a killed task's can be told from a running one's.
   */
  const suffix = String(process.pid,);
  /**
   Where the archive is unpacked first.
   */
  const staging = join(
    toolDirectory,
    `.${version}.staging.${suffix}`,
  );
  /**
   Where the downloaded bytes are written for `tar`.
   */
  const archive = join(
    toolDirectory,
    `.${version}.download.${suffix}`,
  );
  /**
   Where a copy at the version's name is moved while the new one takes its place.
   */
  const aside = join(
    toolDirectory,
    `.${version}.replaced.${suffix}`,
  );
  /**
   The downloaded bytes, checked before anything is written.
   */
  const bytes = await download(row.url,);
  /**
   Their SHA-256.
   */
  const actual = sha256Hex(bytes,);
  if (actual !== row.sha256) {
    throw new MpmShadowLockError(
      `the archive ${row.url} has SHA-256 ${actual}, but toolchain.lock.json records ${row.sha256}; nothing was installed. `
        + 'A changed archive under the same version is refused; run `mpm-shadow update mpm-shadow` to record the new one after checking where it came from.',
    );
  }
  await mkdir(
    toolDirectory,
    { recursive: true, },
  );
  await removeStaleHidden({
    toolDirectory,
    log,
  },);
  await rm(
    staging,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    staging,
    { recursive: true, },
  );
  await writeFile(
    archive,
    bytes,
  );
  try {
    await extractArchive({
      archive,
      directory: staging,
    },);
  }
  catch (error) {
    await rm(
      archive,
      { force: true, },
    );
    await rm(
      staging,
      {
        recursive: true,
        force: true,
      },
    );
    throw error;
  }
  await rm(
    archive,
    { force: true, },
  );
  /**
   The unpacked program, which must print the locked version.
   */
  const unpacked = await findProgramUnder({
    directory: staging,
    depth: SEARCH_DEPTH,
  },);
  if (unpacked === '') {
    await rm(
      staging,
      {
        recursive: true,
        force: true,
      },
    );
    throw new MpmShadowLockError(`the archive ${row.url} holds no file named ${TOOL_DIRECTORY_NAME}, so nothing was installed`,);
  }
  await chmod(
    unpacked,
    EXECUTABLE_MODE,
  );
  /**
   What the unpacked program says it is.
   */
  const answer = await versionOf({
    binary: unpacked,
    timeoutMs: VERSION_TIMEOUT_MS,
  },);
  if ((!answer.usable) || (answer.version !== version)) {
    await rm(
      staging,
      {
        recursive: true,
        force: true,
      },
    );
    throw new MpmShadowLockError(
      `the archive ${row.url} unpacked to a program that ${
        answer.usable ? `prints version ${answer.version}` : `does not run (${answer.reason})`
      }, but toolchain.lock.json names ${version}; nothing was installed`,
    );
  }
  await writeFile(
    join(
      staging,
      COMPLETE_MARKER,
    ),
    `${MPM_SHADOW_TOOL_ID} ${version}\n`,
  );
  await publishStaged({
    staging,
    final,
    version,
    aside,
    move: rename,
  },);
  /**
   The installed program, checked again at its final name.
   */
  const installed = await findProgramUnder({
    directory: final,
    depth: SEARCH_DEPTH,
  },);
  /**
   What the installed program says it is.
   */
  const check = await versionOf({
    binary: installed,
    timeoutMs: VERSION_TIMEOUT_MS,
  },);
  if ((installed === '') || (!check.usable)
    || (check.version !== version)) {
    throw new MpmShadowLockError(`${final} does not hold an mpm-shadow that prints ${version} after the install; remove it and run the task again`,);
  }
  await finishLayout({
    toolDirectory,
    log,
  },);
  return installed;
}

//endregion Installing
