/**
 Direct-dependency versions of a crate, read from its `Cargo.lock`.

 Cargo builds from the lockfile, so for Cargo the lockfile is the resolved
 version source (unlike the pnpm side, where `node_modules` can be deleted
 while `pnpm-lock.yaml` stays).

 @module
 */

import {
  isRegistryPackage,
  type LockedPackage,
  parseCargoLock,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

//region Parse

/**
 Versions a crate's own package depends on directly, by package name.

 The root package is the one named `packageName` with no `source` (a path
 package). Its `dependencies` list names each direct dependency as `name`
 when the lockfile holds one version of it, or `name version` when several.

 @param source - `Cargo.lock` text

 @param packageName - the crate's `package.name`

 @returns package name to the locked versions the crate uses directly

 @throws TomlEditError when the lockfile is not valid TOML

 @example
 ```ts
 directDependencyVersions({ source, packageName: 'forbidden-strings', }).get('anyhow'); // ['1.0.102']
 ```
 */
export function directDependencyVersions(
  {
    source,
    packageName,
  }: {
    readonly source: string;
    readonly packageName: string
  },
): ReadonlyMap<string, readonly string[]> {
  /**
   Every locked package.
   */
  const packages = parseCargoLock(source,);
  /**
   The crate itself.
   */
  const rootPackage = packages.find(function isRoot(entry: LockedPackage,): boolean {
    return (entry.name === packageName) && (entry.source === '');
  },);
  /**
   Versions found per name.
   */
  const versions = new Map<string, string[]>();
  if (rootPackage === undefined)
    return versions;
  for (const dependency of rootPackage.dependencies) {
    /**
     Name and optional version of the dependency string.
     */
    const [name = '', version = '',] = dependency.split(' ',);
    /**
     Locked registry versions of that name.
     */
    const candidates = packages.filter(function isNamed(entry: LockedPackage,): boolean {
      return (entry.name === name) && isRegistryPackage(entry,);
    },);
    /**
     The version this crate uses: the stated one, or the only one.
     */
    const resolved = version !== '' ? version : (candidates.length === 1 ? candidates[0]
      ?.version
      ?? '' : '');
    if (resolved === '')
      continue;
    versions.set(
      name,
      [
        ...(versions.get(name,) ?? []),
        resolved,
      ],
    );
  }
  return versions;
}

//endregion Parse
