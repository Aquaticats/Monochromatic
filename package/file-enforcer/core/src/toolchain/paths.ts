/**
 Path algebra for the toolchain system: worktree root, machine toolchain
 root, exposure directory, env file, and the committed artifact locations.

 Node builtins only, so `bin-paths.ts` stays dependency-free for consumers
 that run it without the workspace's `node_modules`.
 */

import { realpath, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  isAbsolute,
  join,
  resolve,
} from 'node:path';
import { isErrnoException, } from './fs-probe.ts';

//region Roots

/**
 Environment record consulted for `HOME` and `XDG_DATA_HOME`.
 */
export type PathEnv = Readonly<NodeJS.ProcessEnv>;

/**
 Resolved toolchain path set.

 One instance per scope: the repository scope resolves through
 {@link resolveToolchainPaths} and the machine-global scope through
 {@link globalToolchainPaths}
 (`doc/planning/mpm-global-toolchain-migration.md`).
 */
export type ToolchainPaths = {
  /**
   Root of the scope's own files (worktree, or the global config stack).
   */
  readonly worktreeRoot: string;
  /**
   Machine-level install root (`.../monochromatic-dev/toolchain[-global]`).
   */
  readonly toolchainRoot: string;
  /**
   Absolute exposure directory holding one link per declared bin.
   */
  readonly exposureDir: string;
  /**
   Canonical root-relative spelling of {@link ToolchainPaths.exposureDir}.
   */
  readonly exposureDirRelative: string;
  /**
   Absolute tool environment file (gitignored, machine-absolute).
   */
  readonly envFile: string;
  /**
   Absolute directory the postinstall `git` link lands in.
   */
  readonly cliGitBinDir: string;
  /**
   Absolute `toolchain.lock.json` of this scope.
   */
  readonly lockFile: string;
  /**
   Absolute `mpm.packages.toml` of this scope.
   */
  readonly mpmPackagesFile: string;
  /**
   Absolute `mpm.managers.toml` of this scope.
   */
  readonly mpmManagersFile: string;
  /**
   Absolute `bin/` directory inside the machine toolchain root.
   */
  readonly toolchainBinDir: string;
  /**
   Absolute `bin` CLI configuration of this scope (`BIN_CONFIG`), so two
   scopes never share one `default_path` and install database.
   */
  readonly binConfigFile: string;
};

/**
 Worktree-relative exposure directory, fixed by
 `doc/planning/mpm-toolchain-migration.md`.
 */
export const EXPOSURE_DIR_RELATIVE = '.cache/toolchain-bin' as const;

/**
 Resolves the worktree root from this module's own location
 (`<root>/package/file-enforcer/core/src/toolchain`), so callers can run
 the toolchain entry points from any working directory.
 
 @param env - Environment consulted for an explicit override; absent means no override

 @returns Absolute worktree root

 @example
 ```ts
 worktreeRootFromModule({ env: { MONOCHROMATIC_WORKTREE_ROOT: '/work/tree', }, },); // '/work/tree'
 ```
 */
export function worktreeRootFromModule(
  { env, }: { readonly env?: PathEnv; } = {},
): string {
  /**
   Explicit override, used by tests and disposable fixtures.
   */
  const override = env?.MONOCHROMATIC_WORKTREE_ROOT;
  if ((override !== undefined) && (override !== ''))
    return resolve(override,);
  return resolve(
    import.meta.dirname,
    '..',
    '..',
    '..',
    '..',
    '..',
  );
}

/**
 Resolves the machine toolchain root
 (`${XDG_DATA_HOME:-$HOME/.local/share}/monochromatic-dev/toolchain`),
 deriving the home directory from the environment or the runtime homedir.
 
 @param env - Environment consulted for `XDG_DATA_HOME` and `HOME`; the process environment when absent

 @returns Absolute toolchain root

 @example
 ```ts
 toolchainRootFromEnv({ env: { XDG_DATA_HOME: '/data', }, },); // '/data/monochromatic-dev/toolchain'
 ```
 */
export function toolchainRootFromEnv(
  { env, }: { readonly env?: PathEnv; } = {},
): string {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  /**
   Data home: `XDG_DATA_HOME`, else `$HOME/.local/share`, else the runtime
   homedir's `.local/share`.
   */
  const dataHome = (e.XDG_DATA_HOME !== undefined) && (e.XDG_DATA_HOME !== '')
    ? e.XDG_DATA_HOME
    : join(
      homeDirFromEnv({ env: e, }),
      '.local',
      'share',
    );
  return join(
    resolve(dataHome,),
    'monochromatic-dev',
    'toolchain',
  );
}

/**
 Resolves the current user's home directory from `HOME`, falling back to the
 runtime homedir.
 
 @param env - Environment consulted for `HOME`; the process environment when absent

 @returns Absolute home directory

 @example
 ```ts
 homeDirFromEnv({ env: { HOME: '/home/user', }, },); // '/home/user'
 ```
 */
export function homeDirFromEnv(
  { env, }: { readonly env?: PathEnv; } = {},
): string {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  return (e.HOME !== undefined) && (e.HOME !== '')
    ? resolve(e.HOME,)
    : homedir();
}

/**
 Builds the full path set for one worktree.
 
 @param worktreeRoot - Worktree root; absent means the root derived from this module's location

 @param env - Environment overrides; absent means the process environment for the machine roots

 @returns Resolved toolchain paths

 @example
 ```ts
 resolveToolchainPaths({ worktreeRoot: '/work/tree', },).lockFile; // '/work/tree/toolchain.lock.json'
 ```
 */
export function resolveToolchainPaths(
  {
    worktreeRoot,
    env,
  }: {
    readonly worktreeRoot?: string;
    readonly env?: PathEnv;
  } = {},
): ToolchainPaths {
  /**
   Absolute worktree root.
   */
  const root = worktreeRoot !== undefined
    ? resolve(worktreeRoot,)
    : worktreeRootFromModule(env === undefined
      ? {}
      : { env, },);
  /**
   Absolute machine toolchain root.
   */
  const toolchainRoot = toolchainRootFromEnv({ env: env ?? process.env, },);
  /**
   Absolute exposure directory.
   */
  const exposureDir = join(
    root,
    EXPOSURE_DIR_RELATIVE,
  );
  return {
    worktreeRoot: root,
    toolchainRoot,
    exposureDir,
    exposureDirRelative: EXPOSURE_DIR_RELATIVE,
    envFile: join(
      root,
      '.cache',
      'toolchain.env.json',
    ),
    cliGitBinDir: join(
      root,
      '.cache',
      'cli-git-bin',
    ),
    lockFile: join(
      root,
      'toolchain.lock.json',
    ),
    mpmPackagesFile: join(
      root,
      'mpm.packages.toml',
    ),
    mpmManagersFile: join(
      root,
      'mpm.managers.toml',
    ),
    toolchainBinDir: join(
      toolchainRoot,
      'bin',
    ),
    binConfigFile: join(
      toolchainRoot,
      'bin.config.json',
    ),
  };
}

/**
 Resolves the machine-global scope's path set
 (`doc/planning/mpm-global-toolchain-migration.md`):
 the declarations and generated records live in
 `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`,
 payloads in
 `${XDG_DATA_HOME:-$HOME/.local/share}/monochromatic-dev/toolchain-global`,
 and the exposure farm is `<toolchainRoot>/exposure`.
 
 @param env - Environment consulted for `HOME`, `XDG_CONFIG_HOME` and
 `XDG_DATA_HOME`; the process environment when absent

 @returns Resolved global-scope toolchain paths

 @example
 ```ts
 globalToolchainPaths({ env: { HOME: '/home/user', }, },).exposureDir;
 // '/home/user/.local/share/monochromatic-dev/toolchain-global/exposure'
 ```
 */
export function globalToolchainPaths(
  {
    env,
  }: {
    readonly env?: PathEnv;
  } = {},
): ToolchainPaths {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  /**
   Home directory behind the XDG defaults.
   */
  const home = homeDirFromEnv({ env: e, });
  /**
   Config stack directory (declarations, lock, manifests).
   */
  const configDir = join(
    (e.XDG_CONFIG_HOME !== undefined) && (e.XDG_CONFIG_HOME !== '')
      ? resolve(e.XDG_CONFIG_HOME,)
      : join(
        home,
        '.config',
      ),
    'monochromatic-dev',
    'toolchain',
  );
  /**
   Machine payload root of the global scope.
   */
  const toolchainRoot = join(
    (e.XDG_DATA_HOME !== undefined) && (e.XDG_DATA_HOME !== '')
      ? resolve(e.XDG_DATA_HOME,)
      : join(
        home,
        '.local',
        'share',
      ),
    'monochromatic-dev',
    'toolchain-global',
  );
  return {
    worktreeRoot: configDir,
    toolchainRoot,
    exposureDir: join(
      toolchainRoot,
      'exposure',
    ),
    exposureDirRelative: 'exposure',
    envFile: join(
      toolchainRoot,
      'toolchain.env.json',
    ),
    // The global scope's postinstall `git` link is the user-facing
    // `~/.local/bin/git` (`doc/planning/cli-git-global-install.md`).
    cliGitBinDir: join(
      home,
      '.local',
      'bin',
    ),
    lockFile: join(
      configDir,
      'toolchain.lock.json',
    ),
    mpmPackagesFile: join(
      configDir,
      'mpm.packages.toml',
    ),
    mpmManagersFile: join(
      configDir,
      'mpm.managers.toml',
    ),
    toolchainBinDir: join(
      toolchainRoot,
      'bin',
    ),
    binConfigFile: join(
      toolchainRoot,
      'bin.config.json',
    ),
  };
}

//endregion Roots

//region Install directories

/**
 Whether one character may appear verbatim in an install directory name:
 an ASCII letter or digit, `.`, `_` or `-`.

 @param char - One code point of a tool id

 @returns `true` when the character is kept as is
 */
function isInstallDirCharacter(char: string,): boolean {
  return ((char >= 'a') && (char <= 'z'))
    || ((char >= 'A') && (char <= 'Z'))
    || ((char >= '0') && (char <= '9'))
    || ('._-'.includes(char,));
}

/**
 Install directory name of a tool id: each run of characters outside ASCII
 letters, digits, `.`, `_` and `-` becomes one `-`.

 Two linear passes over the code points and one join, so the work is linear
 in the id length.

 @param id - Tool id (`cargo:monochromatic-search-shadow`)

 @returns Directory-safe name (`cargo-monochromatic-search-shadow`)
 */
function installDirName(id: string,): string {
  /**
   Code points of the id; a lone surrogate is one code point, as the old pattern treated it.
   */
  const chars = Array.from(id,);
  /**
   Whether each code point is kept verbatim.
   */
  const kept = chars.map(
    function isKept(char,): boolean {
      return isInstallDirCharacter(char,);
    },
  );
  return chars.map(
    function keepOrCollapse(
      char,
      index,
    ): string {
      if (kept[index] === true)
        return char;
      // A replaced code point after another replaced one belongs to the same run.
      return kept[index - 1] === false
        ? ''
        : '-';
    },
  )
    .join('',);
}

/**
 Absolute install directory of one tool inside the machine toolchain root
 (`<root>/<id>/<version>`).
 
 @param toolchainRoot - Machine toolchain root
 
 @param id - Tool id; characters outside ASCII letters, digits, `.`, `_` and `-` become `-`
 
 @param version - Resolved tool version

 @returns Absolute install directory

 @example
 ```ts
 toolInstallDir({ toolchainRoot: '/root', id: 'cargo:rg', version: '15.2.0', },); // '/root/cargo-rg/15.2.0'
 ```
 */
export function toolInstallDir(
  {
    toolchainRoot,
    id,
    version,
  }: {
    readonly toolchainRoot: string;
    readonly id: string;
    readonly version: string;
  },
): string {
  return join(
    toolchainRoot,
    installDirName(id,),
    version,
  );
}

/**
 Manager-native bin directory candidates a route installs into,
 in resolution order.
 
 @param paths - Resolved toolchain paths of the scope

 @param env - Environment overriding manager homes; the process environment when absent

 @returns Absolute candidate directories

 @example
 ```ts
 managerNativeBinDirs({ paths, },)[0]; // paths.exposureDir
 ```
 */
export function managerNativeBinDirs(
  {
    paths,
    env,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv;
  },
): readonly string[] {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  /**
   Home directory for manager-native defaults.
   */
  const home = homeDirFromEnv({ env: e, });
  /**
   Cargo home bin directory (`cargo binstall` installs here).
   */
  const cargoBin = join(
    (e.CARGO_HOME !== undefined) && (e.CARGO_HOME !== '')
      ? resolve(e.CARGO_HOME,)
      : join(
        home,
        '.cargo',
      ),
    'bin',
  );
  /**
   uv tool executable directory (`uv tool install` links here).
   */
  const uvToolBin = (e.UV_TOOL_BIN_DIR !== undefined) && (e.UV_TOOL_BIN_DIR !== '')
    ? resolve(e.UV_TOOL_BIN_DIR,)
    : join(
      home,
      '.local',
      'bin',
    );
  /**
   npm global prefix bin directory, when the environment pins one.
   */
  const npmPrefixBin = (e.npm_config_prefix !== undefined) && (e.npm_config_prefix !== '')
    ? join(
      resolve(e.npm_config_prefix,),
      'bin',
    )
    : undefined;
  /**
   pnpm global bin directory (`pnpm add --global` refuses to install
   unless it is on `PATH`).
   */
  const pnpmGlobalBin = join(
    (e.PNPM_HOME !== undefined) && (e.PNPM_HOME !== '')
      ? resolve(e.PNPM_HOME,)
      : join(
        home,
        '.local',
        'share',
        'pnpm',
      ),
    'bin',
  );
  return [
    // The exposure directory leads, matching the documented bin-paths contract.
    paths.exposureDir,
    paths.toolchainBinDir,
    cargoBin,
    uvToolBin,
    pnpmGlobalBin,
    ...(npmPrefixBin === undefined
      ? []
      : [npmPrefixBin]),
  ];
}

//endregion Install directories

//region Assertions

/**
 Throws when a path is not absolute, keeping CLI output contracts honest.
 
 @param path - Path to check

 @param field - Field name to name in the error

 @returns The checked path

 @throws Error when the path is relative

 @example
 ```ts
 assertAbsolute({ path: '/usr/bin', field: 'binDir', },); // '/usr/bin'
 ```
 */
export function assertAbsolute(
  {
    path,
    field,
  }: {
    readonly path: string;
    readonly field: string;
  },
): string {
  if (!isAbsolute(path,))
    throw new Error(`${field} must be an absolute path, got ${JSON.stringify(path,)}`,);
  return path;
}

//endregion Assertions

//region Entry point guard

/**
 Whether this module is the process entry point, comparing real paths so
 symlinked home directories (`/home/user` versus the backing store) still
 match.

 @param moduleFilename - Absolute filename of the module asking (`import.meta.filename`)

 @returns `true` when the process was started on this file

 @example
 ```ts
 if (await isMainModule(import.meta.filename,))
   await main();
 ```
 */
export async function isMainModule(moduleFilename: string,): Promise<boolean> {
  /**
   Entry script path the process was started with.
   */
  const [, entry,] = process.argv;
  if (entry === undefined)
    return false;
  try {
    return await realpath(entry,) === await realpath(moduleFilename,);
  } catch (error) {
    // A missing or unreadable entry path means the process was not started on this file.
    if (!isErrnoException(error,))
      throw error;
    return false;
  }
}

//endregion Entry point guard
