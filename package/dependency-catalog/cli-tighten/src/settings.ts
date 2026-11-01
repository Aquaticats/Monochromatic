/**
 Layout-affecting pnpm settings, read through pnpm's own effective config.
 
 pnpm resolves settings from the command line, environment variables, the
 project `pnpm-workspace.yaml`, and the global `~/.config/pnpm/config.yaml`
 (only auth and registry come from `.npmrc`). Parsing `pnpm-workspace.yaml`
 alone would miss a `modulesDir` set by env or global config, so this asks
 pnpm for the merged value via `pnpm config get`.
 
 Of the layout settings, only `modulesDir` changes where the resolver looks:
 it renames the per-importer modules directory (default `node_modules`), and
 the virtual store moves with it. `virtualStoreDir`, `enableGlobalVirtualStore`,
 and `storeDir` relocate the virtual or content-addressable store, which the
 resolver follows transparently because it reads `<modulesDir>/<name>` through
 whatever symlink the importer holds, wherever that store lives (verified
 against real installs, including the global store). So only `modulesDir` is
 queried here.
 */

import spawn from 'nano-spawn';

//region Workspace settings

/**
 Default per-importer modules directory when nothing overrides it.
 */
const DEFAULT_MODULES_DIR = 'node_modules';

/**
 Literal `pnpm config get` prints for a setting that is not configured anywhere.
 */
const UNSET_VALUE = 'undefined';

/**
 Reads the effective `modulesDir` setting via `pnpm config get modules-dir`,
 run in the monorepo so project, env, and global config all apply. Falls back
 to `node_modules` when the setting is unset or pnpm cannot be queried.
 
 @param monorepoRoot - absolute path to run `pnpm config get` in
 
 @returns effective modules directory name, or `node_modules`
 
 @example
 ```ts
 await readModulesDir("/repo") // "node_modules"
 ```
 */
export async function readModulesDir(monorepoRoot: string,): Promise<string> {
  try {
    /**
     pnpm's merged value for `modules-dir`; `undefined`/empty when unconfigured.
     */
    const result = await spawn(
      'pnpm',
      [
        'config',
        'get',
        'modules-dir',
      ],
      { cwd: monorepoRoot, },
    );
    /**
     Trimmed config value; pnpm prints the literal `undefined` for an unset key.
     */
    const value = result.stdout
      .trim();
    if ((value.length
      === 0) || (value === UNSET_VALUE))
      return DEFAULT_MODULES_DIR;
    return value;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // pnpm not on PATH or config read failed; the default modules directory is the safe assumption.
    console.warn('Could not read `pnpm config get modules-dir`; assuming node_modules.',);
    return DEFAULT_MODULES_DIR;
  }
}

//endregion Workspace settings
