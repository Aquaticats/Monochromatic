/**
 Finding, and when needed fetching, the `mpm-shadow` that `toolchain.lock.json` names.

 The root `toolchain:*` tasks run `mpm-shadow`, and the copy on a developer's `PATH` can be older than the lock: a plan
 field that a newer `mpm-shadow` writes (for example `tools.mpm-shadow.options.tagPrefix`) is "not a known field" to an
 older one, which exits 2 before doing anything. So these tasks do not run whatever `PATH` offers. They run the copy
 whose `--version` equals the version the lock records for the `mpm-shadow` tool, looked for in this order:

 1. `.cache/toolchain-bin/mpm-shadow`, the link `mpm-shadow install` itself maintains;
 2. `.cache/mpm-shadow-bootstrap/<version>/mpm-shadow`, the copy fetched earlier;
 3. the lock's archive for this platform, downloaded, checked against the SHA-256 beside it, and unpacked into 2.

 The same steps are in `.github/actions/repo-tools-setup/action.yml` for CI (plain shell there, because it runs before
 any Node exists); this module is the developer-machine counterpart. It and its two sibling modules import only `node:`
 modules, so that they run from source on a checkout that has no `node_modules` yet.

 A platform the lock has no archive row for (the plan's `mpm-shadow` entry is Linux-only) cannot be fetched, so there the
 `mpm-shadow` on `PATH` is used and a note says so.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  fetchLockedMpmShadow,
  pathExists,
  versionOf,
} from './mpm-shadow-lock-fetch.ts';
import {
  BOOTSTRAP_CACHE_DIRECTORY,
  MpmShadowLockError,
  readLockedMpmShadow,
  TOOLCHAIN_BIN_LINK,
} from './mpm-shadow-lock-read.ts';

/**
 Where the copy to run came from.

 `toolchain-bin` and `bootstrap-cache` were already there with the locked version, `downloaded` was fetched and checked now,
 and `path` is the `mpm-shadow` from `PATH` because the lock has no archive row for this platform.

 @example
 ```ts
 const source: MpmShadowSource = 'downloaded';
 ```
 */
export type MpmShadowSource = 'toolchain-bin' | 'bootstrap-cache' | 'downloaded' | 'path';

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
   The version the lock names
   */
  readonly version: string;
};

/**
 Inputs of {@link resolveLockedMpmShadow}.

 @example
 ```ts
 const options: ResolveOptions = { root: '/repo', platformId: 'linux-x64', download: downloadBytes, log: console.error, };
 ```
 */
export type ResolveOptions = {
  /**
   Repository root, where `toolchain.lock.json` and `.cache` are
   */
  readonly root: string;
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
      join(
        root,
        'toolchain.lock.json',
      ),
      'utf8',
    );
  }
  catch (error) {
    throw new MpmShadowLockError(
      `could not read ${join(
        root,
        'toolchain.lock.json',
      )} (${String(error,)})`,
      { cause: error, },
    );
  }
}

/**
 Looks at one existing candidate: whether it is the locked version, and if not, a note on why it was passed over.

 @param path - Absolute path of the candidate

 @param label - How the notes name it

 @param version - The version the lock names

 @returns `{ match: true }` when it prints the locked version, else the note

 @example
 ```ts
 const verdict = await judgeCandidate({ path, label: '.cache/toolchain-bin/mpm-shadow', version: '0.2.1', },);
 ```
 */
async function judgeCandidate(
  {
    path,
    label,
    version,
  }: {
    readonly path: string;
    readonly label: string;
    readonly version: string;
  },
): Promise<{ readonly match: true; } | {
  readonly match: false;
  readonly note: string
}> {
  /**
   What the candidate says it is.
   */
  const answer = await versionOf(path,);
  if (answer.usable && (answer.version === version))
    return { match: true, };
  return {
    match: false,
    note: answer.usable ? `${label} is mpm-shadow ${answer.version}` : `${label} does not run (${answer.reason})`,
  };
}

/**
 Finds the `mpm-shadow` copy to run: the one at the version `toolchain.lock.json` names.

 @param root - Repository root

 @param platformId - The lock's platform id of this machine

 @param download - Fetches an archive's bytes

 @param log - Prints one note for the person running the task

 @returns The copy to run and where it came from

 @throws MpmShadowLockError when the lock cannot be read, or a needed download fails its SHA-256 or version check

 @example
 ```ts
 const copy = await resolveLockedMpmShadow({ root, platformId: 'linux-x64', download: downloadBytes, log: console.error, },);
 ```
 */
export async function resolveLockedMpmShadow(
  {
    root,
    platformId,
    download,
    log,
  }: ResolveOptions,
): Promise<ResolvedMpmShadow> {
  /**
   What the lock names for this platform.
   */
  const locked = readLockedMpmShadow({
    lockText: await readLockText(root,),
    platformId,
  },);
  /**
   The exposure link that `mpm-shadow install` maintains.
   */
  const link = join(
    root,
    TOOLCHAIN_BIN_LINK,
  );
  /**
   The copy fetched earlier.
   */
  const cached = join(
    root,
    BOOTSTRAP_CACHE_DIRECTORY,
    locked.version,
    'mpm-shadow',
  );
  /**
   Notes on candidates that were passed over, printed only when the run goes on to fetch.
   */
  const passedOver: string[] = [];
  if (await pathExists(link,)) {
    /**
     Whether the link is the locked version.
     */
    const verdict = await judgeCandidate({
      path: link,
      label: TOOLCHAIN_BIN_LINK,
      version: locked.version,
    },);
    if (verdict.match)
      return {
        binary: link,
        source: 'toolchain-bin',
        version: locked.version,
      };
    passedOver.push(verdict.note,);
  }
  if (await pathExists(cached,)) {
    /**
     Whether the cached copy is the locked version.
     */
    const verdict = await judgeCandidate({
      path: cached,
      label: `${BOOTSTRAP_CACHE_DIRECTORY}/${locked.version}`,
      version: locked.version,
    },);
    if (verdict.match)
      return {
        binary: cached,
        source: 'bootstrap-cache',
        version: locked.version,
      };
    passedOver.push(verdict.note,);
  }
  if (!locked.archive
    .found) {
    log(
      `toolchain.lock.json has no mpm-shadow archive for ${platformId}, so the mpm-shadow on PATH is used `
        + `(the lock names ${locked.version}; check \`mpm-shadow --version\`)`,
    );
    return {
      binary: 'mpm-shadow',
      source: 'path',
      version: locked.version,
    };
  }
  /**
   The passed-over candidates as one clause, empty when there were none.
   */
  const clause = passedOver.length === 0 ? '' : ` (${passedOver.join('; ',)}; not used)`;
  log(`toolchain.lock.json names mpm-shadow ${locked.version}${clause}; fetching its archive for ${platformId}`,);
  /**
   The new copy.
   */
  const binary = await fetchLockedMpmShadow({
    root,
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
