/**
 Lockfile install, the counterpart of `pnpm install`.

 This is the red stub that the test in `install.unit.test.ts` is written against.
 It inspects nothing and moves nothing, so the test fails until the real implementation lands.

 @module
 */

import type { RunCargo, } from './cargo.ts';
import { lockedCrateDirs, } from './crates.ts';
import type { LockedVersion, } from './lockfile-diff.ts';
import type { Registry, } from './registry.ts';

/**
 Outcome of an install run.

 @example
 ```ts
 const report: InstallReport = { crateDirs: [], stale: [], added: [], problems: [], warnings: [], written: false, };
 ```
 */
export type InstallReport = {
  readonly crateDirs: readonly string[];
  readonly stale: readonly string[];
  readonly added: readonly LockedVersion[];
  readonly problems: readonly string[];
  readonly warnings: readonly string[];
  readonly written: boolean;
};

/**
 Thrown by a frozen install when a lockfile needs an update.

 @example
 ```ts
 throw new StaleLockfilesError({ staleDirs: ['package/cli/a',], });
 ```
 */
export class StaleLockfilesError extends Error {
  readonly staleDirs: readonly string[];

  /**
   Builds the error.

   @param staleDirs - crate roots whose lockfile is stale
   */
  constructor({ staleDirs, }: { readonly staleDirs: readonly string[]; },) {
    super(staleDirs.join(', ',),);
    this.name = 'StaleLockfilesError';
    this.staleDirs = staleDirs;
  }
}

/**
 Stub: reports the crate roots and changes nothing.

 @param root - repository root

 @returns an empty report

 @example
 ```ts
 await installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: Date.now(), runCargo, registry, });
 ```
 */
export async function installRustLockfiles(
  {
    root,
  }: {
    readonly root: string;
    readonly frozenLockfile: boolean;
    readonly dryRun: boolean;
    readonly now: number;
    readonly runCargo: RunCargo;
    readonly registry: Registry;
  },
): Promise<InstallReport> {
  return {
    crateDirs: await lockedCrateDirs(root,),
    stale: [],
    added: [],
    problems: [],
    warnings: [],
    written: false,
  };
}
