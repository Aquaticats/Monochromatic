/**
 List command; enumerates all managed VMs and their sync state.
 
 @module
 */

import type { ReadonlyDeep, } from 'type-fest';
import {
  access,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/module-logger/ts';

import {
  DATA_DIR,
  readConfig,
} from './config.ts';
import {
  CONFIG_FILENAME,
  type VmsyncConfig,
} from './types.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

/**
 Reads the data directory, returning an empty array if it does not exist.
 
 @param rl - {@link Logger} for status output
 
 @returns Directory entries, or empty array on missing directory
 */
async function safeReaddir(rl: Logger,): Promise<string[]> {
  try {
    return await readdir(DATA_DIR,);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.info('data directory does not exist, no VMs managed',);
    return [];
  }
}

/**
 Lists all managed VMs by scanning the vmsync data directory.
 Each subdirectory containing a `vmsync.jsonc` is treated as a managed VM.
 
 @returns Array of VM names
 
 @example
 ```ts
 const names = await listVms();
 // ['alpine', 'fedora-dev']
 ```
 */
export async function listVms(): Promise<readonly string[]> {
  /**
   Tagged logger so list-vm entries are scoped to `listVms` in the output.
   */
  const rl = tagged({
    tag: listVms.name,
    l,
  },);
  rl.info(`scanning ${DATA_DIR}`,);

  /**
   Entries in the data directory; empty if the directory does not exist.
   */
  const entries = await safeReaddir(rl,);

  /**
   Check all entries concurrently for valid config files.
   */
  const checks = await Promise.all(
    entries.map(
      async function checkEntry(entry,) {
        try {
          await access(
            join(
              DATA_DIR,
              entry,
              CONFIG_FILENAME,
            ),
          );
          return entry;
        }
        catch (error) {
          if (!(Error.isError(error,)))
            throw error;

          rl.debug(`skipping ${entry} (no config file)`,);
          return undefined;
        }
      },
    ),
  );

  /**
   VM names with valid config files.
   */
  const names = checks.filter(
    function isDefined(name,): name is string {
      return name !== undefined;
    },
  );

  rl.info(`found ${String(names.length,)} managed VMs`,);
  return names;
}

/**
 Prints a summary table of all managed VMs, sourced via {@link listVms}.
 
 @example
 ```ts
 await printVmList();
 // alpine       synced    kvm      4G  4cpu
 // fedora-dev   dirty     hyperv   8G  8cpu
 ```
 */
export async function printVmList(): Promise<void> {
  /**
   Tagged logger so print-list entries are scoped to `printVmList` in the output.
   */
  const rl = tagged({
    tag: printVmList.name,
    l,
  },);

  /**
   All managed VM names.
   */
  const names = await listVms();

  if (names.length
    === 0) {
    console.log('no managed VMs (use `vmsync import` to add one)',);
    return;
  }

  /**
   Column width for aligned name output.
   */
  const NAME_COL = 20;
  /**
   Column width for sync state.
   */
  const SYNC_COL = 10;
  /**
   Column width for hypervisor.
   */
  const HV_COL = 10;

  /**
   Load all configs concurrently.
   */
  const configs = await Promise.all(
    names.map(
      function loadConfig(name,) {
        return readConfig(name,);
      },
    ),
  );

  configs.forEach(
    function printRow(config: ReadonlyDeep<VmsyncConfig>,) {
      /**
       Sync state label.
       */
      const syncLabel = config.state
        .synced ? 'synced' : 'dirty';
      /**
       Last hypervisor label.
       */
      const hvLabel = config.state
        .lastBootHypervisor
        ?? 'never';

      console.log(
        `${config.name
          .padEnd(NAME_COL,)}${syncLabel.padEnd(SYNC_COL,)}${
          hvLabel.padEnd(HV_COL,)
        }${config.boot
          .memory}  ${String(config.boot
            .cpus,)}cpu`,
      );
    },
  );
}
