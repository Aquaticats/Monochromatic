/**
 Plans and writes tightened `>=` floors for `rust-catalog.toml`.

 A default-catalog floor rises to the highest version any consuming crate's
 `Cargo.lock` holds for that direct dependency (the catalog's one-version
 premise). Named-catalog entries and git pins are skipped, and no lockfile is
 ever written. See `doc/decision/rust-dependency-catalog.md`.

 @module
 */

import {
  compareVersions,
  floorVersion,
  parseVersion,
  type ParsedVersion,
  type RustCatalogDocument,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

//region Types

/**
 What one crate consumes from the default catalog and what its lockfile resolved.

 @example
 ```ts
 const crate: CrateUsage = { crateDir: 'package/cli/a', uses: new Set(['anyhow']), lockVersions: new Map([['anyhow', ['1.0.104']]]), };
 ```
 */
export type CrateUsage = {
  readonly crateDir: string;
  readonly uses: ReadonlySet<string>;
  readonly lockVersions: ReadonlyMap<string, readonly string[]>;
};

/**
 Outcome for one catalog entry, with the statuses the JS tightener prints.

 @example
 ```ts
 const result: CargoTightenResult = { name: 'anyhow', status: 'TIGHT', oldRequirement: '>=1', newRequirement: '>=1.0.104', detail: '' };
 ```
 */
export type CargoTightenResult = {
  readonly name: string;
  readonly status: 'MISS' | 'OK' | 'SKIP' | 'TIGHT';
  readonly oldRequirement: string;
  readonly newRequirement: string;
  readonly detail: string;
};

//endregion Types

//region Plan

/**
 Plans the tightened floor of every catalog entry.

 @param document - parsed catalog

 @param crates - each crate's default-catalog usage and lockfile versions

 @returns one result per default entry, then one SKIP per named-catalog entry

 @example
 ```ts
 planCargoTighten({ document, crates, });
 ```
 */
export function planCargoTighten(
  {
    document,
    crates,
  }: {
    readonly document: RustCatalogDocument;
    readonly crates: readonly CrateUsage[]
  },
): readonly CargoTightenResult[] {
  /**
   Results in catalog order.
   */
  const results: CargoTightenResult[] = [];
  for (const [name, entry,] of document.catalog) {
    if (entry.kind === 'git') {
      results.push({
        name,
        status: 'SKIP',
        oldRequirement: `rev ${entry.rev}`,
        newRequirement: '',
        detail: 'git pin',
      },);
      continue;
    }
    /**
     Package name the lockfiles record (the rename target, when renamed).
     */
    const lockedName = entry.package ?? name;
    /**
     Release versions locked by every consuming crate.
     */
    const locked: ParsedVersion[] = [];
    for (const crate of crates) {
      if (!crate.uses
        .has(name,))
        continue;
      for (const version of crate.lockVersions
        .get(lockedName,)
        ?? []) {
        /**
         Parsed locked version.
         */
        const parsed = parseVersion(version,);
        if (parsed.prerelease
          .length
          === 0)
          locked.push(parsed,);
      }
    }
    if (locked.length === 0) {
      results.push({
        name,
        status: 'MISS',
        oldRequirement: entry.version,
        newRequirement: '',
        detail: 'no consuming crate locks a release of it',
      },);
      continue;
    }
    /**
     Highest locked release.
     */
    const highest = locked.reduce(function pickHigher(
      best: ParsedVersion,
      candidate: ParsedVersion,
    ): ParsedVersion {
      return compareVersions({
        left: candidate,
        right: best,
      },) > 0 ? candidate : best;
    },);
    /**
     Current floor version.
     */
    const floor = parseVersion(floorVersion(entry.version,),);
    /**
     Highest locked version as text.
     */
    const highestText = `${String(highest.major,)}.${String(highest.minor,)}.${String(highest.patch,)}`;
    if (compareVersions({
      left: highest,
      right: floor,
    },) > 0)
      results.push({
        name,
        status: 'TIGHT',
        oldRequirement: entry.version,
        newRequirement: `>=${highestText}`,
        detail: '',
      },);
    else
      results.push({
        name,
        status: 'OK',
        oldRequirement: entry.version,
        newRequirement: entry.version,
        detail: '',
      },);
  }
  for (const [catalogName, catalog,] of document.catalogs) {
    for (const [name, entry,] of catalog) {
      results.push({
        name,
        status: 'SKIP',
        oldRequirement: entry.kind === 'git' ? `rev ${entry.rev}` : entry.version,
        newRequirement: '',
        detail: `named catalog ${catalogName} holds bounded requirements`,
      },);
    }
  }
  return results;
}

//endregion Plan

//region Rewrite

/**
 Rewrites tightened floors in the `[catalog]` table, replacing only each entry's version token.

 @param source - `rust-catalog.toml` text

 @param results - planned results; only `TIGHT` ones change the file

 @returns text with tightened floors and every other byte unchanged

 @example
 ```ts
 rewriteCatalogFloors({ source: '[catalog]\nanyhow = ">=1"\n', results, }); // '[catalog]\nanyhow = ">=1.0.104"\n'
 ```
 */
export function rewriteCatalogFloors(
  {
    source,
    results,
  }: {
    readonly source: string;
    readonly results: readonly CargoTightenResult[]
  },
): string {
  /**
   New requirement per tightened name.
   */
  const tightened = new Map<string, CargoTightenResult>();
  for (const result of results) {
    if (result.status === 'TIGHT')
      tightened.set(
        result.name,
        result,
      );
  }
  /**
   Table header the current line sits under, updated as headers pass.
   */
  const cursor = { section: '', };
  /**
   Lines after rewriting.
   */
  const lines: string[] = [];
  for (const line of source.split('\n',)) {
    /**
     Line without surrounding spaces.
     */
    const trimmed = line.trim();
    if (trimmed.startsWith('[',))
      cursor.section = trimmed;
    /**
     Key before ` =`, for entry lines.
     */
    const key = trimmed.slice(
      0,
      Math.max(
        trimmed.indexOf('=',),
        0,
      ),
    )
      .trim();
    /**
     Planned change for this entry, if any.
     */
    const change = cursor.section === '[catalog]' ? tightened.get(key,) : undefined;
    lines.push(change === undefined ? line : line.replace(
      JSON.stringify(change.oldRequirement,),
      JSON.stringify(change.newRequirement,),
    ),);
  }
  return lines.join('\n',);
}

//endregion Rewrite
