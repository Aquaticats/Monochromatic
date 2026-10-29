/**
 Finds and classifies every dependency use site in one crate manifest.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import type {
  DependencyTablePath,
  SyncIssue,
  UseSite,
} from './types.ts';

//region Rules

/**
 Dependency table names Cargo reads, at top level and under `target.<cfg>`.
 */
const DEPENDENCY_TABLES = [
  'dependencies',
  'dev-dependencies',
  'build-dependencies',
] as const;

/**
 Keys a `workspace = true` use site may carry beside it (Cargo's own rule).
 */
const INHERITED_SITE_KEYS: ReadonlySet<string> = new Set([
  'default-features',
  'features',
  'optional',
  'workspace',
],);

/**
 Keys a literal or path use site may carry; the catalog moves or keeps each.
 */
const LITERAL_SITE_KEYS: ReadonlySet<string> = new Set([
  'default-features',
  'features',
  'git',
  'optional',
  'package',
  'path',
  'rev',
  'version',
],);

/**
 Placeholder literal for sites that are not literals.
 */
const NO_LITERAL: RustCatalogEntry = {
  kind: 'registry',
  version: '',
};

//endregion Rules

//region Helpers

/**
 Whether `value` is a plain table.

 @param value - static TOML value

 @returns whether it is a non-array object

 @example
 ```ts
 isTable({}); // true
 ```
 */
export function isTable(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Every dependency table path present in a manifest value, in Cargo's reading order.

 @param manifest - static value of the whole manifest

 @returns table paths, top-level first, then each `target.<cfg>` in source order

 @example
 ```ts
 dependencyTables({ dependencies: {}, target: { 'cfg(unix)': { dependencies: {}, }, }, });
 // [['dependencies'], ['target', 'cfg(unix)', 'dependencies']]
 ```
 */
export function dependencyTables(manifest: Readonly<Record<string, unknown>>,): readonly DependencyTablePath[] {
  /**
   Paths found so far.
   */
  const paths: DependencyTablePath[] = DEPENDENCY_TABLES
    .filter(function isPresent(table,): boolean {
      return isTable(manifest[table],);
    },)
    .map(function toPath(table,): DependencyTablePath {
      return [table,];
    },);
  /**
   Platform-specific tables.
   */
  const {target} = manifest;
  if (isTable(target,)) {
    for (const [cfg, body,] of Object.entries(target,)) {
      if (!isTable(body,))
        continue;
      for (const table of DEPENDENCY_TABLES) {
        if (isTable(body[table],))
          paths.push([
            'target',
            cfg,
            table,
          ],);
      }
    }
  }
  return paths;
}

/**
 Reads the table at `path` from a static manifest value.

 @param manifest - static value of the whole manifest

 @param path - table path

 @returns the table, or an empty table when absent

 @example
 ```ts
 tableAt({ manifest, path: ['dependencies',], });
 ```
 */
export function tableAt(
  {
    manifest,
    path,
  }: {
    readonly manifest: Readonly<Record<string, unknown>>;
    readonly path: readonly string[]
  },
): Readonly<Record<string, unknown>> {
  /**
   Value reached so far.
   */
  const reached = path.reduce< unknown>(
    function step(
      current: unknown,
      segment: string,
    ): unknown {
      return isTable(current,) ? current[segment] : undefined;
    },
    manifest,
  );
  return isTable(reached,) ? reached : {};
}

//endregion Helpers

//region Classify

/**
 Classifies one dependency entry, recording problems as issues.

 @param manifestPath - manifest the entry is in

 @param table - dependency table path

 @param name - dependency key

 @param value - static entry value

 @param issues - collector the problems are appended to

 @returns the classified site, or none when the entry cannot be handled

 @example
 ```ts
 classifySite({ manifestPath, table: ['dependencies',], name: 'anyhow', value: '1', issues, });
 ```
 */
export function classifySite(
  {
    manifestPath,
    table,
    name,
    value,
    issues,
  }: {
    readonly manifestPath: string;
    readonly table: DependencyTablePath;
    readonly name: string;
    readonly value: unknown;
    readonly issues: SyncIssue[];
  },
): readonly UseSite[] {
  /**
   Dotted location used in diagnostics.
   */
  const location = [
    ...table,
    name,
  ].join('.',);
  /**
   Fields shared by every site of this entry.
   */
  const base = {
    manifestPath,
    table,
    name,
    disablesDefaultFeatures: isTable(value,) && (value['default-features'] === false),
  };
  if ((typeof value) === 'string')
    return [{
      ...base,
      kind: 'literal',
      literal: {
        kind: 'registry',
        version: value,
      },
      path: '',
    },];
  if (!isTable(value,)) {
    issues.push({
      file: manifestPath,
      location,
      problem: 'is neither a version string nor a table',
      remedy: 'write `name = "1.2"` or `name = { workspace = true }`',
    },);
    return [];
  }

  if (value.workspace === true) {
    /**
     Keys Cargo rejects beside `workspace = true`.
     */
    const extra = Object.keys(value,)
      .filter(function isExtra(key,): boolean {
        return !INHERITED_SITE_KEYS.has(key,);
      },);
    if (extra.length > 0) {
      issues.push({
        file: manifestPath,
        location,
        problem: `declares ${extra.map(function quote(key,): string {
          return `\`${key}\``;
        },)
          .join(', ',)} beside \`workspace = true\``,
        remedy: 'move version, git, rev, and package into rust-catalog.toml; keep only features, optional, and default-features here',
      },);
      return [];
    }
    return [{
      ...base,
      kind: 'catalog',
      literal: NO_LITERAL,
      path: '',
    },];
  }

  /**
   Keys the catalog cannot move or keep.
   */
  const unsupported = Object.keys(value,)
    .filter(function isUnsupported(key,): boolean {
      return !LITERAL_SITE_KEYS.has(key,);
    },);
  if (unsupported.length > 0) {
    issues.push({
      file: manifestPath,
      location,
      problem: `declares ${unsupported.join(', ',)}, which the Rust catalog does not support`,
      remedy: 'pin git dependencies with `rev`; registries other than crates.io and artifact dependencies need a decision record before the catalog can carry them',
    },);
    return [];
  }

  /**
   Optional rename target.
   */
  const rename = (typeof value.package) === 'string' ? { package: value.package, } : {};
  if ((typeof value.path) === 'string')
    return [{
      ...base,
      kind: 'path',
      literal: NO_LITERAL,
      path: value.path,
    },];
  if ((typeof value.git) === 'string') {
    if ((typeof value.rev) !== 'string') {
      issues.push({
        file: manifestPath,
        location,
        problem: 'git dependency has no `rev`',
        remedy: 'pin it with `rev = "<full commit hash>"`',
      },);
      return [];
    }
    return [{
      ...base,
      kind: 'literal',
      literal: {
        kind: 'git',
        git: value.git,
        rev: value.rev,
        ...rename,
      },
      path: '',
    },];
  }
  if ((typeof value.version) === 'string')
    return [{
      ...base,
      kind: 'literal',
      literal: {
        kind: 'registry',
        version: value.version,
        ...rename,
      },
      path: '',
    },];
  issues.push({
    file: manifestPath,
    location,
    problem: 'has no version, git, path, or `workspace = true`',
    remedy: 'add a version, or `workspace = true` with a catalog entry',
  },);
  return [];
}

//endregion Classify
