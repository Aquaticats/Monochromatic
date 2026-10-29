/**
 Direct-dependency versions of a crate, read from its `Cargo.lock`.

 Cargo builds from the lockfile, so for Cargo the lockfile is the resolved
 version source (unlike the pnpm side, where `node_modules` can be deleted
 while `pnpm-lock.yaml` stays).

 @module
 */

import {
  parseTomlEdit,
  tomlGetValue,
} from '@monochromatic-dev/module-toml-edit/ts';

//region Types

/**
 One `[[package]]` entry of a lockfile.
 */
type LockedPackage = {
  readonly name: string;
  readonly version: string;
  readonly source: string;
  readonly dependencies: readonly string[];
};

//endregion Types

//region Parse

/**
 Whether `value` is a plain table.

 @param value - materialized TOML value

 @returns whether it is a non-array object

 @example
 ```ts
 isTable({}); // true
 ```
 */
function isTable(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads every `[[package]]` entry.

 @param source - `Cargo.lock` text

 @returns locked packages in file order

 @throws TomlEditError when the lockfile is not valid TOML

 @example
 ```ts
 lockedPackages(source);
 ```
 */
function lockedPackages(source: string,): readonly LockedPackage[] {
  /**
   Whole lockfile as plain values.
   */
  const root = tomlGetValue({
    edit: parseTomlEdit({ source, },),
    path: [],
  },);
  /**
   Raw package entries.
   */
  const raw = isTable(root,) && Array.isArray(root.package,) ? root.package : [];
  /**
   Packages with the fields this tool reads.
   */
  const packages: LockedPackage[] = [];
  for (const entry of raw) {
    if ((!isTable(entry,)) || ((typeof entry.name) !== 'string')
      || ((typeof entry.version) !== 'string'))
      continue;
    /**
     Dependency strings such as `anyhow` or `syn 2.0.110`.
     */
    const dependencies: string[] = [];
    if (Array.isArray(entry.dependencies,)) {
      for (const dependency of entry.dependencies) {
        if ((typeof dependency) === 'string')
          dependencies.push(dependency,);
      }
    }
    packages.push({
      name: entry.name,
      version: entry.version,
      source: (typeof entry.source) === 'string' ? entry.source : '',
      dependencies,
    },);
  }
  return packages;
}

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
  const packages = lockedPackages(source,);
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
      return (entry.name === name)
        && entry.source
          .startsWith('registry+',);
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
