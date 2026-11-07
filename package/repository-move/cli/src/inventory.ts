/**
 The subvolume inventory: the Git registry reconciled with the filesystem's own list, with each subvolume's UUID.

 `btrfs subvolume list -u -q <filesystem>` (root only) prints one row per subvolume ending in
 `parent_uuid <uuid or -> received_uuid <uuid or -> uuid <uuid> path <path from the top level>`; the copy keys every record by that UUID and path.

 @module
 */

import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  MISSING_PATH_CODE,
  pathExists,
} from './completion-records.ts';
import { canonicalOptions, } from './canonical-path.ts';
import { SUBVOLUME_ROOT_INODE, } from './constants.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';
import { isSubvolume, } from './subvolume-root-check.ts';
import {
  describeDisagreement,
  listRegisteredWorktreeSpellings,
} from './registered-worktrees.ts';

/**
 A subvolume with the UUID the inventory gave it; the dash when no inventory was given (print mode only).
 */
export type Subvolume = {
  readonly path: string;
  readonly uuid: string;

  /**
   The UUID of the subvolume this one was snapshotted from: the dash when it has none (a plain `subvolume create`), and
   the question mark when the inventory was taken without `-q`. Information only: no send starts from a related
   subvolume (see `send-plan.ts`).
   */
  readonly parentUuid: string;
};

/**
 What a subvolume's parent UUID is when the inventory was taken without the parent column.
 */
export const PARENT_UNKNOWN = '?';

/**
 The UUID of a subvolume whose inventory is unavailable.
 */
const UNKNOWN_UUID = '-';

/**
 Separator before the UUID in an inventory row.
 */
const UUID_LABEL = ' uuid ';

/**
 Separator before the path in an inventory row.
 */
const PATH_LABEL = ' path ';

/**
 One row of `btrfs subvolume list -u` as printed: its ID, the ID of its parent subvolume (the `top level` column), its
 UUID, and the path column.

 The path column is relative to the subvolume named by `top level` on the real filesystem (the checkout's row reads
 `top level 257 ... path user/Monochromatic` where 257 is `home`, measured by the coordinator with root on
 2026-11-07), and a row nested in a nested subvolume may print a path that already includes its parent's path;
 `resolveFullPaths` accepts both.
 */
export type ListedRow = {
  readonly id: string;
  readonly topLevel: string;
  readonly uuid: string;
  readonly parentUuid: string;
  readonly path: string;
};

/**
 Label before the ID in an inventory row.
 */
const ID_LABEL = 'ID ';

/**
 Label before the UUID of the subvolume a row was snapshotted from; `btrfs subvolume list -q` prints it.
 */
const PARENT_UUID_LABEL = ' parent_uuid ';

/**
 Label before the parent ID in an inventory row.
 */
const TOP_LEVEL_LABEL = ' top level ';

/**
 Reads the parent UUID column of an inventory row.

 @param row - a row of `btrfs subvolume list -u -q`

 @returns the UUID, the dash when the subvolume was not snapshotted from another, or the question mark when the row has no such column
 */
function parentUuidOf(row: string,): string {
  /**
   Where the label starts, or minus one when the row has none.
   */
  const at = row.indexOf(PARENT_UUID_LABEL,);
  if (at === (-1)) {
    return PARENT_UNKNOWN;
  }
  /**
   The text after the label.
   */
  const rest = row.slice(at + PARENT_UUID_LABEL.length,);
  return rest.slice(
    0,
    rest.indexOf(' ',),
  );
}

/**
 Parses one inventory row.

 @param row - a row of `btrfs subvolume list -u`
 
 @returns the row's columns
 
 @throws MoveStopError when the row lacks an ID, a parent ID, a UUID or a path
 */
function parseRow(row: string,): ListedRow {
  /**
   The uuid at of this step.
   */
  const uuidAt = row.indexOf(UUID_LABEL,);
  /**
   The path at of this step.
   */
  const pathAt = row.indexOf(
    PATH_LABEL,
    uuidAt,
  );
  /**
   The top at of this step.
   */
  const topAt = row.indexOf(TOP_LEVEL_LABEL,);
  if ((!row.startsWith(ID_LABEL,)) || (uuidAt === (-1))
    || (pathAt === (-1))
    || (topAt === (-1))) {
    throw new MoveStopError(
      `inventory line not understood or without a UUID (list the filesystem with 'btrfs subvolume list -u -q'): ${row}`,
    );
  }
  return {
    id: row.slice(
      ID_LABEL.length,
      row.indexOf(
        ' ',
        ID_LABEL.length,
      ),
    ),
    topLevel: row.slice(
      topAt + TOP_LEVEL_LABEL.length,
      row.indexOf(
        ' ',
        topAt + TOP_LEVEL_LABEL.length,
      ),
    ),
    uuid: row.slice(
      uuidAt + UUID_LABEL.length,
      pathAt,
    ),
    parentUuid: parentUuidOf(row,),
    path: row.slice(pathAt + PATH_LABEL.length,),
  };
}

/**
 Builds each row's path from the top of the filesystem by following the parent chain.

 A row whose parent is not in the list (the top level, ID 5) keeps its path. A row whose path starts with its parent's
 own path plus a slash is taken as already relative to the parent's anchor; any other path is taken as relative to the
 parent.

 @param rows - every listed row
 
 @returns the rows with full paths, in the same order

 @example
 ```ts
 import { resolveFullPaths, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = resolveFullPaths(rows,);
 ```
 */
export function resolveFullPaths(rows: readonly ListedRow[],): readonly Subvolume[] {
  /**
   The by id of this step.
   */
  const byId = new Map(rows.map(function toEntry(row,) {
    return [
      row.id,
      row,
    ] as const;
  },),);

  /**
   Full path of one row, following parents.

   @param row - the row
   
   @param depth - recursion guard against a cycle in malformed input
   
   @returns the full path
   */
  function fullPath({
    row,
    depth,
  }: {
    readonly row: ListedRow;
    readonly depth: number
  },): string {
    /**
     The parent entry of the subvolume, when it has one.
     */
    const parent = byId.get(row.topLevel,);
    if ((parent === undefined) || (parent.id === row.id)
      || (depth > rows.length)) {
      return row.path;
    }
    /**
     The parent full of this step.
     */
    const parentFull = fullPath({
      row: parent,
      depth: depth + 1,
    },);
    return row.path
      .startsWith(`${parent.path}/`,)
      ? `${parentFull}${row.path
        .slice(parent.path
          .length,)}`
      : `${parentFull}/${row.path}`;
  }

  return rows.map(function toSubvolume(row,) {
    return {
      uuid: row.uuid,
      parentUuid: row.parentUuid,
      path: fullPath({
        row,
        depth: 0,
      },),
    };
  },);
}

/**
 Parses the inventory file into the subvolumes below the main checkout plus the main checkout itself.

 @param text - the content of `btrfs subvolume list -u` output
 
 @param options - the options naming the main checkout's listing path and the prefix mapping
 
 @returns the main checkout's row as listed, unmapped, because only its UUID is used (empty path when it has no row), and the nested subvolumes mapped to real paths, sorted
 
 @throws MoveStopError when a row cannot be understood or a path is outside the prefix

 @example
 ```ts
 import { parseInventory, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = parseInventory({ text, options, },);
 ```
 */
export function parseInventory(
  {
    text,
    options,
  }: {
    readonly text: string;
    readonly options: MoveOptions;
  },
): {
  readonly main: Subvolume;
  readonly nested: readonly Subvolume[];
} {
  /**
   The rows this step builds or reads.
   */
  const rows = resolveFullPaths(text.split('\n',)
    .filter(function notBlank(row,) {
      return row.trim() !== '';
    },)
    .map(parseRow,),);

  /**
   Maps a top-level path to a real path.

   @param path - the path column as resolved
   
   @param uuid - the subvolume UUID
   
   @returns the same UUID at the mapped path
   
   @throws MoveStopError when the path is outside the prefix
   */
  function mapped({
    path,
    uuid,
    parentUuid,
  }: Subvolume,): Subvolume {
    if (!path.startsWith(options.inventoryPrefixFrom,)) {
      throw new MoveStopError(`inventory path outside ${options.inventoryPrefixFrom}: ${path}`,);
    }
    return {
      uuid,
      parentUuid,
      path: `${options.inventoryPrefixTo}${path.slice(options.inventoryPrefixFrom
        .length,)}`,
    };
  }

  /**
   The main checkout.
   */
  const main = rows.find(function isMain(row,) {
    return row.path === options.inventoryMain;
  },);
  return {
    main: main ?? {
      path: '',
      uuid: UNKNOWN_UUID,
      parentUuid: PARENT_UNKNOWN,
    },
    nested: rows.filter(function isBelowMain(row,) {
      return row.path
        .startsWith(`${options.inventoryMain}/`,);
    },)
      .map(mapped,)
      .toSorted(function byPath(
        left,
        right,
      ) {
        return left.path
          .localeCompare(right.path,);
      },),
  };
}

/**
 Builds the list of subvolumes this run covers: the main checkout, then the nested ones in sorted order.

 With an inventory, Git's registered subvolumes and the filesystem's list must agree in both directions and every
 subvolume carries its UUID. Without one (print mode only) the UUIDs are unknown.

 @param options - the options
 
 @param log - sink for the agreement line
 
 @returns main first, then nested
 
 @throws MoveStopError when the two lists differ, or run mode has no inventory

 @example
 ```ts
 import { reconcileInventory, } from './inventory.ts';
 
 const result = await reconcileInventory({ options, log, },);
 ```
 */
export async function reconcileInventory(
  {
    options: givenOptions,
    log,
  }: {
    readonly options: MoveOptions;
    readonly log: (message: string,) => void;
  },
): Promise<readonly Subvolume[]> {
  /**
   The options with the folders the move compares in their canonical spelling.
   */
  const canonical = await canonicalOptions(givenOptions,);
  /**
   The worktrees of this step, each in both spellings.
   */
  const worktrees = await listRegisteredWorktreeSpellings(canonical.mainPath,);
  /**
   The subvolume flags of this step.
   */
  const subvolumeFlags = await Promise.all(worktrees.map(async function check(entry,) {
    return await isSubvolume({
      path: entry.canonical,
      marker: canonical.subvolumeMarker,
    },);
  },),);
  /**
   The registered of this step, in canonical spelling.
   */
  const registered = worktrees.filter(function keepSubvolume(
    _entry,
    index,
  ) {
    return subvolumeFlags[index] === true;
  },)
    .map(function canonicalOf(entry,) {
      return entry.canonical;
    },)
    .toSorted();
  /**
   The limited of this step.
   */
  const limited = registered.slice(
    0,
    canonical.limit,
  );
  if (canonical.inventoryFile === '') {
    if (canonical.run) {
      throw new MoveStopError(
        '--run needs --inventory (root: btrfs subvolume list -u -q of the whole filesystem), because every record is keyed by UUID',
      );
    }
    return [
      {
        path: canonical.mainPath,
        uuid: UNKNOWN_UUID,
        parentUuid: PARENT_UNKNOWN,
      },
      ...limited.map(function unknown(path,) {
        return {
          path,
          uuid: UNKNOWN_UUID,
          parentUuid: PARENT_UNKNOWN,
        };
      },),
    ];
  }
  /**
   The inventory file's text.
   */
  const inventoryText = await readFile(
    canonical.inventoryFile,
    'utf8',
  );
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    main,
    nested,
  } = parseInventory({
    text: inventoryText,
    options: canonical,
  },);
  /**
   The listed paths of this step.
   */
  const listedPaths = nested.map(function pathOf(row,) {
    return row.path;
  },);
  /**
   The unknown to git of this step.
   */
  const unknownToGit = listedPaths.filter(function notRegistered(path,) {
    return !registered.includes(path,);
  },);
  /**
   The not subvolumes of this step.
   */
  const notSubvolumes = registered.filter(function notListed(path,) {
    return !listedPaths.includes(path,);
  },);
  if ((unknownToGit.length > 0) || (notSubvolumes.length > 0)) {
    /**
     The inventory rows as the caller's own mapping spells them, in the same order as the canonical rows.
     */
    const givenRows = parseInventory({
      text: inventoryText,
      options: givenOptions,
    },)
      .nested;
    throw new MoveStopError(describeDisagreement({
      unknownToGit,
      notSubvolumes,
      listedAs: new Map(nested.map(function entryOf(
        row,
        index,
      ): readonly [
        string,
        string,
      ] {
        return [
          row.path,
          givenRows[index]
            ?.path
            ?? row.path,
        ];
      },),),
      registeredAs: new Map(worktrees.map(function entryOf(entry,): readonly [
        string,
        string,
      ] {
        return [
          entry.canonical,
          entry.given,
        ];
      },),),
    },),);
  }
  if (main.path === '') {
    throw new MoveStopError(`the inventory has no row for the main checkout at ${canonical.inventoryMain}`,);
  }
  log(`inventory agrees: ${String(nested.length,)} nested subvolumes`,);
  return [
    {
      path: canonical.mainPath,
      uuid: main.uuid,
      parentUuid: main.parentUuid,
    },
    ...nested.filter(function isCovered(row,) {
      return limited.includes(row.path,);
    },),
  ];
}
