/**
 `bin-paths.ts`: prints the toolchain exposure contract as JSON.

 Consumers (the IDE language layer) run this instead of `mise bin-paths`:

 ```jsonc
 {
   "exposureDir": ".cache/toolchain-bin",
   "pathDirs": ["/abs/path/to/.cache/toolchain-bin", "/abs/path/.cargo/bin"],
   "bins": { "taplo": "/abs/path/.cache/toolchain-bin/taplo" }
 }
 ```

 The output shape is exactly that object. `exposureDir` is the literal
 worktree-relative directory name; `pathDirs` entries are absolute and listed
 only when the directory exists; `bins` maps each declared binary name to its
 absolute symlink path inside the exposure directory and lists only links that
 currently resolve. Unprovisioned toolchains exit 0 with empty `pathDirs` and
 `bins`; only an internal inconsistency (a duplicated bin name across tools)
 exits non-zero with a message on stderr. stdout is exactly the JSON, with
 nothing on stderr on success.

 Runs with any working directory: the worktree root derives from this file's
 location. Node builtins only.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/bin-paths.ts
 ```
 */

import { join, } from 'node:path';
import { pathExists, } from './fs-probe.ts';
import { ALL_TOOLS, } from './plan.ts';
import {
  EXPOSURE_DIR_RELATIVE,
  managerNativeBinDirs,
  resolveToolchainPaths,
  type PathEnv,
  type ToolchainPaths,
  isMainModule,
} from './paths.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Output shape

/**
 The `bin-paths.ts` JSON contract.
 */
export type BinPathsOutput = {
  /**
   Worktree-relative exposure directory, always `.cache/toolchain-bin`.
   */
  readonly exposureDir: '.cache/toolchain-bin';
  /**
   Absolute PATH directories that exist right now.
   */
  readonly pathDirs: readonly string[];
  /**
   Declared binary name to its absolute exposure symlink path.
   */
  readonly bins: Readonly<Record<string, string>>;
};

//endregion Output shape

//region Build

/**
 Builds the exposure contract for one worktree.

 @param paths - Resolved toolchain paths of the worktree

 @param env - Environment overriding manager homes; the process environment when absent

 @param entries - Plan entries whose declared bins are listed; the repository plan when absent

 @returns The exact JSON contract

 @throws Error when two plan entries declare the same binary name

 @example
 ```ts
 const output = await buildBinPathsOutput({ paths: resolveToolchainPaths(), env: process.env, },);
 output.exposureDir; // '.cache/toolchain-bin'
 ```
 */
export async function buildBinPathsOutput(
  {
    paths,
    env,
    entries = ALL_TOOLS,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv;
    readonly entries?: readonly ToolPlanEntry[];
  },
): Promise<BinPathsOutput> {
  /**
   Declared binary names in plan order, so duplicates are caught whether or
   not they are provisioned.
   */
  const names: string[] = [];
  /**
   Declared binary names seen so far.
   */
  const declared = new Set<string>();
  for (const entry of entries) {
    for (const name of entry.bins) {
      if (declared.has(name))
        throw new Error(
          `toolchain plan declares binary name ${
            JSON.stringify(name,)
          } more than once`,
        );
      declared.add(name,);
      names.push(name,);
    }
  }

  /**
   Whether each declared name has a resolving exposure symlink, in the same order as `names`.
   */
  const linked = await Promise.all(names.map(
    function linkResolves(name,): Promise<boolean> {
      return pathExists(join(
        paths.exposureDir,
        name,
      ),);
    },
  ),);
  /**
   Candidate PATH directories, absolute.
   */
  const candidates = managerNativeBinDirs({
    paths,
    env,
  },);
  /**
   Whether each candidate directory exists, in the same order as `candidates`.
   */
  const present = await Promise.all(candidates.map(
    function dirExists(dir,): Promise<boolean> {
      return pathExists(dir,);
    },
  ),);

  return {
    exposureDir: EXPOSURE_DIR_RELATIVE,
    pathDirs: candidates.filter(
      function directoryExists(
        _dir,
        index,
      ): boolean {
        return present[index] === true;
      },
    ),
    bins: Object.fromEntries(
      names.filter(
        function nameIsLinked(
          _name,
          index,
        ): boolean {
          return linked[index] === true;
        },
      )
        .map(
        function linkEntry(name,): readonly [
          string,
          string,
        ] {
          return [
            name,
            join(
              paths.exposureDir,
              name,
            ),
          ];
        },
        ),
    ),
  };
}

//endregion Build

//region CLI

/**
 Entry point: prints the JSON contract on stdout.

 @example
 ```ts
 await main();
 ```
 */
export async function main(): Promise<void> {
  try {
    /**
     Contract for the worktree this file lives in.
     */
    const output = await buildBinPathsOutput({
      paths: resolveToolchainPaths(),
      env: process.env,
    },);
    process.stdout
      .write(`${
      JSON.stringify(
        output,
        null,
        2,
      )
    }\n`,);
  } catch (error) {
    process.stderr
      .write(`bin-paths: ${
      Error.isError(error,)
        ? error.message
        : String(error,)
    }\n`,);
    process.exitCode = 1;
  }
}

//endregion CLI

if (isMainModule(import.meta.filename,))
  await main();
