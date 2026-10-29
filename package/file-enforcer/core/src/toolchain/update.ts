/**
 `toolchain:update`: re-resolves the plan's floating requests and channels to
 their current upstream versions and rewrites the resolved data
 (the `pnpm update` analogue `doc/planning/mpm-toolchain-migration.md` names).

 Explicit pins never move: the command reports them as pinned and leaves any
 change to a plan edit. Fixed channel ranges (`java` stays `temurin-21`) move
 only their resolution stamp.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/update.ts
 node package/file-enforcer/core/src/toolchain/update.ts --check
 ```
 */

import {
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { writeArtifacts, } from './generate.ts';
import {
  ALL_TOOLS,
} from './plan.ts';
import { resolveGithubToken, } from './providers.ts';
import {
  globalToolchainPaths,
  isMainModule,
  resolveToolchainPaths,
  type ToolchainPaths,
} from './paths.ts';
import type { ToolArtifacts, } from './plan.artifacts.ts';
import { loadPlanFromDir, } from './plan.load.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  patchPlanVersions,
  renderPlanArtifactsTs,
} from './update.render.ts';
import {
  resolveAndroid,
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
  readonly note?: string | undefined;
};

/**
 Dispatches one plan entry to its family resolver.
 
 @param options - Plan entry and fetcher
 
 @returns Re-resolved version and rows
 
 @throws Error when the entry's family has no resolver
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
     Newest stable zig, reported against the pin only.
     */
    const latest = await resolveZigLatest({ fetcher, },);
    return {
      version: latest ?? entry.version,
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
 Re-resolves every plan entry, honoring pins.
 
 @param options - Plan entries and fetcher
 
 @returns One outcome per entry, in plan order
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
      /**
       Upstream resolution for this entry.
       */
      const resolution = await resolveEntry({
        entry,
        fetcher,
      },);
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
        && sameArtifacts(
          resolution.artifacts,
          entry.artifacts,
        )) {
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
      const message = Error.isError(error,)
        ? error.message
        : String(error,);
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
        note: Error.isError(error,)
          ? error.message
          : String(error,),
      },);
    }
  }
  return outcomes;
}

/**
 Whether two artifact maps carry identical URLs and digests.
 
 @param options - Maps to compare
 
 @returns `true` when every row matches
 */
function sameArtifacts(
  a: ToolArtifacts,
  b: ToolArtifacts,
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

//endregion Outcomes

//region Report

/**
 Renders the per-tool report lines and the summary.
 
 @param options - Outcomes to report
 
 @returns Report text
 */
export function formatReport(
  {
    outcomes,
  }: {
    readonly outcomes: readonly UpdateOutcome[];
  },
): string {
  /**
   Per-tool lines.
   */
  const lines: string[] = [];
  for (const outcome of outcomes) {
    if (outcome.kind === 'updated') {
      lines.push(`${
        outcome.id
      } ${
        outcome.oldVersion
      } -> ${
        outcome.newVersion
      }`,);
      continue;
    }
    if (outcome.kind === 'pinned') {
      lines.push(`${
        outcome.id
      } pinned (${
        outcome.note ?? 'no newer'
      })`,);
      continue;
    }
    if (outcome.kind === 'failed') {
      lines.push(`${
        outcome.id
      } failed: ${
        outcome.note ?? 'unknown error'
      }`,);
      continue;
    }
    lines.push(`${outcome.id} unchanged`,);
  }
  /**
   Counts per kind.
   */
  function count(kind: UpdateOutcome['kind'],): number { return outcomes
    .filter(
    (outcome,) => outcome.kind === kind,
    )
    .length; }
  lines.push(
    `toolchain update: ${
      String(count('updated',),)
    } updated, ${
      String(count('unchanged',),)
    } unchanged, ${
      String(count('pinned',),)
    } pinned, ${
      String(count('failed',),)
    } failed`,
  );
  return lines.join('\n',);
}

//endregion Report

//region Apply

/**
 Moves one entry to a new resolution, carrying the fields that pin the old
 version string.
 
 @param options - Entry to move, new resolution and matching artifact rows
 
 @returns Entry whose version-carrying fields name the new version
 
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
  for (const [key, value] of Object.entries(entry.env,))
    env[key] = value.replaceAll(
      oldVersion,
      newVersion,
    );
  return {
    ...entry,
    version: newVersion,
    packageId: entry.packageId === oldVersion
      ? newVersion
      : entry.packageId,
    env,
    artifacts,
  };
}

/**
 Builds the plan entries carrying the re-resolved versions and rows.
 
 @param options - Original entries and the update outcomes
 
 @returns Entries with every non-pin resolution applied
 
 @throws Error when an outcome names an unknown entry
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
 
 @param options - Updated entries, outcomes, and optionally the plan source
 directory and scope paths (the machine-global scope writes its own stack)
 
 @returns File names that changed on disk
 */
export function writeResolutions(
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
    readonly planDir?: string | undefined;
    readonly paths?: ToolchainPaths | undefined;
  },
): readonly string[] {
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
   Rendered artifact data and its file.
   */
  const artifactsText = renderPlanArtifactsTs({ artifacts, },);
  const artifactsPath = join(
    planDir,
    'plan.artifacts.ts',
  );
  if (readFileSync(
    artifactsPath,
    'utf8',
  ) !== artifactsText) {
    writeFileSync(
      artifactsPath,
      artifactsText,
      'utf8',
    );
    written.push('plan.artifacts.ts',);
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
     Plan text with the moved versions rewritten.
     */
    const toolsPath = join(
      planDir,
      'plan.tools.ts',
    );
    const patched = patchPlanVersions({
      source: readFileSync(
        toolsPath,
        'utf8',
      ),
      versions,
    },);
    if (patched !== readFileSync(
      toolsPath,
      'utf8',
    )) {
      writeFileSync(
        toolsPath,
        patched,
        'utf8',
      );
      written.push('plan.tools.ts',);
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

//endregion Apply

//region CLI

/**
 Runs the update flow: resolve, report, and write unless `--check`.
 
 @throws Error when a resolution fails and `--check` needs a clean verdict
 */
export async function main(): Promise<void> {
  /**
   Dry-run flag: report only, never write.
   */
  const check = process.argv
    .includes('--check',);
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
   Plan entries of the scope being updated.
   */
  const tools = global
    ? await loadPlanFromDir({ dir: paths.worktreeRoot, },)
    : ALL_TOOLS;
  /**
   Token-bound fetcher for the GitHub family.
   */
  const token = await resolveGithubToken({ env: process.env, },);
  /**
   Network fetcher with GitHub authentication bound in.
   */
  async function fetcher(
    url: Parameters<Fetcher>[0],
    init?: Parameters<Fetcher>[1],
  ): ReturnType<Fetcher> {
    /**
     Request headers with GitHub authentication when the URL needs it.
     */
    const headers: Record<string, string> = {
      ...init?.headers,
      'user-agent': 'monochromatic-toolchain-update',
    };
    if ((token !== null) && url.startsWith('https://api.github.com/'))
      headers.authorization = `Bearer ${token}`;
    /**
     Response from the upstream source.
     */
    const response = await fetch(
      url,
      { headers, },
    );
    /**
     Body bytes, decoded once (a response body reads a single time).
     */
    const bytes = new Uint8Array(await response.arrayBuffer(),);
    return {
      ok: response.ok,
      status: response.status,
      text: new TextDecoder()
        .decode(bytes),
      bytes,
    };
  }
  /**
   Outcomes for every plan entry.
   */
  const outcomes = await resolveAll({
    tools,
    fetcher,
  },);
  process.stdout
    .write(`${
    formatReport({ outcomes, })
  }\n`,);
  /**
   Counts the exit code depends on.
   */
  const updates = outcomes.filter(
    (outcome,) => outcome.kind === 'updated',
  );
  const failures = outcomes.filter(
    (outcome,) => outcome.kind === 'failed',
  );
  if (failures.length > 0) {
    process.exitCode = 1;
    return;
  }
  if (check) {
    if (updates.length > 0)
      process.exitCode = 1;
    return;
  }
  if (updates.length === 0)
    return;
  /**
   Files the resolution rewrote.
   */
  const written = writeResolutions({
    tools: applyResolutions({
      tools,
      outcomes,
    },),
    outcomes,
    planDir: global
      ? paths.worktreeRoot
      : undefined,
    paths: global
      ? paths
      : undefined,
  },);
  process.stdout
    .write(`toolchain update: wrote ${
    written.join(', ',)
  }\n`,);
}

if (isMainModule(import.meta.filename,))
  await main();

//endregion CLI
