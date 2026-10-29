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
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
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

 @example
 ```ts
 await targetIsCurrent({ target: '/repo/toolchain.lock.json', content, },);
 ```
 */
async function targetIsCurrent(
  {
    target,
    content,
  }: {
    readonly target: string;
    readonly content: string;
  },
): Promise<boolean> {
  try {
    return (await readFile(
      target,
      'utf8',
    )) === content;
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    return false;
  }
}

/**
 One artifact to write: its file name, output path, and rendered content.
 */
type ArtifactJob = {
  readonly name: string;
  readonly target: string;
  readonly content: string;
};

/**
 One artifact's write outcome.
 */
type ArtifactOutcome = {
  readonly name: string;
  readonly changed: boolean;
};

/**
 File name of a write outcome.

 @param outcome - write outcome

 @returns its file name

 @example
 ```ts
 nameOfOutcome({ name: 'mpm.packages.toml', changed: true, },); // 'mpm.packages.toml'
 ```
 */
function nameOfOutcome(outcome: ArtifactOutcome,): string {
  return outcome.name;
}

/**
 Writes one artifact when its content differs.

 @param target - Absolute output path

 @param content - Rendered content

 @returns `true` when the file was written, `false` when it was already current

 @example
 ```ts
 await writeIfChanged({ target: '/repo/mpm.packages.toml', content, },);
 ```
 */
async function writeIfChanged(
  {
    target,
    content,
  }: {
    readonly target: string;
    readonly content: string;
  },
): Promise<boolean> {
  await mkdir(
    dirname(target,),
    { recursive: true, },
  );
  if (await targetIsCurrent({
    target,
    content,
  },))
    return false;
  await writeFile(
    target,
    content,
    'utf8',
  );
  return true;
}

/**
 Writes the generated root artifacts when their content differs.

 @param tools - Plan entries to render the artifacts from

 @param paths - Resolved toolchain paths whose artifact files receive the output

 @returns Which files were written and which were already current

 @example
 ```ts
 (await writeArtifacts({ tools: ALL_TOOLS, paths: resolveToolchainPaths(), },)).written; // ['toolchain.lock.json']
 ```
 */
export async function writeArtifacts(
  {
    tools,
    paths,
  }: {
    readonly tools: readonly ToolPlanEntry[];
    readonly paths: ReturnType<typeof resolveToolchainPaths>;
  },
): Promise<GenerateReport> {
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
   One write job per artifact that has an output path.
   */
  const jobs = Object.entries(artifacts,)
    .flatMap(function toJob([name, content,]: readonly [
      string,
      string,
    ],): readonly ArtifactJob[] {
      /**
       Absolute output path for this artifact.
       */
      const target = targets[name];
      return target === undefined
        ? []
        : [
          {
            name,
            target,
            content,
          },
        ];
    },);
  /**
   Each artifact with whether it was written; the files are independent, so they are written concurrently.
   */
  const outcomes = await Promise.all(jobs.map(async function runJob(job: ArtifactJob,): Promise<ArtifactOutcome> {
    return {
      name: job.name,
      changed: await writeIfChanged(job,),
    };
  },),);
  return {
    written: outcomes.filter(function wasWritten(outcome: ArtifactOutcome,): boolean {
      return outcome.changed;
    },)
      .map(nameOfOutcome,),
    unchanged: outcomes.filter(function wasCurrent(outcome: ArtifactOutcome,): boolean {
      return !outcome.changed;
    },)
      .map(nameOfOutcome,),
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
  const report = await writeArtifacts({
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

if (await isMainModule(import.meta.filename,))
  await main();
