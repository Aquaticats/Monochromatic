/**
 Deterministic generator for `toolchain.lock.json`
 (doc/planning/mpm-toolchain-migration.md, section `toolchain.lock.json`).
 */

import type {
  ToolPlanEntry,
  ToolchainLock,
  ToolchainLockTool,
} from './types.ts';

//region Generation

/**
 Projects one plan entry onto its `toolchain.lock.json` record,
 inserting keys in the fixed order the schema documents.
 
 @param entry - Plan entry to project

 @returns Lock record with stable key order

 @example
 ```ts
 lockToolFromPlanEntry(toolById('bun',));
 ```
 */
export function lockToolFromPlanEntry(entry: ToolPlanEntry,): ToolchainLockTool {
  return {
    request: entry.request,
    version: entry.version,
    route: entry.route,
    manager: entry.manager,
    packageId: entry.packageId,
    bins: entry.bins,
    os: entry.os,
    env: entry.env,
    options: entry.options,
    artifacts: entry.artifacts,
  };
}

/**
 Builds the `toolchain.lock.json` document from plan entries,
 keeping plan order (the pinned `mpm` bootstrap first,
 then `mise.no-env.toml` `[tools]` order).
 
 @param tools - Plan entries in lock order

 @returns Lock document

 @example
 ```ts
 toolchainLockFromTools({ tools: ALL_TOOLS, });
 ```
 */
export function toolchainLockFromTools(
  { tools, }: { readonly tools: readonly ToolPlanEntry[]; },
): ToolchainLock {
  /**
   Tools map built in plan order, so rendering is deterministic.
   */
  const lockTools: Record<string, ToolchainLockTool> = {};
  for (const entry of tools)
    lockTools[entry.id] = lockToolFromPlanEntry(entry,);
  return {
    schemaVersion: 1,
    generatedFrom: 'package/file-enforcer/core/src/toolchain/plan.ts',
    tools: lockTools,
  };
}

//endregion Generation

//region Rendering

/**
 Renders the lock document as pretty-printed JSON with a trailing newline.
 
 Key order is the insertion order {@link toolchainLockFromTools} and
 {@link lockToolFromPlanEntry} fix, so the output is byte-stable.
 
 @param lock - Lock document to render

 @returns JSON text

 @example
 ```ts
 renderToolchainLockJson(toolchainLockFromTools({ tools: ALL_TOOLS, },));
 ```
 */
export function renderToolchainLockJson(lock: ToolchainLock,): string {
  return `${
    JSON.stringify(
      lock,
      null,
      2,
    )
  }\n`;
}

//endregion Rendering
