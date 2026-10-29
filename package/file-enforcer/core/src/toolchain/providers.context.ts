/**
 Provider contract: the context every provider receives, plus the plan and
 artifact lookups all providers share.
 */

import { hostPlatformId, } from './layout.ts';
import { ALL_TOOLS, } from './plan.ts';
import type {
  PathEnv,
  ToolchainPaths,
} from './paths.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Context

/**
 Shared context every provider receives.
 */
export type ProviderContext = {
  readonly entry: ToolPlanEntry;
  readonly paths: ToolchainPaths;
  readonly env: PathEnv;
  /**
   Extra bin search directories a staged tool contributes.
   */
  readonly extraDirs: readonly string[];
  /**
   Plan of the scope being synced (sibling lookups for `pnpm`, `uv`, `go`,
   `bin` and `mpm`); absent means the repository plan.
   */
  readonly tools?: readonly ToolPlanEntry[] | undefined;
};

/**
 Looks one sibling tool up in the scope's own plan, so a sync never resolves
 another scope's install directory for a shared tool id.

 @param tools - Scope plan; absent means the repository plan

 @param id - Tool id to look up

 @returns Matching plan entry

 @throws Error when the plan declares no such tool

 @example
 ```ts
 const bin = planTool({ id: 'bin', },);
 ```
 */
export function planTool(
  {
    tools,
    id,
  }: {
    readonly tools?: readonly ToolPlanEntry[] | undefined;
    readonly id: string;
  },
): ToolPlanEntry {
  /**
   Effective plan for this lookup.
   */
  const plan = tools ?? ALL_TOOLS;
  /**
   Matching entry, or `undefined` when the id is unknown here.
   */
  const entry = plan.find(
    function byId(candidate,): boolean {
      return candidate.id === id;
    },
  );
  if (entry === undefined)
    throw new Error(`toolchain plan has no tool ${JSON.stringify(id,)}`,);
  return entry;
}

/**
 Absolute artifact URL and SHA-256 for the current host platform.

 @param entry - Plan entry to resolve

 @returns Artifact row for the host platform

 @throws Error when the plan records no artifact for the host

 @example
 ```ts
 const { url, sha256, } = artifactForHost(planTool({ id: 'node', },),);
 ```
 */
export function artifactForHost(entry: ToolPlanEntry,): {
  readonly url: string;
  readonly sha256: string;
} {
  /**
   Platform id of the running host.
   */
  const platform = hostPlatformId();
  /**
   Artifact row recorded for the host platform.
   */
  const row = entry.artifacts[platform];
  if (row === undefined) {
    throw new Error(
      `toolchain plan has no artifact for ${entry.id} on ${platform}`,
    );
  }
  return row;
}

//endregion Context
