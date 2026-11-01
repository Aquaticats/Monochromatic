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
 Discovers all workspace package directories under `package/{category}/{pkg}`.
 Cached per `monorepoRoot` after first call.
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @returns array of absolute paths to workspace package directories
 
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
  const cached = workspaceRootsCache.get(monorepoRoot,);
  if (cached !== undefined)
    return cached;

  /**
   Top-level `package/` directory; each entry is a category subdir holding individual workspace packages.
   */
  const packagesDir = join(
    monorepoRoot,
    'package',
  );

  try {
    /**
     Direct children of `package/`, expected to be category directories (e.g. `module`, `dev-script`).
     */
    const categories = await readdir(
      packagesDir,
      { withFileTypes: true, },
    );
    /**
     Workspace package roots discovered per package category.
     */
    const rootsByCategory = await Promise.all(categories
      .filter(function isDirectory(cat: Readonly<(typeof categories)[number]>,): boolean {
        return cat.isDirectory();
      },)
      .map(async function readPackageCategory(
        cat: Readonly<(typeof categories)[number]>,
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
        const pkgs = await readdir(
          catPath,
          { withFileTypes: true, },
        );
        return pkgs
          .filter(function isPackageDirectory(pkg: Readonly<(typeof pkgs)[number]>,): boolean {
            return pkg.isDirectory();
          },)
          .map(function toPackageRoot(pkg: Readonly<(typeof pkgs)[number]>,): string {
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
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    workspaceRootsCache.set(
      monorepoRoot,
      [],
    );
    return [];
  }
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
