/**
 Sets aside a newly locked version the trust policy refuses and locks an older release instead,
 the way pnpm's picker sets aside a trust downgrade and picks the next candidate
 (pnpm `resolving/npm-resolver/src/trustChecks.ts`, `pickWithoutTrustDowngrade`).

 Cargo cannot exclude one version from a resolution, so each crate that locks the refused version
 is moved with `cargo update --package <name>@<version> --precise <candidate>`.
 A candidate is an older release in the same caret-compatibility class, published before the
 release-age cutoff, not yanked, not set aside already, and passing the trust policy against
 the full history (setting a version aside never removes the history another version is compared with).

 @module
 */

import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { RustCatalogSettings, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import type { RunCargo, } from './cargo.ts';
import {
  type TrustRefusal,
  trustRefusals,
} from './checks.ts';
import {
  type LockSnapshot,
  readLocks,
} from './crates.ts';
import {
  addedVersions,
  registryVersions,
  versionKey,
} from './lockfile-diff.ts';
import { l, } from './logger.ts';
import {
  trustProblems,
  type VersionRecord,
} from './policy.ts';
import type { Registry, } from './registry.ts';
import {
  caretCompatible,
  compareVersions,
  parseVersion,
} from './version-order.ts';

//region Limits

/**
 Set-aside rounds before the update gives up and reports what is left;
 a round can lock new transitive versions that need their own check.
 */
export const SET_ASIDE_ROUNDS = 5;

/**
 `cargo update --precise` attempts per refused version and crate, newest candidate first.
 */
export const PRECISE_ATTEMPTS = 10;

//endregion Limits

//region Candidates

/**
 One refused version moved to an older release.

 @example
 ```ts
 const moved: SetAside = { name: 'dep', version: '1.0.1', lockedInstead: '1.0.0', crateDirs: ['package/cli/a',], };
 ```
 */
export type SetAside = {
  readonly name: string;
  readonly version: string;
  readonly lockedInstead: string;
  readonly crateDirs: readonly string[];
};

/**
 Whether a release can replace the refused version.

 @param record - candidate release

 @param version - refused version

 @param prereleaseAllowed - whether the refused version is itself a prerelease

 @param cutoffMs - release-age cutoff in milliseconds, or 0 for none

 @param yanked - yanked versions of the crate

 @param rejected - `name@version` keys already set aside

 @param name - crate name

 @returns true when the release passes every filter except the trust policy

 @example
 ```ts
 isCandidate({ record, version: '1.0.1', prereleaseAllowed: false, cutoffMs: 0, yanked: new Set(), rejected: new Set(), name: 'dep', });
 ```
 */
function isCandidate(
  {
    record,
    version,
    prereleaseAllowed,
    cutoffMs,
    yanked,
    rejected,
    name,
  }: {
    readonly record: VersionRecord;
    readonly version: string;
    readonly prereleaseAllowed: boolean;
    readonly cutoffMs: number;
    readonly yanked: ReadonlySet<string>;
    readonly rejected: ReadonlySet<string>;
    readonly name: string;
  },
): boolean {
  if (yanked.has(record.num,) || rejected.has(`${name}@${record.num}`,))
    return false;
  if ((cutoffMs !== 0) && (Date.parse(record.createdAt,) > cutoffMs))
    return false;
  if ((!prereleaseAllowed) && (parseVersion(record.num,)
    .prerelease
    .length
    > 0))
    return false;
  return (compareVersions({
    left: record.num,
    right: version,
  },) < 0) && caretCompatible({
    version,
    candidate: record.num,
  },);
}

/**
 Older releases that can replace a refused version, newest first.

 @param refusal - refused version

 @param registry - registry lookups

 @param settings - catalog settings

 @param now - current time in milliseconds

 @param cutoffMs - release-age cutoff in milliseconds, or 0 for none

 @param rejected - `name@version` keys already set aside

 @returns candidate versions

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await setAsideCandidates({ refusal, registry, settings, now, cutoffMs: 0, rejected: new Set(), });
 ```
 */
export async function setAsideCandidates(
  {
    refusal,
    registry,
    settings,
    now,
    cutoffMs,
    rejected,
  }: {
    readonly refusal: TrustRefusal;
    readonly registry: Registry;
    readonly settings: RustCatalogSettings;
    readonly now: number;
    readonly cutoffMs: number;
    readonly rejected: ReadonlySet<string>;
  },
): Promise<readonly string[]> {
  /**
   Every release of the crate.
   */
  const history = await registry.history({
    name: refusal.name,
    versions: [refusal.version,],
  },);
  /**
   Yanked releases, which cargo should never be pointed at.
   */
  const yanked = await registry.yanked(refusal.name,);
  /**
   Whether the refused version is a prerelease, which lets prerelease candidates in.
   */
  const prereleaseAllowed = parseVersion(refusal.version,)
    .prerelease
    .length
    > 0;
  return history
    .filter(function passes(record: VersionRecord,): boolean {
      return isCandidate({
        record,
        version: refusal.version,
        prereleaseAllowed,
        cutoffMs,
        yanked,
        rejected,
        name: refusal.name,
      },) && (trustProblems({
        name: refusal.name,
        version: record.num,
        history,
        settings,
        now,
        ownerIds: new Set<string>(),
      },)
        .length
        === 0);
    },)
    .map(function toVersion(record: VersionRecord,): string {
      return record.num;
    },)
    .toSorted(function newestFirst(
      left: string,
      right: string,
    ): number {
      return compareVersions({
        left: right,
        right: left,
      },);
    },);
}

//endregion Candidates

//region Set aside

/**
 Lockfile text of a snapshot.

 @param snapshot - one crate's lockfile

 @returns its text

 @example
 ```ts
 [snapshot,].map(sourceOf);
 ```
 */
function sourceOf(snapshot: LockSnapshot,): string {
  return snapshot.source;
}

/**
 Crates whose lockfile holds a version.

 @param snapshots - current lockfiles

 @param key - `name@version`

 @returns their crate directories

 @example
 ```ts
 cratesLocking({ snapshots, key: 'dep@1.0.1', });
 ```
 */
function cratesLocking(
  {
    snapshots,
    key,
  }: {
    readonly snapshots: readonly LockSnapshot[];
    readonly key: string
  },
): readonly string[] {
  return snapshots
    .filter(function holds(snapshot: LockSnapshot,): boolean {
      return registryVersions([snapshot.source,],)
        .some(function matches(locked,): boolean {
          return versionKey(locked,) === key;
        },);
    },)
    .map(function toDir(snapshot: LockSnapshot,): string {
      return snapshot.crateDir;
    },);
}

/**
 Moves one crate's lockfile from a refused version to the newest candidate cargo accepts.

 @param root - repository root

 @param crateDir - crate to move

 @param refusal - refused version

 @param candidates - candidates, newest first

 @param runCargo - cargo runner

 @returns the version now locked, or an empty string when no candidate resolved

 @example
 ```ts
 await moveCrate({ root, crateDir, refusal, candidates: ['1.0.0',], runCargo, });
 ```
 */
async function moveCrate(
  {
    root,
    crateDir,
    refusal,
    candidates,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDir: string;
    readonly refusal: TrustRefusal;
    readonly candidates: readonly string[];
    readonly runCargo: RunCargo;
  },
): Promise<string> {
  /**
   Logger for attempts.
   */
  const log = tagged({
    tag: moveCrate.name,
    l,
  },);
  for (const candidate of candidates.slice(
    0,
    PRECISE_ATTEMPTS,
  )) {
    /**
     Attempt outcome.
     */
    // oxlint-disable-next-line no-await-in-loop -- each attempt edits the same lockfile, and the next runs only if this one failed.
    const outcome = await runCargo({
      cwd: join(
        root,
        crateDir,
      ),
      args: [
        'update',
        '--package',
        `${refusal.name}@${refusal.version}`,
        '--precise',
        candidate,
      ],
    },);
    if (outcome.ok)
      return candidate;
    log.debug(`${crateDir}: cargo refused ${refusal.name} ${candidate} (${outcome.output
      .trim()})`,);
  }
  return '';
}

/**
 Sets aside each refused version in every crate that locks it.

 @param root - repository root

 @param crateDirs - every crate with a lockfile

 @param refusals - versions the trust policy refused, not yet set aside

 @param registry - registry lookups

 @param settings - catalog settings

 @param now - current time in milliseconds

 @param cutoffMs - release-age cutoff in milliseconds, or 0 for none

 @param rejected - `name@version` keys set aside so far, including these refusals

 @param runCargo - cargo runner

 @returns the versions moved

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await setAsideRefusals({ root, crateDirs, refusals, registry, settings, now, cutoffMs: 0, rejected: new Set(), runCargo, });
 ```
 */
export async function setAsideRefusals(
  {
    root,
    crateDirs,
    refusals,
    registry,
    settings,
    now,
    cutoffMs,
    rejected,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDirs: readonly string[];
    readonly refusals: readonly TrustRefusal[];
    readonly registry: Registry;
    readonly settings: RustCatalogSettings;
    readonly now: number;
    readonly cutoffMs: number;
    readonly rejected: ReadonlySet<string>;
    readonly runCargo: RunCargo;
  },
): Promise<readonly SetAside[]> {
  /**
   Logger for the round.
   */
  const log = tagged({
    tag: setAsideRefusals.name,
    l,
  },);
  /**
   Versions moved this round.
   */
  const moved: SetAside[] = [];
  for (const refusal of refusals) {
    /**
     Key of the refused version.
     */
    const key = `${refusal.name}@${refusal.version}`;
    /**
     Candidates, newest first.
     */
    // oxlint-disable-next-line no-await-in-loop -- crates.io allows one API request per second, and cargo runs share one lock.
    const candidates = await setAsideCandidates({
      refusal,
      registry,
      settings,
      now,
      cutoffMs,
      rejected,
    },);
    if (candidates.length === 0) {
      log.info(`no older release can replace ${key}; its finding stands`,);
      continue;
    }
    /**
     Current lockfiles.
     */
    // oxlint-disable-next-line no-await-in-loop -- an earlier move in this round may have changed the lockfiles.
    const snapshots = await readLocks({
      root,
      crateDirs,
    },);
    /**
     Crates locking the refused version now.
     */
    const dirs = cratesLocking({
      snapshots,
      key,
    },);
    /**
     Crates moved, by the version each now locks.
     */
    const lockedBy = new Map<string, string[]>();
    for (const crateDir of dirs) {
      /**
       Version the crate now locks.
       */
      // oxlint-disable-next-line no-await-in-loop -- cargo runs share one package cache lock.
      const lockedInstead = await moveCrate({
        root,
        crateDir,
        refusal,
        candidates,
        runCargo,
      },);
      if (lockedInstead === '') {
        log.info(`${crateDir}: no candidate for ${key} resolved; its finding stands`,);
        continue;
      }
      lockedBy.set(
        lockedInstead,
        [
          ...(lockedBy.get(lockedInstead,) ?? []),
          crateDir,
        ],
      );
    }
    for (const [lockedInstead, crates,] of lockedBy) {
      log.info(`set aside ${key}; locked ${lockedInstead} in ${crates.join(', ',)}`,);
      moved.push({
        name: refusal.name,
        version: refusal.version,
        lockedInstead,
        crateDirs: crates,
      },);
    }
  }
  return moved;
}

/**
 Sets aside refused versions round by round until the trust check passes,
 nothing more can move, or the round limit is reached.

 Each round re-reads the lockfiles, since a move can lock different transitive versions.
 Whatever is still refused afterwards is left for the update's final checks to report.

 @param root - repository root

 @param crateDirs - every crate with a lockfile

 @param before - lockfile texts before the update

 @param registry - registry lookups

 @param settings - catalog settings

 @param now - current time in milliseconds

 @param cutoffMs - release-age cutoff in milliseconds, or 0 for none

 @param runCargo - cargo runner

 @returns one warning per version moved

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await setAsideUntilClean({ root, crateDirs, before, registry, settings, now, cutoffMs: 0, runCargo, });
 ```
 */
export async function setAsideUntilClean(
  {
    root,
    crateDirs,
    before,
    registry,
    settings,
    now,
    cutoffMs,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDirs: readonly string[];
    readonly before: readonly string[];
    readonly registry: Registry;
    readonly settings: RustCatalogSettings;
    readonly now: number;
    readonly cutoffMs: number;
    readonly runCargo: RunCargo;
  },
): Promise<readonly string[]> {
  /**
   Keys set aside so far.
   */
  const rejected = new Set<string>();
  /**
   Warnings for the report.
   */
  const warnings: string[] = [];
  for (let round = 0; round < SET_ASIDE_ROUNDS; round += 1) {
    /**
     Lockfiles as of this round.
     */
    // oxlint-disable-next-line no-await-in-loop -- each round reads the lockfiles the previous round rewrote.
    const current = await readLocks({
      root,
      crateDirs,
    },);
    /**
     Versions newly locked as of this round.
     */
    const added = addedVersions({
      before,
      after: current.map(sourceOf,),
    },);
    /**
     Refusals not set aside yet.
     */
    // oxlint-disable-next-line no-await-in-loop -- crates.io allows one API request per second.
    const pending = (await trustRefusals({
      versions: added,
      settings,
      registry,
      now,
    },)).filter(function isNew(refusal: TrustRefusal,): boolean {
      return !rejected.has(`${refusal.name}@${refusal.version}`,);
    },);
    if (pending.length === 0)
      break;
    for (const refusal of pending)
      rejected.add(`${refusal.name}@${refusal.version}`,);
    /**
     Versions moved this round.
     */
    // oxlint-disable-next-line no-await-in-loop -- the next round depends on the lockfiles this round rewrote.
    const moved = await setAsideRefusals({
      root,
      crateDirs,
      refusals: pending,
      registry,
      settings,
      now,
      cutoffMs,
      rejected,
      runCargo,
    },);
    for (const entry of moved)
      warnings.push(`Set aside ${entry.name}@${entry.version} (trust policy) and locked ${entry.lockedInstead} in ${entry.crateDirs
        .join(', ',)}.`,);
    if (moved.length === 0)
      break;
  }
  return warnings;
}

//endregion Set aside
