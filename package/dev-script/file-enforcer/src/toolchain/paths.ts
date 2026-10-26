/**
 Path algebra for the toolchain system: worktree root, machine toolchain
 root, exposure directory, env file, and the committed artifact locations.

 Node builtins only, so `bin-paths.ts` stays dependency-free for consumers
 that run it without the workspace's `node_modules`.
 */

import { homedir, } from 'node:os';
import { isAbsolute, join, resolve, } from 'node:path';

//region Roots

/**
 Environment record consulted for `HOME` and `XDG_DATA_HOME`.
 */
export type PathEnv = Readonly<Record<string, string | undefined>>;

/**
 Resolved toolchain path set.
 */
export type ToolchainPaths = {
  /** Repository root (the worktree containing `package/`). */
  readonly worktreeRoot: string;
  /** Machine-level install root (`.../monochromatic-dev/toolchain`). */
  readonly toolchainRoot: string;
  /** Absolute per-worktree exposure directory (`.cache/toolchain-bin`). */
  readonly exposureDir: string;
  /** Canonical worktree-relative spelling of {@link ToolchainPaths.exposureDir}. */
  readonly exposureDirRelative: '.cache/toolchain-bin';
  /** Absolute `.cache/toolchain.env.json` (gitignored, machine-absolute). */
  readonly envFile: string;
  /** Absolute `.cache/cli-git-bin` (the postinstall `git` link directory). */
  readonly cliGitBinDir: string;
  /** Absolute committed `toolchain.lock.json`. */
  readonly lockFile: string;
  /** Absolute committed `mpm.packages.toml`. */
  readonly mpmPackagesFile: string;
  /** Absolute committed `mpm.managers.toml`. */
  readonly mpmManagersFile: string;
  /** Absolute `bin/` directory inside the machine toolchain root. */
  readonly toolchainBinDir: string;
};

/**
 Worktree-relative exposure directory, fixed by
 `doc/planning/mpm-toolchain-migration.md`.
 */
export const EXPOSURE_DIR_RELATIVE = '.cache/toolchain-bin' as const;

/**
 Resolves the worktree root from this module's own location
 (`<root>/package/dev-script/file-enforcer/src/toolchain`), so callers can run
 the toolchain entry points from any working directory.
 
 @param env - Environment consulted for an explicit override
 
 @returns Absolute worktree root
 */
export function worktreeRootFromModule(
  { env, }: { readonly env?: PathEnv | undefined; } = {},
): string {
  /**
   Explicit override, used by tests and disposable fixtures.
   */
  const override = env?.MONOCHROMATIC_WORKTREE_ROOT;
  if (override !== undefined && override !== '')
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
 
 @param env - Environment consulted for `XDG_DATA_HOME` and `HOME`
 
 @returns Absolute toolchain root
 */
export function toolchainRootFromEnv(
  { env, }: { readonly env?: PathEnv | undefined; } = {},
): string {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  /**
   Data home: `XDG_DATA_HOME`, else `$HOME/.local/share`, else the runtime
   homedir's `.local/share`.
   */
  const dataHome = e.XDG_DATA_HOME !== undefined && e.XDG_DATA_HOME !== ''
    ? e.XDG_DATA_HOME
    : join(homeDirFromEnv({ env: e, }), '.local', 'share',);
  return join(
    resolve(dataHome,),
    'monochromatic-dev',
    'toolchain',
  );
}

/**
 Resolves the current user's home directory from `HOME`, falling back to the
 runtime homedir.
 
 @param env - Environment consulted for `HOME`
 
 @returns Absolute home directory
 */
export function homeDirFromEnv(
  { env, }: { readonly env?: PathEnv | undefined; } = {},
): string {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  return e.HOME !== undefined && e.HOME !== ''
    ? resolve(e.HOME,)
    : homedir();
}

/**
 Builds the full path set for one worktree.
 
 @param options - Worktree root and environment overrides
 
 @returns Resolved toolchain paths
 */
export function resolveToolchainPaths(
  {
    worktreeRoot,
    env,
  }: {
    readonly worktreeRoot?: string;
    readonly env?: PathEnv | undefined;
  } = {},
): ToolchainPaths {
  /**
   Absolute worktree root.
   */
  const root = worktreeRoot !== undefined
    ? resolve(worktreeRoot,)
    : worktreeRootFromModule({ env, });
  /**
   Absolute machine toolchain root.
   */
  const toolchainRoot = toolchainRootFromEnv({ env, });
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
  };
}

//endregion Roots

//region Install directories

/**
 Absolute install directory of one tool inside the machine toolchain root
 (`<root>/<id>/<version>`).
 
 @param options - Toolchain root, tool id, and resolved version
 
 @returns Absolute install directory
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
    id.replaceAll(
      /[^A-Za-z0-9._-]+/g,
      '-',
    ),
    version,
  );
}

/**
 Manager-native bin directory candidates a route installs into,
 in resolution order.
 
 @param options - Path set and environment
 
 @returns Absolute candidate directories
 */
export function managerNativeBinDirs(
  {
    paths,
    env,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv | undefined;
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
    e.CARGO_HOME !== undefined && e.CARGO_HOME !== ''
      ? resolve(e.CARGO_HOME,)
      : join(home, '.cargo',),
    'bin',
  );
  /**
   uv tool executable directory (`uv tool install` links here).
   */
  const uvToolBin = e.UV_TOOL_BIN_DIR !== undefined && e.UV_TOOL_BIN_DIR !== ''
    ? resolve(e.UV_TOOL_BIN_DIR,)
    : join(home, '.local', 'bin',);
  /**
   npm global prefix bin directory, when the environment pins one.
   */
  const npmPrefixBin = e.npm_config_prefix !== undefined && e.npm_config_prefix !== ''
    ? join(resolve(e.npm_config_prefix,), 'bin',)
    : undefined;
  return [
    paths.toolchainBinDir,
    cargoBin,
    uvToolBin,
    ...(npmPrefixBin === undefined
      ? []
      : [npmPrefixBin]),
  ];
}

//endregion Install directories

//region Assertions

/**
 Throws when a path is not absolute, keeping CLI output contracts honest.
 
 @param options - Path to check and the field name to name in the error
 
 @returns The checked path
 
 @throws {Error} when the path is relative
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
