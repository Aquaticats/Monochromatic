/**
 PnP-layout version reader for catalog-tighten.
 
 Under pnpm's `nodeLinker: pnp` there is no `node_modules/<name>` to read, so
 the installed version comes from the `.pnp.cjs` runtime API instead. pnpm
 inlines the data into `.pnp.cjs` (no separate `.pnp.data.json` as Yarn emits),
 exposing the standard Yarn PnP API: `getAllLocators()` lists installed
 `{ name, reference }` pairs and `getPackageInformation(locator)` gives each a
 `packageLocation` directory whose `package.json` carries the real version.
 Requiring `.pnp.cjs` returns its API (`module.exports`) without running the
 loader `setup()`, so it does not patch this process's module resolution
 (verified against a real pnpm pnp install).
 */

import {
  createRequire,
} from 'node:module';
import {
  join,
} from 'node:path';

import {
  NO_INSTALLED_VERSION,
  NO_MANIFEST_VERSION,
  readVersionFromPackageJson,
} from './version-read.ts';

//region PnP resolution

/**
 One installed package identity from the PnP graph.
 */
type PnpLocator = {
  /**
   Package name, e.g. `picomatch`.
   */
  readonly name: string;
  /**
   Opaque install reference (often the version) used to fetch package information.
   */
  readonly reference: string;
};

/**
 Per-package information from the PnP API; only the install directory is read here.
 */
type PnpPackageInformation = {
  /**
   Absolute directory holding the package's real files, including its `package.json`.
   */
  readonly packageLocation: string;
};

/**
 Minimal slice of the Yarn-style PnP API that `.pnp.cjs` exports.
 */
type PnpApi = {
  /**
   Lists every installed package locator in the dependency graph.
   */
  readonly getAllLocators: () => readonly PnpLocator[];
  /**
   Resolves a locator to its on-disk information, or `null` for an unknown locator.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors the external Yarn PnP API, which returns null for an unknown locator
  readonly getPackageInformation: (locator: PnpLocator,) => PnpPackageInformation | null;
};

/**
 Sentinel returned by {@link loadPnpApi} when no `.pnp.cjs` is present or it
 cannot be loaded. A `unique symbol`; callers narrow with `=== NO_PNP`.
 */
const NO_PNP: unique symbol = Symbol('catalog-tighten/no pnp manifest',);

/**
 Loads the PnP API from `<monorepoRoot>/.pnp.cjs`. Returns {@link NO_PNP} when
 the file is absent or unreadable (the common non-PnP case).
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @returns the PnP API, or {@link NO_PNP}
 
 @example
 ```ts
 const api = loadPnpApi("/repo");
 ```
 */
function loadPnpApi(monorepoRoot: string,): PnpApi | typeof NO_PNP {
  /**
   CJS require anchored at the monorepo root, used to load the repo-local `.pnp.cjs`.
   */
  const require = createRequire(join(
    monorepoRoot,
    'noop.cjs',
  ),);
  try {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- .pnp.cjs is an untyped repo-local CJS module exposing the Yarn PnP API
    return require(join(
      monorepoRoot,
      '.pnp.cjs',
    ),) as PnpApi;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return NO_PNP;
  }
}

/**
 Reads the installed version of `npmName` from a PnP install layout.
 
 Loads the PnP API, finds every locator matching the name, and reads the
 `version` from each locator's `packageLocation/package.json`, returning the
 first found. A catalog pins one version, so the first match is the active
 version. Returns {@link NO_INSTALLED_VERSION} when there is no PnP manifest
 or the name is not in the graph.
 
 @param npmName - npm package name to look up
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @returns installed version string, or {@link NO_INSTALLED_VERSION}
 
 @example
 ```ts
 await readVersionFromPnp({ npmName: "picomatch", monorepoRoot: "/repo" }) // "4.0.4"
 ```
 */
export async function readVersionFromPnp(
  {
    npmName,
    monorepoRoot,
  }: {
    readonly npmName: string;
    readonly monorepoRoot: string;
  },
): Promise<string | typeof NO_INSTALLED_VERSION> {
  /**
   Loaded PnP API, or the no-manifest sentinel when this is not a PnP layout.
   */
  const api = loadPnpApi(monorepoRoot,);
  if (api === NO_PNP)
    return NO_INSTALLED_VERSION;

  /**
   Every locator in the graph whose name matches the requested package.
   */
  const matches = api.getAllLocators()
    .filter(function matchesName(locator,): boolean {
      return locator.name === npmName;
    },);
  /**
   Version read from each matching locator's install directory.
   */
  const versions = await Promise.all(matches.map(async function readLocatorVersion(
    locator,
  ): Promise<string | typeof NO_MANIFEST_VERSION> {
    /**
     On-disk information for this locator; `null` when the API does not know it.
     */
    const info = api.getPackageInformation(locator,);
    if (info === null)
      return NO_MANIFEST_VERSION;
    return await readVersionFromPackageJson(join(
      info.packageLocation,
      'package.json',
    ),);
  },),);
  /**
   First locator that resolved to a real installed version.
   */
  const found = versions.find(function hasVersion(
    version,
  ): version is string {
    return version !== NO_MANIFEST_VERSION;
  },);
  return found ?? NO_INSTALLED_VERSION;
}

//endregion PnP resolution
