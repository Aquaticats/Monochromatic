/**
 Provider installs: one function per toolchain route, each pinned to
 `toolchain.lock.json` and idempotent (the caller probes observed state first).
 */

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { homedir, } from 'node:os';
import { basename, dirname, join, } from 'node:path';
import { gunzipSync, } from 'node:zlib';
import {
  downloadVerified,
  extractArchive,
  makeExecutable,
  run,
  type RunResult,
} from './exec.ts';
import {
  findToolBin,
  hostPlatformId,
} from './layout.ts';
import {
  homeDirFromEnv,
  managerNativeBinDirs,
  toolInstallDir,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';
import {
  BIN_BOOTSTRAP,
  MPM_BOOTSTRAP,
  toolById,
} from './plan.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Context

/**
 Shared context every provider receives.
 */
export type ProviderContext = {
  readonly entry: ToolPlanEntry;
  readonly paths: ToolchainPaths;
  readonly env: PathEnv;
  /** Extra bin search directories a staged tool contributes. */
  readonly extraDirs: readonly string[];
};

/**
 Absolute artifact URL and SHA-256 for the current host platform.
 
 @param entry - Plan entry to resolve
 
 @returns Artifact row for the host platform
 
 @throws {Error} when the plan records no artifact for the host
 */
export function artifactForHost(entry: ToolPlanEntry,): {
  readonly url: string;
  readonly sha256: string;
} {
  /**
   Platform id of the running host.
   */
  const platform = hostPlatformId();
  /**
   Artifact row recorded for the host platform.
   */
  const row = entry.artifacts[platform];
  if (row === undefined) {
    throw new Error(
      `toolchain plan has no artifact for ${entry.id} on ${platform}`,
    );
  }
  return row;
}

//endregion Context

//region Tool resolution

/**
 Absolute path of the `bin` CLI at the bootstrap's own install directory.

 The archive route stages the CLI under `<root>/bin/<version>/`; install and
 status calls must not assume the flat `<root>/bin/<name>` layout the CLI
 itself uses for the tools it downloads.

 @param options - Path set and environment

 @returns Absolute CLI path

 @throws Error when the CLI is not installed
 */
export function binCliPath(
  {
    paths,
    env,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv | undefined;
  },
): string {
  /**
   Resolved CLI from the bootstrap entry's install directory.
   */
  const resolved = findToolBin({
    entry: BIN_BOOTSTRAP,
    name: 'bin',
    paths,
    env,
  },);
  if (resolved === null) {
    throw new Error(
      `bin CLI is not installed; expected it under ${
        toolInstallDir({
          toolchainRoot: paths.toolchainRoot,
          id: BIN_BOOTSTRAP.id,
          version: BIN_BOOTSTRAP.version,
        },)
      }`,
    );
  }
  return resolved;
}

/**
 Environment for manager commands: the toolchain's own bin directories lead
 `PATH`, so retired machine shims cannot divert an install run.

 @param options - Path set and environment

 @returns Environment whose `PATH` starts with the toolchain directories
 */
export function toolchainPathEnv(
  {
    paths,
    env,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv | undefined;
  },
): Record<string, string> {
  /**
   Effective environment.
   */
  const e = env ?? process.env;
  /**
   Environment with unset variables dropped.
   */
  const merged: Record<string, string> = {};
  for (const [key, value,] of Object.entries(process.env,)) {
    if (value !== undefined)
      merged[key] = value;
  }
  merged.HOME = homeDirFromEnv({ env: e, });
  merged.PATH = [
    ...managerNativeBinDirs({
      paths,
      env: e,
    },),
    e.PATH ?? '',
  ].join(':',);
  return merged;
}

//endregion Tool resolution

//region Archive provider

/**
 Installs one `archive`-route tool:
 download to the toolchain root, verify SHA-256, extract into
 `<root>/<id>/<version>/`, and copy single-file payloads in place.
 
 @param context - Provider context
 
 @returns The install directory
 */
export async function installArchive(
  context: ProviderContext,
): Promise<string> {
  const { entry, paths, env, } = context;
  /**
   Verified artifact for the host platform.
   */
  const artifact = artifactForHost(entry,);
  /**
   Download path inside the toolchain root.
   */
  const downloadPath = join(
    paths.toolchainRoot,
    'downloads',
    `${entry.id.replaceAll(
      /[^A-Za-z0-9._-]+/g,
      '-',
    )}-${
      entry.version
    }-${
      basename(artifact.url,)
    }`,
  );
  await downloadVerified({
    url: artifact.url,
    target: downloadPath,
    sha256: artifact.sha256,
  },);
  /**
   Machine install directory of this tool.
   */
  const installDir = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: entry.version,
  });
  mkdirSync(
    installDir,
    { recursive: true },
  );
  if (/\.(tar\.gz|tgz|tar\.xz|tar\.bz2|tar|zip)$/u.test(artifact.url,)) {
    await extractArchive({
      archive: downloadPath,
      into: installDir,
    },);
    return installDir;
  }
  // Single-file payload (a standalone binary release asset).
  /**
   Destination file name: the tool's first declared bin.
   */
  const name = entry.bins[0] ?? basename(artifact.url,);
  /**
   Destination path of the payload inside the install directory.
   */
  const target = join(
    installDir,
    name,
  );
  if (artifact.url.endsWith('.gz',)) {
    // Gzip single-file payloads (taplo ships a .gz binary) must be
    // decompressed before the file can run.
    writeFileSync(
      target,
      gunzipSync(readFileSync(downloadPath,),),
    );
  }
  else {
    copyFileSync(
      downloadPath,
      target,
    );
  }
  makeExecutable(target,);
  return installDir;
}

//endregion Archive provider

//region bin-sourced provider

/**
 Resolves the release-tag URL behind one artifact download URL
 (`https://github.com/<owner>/<repo>/releases/download/<tag>/<asset>` to
 `https://github.com/<owner>/<repo>/releases/tag/<tag>`).
 
 @param artifactUrl - Artifact download URL from `toolchain.lock.json`
 
 @returns Release-tag URL for `bin install`, or `null` when the URL is not a
 GitHub release download
 */
export function releaseTagUrl(artifactUrl: string,): string | null {
  /**
   Parsed GitHub release-download components.
   */
  const match = artifactUrl.match(
    /^(https:\/\/github\.com\/[^/]+\/[^/]+)\/releases\/download\/([^/]+)\//u,
  );
  if (match === null)
    return null;
  /**
   Release repository URL of the artifact.
   */
  const repo = match[1];
  /**
   URL-encoded release tag of the artifact.
   */
  const tag = match[2];
  if (repo === undefined || tag === undefined)
    return null;
  return `${repo}/releases/tag/${decodeURIComponent(tag,)}`;
}

/**
 Pre-seeds the `bin` CLI configuration so no interactive prompt appears:
 `bin` asks for its download directory until `default_path` is configured.
 The file `bin` actually resolves is seeded
 (`BIN_CONFIG`, else `$HOME/.bin/config.json`, else
 `$XDG_CONFIG_HOME/bin/config.json`, else `$HOME/.config/bin/config.json`),
 and an existing configuration is left alone.
 
 @param options - Environment and the download directory to record
 
 @returns The configuration path `bin` will read
 */
export function seedBinConfig(
  {
    env,
    downloadDir,
  }: {
    readonly env: PathEnv;
    readonly downloadDir: string;
  },
): string {
  /**
   Effective environment.
   */
  const e = env ?? process.env;
  /**
   Home directory for the fallback locations.
   */
  const home = homeDirFromEnv({ env: e, });
  /**
   Candidate configuration paths, in `bin`'s own resolution order.
   */
  const candidates = [
    ...(e.BIN_CONFIG !== undefined && e.BIN_CONFIG !== ''
      ? [e.BIN_CONFIG]
      : []),
    join(
      home,
      '.bin',
      'config.json',
    ),
    ...(e.XDG_CONFIG_HOME !== undefined && e.XDG_CONFIG_HOME !== ''
      ? [join(
        e.XDG_CONFIG_HOME,
        'bin',
        'config.json',
      )]
      : []),
    join(
      home,
      '.config',
      'bin',
      'config.json',
    ),
    join(
      home,
      '.bin',
      'config.json',
    ),
  ];
  /**
   File `bin` resolves first: the first candidate that exists (an empty
   `~/.bin/config.json` shadows the later ones), else the first candidate.
   The seed converges that file instead of leaving a shadowing empty one.
   */
  const target = candidates.find(
    function exists(candidate,): boolean {
      return existsSync(candidate,);
    },
  ) ?? candidates[0];
  if (target === undefined)
    throw new Error('no bin configuration path is derivable from the environment',);
  /**
   Existing settings kept, with `default_path` converged to the toolchain.
   */
  const existing: Record<string, unknown> = {};
  if (existsSync(target,)) {
    try {
      Object.assign(
        existing,
        JSON.parse(readFileSync(
          target,
          'utf8',
        ),) as Record<string, unknown>,
      );
    }
    catch (error) {
      // Only a malformed file reaches here; it is replaced below.
      if (!(error instanceof SyntaxError))
        throw error;
    }
  }
  mkdirSync(
    dirname(target,),
    { recursive: true },
  );
  writeFileSync(
    target,
    `${
      JSON.stringify(
        {
          ...existing,
          default_path: downloadDir,
          bins: existing.bins ?? {},
        },
        null,
        4,
      )
    }\n`,
    'utf8',
  );
  return target;
}

/**
 Installs one `bin-sourced` tool:
 `bin install <release-tag-url> -n <asset> glob <bin>` per declared primary
 binary, with fallbacks for root-level archive files and raw release assets.
 
 @param context - Provider context
 
 @returns `bin list` style descriptions of the installed files
 */
export async function installBinSourced(
  context: ProviderContext,
): Promise<RunResult> {
  const { entry, paths, env, } = context;
  /**
   Verified artifact for the host platform, whose URL carries the tag.
   */
  const artifact = artifactForHost(entry,);
  /**
   Release-tag URL `bin install` resolves to the locked release.
   */
  const tagUrl = releaseTagUrl(artifact.url,);
  if (tagUrl === null)
    throw new Error(`cannot derive a release-tag URL from ${artifact.url}`,);
  /**
   Asset file name inside the release.
   */
  const asset = basename(artifact.url,);
  /**
   Alias map: extra exposure names sharing one installed file.
   */
  const aliases = entry.options.binAliases ?? {};
  /**
   Primary binary names `bin install` copies (aliases link to these).
   */
  const primaries = entry.bins.filter(
    function isPrimary(name,): boolean {
      return aliases[name] === undefined;
    },
  );
  seedBinConfig({
    env,
    downloadDir: paths.toolchainBinDir,
  },);
  /**
   Absolute `bin` CLI at the bootstrap's install directory.
   */
  const cli = binCliPath({
    paths,
    env,
  },);
  /**
   Last observed run result.
   */
  let result: RunResult = {
    stdout: '',
    stderr: '',
    code: 0,
  };
  for (const name of primaries) {
    /**
     Candidate `-n` patterns: nested archive file, root-level archive file,
     then the raw asset with a destination rename.
     */
    const attempts = [
      [
        '-n',
        `${asset}/**/${name}`,
      ],
      [
        '-n',
        `${asset}/${name}`,
      ],
      [
        '-n',
        asset,
        name,
      ],
    ];
    /**
     Whether one attempt succeeded for this binary.
     */
    let installed = false;
    for (const attempt of attempts) {
      result = await run({
        command: cli,
        args: [
          'install',
          tagUrl,
          ...attempt,
        ],
        env: {
          ...process.env,
          HOME: homeDirFromEnv({ env, }),
        },
      },);
      if (result.code === 0) {
        installed = true;
        break;
      }
    }
    if (!installed) {
      throw new Error(
        `bin install failed for ${
          entry.id
        } (${
          name
        }): ${
          result.stderr || result.stdout
        }`,
      );
    }
  }
  return result;
}

//endregion bin-sourced provider

//region cargo-binstall provider

/**
 Installs one `cargo-binstall` route tool at its locked version
 (the repository boundary install the maintainer confirmed).
 
 @param context - Provider context
 
 @returns The `cargo binstall` run result
 */
export async function installCargoBinstall(
  context: ProviderContext,
): Promise<RunResult> {
  const {
    entry,
    paths,
    env,
  } = context;
  /**
   `cargo binstall` arguments with the exact locked version.
   */
  const args = [
    'binstall',
    '--no-confirm',
    '--version',
    entry.version,
  ];
  for (const feature of entry.options.cargoFeatures ?? []) {
    args.push(
      '--features',
      feature,
    );
  }
  args.push(entry.packageId ?? entry.id,);
  /**
   Install run result through the toolchain's own manager directories.
   */
  const result = await run({
    command: 'cargo',
    args,
    env: toolchainPathEnv({
      paths,
      env,
    },),
  },);
  if (result.code !== 0) {
    throw new Error(
      `cargo binstall failed for ${
        entry.id
      }: ${
        result.stderr || result.stdout
      }`,
    );
  }
  return result;
}

//endregion cargo-binstall provider

//region pnpm and uvx providers

/**
 Installs one `mpm-pnpm` route tool at its locked version
 (`pnpm add --global <name>@<version>`, the pin the shipped MPM `pnpm`
 adapter cannot forward).
 
 @param context - Provider context
 
 @returns The `pnpm` run result
 */
export async function installMpmPnpm(
  context: ProviderContext,
): Promise<RunResult> {
  const {
    entry,
    paths,
    env,
    extraDirs,
  } = context;
  /**
   Toolchain `pnpm` the install must run through; retired machine shims
   must never receive the install.
   */
  const pnpmBin = findToolBin({
    entry: toolById('pnpm',),
    name: 'pnpm',
    paths,
    env,
    extraDirs,
  },);
  if (pnpmBin === null) {
    throw new Error(
      'pnpm is not installed in the toolchain; the pnpm stage must run first',
    );
  }
  /**
   Global install run result.
   */
  const result = await run({
    command: pnpmBin,
    args: [
      'add',
      '--global',
      `${entry.packageId ?? entry.id}@${entry.version}`,
    ],
    env: toolchainPathEnv({
      paths,
      env,
    },),
  },);
  if (result.code !== 0) {
    throw new Error(
      `pnpm add --global failed for ${
        entry.id
      }: ${
        result.stderr || result.stdout
      }`,
    );
  }
  return result;
}

/**
 Installs one `mpm-uvx` route tool at its locked version
 (`uv tool install <name>==<version>`).
 
 @param context - Provider context
 
 @returns The `uv tool install` run result
 */
export async function installMpmUvx(
  context: ProviderContext,
): Promise<RunResult> {
  const {
    entry,
    paths,
    env,
    extraDirs,
  } = context;
  /**
   Toolchain `uv` the install must run through; retired machine shims must
   never receive the install.
   */
  const uvBin = findToolBin({
    entry: toolById('uv',),
    name: 'uv',
    paths,
    env,
    extraDirs,
  },);
  if (uvBin === null) {
    throw new Error(
      'uv is not installed in the toolchain; the uv stage must run first',
    );
  }
  if (entry.options.bootstrap?.distribution === 'wheel') {
    // The pinned `mpm` wheel is downloaded and SHA-256 checked first.
    /**
     Verified wheel for the host platform.
     */
    const artifact = artifactForHost(entry,);
    /**
     Download path inside the toolchain root.
     */
    const wheelPath = join(
      paths.toolchainRoot,
      'downloads',
      basename(artifact.url,),
    );
    await downloadVerified({
      url: artifact.url,
      target: wheelPath,
      sha256: artifact.sha256,
    },);
    /**
     Wheel install run result.
     */
    const wheelInstall = await run({
      command: uvBin,
      args: [
        'tool',
        'install',
        '--force',
        wheelPath,
      ],
      env: toolchainPathEnv({
        paths,
        env,
      },),
    },);
    if (wheelInstall.code !== 0) {
      throw new Error(
        `uv tool install failed for ${
          entry.id
        }: ${
          wheelInstall.stderr || wheelInstall.stdout
        }`,
      );
    }
    return wheelInstall;
  }
  /**
   Registry install run result.
   */
  const installed = await run({
    command: uvBin,
    args: [
      'tool',
      'install',
      `${entry.packageId ?? entry.id}==${entry.version}`,
    ],
    env: toolchainPathEnv({
      paths,
      env,
    },),
  },);
  if (installed.code !== 0) {
    throw new Error(
      `uv tool install failed for ${
        entry.id
      }: ${
        installed.stderr || installed.stdout
      }`,
    );
  }
  return installed;
}

//endregion pnpm and uvx providers

//region Rust and Android providers

/**
 Installs the Rust toolchain through MPM's `rustup` manager (the capable
 `mpm restore` slice; toolchain ids carry the exact channel), falling back to
 `rustup toolchain install` with the same id.
 
 @param context - Provider context
 
 @returns The run result of whichever path converged the toolchain
 */
export async function installRustToolchain(
  context: ProviderContext,
): Promise<RunResult> {
  const {
    entry,
    paths,
    env,
    extraDirs,
  } = context;
  /**
   Toolchain `mpm`, when the mpm bootstrap staged it.
   */
  const mpmBin = findToolBin({
    entry: MPM_BOOTSTRAP,
    name: 'mpm',
    paths,
    env,
    extraDirs,
  },);
  if (mpmBin !== null) {
    /**
     `mpm restore` limited to the rustup manager.
     */
    const viaMpm = await run({
      command: mpmBin,
      args: [
        '--rustup',
        'restore',
        paths.mpmPackagesFile,
      ],
      env: toolchainPathEnv({
        paths,
        env,
      },),
    },);
    if (viaMpm.code === 0)
      return viaMpm;
  }
  /**
   Direct rustup fallback for the same locked channel.
   */
  const viaRustup = await run({
    command: 'rustup',
    args: [
      'toolchain',
      'install',
      entry.packageId ?? entry.version,
    ],
    env: toolchainPathEnv({
      paths,
      env,
    },),
  },);
  if (viaRustup.code !== 0) {
    throw new Error(
      `rustup toolchain install failed for ${
        entry.id
      }: ${
        viaRustup.stderr || viaRustup.stdout
      }`,
    );
  }
  return viaRustup;
}

/**
 Adds the plan's Rust components and Android targets to the locked toolchain.
 
 @param context - Provider context
 
 @returns The last `rustup` run result
 */
export async function installRustComponents(
  context: ProviderContext,
): Promise<RunResult> {
  const {
    entry,
    paths,
    env,
  } = context;
  /**
   Last observed run result.
   */
  let result: RunResult = {
    stdout: '',
    stderr: '',
    code: 0,
  };
  for (const component of entry.options.rustComponents ?? []) {
    result = await run({
      command: 'rustup',
      args: [
        'component',
        'add',
        component,
        '--toolchain',
        entry.version,
      ],
      env: toolchainPathEnv({
        paths,
        env,
      },),
    },);
    if (result.code !== 0) {
      throw new Error(
        `rustup component add failed for ${
          entry.id
        } (${
          component
        }): ${
          result.stderr || result.stdout
        }`,
      );
    }
  }
  for (const target of entry.options.rustTargets ?? []) {
    result = await run({
      command: 'rustup',
      args: [
        'target',
        'add',
        target,
        '--toolchain',
        entry.version,
      ],
      env: toolchainPathEnv({
        paths,
        env,
      },),
    },);
    if (result.code !== 0) {
      throw new Error(
        `rustup target add failed for ${
          entry.id
        } (${
          target
        }): ${
          result.stderr || result.stdout
        }`,
      );
    }
  }
  return result;
}

/**
 Installs the Android SDK: cmdline-tools from the locked archive, then the
 plan's package pins through the SDK's own `android` CLI
 (`android sdk install <package>...`).
 
 @param context - Provider context
 
 @returns The last `android` run result
 */
export async function installAndroidSdk(
  context: ProviderContext,
): Promise<RunResult> {
  const { entry, paths, env, } = context;
  /**
   Verified cmdline-tools archive for the host platform.
   */
  const artifact = artifactForHost(entry,);
  /**
   Download path inside the toolchain root.
   */
  const downloadPath = join(
    paths.toolchainRoot,
    'downloads',
    `android-sdk-${entry.version}-${basename(artifact.url,)}`,
  );
  await downloadVerified({
    url: artifact.url,
    target: downloadPath,
    sha256: artifact.sha256,
  },);
  /**
   Machine Android SDK root (`<root>/android-sdk/<version>`).
   */
  const sdkRoot = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: entry.version,
  });
  /**
   Staging directory the archive extracts into (its payload is a
   `cmdline-tools/` tree).
   */
  const staging = join(
    sdkRoot,
    '.cmdline-tools-staging',
  );
  rmSync(
    staging,
    {
      recursive: true,
      force: true,
    },
  );
  await extractArchive({
    archive: downloadPath,
    into: staging,
  },);
  mkdirSync(
    join(
      sdkRoot,
      'cmdline-tools',
    ),
    { recursive: true },
  );
  renameSync(
    join(
      staging,
      'cmdline-tools',
    ),
    join(
      sdkRoot,
      'cmdline-tools',
      entry.version,
    ),
  );
  rmSync(
    staging,
    {
      recursive: true,
      force: true,
    },
  );
  /**
   `android` CLI inside the installed cmdline-tools.
   */
  const androidCli = join(
    sdkRoot,
    'cmdline-tools',
    entry.version,
    'bin',
    'android',
  );
  /**
   SDK install run result.
   */
  const result = await run({
    command: androidCli,
    args: [
      'sdk',
      'install',
      ...(entry.options.androidPackages ?? []),
    ],
    env: {
      ...toolchainPathEnv({
        paths,
        env,
      },),
      ANDROID_HOME: sdkRoot,
      ANDROID_SDK_ROOT: sdkRoot,
    },
  },);
  if (result.code !== 0) {
    throw new Error(
      `android sdk install failed for ${
        entry.id
      }: ${
        result.stderr || result.stdout
      }`,
    );
  }
  return result;
}

//endregion Rust and Android providers

//region Probes

/**
 Reads one `bin list` output table into name to version pairs.
 
 @param output - `bin list` stdout
 
 @returns Map from binary file name to reported version
 */
export function parseBinList(output: string,): ReadonlyMap<string, string> {
  /**
   Parsed name to version pairs.
   */
  const versions = new Map<string, string>();
  for (const line of output.split('\n',)) {
    /**
     One `Path  Version  URL  Status` row.
     */
    const match = line.match(/^(\S.*?)\s{2,}\*?v?(\S+)\s{2,}/u);
    if (match === null)
      continue;
    /**
     Path and version captured from the row.
     */
    const [, rowPath, rowVersion,] = match;
    if (rowPath === undefined || rowVersion === undefined)
      continue;
    versions.set(
      basename(rowPath,),
      rowVersion,
    );
  }
  return versions;
}

/**
 Reads `cargo binstall --list` output into crate to version pairs.
 
 @param output - `cargo binstall --list` stdout
 
 @returns Map from crate name to installed version
 */
export function parseCargoBinstallList(
  output: string,
): ReadonlyMap<string, string> {
  /**
   Parsed crate to version pairs.
   */
  const versions = new Map<string, string>();
  for (const line of output.split('\n',)) {
    /**
     One `Package  Cargo Ver  Binstall Ver  Method` row.
     */
    const match = line.match(/^(\S+)\s+v?(\S+)\s+(\S+)\s+\S/u);
    if (match === null || match[1] === 'Package' || match[1] === 'Total:')
      continue;
    /**
     Crate name and version columns of the row.
     */
    const [, crate, cargoVer, binstallVer,] = match;
    if (crate === undefined || cargoVer === undefined || binstallVer === undefined)
      continue;
    versions.set(
      crate,
      binstallVer === '-'
        ? cargoVer
        : binstallVer,
    );
  }
  return versions;
}

//endregion Probes
