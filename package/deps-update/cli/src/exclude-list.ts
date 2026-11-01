/**
 Reads `minimumReleaseAgeExclude` and `minimumReleaseAge` from
 `pnpm-workspace.yaml` text, and diffs exclude lists to find the entries
 pnpm appended during a loose-mode resolution.

 @module
 */

import { parse as parseYaml, } from 'yaml';

import {
  expandExcludeEntry,
  type PackageSpec,
} from './spec.ts';

//region Errors

/**
 Thrown when a workspace manifest field has a shape pnpm itself would reject.
 */
export class WorkspaceManifestShapeError extends Error {
  /**
   Builds the error naming the field and the shape found.

   @param field - workspace manifest key that failed validation

   @param found - human-readable description of the offending value

   @example
   ```ts
   throw new WorkspaceManifestShapeError({ field: 'minimumReleaseAgeExclude', found: 'a string' });
   ```
   */
  constructor({
    field,
    found,
  }: {
    readonly field: string;
    readonly found: string;
  },) {
    super(`pnpm-workspace.yaml ${field} has an unexpected shape: ${found}`,);
    this.name = 'WorkspaceManifestShapeError';
  }
}

//endregion Errors

//region Read

/**
 Parses workspace manifest text into a plain record.

 @param yamlText - `pnpm-workspace.yaml` contents

 @returns top-level mapping; empty when the document is empty

 @throws WorkspaceManifestShapeError when the document root is not a mapping

 @example
 ```ts
 parseManifest('minimumReleaseAge: 1440\n');
 ```
 */
function parseManifest(yamlText: string,): object {
  /**
   Parsed document root, untyped until checked.
   */
  const root: unknown = parseYaml(yamlText,);
  if ((root === null) || (root === undefined))
    return {};
  if (((typeof root) !== 'object') || Array.isArray(root,)) {
    throw new WorkspaceManifestShapeError({
      field: '(root)',
      found: typeof root,
    },);
  }
  return root;
}

/**
 Reads one top-level manifest key without asserting the manifest's shape.

 @param yamlText - `pnpm-workspace.yaml` contents

 @param key - top-level key

 @returns raw value, `undefined` when absent

 @example
 ```ts
 readManifestKey({ yamlText: 'a: 1\n', key: 'a' }); // 1
 ```
 */
function readManifestKey({
  yamlText,
  key,
}: {
  readonly yamlText: string;
  readonly key: string;
},): unknown {
  /**
   Parsed manifest mapping.
   */
  const manifest = parseManifest(yamlText,);
  return key in manifest
    ? Reflect.get(
      manifest,
      key,
    )
    : undefined;
}

/**
 Reads `minimumReleaseAgeExclude` entries.

 @param yamlText - `pnpm-workspace.yaml` contents

 @returns exclude entries in file order; empty when the key is absent

 @throws WorkspaceManifestShapeError when the key is not a list of strings

 @example
 ```ts
 readExcludeList("minimumReleaseAgeExclude:\n  - 'left-pad'\n"); // ['left-pad']
 ```
 */
export function readExcludeList(yamlText: string,): readonly string[] {
  /**
   Raw exclude value before validation.
   */
  const value = readManifestKey({
    yamlText,
    key: 'minimumReleaseAgeExclude',
  },);
  if ((value === undefined) || (value === null))
    return [];
  if (!Array.isArray(value,)) {
    throw new WorkspaceManifestShapeError({
      field: 'minimumReleaseAgeExclude',
      found: typeof value,
    },);
  }
  return value.map(function requireString(entry: unknown,): string {
    if ((typeof entry) !== 'string') {
      throw new WorkspaceManifestShapeError({
        field: 'minimumReleaseAgeExclude',
        found: `entry of type ${typeof entry}`,
      },);
    }
    return entry;
  },);
}

/**
 Sentinel for an absent `minimumReleaseAge`, so callers skip maturity times
 rather than guess pnpm's default.
 */
export const NO_MINIMUM_RELEASE_AGE: unique symbol = Symbol('deps-update/no-minimum-release-age',);

/**
 Reads `minimumReleaseAge` in minutes.

 @param yamlText - `pnpm-workspace.yaml` contents

 @returns configured minutes, or {@link NO_MINIMUM_RELEASE_AGE} when unset

 @throws WorkspaceManifestShapeError when the key is set but not a non-negative number

 @example
 ```ts
 readMinimumReleaseAge('minimumReleaseAge: 1440\n'); // 1440
 ```
 */
export function readMinimumReleaseAge(yamlText: string,): number | typeof NO_MINIMUM_RELEASE_AGE {
  /**
   Raw age value before validation.
   */
  const value = readManifestKey({
    yamlText,
    key: 'minimumReleaseAge',
  },);
  if ((value === undefined) || (value === null))
    return NO_MINIMUM_RELEASE_AGE;
  if (((typeof value) !== 'number') || (!Number.isFinite(value,))
    || (value < 0)) {
    throw new WorkspaceManifestShapeError({
      field: 'minimumReleaseAge',
      found: JSON.stringify(value,),
    },);
  }
  return value;
}

//endregion Read

//region Diff

/**
 Joins a spec into a lookup key.

 @param spec - exact package version

 @returns `name@version`

 @example
 ```ts
 specKey({ name: 'a', version: '1.0.0' }); // 'a@1.0.0'
 ```
 */
function specKey(spec: PackageSpec,): string {
  return `${spec.name}@${spec.version}`;
}

/**
 Lists exact versions named in `after` but not in `before`.

 pnpm's loose mode records each immature pick as an exact version, either as
 a new `name@version` entry or merged into an existing entry for the same
 name (`name@old || new`). Comparing expanded versions, not raw entry
 strings, finds the picks under both shapes.

 @param before - exclude list prior to resolution

 @param after - exclude list pnpm wrote back

 @returns newly exempted versions, in `after` order

 @example
 ```ts
 addedVersions({ before: ['a@1.0.0'], after: ['a@1.0.0 || 1.1.0'] });
 // [{ name: 'a', version: '1.1.0' }]
 ```
 */
export function addedVersions({
  before,
  after,
}: {
  readonly before: readonly string[];
  readonly after: readonly string[];
},): readonly PackageSpec[] {
  /**
   Versions already exempted before resolution.
   */
  const existing = new Set(before.flatMap(function expandBefore(entry,): readonly PackageSpec[] {
    return expandExcludeEntry(entry,);
  },)
    .map(function keyBefore(spec,): string {
      return specKey(spec,);
    },),);
  return after.flatMap(function expandAfter(entry,): readonly PackageSpec[] {
    return expandExcludeEntry(entry,);
  },)
    .filter(function isNew(spec,): boolean {
      return !existing.has(specKey(spec,),);
    },);
}

//endregion Diff
