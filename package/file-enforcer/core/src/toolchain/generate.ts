/**
 `generate.ts`: writes the three committed root artifacts from the typed plan:

 - `toolchain.lock.json`
 - `mpm.packages.toml`
 - `mpm.managers.toml`

 Deterministic: running it twice produces byte-identical files.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/generate.ts
 ```
 */

import {
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { dirname, } from 'node:path';
import { isErrnoException, } from './fs-probe.ts';
import {
  renderToolchainLockJson,
  toolchainLockFromTools,
} from './lock.ts';
import {
  renderMpmManagersToml,
  renderMpmPackagesToml,
} from './mpm-manifest.ts';
import {
  globalToolchainPaths,
  resolveToolchainPaths,
  isMainModule,
} from './paths.ts';
import { ALL_TOOLS, } from './plan.ts';
import { loadPlanFromDir, } from './plan.load.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Render

/**
 Renders every generated artifact from plan entries.
 
 @param tools - Plan entries to render from

 @returns File name to file content

 @example
 ```ts
 renderArtifacts({ tools: ALL_TOOLS, },)['toolchain.lock.json']; // '{\n  "schemaVersion": 1,...'
 ```
 */
export function renderArtifacts(
  { tools, }: { readonly tools: readonly ToolPlanEntry[]; },
): Readonly<Record<string, string>> {
  return {
    'toolchain.lock.json': renderToolchainLockJson(toolchainLockFromTools({
      tools,
    },),),
    'mpm.packages.toml': renderMpmPackagesToml({ tools, },),
    'mpm.managers.toml': renderMpmManagersToml({ tools, },),
  };
}

//endregion Render

//region Write

/**
 Outcome of one generation run.
 */
export type GenerateReport = {
  /**
   Files whose content changed and were written.
   */
  readonly written: readonly string[];
  /**
   Files already byte-identical to the rendered content.
   */
  readonly unchanged: readonly string[];
};

/**
 Whether a target file already holds exactly the rendered content.

 A file that cannot be read (absent, or unreachable) is not current, so the caller rewrites it.

 @param target - Absolute path of the artifact file

 @param content - Rendered content the file should hold

 @returns `true` when the file content equals `content`
 */
function targetIsCurrent(
  {
    target,
    content,
  }: {
    readonly target: string;
    readonly content: string;
  },
): boolean {
  try {
    return readFileSync(
      target,
      'utf8',
    ) === content;
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    return false;
  }
}

/**
 Writes the generated root artifacts when their content differs.

 @param tools - Plan entries to render the artifacts from

 @param paths - Resolved toolchain paths whose artifact files receive the output

 @returns Which files were written and which were already current

 @example
 ```ts
 writeArtifacts({ tools: ALL_TOOLS, paths: resolveToolchainPaths(), },).written; // ['toolchain.lock.json']
 ```
 */
export function writeArtifacts(
  {
    tools,
    paths,
  }: {
    readonly tools: readonly ToolPlanEntry[];
    readonly paths: ReturnType<typeof resolveToolchainPaths>;
  },
): GenerateReport {
  /**
   Rendered artifacts keyed by file name.
   */
  const artifacts = renderArtifacts({ tools, },);
  /**
   Absolute output paths keyed by file name.
   */
  const targets: Record<string, string> = {
    'toolchain.lock.json': paths.lockFile,
    'mpm.packages.toml': paths.mpmPackagesFile,
    'mpm.managers.toml': paths.mpmManagersFile,
  };
  /**
   Files written this run.
   */
  const written: string[] = [];
  /**
   Files already current.
   */
  const unchanged: string[] = [];
  for (const [name, content,] of Object.entries(artifacts,)) {
    /**
     Absolute output path for this artifact.
     */
    const target = targets[name];
    if (target === undefined)
      continue;
    mkdirSync(
      dirname(target,),
      { recursive: true },
    );
    if (targetIsCurrent({
      target,
      content,
    },)) {
      unchanged.push(name,);
      continue;
    }
    writeFileSync(
      target,
      content,
      'utf8',
    );
    written.push(name,);
  }
  return {
    written,
    unchanged,
  };
}

//endregion Write

//region CLI

/**
 Entry point: regenerates the root artifacts.
 
 `--global` regenerates the machine-global scope's records
 (`doc/planning/mpm-global-toolchain-migration.md`).

 @example
 ```ts
 await main();
 ```
 */
export async function main(): Promise<void> {
  /**
   Whether this run targets the machine-global scope.
   */
  const global = process.argv
    .includes('--global',);
  /**
   Resolved scope paths.
   */
  const paths = global
    ? globalToolchainPaths({ env: process.env, },)
    : resolveToolchainPaths();
  /**
   Generation report for this run.
   */
  const report = writeArtifacts({
    tools: global
      ? await loadPlanFromDir({ dir: paths.worktreeRoot, },)
      : ALL_TOOLS,
    paths,
  },);
  for (const name of report.written)
    process.stdout
      .write(`wrote ${name}\n`,);
  for (const name of report.unchanged)
    process.stdout
      .write(`unchanged ${name}\n`,);
}

//endregion CLI

if (isMainModule(import.meta.filename,))
  await main();
