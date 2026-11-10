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
  isAbsentPathError,
} from './absent-path-error.ts';
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
 field of one `package.json`. A missing manifest contributes no names, so a
 stripped-down importer simply declares nothing.
 
 A manifest that exists but cannot be read or parsed is not the same thing and
 is thrown: folding it into "declares nothing" would let `UNDCL` accuse a
 package the importer does declare, which this module exists to prevent.
 
 @param pkgJsonPath - absolute path to a `package.json`
 
 @returns set of declared dependency names
 
 @throws Error when the manifest exists but cannot be read or is not valid JSON
 
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
    const parsed = parseDeclaredDeps({
      pkgJsonPath,
      content,
    },);
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

    // Only a genuinely absent manifest declares nothing; every other failure reaches the caller.
    if (isAbsentPathError(error,))
      return new Set();

    throw error;
  }
}

/**
 Parses manifest text into the dependency-field maps the check reads.
 
 @param pkgJsonPath - path the text came from, named in the failure message
 
 @param content - raw `package.json` text
 
 @returns manifest narrowed to its dependency fields
 
 @throws Error when the text is not valid JSON
 
 @example
 ```ts
 parseDeclaredDeps({ pkgJsonPath: '/repo/package.json', content: '{"dependencies":{"picomatch":"catalog:"}}', }) // { dependencies: { picomatch: 'catalog:' } }
 ```
 */
function parseDeclaredDeps(
  {
    pkgJsonPath,
    content,
  }: {
    readonly pkgJsonPath: string;
    readonly content: string;
  },
): ManifestDeps {
  try {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON structure from package.json is well-known
    return JSON.parse(content,) as ManifestDeps;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    throw new Error(
      `importer manifest ${pkgJsonPath} is not valid JSON: it cannot be read as declaring nothing, because that would label a package it declares as undeclared`,
      { cause: error, },
    );
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
 
 @throws Error when an importer manifest exists but cannot be read or parsed,
 or a workspace directory cannot be scanned
 
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
