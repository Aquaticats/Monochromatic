/**
 Pure sync of crate manifests with `rust-catalog.toml`.

 Reads the catalog and every manifest, adopts literal versions, resolves each
 crate's generated `[workspace.dependencies]` table and internal path versions,
 finds unused entries, and returns new file texts. Every problem is collected
 first and thrown together, so a sync never writes half its changes.

 @module
 */

import { posix, } from 'node:path';

import {
  parseRustCatalog,
  type RustCatalogEntry,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import {
  planAdoption,
  siteLocation,
} from './adopt.ts';
import {
  type CrateInfo,
  readCrateInfo,
} from './crate-info.ts';
import { generatedEntries, } from './default-features.ts';
import { catalogSplices, } from './edit-catalog.ts';
import {
  generatedTableSplices,
  inheritSplices,
  pathVersionSplices,
} from './edit-manifest.ts';
import { locateSource, } from './locate.ts';
import {
  applySplices,
  type Splice,
} from './text-splice.ts';
import type {
  GeneratedEntry,
  ManifestFile,
  SyncIssue,
  SyncResult,
} from './types.ts';

//region Errors

/**
 Thrown when manifests and the catalog cannot be brought in line without a human decision.

 @example
 ```ts
 try { syncRustCatalog(input); } catch (error) { if (error instanceof RustCatalogSyncError) console.error(error.issues); throw error; }
 ```
 */
export class RustCatalogSyncError extends Error {
  /**
   Every problem found, in discovery order.
   */
  readonly issues: readonly SyncIssue[];

  /**
   Builds the error from one pass's issues.

   @param issues - problems to report
   */
  constructor({ issues, }: { readonly issues: readonly SyncIssue[]; },) {
    super([
      `The Rust dependency catalog sync found ${String(issues.length,)} problem(s); nothing was written:`,
      ...issues.map(function format(issue,): string {
        return `- ${issue.file} ${issue.location}: ${issue.problem}.\n  Fix: ${issue.remedy}.`;
      },),
      'See doc/decision/rust-dependency-catalog.md.',
    ].join('\n',),);
    this.name = 'RustCatalogSyncError';
    this.issues = issues;
  }
}

//endregion Errors

//region Resolve

/**
 Joined key for a named-catalog entry in usage sets.

 @param catalogName - named catalog

 @param name - dependency name

 @returns usage key

 @example
 ```ts
 namedKey({ catalogName: 'pinned', name: 'salsa', });
 ```
 */
function namedKey({
  catalogName,
  name,
}: {
  readonly catalogName: string;
  readonly name: string
},): string {
  return `${catalogName}\u0000${name}`;
}

/**
 Resolves one crate's generated entries and path-version splices.

 @param info - crate information

 @param defaultCatalog - default catalog after adoption

 @param namedCatalogs - named catalogs

 @param internal - first-party crates by directory

 @param usage - default and named usage collectors

 @param issues - collector the problems are appended to

 @returns generated entries and splices for this manifest

 @example
 ```ts
 resolveCrate({ info, defaultCatalog, namedCatalogs, internal, usage, issues, });
 ```
 */
function resolveCrate(
  {
    info,
    defaultCatalog,
    namedCatalogs,
    internal,
    usage,
    issues,
  }: {
    readonly info: CrateInfo;
    readonly defaultCatalog: ReadonlyMap<string, RustCatalogEntry>;
    readonly namedCatalogs: ReadonlyMap<string, ReadonlyMap<string, RustCatalogEntry>>;
    readonly internal: ReadonlyMap<string, CrateInfo>;
    readonly usage: {
      readonly defaultNames: Set<string>;
      readonly namedNames: Set<string>
    };
    readonly issues: SyncIssue[];
  },
): {
  readonly entries: ReadonlyMap<string, GeneratedEntry>;
  readonly splices: readonly Splice[]
} {
  /**
   Resolved catalog entries for this crate.
   */
  const resolvedEntries = new Map<string, RustCatalogEntry>();
  /**
   Splices for this manifest's use sites.
   */
  const splices: Splice[] = [];
  /**
   Names this crate resolves through the catalog, to validate selections.
   */
  const catalogNames = new Set<string>();
  for (const site of info.sites) {
    if (site.kind === 'path') {
      /**
       First-party crate the path points at.
       */
      const target = internal.get(
        posix.normalize(posix.join(
          info.crateDir,
          site.path,
        ),),
      );
      if ((target === undefined) || (target.packageVersion === '')) {
        issues.push({
          file: info.manifest
            .path,
          location: siteLocation(site,),
          problem: `path ${JSON.stringify(site.path,)} does not point at a first-party crate with a package.version`,
          remedy: 'point `path` at a crate directory under package/ that declares `version`, or depend on a registry release instead',
        },);
        continue;
      }
      splices.push(...pathVersionSplices({
        source: info.manifest
          .source,
        located: info.located,
        site,
        version: target.packageVersion,
      },),);
      continue;
    }
    catalogNames.add(site.name,);
    /**
     Named catalog this crate selects for the dependency, if any.
     */
    const catalogName = info.selections
      .get(site.name,);
    /**
     Resolved entry.
     */
    const entry = catalogName === undefined ? defaultCatalog.get(site.name,) : namedCatalogs.get(catalogName,)
      ?.get(site.name,);
    // A literal without an entry already failed adoption, which reported why;
    // a second "missing entry" issue would blame crates that are not at fault.
    if ((entry === undefined) && (site.kind === 'literal'))
      continue;
    if (entry === undefined) {
      issues.push({
        file: info.manifest
          .path,
        location: siteLocation(site,),
        problem: catalogName === undefined
          ? `uses \`workspace = true\` but rust-catalog.toml [catalog] has no ${site.name} entry`
          : `selects catalog "${catalogName}", which has no ${site.name} entry`,
        remedy: `add ${site.name} to rust-catalog.toml, or write a version here and run the sync to adopt it`,
      },);
      continue;
    }
    if (catalogName === undefined)
      usage.defaultNames
        .add(site.name,);
    else
      usage.namedNames
        .add(namedKey({
          catalogName,
          name: site.name,
        },),);
    resolvedEntries.set(
      site.name,
      entry,
    );
    if (site.kind === 'literal')
      splices.push(...inheritSplices({
        source: info.manifest
          .source,
        located: info.located,
        site,
      },),);
  }
  for (const [name, catalogName,] of info.selections) {
    if (!catalogNames.has(name,)) {
      issues.push({
        file: info.manifest
          .path,
        location: `package.metadata.rust-catalog.catalogs.${name}`,
        problem: `selects catalog "${catalogName}" for ${name}, which this crate does not depend on`,
        remedy: 'remove the selection',
      },);
    }
  }
  return {
    entries: generatedEntries({
      info,
      resolvedEntries,
      issues,
    },),
    splices,
  };
}

//endregion Resolve

//region Sync

/**
 Brings every manifest and the catalog in line.

 @param catalogPath - catalog path for diagnostics

 @param catalogSource - catalog text

 @param manifests - every first-party crate manifest

 @returns new texts, affected crates, and unused-entry warnings

 @throws RustCatalogError when the catalog itself is invalid

 @throws RustCatalogSyncError when manifests need a human decision

 @example
 ```ts
 const result = syncRustCatalog({ catalogPath: 'rust-catalog.toml', catalogSource, manifests, });
 ```
 */
export function syncRustCatalog(
  {
    catalogPath,
    catalogSource,
    manifests,
  }: {
    readonly catalogPath: string;
    readonly catalogSource: string;
    readonly manifests: readonly ManifestFile[];
  },
): SyncResult {
  /**
   Validated catalog.
   */
  const document = parseRustCatalog({
    source: catalogSource,
    sourcePath: catalogPath,
  },);
  /**
   Problems across every file.
   */
  const issues: SyncIssue[] = [];
  /**
   Parsed manifests.
   */
  const crates = manifests.map(function read(manifest,): CrateInfo {
    return readCrateInfo({
      manifest,
      issues,
    },);
  },);
  /**
   First-party crates by directory.
   */
  const internal = new Map(crates.map(function byDir(info,): readonly [
    string,
    CrateInfo
  ] {
    return [
      info.crateDir,
      info,
    ];
  },),);
  /**
   Adoption of literal versions.
   */
  const adoption = planAdoption({
    document,
    sites: crates.flatMap(function sitesOf(info,) {
      return info.sites;
    },),
    selections: new Map(crates.map(function selectionsOf(info,): readonly [
      string,
      ReadonlyMap<string, string>
    ] {
      return [
        info.manifest
          .path,
        info.selections,
      ];
    },),),
    issues,
  },);
  /**
   Catalog entries referenced by any crate.
   */
  const usage = {
    defaultNames: new Set<string>(),
    namedNames: new Set<string>(),
  };
  /**
   Per-crate resolution.
   */
  const resolved = crates.map(function resolve(info,) {
    return {
      info,
      ...resolveCrate({
        info,
        defaultCatalog: adoption.defaultCatalog,
        namedCatalogs: document.catalogs,
        internal,
        usage,
        issues,
      },),
    };
  },);

  /**
   Unused entry paths.
   */
  const unused = [
    ...[...adoption.defaultCatalog
      .keys(),].filter(function isUnused(name,): boolean {
        return !usage.defaultNames
          .has(name,);
      },)
      .map(function toPath(name,): readonly string[] {
        return [
          'catalog',
          name,
        ];
      },),
    ...[...document.catalogs
      .keys(),].flatMap(function unusedNamed(catalogName: string,): readonly (readonly string[])[] {
        return [...(document.catalogs
          .get(catalogName,)
          ?.keys()
          ?? []),].filter(function isUnused(name: string,): boolean {
            return !usage.namedNames
              .has(namedKey({
                catalogName,
                name,
              },),);
          },)
          .map(function toPath(name,): readonly string[] {
            return [
              'catalogs',
              catalogName,
              name,
            ];
          },);
      },),
  ];
  if (issues.length > 0)
    throw new RustCatalogSyncError({ issues, },);

  /**
   New manifest texts and the crates whose generated table changed.
   */
  const manifestTexts = new Map<string, string>();
  /**
   Crate directories whose generated table changed.
   */
  const affectedCrates: string[] = [];
  for (const {
    info,
    entries,
    splices,
  } of resolved) {
    /**
     Splice for the generated table.
     */
    const tableSplices = generatedTableSplices({
      source: info.manifest
        .source,
      located: info.located,
      entries,
    },);
    if (tableSplices.length > 0)
      affectedCrates.push(info.crateDir,);
    manifestTexts.set(
      info.manifest
        .path,
      applySplices({
        source: info.manifest
          .source,
        splices: [
          ...splices,
          ...tableSplices,
        ],
      },),
    );
  }
  /**
   Pruned paths when pruning is on; otherwise they become warnings.
   */
  const prunes = document.settings
    .catalogPrune ? unused : [];
  return {
    catalog: applySplices({
      source: catalogSource,
      splices: catalogSplices({
        source: catalogSource,
        located: locateSource(catalogSource,),
        additions: adoption.additions,
        floorRaises: adoption.floorRaises,
        prunes,
      },),
    },),
    manifests: manifestTexts,
    affectedCrates,
    warnings: document.settings
      .catalogPrune ? [] : unused.map(function toWarning(path,): string {
        return `${catalogPath}: ${path.join('.',)} is used by no crate; set catalogPrune = true to remove unused entries on sync`;
      },),
  };
}

//endregion Sync
