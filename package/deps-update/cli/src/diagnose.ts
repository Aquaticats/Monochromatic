/**
 Finds which versions tripped pnpm's strict `minimumReleaseAge` gate.

 pnpm's `ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE` names no package.
 Resolving the same workspace in loose mode makes pnpm append every
 immature pick to `minimumReleaseAgeExclude`, so the diagnosis runs that
 resolution in a scratch copy and reads the appended versions back.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  addedVersions,
  type NO_MINIMUM_RELEASE_AGE,
  readExcludeList,
  readMinimumReleaseAge,
} from './exclude-list.ts';
import {
  PnpmCommandError,
  type RunPnpm,
} from './pnpm.ts';
import {
  fetchPublishTime,
  type FetchLike,
} from './registry.ts';
import type { ImmaturePick, } from './report.ts';
import { createScratchWorkspace, } from './scratch.ts';
import type { PackageSpec, } from './spec.ts';

/**
 Module logger for the diagnosis.
 */
const l = tagged({ tag: 'deps-update/diagnose', },);

//region Types

/**
 Diagnosis outcome: the blocked picks plus the configured age.
 */
export type Diagnosis = {
  /**
   Versions the strict gate would block, sorted by `name@version`.
   */
  readonly picks: readonly ImmaturePick[];
  /**
   Configured `minimumReleaseAge` in minutes, or its absence sentinel.
   */
  readonly minutes: number | typeof NO_MINIMUM_RELEASE_AGE;
  /**
   pnpm diagnostic from a failure the resolution hit after the age check;
   the real update will stop there once the picks are handled.
   */
  readonly followUp?: string;
};

//endregion Types

//region Errors

/**
 Characters of unexpected pnpm output quoted in {@link ProjectListError}.
 */
const PROJECT_LIST_EXCERPT = 200;

/**
 Thrown when pnpm's project listing is not the expected JSON array.
 */
export class ProjectListError extends Error {
  /**
   Builds the error with the unexpected output excerpt.

   @param output - pnpm's stdout, truncated for the message

   @example
   ```ts
   throw new ProjectListError('{}');
   ```
   */
  constructor(output: string,) {
    super(`pnpm list --recursive --depth -1 --json returned unexpected output: ${output.slice(
      0,
      PROJECT_LIST_EXCERPT,
    )}`,);
    this.name = 'ProjectListError';
  }
}

//endregion Errors

//region Helpers

/**
 Loose-mode resolution arguments: lockfile only (no downloads or linking),
 strict gate off so pnpm records immature picks instead of refusing.
 */
export const LOOSE_RESOLUTION_ARGS = [
  'update',
  '--recursive',
  '--lockfile-only',
  '--config.minimum-release-age-strict=false',
] as const;

/**
 Sentinel for a loose resolution that exited 0.
 */
const LOOSE_RESOLUTION_SUCCEEDED: unique symbol = Symbol('deps-update/loose-resolution-succeeded',);

/**
 Runs the loose resolution, returning a pnpm failure instead of throwing it,
 because pnpm records immature picks before later checks can fail the run.

 @param dir - scratch workspace

 @param runPnpm - pnpm runner

 @returns success sentinel, or pnpm's failure

 @throws Error when running pnpm fails for a reason other than pnpm's exit

 @example
 ```ts
 const failure = await runLooseResolution({ dir: '/scratch', runPnpm });
 ```
 */
async function runLooseResolution({
  dir,
  runPnpm,
}: {
  readonly dir: string;
  readonly runPnpm: RunPnpm;
},): Promise<typeof LOOSE_RESOLUTION_SUCCEEDED | PnpmCommandError> {
  try {
    await runPnpm({
      args: LOOSE_RESOLUTION_ARGS,
      cwd: dir,
    },);
    return LOOSE_RESOLUTION_SUCCEEDED;
  }
  catch (error) {
    if (!(error instanceof PnpmCommandError))
      throw error;
    l.debug(`loose resolution failed; reading recorded picks anyway`,);
    return error;
  }
}

/**
 Finds pnpm's first `ERR_PNPM_*` diagnostic line.

 @param output - pnpm output

 @returns trimmed diagnostic line, or pnpm's last nonblank line when it printed no code

 @example
 ```ts
 firstDiagnosticLine('x\n[ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies\n');
 // '[ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies'
 ```
 */
export function firstDiagnosticLine(output: string,): string {
  /**
   Nonblank output lines.
   */
  const lines = output
    .split('\n',)
    .map(function trimLine(line,): string {
      return line.trim();
    },)
    .filter(function isNonblank(line,): boolean {
      return line !== '';
    },);
  return lines.find(function isDiagnostic(line,): boolean {
    return line.includes('ERR_PNPM_',);
  },)
    ?? lines.at(-1,)
    ?? 'pnpm printed no output';
}

/**
 Lists every workspace project directory, root included.

 @param root - workspace root

 @param runPnpm - pnpm runner

 @returns absolute project directories

 @throws ProjectListError when pnpm's JSON lacks `path` strings

 @example
 ```ts
 await listProjectDirs({ root: '/repo', runPnpm });
 ```
 */
async function listProjectDirs({
  root,
  runPnpm,
}: {
  readonly root: string;
  readonly runPnpm: RunPnpm;
},): Promise<readonly string[]> {
  /**
   JSON array of workspace projects.
   */
  const output = await runPnpm({
    args: [
      'list',
      '--recursive',
      '--depth',
      '-1',
      '--json',
    ],
    cwd: root,
  },);
  /**
   Parsed listing, untyped until checked.
   */
  const parsed: unknown = JSON.parse(output,);
  if (!Array.isArray(parsed,))
    throw new ProjectListError(output,);
  return parsed.map(function toPath(project: unknown,): string {
    if (((typeof project) !== 'object') || (project === null)
      || (!('path' in project))
      || ((typeof project.path) !== 'string'))
      throw new ProjectListError(output,);
    return project.path;
  },);
}

/**
 Resolves the registry serving a package, honoring `@scope:registry`.

 @param root - workspace root, so workspace `.npmrc` settings apply

 @param name - npm package name

 @param runPnpm - pnpm runner

 @returns registry base URL

 @example
 ```ts
 await registryFor({ root: '/repo', name: '@a/b', runPnpm });
 ```
 */
async function registryFor({
  root,
  name,
  runPnpm,
}: {
  readonly root: string;
  readonly name: string;
  readonly runPnpm: RunPnpm;
},): Promise<string> {
  if (name.startsWith('@',)) {
    /**
     Scope part of the name, e.g. `@earendil-works`.
     */
    const scope = name.slice(
      0,
      name.indexOf('/',),
    );
    /**
     Scoped registry setting; pnpm prints `undefined` when unset.
     */
    const scoped = (await runPnpm({
      args: [
        'config',
        'get',
        `${scope}:registry`,
      ],
      cwd: root,
    },)).trim();
    if ((scoped !== '') && (scoped !== 'undefined'))
      return scoped;
  }
  return (await runPnpm({
    args: [
      'config',
      'get',
      'registry',
    ],
    cwd: root,
  },)).trim();
}

/**
 Lists direct dependents of one version, as `name@version`.

 @param dir - workspace whose lockfile holds the pick

 @param spec - pick to explain

 @param runPnpm - pnpm runner

 @returns dependents; empty when a workspace project depends on it directly

 @example
 ```ts
 await dependentsOf({ dir: '/scratch', spec, runPnpm });
 ```
 */
async function dependentsOf({
  dir,
  spec,
  runPnpm,
}: {
  readonly dir: string;
  readonly spec: PackageSpec;
  readonly runPnpm: RunPnpm;
},): Promise<readonly string[]> {
  /**
   `pnpm why` JSON: one entry per installed version with its dependents.
   */
  const output = await runPnpm({
    args: [
      'why',
      spec.name,
      '--recursive',
      '--depth',
      '1',
      '--json',
    ],
    cwd: dir,
  },);
  /**
   Parsed tree, untyped until checked.
   */
  const parsed: unknown = JSON.parse(output,);
  if (!Array.isArray(parsed,))
    return [];
  /**
   Version nodes, element type pinned to `unknown` before inspection.
   */
  const nodes: readonly unknown[] = parsed;
  /**
   Node for this exact version, if pnpm reported one.
   */
  const node: unknown = nodes.find(function isThisVersion(candidate: unknown,): boolean {
    return ((typeof candidate) === 'object') && (candidate !== null)
      && ('version' in candidate)
      && (candidate.version === spec.version);
  },);
  if (((typeof node) !== 'object') || (node === null)
    || (!('dependents' in node)))
    return [];
  /**
   Dependents recorded for this exact version.
   */
  const dependents: unknown = node.dependents;
  if (!Array.isArray(dependents,))
    return [];
  /**
   Dependent entries, element type pinned to `unknown` before inspection.
   */
  const entries: readonly unknown[] = dependents;
  return [...new Set(entries.flatMap(function toSpec(dependent: unknown,): readonly string[] {
    if (((typeof dependent) !== 'object') || (dependent === null)
      || (!('name' in dependent))
      || ((typeof dependent.name) !== 'string'))
      return [];
    return ('version' in dependent) && ((typeof dependent.version) === 'string')
      ? [`${dependent.name}@${dependent.version}`,]
      : [dependent.name,];
  },),),];
}

//endregion Helpers

//region Diagnose

/**
 Runs the loose-mode resolution in a scratch copy and describes each
 version the strict gate would block.

 @param root - workspace root whose update was refused

 @param runPnpm - pnpm runner

 @param fetchImpl - `fetch` used for registry publish times

 @returns blocked picks with publish times and dependents

 @example
 ```ts
 const { picks, minutes } = await diagnoseImmaturePicks({ root: '/repo', runPnpm, fetchImpl: fetch });
 ```
 */
export async function diagnoseImmaturePicks({
  root,
  runPnpm,
  fetchImpl,
}: {
  readonly root: string;
  readonly runPnpm: RunPnpm;
  readonly fetchImpl: FetchLike;
},): Promise<Diagnosis> {
  /**
   Logger tagged with this function.
   */
  const dl = tagged({
    tag: diagnoseImmaturePicks.name,
    l,
  },);
  /**
   Real workspace manifest text, source of the configured age.
   */
  const manifest = await readFile(
    join(
      root,
      'pnpm-workspace.yaml',
    ),
    'utf8',
  );
  /**
   Configured age in minutes, or its absence sentinel.
   */
  const minutes = readMinimumReleaseAge(manifest,);
  /**
   Every workspace project directory to mirror.
   */
  const projectDirs = await listProjectDirs({
    root,
    runPnpm,
  },);
  dl.info(`resolving ${String(projectDirs.length,)} projects in loose mode to find immature picks`,);
  /**
   Scratch copy for the loose resolution; deleted when this function returns.
   */
  await using scratch = await createScratchWorkspace({
    root,
    projectDirs,
  },);
  /**
   Scratch manifest path pnpm rewrites.
   */
  const scratchManifest = join(
    scratch.dir,
    'pnpm-workspace.yaml',
  );
  /**
   Exclude list before pnpm appends picks.
   */
  const before = readExcludeList(await readFile(
    scratchManifest,
    'utf8',
  ),);
  /**
   Failure of the loose resolution, if any; pnpm writes the exclude list
   before later checks (such as strict peers) can still fail the run.
   */
  const looseFailure = await runLooseResolution({
    dir: scratch.dir,
    runPnpm,
  },);
  /**
   Exclude list after pnpm appended picks.
   */
  const after = readExcludeList(await readFile(
    scratchManifest,
    'utf8',
  ),);
  /**
   Versions pnpm had to exempt, sorted for stable output.
   */
  const specs = [...addedVersions({
    before,
    after,
  },),].toSorted(function byKey(
    left,
    right,
  ): number {
    return `${left.name}@${left.version}`.localeCompare(`${right.name}@${right.version}`,);
  },);
  dl.info(`found ${String(specs.length,)} immature pick(s)`,);
  if ((looseFailure !== LOOSE_RESOLUTION_SUCCEEDED) && (specs.length === 0))
    throw looseFailure;
  /**
   First pnpm diagnostic of a loose-resolution failure that happened after the age check.
   */
  const followUp = looseFailure === LOOSE_RESOLUTION_SUCCEEDED
    ? {}
    : { followUp: firstDiagnosticLine(looseFailure.output,), };
  if ('followUp' in followUp)
    dl.warn(`loose resolution also failed after recording picks: ${followUp.followUp}`,);
  /**
   Each pick with its publish time and direct dependents.
   */
  const picks = await Promise.all(specs.map(async function describePick(spec,): Promise<ImmaturePick> {
    /**
     Registry and dependents fetched concurrently.
     */
    const [publishedAt, dependents,] = await Promise.all([
      (async function lookupTime(): Promise<Date> {
        return fetchPublishTime({
          registry: await registryFor({
            root,
            name: spec.name,
            runPnpm,
          },),
          spec,
          fetchImpl,
        },);
      })(),
      dependentsOf({
        dir: scratch.dir,
        spec,
        runPnpm,
      },),
    ],);
    return {
      name: spec.name,
      version: spec.version,
      publishedAt,
      dependents,
    };
  },),);
  return {
    picks,
    minutes,
    ...followUp,
  };
}

//endregion Diagnose
