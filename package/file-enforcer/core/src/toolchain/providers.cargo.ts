/**
 The `cargo-binstall` and `cargo-git` routes: crates installed at a locked
 version or a pinned git revision.
 */

import { mkdir, } from 'node:fs/promises';
import {
  run,
  type RunResult,
} from './exec.ts';
import { toolInstallDir, } from './paths.ts';
import type { ProviderContext, } from './providers.context.ts';
import {
  NO_GITHUB_TOKEN,
  resolveGithubToken,
  toolchainPathEnv,
} from './providers.environment.ts';

//region cargo-binstall provider

/**
 Installs one `cargo-binstall` route tool at its locked version
 (the repository boundary install the maintainer confirmed).

 @param context - Provider context

 @returns The `cargo binstall` run result

 @throws Error when the install command fails

 @example
 ```ts
 const outcome = await installCargoBinstall({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installCargoBinstall(
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
   `cargo binstall` arguments with the exact locked version.
   */
  const args = [
    'binstall',
    '--no-confirm',
    '--version',
    entry.version,
  ];
  for (const feature of entry.options
    .cargoFeatures
    ?? []) {
    // The equals form keeps the crate positional from being consumed as
    // another `--features` value (clap multi-value parsing).
    args.push(`--features=${feature}`);
  }
  args.push(entry.packageId ?? entry.id,);
  /**
   GitHub token so `cargo binstall` release lookups do not hit the unauthenticated API quota.
   Without it, an exhausted quota stalls resolution past its timeout and binstall silently compiles from source
   (`doc/troubleshooting/cargo-binstall-compiles-when-github-api-rate-limited.md`).
   */
  const token = await resolveGithubToken({ env, },);
  /**
   Install run result through the toolchain's own manager directories.
   */
  const result = await run({
    command: 'cargo',
    args,
    env: {
      ...toolchainPathEnv({
        paths,
        env,
      },),
      ...(token === NO_GITHUB_TOKEN
        ? {}
        : { GITHUB_TOKEN: token, }),
    },
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

//region cargo-git provider

/**
 Installs one `cargo-git` route tool: one package built from a git source at
 the pinned revision (`cargo install --git <url> --rev <rev> --root <dir>`),
 scoped into the tool's own install directory.

 @param context - Provider context

 @returns The `cargo install` run result

 @throws Error when the entry carries no git source or the build fails

 @example
 ```ts
 const outcome = await installCargoGit({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installCargoGit(
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
   Git source recorded for this entry.
   */
  const url = entry.options
    .cargoGit
    ?.url;
  if (url === undefined) {
    throw new Error(
      `toolchain plan entry ${
        entry.id
      } carries no cargoGit source url`,
    );
  }
  /**
   Pinned revision from the request (`ref:<rev>`) or the version literal.
   */
  const pinned = 'pinned' in entry.request
    ? entry.request
      .pinned
    : entry.version;
  /**
   Bare revision the `--rev` argument takes.
   */
  const rev = pinned.startsWith('ref:')
    ? pinned.slice('ref:'.length,)
    : pinned;
  /**
   Machine install directory of this tool (`cargo install --root` puts its
   `bin/` level inside).
   */
  const installDir = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: entry.version,
  },);
  await mkdir(
    installDir,
    { recursive: true },
  );
  /**
   Install run result through the toolchain's own manager directories.
   */
  const result = await run({
    command: 'cargo',
    args: [
      'install',
      '--git',
      url,
      '--rev',
      rev,
      '--root',
      installDir,
      entry.packageId ?? entry.id,
    ],
    env: toolchainPathEnv({
      paths,
      env,
    },),
  },);
  if (result.code !== 0) {
    throw new Error(
      `cargo install --git failed for ${
        entry.id
      }: ${
        result.stderr || result.stdout
      }`,
    );
  }
  return result;
}

//endregion cargo-git provider
