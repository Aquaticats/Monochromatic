/**
 Reading `toolchain.lock.json` for the `mpm-shadow` tool, and naming this machine's platform the way the lock does.

 Imports only `node:` modules, like the rest of the `mpm-shadow` bootstrap, so that it runs from source on a checkout that
 has no `node_modules` yet (see `mpm-shadow-lock-logic.ts`).

 @module
 */

import { readdir, } from 'node:fs/promises';

//region Names: the tool, the cache and the failure

/**
 Tool id of the `mpm-shadow` archive entry in `toolchain.plan.jsonc` and `toolchain.lock.json`.

 @example
 ```ts
 const row = lock.tools[MPM_SHADOW_TOOL_ID];
 ```
 */
export const MPM_SHADOW_TOOL_ID = 'mpm-shadow';

/**
 Gitignored directory, relative to the repository root, that holds the copies the bootstrap fetched, one folder per version.

 @example
 ```ts
 const directory = join(root, BOOTSTRAP_CACHE_DIRECTORY, version);
 ```
 */
export const BOOTSTRAP_CACHE_DIRECTORY = '.cache/mpm-shadow-bootstrap';

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

//endregion Names: the tool, the cache and the failure

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
