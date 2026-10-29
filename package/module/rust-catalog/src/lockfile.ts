/**
 Reads the `[[package]]` entries of a `Cargo.lock`.

 Cargo builds from the lockfile, so it is the resolved-version source for
 every Rust catalog tool: `catalog-tighten` reads direct-dependency versions
 from it, and `rust-catalog-update` diffs it before and after an update.

 @module
 */

import {
  parseTomlEdit,
  tomlGetValue,
} from '@monochromatic-dev/module-toml-edit/ts';

//region Types

/**
 One `[[package]]` entry of a lockfile.

 @example
 ```ts
 const locked: LockedPackage = { name: 'anyhow', version: '1.0.104', source: 'registry+https://github.com/rust-lang/crates.io-index', dependencies: [], };
 ```
 */
export type LockedPackage = {
  readonly name: string;
  readonly version: string;
  /**
   `registry+...` or `git+...` URL; empty for a path package.
   */
  readonly source: string;
  /**
   Dependency strings such as `anyhow` or `syn 2.0.110`.
   */
  readonly dependencies: readonly string[];
};

//endregion Types

//region Parse

/**
 Prefix of a crates.io or other registry `source`.
 */
const REGISTRY_SOURCE_PREFIX = 'registry+';

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
 Reads every `[[package]]` entry that has a string name and version.

 @param source - `Cargo.lock` text

 @returns locked packages in file order

 @throws TomlEditError when the lockfile is not valid TOML

 @example
 ```ts
 parseCargoLock('[[package]]\nname = "anyhow"\nversion = "1.0.104"\n'); // [{ name: 'anyhow', ... }]
 ```
 */
export function parseCargoLock(source: string,): readonly LockedPackage[] {
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
   Packages with the fields the catalog tools read.
   */
  const packages: LockedPackage[] = [];
  for (const entry of raw) {
    if ((!isTable(entry,)) || ((typeof entry.name) !== 'string')
      || ((typeof entry.version) !== 'string'))
      continue;
    /**
     Dependency strings.
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
 Whether a locked package comes from a registry (crates.io or another index).

 @param locked - locked package

 @returns whether its `source` is a registry URL

 @example
 ```ts
 isRegistryPackage({ name: 'a', version: '1.0.0', source: 'registry+https://github.com/rust-lang/crates.io-index', dependencies: [], }); // true
 ```
 */
export function isRegistryPackage(locked: LockedPackage,): boolean {
  return locked.source.startsWith(REGISTRY_SOURCE_PREFIX,);
}

//endregion Parse
