/**
 Reads what the catalog sync needs from one crate manifest.

 @module
 */

import { posix, } from 'node:path';

import {
  type LocatedDocument,
  type LocatedPair,
  locateSource,
  pathKey,
} from './locate.ts';
import {
  classifySite,
  dependencyTables,
  isTable,
  tableAt,
} from './scan.ts';
import type {
  ManifestFile,
  SyncIssue,
  UseSite,
} from './types.ts';

//region Types

/**
 One parsed crate manifest with its use sites and named-catalog selections.

 @example
 ```ts
 const info = readCrateInfo({ manifest, issues, });
 info.sites.length;
 ```
 */
export type CrateInfo = {
  readonly manifest: ManifestFile;
  readonly located: LocatedDocument;
  readonly crateDir: string;
  readonly packageName: string;
  readonly packageVersion: string;
  readonly selections: ReadonlyMap<string, string>;
  readonly sites: readonly UseSite[];
};

//endregion Types

//region Read

/**
 Reads `[package.metadata.rust-catalog.catalogs]`, the crate's named-catalog selections.

 @param manifestPath - manifest path for diagnostics

 @param value - static manifest value

 @param issues - collector the problems are appended to

 @returns dependency name to catalog name

 @example
 ```ts
 readSelections({ manifestPath, value, issues, });
 ```
 */
function readSelections(
  {
    manifestPath,
    value,
    issues,
  }: {
    readonly manifestPath: string;
    readonly value: Readonly<Record<string, unknown>>;
    readonly issues: SyncIssue[]
  },
): ReadonlyMap<string, string> {
  /**
   Selection table, empty when the crate selects nothing.
   */
  const table = tableAt({
    manifest: value,
    path: [
      'package',
      'metadata',
      'rust-catalog',
      'catalogs',
    ],
  },);
  /**
   Selections accepted so far.
   */
  const selections = new Map<string, string>();
  for (const [name, catalogName,] of Object.entries(table,)) {
    if ((typeof catalogName) !== 'string') {
      issues.push({
        file: manifestPath,
        location: `package.metadata.rust-catalog.catalogs.${name}`,
        problem: 'is not a catalog name',
        remedy: `write \`${name} = "<catalog name>"\` naming a \`[catalogs.<name>]\` table in rust-catalog.toml`,
      },);
      continue;
    }
    selections.set(
      name,
      catalogName,
    );
  }
  return selections;
}

/**
 Parses one manifest and classifies its dependency entries.

 @param manifest - manifest path and text

 @param issues - collector the problems are appended to

 @returns crate information for the sync

 @throws Error from `toml-eslint-parser` when the manifest is not valid TOML

 @example
 ```ts
 const info = readCrateInfo({ manifest: { path: 'package/a/b/Cargo.toml', source, }, issues, });
 ```
 */
export function readCrateInfo({
  manifest,
  issues,
}: {
  readonly manifest: ManifestFile;
  readonly issues: SyncIssue[]
},): CrateInfo {
  /**
   Projected document with source ranges and the plain value.
   */
  const located = locateSource(manifest.source,);
  /**
   Manifest value as a table.
   */
  const value = isTable(located.value,) ? located.value : {};
  /**
   `[package]` table.
   */
  const pkg = tableAt({
    manifest: value,
    path: ['package',],
  },);
  if ((!located.tables
    .has(pathKey([
      'workspace',
      'dependencies',
    ],),))
    && located.pairs
      .some(function isWorkspaceDependency(pair: LocatedPair,): boolean {
        return (pair.path[0] === 'workspace') && (pair.path[1] === 'dependencies');
      },)) {
    issues.push({
      file: manifest.path,
      location: 'workspace.dependencies',
      problem: 'is written as an inline table or dotted keys',
      remedy: 'delete it; file-enforcer generates a `[workspace.dependencies]` table from rust-catalog.toml',
    },);
  }
  /**
   Every classified use site.
   */
  const sites = dependencyTables(value,)
    .flatMap(function sitesOf(table,): readonly UseSite[] {
      return Object.entries(tableAt({
        manifest: value,
        path: table,
      },),)
        .flatMap(function classify([name, entry,],): readonly UseSite[] {
          return classifySite({
            manifestPath: manifest.path,
            table,
            name,
            value: entry,
            issues,
          },);
        },);
    },);
  return {
    manifest,
    located,
    crateDir: posix.dirname(manifest.path,),
    packageName: (typeof pkg.name) === 'string' ? pkg.name : '',
    packageVersion: (typeof pkg.version) === 'string' ? pkg.version : '',
    selections: readSelections({
      manifestPath: manifest.path,
      value,
      issues,
    },),
    sites,
  };
}

//endregion Read
