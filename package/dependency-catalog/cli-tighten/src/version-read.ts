/**
 Low-level version reading for catalog-tighten.
 
 Reads installed package versions from `package.json` files on disk. The tool
 resolves the actual installed version, never the lockfile (a lockfile
 survives a deleted `node_modules` and would report uninstalled versions), so
 every read here targets a real `package.json` reachable through the install
 layout.
 */

import {
  readFile,
} from 'node:fs/promises';

//region Version reading

/**
 Sentinel returned by {@link readVersionFromPackageJson} when a manifest is
 missing, unreadable, or has no `version` field. A `unique symbol`; callers
 narrow with `=== NO_MANIFEST_VERSION`.
 */
export const NO_MANIFEST_VERSION: unique symbol = Symbol('catalog-tighten/no-manifest-version',);

/**
 Sentinel returned by {@link readInstalledVersion} when no installed version is
 found through any layout. A `unique symbol`; callers narrow with
 `=== NO_INSTALLED_VERSION`.
 */
export const NO_INSTALLED_VERSION: unique symbol = Symbol('catalog-tighten/no-installed-version',);

/**
 Reads the `version` field from a `package.json` file path. Following a
 symlink (the pnpm isolated store links `node_modules/<name>` into `.pnpm`)
 and bypassing the package's `exports` map, since this reads the file
 directly rather than resolving a subpath.
 
 @param pkgJsonPath - absolute path to a package.json file
 
 @returns version string, or {@link NO_MANIFEST_VERSION} if file does not exist or has no version
 
 @example
 ```ts
 await readVersionFromPackageJson("/repo/node_modules/oxlint/package.json") // "1.71.0"
 ```
 */
export async function readVersionFromPackageJson(pkgJsonPath: string,): Promise<string | typeof NO_MANIFEST_VERSION> {
  try {
    /**
     Raw `package.json` text read from disk.
     */
    const content = await readFile(
      pkgJsonPath,
      'utf8',
    );
    /**
     Parsed manifest narrowed to the only field this helper consults: `version`.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON structure from package.json is well-known
    const parsed = JSON.parse(content,) as { version?: string; };
    return parsed.version ?? NO_MANIFEST_VERSION;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return NO_MANIFEST_VERSION;
  }
}

//endregion Version reading
