/**
 Finding, and when needed fetching, the `mpm-shadow` that `toolchain.lock.json` names.

 The root `toolchain:*` tasks run `mpm-shadow`, and the copy on a developer's `PATH` can be older than the lock: a plan
 field that a newer `mpm-shadow` writes (for example `tools.mpm-shadow.options.tagPrefix`) is "not a known field" to an
 older one, which exits 2 before doing anything. So these tasks do not run whatever `PATH` offers. The owner's rule
 ("At least the lock's version"):

 - Run the newest installed copy of this repository's `mpm-shadow` whose version is at or above the lock's. Copies are looked
   for in `.cache/toolchain-bin/mpm-shadow`, the link `mpm-shadow install` maintains, and in the finished version folders of
   `<toolchain root>/mpm-shadow/`. When the copy that runs is newer than the lock, a note says so, names the lock file it
   compared (a repository of worktrees holds one lock each) and the upgrade command that moves that lock.
 - Only when no such copy exists, fetch the lock's archive for this platform, check it against the SHA-256 beside it, and
   install it into the toolchain root as `mpm-shadow install` does (`mpm-shadow-lock-install.ts`).
 - A copy older than the lock never runs.

 The same fetch-and-check steps are in `.github/actions/repo-tools-setup/action.yml` for CI, which runs
 `repo-tools-setup.ts` beside this module (also from source, with the Node already on `PATH`); this module is the
 developer-machine counterpart. It and its sibling modules import only `node:`
 modules, so that they run from source on a checkout that has no `node_modules` yet.

 A platform the lock has no archive row for (the plan's `mpm-shadow` entry is Linux-only) cannot be fetched, so there the
 `mpm-shadow` on `PATH` is used and a note says so.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  pathExists,
  VERSION_TIMEOUT_MS,
  versionOf,
} from './mpm-shadow-lock-fetch.ts';
import {
  findProgramUnder,
  isFinishedVersion,
  listVersionNames,
  SEARCH_DEPTH,
} from './mpm-shadow-lock-folder.ts';
import { installLockedArchive, } from './mpm-shadow-lock-install.ts';
import {
  compareVersions,
  lockFilePath,
  MpmShadowLockError,
  olderLockNote,
  readLockedMpmShadow,
  TOOL_DIRECTORY_NAME,
  TOOLCHAIN_BIN_LINK,
} from './mpm-shadow-lock-read.ts';

/**
 Where the copy to run came from.

 `toolchain-bin` is the link `mpm-shadow install` maintains in the worktree, `toolchain-root` a finished version folder of the
 toolchain root, `downloaded` the lock's archive fetched, checked and installed now, and `path` the `mpm-shadow` from `PATH`
 because the lock has no archive row for this platform.

 @example
 ```ts
 const source: MpmShadowSource = 'downloaded';
 ```
 */
export type MpmShadowSource = 'toolchain-bin' | 'toolchain-root' | 'downloaded' | 'path';

/**
 The copy to run and where it came from.

 @example
 ```ts
 const copy: ResolvedMpmShadow = { binary: '/repo/.cache/toolchain-bin/mpm-shadow', source: 'toolchain-bin', version: '0.2.1', };
 ```
 */
export type ResolvedMpmShadow = {
  /**
   Absolute path of the copy to run, or `mpm-shadow` when it is found through `PATH`
   */
  readonly binary: string;
  /**
   Where it came from
   */
  readonly source: MpmShadowSource;
  /**
   The version of the copy that runs: the lock's version or a newer one
   */
  readonly version: string;
};

/**
 Inputs of {@link resolveLockedMpmShadow}.

 @example
 ```ts
 const options: ResolveOptions = { root: '/repo', toolchainRoot, platformId: 'linux-x64', download: downloadBytes, log: console.error, };
 ```
 */
export type ResolveOptions = {
  /**
   Repository root, where `toolchain.lock.json` and `.cache` are
   */
  readonly root: string;
  /**
   The toolchain root `mpm-shadow install` installs into, where a fetched copy is installed
   */
  readonly toolchainRoot: string;
  /**
   Every toolchain root a copy can sit in (the repository root and the machine-global one); the newest copy wins wherever it is
   */
  readonly scannedRoots: readonly string[];
  /**
   The lock's platform id of this machine
   */
  readonly platformId: string;
  /**
   Fetches an archive's bytes
   */
  readonly download: (url: string,) => Promise<Uint8Array>;
  /**
   Prints one note for the person running the task (standard error, so that standard output stays the program's)
   */
  readonly log: (line: string,) => void;
};

/**
 An installed copy of `mpm-shadow` that printed its version.

 @example
 ```ts
 const candidate: Candidate = { binary: '/x/mpm-shadow', version: '0.2.2', source: 'toolchain-root', };
 ```
 */
type Candidate = {
  readonly binary: string;
  readonly version: string;
  readonly source: 'toolchain-bin' | 'toolchain-root';
};

/**
 What a look at the installed copies found.

 @example
 ```ts
 const found: Candidates = { candidates: [], notes: [], };
 ```
 */
type Candidates = {
  /**
   Copies that print a version, the exposure link first, then the toolchain root's newest first
   */
  readonly candidates: readonly Candidate[];
  /**
   Why other copies were passed over
   */
  readonly notes: readonly string[];
};

/**
 Reads the text of the repository's lock.

 @param root - Repository root

 @returns The lock's text

 @throws MpmShadowLockError when the file cannot be read

 @example
 ```ts
 const lockText = await readLockText('/repo',);
 ```
 */
async function readLockText(root: string,): Promise<string> {
  try {
    return await readFile(
      lockFilePath(root,),
      'utf8',
    );
  }
  catch (error) {
    throw new MpmShadowLockError(
      `could not read ${lockFilePath(root,)} (${String(error,)})`,
      { cause: error, },
    );
  }
}

/**
 Asks one existing program for its version.

 @param binary - Path of the program

 @param label - How the notes name it

 @param source - Where it was found

 @returns The candidate when it printed a version, else the note on why it was passed over

 @example
 ```ts
 const asked = await askVersion({ binary, label: '.cache/toolchain-bin/mpm-shadow', source: 'toolchain-bin', },);
 ```
 */
async function askVersion(
  {
    binary,
    label,
    source,
  }: {
    readonly binary: string;
    readonly label: string;
    readonly source: 'toolchain-bin' | 'toolchain-root';
  },
): Promise<{ readonly candidate: Candidate; } | { readonly note: string; }> {
  /**
   What the program says it is.
   */
  const answer = await versionOf({
    binary,
    timeoutMs: VERSION_TIMEOUT_MS,
  },);
  if (answer.usable)
    return { candidate: {
      binary,
      version: answer.version,
      source,
    }, };
  return { note: `${label} does not run (${answer.reason})`, };
}

/**
 Looks at the copies installed for this repository: the exposure link in the worktree, and every finished version folder in
 the toolchain root.

 @param root - Repository root

 @param scannedRoots - Every toolchain root a copy can sit in

 @returns The copies that print a version and the reasons the others were passed over

 @example
 ```ts
 const { candidates, notes, } = await gatherCandidates({ root, scannedRoots, },);
 ```
 */
async function gatherCandidates(
  {
    root,
    scannedRoots,
  }: {
    readonly root: string;
    readonly scannedRoots: readonly string[];
  },
): Promise<Candidates> {
  /**
   The exposure link, when there is one.
   */
  const link = join(
    root,
    TOOLCHAIN_BIN_LINK,
  );
  /**
   What the exposure link says, as a candidate or as the reason it was passed over; none when there is no link.
   */
  const linkAsked = (await pathExists(link,))
    ? [await askVersion({
      binary: link,
      label: TOOLCHAIN_BIN_LINK,
      source: 'toolchain-bin',
    },),]
    : [];
  /**
   What each finished-looking version folder of each root says, as a candidate or as the reason it was passed over.
   */
  const rootAsked = (await Promise.all(scannedRoots.map(async function askRoot(toolchainRoot,) {
    /**
     The tool folder of this root.
     */
    const toolDirectory = join(
      toolchainRoot,
      TOOL_DIRECTORY_NAME,
    );
    /**
     The names of its version folders, newest first.
     */
    const names = await listVersionNames(toolDirectory,);
    return await Promise.all(names.map(async function askFolder(name,) {
      /**
       The version folder.
       */
      const folder = join(
        toolDirectory,
        name,
      );
      /**
       Path of the program, empty when the install did not finish.
       */
      const binary = (await isFinishedVersion(folder,)) ? await findProgramUnder({
        directory: folder,
        depth: SEARCH_DEPTH,
      },) : '';
      if (binary === '')
        return { note: `${folder} is not a finished install`, };
      return await askVersion({
        binary,
        label: folder,
        source: 'toolchain-root',
      },);
    },),);
  },),)).flat();
  /**
   Everything that was asked, the link first.
   */
  const asked = [
    ...linkAsked,
    ...rootAsked,
  ];
  return {
    candidates: asked.flatMap(function candidatesOf(entry,) {
      return 'candidate' in entry ? [entry.candidate,] : [];
    },),
    notes: asked.flatMap(function notesOf(entry,) {
      return 'note' in entry ? [entry.note,] : [];
    },),
  };
}

/**
 The last resort on a platform for which the lock has no archive row and no installed copy is at or above the lock's version:
 the `mpm-shadow` on `PATH`, but only when it prints a version at or above the lock's. A copy older than the lock never runs
 here either.

 @param lockedVersion - The version the lock names

 @param lockFile - Path of the lock, named in the note of a copy that runs

 @param platformId - The lock's platform id of this machine

 @param passedOver - Why the installed copies were passed over, for the message when this one is refused too

 @param log - Prints one note for the person running the task

 @returns The `PATH` copy as the one to run

 @throws MpmShadowLockError when the `mpm-shadow` on `PATH` does not run or is older than the lock's version

 @example
 ```ts
 const copy = await usePathCopy({ lockedVersion: '0.2.2', lockFile: '/repo/toolchain.lock.json', platformId: 'macos-arm64', passedOver: [], log, },);
 ```
 */
async function usePathCopy(
  {
    lockedVersion,
    lockFile,
    platformId,
    passedOver,
    log,
  }: {
    readonly lockedVersion: string;
    readonly lockFile: string;
    readonly platformId: string;
    readonly passedOver: readonly string[];
    readonly log: (line: string,) => void;
  },
): Promise<ResolvedMpmShadow> {
  /**
   What the `mpm-shadow` found through `PATH` says it is.
   */
  const answer = await versionOf({
    binary: 'mpm-shadow',
    timeoutMs: VERSION_TIMEOUT_MS,
  },);
  /**
   Why no copy can run, for the refusals.
   */
  const context = `toolchain.lock.json has no mpm-shadow archive for ${platformId}, so none can be fetched${
     passedOver.length === 0 ? '' : ` (${passedOver.join('; ',)})`}`;
  if (!answer.usable) {
    throw new MpmShadowLockError(
      `${context}, and the mpm-shadow on PATH does not run (${answer.reason}); install mpm-shadow ${lockedVersion} or newer`,
    );
  }
  if (compareVersions({
    left: answer.version,
    right: lockedVersion,
  },) < 0) {
    throw new MpmShadowLockError(
      `${context}, and the mpm-shadow on PATH is ${answer.version}, older than the lock's ${lockedVersion}, which never runs; install mpm-shadow ${lockedVersion} or newer`,
    );
  }
  log(`${context}, so the mpm-shadow on PATH (${answer.version}) is used`,);
  if (compareVersions({
    left: answer.version,
    right: lockedVersion,
  },) > 0)
    log(
      olderLockNote({
        lockFile,
        lockedVersion,
        usedVersion: answer.version,
      },),
    );
  return {
    binary: 'mpm-shadow',
    source: 'path',
    version: answer.version,
  };
}

/**
 Finds the `mpm-shadow` copy to run: the newest installed one at or above the version `toolchain.lock.json` names, else the
 lock's archive fetched, checked and installed.

 @param root - Repository root

 @param toolchainRoot - The toolchain root `mpm-shadow install` installs into

 @param scannedRoots - Every toolchain root a copy can sit in: the repository root and the machine-global one

 @param platformId - The lock's platform id of this machine

 @param download - Fetches an archive's bytes

 @param log - Prints one note for the person running the task

 @returns The copy to run and where it came from

 @throws MpmShadowLockError when the lock cannot be read, or a needed download fails its SHA-256 or version check

 @example
 ```ts
 const copy = await resolveLockedMpmShadow({ root, toolchainRoot, scannedRoots, platformId: 'linux-x64', download: downloadBytes, log: console.error, },);
 ```
 */
export async function resolveLockedMpmShadow(
  {
    root,
    toolchainRoot,
    scannedRoots,
    platformId,
    download,
    log,
  }: ResolveOptions,
): Promise<ResolvedMpmShadow> {
  /**
   Path of the lock this run compares, named in every note and in every failure that names a version.
   */
  const lockFile: string = lockFilePath(root,);
  /**
   What the lock names for this platform.
   */
  const locked = readLockedMpmShadow({
    lockText: await readLockText(root,),
    platformId,
  },);
  /**
   The installed copies that print a version, and the reasons the others were passed over.
   */
  const {
    candidates,
    notes,
  } = await gatherCandidates({
    root,
    scannedRoots,
  },);
  /**
   The copies that are at or above the lock's version, the newest first; the exposure link wins a tie because it comes first.
   */
  const eligible = candidates.filter(function isAtOrAboveLock(candidate,) {
    return compareVersions({
      left: candidate.version,
      right: locked.version,
    },) >= 0;
  },)
    .toSorted(function newestFirst(
      left,
      right,
    ) {
      return compareVersions({
        left: right.version,
        right: left.version,
      },);
    },);
  /**
   The copy that runs, when there is one.
   */
  const [best,] = eligible;
  if (best !== undefined) {
    if (compareVersions({
      left: best.version,
      right: locked.version,
    },) > 0)
      log(
        olderLockNote({
          lockFile,
          lockedVersion: locked.version,
          usedVersion: best.version,
        },),
      );
    return {
      binary: best.binary,
      source: best.source,
      version: best.version,
    };
  }
  /**
   Why no installed copy runs: those that print a version are older than the lock.
   */
  const older = candidates.map(function describeOlder(candidate,) {
    return `${candidate.binary} is mpm-shadow ${candidate.version}, older than the lock's ${locked.version}, and never runs`;
  },);
  /**
   Every reason a copy was passed over.
   */
  const passedOver = [
    ...older,
    ...notes,
  ];
  if (!locked.archive
    .found) {
    return await usePathCopy({
      lockedVersion: locked.version,
      lockFile,
      platformId,
      passedOver,
      log,
    },);
  }
  /**
   The passed-over copies as one clause, empty when there were none.
   */
  const clause = passedOver.length === 0 ? '' : ` (${passedOver.join('; ',)})`;
  log(`toolchain.lock.json names mpm-shadow ${locked.version} and no installed copy is at that version or above${clause}; fetching its archive for ${platformId}`,);
  /**
   The new copy, installed into the toolchain root.
   */
  const binary = await installLockedArchive({
    toolchainRoot,
    version: locked.version,
    row: locked.archive
      .row,
    download,
    log,
  },);
  log(`fetched and verified mpm-shadow ${locked.version} (SHA-256 ${locked.archive
    .row
    .sha256}), running ${binary}`,);
  return {
    binary,
    source: 'downloaded',
    version: locked.version,
  };
}
