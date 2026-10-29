/**
 Loads and validates a scope's plan data modules:
 `plan.tools.data.ts` (declarations) plus the generated `plan.artifacts.data.ts`
 (per-platform artifact rows).

 The machine-global scope keeps those files outside the repository
 (`doc/planning/mpm-global-toolchain-migration.md`), as import-free data
 modules; this loader is where they become the typed `ToolPlanEntry[]` the
 sync, status and update flows consume.

 Node builtins only, so the entry points stay dependency-free.
 */

import { pathToFileURL, } from 'node:url';
import { join, } from 'node:path';
import {
  TOOL_ROUTES,
  type ToolPlanEntry,
} from './types.ts';

//region Raw shapes

/**
 Raw declaration record as the data module writes it (unvalidated).
 */
type RawEntry = Record<string, unknown>;

/**
 Raw `plan.artifacts.data.ts` payload.
 */
type RawArtifacts = Readonly<Record<string, Readonly<Record<string, {
  readonly url: string;
  readonly sha256: string;
}>>>>;

//endregion Raw shapes

//region Validation

/**
 Throws a plan validation error naming the input it failed on.
 
 @param options - Plan file path, tool id and the defect to report
 
 @throws Always
 */
function planError(
  {
    file,
    id,
    problem,
  }: {
    readonly file: string;
    readonly id: string | null;
    readonly problem: string;
  },
): never {
  throw new Error(
    `${file}: ${
      id === null
        ? ''
        : `tool ${JSON.stringify(id,)}: `
    }${problem}`,
  );
}

/**
 Validates one raw declaration into a typed plan entry.
 
 @param options - Raw record, tool id, artifact rows and the file name to
 report against
 
 @returns Typed plan entry
 
 @throws Error when a field is missing or malformed
 */
function parseEntry(
  {
    raw,
    artifacts,
    file,
  }: {
    readonly raw: RawEntry;
    readonly artifacts: RawArtifacts;
    readonly file: string;
  },
): ToolPlanEntry {
  /**
   Declared tool id.
   */
  const {id} = raw;
  if (((typeof id) !== 'string') || (id === '')) {
    planError({
      file,
      id: null,
      problem: 'entry carries no string id',
    },);
  }
  /**
   Request intent of the entry.
   */
  const {request} = raw;
  if ((request === null) || ((typeof request) !== 'object')) {
    planError({
      file,
      id,
      problem: 'request is missing',
    },);
  }
  /**
   Request record with exactly one intent key.
   */
  const requestRecord = request as Record<string, unknown>;
  const intentKeys = [
    'track',
    'pinned',
    'channel',
  ]
    .filter(
    function present(key,): boolean {
      return requestRecord[key] !== undefined;
    },
    );
  if ((intentKeys.length !== 1)
    || ((typeof requestRecord[intentKeys[0] ?? '']) !== 'string')) {
    planError({
      file,
      id,
      problem: 'request needs exactly one string intent (track, pinned or channel)',
    },);
  }
  /**
   Resolved version literal.
   */
  const {version} = raw;
  if (((typeof version) !== 'string') || (version === '')) {
    planError({
      file,
      id,
      problem: 'version is missing',
    },);
  }
  /**
   Install route.
   */
  const {route} = raw;
  if (((typeof route) !== 'string')
    || (!TOOL_ROUTES.includes(route as (typeof TOOL_ROUTES)[number],))) {
    planError({
      file,
      id,
      problem: `route ${JSON.stringify(route,)} is not one of ${
        TOOL_ROUTES.join(', ',)
      }`,
    },);
  }
  /**
   Declared binary names.
   */
  const {bins} = raw;
  if ((!Array.isArray(bins,))
    || (bins.length === 0)
    || bins.some(
      function notName(name,): boolean {
        return ((typeof name) !== 'string') || (name === '');
      },
    )) {
    planError({
      file,
      id,
      problem: 'bins must be a non-empty list of names',
    },);
  }
  /**
   Optional alias map.
   */
  const options = raw.options === undefined
    ? {}
    : raw.options;
  if ((options === null) || ((typeof options) !== 'object')) {
    planError({
      file,
      id,
      problem: 'options must be an object',
    },);
  }
  /**
   Optional OS filter.
   */
  const os = raw.os ?? null;
  if ((os !== null)
    && ((!Array.isArray(os,))
      || os.some(
        function notFamily(name,): boolean {
          return (name !== 'linux') && (name !== 'macos')
            && (name !== 'windows');
        },
      ))) {
    planError({
      file,
      id,
      problem: 'os must be null or a list of linux, macos and windows',
    },);
  }
  /**
   Optional environment exports.
   */
  const env = raw.env ?? {};
  if ((env === null) || ((typeof env) !== 'object')) {
    planError({
      file,
      id,
      problem: 'env must be an object',
    },);
  }
  return {
    id,
    request: requestRecord as ToolPlanEntry['request'],
    version,
    route: route as ToolPlanEntry['route'],
    manager: (typeof raw.manager) === 'string'
      ? raw.manager
      : null,
    packageId: (typeof raw.packageId) === 'string'
      ? raw.packageId
      : null,
    bins: bins as string[],
    os: os as ToolPlanEntry['os'],
    options: options as ToolPlanEntry['options'],
    env: env as ToolPlanEntry['env'],
    artifacts: artifacts[id] ?? {},
  };
}

//endregion Validation

//region Loading

/**
 Loads, validates and merges one scope's plan data modules.
 
 @param options - Directory holding `plan.tools.data.ts` and `plan.artifacts.data.ts`
 
 @returns Typed plan entries in declaration order
 
 @throws Error when a module is missing, malformed or inconsistent
 */
export async function loadPlanFromDir(
  {
    dir,
  }: {
    readonly dir: string;
  },
): Promise<readonly ToolPlanEntry[]> {
  /**
   Declarations module of this scope.
   */
  const toolsModule = await import(pathToFileURL(join(
    dir,
    'plan.tools.data.ts',
  ),)
    .href,) as { TOOLS?: unknown; };
  if (!Array.isArray(toolsModule.TOOLS,)) {
    throw new Error(
      `${join(
        dir,
        'plan.tools.data.ts',
      )}: expected a TOOLS array export`,
    );
  }
  /**
   Generated artifact rows of this scope.
   */
  const artifactsModule = await import(pathToFileURL(join(
    dir,
    'plan.artifacts.data.ts',
  ),)
    .href,) as { ARTIFACTS?: unknown; };
  if ((artifactsModule.ARTIFACTS === undefined)
    || (artifactsModule.ARTIFACTS === null)
    || ((typeof artifactsModule.ARTIFACTS) !== 'object')) {
    throw new Error(
      `${join(
        dir,
        'plan.artifacts.data.ts',
      )}: expected an ARTIFACTS object export`,
    );
  }
  /**
   Artifact rows keyed by tool id.
   */
  const artifacts = artifactsModule.ARTIFACTS as RawArtifacts;
  /**
   Parsed entries in declaration order.
   */
  const entries: ToolPlanEntry[] = [];
  /**
   Ids already seen, for the uniqueness check.
   */
  const seenIds = new Set<string>();
  /**
   Exposure name to the owning entry id (a name may appear both as a bin and
   as an alias of one entry, but never under two entries).
   */
  const seenNames = new Map<string, string>();
  for (const raw of toolsModule.TOOLS as RawEntry[]) {
    /**
     Validated entry for this record.
     */
    const entry = parseEntry({
      raw,
      artifacts,
      file: join(
        dir,
        'plan.tools.data.ts',
      ),
    },);
    if (seenIds.has(entry.id,)) {
      planError({
        file: join(
          dir,
          'plan.tools.data.ts',
        ),
        id: entry.id,
        problem: 'declared twice',
      },);
    }
    seenIds.add(entry.id,);
    for (const name of new Set([
      ...entry.bins,
      ...Object.keys(entry.options
        .binAliases
        ?? {},),
    ])) {
      /**
       Entry already claiming this exposure name, when any.
       */
      const owner = seenNames.get(name,);
      if ((owner !== undefined) && (owner !== entry.id)) {
        planError({
          file: join(
            dir,
            'plan.tools.data.ts',
          ),
          id: entry.id,
          problem: `exposure name ${
            JSON.stringify(name,)
          } is already declared by ${
            JSON.stringify(owner,)
          }`,
        },);
      }
      seenNames.set(
        name,
        entry.id,
      );
    }
    entries.push(entry,);
  }
  return entries;
}

//endregion Loading
