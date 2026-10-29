/**
 Exposure directory rebuild, tool environment exports, and postinstall links.
 */

import { mkdirSync, } from 'node:fs';
import {
  mkdir,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  isAbsolute,
  join,
} from 'node:path';
import { atomicSymlink, } from './exec.ts';
import { pathExists, } from './fs-probe.ts';
import { findToolBin, } from './layout.ts';
import type {
  PathEnv,
  ToolchainPaths,
} from './paths.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Exposure directory

/**
 Outcome of one exposure rebuild.
 */
export type ExposureReport = {
  /**
   Declared binary names whose exposure link resolves.
   */
  readonly linked: readonly string[];
  /**
   Declared binary names with no resolvable target.
   */
  readonly missing: readonly string[];
};

/**
 Rebuilds the per-worktree exposure directory:
 one symlink per declared binary name, created atomically (temporary name
 plus rename). Alias names link the same target as their primary.

 @param paths - Resolved toolchain paths of the scope

 @param env - Environment the bin search resolves manager homes from

 @param entries - Plan entries whose declared bins get links

 @param extraDirs - Extra bin directories staged tools contribute

 @returns Which declared names linked and which are missing

 @example
 ```ts
 const report = rebuildExposure({ paths, env: process.env, entries: ALL_TOOLS, },);
 report.missing; // []
 ```
 */
export function rebuildExposure(
  {
    paths,
    env,
    entries,
    extraDirs,
  }: {
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
    readonly entries: readonly ToolPlanEntry[];
    readonly extraDirs?: readonly string[];
  },
): ExposureReport {
  mkdirSync(
    paths.exposureDir,
    { recursive: true },
  );
  /**
   Names that linked successfully.
   */
  const linked: string[] = [];
  /**
   Names with no resolvable target.
   */
  const missing: string[] = [];
  for (const entry of entries) {
    /**
     Alias map of this entry.
     */
    const aliases = entry.options
      .binAliases
      ?? {};
    for (const name of entry.bins) {
      /**
       Primary binary this name resolves through.
       */
      const primary = aliases[name] ?? name;
      /**
       Absolute target behind the primary name.
       */
      const target = findToolBin({
        entry,
        name: primary,
        paths,
        env,
        extraDirs,
      },);
      if (target === null) {
        missing.push(name,);
        continue;
      }
      atomicSymlink({
        linkPath: join(
          paths.exposureDir,
          name,
        ),
        target,
      },);
      linked.push(name,);
    }
  }
  return {
    linked,
    missing,
  };
}

//endregion Exposure directory

//region Tool environment exports

/**
 Writes the gitignored `.cache/toolchain.env.json`
 (`ANDROID_HOME`, `ANDROID_SDK_ROOT`, `JAVA_HOME`, `RUSTUP_TOOLCHAIN`).
 Values the plan records as toolchain-root relative paths are resolved to
 absolute machine paths here; literal values pass through.

 @param paths - Resolved toolchain paths of the scope

 @param entries - Plan entries whose `env` blocks are merged

 @returns The environment record written

 @example
 ```ts
 const env = await writeToolEnvFile({ paths, entries: ALL_TOOLS, },);
 env.JAVA_HOME; // '/.../jdk/25'
 ```
 */
export async function writeToolEnvFile(
  {
    paths,
    entries,
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
  },
): Promise<Readonly<Record<string, string>>> {
  /**
   Environment record assembled from every entry's `env` block.
   */
  const env: Record<string, string> = {};
  for (const entry of entries) {
    for (const [name, value,] of Object.entries(entry.env,)) {
      env[name] = value.includes('/')
        ? join(
          paths.toolchainRoot,
          value,
        )
        : value;
    }
  }
  await mkdir(
    dirname(paths.envFile,),
    { recursive: true },
  );
  /**
   Temporary file beside the final env file.
   */
  const temp = `${paths.envFile}.${
    process.pid
  }.tmp`;
  await writeFile(
    temp,
    `${
      JSON.stringify(
        env,
        null,
        2,
      )
    }\n`,
    'utf8',
  );
  await rename(
    temp,
    paths.envFile,
  );
  return env;
}

//endregion Tool environment exports

//region Postinstall

/**
 Runs the plan's postinstall link steps
 (the `cli-git-bin` pattern: atomic temporary name plus rename).

 @param paths - Resolved toolchain paths of the scope

 @param entries - Plan entries whose postinstall steps run

 @param env - Environment the bin search resolves manager homes from

 @param extraDirs - Extra bin directories staged tools contribute

 @returns Names of the links created

 @example
 ```ts
 runPostinstallLinks({ paths, entries: ALL_TOOLS, env: process.env, },); // ['git']
 ```
 */
export function runPostinstallLinks(
  {
    paths,
    entries,
    env,
    extraDirs,
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
    readonly env: PathEnv;
    readonly extraDirs?: readonly string[];
  },
): readonly string[] {
  /**
   Link names created.
   */
  const created: string[] = [];
  for (const entry of entries) {
    /**
     Postinstall link step of this entry, if any.
     */
    const { postinstall, } = entry.options;
    if (postinstall === undefined)
      continue;
    /**
     Absolute source binary behind the step's `sourceBin`.
     */
    const target = findToolBin({
      entry,
      name: postinstall.link
        .sourceBin,
      paths,
      env,
      extraDirs,
    },);
    if (target === null)
      continue;
    /**
     Absolute directory holding the link: an absolute `linkDir` verbatim, a
     worktree-relative `linkDir` under the worktree root, else the scope's own
     link directory (`paths.cliGitBinDir`).
     */
    const linkDir = postinstall.link
      .linkDir
      === undefined
      ? paths.cliGitBinDir
      : isAbsolute(postinstall.link
        .linkDir,)
        ? postinstall.link
          .linkDir
        : join(
          paths.worktreeRoot,
          postinstall.link
            .linkDir,
        );
    atomicSymlink({
      linkPath: join(
        linkDir,
        postinstall.link
          .linkName,
      ),
      target,
    },);
    created.push(postinstall.link
      .linkName,);
  }
  return created;
}

//endregion Postinstall

//region Verification

/**
 Verifies every declared `bins` entry resolves through the exposure directory.

 @param paths - Resolved toolchain paths of the scope

 @param entries - Plan entries whose declared bins are checked

 @returns Declared names that do not resolve

 @example
 ```ts
 await unresolvableBins({ paths, entries: ALL_TOOLS, },); // []
 ```
 */
export async function unresolvableBins(
  {
    paths,
    entries,
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
  },
): Promise<readonly string[]> {
  /**
   Declared names across every entry, in plan order.
   */
  const names = entries.flatMap(
    function declaredNames(entry,): readonly string[] {
      return entry.bins;
    },
  );
  /**
   Whether each declared name resolves, in the same order as `names`.
   */
  const resolves = await Promise.all(names.map(
    function nameResolves(name,): Promise<boolean> {
      return pathExists(join(
        paths.exposureDir,
        name,
      ),);
    },
  ),);
  return names.filter(
    function isUnresolved(
      _name,
      index,
    ): boolean {
      return resolves[index] !== true;
    },
  );
}

//endregion Verification

/**
 Whether a parsed JSON value is an object whose values are all strings.

 @param value - Parsed JSON value to test

 @returns `true` when every property value is a string
 */
function isStringRecord(
  value: unknown,
): value is Readonly<Record<string, string>> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,))
    && Object.values(value,)
      .every(
      function isString(propertyValue: unknown,): boolean {
        return (typeof propertyValue) === 'string';
      },
      );
}

/**
 Reads a generated `.cache/toolchain.env.json` back.

 @param paths - Resolved toolchain paths of the scope

 @returns The recorded environment; empty when the file is absent

 @throws TypeError when the file exists but is not an object of string values

 @example
 ```ts
 const env = await readToolEnvFile(paths,);
 ```
 */
export async function readToolEnvFile(
  paths: ToolchainPaths,
): Promise<Readonly<Record<string, string>>> {
  if (!(await pathExists(paths.envFile,)))
    return {};
  /**
   Parsed file content.
   */
  const parsed: unknown = JSON.parse(await readFile(
    paths.envFile,
    'utf8',
  ),);
  if (!isStringRecord(parsed,))
    throw new TypeError(`${paths.envFile} is not an object of string values`,);
  return parsed;
}

/**
 Removes the per-worktree exposure directory with its contents (used by tests and
 rebuilds); the next rebuild recreates it.

 @param paths - Resolved toolchain paths of the scope

 @example
 ```ts
 await clearExposure(paths,);
 ```
 */
export async function clearExposure(paths: ToolchainPaths,): Promise<void> {
  await rm(
    paths.exposureDir,
    {
      recursive: true,
      force: true,
    },
  );
}
