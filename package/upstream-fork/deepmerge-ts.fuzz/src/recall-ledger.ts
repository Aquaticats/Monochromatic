/**
 Shapes of the historical-recall ledger: every bug upstream fixed in a
 release, with what the recall run needs to re-create it and to confirm it
 reproduces before asking whether the sidecar detects it.

 Method and results: `doc/audit/deepmerge-ts-recall-2026-09-24.md`.

 @module
 */

import type { DeepmergeTarget, } from './target.ts';

/**
 One exact-text replacement in an upstream source file, relative to `src/`.
 */
export type SourceEdit = {
  readonly file: string;
  readonly find: string;
  readonly replace: string;
  readonly count: number;
};

/**
 Where a runtime bug's buggy source comes from: the v8.0.2 tree with the
 historical fault re-applied, or the real parent tree of the fix commit.
 */
export type RuntimeSource = {
  readonly kind: 'transplant';
  readonly edits: readonly SourceEdit[];
} | {
  readonly kind: 'parent';
  readonly buggyTree: string;
  readonly fixedTree: string;
  readonly shimFastUnsafe: boolean;
};

/**
 Fields every ledger row carries.
 */
type LedgerRowBase = {
  readonly id: string;
  readonly commit: string;
  readonly version: string;
  readonly issue: string;
  readonly broke: string;
  readonly regressionTest: string;
};

/**
 A runtime bug the recall run re-creates as a bundle.

 `reproduces` is the positive control: ported from upstream's regression test
 or issue repro, true on the buggy bundle and false on the fixed one.
 */
export type RuntimeBug = LedgerRowBase & {
  readonly source: RuntimeSource;
  readonly reproduces: (library: DeepmergeTarget,) => boolean;
};

/**
 A result-type bug checked by type-checking the sidecar against the npm
 release before the fix and the release with it.

 `control` is upstream's regression assertion as a standalone module that
 must fail to type-check against `buggy` and pass against `fixed`.
 */
export type TypeBug = LedgerRowBase & {
  readonly buggy: string;
  readonly fixed: string;
  readonly control: string;
};

/**
 A packaging, tooling, or docs bug: the sidecar never loads the surface, so
 the row records which surface and why the sidecar cannot see it.
 */
export type SurfaceBug = LedgerRowBase & {
  readonly surface: 'docs' | 'engines' | 'install' | 'packaging' | 'repo-internal' | 'type-checker';
  readonly buggy: string;
  readonly fixed: string;
};
