/**
 The Rust and Android SDK routes: the rustup toolchain with its components
 and targets, and the Android command-line tools with their package pins.
 */

import {
  mkdir,
  rename,
  rm,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import {
  downloadVerified,
  extractArchive,
  run,
  type RunResult,
} from './exec.ts';
import { findToolBin, } from './layout.ts';
import { toolInstallDir, } from './paths.ts';
import {
  artifactForHost,
  planTool,
  type ProviderContext,
} from './providers.context.ts';
import { toolchainPathEnv, } from './providers.environment.ts';

//region Rust providers

/**
 Installs the Rust toolchain through MPM's `rustup` manager (the capable
 `mpm restore` slice; toolchain ids carry the exact channel), falling back to
 `rustup toolchain install` with the same id.

 @param context - Provider context

 @returns The run result of whichever path converged the toolchain

 @throws Error when the `rustup` fallback fails

 @example
 ```ts
 const outcome = await installRustToolchain({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installRustToolchain(
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
    tools,
  } = context;
  /**
   Toolchain `mpm`, when the mpm bootstrap staged it.
   */
  const mpmBin = findToolBin({
    entry: planTool({
      tools,
      id: 'mpm',
    },),
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

 @throws Error when a component or target fails to install

 @example
 ```ts
 const outcome = await installRustComponents({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installRustComponents(
  context: ProviderContext,
): Promise<RunResult> {
  /**
   Plan entry, path set and environment this install reads.
   */
  const {
    entry,
    paths,
    env,
  } = context;
  /**
   Last observed run result.
   */
  const results: RunResult[] = [{
    stdout: '',
    stderr: '',
    code: 0,
  }];
  for (const component of entry.options
    .rustComponents
    ?? []) {
    /**
     Run of `rustup component add` for this component.
     */
    // oxlint-disable-next-line no-await-in-loop -- rustup locks the toolchain while it changes it, so additions must not overlap
    const added = await run({
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
    if (added.code !== 0) {
      throw new Error(
        `rustup component add failed for ${
          entry.id
        } (${
          component
        }): ${
          added.stderr || added.stdout
        }`,
      );
    }
    results.push(added,);
  }
  for (const target of entry.options
    .rustTargets
    ?? []) {
    /**
     Run of `rustup target add` for this target.
     */
    // oxlint-disable-next-line no-await-in-loop -- rustup locks the toolchain while it changes it, so additions must not overlap
    const added = await run({
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
    if (added.code !== 0) {
      throw new Error(
        `rustup target add failed for ${
          entry.id
        } (${
          target
        }): ${
          added.stderr || added.stdout
        }`,
      );
    }
    results.push(added,);
  }
  return results.at(-1,) ?? {
    stdout: '',
    stderr: '',
    code: 0,
  };
}

//endregion Rust providers

//region Android provider

/**
 Installs the Android SDK: cmdline-tools from the locked archive, then the
 plan's package pins through the SDK's own `android` CLI
 (`android sdk install <package>...`).

 @param context - Provider context

 @returns The last `android` run result

 @throws Error when the archive cannot be verified or extracted, or the SDK
 install fails

 @example
 ```ts
 const outcome = await installAndroidSdk({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installAndroidSdk(
  context: ProviderContext,
): Promise<RunResult> {
  /**
   Plan entry, path set and environment this install reads.
   */
  const {
    entry,
    paths,
    env,
  } = context;
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
  await rm(
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
  await mkdir(
    join(
      sdkRoot,
      'cmdline-tools',
    ),
    { recursive: true },
  );
  await rename(
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
  await rm(
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
      ...(entry.options
        .androidPackages
        ?? []),
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

//endregion Android provider
