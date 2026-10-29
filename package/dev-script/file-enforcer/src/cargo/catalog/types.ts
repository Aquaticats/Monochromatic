/**
 Types for syncing crate manifests with `rust-catalog.toml`.

 See `doc/decision/rust-dependency-catalog.md`.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/module-rust-catalog/ts';

//region Inputs

/**
 One crate manifest as read from disk.

 @example
 ```ts
 const manifest: ManifestFile = { path: 'package/forbidden-strings/cli/Cargo.toml', source: '[package]\n', };
 ```
 */
export type ManifestFile = {
  readonly path: string;
  readonly source: string;
};

/**
 Dependency tables a manifest may declare, as key paths.

 @example
 ```ts
 const table: DependencyTablePath = ['target', 'cfg(unix)', 'dependencies',];
 ```
 */
export type DependencyTablePath = readonly string[];

/**
 How a dependency entry at a use site refers to its source.

 - `catalog`: `workspace = true`, resolved through the catalog.
 - `path`: an internal crate by `path`.
 - `literal`: a registry version or git pin written in the manifest, to adopt into the catalog.

 @example
 ```ts
 const kind: UseSiteKind = 'literal';
 ```
 */
export type UseSiteKind = 'catalog' | 'literal' | 'path';

/**
 One dependency entry found in a manifest.

 @example
 ```ts
 const site: UseSite = {
   manifestPath: 'package/desktop-app/ide/Cargo.toml',
   table: ['dependencies',],
   name: 'anyhow',
   kind: 'literal',
   literal: { kind: 'registry', version: '1', },
   path: '',
   disablesDefaultFeatures: false,
 };
 ```
 */
export type UseSite = {
  readonly manifestPath: string;
  readonly table: DependencyTablePath;
  readonly name: string;
  readonly kind: UseSiteKind;
  /**
   Literal requirement or git pin; meaningful only for `literal` sites.
   */
  readonly literal: RustCatalogEntry;
  /**
   Relative path of an internal dependency; empty for other kinds.
   */
  readonly path: string;
  /**
   Whether the entry writes `default-features = false`.
   Cargo before 1.99 rejects that key beside `workspace = true` unless the
   workspace entry also disables default features, so the generated entry must carry it.
   */
  readonly disablesDefaultFeatures: boolean;
};

/**
 One entry of a crate's generated `[workspace.dependencies]` table.

 @example
 ```ts
 const generated: GeneratedEntry = { entry: { kind: 'registry', version: '>=0.3', }, defaultFeatures: false, };
 ```
 */
export type GeneratedEntry = {
  readonly entry: RustCatalogEntry;
  /**
   False when every use site in the crate disables default features.
   */
  readonly defaultFeatures: boolean;
};

//endregion Inputs

//region Results

/**
 One problem that stops the sync, naming the manifest or catalog location and the fix.

 @example
 ```ts
 const issue: SyncIssue = {
   file: 'package/desktop-app/file-manager/Cargo.toml',
   location: 'target.cfg(windows).dependencies.windows',
   problem: 'requires 0.58, which the catalog floor >=0.62 does not allow',
   remedy: 'add `[catalogs.windows-legacy] windows = "0.58"` and select it in this crate',
 };
 ```
 */
export type SyncIssue = {
  readonly file: string;
  readonly location: string;
  readonly problem: string;
  readonly remedy: string;
};

/**
 Output of one sync pass: new file texts, crates whose generated table changed, and warnings.

 @example
 ```ts
 const result: SyncResult = { catalog: '[catalog]\n', manifests: new Map(), affectedCrates: [], warnings: [], };
 ```
 */
export type SyncResult = {
  readonly catalog: string;
  readonly manifests: ReadonlyMap<string, string>;
  readonly affectedCrates: readonly string[];
  readonly warnings: readonly string[];
};

//endregion Results
