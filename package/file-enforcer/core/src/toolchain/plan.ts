/**
 The typed toolchain plan.

 This module is the source `toolchain.lock.json`, `mpm.packages.toml` and
 `mpm.managers.toml` are generated from (see `generate.ts`), and the record
 `toolchain.lock.json` names in its `generatedFrom` field.

 The tool data lives in `plan.tools.data.ts` (tools) and `plan.artifacts.data.ts`
 (per-platform artifact provenance).
 */

import { ALL_TOOLS, } from './plan.tools.data.ts';
import type { ToolPlanEntry, } from './types.ts';

export {
  ALL_TOOLS,
  BIN_BOOTSTRAP,
  MPM_BOOTSTRAP,
  TOOLS,
} from './plan.tools.data.ts';
export { ARTIFACTS, } from './plan.artifacts.data.ts';
export type {
  ArtifactRow,
  ToolArtifacts,
} from './plan.artifacts.data.ts';
export { TOOL_ROUTES, } from './types.ts';
export type {
  PostinstallLink,
  ToolchainLock,
  ToolchainLockTool,
  ToolEnv,
  ToolOptions,
  ToolPlanEntry,
  ToolRequest,
  ToolRoute,
} from './types.ts';

//region Lookups

/**
 Plan entries keyed by tool id.
 */
export const TOOLS_BY_ID: ReadonlyMap<string, ToolPlanEntry> = new Map(
  ALL_TOOLS.map(
    function byId(entry,): [
      string,
      ToolPlanEntry
    ] {
      return [
        entry.id,
        entry
      ];
    },
  ),
);

/**
 Looks one plan entry up by its stable id.
 
 @param id - Stable tool id (`bun`, `cargo:fd-find`, `mpm`, ...)
 
 @returns Matching plan entry

 @throws Error when the id names no plan entry

 @example
 ```ts
 toolById('bun',).version;
 ```
 */
export function toolById(id: string,): ToolPlanEntry {
  /**
   Matching entry, or `undefined` when the id is unknown.
   */
  const entry = TOOLS_BY_ID.get(id,);
  if (entry === undefined)
    throw new Error(`toolchain plan has no tool ${JSON.stringify(id,)}`,);
  return entry;
}

/**
 Lowercased bare package names the toolchain plan can install,
 for `src/package/mise.generate-index.ts`'s
 "installable via the toolchain" filter.

 Prefixed ids (`cargo:fd-find`, `npm:yaml-language-server`,
 `github:owner/repo`) contribute their bare name; every tool also
 contributes its own id lowercased.

 @returns Lowercased names, ids, repository names, and package ids

 @example
 ```ts
 toolPlanInstallableNames().has('fd-find',); // true
 ```
 */
export function toolPlanInstallableNames(): ReadonlySet<string> {
  /**
   Lowercased installable names collected from the plan.
   */
  const names = new Set<string>();
  for (const entry of ALL_TOOLS) {
    names.add(entry.id
      .toLowerCase(),);
    /**
     Portion of the tool id after its backend prefix, or the whole id.
     */
    const bare = (entry.id
      .includes(':',)
      ? entry.id
        .slice(entry.id
          .indexOf(':',)
          + 1,)
      : entry.id);
    names.add(bare.toLowerCase(),);
    if (bare.includes('/',)) {
      /**
       Last path segment of a `github:owner/repo` style id.
       */
      const repo = bare.slice(bare.lastIndexOf('/',) + 1,);
      names.add(repo.toLowerCase(),);
    }
    if (entry.packageId !== null)
      names.add(entry.packageId
        .toLowerCase(),);
  }
  return names;
}

//endregion Lookups
