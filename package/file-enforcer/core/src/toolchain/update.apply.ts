/**
 Applies re-resolved versions and rows to the plan entries and writes the
 plan sources and generated root artifacts for `toolchain:update`.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { writeArtifacts, } from './generate.ts';
import {
  type ToolchainPaths,
  resolveToolchainPaths,
} from './paths.ts';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  patchPlanVersions,
  renderPlanArtifactsTs,
} from './update.render.ts';
import type { UpdateOutcome, } from './update.resolve.ts';

/**
 Moves one entry to a new resolution, carrying the fields that pin the old
 version string.

 @param entry - entry to move

 @param newVersion - version the resolution moved to

 @param artifacts - artifact rows matching the new version

 @returns entry whose version-carrying fields name the new version

 @remarks
 The rustup package id is the dated toolchain name the adapter installs,
 and env values (`RUSTUP_TOOLCHAIN`, `java/<version>`) embed the same string.
 A resolution moving only `version` leaves sync installing one toolchain
 while `toolchain:status` and the task environment look for another.
 */
function moveVersionPins(
  {
    entry,
    newVersion,
    artifacts,
  }: {
    readonly entry: ToolPlanEntry;
    readonly newVersion: string;
    readonly artifacts: ToolArtifacts;
  },
): ToolPlanEntry {
  /**
   Version string the entry pinned before this resolution.
   */
  const oldVersion = entry.version;
  /**
   Env map rebuilt so values pinning the old version move with it.
   */
  const env: Record<string, string> = {};
  for (const [key, value,] of Object.entries(entry.env,))
    env[key] = value.replaceAll(
      oldVersion,
      newVersion,
    );
  return {
    ...entry,
    version: newVersion,
    ...(entry.packageId === oldVersion ? { packageId: newVersion, } : {}),
    env,
    artifacts,
  };
}

/**
 Builds the plan entries carrying the re-resolved versions and rows.

 @param tools - original entries

 @param outcomes - update outcomes, one per entry

 @returns entries with every non-pin resolution applied

 @throws Error when an entry has no recorded outcome

 @example
 ```ts
 applyResolutions({ tools, outcomes, },); // entries with the updated versions
 ```
 */
export function applyResolutions(
  {
    tools,
    outcomes,
  }: {
    readonly tools: readonly ToolPlanEntry[];
    readonly outcomes: readonly UpdateOutcome[];
  },
): readonly ToolPlanEntry[] {
  /**
   Outcomes keyed by tool id.
   */
  const byId = new Map<string, UpdateOutcome>();
  for (const outcome of outcomes)
    byId.set(
      outcome.id,
      outcome,
    );
  return tools.map(
    function apply(entry,) {
      /**
       Outcome recorded for this entry.
       */
      const outcome = byId.get(entry.id,);
      if (outcome === undefined)
        throw new Error(`no update outcome recorded for ${entry.id}`,);
      return outcome.kind === 'updated'
        ? moveVersionPins({
          entry,
          newVersion: outcome.newVersion,
          artifacts: outcome.artifacts,
        },)
        : entry;
    },
  );
}

/**
 Writes the plan sources and the generated root artifacts.

 @param tools - updated entries

 @param outcomes - update outcomes that name the moved versions

 @param planDir - directory of the plan sources, default this package's toolchain directory

 @param paths - scope paths (the machine-global scope writes its own stack)

 @returns file names that changed on disk

 @example
 ```ts
 await writeResolutions({ tools, outcomes, },); // ['plan.artifacts.data.ts', 'toolchain.lock.json']
 ```
 */
export async function writeResolutions(
  {
    tools,
    outcomes,
    planDir = new URL(
      '.',
      import.meta.url,
    )
      .pathname,
    paths = resolveToolchainPaths(),
  }: {
    readonly tools: readonly ToolPlanEntry[];
    readonly outcomes: readonly UpdateOutcome[];
    readonly planDir?: string;
    readonly paths?: ToolchainPaths;
  },
): Promise<readonly string[]> {
  /**
   New artifact map keyed by tool id (pins keep their recorded rows).
   */
  const artifacts: Record<string, ToolArtifacts> = {};
  for (const tool of tools)
    artifacts[tool.id] = tool.artifacts;
  /**
   Files written this run.
   */
  const written: string[] = [];
  /**
   Rendered artifact data.
   */
  const artifactsText = renderPlanArtifactsTs({ artifacts, },);
  /**
   File that holds the artifact data.
   */
  const artifactsPath = join(
    planDir,
    'plan.artifacts.data.ts',
  );
  if (await readFile(
    artifactsPath,
    'utf8',
  ) !== artifactsText) {
    await writeFile(
      artifactsPath,
      artifactsText,
      'utf8',
    );
    written.push('plan.artifacts.data.ts',);
  }
  /**
   Version patches keyed by tool id.
   */
  const versions = new Map<string, string>();
  for (const outcome of outcomes) {
    if (outcome.kind === 'updated')
      versions.set(
        outcome.id,
        outcome.newVersion,
      );
  }
  if (versions.size > 0) {
    /**
     File that holds the plan entries.
     */
    const toolsPath = join(
      planDir,
      'plan.tools.data.ts',
    );
    /**
     Plan text before the moved versions are rewritten.
     */
    const original = await readFile(
      toolsPath,
      'utf8',
    );
    /**
     Plan text with the moved versions rewritten.
     */
    const patched = patchPlanVersions({
      source: original,
      versions,
    },);
    if (patched !== original) {
      await writeFile(
        toolsPath,
        patched,
        'utf8',
      );
      written.push('plan.tools.data.ts',);
    }
  }
  /**
   Generated root artifacts (the lock and the mpm manifests).
   */
  for (const name of writeArtifacts({
    tools,
    paths,
  },)
    .written)
    written.push(name,);
  return written;
}
