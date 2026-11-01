/**
 Exact installed package identity from declaration or implementation path.
 
 @module
 */

import { readFileSync, } from 'node:fs';
import {
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { ancestorDirectories, } from './ancestor-directories.ts';

/**
 Package identity logger.
 */
const l = tagged({ tag: 'installed-package-identity', },);

/**
 Sentinel when file does not belong to readable package.
 */
export const INSTALLED_PACKAGE_UNAVAILABLE: unique symbol = Symbol(
  'installed package identity unavailable',
);

/**
 Exact package manifest identity needed by implementation inference.
 */
export type InstalledPackageIdentity = {
  readonly root: string;
  readonly name: string;
  readonly version: string;
  readonly major: number;
  readonly manifest: Readonly<Record<string, unknown>>;
};

/**
 Tests whether unknown JSON value is a package manifest record.
 
 @param value - Parsed package manifest.
 
 @returns whether required identity fields are strings.
 */
function isManifest(value: unknown,): value is Readonly<Record<string, unknown>> & {
  readonly name: string;
  readonly version: string;
} {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('name' in value)
    && ((typeof value.name) === 'string')
    && ('version' in value)
    && ((typeof value.version) === 'string');
}

/**
 Resolves package root from final installed-package boundary.
 
 @param fileName - Declaration or implementation path.
 
 @returns installed package root or unavailable sentinel.
 */
function nodeModulesPackageRoot(
  fileName: string,
): string | typeof INSTALLED_PACKAGE_UNAVAILABLE {
  /**
   Portable separators for installed package segment scan.
   */
  const normalized = fileName.replaceAll(
    '\\',
    '/',
  );
  /**
   Final installed-package boundary handles pnpm virtual stores.
   */
  const marker = '/node_modules/';
  /**
   Final boundary before actual package name.
   */
  const markerIndex = normalized.lastIndexOf(marker,);
  if (markerIndex === (-1))
    return INSTALLED_PACKAGE_UNAVAILABLE;
  /**
   Path segments beginning with package scope or name.
   */
  const segments = normalized
    .slice(markerIndex + marker.length,)
    .split('/');
  /**
   First unscoped package name or scope.
   */
  const [first, second,] = segments;
  if ((first === undefined) || (first.length === 0))
    return INSTALLED_PACKAGE_UNAVAILABLE;
  /**
   Exact package name reconstructed from scoped segments.
   */
  const packageName = first.startsWith('@',)
    ? ((second === undefined) || (second.length === 0)
      ? INSTALLED_PACKAGE_UNAVAILABLE
      : `${first}/${second}`)
    : first;
  if (packageName === INSTALLED_PACKAGE_UNAVAILABLE)
    return INSTALLED_PACKAGE_UNAVAILABLE;
  return normalized.slice(
    0,
    markerIndex + marker.length
      + packageName.length,
  );
}

/**
 Resolves nearest workspace package root by manifest identity.
 
 @param fileName - Source path outside installed package boundary.
 
 @returns package root or unavailable sentinel.
 */
function workspacePackageRoot(
  fileName: string,
): string | typeof INSTALLED_PACKAGE_UNAVAILABLE {
  for (const directory of ancestorDirectories(dirname(fileName,),)) {
    /**
     Candidate package manifest at current ancestor.
     */
    const manifestPath = join(
      directory,
      'package.json',
    );
    try {
      /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor resolves exact package identity before effect analysis. */
      /**
       Parsed candidate workspace package manifest.
       */
      const parsed: unknown = JSON.parse(readFileSync(
        manifestPath,
        'utf8',
      ),);
      /* oxlint-enable no-restricted-syntax/no-sync */
      if (isManifest(parsed,))
        return directory;
    }
    catch (error) {
      l.debug(`package manifest probe skipped for ${manifestPath}: ${String(error,)}`,);
    }
  }
  return INSTALLED_PACKAGE_UNAVAILABLE;
}

/**
 Parses exact package identity from package root.
 
 @param root - Installed or workspace package root.
 
 @returns exact package identity or unavailable sentinel.
 */
function packageIdentityAtRoot(
  root: string,
): InstalledPackageIdentity | typeof INSTALLED_PACKAGE_UNAVAILABLE {
  /**
   Package manifest path.
   */
  const manifestPath = join(
    root,
    'package.json',
  );
  try {
    /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor resolves exact package identity before effect analysis. */
    /**
     Parsed exact package manifest.
     */
    const parsed: unknown = JSON.parse(readFileSync(
      manifestPath,
      'utf8',
    ),);
    /* oxlint-enable no-restricted-syntax/no-sync */
    if (!isManifest(parsed,))
      return INSTALLED_PACKAGE_UNAVAILABLE;
    /**
     Semantic-version major component.
     */
    const [majorText,] = parsed.version
      .split('.',);
    if (majorText === undefined)
      return INSTALLED_PACKAGE_UNAVAILABLE;
    /**
     Numeric exact major for provenance and diagnostics.
     */
    const major = Number(majorText,);
    if ((!Number.isInteger(major,)) || (major < 0))
      return INSTALLED_PACKAGE_UNAVAILABLE;
    /**
     Validated exact package identity.
     */
    const identity: InstalledPackageIdentity = {
      root,
      name: parsed.name,
      version: parsed.version,
      major,
      manifest: parsed,
    };
    return identity;
  }
  catch (error) {
    l.debug(`package identity unavailable for ${manifestPath}: ${String(error,)}`,);
    return INSTALLED_PACKAGE_UNAVAILABLE;
  }
}

/**
 Resolves exact package identity for declaration or implementation file.
 
 @param fileName - Source path whose owning package is required.
 
 @returns exact package identity or unavailable sentinel.
 
 @example
 ```ts
 const identity = installedPackageForFile('/repo/node_modules/pkg/index.d.ts');
 ```
 */
export function installedPackageForFile(
  fileName: string,
): InstalledPackageIdentity | typeof INSTALLED_PACKAGE_UNAVAILABLE {
  /**
   Installed root or workspace fallback.
   */
  const installedRoot = nodeModulesPackageRoot(fileName,);
  /**
   Final package root selected by path class.
   */
  const root = installedRoot === INSTALLED_PACKAGE_UNAVAILABLE
    ? workspacePackageRoot(fileName,)
    : installedRoot;
  return root === INSTALLED_PACKAGE_UNAVAILABLE
    ? INSTALLED_PACKAGE_UNAVAILABLE
    : packageIdentityAtRoot(root,);
}
