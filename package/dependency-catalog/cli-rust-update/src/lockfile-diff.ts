/**
 Registry versions a set of lockfiles holds, and which ones an update added.

 The release-age and trust checks run only on versions an update newly
 locked, so a run makes one crates.io request per moved crate instead of one
 per locked crate.

 @module
 */

import {
  isRegistryPackage,
  type LockedPackage,
  parseCargoLock,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

//region Types

/**
 One registry crate version.

 @example
 ```ts
 const locked: LockedVersion = { name: 'anyhow', version: '1.0.104', };
 ```
 */
export type LockedVersion = {
  readonly name: string;
  readonly version: string;
};

//endregion Types

//region Versions

/**
 Joined `name@version` key.

 @param locked - crate version

 @returns key such as `anyhow@1.0.104`

 @example
 ```ts
 versionKey({ name: 'anyhow', version: '1.0.104', }); // 'anyhow@1.0.104'
 ```
 */
export function versionKey(locked: LockedVersion,): string {
  return `${locked.name}@${locked.version}`;
}

/**
 Registry versions of several lockfiles, deduplicated and sorted by name then version text.

 @param sources - `Cargo.lock` texts

 @returns distinct registry versions

 @throws TomlEditError when a lockfile is not valid TOML

 @example
 ```ts
 registryVersions(['[[package]]\nname = "a"\nversion = "1.0.0"\nsource = "registry+x"\n',]); // [{ name: 'a', version: '1.0.0', }]
 ```
 */
export function registryVersions(sources: readonly string[],): readonly LockedVersion[] {
  /**
   Versions by key, for deduplication.
   */
  const byKey = new Map<string, LockedVersion>();
  for (const source of sources) {
    for (const locked of parseCargoLock(source,)
      .filter(function isRegistry(entry: LockedPackage,): boolean {
        return isRegistryPackage(entry,);
      },)) {
      /**
       Version without the fields the checks do not read.
       */
      const version = toLockedVersion(locked,);
      byKey.set(
        versionKey(version,),
        version,
      );
    }
  }
  return [...byKey.values(),].toSorted(function byNameThenVersion(
    left: LockedVersion,
    right: LockedVersion,
  ): number {
    return versionKey(left,)
      .localeCompare(versionKey(right,),);
  },);
}

/**
 Name and version of a locked package.

 @param locked - locked package

 @returns crate version

 @example
 ```ts
 toLockedVersion(locked);
 ```
 */
function toLockedVersion(locked: LockedPackage,): LockedVersion {
  return {
    name: locked.name,
    version: locked.version,
  };
}

/**
 Registry versions present after an update that no lockfile held before it.

 @param before - lockfile texts before the update

 @param after - lockfile texts after the update

 @returns newly locked versions, sorted

 @throws TomlEditError when a lockfile is not valid TOML

 @example
 ```ts
 addedVersions({ before: [oldLock,], after: [newLock,], });
 ```
 */
export function addedVersions({
  before,
  after,
}: {
  readonly before: readonly string[];
  readonly after: readonly string[]
},): readonly LockedVersion[] {
  /**
   Keys locked before the update.
   */
  const previous = new Set(registryVersions(before,)
    .map(versionKey,),);
  return registryVersions(after,)
    .filter(function isNew(locked: LockedVersion,): boolean {
      return !previous.has(versionKey(locked,),);
    },);
}

//endregion Versions
