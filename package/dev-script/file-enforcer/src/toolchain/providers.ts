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
  toolInstallDir,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';
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
  copyFileSync(
    downloadPath,
    join(
      installDir,
      name,
    ),
  );
  makeExecutable(join(
    installDir,
    name,
  ),);
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
  for (const candidate of candidates) {
    if (existsSync(candidate,))
      return candidate;
  }
  /**
   First candidate `bin` would resolve on a clean machine.
   */
  /**
   First candidate `bin` resolves on a clean machine.
   */
  const target = candidates.at(-2) ?? candidates.at(-1);
  if (target === undefined)
    throw new Error('no bin configuration path is derivable from the environment',);
  mkdirSync(
    dirname(target,),
    { recursive: true },
  );
  writeFileSync(
    target,
    `${
      JSON.stringify(
        {
          default_path: downloadDir,
          bins: {},
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
        command: join(
          paths.toolchainBinDir,
          'bin',
        ),
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
    if (!installed)
      return result;
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
  const { entry, } = context;
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
  return run({
    command: 'cargo',
    args,
  },);
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
  const { entry, } = context;
  return run({
    command: 'pnpm',
    args: [
      'add',
      '--global',
      `${entry.packageId ?? entry.id}@${entry.version}`,
    ],
  },);
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
  const { entry, paths, env, } = context;
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
    return run({
      command: 'uv',
      args: [
        'tool',
        'install',
        '--force',
        wheelPath,
      ],
      env: {
        ...process.env,
        HOME: homeDirFromEnv({ env, }),
      },
    },);
  }
  return run({
    command: 'uv',
    args: [
      'tool',
      'install',
      `${entry.packageId ?? entry.id}==${entry.version}`,
    ],
    env: {
      ...process.env,
      HOME: homeDirFromEnv({ env, }),
    },
  },);
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
  const { entry, paths, env, } = context;
  /**
   `mpm restore` limited to the rustup manager.
   */
  const viaMpm = await run({
    command: join(
      paths.toolchainBinDir,
      'mpm',
    ),
    args: [
      '--rustup',
      'restore',
      paths.mpmPackagesFile,
    ],
    env: {
      ...process.env,
      HOME: homeDirFromEnv({ env, }),
    },
  },);
  if (viaMpm.code === 0)
    return viaMpm;
  return run({
    command: 'rustup',
    args: [
      'toolchain',
      'install',
      entry.packageId ?? entry.version,
    ],
  },);
}

/**
 Adds the plan's Rust components and Android targets to the locked toolchain.
 
 @param context - Provider context
 
 @returns The last `rustup` run result
 */
export async function installRustComponents(
  context: ProviderContext,
): Promise<RunResult> {
  const { entry, } = context;
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
    },);
    if (result.code !== 0)
      return result;
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
    },);
    if (result.code !== 0)
      return result;
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
  return run({
    command: androidCli,
    args: [
      'sdk',
      'install',
      ...(entry.options.androidPackages ?? []),
    ],
    env: {
      ...process.env,
      ANDROID_HOME: sdkRoot,
      ANDROID_SDK_ROOT: sdkRoot,
      HOME: homeDirFromEnv({ env, }),
    },
  },);
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
