//region Package manager identifiers

/**
 Supported OS-level package managers.
 Each corresponds to a detection predicate, is-installed check, and install command.
 */
export type PackageManager =
  | 'apk'
    | 'apt'
    | 'brew'
    | 'choco'
    | 'dnf'
    | 'pacman'
    | 'scoop'
    | 'zypper';

//endregion Package manager identifiers

//region Package entry

/**
 Per-manager package name overrides.
 Only managers whose package name differs from the effname need an entry;
 every key is optional so a sparse map carries just the differing managers.
 Spelled as explicit `?:` properties rather than `Partial<Record<...>>`,
 which `no-optional-escape` bans as a blanket optionality dodge.
 */
export type PackageMapping = {
  readonly apk?: string;
  readonly apt?: string;
  readonly brew?: string;
  readonly choco?: string;
  readonly dnf?: string;
  readonly pacman?: string;
  readonly scoop?: string;
  readonly zypper?: string;
};

/**
 Single element in the {@link PackageSpec.yes} availability array.
 
 - **String**: manager name; package name is the effname
 - **Tuple**: `[manager, packageName]` when the repo uses a different name
 
 @example
 ```ts
 'apt'                    // available in apt as the effname
 ['pacman', 'acpica-utils'] // available in pacman as 'acpica-utils'
 ```
 */
export type ManagerAvailability = PackageManager | readonly [
  PackageManager,
  string,
];

/**
 Object form accepted by {@link p} when the binary name or effname
 differs from the default, or when per-manager availability is known.
 
 - `bin`: binary name to check on PATH (defaults to `effname`)
 - `check`: custom flag for existence check (defaults to `--version`);
   some binaries use `-V`, `-v`, `--help`, or `version` instead
 - `effname`: Repology canonical project name (used as fallback package name)
 - `yes`: managers where Repology confirms the package exists;
   each element is a bare manager name (uses effname) or a
   `[manager, packageName]` tuple. Managers absent from this list
   are treated as unavailable, skipping the live `canProvide` check.
   When omitted, all managers are assumed available (for overrides
   that only correct `bin`/`check`).
 
 @example
 ```ts
 { bin: 'rg', effname: 'ripgrep' }
 { effname: 'acpica', yes: ['apt', ['dnf', 'acpica-tools'], ['pacman', 'acpica-utils']] }
 { bin: 'openssl', check: 'version', effname: 'openssl' }
 ```
 */
export type PackageSpec = {
  readonly bin?: string;
  readonly check?: string;
  readonly effname: string;
  readonly yes?: readonly ManagerAvailability[];
};

/**
 Normalized package entry produced by {@link p}.
 Immutable value object; safe to store in arrays and indexes.
 */
export type PackageEntry = {
  /**
   Managers where the package is known to be available (from Repology data).
   Absent (omitted) means no availability restriction (all managers assumed
   available). When the detected manager is not in this set, {@link ensurePackage}
   skips the live `canProvide` check and fails fast.
   */
  readonly available?: ReadonlySet<PackageManager>;
  /**
   Binary name to check on PATH.
   */
  readonly bin: string;
  /**
   Flag passed to the binary for existence check (default: `--version`).
   */
  readonly check: string;
  /**
   Repology canonical project name; fallback package name when no override exists.
   */
  readonly effname: string;
  /**
   Per-manager package name overrides (only entries that differ from effname).
   */
  readonly overrides: PackageMapping;
};

//endregion Package entry
