/**
 Lock-only reconciliation: makes every first-party `Cargo.lock` satisfy its manifests again after a manifest change.

 This is the red stub that the test in `reconcile.unit.test.ts` is written against.
 It inspects nothing and moves nothing, so the test fails until the real implementation lands.

 @module
 */

import type { RunCargo, } from './cargo.ts';
import { lockedCrateDirs, } from './crates.ts';
import type { LockedVersion, } from './lockfile-diff.ts';
import type { Registry, } from './registry.ts';

//region Types

/**
 Outcome of a reconcile run.

 @example
 ```ts
 const report: ReconcileReport = { crateDirs: [], reconciled: [], added: [], problems: [], warnings: [], written: false, };
 ```
 */
export type ReconcileReport = {
  /**
   Every crate root that has a `Cargo.lock`.
   */
  readonly crateDirs: readonly string[];
  /**
   Crate roots whose lockfile did not satisfy its manifests and was regenerated.
   */
  readonly reconciled: readonly string[];
  /**
   Registry versions the reconcile newly locked (kept or restored).
   */
  readonly added: readonly LockedVersion[];
  /**
   Findings that refused the reconcile.
   */
  readonly problems: readonly string[];
  readonly warnings: readonly string[];
  /**
   Whether the regenerated lockfiles were kept.
   */
  readonly written: boolean;
};

//endregion Types

//region Reconcile

/**
 Stub: reports the crate roots and changes nothing.

 @param root - repository root

 @returns an empty report

 @example
 ```ts
 await reconcileRustLockfiles({ root, dryRun: false, now: Date.now(), runCargo, registry, });
 ```
 */
export async function reconcileRustLockfiles(
  {
    root,
  }: {
    readonly root: string;
    readonly dryRun: boolean;
    readonly now: number;
    readonly runCargo: RunCargo;
    readonly registry: Registry;
  },
): Promise<ReconcileReport> {
  return {
    crateDirs: await lockedCrateDirs(root,),
    reconciled: [],
    added: [],
    problems: [],
    warnings: [],
    written: false,
  };
}

//endregion Reconcile
