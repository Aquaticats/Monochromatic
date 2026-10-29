/**
 Exposure directory rebuild and postinstall links.
 */

import {
  mkdir,
  rm,
} from 'node:fs/promises';
import {
  delimiter,
  isAbsolute,
  join,
} from 'node:path';
import {
  atomicSymlink,
  run,
} from './exec.ts';
import { pathExists, } from './fs-probe.ts';
import {
  BIN_NOT_FOUND,
  findToolBin,
} from './layout.ts';
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
 const report = await rebuildExposure({ paths, env: process.env, entries: ALL_TOOLS, },);
 report.missing; // []
 ```
 */
export async function rebuildExposure(
  {
    paths,
    env,
    entries,
    extraDirs = [],
  }: {
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
    readonly entries: readonly ToolPlanEntry[];
    readonly extraDirs?: readonly string[];
  },
): Promise<ExposureReport> {
  await mkdir(
    paths.exposureDir,
    { recursive: true },
  );
  /**
   Every declared bin name with the entry that declares it, in plan order.
   */
  const declarations = entries.flatMap(
    function declaredBins(entry,): readonly {
      readonly entry: ToolPlanEntry;
      readonly name: string;
    }[] {
      return entry.bins
        .map(
        function declaration(name,): {
          readonly entry: ToolPlanEntry;
          readonly name: string;
        } {
          return {
            entry,
            name,
          };
        },
        );
    },
  );
  /**
   Outcome of each declared name, in the same order as `declarations`; the links are independent, so they
   resolve and land concurrently.
   */
  const outcomes = await Promise.all(declarations.map(
    async function linkDeclared(
      {
        entry,
        name,
      },
    ): Promise<{
      readonly name: string;
      readonly linked: boolean;
    }> {
      /**
       Alias map of this entry.
       */
      const aliases = entry.options
        .binAliases
        ?? {};
      /**
       Primary binary this name resolves through.
       */
      const primary = aliases[name] ?? name;
      /**
       Absolute target behind the primary name.
       */
      const target = await findToolBin({
        entry,
        name: primary,
        paths,
        env,
        extraDirs,
      },);
      if (target === BIN_NOT_FOUND) {
        return {
          name,
          linked: false,
        };
      }
      await atomicSymlink({
        linkPath: join(
          paths.exposureDir,
          name,
        ),
        target,
      },);
      return {
        name,
        linked: true,
      };
    },
  ),);
  /**
   Names that linked successfully.
   */
  const linked: string[] = [];
  /**
   Names with no resolvable target.
   */
  const missing: string[] = [];
  for (const outcome of outcomes) {
    if (outcome.linked)
      linked.push(outcome.name,);
    else
      missing.push(outcome.name,);
  }
  return {
    linked,
    missing,
  };
}

//endregion Exposure directory

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
 await runPostinstallLinks({ paths, entries: ALL_TOOLS, env: process.env, },); // ['git']
 ```
 */
export async function runPostinstallLinks(
  {
    paths,
    entries,
    env,
    extraDirs = [],
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
    readonly env: PathEnv;
    readonly extraDirs?: readonly string[];
  },
): Promise<readonly string[]> {
  /**
   Link names created by each entry, in the same order as `entries`; each entry's link is independent, so they
   resolve and land concurrently.
   */
  const created = await Promise.all(entries.map(
    async function linkEntry(entry,): Promise<readonly string[]> {
      /**
       Postinstall link step of this entry, if any.
       */
      const { postinstall, } = entry.options;
      if ((postinstall === undefined) || (!('link' in postinstall)))
        return [];
      /**
       Absolute source binary behind the step's `sourceBin`.
       */
      const target = await findToolBin({
        entry,
        name: postinstall.link
          .sourceBin,
        paths,
        env,
        extraDirs,
      },);
      if (target === BIN_NOT_FOUND)
        return [];
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
      await atomicSymlink({
        linkPath: join(
          linkDir,
          postinstall.link
            .linkName,
        ),
        target,
      },);
      return [postinstall.link
        .linkName,];
    },
  ),);
  return created.flat();
}

/**
 Runs the plan's postinstall command steps, after the exposure directory is
 rebuilt: the command starts from the worktree root with the exposure
 directory and the toolchain bin directory first on `PATH`, so a tool that
 links other tools (`search-shadow link`) finds the locked ones.

 @param paths - Resolved toolchain paths of the scope

 @param entries - Plan entries whose postinstall steps run

 @param env - Environment the command inherits and the bin search resolves manager homes from

 @param extraDirs - Extra bin directories staged tools contribute

 @returns Ids of the entries whose command exited with status zero, and for each other entry its command line with the output

 @example
 ```ts
 await runPostinstallCommands({ paths, entries: ALL_TOOLS, env: process.env, },);
 // { succeeded: ['cargo:monochromatic-search-shadow'], failed: {} }
 ```
 */
export async function runPostinstallCommands(
  {
    paths,
    entries,
    env,
    extraDirs = [],
  }: {
    readonly paths: ToolchainPaths;
    readonly entries: readonly ToolPlanEntry[];
    readonly env: PathEnv;
    readonly extraDirs?: readonly string[];
  },
): Promise<{
  readonly succeeded: readonly string[];
  readonly failed: Readonly<Record<string, string>>;
}> {
  /**
   Entry ids whose command succeeded.
   */
  const succeeded: string[] = [];
  /**
   Entry ids to the failing command line and its output.
   */
  const failed: Record<string, string> = {};
  /**
   `PATH` of the command: exposure directory, toolchain bin directory, then the inherited entries.
   */
  const commandPath = [
    paths.exposureDir,
    paths.toolchainBinDir,
    ...(env.PATH === undefined ? [] : [env.PATH,]),
  ].join(delimiter,);
  /**
   Environment of the command, without the undefined values `PathEnv` allows.
   */
  const commandEnv: Record<string, string> = { PATH: commandPath, };
  for (const [name, text,] of Object.entries(env,)) {
    if ((text !== undefined) && (name !== 'PATH'))
      commandEnv[name] = text;
  }
  for (const entry of entries) {
    /**
     Postinstall step of this entry, if any.
     */
    const { postinstall, } = entry.options;
    if ((postinstall === undefined) || (!('run' in postinstall)))
      continue;
    /**
     Command the step runs: a bin of this entry plus its arguments.
     */
    const { run: step, } = postinstall;
    /* oxlint-disable no-await-in-loop -- commands of different entries may depend on each other's links, so one at a time */
    /**
     Absolute bin behind the step's `sourceBin`.
     */
    const target = await findToolBin({
      entry,
      name: step.sourceBin,
      paths,
      env,
      extraDirs,
    },);
    if (target === BIN_NOT_FOUND)
      continue;
    /**
     Outcome of the step's command.
     */
    const result = await run({
      command: target,
      args: step.args,
      env: commandEnv,
      cwd: paths.worktreeRoot,
    },);
    /* oxlint-enable no-await-in-loop */
    if (result.code !== 0) {
      failed[entry.id] = `postinstall command failed with exit code ${String(result.code,)}: `
        + `${target} ${
          step.args
            .join(' ',)
        }\n${result.stdout}${result.stderr}`;
      continue;
    }
    succeeded.push(entry.id,);
  }
  return {
    succeeded,
    failed,
  };
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
