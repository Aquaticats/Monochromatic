/**
 Re-resolution of every plan entry for `toolchain:update`: dispatch to the
 family resolver, honor pins, and record one outcome per entry.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  resolveAndroid,
  resolveAws,
  resolveCargo,
  resolveConda,
  resolveGithubFamily,
  resolveGoModule,
  resolveGoRuntime,
  resolveJava,
  resolveNode,
  resolveNpm,
  resolvePypi,
  resolveRust,
  resolveRustup,
  resolveZigLatest,
  type Fetcher,
  type ToolResolution,
} from './update.resolvers.ts';

//region Outcomes

/**
 What the updater decided for one tool.
 */
export type UpdateOutcome = {
  readonly id: string;
  readonly kind: 'unchanged' | 'updated' | 'pinned' | 'failed';
  readonly oldVersion: string;
  readonly newVersion: string;
  readonly artifacts: ToolArtifacts;
  readonly note?: string;
};

/**
 Dispatches one plan entry to its family resolver.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns re-resolved version and rows
 */
async function resolveEntry(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  if (entry.id === 'node')
    return resolveNode({
      entry,
      fetcher,
    },);
  if (entry.id === 'go') {
    return resolveGoRuntime({
      entry,
      fetcher,
    },);
  }
  if (entry.id === 'zig') {
    /**
     Newest stable zig, reported against the pin only; empty when only dev builds are listed.
     */
    const latest = await resolveZigLatest({ fetcher, },);
    return {
      version: latest === ''
        ? entry.version
        : latest,
      artifacts: entry.artifacts,
    };
  }
  if (entry.id === 'java') {
    return resolveJava({
      entry,
      fetcher,
    },);
  }
  if (entry.id === 'rustup') {
    // rustup rides the static.rust-lang.org channel archive, not GitHub
    // releases (`doc/planning/mpm-global-toolchain-migration.md`).
    return resolveRustup({
      entry,
      fetcher,
    },);
  }
  if (entry.id === 'aws') {
    // aws rides awscli.amazonaws.com zips named by the aws/aws-cli tags, not
    // GitHub release assets.
    return resolveAws({
      entry,
      fetcher,
    },);
  }
  if (entry.route === 'cargo-binstall') {
    return resolveCargo({
      entry,
      fetcher,
    },);
  }
  if (entry.route === 'cargo-git') {
    // A pinned git ref has no upstream resolution to chase; the pin moves
    // only through a plan edit.
    return {
      version: entry.version,
      artifacts: {},
    };
  }
  if (entry.route === 'mpm-pnpm') {
    return resolveNpm({
      entry,
      fetcher,
    },);
  }
  if (entry.route === 'mpm-uvx') {
    return resolvePypi({
      entry,
      fetcher,
    },);
  }
  if (entry.route === 'go-install') {
    return resolveGoModule({
      entry,
      fetcher,
    },);
  }
  if (entry.route === 'rustup-toolchain') {
    return resolveRust({
      entry,
      fetcher,
    },);
  }
  if (entry.route === 'android-sdk') {
    return resolveAndroid({
      entry,
      fetcher,
    },);
  }
  if (entry.id
    .startsWith('conda:',)) {
    return resolveConda({
      entry,
      fetcher,
    },);
  }
  return resolveGithubFamily({
    entry,
    fetcher,
  },);
}

/**
 Whether two artifact maps carry identical URLs and digests.

 @param a - first map

 @param b - second map

 @returns `true` when every row matches
 */
function sameArtifacts(
  {
    a,
    b,
  }: {
    readonly a: ToolArtifacts;
    readonly b: ToolArtifacts;
  },
): boolean {
  /**
   Platform ids to compare.
   */
  const platforms = new Set([
    ...Object.keys(a,),
    ...Object.keys(b,),
  ]);
  for (const platform of platforms) {
    if ((a[platform]
      ?.url
      !== b[platform]
        ?.url)
      || (a[platform]
        ?.sha256
        !== b[platform]
          ?.sha256))
      return false;
  }
  return true;
}

/**
 Re-resolves every plan entry, honoring pins.

 @param tools - plan entries to re-resolve

 @param fetcher - injected GET

 @returns one outcome per entry, in plan order

 @example
 ```ts
 await resolveAll({ tools: ALL_TOOLS, fetcher, },); // [{ id: 'age', kind: 'unchanged', ... }]
 ```
 */
export async function resolveAll(
  {
    tools,
    fetcher,
  }: {
    readonly tools: readonly ToolPlanEntry[];
    readonly fetcher: Fetcher;
  },
): Promise<readonly UpdateOutcome[]> {
  /**
   Outcomes in plan order.
   */
  const outcomes: UpdateOutcome[] = [];
  for (const entry of tools) {
    try {
      /* oxlint-disable no-await-in-loop -- upstream registries rate-limit parallel requests and the report keeps plan order */
      /**
       Upstream resolution for this entry.
       */
      const resolution = await resolveEntry({
        entry,
        fetcher,
      },);
      /* oxlint-enable no-await-in-loop */
      /**
       Whether the entry is an explicit pin.
       */
      const pinned = 'pinned' in entry.request;
      if (pinned) {
        outcomes.push({
          id: entry.id,
          kind: 'pinned',
          oldVersion: entry.version,
          newVersion: entry.version,
          artifacts: entry.artifacts,
          note: resolution.version === entry.version
            ? 'no newer'
            : `newer available: ${resolution.version}`,
        },);
        continue;
      }
      if ((resolution.version === entry.version)
        && sameArtifacts({
          a: resolution.artifacts,
          b: entry.artifacts,
        },)) {
        outcomes.push({
          id: entry.id,
          kind: 'unchanged',
          oldVersion: entry.version,
          newVersion: entry.version,
          artifacts: entry.artifacts,
        },);
        continue;
      }
      outcomes.push({
        id: entry.id,
        kind: 'updated',
        oldVersion: entry.version,
        newVersion: resolution.version,
        artifacts: resolution.artifacts,
      },);
    }
    catch (error) {
      /**
       Failure text for the report line.
       */
      const message = caughtValueText(error,);
      if ('pinned' in entry.request) {
        // Pins never move; an unresolvable pin is reported, not fatal.
        outcomes.push({
          id: entry.id,
          kind: 'pinned',
          oldVersion: entry.version,
          newVersion: entry.version,
          artifacts: entry.artifacts,
          note: `unknown: ${message}`,
        },);
        continue;
      }
      outcomes.push({
        id: entry.id,
        kind: 'failed',
        oldVersion: entry.version,
        newVersion: entry.version,
        artifacts: entry.artifacts,
        note: message,
      },);
    }
  }
  return outcomes;
}

//endregion Outcomes
