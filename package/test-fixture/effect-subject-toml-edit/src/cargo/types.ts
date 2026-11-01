/**
 Types for the canonical `Cargo.toml` enforcement engine.
 
 A {@link CargoManifestSpec} maps one manifest path to a {@link CargoManifestPlan}:
 a list of guarded keyed value enforcements plus a list of whole-block
 insertions for tables that must be materialized where absent.
 
 @module
 */

import type { Path, } from '../types.ts';

/**
 Deep-readonly TOML value that a canonical enforcement writes.
 
 Canonical values are constant configuration data (never mutated after
 authoring), so an honestly deep-readonly type is the correct shape: it carries
 no mutable ownership, so the readonly-parameter analysis needs neither a
 `@mutates` contract nor a `ForeignBorrowed` marker when the value flows into
 `tomlSet` (whose input is `unknown`, accepting readonly data).
 
 @example
 ```ts
 const value: CanonicalTomlValue = { version: '4', features: ['derive'] };
 ```
 */
export type CanonicalTomlValue =
  | string
    | number
    | boolean
    | readonly CanonicalTomlValue[]
    | { readonly [key: string]: CanonicalTomlValue; };

/**
 One guarded keyed enforcement.
 
 `guardPath` gates the edit: enforcement runs only when the guard resolves in
 the manifest, which keeps present-seeded properties (a dependency, a license)
 from being conjured into crates that never declared them, and lets a key be
 created inside an existing table without landing at top level.
 
 @example
 ```ts
 const editionEnforcement: CargoEnforcement = {
   guardPath: ['package',],
   path: ['package', 'edition',],
   value: '2024',
 };
 ```
 */
export type CargoEnforcement = {
  readonly guardPath: Path;
  readonly path: Path;
  readonly value: CanonicalTomlValue;
};

/**
 One whole-block insertion.
 
 `absentPath` gates insertion: the block text is appended only when the path
 does not already resolve, so a crate that already carries the table keeps its
 bespoke comments untouched.
 
 @example
 ```ts
 const workspaceBlock: CargoBlockInsertion = {
   absentPath: ['workspace',],
   text: '[workspace]\n',
 };
 ```
 */
export type CargoBlockInsertion = {
  readonly absentPath: Path;
  readonly text: string;
};

/**
 Plan for one manifest: guarded enforcements and absent-block insertions.
 
 @example
 ```ts
 const plan: CargoManifestPlan = { enforcements: [], blocks: [], };
 ```
 */
export type CargoManifestPlan = {
  readonly enforcements: readonly CargoEnforcement[];
  readonly blocks: readonly CargoBlockInsertion[];
};

/**
 Builds a {@link CargoManifestPlan} for one manifest path.
 
 @param manifestPath - Repo-relative path used to derive per-crate values (profile preset, homepage)
 
 @returns Plan describing every owned edit for that manifest
 
 @example
 ```ts
 const spec: CargoManifestSpec = ({ manifestPath, }) => ({ enforcements: [], blocks: [], });
 ```
 */
export type CargoManifestSpec = (
  args: { readonly manifestPath: string; },
) => CargoManifestPlan;
