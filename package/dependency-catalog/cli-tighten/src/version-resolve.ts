/**
 Version resolution for catalog-tighten.
 
 Resolves the installed version of a catalog package from the actual on-disk
 install layout. Reads `node_modules/<name>/package.json` directly from the
 monorepo root and every workspace package, following the pnpm symlink farm
 and bypassing exports-gated `require.resolve` (which throws
 `ERR_PACKAGE_PATH_NOT_EXPORTED` for packages whose `exports` map omits
 `./package.json`, the failure that left most catalog entries unresolved).
 Also resolves `npm:` alias names.
 */

import type {
  Dirent,
} from 'node:fs';
import {
  readdir,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  isValidPackageName,
} from '@monochromatic-dev/dependency-catalog-core-pnpm/ts';

import {
  isAbsentPathError,
} from './absent-path-error.ts';

import {
  readVersionFromPnp,
} from './version-pnp.ts';
import {
  type NO_INSTALLED_VERSION,
  NO_MANIFEST_VERSION,
  readVersionFromPackageJson,
} from './version-read.ts';

//region Version resolution

/**
 Resolves candidate npm package names to look up in node_modules.
 Bun and pnpm install `npm:` aliased packages under the **key** name
 (e.g. `zod`), not the registry target (e.g. `@jsr/zod__zod`). Returns the key
 first, then the alias target as fallback.
 
 @param catalogKey - package name key in catalog, e.g. `"zod"`
 
 @param catalogValue - raw catalog value, e.g. `"npm:@jsr/zod__zod@>=4.1.8"`
 
 @returns ordered list of npm names to try resolving
 
 @example
 ```ts
 resolveNpmNames({ catalogKey: "zod", catalogValue: "npm:\@jsr/zod__zod\@>=4.1.8" }) // ["zod", "\@jsr/zod__zod"]
 resolveNpmNames({ catalogKey: "oxlint", catalogValue: ">=0.20.0" }) // ["oxlint"]
 ```
 */
export function resolveNpmNames(
  {
    catalogKey,
    catalogValue,
  }: {
    readonly catalogKey: string;
    readonly catalogValue: string;
  },
): string[] {
  /**
   Length of the `npm:` prefix
   */
  const NPM_PREFIX_LENGTH = 4;
  if (!isValidPackageName(catalogKey,))
    return [];
  if (catalogValue.startsWith('npm:',)) {
    /**
     Catalog value with the `npm:` prefix stripped, leaving `<target>@<range>` or `<target>` for further parsing.
     */
    const withoutNpm = catalogValue.slice(NPM_PREFIX_LENGTH,);
    // Find the last @ that isn't position 0 (scoped package)
    /**
     Index of the version separator `@`; skipped at position 0 so scoped names like `@scope/name` survive intact.
     */
    const lastAt = withoutNpm.lastIndexOf('@',);
    /**
     Registry-target name without the version suffix; the actual install lives here when the alias is honoured.
     */
    const aliasTarget = lastAt > 0
      ? withoutNpm.slice(
        0,
        lastAt,
      )
      : withoutNpm;
    // Key first (installed under alias name), then registry target as fallback.
    // Invalid targets are never joined into a filesystem path.
    if (!isValidPackageName(aliasTarget,))
      return [catalogKey,];
    if (aliasTarget !== catalogKey) {
      return [
        catalogKey,
        aliasTarget,
      ];
    }
    return [catalogKey,];
  }
  return [catalogKey,];
}

/**
 Cached workspace root directories keyed by monorepo root path.
 */
const workspaceRootsCache = new Map<string, string[]>();

/**
 Reads one directory's entries, reporting an absent directory as empty.
 
 A directory that is not there simply holds no packages, which is the
 ordinary case in a repo without a `package/` layout. Any other failure, such
 as a directory this process may not read, is thrown with the directory named:
 an empty list would make every catalog entry look uninstalled and every
 package look undeclared, and the run cannot tell that apart from the truth.
 
 @param dir - absolute path to the directory to list
 
 @returns the directory's entries, or an empty list when it does not exist
 
 @throws Error wrapping the read failure for every cause other than a missing
 or non-directory path
 
 @example
 ```ts
 await readDirectoryEntries("/repo/package") // [Dirent { name: "logger" }, ...]
 ```
 */
async function readDirectoryEntries(dir: string,): Promise<Dirent[]> {
  try {
    return await readdir(
      dir,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    if (isAbsentPathError(error,))
      return [];

    throw new Error(
      `cannot read directory ${dir}: a failed scan would report every catalog entry as missing, so the run stops instead`,
      { cause: error, },
    );
  }
}

/**
 Discovers all workspace package directories under `package/{category}/{pkg}`.
 Cached per `monorepoRoot` after first call; a failed scan is never cached.
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @returns array of absolute paths to workspace package directories
 
 @throws Error when `package/` or one of its category directories exists but
 cannot be read, so a broken scan is never reported as an empty workspace
 
 @example
 ```ts
 await discoverWorkspaceRoots("/home/user/Monochromatic")
 // ["/home/user/Monochromatic/package/logger/core", ...]
 ```
 */
export async function discoverWorkspaceRoots(monorepoRoot: string,): Promise<string[]> {
  /**
   Previously-computed roots for this `monorepoRoot`, if any; short-circuits the directory scan.
   */
  const cached = workspaceRootsCache.get(
    monorepoRoot,
  );
  if (cached !== undefined)
    return cached;

  /**
   Top-level `package/` directory; each entry is a category subdir holding individual workspace packages.
   */
  const packagesDir = join(
    monorepoRoot,
    'package',
  );

  /**
   Direct children of `package/`, expected to be category directories (e.g. `module`, `dev-script`).
   */
  const categories = await readDirectoryEntries(packagesDir,);
  /**
   Workspace package roots discovered per package category.
   */
  const rootsByCategory = await Promise.all(categories
    .filter(function isDirectory(cat: Readonly<Dirent>,): boolean {
      return cat.isDirectory();
    },)
    .map(async function readPackageCategory(
      cat: Readonly<Dirent>,
    ): Promise<readonly string[]> {
      /**
       Absolute path to one category directory, scanned for the actual package folders.
       */
      const catPath = join(
        packagesDir,
        cat.name,
      );
      /**
       Individual workspace packages nested under the category; each becomes one entry in `roots`.
       */
      const pkgs = await readDirectoryEntries(catPath,);
      return pkgs
        .filter(function isPackageDirectory(pkg: Readonly<Dirent>,): boolean {
          return pkg.isDirectory();
        },)
        .map(function toPackageRoot(pkg: Readonly<Dirent>,): string {
          return join(
            catPath,
            pkg.name,
          );
        },);
    },),);
  /**
   Flattened workspace package roots cached for the current monorepo.
   */
  const roots = rootsByCategory.flat();
  workspaceRootsCache.set(
    monorepoRoot,
    roots,
  );
  return roots;
}

/**
 Reads the installed version of a package from the on-disk install layout.
 
 Reads `node_modules/<name>/package.json` directly from the monorepo root and
 every workspace package, following the pnpm symlink farm and bypassing the
 package's `exports` map. Returns the first version found: a catalog pins one
 version across all importers, so the first on-disk hit is the active version.
 Returns {@link NO_INSTALLED_VERSION} when the package is installed nowhere
 (for example a dependency of a paused package that was never installed),
 which the caller reports as MISS and skips.
 
 @param npmName - npm package name to look up
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @param modulesDir - per-importer modules directory name (the `modulesDir` setting; usually `node_modules`)
 
 @returns installed version string, or {@link NO_INSTALLED_VERSION} if not found
 
 @example
 ```ts
 await readInstalledVersion({ npmName: "oxlint", monorepoRoot: "/home/user/Monochromatic", modulesDir: "node_modules" }) // "1.71.0"
 ```
 */
export async function readInstalledVersion(
  {
    npmName,
    monorepoRoot,
    modulesDir,
  }: {
    readonly npmName: string;
    readonly monorepoRoot: string;
    readonly modulesDir: string;
  },
): Promise<string | typeof NO_INSTALLED_VERSION> {
  /**
   Workspace package directories; each may hold a per-importer `<modulesDir>/<name>` symlink.
   */
  const workspaceRoots = await discoverWorkspaceRoots(monorepoRoot,);
  /**
   Importer directories to probe: the monorepo root (hoisted deps) plus every workspace package.
   */
  const candidateDirs = [
    monorepoRoot,
    ...workspaceRoots,
  ];
  /**
   Version read from each candidate's `<modulesDir>/<name>/package.json`; `NO_MANIFEST_VERSION` when absent.
   */
  const versions = await Promise.all(candidateDirs.map(async function readCandidate(
    dir,
  ): Promise<string | typeof NO_MANIFEST_VERSION> {
    return await readVersionFromPackageJson(join(
      dir,
      modulesDir,
      npmName,
      'package.json',
    ),);
  },),);
  /**
   First candidate that resolved to a real installed version.
   */
  const found = versions.find(function hasVersion(
    version,
  ): version is string {
    return version !== NO_MANIFEST_VERSION;
  },);
  if (found !== undefined)
    return found;

  // No node_modules entry resolved; fall back to the PnP layout (no node_modules at all).
  return await readVersionFromPnp({
    npmName,
    monorepoRoot,
  },);
}

//endregion Version resolution
