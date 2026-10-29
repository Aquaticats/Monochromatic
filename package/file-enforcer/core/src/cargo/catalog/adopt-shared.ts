/**
 Helpers shared by the adoption planners.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import type { UseSite, } from './types.ts';

/**
 Named-catalog selections: manifest path, then dependency name, to catalog name.

 @example
 ```ts
 const selections: Selections = new Map([['a/Cargo.toml', new Map([['windows', 'windows-legacy']])]]);
 ```
 */
export type Selections = ReadonlyMap<string, ReadonlyMap<string, string>>;

/**
 Dotted location of a site for diagnostics.

 @param site - use site

 @returns e.g. `target.cfg(unix).dependencies.libc`

 @example
 ```ts
 siteLocation(site);
 ```
 */
export function siteLocation(site: UseSite,): string {
  return [
    ...site.table,
    site.name,
  ].join('.',);
}

/**
 Whether two catalog entries are the same source.

 @param left - entry

 @param right - entry

 @returns whether kind, version or git pin, and rename agree

 @example
 ```ts
 sameEntry({ left, right, });
 ```
 */
export function sameEntry({
  left,
  right,
}: {
  readonly left: RustCatalogEntry;
  readonly right: RustCatalogEntry
},): boolean {
  if (left.package !== right.package)
    return false;
  if ((left.kind === 'git') && (right.kind === 'git'))
    return (left.git === right.git) && (left.rev === right.rev);
  if ((left.kind === 'registry') && (right.kind === 'registry'))
    return left.version === right.version;
  return false;
}

/**
 Remedy text for a literal that needs a named catalog.

 @param site - literal site

 @param requirement - the literal's requirement

 @returns instructions naming the catalog table and the crate's selection table

 @example
 ```ts
 namedCatalogRemedy({ site, requirement: '=1.18.1', });
 ```
 */
export function namedCatalogRemedy({
  site,
  requirement,
}: {
  readonly site: UseSite;
  readonly requirement: string
},): string {
  return [
    `add \`${site.name} = ${JSON.stringify(requirement,)}\` to a named catalog in rust-catalog.toml (for example \`[catalogs.pinned]\`)`,
    `and select it in ${site.manifestPath} with \`[package.metadata.rust-catalog.catalogs]\` \`${site.name} = "pinned"\``,
  ].join(' ',);
}
