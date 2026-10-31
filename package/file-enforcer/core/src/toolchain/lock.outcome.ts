/**
 `lock.outcome.ts`: decides what the sync and status do with a tool whose installed version differs from the lock.

 The rule (user ruling 2026-10-31: "No older repo-maintained tools should ever exist when newer is present."):
 a tool this repository publishes is never replaced by an older version. When the installed version is newer than this
 tree's lock, the install is kept and the tree runs the newer tool. Third-party tools keep converging to the lock.

 `doc/troubleshooting/toolchain-sync-older-lock-downgrades-shared-cargo-installs.md` records the incident behind it.
 */

import type { ToolPlanEntry, } from './types.ts';
import { versionMatches, } from './version.ts';
import { isNewerVersion, } from './version.order.ts';

//region Repository-published tools

/**
 Name prefix every crate this repository publishes carries (`monochromatic-lint`, `monochromatic-git-policy-cli`).

 The same signal `package/mpm-shadow/cli/src/constants.rs` (`OWN_CRATE_PREFIX`) uses to decide which crates in cargo's
 registry are this repository's own, as `doc/decision/mpm-toolchain-adoption.md` ("Pruning the repository's own
 crates") records. A crate published without the prefix is not recognized, so it keeps third-party behavior.
 */
export const REPOSITORY_CRATE_PREFIX = 'monochromatic-';

/**
 Whether a plan entry is a tool this repository publishes.

 Only the `cargo-binstall` route shares one machine-wide install (`$CARGO_HOME/bin`), so only that route can be
 downgraded for other trees; every other route installs under a version-keyed directory.

 @param entry - Plan entry to classify

 @returns `true` when the entry installs a crate whose name carries {@link REPOSITORY_CRATE_PREFIX}

 @example
 ```ts
 isRepositoryPublished(toolById('cargo:monochromatic-lint',),); // true
 ```
 */
export function isRepositoryPublished(entry: ToolPlanEntry,): boolean {
  return (entry.route === 'cargo-binstall')
    && (entry.packageId ?? entry.id).startsWith(REPOSITORY_CRATE_PREFIX,);
}

//endregion Repository-published tools

//region Outcome

/**
 What the sync and status do with one entry given its installed version: `absent` (install it), `at-lock` (nothing to do),
 `kept-newer` (a repository-published tool installed newer than the lock stays), `replace` (converge to the lock).
 */
export type LockOutcome = 'absent' | 'at-lock' | 'kept-newer' | 'replace';

/**
 Classifies the installed version of one entry against its locked version.

 @param entry - Plan entry; supplies the locked version and the repository-published signal

 @param observed - Installed version text, or `undefined` when nothing is installed

 @returns The outcome the sync and status act on

 @example
 ```ts
 lockOutcome({ entry: toolById('cargo:monochromatic-lint',), observed: '9.9.9', },); // 'kept-newer'
 ```
 */
export function lockOutcome(
  {
    entry,
    observed,
  }: {
    readonly entry: ToolPlanEntry;
    readonly observed: string | undefined;
  },
): LockOutcome {
  if (observed === undefined)
    return 'absent';
  if (versionMatches({
    observed,
    locked: entry.version,
  },))
    return 'at-lock';
  return (isRepositoryPublished(entry,) && isNewerVersion({
    candidate: observed,
    than: entry.version,
  },))
    ? 'kept-newer'
    : 'replace';
}

/**
 The warning for a repository-published tool kept at a version newer than this tree's lock.

 It is logged on every sync and status run that meets the situation, because the tree is running a tool its lock does
 not pin.

 @param id - Plan entry id the warning names

 @param observed - Installed version, which is the one that runs

 @param locked - Version this tree's lock pins

 @returns One warning line naming the tool, both versions, why the newer one is kept, and how to upgrade the lock
 */
export function keptNewerWarning(
  {
    id,
    observed,
    locked,
  }: {
    readonly id: string;
    readonly observed: string;
    readonly locked: string;
  },
): string {
  return `${id}: this tree's lock pins ${locked} but ${observed} is installed, so this tree runs ${observed}; `
    + 'older repository tools are never kept when a newer one is installed, and installing '
    + `${locked} would downgrade it for every tree that shares this install. `
    + 'Upgrade this tree\'s lock with `mise run toolchain:update` or by merging main.';
}

//endregion Outcome
