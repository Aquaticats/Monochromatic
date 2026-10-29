/**
 Loads and validates a scope's plan data modules:
 `plan.tools.data.ts` (declarations) plus the generated `plan.artifacts.data.ts`
 (per-platform artifact rows).

 The machine-global scope keeps those files outside the repository
 (`doc/planning/mpm-global-toolchain-migration.md`), as import-free data
 modules; this loader is where they become the typed `ToolPlanEntry[]` the
 sync, status and update flows consume. Field validation lives in
 `plan.load.guards.ts`.

 Node builtins only, so the entry points stay dependency-free.
 */

import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import {
  type Fail,
  isRecord,
  optionalName,
  parseArtifacts,
  parseBins,
  parseEnv,
  parseOptions,
  parseOs,
  parseRequest,
  parseRoute,
} from './plan.load.guards.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Validation

/**
 Builds the error reporter for one plan file, optionally scoped to one tool.

 @param file - plan file path

 @param id - tool id the defects belong to; absent for file-level defects

 @returns reporter that throws an error naming the file, the tool, and the defect

 @example
 ```ts
 planFailure({ file: 'plan.tools.data.ts', id: 'age', })('version is missing');
 ```
 */
function planFailure(
  {
    file,
    id,
  }: {
    readonly file: string;
    readonly id?: string;
  },
): Fail {
  return function fail(problem: string,): never {
    throw new Error(`${file}: ${id === undefined ? '' : `tool ${JSON.stringify(id,)}: `}${problem}`,);
  };
}

/**
 Validates one raw declaration into a typed plan entry.

 @param raw - raw record as the data module writes it

 @param artifacts - artifact rows keyed by tool id

 @param file - plan file to name in errors

 @returns typed plan entry

 @throws Error when a field is missing or malformed

 @example
 ```ts
 parseEntry({ raw, artifacts, file: 'plan.tools.data.ts', });
 ```
 */
function parseEntry(
  {
    raw,
    artifacts,
    file,
  }: {
    readonly raw: unknown;
    readonly artifacts: Readonly<Record<string, ToolArtifacts>>;
    readonly file: string;
  },
): ToolPlanEntry {
  if (!isRecord(raw,))
    return planFailure({ file, },)('entry is not an object',);
  /**
   Declared tool id.
   */
  const { id, } = raw;
  if (((typeof id) !== 'string') || (id === ''))
    return planFailure({ file, },)('entry carries no string id',);
  /**
   Reporter for this tool's defects.
   */
  const fail = planFailure({
    file,
    id,
  },);
  /**
   Resolved version literal.
   */
  const { version, } = raw;
  if (((typeof version) !== 'string') || (version === ''))
    return fail('version is missing',);
  /**
   MPM manager, when one declares the tool.
   */
  const [manager,] = optionalName({
    key: 'manager',
    value: raw.manager,
    fail,
  },);
  /**
   Package id under the manager, when one declares the tool.
   */
  const [packageId,] = optionalName({
    key: 'packageId',
    value: raw.packageId,
    fail,
  },);
  /**
   OS filter, when the tool is not for every OS.
   */
  const [os,] = parseOs({
    value: raw.os,
    fail,
  },);
  return {
    id,
    request: parseRequest({
      value: raw.request,
      fail,
    },),
    version,
    route: parseRoute({
      value: raw.route,
      fail,
    },),
    ...(manager === undefined ? {} : { manager, }),
    ...(packageId === undefined ? {} : { packageId, }),
    bins: parseBins({
      value: raw.bins,
      fail,
    },),
    ...(os === undefined ? {} : { os, }),
    options: parseOptions({
      value: raw.options,
      fail,
    },),
    env: parseEnv({
      value: raw.env,
      fail,
    },),
    artifacts: artifacts[id] ?? {},
  };
}

//endregion Validation

//region Loading

/**
 One named export of a dynamically imported module.

 @param path - module file

 @param name - export name

 @returns the export's value, `undefined` when the module lacks it

 @example
 ```ts
 await moduleExport({ path: '/x/plan.tools.data.ts', name: 'TOOLS', });
 ```
 */
async function moduleExport(
  {
    path,
    name,
  }: {
    readonly path: string;
    readonly name: string;
  },
): Promise<unknown> {
  /**
   Imported module namespace.
   */
  const namespace: unknown = await import(pathToFileURL(path,)
    .href);
  return isRecord(namespace,) ? namespace[name] : undefined;
}

/**
 Loads, validates and merges one scope's plan data modules.

 @param dir - directory holding `plan.tools.data.ts` and `plan.artifacts.data.ts`

 @returns typed plan entries in declaration order

 @throws Error when a module is missing, malformed or inconsistent

 @example
 ```ts
 await loadPlanFromDir({ dir: globalToolchainPaths().worktreeRoot, });
 ```
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
  const toolsFile = join(
    dir,
    'plan.tools.data.ts',
  );
  /**
   Generated artifact module of this scope.
   */
  const artifactsFile = join(
    dir,
    'plan.artifacts.data.ts',
  );
  /**
   Raw declarations.
   */
  const tools = await moduleExport({
    path: toolsFile,
    name: 'TOOLS',
  },);
  if (!Array.isArray(tools,))
    return planFailure({ file: toolsFile, },)('expected a TOOLS array export',);
  /**
   Artifact rows keyed by tool id.
   */
  const artifacts = parseArtifacts({
    value: await moduleExport({
      path: artifactsFile,
      name: 'ARTIFACTS',
    },),
    fail: planFailure({ file: artifactsFile, },),
  },);
  /**
   Ids already seen, for the uniqueness check.
   */
  const seenIds = new Set<string>();
  /**
   Exposure name to the owning entry id (a name may appear both as a bin and
   as an alias of one entry, but never under two entries).
   */
  const seenNames = new Map<string, string>();
  return tools.map(function validate(raw: unknown,): ToolPlanEntry {
    /**
     Validated entry for this record.
     */
    const entry = parseEntry({
      raw,
      artifacts,
      file: toolsFile,
    },);
    /**
     Reporter for this entry's cross-entry defects.
     */
    const fail = planFailure({
      file: toolsFile,
      id: entry.id,
    },);
    if (seenIds.has(entry.id,))
      return fail('declared twice',);
    seenIds.add(entry.id,);
    for (const name of new Set([
      ...entry.bins,
      ...Object.keys(entry.options
        .binAliases
        ?? {},),
    ],)) {
      /**
       Entry already claiming this exposure name, when any.
       */
      const owner = seenNames.get(name,);
      if ((owner !== undefined) && (owner !== entry.id))
        return fail(`exposure name ${JSON.stringify(name,)} is already declared by ${JSON.stringify(owner,)}`,);
      seenNames.set(
        name,
        entry.id,
      );
    }
    return entry;
  },);
}

//endregion Loading
