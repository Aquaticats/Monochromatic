/**
 The manager routes that build or install through a staged toolchain
 manager: `go-install`, `mpm-pnpm` and `mpm-uvx`.
 */

import { mkdir, } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import {
  downloadVerified,
  run,
  type RunResult,
} from './exec.ts';
import {
  BIN_NOT_FOUND,
  findToolBin,
} from './layout.ts';
import { toolInstallDir, } from './paths.ts';
import { ALL_TOOLS, } from './plan.ts';
import {
  artifactForHost,
  planTool,
  type ProviderContext,
} from './providers.context.ts';
import { toolchainPathEnv, } from './providers.environment.ts';

//region go-install provider

/**
 Installs one `go-install` route tool: one Go module built at its locked
 version into a scoped `GOBIN`
 (`go install <module>@v<version>`),
 preserving the build-from-source semantics the declaration names.

 @param context - Provider context

 @returns The `go install` run result

 @throws Error when `go` is unstaged or the build fails

 @example
 ```ts
 const outcome = await installGoInstall({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installGoInstall(
  context: ProviderContext,
): Promise<RunResult> {
  /**
   Plan entry, path set, environment, extra bin directories and scope plan
   this install reads.
   */
  const {
    entry,
    paths,
    env,
    extraDirs,
    tools = ALL_TOOLS,
  } = context;
  /**
   Toolchain `go` the build must run through; retired machine shims must
   never receive the install.
   */
  const goBin = await findToolBin({
    entry: planTool({
      tools,
      id: 'go',
    },),
    name: 'go',
    paths,
    env,
    extraDirs,
  },);
  if (goBin === BIN_NOT_FOUND) {
    throw new Error(
      'go is not installed in the toolchain; the go stage must run first',
    );
  }
  /**
   Machine install directory of this tool; `GOBIN` is its `bin/` level.
   */
  const installDir = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: entry.version,
  },);
  /**
   Scoped `GOBIN` the module's binaries land in.
   */
  const gobin = join(
    installDir,
    'bin',
  );
  await mkdir(
    gobin,
    { recursive: true },
  );
  /**
   Module version the `go install` argument takes (`@v<version>`).
   */
  const moduleVersion = entry.version
    .startsWith('v',)
    ? entry.version
    : `v${entry.version}`;
  /**
   Build run result.
   */
  const result = await run({
    command: goBin,
    args: [
      'install',
      `${entry.packageId ?? entry.id}@${moduleVersion}`,
    ],
    env: {
      ...toolchainPathEnv({
        paths,
        env,
      },),
      GOBIN: gobin,
    },
  },);
  if (result.code !== 0) {
    throw new Error(
      `go install failed for ${
        entry.id
      }: ${
        result.stderr || result.stdout
      }`,
    );
  }
  return result;
}

//endregion go-install provider

//region pnpm and uvx providers

/**
 Installs one `mpm-pnpm` route tool at its locked version
 (`pnpm add --global <name>@<version>`, the pin the shipped MPM `pnpm`
 adapter cannot forward).

 @param context - Provider context

 @returns The `pnpm` run result

 @throws Error when `pnpm` is unstaged or the install fails

 @example
 ```ts
 const outcome = await installMpmPnpm({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installMpmPnpm(
  context: ProviderContext,
): Promise<RunResult> {
  /**
   Plan entry, path set, environment, extra bin directories and scope plan
   this install reads.
   */
  const {
    entry,
    paths,
    env,
    extraDirs,
    tools = ALL_TOOLS,
  } = context;
  /**
   Toolchain `pnpm` the install must run through; retired machine shims
   must never receive the install.
   */
  const pnpmBin = await findToolBin({
    entry: planTool({
      tools,
      id: 'pnpm',
    },),
    name: 'pnpm',
    paths,
    env,
    extraDirs,
  },);
  if (pnpmBin === BIN_NOT_FOUND) {
    throw new Error(
      'pnpm is not installed in the toolchain; the pnpm stage must run first',
    );
  }
  /**
   Global install arguments: the pinned spec plus the build-script
   allowances (`--allow-build`, repeatable).
   */
  const args = [
    'add',
    '--global',
  ];
  for (const allowed of entry.options
    .allowBuilds
    ?? []) {
    args.push(
      '--allow-build',
      allowed,
    );
  }
  args.push(`${entry.packageId ?? entry.id}@${entry.version}`,);
  /**
   Global install run result.
   */
  const result = await run({
    command: pnpmBin,
    args,
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

 @throws Error when `uv` is unstaged or the install fails

 @example
 ```ts
 const outcome = await installMpmUvx({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installMpmUvx(
  context: ProviderContext,
): Promise<RunResult> {
  /**
   Plan entry, path set, environment, extra bin directories and scope plan
   this install reads.
   */
  const {
    entry,
    paths,
    env,
    extraDirs,
    tools = ALL_TOOLS,
  } = context;
  /**
   Toolchain `uv` the install must run through; retired machine shims must
   never receive the install.
   */
  const uvBin = await findToolBin({
    entry: planTool({
      tools,
      id: 'uv',
    },),
    name: 'uv',
    paths,
    env,
    extraDirs,
  },);
  if (uvBin === BIN_NOT_FOUND) {
    throw new Error(
      'uv is not installed in the toolchain; the uv stage must run first',
    );
  }
  if (entry.options
    .bootstrap
    ?.distribution
    === 'wheel') {
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
   Registry install run result; `--force` converges executables an earlier
   machine install (retired Mise-era `~/.local/bin` links) still holds.
   */
  const installed = await run({
    command: uvBin,
    args: [
      'tool',
      'install',
      '--force',
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
