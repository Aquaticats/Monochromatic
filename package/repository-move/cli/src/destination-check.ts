/**
 The check that comes before any send: `btrfs receive` finds a clone source or a parent by the received UUID of a
 subvolume on the destination filesystem, so two subvolumes with one received UUID make it pick either (measured on the
 fixture: sharing was kept, but against the older copy), and a subvolume in the received folder that no record knows is
 a partial receive that a rerun would trip over.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  type Records,
  pathExists,
} from './completion-records.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';

/**
 Label before a row's received UUID in `btrfs subvolume list -R`.
 */
const RECEIVED_LABEL = ' received_uuid ';

/**
 Label before a row's path.
 */
const PATH_LABEL = ' path ';

/**
 The received UUID of a subvolume that was not received.
 */
const NOT_RECEIVED = '-';

/**
 One listed subvolume that was received.
 */
type ReceivedRow = {
  readonly uuid: string;
  readonly path: string;
};

/**
 Groups the listed subvolumes by received UUID and names every UUID that more than one subvolume holds.

 @param listing - the output of `btrfs subvolume list -R`

 @returns one sentence per duplicated received UUID, naming every path

 @example
 ```ts
 import { duplicateReceivedUuids, } from './destination-check.ts';

 const problems = duplicateReceivedUuids('ID 5 gen 1 top level 5 received_uuid r1 path a\nID 6 gen 1 top level 5 received_uuid r1 path b',);
 ```
 */
export function duplicateReceivedUuids(listing: string,): readonly string[] {
  /**
   The paths by received UUID.
   */
  const byUuid = Map.groupBy(
    listing.split('\n',)
      .filter(function isReceived(row,) {
        return row.includes(RECEIVED_LABEL,) && row.includes(PATH_LABEL,);
      },)
      .map(function parse(row,): ReceivedRow {
        /**
         The text after the received label.
         */
        const afterLabel = row.slice(row.indexOf(RECEIVED_LABEL,) + RECEIVED_LABEL.length,);
        return {
          uuid: afterLabel.slice(
            0,
            afterLabel.indexOf(' ',),
          ),
          path: row.slice(row.indexOf(PATH_LABEL,) + PATH_LABEL.length,),
        };
      },)
      .filter(function wasReceived(entry: Readonly<ReceivedRow>,) {
        return entry.uuid !== NOT_RECEIVED;
      },),
    function uuidOf(entry: Readonly<ReceivedRow>,) {
      return entry.uuid;
    },
  );
  return [...byUuid.entries(),]
    .filter(function isDuplicated([, entries,]: readonly [
      string,
      readonly ReceivedRow[]
    ],) {
      return entries.length > 1;
    },)
    .map(function describe([uuid, entries,]: readonly [
      string,
      readonly ReceivedRow[]
    ],) {
      return `the destination holds more than one subvolume with received UUID ${uuid}: ${
        entries.map(function pathOf(entry: Readonly<ReceivedRow>,) {
          return entry.path;
        },)
          .join(', ',)
      }; btrfs receive resolves a clone source by received UUID and would pick either. Nothing was sent. Inspect them and `
        + 'delete the extra one yourself with \'btrfs subvolume delete <path>\'; nothing was deleted by this script.';
    },);
}

/**
 Names every entry of the received folder that has no completion record.

 @param options - the options

 @param records - the run's record store

 @returns the entries as paths, sorted

 @example
 ```ts
 import { unrecordedReceives, } from './destination-check.ts';

 const strays = await unrecordedReceives({ options, records, },);
 ```
 */
export async function unrecordedReceives(
  {
    options,
    records,
  }: {
    readonly options: MoveOptions;
    readonly records: Records;
  },
): Promise<readonly string[]> {
  if (!await pathExists(options.receivedFolder,)) {
    return [];
  }
  /**
   The entries of the received folder.
   */
  const entries = (await readdir(options.receivedFolder,)).toSorted();
  /**
   Whether each entry has a record of a finished receive.
   */
  const recorded = await Promise.all(entries.map(async function check(name,) {
    return await records.has({
      name,
      step: 'received',
    },);
  },),);
  return entries.filter(function isStray(
    _name,
    index,
  ) {
    return recorded[index] === false;
  },)
    .map(function pathOf(name,) {
      return join(
        options.receivedFolder,
        name,
      );
    },);
}

/**
 Refuses a destination that a send could not resolve a clone source on, or that holds a partial receive.

 @param options - the options

 @param runner - the runner

 @param records - the run's record store

 @throws MoveStopError naming every path and the delete the owner runs, when a received UUID is held twice or a stray subvolume is in the received folder

 @example
 ```ts
 import { checkDestination, } from './destination-check.ts';

 await checkDestination({ options, runner, records, },);
 ```
 */
export async function checkDestination(
  {
    options,
    runner,
    records,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
    readonly records: Records;
  },
): Promise<void> {
  /**
   The entries of the received folder that no record knows.
   */
  const strays = await unrecordedReceives({
    options,
    records,
  },);
  if (strays.length > 0) {
    throw new MoveStopError(
      `${strays.join(' and ',)} exists without a completion record: a receive was interrupted. Inspect it, delete it with `
        + `${strays.map(function deleteCommand(path,) {
          return `'btrfs subvolume delete ${path}'`;
        },)
          .join(' and ',)} yourself, and run again. Nothing was deleted by this script.`,
    );
  }
  /**
   The listing of every subvolume of the destination filesystem with its received UUID.
   */
  const listing = await runner.exec({
    command: 'btrfs',
    args: [
      'subvolume',
      'list',
      '-R',
      options.receivedFolder,
    ],
  },);
  /**
   The received UUIDs that more than one subvolume holds.
   */
  const duplicates = duplicateReceivedUuids(listing,);
  if (duplicates.length > 0) {
    throw new MoveStopError(duplicates.join(' ',),);
  }
}
