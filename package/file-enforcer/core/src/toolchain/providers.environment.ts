/**
 Tool and environment resolution shared by the providers: the GitHub token,
 the `bin` CLI path, and the environment manager commands run under.
 */

import {
  BIN_NOT_FOUND,
  findToolBin,
} from './layout.ts';
import { run, } from './exec.ts';
import {
  homeDirFromEnv,
  managerNativeBinDirs,
  toolInstallDir,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';
import { planTool, } from './providers.context.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Tool resolution

/**
 Key of the single cached GitHub token lookup.
 */
const GITHUB_TOKEN_CACHE_KEY = 'github-token';

/**
 Marks that no GitHub token is available; callers then make unauthenticated requests.
 */
export const NO_GITHUB_TOKEN: unique symbol = Symbol('no GitHub token available',);

/**
 Cached GitHub token for `bin` release lookups; the unauthenticated API
 quota cannot cover one full sync.
 */
const githubTokenCache = new Map<typeof GITHUB_TOKEN_CACHE_KEY, string | typeof NO_GITHUB_TOKEN>();

/**
 Resolves a GitHub token: the environment when present, else the `gh`
 login on this machine (best effort).

 @param env - Environment to read `GITHUB_TOKEN` from

 @returns Token string, or {@link NO_GITHUB_TOKEN} when none is available

 @example
 ```ts
 const token = await resolveGithubToken({ env: process.env, },);
 ```
 */
export async function resolveGithubToken(
  {
    env,
  }: {
    readonly env: PathEnv;
  },
): Promise<string | typeof NO_GITHUB_TOKEN> {
  /**
   Earlier lookup result, when this process already resolved a token.
   */
  const cached = githubTokenCache.get(GITHUB_TOKEN_CACHE_KEY,);
  if (cached !== undefined)
    return cached;
  /**
   Token from the environment, which CI and interactive shells set.
   */
  const fromEnv = env.GITHUB_TOKEN
    ?? process.env
      .GITHUB_TOKEN;
  if ((fromEnv !== undefined) && (fromEnv !== '')) {
    githubTokenCache.set(
      GITHUB_TOKEN_CACHE_KEY,
      fromEnv,
    );
    return fromEnv;
  }
  /**
   Token from the machine's `gh` login, when it has one.
   */
  const viaGh = await run({
    command: 'gh',
    args: [
      'auth',
      'token'
    ],
  },);
  /**
   `gh` output without the trailing newline.
   */
  const ghOutput = viaGh.stdout
    .trim();
  /**
   Trimmed `gh` output, or the no-token marker when `gh` failed or printed nothing.
   */
  const fromGh = (viaGh.code === 0) && (ghOutput !== '')
    ? ghOutput
    : NO_GITHUB_TOKEN;
  githubTokenCache.set(
    GITHUB_TOKEN_CACHE_KEY,
    fromGh,
  );
  return fromGh;
}

/**
 Absolute path of the `bin` CLI at the bootstrap's own install directory.

 The archive route stages the CLI under `<root>/bin/<version>/`; install and
 status calls must not assume the flat `<root>/bin/<name>` layout the CLI
 itself uses for the tools it downloads.

 @param paths - Toolchain path set

 @param env - Environment the lookup reads

 @param tools - Scope plan; absent means the repository plan

 @returns Absolute CLI path

 @throws Error when the CLI is not installed

 @example
 ```ts
 const cli = await binCliPath({ paths, },);
 ```
 */
export async function binCliPath(
  {
    paths,
    env,
    tools,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv;
    readonly tools?: readonly ToolPlanEntry[];
  },
): Promise<string> {
  /**
   Bootstrap entry the CLI path is expected under.
   */
  const bootstrap = planTool({
    ...(tools === undefined
      ? {}
      : { tools, }),
    id: 'bin',
  },);
  /**
   Resolved CLI from the bootstrap entry's install directory.
   */
  const resolved = await findToolBin({
    entry: bootstrap,
    name: 'bin',
    paths,
    ...(env === undefined
      ? {}
      : { env, }),
  },);
  if (resolved === BIN_NOT_FOUND) {
    throw new Error(
      `bin CLI is not installed; expected it under ${
        toolInstallDir({
          toolchainRoot: paths.toolchainRoot,
          id: bootstrap.id,
          version: bootstrap.version,
        },)
      }`,
    );
  }
  return resolved;
}

/**
 Environment for manager commands: the toolchain's own bin directories lead
 `PATH`, so retired machine shims cannot divert an install run.

 @param paths - Toolchain path set

 @param env - Environment the result derives from; absent means the process

 @returns Environment whose `PATH` starts with the toolchain directories

 @example
 ```ts
 const managerEnv = toolchainPathEnv({ paths, },);
 ```
 */
export function toolchainPathEnv(
  {
    paths,
    env,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv;
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
