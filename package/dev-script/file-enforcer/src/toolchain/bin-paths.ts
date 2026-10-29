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
 node package/dev-script/file-enforcer/src/toolchain/bin-paths.ts
 ```
 */

import { existsSync, } from 'node:fs';
import { join, } from 'node:path';
import { ALL_TOOLS, } from './plan.ts';
import {
  EXPOSURE_DIR_RELATIVE,
  managerNativeBinDirs,
  resolveToolchainPaths,
  type ToolchainPaths,
  isMainModule,
} from './paths.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Output shape

/**
 The `bin-paths.ts` JSON contract.
 */
export type BinPathsOutput = {
  /** Worktree-relative exposure directory, always `.cache/toolchain-bin`. */
  readonly exposureDir: '.cache/toolchain-bin';
  /** Absolute PATH directories that exist right now. */
  readonly pathDirs: readonly string[];
  /** Declared binary name to its absolute exposure symlink path. */
  readonly bins: Readonly<Record<string, string>>;
};

//endregion Output shape

//region Build

/**
 Builds the exposure contract for one worktree.

 @param options - Resolved toolchain paths and environment overrides
 
 @returns The exact JSON contract
 
 @throws {Error} when two plan entries declare the same binary name
 */
export function buildBinPathsOutput(
  {
    paths,
    env,
    entries = ALL_TOOLS,
  }: {
    readonly paths: ToolchainPaths;
    readonly env?: NodeJS.ProcessEnv | undefined;
    readonly entries?: readonly ToolPlanEntry[];
  },
): BinPathsOutput {
  /**
   Binary name to absolute exposure symlink path, built in plan order.
   */
  const bins: Record<string, string> = {};
  /**
   Declared binary names seen so far, so duplicates are caught whether or not
   they are provisioned.
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
      /**
       Absolute path of this binary's exposure symlink.
       */
      const linkPath = join(
        paths.exposureDir,
        name,
      );
      if (existsSync(linkPath,))
        bins[name] = linkPath;
    }
  }

  /**
   Candidate PATH directories, absolute, keeping only the ones that exist.
   */
  const pathDirs = managerNativeBinDirs({
    paths,
    env,
  },)
    .filter(
      function exists(dir,): boolean {
        return existsSync(dir,);
      },
    );

  return {
    exposureDir: EXPOSURE_DIR_RELATIVE,
    pathDirs,
    bins,
  };
}

//endregion Build

//region CLI

/**
 Entry point: prints the JSON contract on stdout.

 @returns Promise resolved after the JSON is written
 */
export async function main(): Promise<void> {
  try {
    /**
     Contract for the worktree this file lives in.
     */
    const output = buildBinPathsOutput({
      paths: resolveToolchainPaths(),
      env: process.env,
    },);
    process.stdout.write(`${
      JSON.stringify(
        output,
        null,
        2,
      )
    }\n`,);
  } catch (error) {
    process.stderr.write(`bin-paths: ${
      error instanceof Error
        ? error.message
        : String(error,)
    }\n`,);
    process.exitCode = 1;
  }
}

//endregion CLI

if (isMainModule(import.meta.filename,))
  await main();
