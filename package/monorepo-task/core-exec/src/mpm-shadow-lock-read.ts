/**
 Reading `toolchain.lock.json` for the `mpm-shadow` tool, and naming this machine's platform the way the lock does.

 Imports only `node:` modules, like the rest of the `mpm-shadow` bootstrap, so that it runs from source on a checkout that
 has no `node_modules` yet (see `mpm-shadow-lock-logic.ts`).

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

//region Names: the tool, its install folder and the failure

/**
 Tool id of the `mpm-shadow` archive entry in `toolchain.plan.jsonc` and `toolchain.lock.json`.

 @example
 ```ts
 const row = lock.tools[MPM_SHADOW_TOOL_ID];
 ```
 */
export const MPM_SHADOW_TOOL_ID = 'mpm-shadow';

/**
 Name of the directory that holds every installed version of the `mpm-shadow` tool, below the toolchain root.

 It is the install directory name `mpm-shadow` gives the tool id (`mpm-shadow`): the versions sit in
 `<toolchain root>/mpm-shadow/<version>/`, as `mpm-shadow install` puts them (`archive_staging.rs`, `layout.rs`).

 @example
 ```ts
 const toolDirectory = join(toolchainRoot, TOOL_DIRECTORY_NAME);
 ```
 */
export const TOOL_DIRECTORY_NAME = 'mpm-shadow';

/**
 File whose presence shows that an install of a version finished; `mpm-shadow install` writes it last
 (`archive_staging.rs`, `COMPLETE_MARKER`), and its content is `<tool id> <version>` and a line break.

 @example
 ```ts
 await writeFile(join(staging, COMPLETE_MARKER), `${MPM_SHADOW_TOOL_ID} ${version}\n`);
 ```
 */
export const COMPLETE_MARKER = '.mpm-shadow-installed';

/**
 Name of the link, inside the tool directory, that names the newest finished version (`published_archive.rs`).

 @example
 ```ts
 const link = join(toolDirectory, CURRENT_LINK_NAME);
 ```
 */
export const CURRENT_LINK_NAME = 'current';

/**
 The command that moves the lock to the newest `mpm-shadow` release, named in the note printed when a newer copy runs.
 It is the root task `toolchain:update` with the tool id appended, which runs `mpm-shadow update mpm-shadow`,
 and it moves the lock of the tree it is run in, which is the tree the entry searched upward from.

 @example
 ```ts
 log(`upgrade that lock with: ${UPGRADE_COMMAND}`,);
 ```
 */
export const UPGRADE_COMMAND = 'mise run toolchain:update mpm-shadow';

/**
 Name of the lock file, at the root of a worktree and in the machine-global toolchain folder.

 @example
 ```ts
 const lockFile = join(root, LOCK_FILE_NAME);
 ```
 */
export const LOCK_FILE_NAME = 'toolchain.lock.json';

/**
 Path of the lock of the tree at `root`.

 @param root - Repository root

 @returns Absolute path of its lock file

 @example
 ```ts
 const lockFile = lockFilePath('/repo',);
 ```
 */
export function lockFilePath(root: string,): string {
  return join(
    root,
    LOCK_FILE_NAME,
  );
}

/**
 The note that says a newer `mpm-shadow` runs than the lock names, which lock it compared, and how to move that lock.

 The lock's path is part of the note because a repository of worktrees holds one lock each: the version pair alone does not
 say which worktree is behind, and the copy that runs can come from any worktree.

 @param lockFile - Path of the lock the note compared

 @param lockedVersion - The version the lock names

 @param usedVersion - The version of the copy that runs

 @returns The note, one line

 @example
 ```ts
 log(olderLockNote({ lockFile: '/repo/toolchain.lock.json', lockedVersion: '0.2.2', usedVersion: '0.2.3', },),);
 ```
 */
export function olderLockNote(
  {
    lockFile,
    lockedVersion,
    usedVersion,
  }: {
    readonly lockFile: string;
    readonly lockedVersion: string;
    readonly usedVersion: string;
  },
): string {
  return `locked at ${lockedVersion} in ${lockFile}, ${usedVersion} runs; upgrade that lock with: ${UPGRADE_COMMAND}`;
}

/**
 The exposure link that `mpm-shadow install` maintains, relative to the repository root.

 @example
 ```ts
 const link = join(root, TOOLCHAIN_BIN_LINK);
 ```
 */
export const TOOLCHAIN_BIN_LINK = '.cache/toolchain-bin/mpm-shadow';

/**
 The lock's `mpm-shadow` could not be found, fetched or trusted.

 @example
 ```ts
 throw new MpmShadowLockError('toolchain.lock.json has no mpm-shadow tool',);
 ```
 */
export class MpmShadowLockError extends Error {
  /**
   Names the failure for logs and tests
   */
  override readonly name = 'MpmShadowLockError';
}

//endregion Names: the tool, its install folder and the failure

//region The lock: version and archive row

/**
 What the lock records for the `mpm-shadow` archive of one platform.

 @example
 ```ts
 const row: ArchiveRow = { url: 'https://example.invalid/mpm-shadow.tar.gz', sha256: 'ab12', };
 ```
 */
export type ArchiveRow = {
  /**
   Download address of the release archive
   */
  readonly url: string;
  /**
   Lowercase hex SHA-256 of the archive
   */
  readonly sha256: string;
};

/**
 The `mpm-shadow` tool as the lock records it, for one platform.

 `archive.found` is false when the lock holds no archive row for the platform.

 @example
 ```ts
 const locked: LockedMpmShadow = { version: '0.2.1', archive: { found: false, }, };
 ```
 */
export type LockedMpmShadow = {
  /**
   Version the lock names
   */
  readonly version: string;
  /**
   The archive row for the asked platform, or a note that there is none
   */
  readonly archive: {
    readonly found: true;
    readonly row: ArchiveRow
  } | { readonly found: false; };
};

/**
 Whether a parsed JSON value is a plain object.

 @param value - Parsed JSON

 @returns True for an object that is not an array and not null

 @example
 ```ts
 isRecord({ a: 1, },); // true
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Parses the text of the lock.

 @param lockText - Content of the lock file

 @returns The parsed value, whose shape the caller checks

 @throws MpmShadowLockError when the text is not JSON

 @example
 ```ts
 const lock = parseLockText('{"tools":{}}',);
 ```
 */
function parseLockText(lockText: string,): unknown {
  try {
    return JSON.parse(lockText,);
  }
  catch (error) {
    throw new MpmShadowLockError(
      `toolchain.lock.json is not valid JSON (${String(error,)})`,
      { cause: error, },
    );
  }
}

/**
 Reads the `mpm-shadow` tool out of the text of `toolchain.lock.json`.

 @param lockText - Content of the lock file

 @param platformId - The lock's platform id, for example `linux-x64`

 @returns The locked version and the archive row for the platform

 @throws MpmShadowLockError when the text is not JSON, has no `mpm-shadow` tool, or the tool has no version

 @example
 ```ts
 const locked = readLockedMpmShadow({ lockText, platformId: 'linux-x64', },);
 ```
 */
export function readLockedMpmShadow(
  {
    lockText,
    platformId,
  }: {
    readonly lockText: string;
    readonly platformId: string;
  },
): LockedMpmShadow {
  /**
   Parsed lock; its shape is checked below.
   */
  const lock = parseLockText(lockText,);
  if ((!isRecord(lock,)) || (!isRecord(lock.tools,)))
    throw new MpmShadowLockError('toolchain.lock.json has no "tools" object',);
  /**
   The `mpm-shadow` row of the lock.
   */
  const tool = lock.tools[MPM_SHADOW_TOOL_ID];
  if (!isRecord(tool,))
    throw new MpmShadowLockError(`toolchain.lock.json has no "${MPM_SHADOW_TOOL_ID}" tool`,);
  /**
   The version the lock records.
   */
  const {version} = tool;
  if (((typeof version) !== 'string') || (version === ''))
    throw new MpmShadowLockError(`the "${MPM_SHADOW_TOOL_ID}" tool of toolchain.lock.json has no version`,);
  /**
   The archive rows by platform id.
   */
  const {artifacts} = tool;
  /**
   The row for this platform.
   */
  const row = isRecord(artifacts,) ? artifacts[platformId] : {};
  if (isRecord(row,) && ((typeof row.url) === 'string')
    && ((typeof row.sha256) === 'string'))
    return {
      version,
      archive: {
        found: true,
        row: {
          url: row.url,
          sha256: row.sha256
            .toLowerCase(),
        },
      },
    };
  return {
    version,
    archive: { found: false, },
  };
}

//endregion The lock: version and archive row

//region The platform: the lock's id for this machine

/**
 The lock's platform id for a machine.

 @param platform - Node's `process.platform`

 @param arch - Node's `process.arch`

 @param musl - Whether the machine's C library is musl (only a Linux machine is asked)

 @returns For example `linux-x64`, `linux-arm64-musl` or `macos-arm64`

 @example
 ```ts
 lockPlatformId({ platform: 'linux', arch: 'x64', musl: false, },); // 'linux-x64'
 ```
 */
export function lockPlatformId(
  {
    platform,
    arch,
    musl,
  }: {
    readonly platform: string;
    readonly arch: string;
    readonly musl: boolean;
  },
): string {
  /**
   Operating system names the lock uses where Node's differ.
   */
  const systemNames: Readonly<Record<string, string>> = {
    darwin: 'macos',
    win32: 'windows',
  };
  /**
   The operating system part of the id.
   */
  const system = systemNames[platform] ?? platform;
  /**
   The C library suffix, only for Linux.
   */
  const suffix = (platform === 'linux') && musl ? '-musl' : '';
  return `${system}-${arch}${suffix}`;
}

/**
 Whether `/lib` holds a musl loader (`ld-musl-<arch>.so.1`), which is how a musl Linux is told from a glibc one.

 @returns True on a musl machine; false on glibc, and when `/lib` cannot be read

 @example
 ```ts
 const musl = await hasMuslLoader();
 ```
 */
async function hasMuslLoader(): Promise<boolean> {
  try {
    return (await readdir('/lib',)).some(function isMuslLoader(name,) {
      return name.startsWith('ld-musl-',);
    },);
  }
  catch (error) {
    // No readable /lib: the machine counts as glibc, whose rows the lock has for every Linux platform it lists.
    console.error(`[toolchain] could not read /lib to tell musl from glibc: ${String(error,)}`,);
    return false;
  }
}

/**
 The lock's platform id of this machine.

 @returns For example `linux-x64`

 @example
 ```ts
 const platformId = await hostLockPlatformId();
 ```
 */
export async function hostLockPlatformId(): Promise<string> {
  return lockPlatformId({
    platform: process.platform,
    arch: process.arch,
    musl: (process.platform === 'linux') && await hasMuslLoader(),
  },);
}

//endregion The platform: the lock's id for this machine

//region The program's own answer

/**
 Extracts the version from the first line `mpm-shadow --version` prints (`mpm-shadow 0.2.1`).

 @param output - Standard output of `--version`

 @returns The version, or an empty string when the output is not that line

 @example
 ```ts
 parseVersionOutput('mpm-shadow 0.2.1\n',); // '0.2.1'
 ```
 */
export function parseVersionOutput(output: string,): string {
  /**
   The words of the first line.
   */
  const words = output.trim()
    .split('\n',)[0]
    ?.trim()
    .split(' ',)
    ?? [];
  return (words[0] === 'mpm-shadow') && (words.length === 2) ? (words[1] ?? '') : '';
}

//endregion The program's own answer

//region Versions and the toolchain root

/**
 How many numeric parts of a version order it (`major.minor.patch`).
 */
const CORE_PARTS = 3;

/**
 The first three numeric parts of a version, missing or non-numeric parts counting as 0 (`0.2.1-rc.1` is `[0, 2, 1]`).

 @param version - Version text

 @returns Three numbers

 @example
 ```ts
 numericCore('0.2.10',); // [0, 2, 10]
 ```
 */
function numericCore(version: string,): readonly number[] {
  /**
   The text before the first `-` or `+`.
   */
  const core = version.split('+',)[0]
    ?.split('-',)[0]
    ?? '';
  /**
   The parts as whole numbers, a part that is not a number counting as 0.
   */
  const parts = core.split('.',)
    .map(function toNumber(piece,) {
      return Math.trunc(Number(piece,),) || 0;
    },);
  return Array.from(
    { length: CORE_PARTS, },
    function partAt(
      _unused,
      index,
    ) {
      return parts[index] ?? 0;
    },
  );
}

/**
 The pre-release text of a version: after the first `-` and before any `+`.

 @param version - Version text

 @returns An empty list for a release, else a list holding the text (empty for `1.0.0-`)

 @example
 ```ts
 preReleaseOf('0.2.2-rc.1+build',); // ['rc.1']
 ```
 */
function preReleaseOf(version: string,): readonly string[] {
  return version.includes('-',) ? [version.slice(version.indexOf('-',) + 1,)
    .split('+',)[0]
    ?? '',] : [];
}

/**
 Whether a pre-release identifier is made of ASCII digits only.

 @param identifier - One dot-separated pre-release identifier

 @returns True for a non-empty run of digits

 @example
 ```ts
 isNumericIdentifier('10',); // true
 ```
 */
function isNumericIdentifier(identifier: string,): boolean {
  // The digit test stays local: this module and the rest of the `mpm-shadow`
  // bootstrap import only `node:` modules so they run from source on a checkout
  // that has no node_modules, where the shared text-scan import would not resolve.
  return (identifier !== '')
    && Array.from(identifier,)
      .every(function isDigit(character,) {
        return (character >= '0') && (character <= '9');
      },);
}

/**
 Orders two pre-release identifiers by SemVer 2.0.0: two numeric ones as numbers, a numeric one below an alphanumeric one, two
 alphanumeric ones by ASCII (UTF-16 code unit) order.

 @param left - Identifier from the first version

 @param right - Identifier from the second version

 @returns A negative number, 0, or a positive number

 @example
 ```ts
 compareIdentifiers({ left: '2', right: '10', },); // negative
 ```
 */
function compareIdentifiers(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  if (isNumericIdentifier(left,) && isNumericIdentifier(right,))
    return Number(left,) - Number(right,);
  if (isNumericIdentifier(left,))
    return -1;
  if (isNumericIdentifier(right,))
    return 1;
  if (left === right)
    return 0;
  return left < right ? -1 : 1;
}

/**
 Orders two pre-release texts by SemVer 2.0.0 (`rc.2` is below `rc.10`, `alpha` below `alpha.1`): identifier by identifier, and when
 all shared ones are equal the text with more identifiers is newer.

 @param left - Pre-release text of the first version

 @param right - Pre-release text of the second version

 @returns A negative number, 0, or a positive number

 @example
 ```ts
 comparePreRelease({ left: 'rc.2', right: 'rc.10', },); // negative
 ```
 */
function comparePreRelease(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   The dot-separated identifiers of the first text.
   */
  const leftIdentifiers = left.split('.',);
  /**
   The dot-separated identifiers of the second text.
   */
  const rightIdentifiers = right.split('.',);
  /**
   The first identifier pair that differs, if any.
   */
  const firstDifference = leftIdentifiers.map(function compareAt(
    identifier,
    index,
  ) {
    return index < rightIdentifiers.length
      ? compareIdentifiers({
        left: identifier,
        right: rightIdentifiers[index] ?? '',
      },)
      : 0;
  },)
    .find(function differs(delta,) {
      return delta !== 0;
    },);
  return firstDifference ?? (leftIdentifiers.length - rightIdentifiers.length);
}

/**
 Orders two versions: by the numeric core as numbers (`0.2.10` is newer than `0.2.9`), then a release above its own
 pre-release, as `mpm-shadow` does (`version.rs`, `compare_versions`), and then pre-release texts by SemVer 2.0.0 precedence
 (`rc.2` below `rc.10`), where `mpm-shadow` compares them as plain text. The repository has SemVer comparisons in
 `package/config/pnpr/src/version-order.ts` and `package/dependency-catalog/core-rust/src/semver.ts`; this module cannot import a
 workspace package because it must run before `pnpm install`, so it carries the same rule.

 @param left - First version

 @param right - Second version

 @returns A negative number when `left` is older, 0 when equal, a positive number when newer

 @example
 ```ts
 compareVersions({ left: '0.2.10', right: '0.2.9', },); // 1
 ```
 */
export function compareVersions(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   The first numeric difference, if any.
   */
  const byCore = numericCore(left,)
    .map(function difference(
      part,
      index,
    ) {
      return part - (numericCore(right,)[index] ?? 0);
    },)
    .find(function isDifferent(delta,) {
      return delta !== 0;
    },);
  if (byCore !== undefined)
    return byCore;
  /**
   The pre-release text of the left version, none for a release.
   */
  const [leftPre,] = preReleaseOf(left,);
  /**
   The pre-release text of the right version, none for a release.
   */
  const [rightPre,] = preReleaseOf(right,);
  if ((leftPre === undefined) && (rightPre === undefined))
    return 0;
  if (leftPre === undefined)
    return 1;
  if (rightPre === undefined)
    return -1;
  return comparePreRelease({
    left: leftPre,
    right: rightPre,
  },);
}

/**
 Refuses a version that is not one plain directory name, because it is joined onto the toolchain root and a crafted lock must
 not reach outside the tool's directory (`segment.rs`, `check_version`): empty, `.`, `..`, a separator, NUL, a leading dot,
 and `current`.

 @param version - Version text from the lock

 @throws MpmShadowLockError when the version could not be a directory name

 @example
 ```ts
 checkVersionName('0.2.1',);
 ```
 */
export function checkVersionName(version: string,): void {
  if (
    (version === '') || (version === '.')
      || (version === '..')
      || version.includes('/',)
      || version.includes('\\',)
      || version.includes('\0',)
      || version.startsWith('.',)
      || (version === CURRENT_LINK_NAME)
  ) {
    throw new MpmShadowLockError(
      `the version "${version}" that toolchain.lock.json names for ${MPM_SHADOW_TOOL_ID} is not one plain directory name, so nothing is installed under it`,
    );
  }
}

/**
 The data home `mpm-shadow` keeps its toolchain roots in: `$XDG_DATA_HOME`, else `$HOME/.local/share` (`$USERPROFILE` when there
 is no `$HOME`) (`scope.rs`, `data_home`).

 @param environment - Environment variables

 @returns Absolute path of the data home

 @throws MpmShadowLockError when neither the data home nor a home directory can be found

 @example
 ```ts
 const dataHome = dataHomeOf(process.env,);
 ```
 */
function dataHomeOf(environment: Readonly<NodeJS.ProcessEnv>,): string {
  /**
   The explicit data home, if the variable is set and not empty.
   */
  const explicit = environment.XDG_DATA_HOME;
  if ((explicit !== undefined) && (explicit !== ''))
    return explicit;
  /**
   The home directory.
   */
  const home = environment.HOME ?? environment.USERPROFILE;
  if ((home === undefined) || (home === ''))
    throw new MpmShadowLockError('neither HOME nor USERPROFILE nor XDG_DATA_HOME is set, so the toolchain root cannot be found',);
  return join(
    home,
    '.local',
    'share',
  );
}

/**
 The toolchain root `mpm-shadow install` installs into for a worktree: `<data home>/monochromatic-dev/toolchain`
 (`scope.rs`, `repo_scope`). A fetched copy is installed here.

 @param environment - Environment variables

 @returns Absolute path of the toolchain root

 @throws MpmShadowLockError when neither the data home nor a home directory can be found

 @example
 ```ts
 const root = toolchainRootOf(process.env,);
 ```
 */
export function toolchainRootOf(environment: Readonly<NodeJS.ProcessEnv>,): string {
  return join(
    dataHomeOf(environment,),
    'monochromatic-dev',
    'toolchain',
  );
}

/**
 Every toolchain root a copy of this repository's `mpm-shadow` can sit in: the repository root `toolchain` and the machine-global
 `toolchain-global`, in the order `mpm-shadow` lists them (`scope.rs`, `toolchain_roots`). The owner's rule is that this
 repository's tools are one set across both roots and the newest copy wins wherever it is.

 @param environment - Environment variables

 @returns Absolute paths of the two roots

 @throws MpmShadowLockError when neither the data home nor a home directory can be found

 @example
 ```ts
 const roots = toolchainRootsOf(process.env,); // ['.../monochromatic-dev/toolchain', '.../monochromatic-dev/toolchain-global']
 ```
 */
export function toolchainRootsOf(environment: Readonly<NodeJS.ProcessEnv>,): readonly string[] {
  /**
   The directory that holds both roots.
   */
  const base = join(
    dataHomeOf(environment,),
    'monochromatic-dev',
  );
  return [
    join(
      base,
      'toolchain',
    ),
    join(
      base,
      'toolchain-global',
    ),
  ];
}

//endregion Versions and the toolchain root
