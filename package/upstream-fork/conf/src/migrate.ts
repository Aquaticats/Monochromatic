/**
 Migration bookkeeping and step execution.
 
 Mirrors upstream `conf` 15.1.0's migration semantics: versions may be
 concrete semver versions or ranges,
 steps run in ascending order,
 a failed step restores the store to its pre-step snapshot,
 and the recorded version lives under the reserved `__internal__` key.
 
 @module
 */

import { getProperty, } from 'dot-prop';
import semver from 'semver';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  MigrationFailedError,
  MissingProjectVersionError,
} from './errors.ts';
import { MIGRATION_KEY, } from './internal-key.ts';
import {
  createPlainObject,
  isStoreContentEqual,
} from './store-access.ts';
import type { MigrationHost, } from './migration-host.ts';
import type {
  BeforeEachMigrationCallback,
  Migrations,
} from './types.ts';

//region Constants

/**
 Version recorded when the file holds nothing comparable.
 
 @example
 ```ts
 UNKNOWN_VERSION; // '0.0.0'
 ```
 */
const UNKNOWN_VERSION = '0.0.0';

//endregion Constants

//region Version predicates

/**
 Reports whether a recorded or configured version is a semver range rather
 than a concrete version.
 
 @param version - Version or range string as written by the caller or file.
 
 @returns `true` when the string is not a clean concrete version.
 
 @example
 ```ts
 isVersionInRangeFormat('>=2.0.0'); // => true
 isVersionInRangeFormat('1.0.0'); // => false
 ```
 */
export function isVersionInRangeFormat(version: string,): boolean {
  return semver.clean(version,) === null;
}

/**
 Decides whether one migration entry applies in a pass.
 
 Ranges apply when the target version satisfies them and a previous
 migration has not already covered them;
 concrete versions apply when they sit after the last migrated version and
 at or below the target.
 
 @param candidateVersion - Migration key: a concrete version or a range.
 
 @param previousMigratedVersion - Last version recorded as migrated.
 
 @param versionToMigrate - Project version this pass targets.
 
 @returns `true` when the step should run.
 
 @example
 ```ts
 shouldPerformMigration({
   candidateVersion: '1.0.0',
   previousMigratedVersion: '0.9.0',
   versionToMigrate: '1.0.0',
 }); // => true
 ```
 */
export function shouldPerformMigration({
  candidateVersion,
  previousMigratedVersion,
  versionToMigrate,
}: {
  readonly candidateVersion: string;
  readonly previousMigratedVersion: string;
  readonly versionToMigrate: string;
},): boolean {
  if (isVersionInRangeFormat(candidateVersion,)) {
    if ((previousMigratedVersion !== UNKNOWN_VERSION)
      && semver.satisfies(
        previousMigratedVersion,
        candidateVersion,
      ))
      return false;
    return semver.satisfies(
      versionToMigrate,
      candidateVersion,
    );
  }
  if (semver.lte(
    candidateVersion,
    previousMigratedVersion,
  ))
    return false;
  if (semver.gt(
    candidateVersion,
    versionToMigrate,
  ))
    return false;
  return true;
}

//endregion Version predicates

//region Step execution

/**
 Runs every applicable migration step against the store,
 restoring the pre-step snapshot when a step throws.
 
 @param host - Store access surface this runner mutates through.
 
 @param migrations - Version-to-handler map from the `migrations` option.
 
 @param projectVersion - Target version for the whole pass.
 
 @param beforeEachMigration - Optional per-step hook.
 
 @throws MigrationFailedError When a step throws; the store is restored
 first.
 
 @example
 ```ts
 runMigrationSteps({
   host,
   migrations: { '1.0.0': function toV1(store): void { store.delete('debugPhase'); }, },
   projectVersion: '1.0.0',
 });
 ```
 */
export function runMigrationSteps<T extends Record<string, unknown>>({
  host,
  migrations,
  projectVersion,
  beforeEachMigration,
}: {
  readonly host: MigrationHost<T>;
  readonly migrations: Migrations<T>;
  readonly projectVersion: string;
  readonly beforeEachMigration?: BeforeEachMigrationCallback<T>;
},): void {
  /**
   Version recorded in the file,
   defaulting to the unknown sentinel.
   */
  const storedVersion = getProperty(
    host.readRawStore(),
    MIGRATION_KEY,
    UNKNOWN_VERSION,
  );
  /**
   Loop state: the version each step migrates from and the snapshot a failed
   step restores.
   */
  const state = {
    previousMigratedVersion: isVersionInRangeFormat(storedVersion,) ? UNKNOWN_VERSION : storedVersion,
    backup: structuredClone(host.readUserStore(),),
  };
  /**
   Migration keys that apply in this pass,
   in their configured order.
   */
  const newerVersions = Object.keys(migrations,)
    .filter(
    function isCandidate(candidateVersion: string,): boolean {
      return shouldPerformMigration({
        candidateVersion,
        previousMigratedVersion: state.previousMigratedVersion,
        versionToMigrate: projectVersion,
      },);
    },
    );
  for (const version of newerVersions) {
    try {
      beforeEachMigration?.({
        store: host.store,
        context: {
          fromVersion: state.previousMigratedVersion,
          toVersion: version,
          finalVersion: projectVersion,
          versions: newerVersions,
        },
      },);
      /**
       Migration handler for this version,
       absent for range keys that only order the pass.
       */
      const migration = migrations[version];
      migration?.(host.store,);
      if (!isVersionInRangeFormat(version,))
        host.recordVersion(version,);
      state.previousMigratedVersion = version;
      state.backup = structuredClone(host.readUserStore(),);
    }
    catch (error) {
      host.writeUserStore(state.backup,);
      throw new MigrationFailedError({
        reason: caughtValueText(error,),
      },);
    }
  }
  if (isVersionInRangeFormat(state.previousMigratedVersion,)
    || (!semver.eq(
      state.previousMigratedVersion,
      projectVersion,
    )))
    host.recordVersion(projectVersion,);
}

//endregion Step execution

//region Pass orchestration

/**
 Runs the whole migration pass for one store:
 merge defaults into an existing file,
 start a brand-new store at the current version without running steps,
 then run and record the steps.
 
 @param host - Store access surface this runner mutates through.
 
 @param migrations - Version-to-handler map from the `migrations` option.
 
 @param projectVersion - Target version for the whole pass.
 
 @param defaults - Default values merged into the store before steps run.
 
 @param beforeEachMigration - Optional per-step hook.
 
 @throws MissingProjectVersionError When no target version is provided.
 
 @throws MigrationFailedError When a step throws.
 
 @example
 ```ts
 applyMigrations({
   host,
   migrations,
   projectVersion: '2.0.0',
   defaults: { theme: 'light', },
 });
 ```
 */
export function applyMigrations<T extends Record<string, unknown>>({
  host,
  migrations,
  projectVersion,
  defaults,
  beforeEachMigration,
}: {
  readonly host: MigrationHost<T>;
  readonly migrations: Migrations<T>;
  readonly projectVersion: string;
  readonly defaults?: Readonly<T>;
  readonly beforeEachMigration?: BeforeEachMigrationCallback<T>;
},): void {
  if (projectVersion === '')
    throw new MissingProjectVersionError();
  /**
   Whether the config file pre-existed this store;
   a store that does not exist yet has nothing to migrate.
   */
  const isNewStore = !host.configFileExists();
  /**
   File contents before defaults merge.
   */
  const fileStore = host.readRawStore();
  /**
   File contents with defaults filled in for missing keys.
   */
  const storeWithDefaults = Object.assign(
    createPlainObject(),
    defaults ?? {},
    fileStore,
  );
  if (!isStoreContentEqual({
    left: fileStore,
    right: storeWithDefaults,
  },))
    host.writeStoreWithoutEvents(storeWithDefaults,);
  if (isNewStore) {
    host.recordVersion(projectVersion,);
    return;
  }
  runMigrationSteps({
    host,
    migrations,
    projectVersion,
    ...(beforeEachMigration === undefined ? {} : { beforeEachMigration }),
  },);
}

//endregion Pass orchestration
