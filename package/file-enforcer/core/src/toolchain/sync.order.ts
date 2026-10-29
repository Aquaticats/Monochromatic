/**
 Sync ordering and host filtering: which tools stage first, and which plan
 entries apply to this machine.

 Split from `sync.ts`, which keeps the install loop and the CLI.
 */

import { ALL_TOOLS, } from './plan.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Staging order

/**
 Tool ids staged before the plan-order remainder, in staging order.
 `rustup` precedes `rust` and `go` precedes any `go-install` tool, which the
 global-scope plan (`doc/planning/mpm-global-toolchain-migration.md`) needs.
 */
const STAGING_IDS: readonly string[] = [
  'bin',
  'uv',
  'mpm',
  'rustup',
  'rust',
  'cargo-binstall',
  'go',
  'node',
  'pnpm',
] as const;

/**
 Orders plan entries for installation: the staging prerequisites first, then
 every remaining entry in plan order.

 @param entries - Plan entries to order

 @returns Ordered entries

 @example
 ```ts
 syncOrder({ entries: ALL_TOOLS, },).map((entry,) => entry.id,).slice(0, 3,); // ['bin', 'uv', 'mpm']
 ```
 */
export function syncOrder(
  { entries, }: { readonly entries: readonly ToolPlanEntry[]; },
): readonly ToolPlanEntry[] {
  /**
   First plan entry per tool id.
   */
  const firstById = new Map<string, ToolPlanEntry>();
  for (const entry of entries) {
    if (!firstById.has(entry.id,))
      firstById.set(
        entry.id,
        entry,
      );
  }
  /**
   Staged entries, in staging order.
   */
  const staged: ToolPlanEntry[] = [];
  for (const id of STAGING_IDS) {
    /**
     Matching plan entry for this staging id.
     */
    const entry = firstById.get(id,);
    if (entry !== undefined)
      staged.push(entry,);
  }
  return [
    ...staged,
    ...entries.filter(
      function notStaged(entry,): boolean {
        return !STAGING_IDS.includes(entry.id,);
      },
    ),
  ];
}

//endregion Staging order

//region Platform filter

/**
 Whether one plan entry applies to the running host.
 
 @param entry - Plan entry to test
 
 @returns `true` when the entry's OS filter admits this platform

 @example
 ```ts
 appliesToHost(toolById('node',),); // true
 ```
 */
export function appliesToHost(entry: ToolPlanEntry,): boolean {
  if (entry.os === undefined)
    return true;
  /**
   Platform family of the running host.
   */
  const family = process.platform === 'linux'
    ? 'linux'
    : process.platform === 'darwin'
      ? 'macos'
      : process.platform === 'win32'
        ? 'windows'
        : process.platform;
  /**
   The entry's OS filter, widened so any host platform name can be tested against it.
   */
  const admitted: readonly string[] = entry.os;
  return admitted.includes(family,);
}

//endregion Platform filter
