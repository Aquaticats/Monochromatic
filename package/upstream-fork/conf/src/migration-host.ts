/**
 Store access surface the migration runner mutates through.
 
 Keeping the runner off the concrete store object lets migration tests
 drive the step logic with a stub host while production passes the real
 store's accessors.
 
 @module
 */

import type { Conf, } from './conf.ts';

//region Types

/**
 Everything the migration runner needs from the store it operates on.
 
 @example
 ```ts
 const host: MigrationHost<Record<string, unknown>> = {
   store: config,
   configFileExists: function configFileExists(): boolean { return true; },
   readRawStore: function readRawStore(): Record<string, unknown> { return {}; },
   readUserStore: function readUserStore(): Record<string, unknown> { return {}; },
   writeUserStore: function writeUserStore(): void {},
   writeStoreWithoutEvents: function writeStoreWithoutEvents(): void {},
   recordVersion: function recordVersion(): void {},
 };
 ```
 */
export type MigrationHost<T extends Record<string, unknown>> = {
  /**
   Store instance migration handlers receive and mutate.
   */
  readonly store: Conf<T>;
  /**
   Whether the config file exists before this pass touches it.
   */
  readonly configFileExists: () => boolean;
  /**
   File contents including the reserved bookkeeping keys.
   */
  readonly readRawStore: () => Record<string, unknown>;
  /**
   User-visible store contents with reserved keys stripped.
   */
  readonly readUserStore: () => T;
  /**
   Replaces the user-visible store contents.
   */
  readonly writeUserStore: (store: T,) => void;
  /**
   Writes raw file contents without validating or dispatching change
   events,
   used for the pre-step defaults merge.
   */
  readonly writeStoreWithoutEvents: (store: Record<string, unknown>,) => void;
  /**
   Records a migrated version under the reserved bookkeeping key.
   */
  readonly recordVersion: (version: string,) => void;
};

//endregion Types
