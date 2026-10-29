import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { lazyOnce, } from '../lazy-once.ts';
import { l, } from '../logger.ts';
import {
  binaryExists,
  canProvide,
  detectManager,
  installPackage,
  NO_PACKAGE_MANAGER,
} from './manager.ts';
import type {
  PackageEntry,
  PackageManager,
} from './types.ts';

//region Package index

/**
 Registered package entries awaiting indexing.
 Populated by consumers who import a data file and pass its entries
 to {@link registerPackages} before calling {@link ensurePackage}.
 */
const registered: PackageEntry[] = [];

/**
 Builds the binary-name lookup index from registered entries.
 Called lazily on first {@link ensurePackage} invocation.
 
 @returns Map from binary name to package entry
 */
function buildIndex(): ReadonlyMap<string, PackageEntry> {
  /**
   Lookup map from binary name to package entry; returned read-only after the loop.
   */
  const map = new Map<string, PackageEntry>();
  for (const entry of registered) {
    map.set(
      entry.bin,
      entry,
    );
  }
  return map;
}

/**
 Lazily-built binary lookup index, cleared by {@link registerPackages}.
 */
const packageIndex = lazyOnce({ compute: buildIndex, },);

/**
 Registers package entries for lookup by {@link ensurePackage}.
 Call this once with the array exported from a data file.
 Clears any previously built index so the next lookup rebuilds it.
 
 @param entries - Package entries produced by {@link p}
 
 @example
 ```ts
 import { packages } from '../data/packages.ts';
 registerPackages(packages);
 ```
 */
export function registerPackages(entries: readonly PackageEntry[],): void {
  registered.length = 0;
  registered.push(...entries,);
  packageIndex.reset();
}

//endregion Package index

//region Resolve package name

/**
 Sentinel returned by {@link resolvePackageName} when Repology data confirms
 the package is unavailable on the detected manager. A unique `Symbol` keeps
 the absent case out of a banned `string | undefined` union while staying
 distinguishable from any real package name (including the empty string).
 */
const UNPROVIDABLE_PACKAGE = Symbol('file-enforcer/package: detected manager cannot supply requested binary according to Repology data',);

/**
 Resolves the package name for a given entry and manager.
 Returns {@link UNPROVIDABLE_PACKAGE} when Repology data confirms the package is
 unavailable (manager not in `available` set), avoiding the expensive live
 `canProvide` check. Uses the per-manager override if present, otherwise falls
 back to effname.
 
 @param entry - Package entry to resolve
 
 @param manager - Detected package manager
 
 @returns Package name to pass to the install command, or {@link UNPROVIDABLE_PACKAGE} when unavailable
 */
function resolvePackageName(
  {
    entry,
    manager,
  }: {
    readonly entry: PackageEntry;
    readonly manager: PackageManager;
  },
): string | typeof UNPROVIDABLE_PACKAGE {
  if ((entry.available
    !== undefined) && (!entry.available
      .has(manager,)))
    return UNPROVIDABLE_PACKAGE;
  return entry.overrides[manager]
    ?? entry
      .effname;
}

//endregion Resolve package name

//region ensurePackage

/**
 Ensures a binary is available on the system.
 Checks PATH first; if absent, installs the corresponding package
 via the detected OS package manager.
 
 **For mise-installable tools** (node, python, jq, etc.), prefer
 `exec({ cmd: 'mise', args: ['use', '-g', '<tool>'] })` instead; it avoids root,
 manages versions, and works on immutable distros. Use this function
 for system-level packages that mise does not cover.
 
 Lookup order:
 1. Check if the binary exists on PATH via {@link binaryExists}; return immediately if so
 2. Look up the binary in the registered package index
 3. Detect the system's package manager via {@link detectManager}
 4. Resolve the per-manager package name
 5. Install the package via {@link installPackage}
 6. Verify the binary now exists on PATH
 
 @param binary - Name of the binary to ensure (e.g. `'rg'`, `'curl'`)
 
 @throws {@link NoManagerError} When no package manager is detected
 
 @throws {@link PackageNotFoundError} When the binary is not in the index and cannot be inferred
 
 @throws When the install command fails
 
 @throws {@link VerificationError} When the binary is still not on PATH after installation
 
 @example
 ```ts
 await ensurePackage('curl');
 await ensurePackage('rg');
 ```
 */
export async function ensurePackage(binary: string,): Promise<void> {
  /**
   Lazily-built lookup index; reuses prior build until {@link registerPackages} clears it.
   */
  const index = packageIndex.get();

  /**
   Registered entry for `binary`, or `undefined` when the caller never declared it.
   */
  const entry = index.get(binary,);
  /**
   Entry actually used downstream; falls back to a self-named default for unregistered binaries.
   */
  const effectiveEntry: PackageEntry = entry ?? {
    bin: binary,
    check: '--version',
    effname: binary,
    overrides: Object.freeze({},),
  };

  /**
   Whether the binary already responds to its existence check; short-circuits the install path.
   */
  const alreadyInstalled = await binaryExists({
    binary,
    checkFlag: effectiveEntry.check,
  },);
  if (alreadyInstalled)
    return;

  /**
   Detected manager on this host, or NO_PACKAGE_MANAGER when nothing supported is installed.
   */
  const manager = await detectManager();
  if (manager === NO_PACKAGE_MANAGER)
    throw new NoManagerError(binary,);

  /**
   Manager-specific package name; UNPROVIDABLE_PACKAGE when Repology says this manager cannot supply it.
   */
  const packageName = resolvePackageName({
    entry: effectiveEntry,
    manager,
  },);

  if (packageName === UNPROVIDABLE_PACKAGE) {
    throw new PackageNotFoundError(
      binary,
      manager,
      effectiveEntry.effname,
    );
  }

  /**
   Live confirmation from the manager's search command that the package is installable now.
   */
  const available = await canProvide({
    manager,
    packageName,
  },);
  if (!available) {
    throw new PackageNotFoundError(
      binary,
      manager,
      packageName,
    );
  }

  /**
   Function-scoped logger tagged with the call site for traceable install logs.
   */
  const rl = tagged({
    tag: ensurePackage.name,
    l,
  },);
  rl.info(`installing ${packageName} via ${manager} (binary: ${binary})`,);
  await installPackage({
    manager,
    packageName,
  },);

  /**
   Post-install existence check; non-true here means the package failed to provide the binary.
   */
  const verified = await binaryExists({
    binary,
    checkFlag: effectiveEntry.check,
  },);
  if (!verified) {
    throw new VerificationError(
      binary,
      manager,
      packageName,
    );
  }
}

//endregion ensurePackage

//region Errors

/**
 Thrown when the detected package manager cannot provide the requested package.
 */
class PackageNotFoundError extends Error {
  /**
   @param binary - Binary that triggered the install attempt
   
   @param manager - Detected package manager
   
   @param packageName - Package name that was searched for
   */
  constructor(
    binary: string,
    manager: PackageManager,
    packageName: string,
  ) {
    super(
      `Package "${packageName}" not found in ${manager} repository (binary: "${binary}")`,
    );
    this.name = 'PackageNotFoundError';
  }
}

/**
 Thrown when no package manager is detected on the system.
 */
class NoManagerError extends Error {
  /**
   @param binary - Binary that triggered the install attempt
   */
  constructor(binary: string,) {
    super(
      `Cannot install "${binary}": no supported package manager detected`,
    );
    this.name = 'NoManagerError';
  }
}

/**
 Thrown when a package was installed but the binary still cannot be found on PATH.
 */
class VerificationError extends Error {
  /**
   @param binary - Binary that was expected on PATH
   
   @param manager - Package manager used for installation
   
   @param packageName - Package name that was installed
   */
  constructor(
    binary: string,
    manager: PackageManager,
    packageName: string,
  ) {
    super(
      `Installed "${packageName}" via ${manager} but "${binary}" is still not on PATH`,
    );
    this.name = 'VerificationError';
  }
}

//endregion Errors
