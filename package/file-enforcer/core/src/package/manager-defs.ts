import type { PackageManager, } from './types.ts';

//region Manager definitions

/**
 Per-manager command templates.
 `{pkg}` is replaced with the resolved package name at call time.
 
 - `check`: predicate to detect whether this manager is available
 - `search`: command to verify a package exists in the repo (exit 0 = available)
 - `install`: command template for installing a package
 - `needsRoot`: whether install requires privilege escalation
 */
export type ManagerDef = {
  readonly check: readonly string[];
  readonly search: readonly string[];
  readonly install: readonly string[];
  readonly needsRoot: boolean;
};

/**
 OS-level package manager command templates, ordered by detection priority.
 Iteration order of a `Map` matches insertion order, so
 `MANAGERS.keys()` yields managers in priority order and
 `detectManager` can iterate the map directly.
 
 **Prefer mise for developer tools.** When the target binary is a
 mise-installable tool (node, python, go, rust, jq, etc.), callers
 should install it via mise instead of falling through to an OS manager:
 
 ```ts
 await exec({ cmd: 'mise', args: ['use', '-g', 'jq'] });
 ```
 
 Mise installs to `~/.local/share/mise` without root, manages versions,
 and works identically on every OS including immutable distros.
 Reserve {@link ensurePackage} and these OS managers for system-level
 packages that mise does not cover (coreutils, networking tools,
 system libraries).
 
 Brew is first because it works on immutable (image-based) distros
 like Fedora Atomic, Universal Blue, and NixOS where the native package
 manager either operates on the base image layer (requiring a reboot)
 or is read-only at runtime. Brew installs to a mutable prefix
 (`/home/linuxbrew/.linuxbrew`) that survives image updates,
 making it the only practical option for runtime package installation
 on these systems.
 
 After brew, native managers are ordered from most common to least common,
 followed by Windows managers.
 */
export const MANAGERS: ReadonlyMap<PackageManager, ManagerDef> = new Map<PackageManager,
  ManagerDef>([
    [
      'brew',
      {
        check: [
          'brew',
          '--version',
        ],
        search: [
          'brew',
          'info',
          '{pkg}',
        ],
        install: [
          'brew',
          'install',
          '{pkg}',
        ],
        needsRoot: false,
      },
    ],
    [
      'apt',
      {
        check: [
          'apt-get',
          '--version',
        ],
        search: [
          'apt-cache',
          'show',
          '{pkg}',
        ],
        install: [
          'apt-get',
          'install',
          '--yes',
          '{pkg}',
        ],
        needsRoot: true,
      },
    ],
    [
      'dnf',
      {
        check: [
          'dnf',
          '--version',
        ],
        search: [
          'dnf',
          'info',
          '{pkg}',
        ],
        install: [
          'dnf',
          'install',
          '--assumeyes',
          '{pkg}',
        ],
        needsRoot: true,
      },
    ],
    [
      'pacman',
      {
        check: [
          'pacman',
          '--version',
        ],
        search: [
          'pacman',
          '-Si',
          '{pkg}',
        ],
        install: [
          'pacman',
          '-S',
          '--noconfirm',
          '{pkg}',
        ],
        needsRoot: true,
      },
    ],
    [
      'apk',
      {
        check: [
          'apk',
          '--version',
        ],
        search: [
          'apk',
          'info',
          '--description',
          '{pkg}',
        ],
        install: [
          'apk',
          'add',
          '{pkg}',
        ],
        needsRoot: true,
      },
    ],
    [
      'zypper',
      {
        check: [
          'zypper',
          '--version',
        ],
        search: [
          'zypper',
          'info',
          '{pkg}',
        ],
        install: [
          'zypper',
          'install',
          '--non-interactive',
          '{pkg}',
        ],
        needsRoot: true,
      },
    ],
    [
      'scoop',
      {
        check: [
          'scoop',
          '--version',
        ],
        search: [
          'scoop',
          'info',
          '{pkg}',
        ],
        install: [
          'scoop',
          'install',
          '{pkg}',
        ],
        needsRoot: false,
      },
    ],
    [
      'choco',
      {
        check: [
          'choco',
          '--version',
        ],
        search: [
          'choco',
          'info',
          '{pkg}',
        ],
        install: [
          'choco',
          'install',
          '{pkg}',
          '--yes',
        ],
        needsRoot: false,
      },
    ],
  ],);

/**
 Set of all known {@link PackageManager} names, derived from {@link MANAGERS}.
 Used by {@link p} to separate manager overrides from structural fields when
 destructuring a {@link PackageSpec}.
 */
export const MANAGER_KEYS: ReadonlySet<string> = new Set<PackageManager>(
  MANAGERS.keys(),
);

//endregion Manager definitions
