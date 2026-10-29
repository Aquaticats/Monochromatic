/**
 Types for the parsed `rust-catalog.toml` document.

 The shapes follow pnpm catalogs: one default catalog (`[catalog]`) and any
 number of named catalogs (`[catalogs.<name>]`), plus pnpm's supply-chain
 settings under pnpm's own key names. See
 `doc/decision/rust-dependency-catalog.md`.

 @module
 */

//region Entries

/**
 Registry dependency entry: a version requirement and an optional Cargo `package` rename target.

 Default-catalog entries are `>=` floors; named-catalog entries are bounded
 (caret, tilde, or exact). Features, optionality, and default features belong
 to the crate's use site, never to the catalog, because Cargo's workspace
 inheritance only adds features on top of the inherited entry.

 @example
 ```ts
 const entry: RustCatalogRegistryEntry = { kind: 'registry', version: '>=1.0.104', };
 ```
 */
export type RustCatalogRegistryEntry = {
  readonly kind: 'registry';
  readonly version: string;
  readonly package?: string;
};

/**
 Git dependency entry pinned to one revision.

 @example
 ```ts
 const entry: RustCatalogGitEntry = {
   kind: 'git',
   git: 'https://github.com/helix-editor/helix',
   rev: 'ba40e547426b0f9896c8bdc699a4ab11f2b37dbc',
 };
 ```
 */
export type RustCatalogGitEntry = {
  readonly kind: 'git';
  readonly git: string;
  readonly rev: string;
  readonly package?: string;
};

/**
 One catalog entry, registry or git.

 @example
 ```ts
 const entries: readonly RustCatalogEntry[] = [{ kind: 'registry', version: '>=1', }];
 ```
 */
export type RustCatalogEntry = RustCatalogRegistryEntry | RustCatalogGitEntry;

/**
 Entries of one catalog keyed by dependency name, in source order.

 @example
 ```ts
 const catalog: RustCatalogMap = new Map([['anyhow', { kind: 'registry', version: '>=1', }]]);
 ```
 */
export type RustCatalogMap = ReadonlyMap<string, RustCatalogEntry>;

//endregion Entries

//region Settings

/**
 Trust policy modes, as pnpm's `trustPolicy` setting names them.

 @example
 ```ts
 const mode: RustCatalogTrustPolicy = 'no-downgrade';
 ```
 */
export type RustCatalogTrustPolicy = 'no-downgrade' | 'off';

/**
 Supply-chain settings with pnpm's names and pnpm's documented defaults.

 @example
 ```ts
 const settings: RustCatalogSettings = {
   minimumReleaseAge: 1440,
   minimumReleaseAgeExclude: [],
   minimumReleaseAgeExcludePrune: false,
   minimumReleaseAgeIgnoreMissingTime: true,
   minimumReleaseAgeStrict: true,
   trustPolicy: 'no-downgrade',
   trustPolicyExclude: [],
   trustPolicyExcludePrune: false,
 };
 ```
 */
export type RustCatalogSettings = {
  readonly minimumReleaseAge: number;
  readonly minimumReleaseAgeExclude: readonly string[];
  readonly minimumReleaseAgeExcludePrune: boolean;
  readonly minimumReleaseAgeIgnoreMissingTime: boolean;
  readonly minimumReleaseAgeStrict: boolean;
  readonly trustPolicy: RustCatalogTrustPolicy;
  readonly trustPolicyExclude: readonly string[];
  readonly trustPolicyExcludePrune: boolean;
  readonly trustPolicyIgnoreAfter?: number;
};

//endregion Settings

//region Document

/**
 Parsed and validated `rust-catalog.toml`.

 @example
 ```ts
 const document = parseRustCatalog({ source, sourcePath: 'rust-catalog.toml', });
 console.info(document.catalog.size);
 ```
 */
export type RustCatalogDocument = {
  readonly catalog: RustCatalogMap;
  readonly catalogs: ReadonlyMap<string, RustCatalogMap>;
  readonly settings: RustCatalogSettings;
};

/**
 One validation problem, naming where it is and how to fix it.

 @example
 ```ts
 const issue: RustCatalogIssue = {
   location: 'catalog.tokio',
   problem: 'declares `features`',
   remedy: 'move `features` to the crate that uses tokio',
 };
 ```
 */
export type RustCatalogIssue = {
  readonly location: string;
  readonly problem: string;
  readonly remedy: string;
};

//endregion Document
