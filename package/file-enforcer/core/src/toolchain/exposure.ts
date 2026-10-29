/**
 Exposure directory rebuild, tool environment exports, and postinstall links.
 */

import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import {
  dirname,
  isAbsolute,
  join,
} from 'node:path';
import { atomicSymlink, } from './exec.ts';
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
 
 @param options - Paths, environment, plan entries and extra bin directories
 
 @returns Which declared names linked and which are missing
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
    readonly extraDirs?: readonly string[] | undefined;
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
 
 @param options - Paths, plan entries and environment
 
 @returns The environment record written
 */
export function writeToolEnvFile(
  {
    paths,
    entries,
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
  },
): Readonly<Record<string, string>> {
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
  mkdirSync(
    dirname(paths.envFile,),
    { recursive: true },
  );
  /**
   Temporary file beside the final env file.
   */
  const temp = `${paths.envFile}.${
    process.pid
  }.tmp`;
  writeFileSync(
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
  renameSync(
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
 
 @param options - Paths, plan entries and environment
 
 @returns Names of the links created
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
    readonly extraDirs?: readonly string[] | undefined;
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
    const {postinstall} = entry.options;
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
 
 @param options - Paths and plan entries
 
 @returns Declared names that do not resolve
 */
export function unresolvableBins(
  {
    paths,
    entries,
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
  },
): readonly string[] {
  /**
   Names that do not resolve.
   */
  const missing: string[] = [];
  for (const entry of entries) {
    for (const name of entry.bins) {
      if (!existsSync(join(
        paths.exposureDir,
        name,
      ),))
        missing.push(name,);
    }
  }
  return missing;
}

//endregion Verification

/**
 Reads a generated `.cache/toolchain.env.json` back.
 
 @param paths - Resolved toolchain paths
 
 @returns The recorded environment, or `null` when the file is absent
 */
export function readToolEnvFile(
  paths: ToolchainPaths,
): Readonly<Record<string, string>> | null {
  if (!existsSync(paths.envFile,))
    return null;
  return JSON.parse(readFileSync(
    paths.envFile,
    'utf8',
  ),) as Readonly<Record<string, string>>;
}

/**
 Removes the per-worktree exposure directory contents (used by tests and
 rebuilds). The directory itself is kept.
 
 @param paths - Resolved toolchain paths
 */
export function clearExposure(paths: ToolchainPaths,): void {
  rmSync(
    paths.exposureDir,
    {
      recursive: true,
      force: true,
    },
  );
}
