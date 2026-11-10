/**
 Exact pnpm lockfile package-version eligibility.
 
 @module
 */

import {
  existsSync,
  readFileSync,
  statSync,
} from 'node:fs';
import {
  dirname,
  join,
} from 'node:path';

import { ancestorDirectories, } from './ancestor-directories.ts';
import type { InstalledPackageIdentity, } from './installed-package-identity.ts';

//region Lockfile lookup
// The sentinel for a project no pnpm lockfile governs, the cached keys per
// lockfile path, and the search for the nearest governing lockfile.

/**
 Sentinel when configured project has no governing pnpm lockfile.
 */
const PNPM_LOCKFILE_UNAVAILABLE: unique symbol = Symbol(
  'configured project governing pnpm lockfile was unavailable',
);

/**
 Cached parsed package keys and metadata identity by exact lockfile path.
 */
const lockKeysByPath = new Map<string, {
  readonly signature: string;
  readonly keys: ReadonlySet<string>;
}>();

/**
 Finds nearest pnpm lockfile governing configured project.
 
 @param configFileName - Configured TypeScript project path.
 
 @returns lockfile path or unavailable sentinel.
 */
function nearestPnpmLockfile(
  configFileName: string,
): string | typeof PNPM_LOCKFILE_UNAVAILABLE {
  for (const directory of ancestorDirectories(dirname(configFileName,),)) {
    /**
     Candidate lockfile at current ancestor.
     */
    const candidate = join(
      directory,
      'pnpm-lock.yaml',
    );
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor verifies package eligibility before implementation analysis.
    if (existsSync(candidate,))
      return candidate;
  }
  return PNPM_LOCKFILE_UNAVAILABLE;
}

//endregion Lockfile lookup

//region Key parsing
// The sentinel for a line carrying no package key, the indentation width every
// such key sits at, and the parse of one line.

/**
 Sentinel for lockfile lines carrying no package key.
 */
const LINE_WITHOUT_PACKAGE_KEY: unique symbol = Symbol(
  'lockfile line carries no package or snapshot key',
);

/**
 Key indentation width preceding every package or snapshot key line.
 */
const KEY_INDENT_WIDTH = 2;

/**
 Key start offset when indentation is followed by an opening quote.
 */
const QUOTED_KEY_START = KEY_INDENT_WIDTH + 1;

/**
 Extracts package key from one lockfile line when line carries one.
 
 Package and snapshot keys sit at two-space indentation as
 `  name@version:`, `  name@version(peers):`, or their single-quoted
 variants used by scoped names. Extraction reads between indentation
 (plus optional opening quote) and matching delimiter, so membership
 of `name@version` in extracted keys equals prior per-call prefix and
 delimiter probing.
 
 @param line - One lockfile line.
 
 @returns exact `name@version` key or no-key sentinel.
 */
function lockfileLinePackageKey(
  line: string,
): string | typeof LINE_WITHOUT_PACKAGE_KEY {
  if (!line.startsWith('  ',))
    return LINE_WITHOUT_PACKAGE_KEY;
  /**
   Whether key uses single-quoted form.
   */
  const quoted = line[KEY_INDENT_WIDTH] === "'";
  /**
   First character index of candidate key text.
   */
  const start = quoted ? QUOTED_KEY_START : KEY_INDENT_WIDTH;
  /* Linear delimiter scan; quoted keys end at closing quote or peer
   * parenthesis, unquoted keys end at colon or peer parenthesis. */
  for (let index = start; index < line.length; index++) {
    /**
     Current scanned character.
     */
    const character = line[index];
    if ((character === '(')
      || (quoted ? (character === "'") : (character === ':')))
      return line.slice(
        start,
        index,
      );
  }
  return LINE_WITHOUT_PACKAGE_KEY;
}

//endregion Key parsing

//region Reading the keys
// Every package and snapshot key of one lockfile, read and parsed once per
// metadata identity.

/**
 Parses every package and snapshot key from lockfile text.
 
 @param text - Complete governing lockfile text.
 
 @returns exact `name@version` keys occurring in lockfile.
 */
function lockfilePackageKeys(text: string,): ReadonlySet<string> {
  /**
   Collected package keys for one lockfile snapshot.
   */
  const keys = new Set<string>();
  for (const line of text.split('\n',)) {
    /**
     Extracted key for current line, or no-key sentinel.
     */
    const key = lockfileLinePackageKey(line,);
    if ((typeof key) === 'string')
      keys.add(key,);
  }
  return keys;
}

/**
 Reads and parses governing lockfile once per metadata identity.
 
 @param path - Exact pnpm lockfile path.
 
 @returns parsed package keys.
 */
function lockfileKeys(path: string,): ReadonlySet<string> {
  /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor validates governing lockfile metadata before cached reuse. */
  /**
   Current governing lockfile metadata.
   */
  const metadata = statSync(path,);
  /* oxlint-enable no-restricted-syntax/no-sync */
  /**
   Metadata signature invalidated by ordinary lockfile replacement or write.
   */
  const signature = `${String(metadata.dev,)}:${String(metadata.ino,)}:${String(metadata.size,)}:${String(metadata.mtimeMs,)}:${String(metadata.ctimeMs,)}`;
  /**
   Prior parsed keys retained for exact metadata identity.
   */
  const cached = lockKeysByPath.get(path,);
  if ((cached !== undefined) && (cached.signature === signature))
    return cached.keys;
  /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor reads governing lockfile after metadata cache miss. */
  /**
   Exact governing lockfile text.
   */
  const text = readFileSync(
    path,
    'utf8',
  );
  /* oxlint-enable no-restricted-syntax/no-sync */
  /**
   Package keys parsed once for every later eligibility query.
   */
  const keys = lockfilePackageKeys(text,);
  lockKeysByPath.set(
    path,
    {
      signature,
      keys,
    },
  );
  return keys;
}

//endregion Reading the keys

//region Eligibility
// The process-local reset, and the exported test for whether one installed
// version occurs in the governing lockfile.

/**
 Clears process-local lockfile keys at semantic lifecycle boundary.
 
 @example
 ```ts
 clearLockfilePackageEligibilityCache();
 ```
 */
export function clearLockfilePackageEligibilityCache(): void {
  lockKeysByPath.clear();
}

/**
 Tests whether exact installed package version occurs in governing pnpm lockfile.
 
 @param configFileName - Consumer configured-project path.
 
 @param identity - Exact installed package identity.
 
 @returns whether package version is eligible for implementation inference.
 
 @example
 ```ts
 packageVersionIsLocked({ configFileName, identity });
 ```
 */
export function packageVersionIsLocked({
  configFileName,
  identity,
}: {
  readonly configFileName: string;
  readonly identity: InstalledPackageIdentity;
}): boolean {
  /**
   Governing pnpm lockfile path.
   */
  const path = nearestPnpmLockfile(configFileName,);
  if (path === PNPM_LOCKFILE_UNAVAILABLE)
    return false;
  return lockfileKeys(path,)
    .has(`${identity.name}@${identity.version}`,);
}
//endregion Eligibility
