/**
 Direct-declaration check for catalog-tighten's MISS classification.
 
 The store probe alone cannot tell "present only as a transitive dependency,
 declared by nobody" (a genuine `UNDCL`) from "declared directly, but the
 install's resolution path is broken" (for example pnpm `symlink: false` with a
 removed `.pnp.cjs`, where no importer symlink exists yet the package is still
 in the `.pnpm` store). Both look identical to a symlink probe. `UNDCL`'s claim
 ("no live package declares it directly") is therefore verified here against
 the importer `package.json` files, so the label never accuses a declared
 package of being undeclared.
 */

import {
  readFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  discoverWorkspaceRoots,
} from './version-resolve.ts';

//region Declaration check

/**
 Dependency-field maps a `package.json` may carry; every field is optional and
 only its keys (the declared names) are consulted.
 */
type ManifestDeps = {
  /**
   Runtime dependencies.
   */
  readonly dependencies?: Record<string, string>;
  /**
   Development-only dependencies.
   */
  readonly devDependencies?: Record<string, string>;
  /**
   Optional dependencies.
   */
  readonly optionalDependencies?: Record<string, string>;
  /**
   Peer dependencies.
   */
  readonly peerDependencies?: Record<string, string>;
};

/**
 Reads the union of direct-dependency names declared across every dependency
 field of one `package.json`. A missing or unreadable manifest contributes no
 names, so a stripped-down importer simply declares nothing.
 
 @param pkgJsonPath - absolute path to a `package.json`
 
 @returns set of declared dependency names
 
 @example
 ```ts
 await readDeclaredDeps("/repo/packages/grp/consumer-a/package.json") // Set { "picomatch" }
 ```
 */
async function readDeclaredDeps(pkgJsonPath: string,): Promise<ReadonlySet<string>> {
  try {
    /**
     Raw `package.json` text.
     */
    const content = await readFile(
      pkgJsonPath,
      'utf8',
    );
    /**
     Manifest narrowed to the dependency-field maps whose keys name direct dependencies.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON structure from package.json is well-known
    const parsed = JSON.parse(content,) as ManifestDeps;
    return new Set([
      ...Object.keys(parsed.dependencies
        ?? {},),
      ...Object.keys(parsed.devDependencies
        ?? {},),
      ...Object.keys(parsed.optionalDependencies
        ?? {},),
      ...Object.keys(parsed.peerDependencies
        ?? {},),
    ],);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return new Set();
  }
}

/**
 Reports whether any live importer (the monorepo root or a `package/*\/*`
 package) declares one of `npmNames` as a direct dependency in its
 `package.json`. Used to confirm `UNDCL` before emitting it: a package a live
 importer declares is never labelled undeclared, even when its resolution path
 is broken.
 
 @param npmNames - candidate npm names (catalog key first, alias target next)
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @returns whether a live importer declares any candidate directly
 
 @example
 ```ts
 await isDeclaredByLiveImporter({ npmNames: ["picomatch"], monorepoRoot: "/repo" }) // true
 ```
 */
export async function isDeclaredByLiveImporter(
  {
    npmNames,
    monorepoRoot,
  }: {
    readonly npmNames: readonly string[];
    readonly monorepoRoot: string;
  },
): Promise<boolean> {
  /**
   Importer directories to inspect: the monorepo root plus every workspace package.
   */
  const importerDirs = [
    monorepoRoot,
    ...await discoverWorkspaceRoots(monorepoRoot,),
  ];
  /**
   Declared-dependency name sets, one per importer manifest.
   */
  const declaredSets = await Promise.all(importerDirs.map(async function readImporter(
    dir,
  ): Promise<ReadonlySet<string>> {
    return await readDeclaredDeps(join(
      dir,
      'package.json',
    ),);
  },),);
  return declaredSets.some(function declaresAny(declared,): boolean {
    return npmNames.some(function isDeclared(name,): boolean {
      return declared.has(name,);
    },);
  },);
}

//endregion Declaration check
