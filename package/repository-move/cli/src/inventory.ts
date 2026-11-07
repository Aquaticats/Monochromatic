/**
 The subvolume inventory: the Git registry reconciled with the filesystem's own list, with each subvolume's UUID.

 `btrfs subvolume list -u <filesystem>` (root only) prints one row per subvolume ending in
 `uuid <uuid> path <path from the top level>`; the copy keys every record by that UUID and path.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  MISSING_PATH_CODE,
  pathExists,
} from './completion-records.ts';
import { SUBVOLUME_ROOT_INODE, } from './constants.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';

/**
 A subvolume with the UUID the inventory gave it; the dash when no inventory was given (print mode only).
 */
export type Subvolume = {
  readonly path: string;
  readonly uuid: string;
};

/**
 Runs a program and waits for it to end.
 */
const execFileAsync = promisify(execFile,);

/**
 The UUID of a subvolume whose inventory is unavailable.
 */
const UNKNOWN_UUID = '-';

/**
 Git's own program, never the repository's wrapper.
 */
const GIT = '/usr/bin/git';

/**
 Separator before the UUID in an inventory row.
 */
const UUID_LABEL = ' uuid ';

/**
 Separator before the path in an inventory row.
 */
const PATH_LABEL = ' path ';

/**
 Lists the worktrees Git registers below the main checkout.

 @param mainPath - the main checkout
 
 @returns absolute paths, in Git's order
 
 @throws MoveStopError when Git cannot list them
 */
export async function listRegisteredWorktrees(mainPath: string,): Promise<readonly string[]> {
  try {
    /**
     The output of the listing command.
     */
    const listing = await execFileAsync(
      GIT,
      [
        '--no-optional-locks',
        '-C',
        mainPath,
        'worktree',
        'list',
        '--porcelain',
      ],
      { encoding: 'utf8', },
    );
    return listing.stdout
      .split('\n',)
      .filter(function isWorktreeLine(line,) {
        return line.startsWith('worktree ',);
      },)
      .map(function stripLabel(line,) {
        return line.slice('worktree '.length,);
      },)
      .filter(function isBelowMain(path,) {
        return path.startsWith(`${mainPath}/`,);
      },);
  }
  catch (error) {
    throw new MoveStopError(
      `cannot list worktrees below ${mainPath}`,
      { cause: error, },
    );
  }
}

/**
 Tells whether a folder is a subvolume root: inode 256 on btrfs, or the marker file in tests.

 @param path - the folder
 
 @param marker - name of the marker file that stands for a subvolume in tests; empty on a real filesystem
 
 @returns true for a subvolume root
 */
export async function isSubvolume(
  {
    path,
    marker,
  }: {
    readonly path: string;
    readonly marker: string;
  },
): Promise<boolean> {
  if (marker !== '') {
    return await pathExists(join(
      path,
      marker,
    ),);
  }
  try {
    return (await stat(
      path,
      { bigint: true, },
    )).ino === SUBVOLUME_ROOT_INODE;
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

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
  readonly path: string;
};

/**
 Label before the ID in an inventory row.
 */
const ID_LABEL = 'ID ';

/**
 Label before the parent ID in an inventory row.
 */
const TOP_LEVEL_LABEL = ' top level ';

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
      `inventory line not understood or without a UUID (list the filesystem with 'btrfs subvolume list -u'): ${row}`,
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
  }: Subvolume,): Subvolume {
    if (!path.startsWith(options.inventoryPrefixFrom,)) {
      throw new MoveStopError(`inventory path outside ${options.inventoryPrefixFrom}: ${path}`,);
    }
    return {
      uuid,
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
 */
export async function reconcileInventory(
  {
    options,
    log,
  }: {
    readonly options: MoveOptions;
    readonly log: (message: string,) => void;
  },
): Promise<readonly Subvolume[]> {
  /**
   The worktrees of this step.
   */
  const worktrees = await listRegisteredWorktrees(options.mainPath,);
  /**
   The subvolume flags of this step.
   */
  const subvolumeFlags = await Promise.all(worktrees.map(async function check(path,) {
    return await isSubvolume({
      path,
      marker: options.subvolumeMarker,
    },);
  },),);
  /**
   The registered of this step.
   */
  const registered = worktrees.filter(function keepSubvolume(
    _path,
    index,
  ) {
    return subvolumeFlags[index] === true;
  },)
    .toSorted();
  /**
   The limited of this step.
   */
  const limited = registered.slice(
    0,
    options.limit,
  );
  if (options.inventoryFile === '') {
    if (options.run) {
      throw new MoveStopError(
        '--run needs --inventory (root: btrfs subvolume list -u of the whole filesystem), because every record is keyed by UUID',
      );
    }
    return [
      {
        path: options.mainPath,
        uuid: UNKNOWN_UUID,
      },
      ...limited.map(function unknown(path,) {
        return {
          path,
          uuid: UNKNOWN_UUID,
        };
      },),
    ];
  }
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    main,
    nested,
  } = parseInventory({
    text: await readFile(
      options.inventoryFile,
      'utf8',
    ),
    options,
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
    throw new MoveStopError(
      `inventory differs from the Git registry; subvolumes Git does not know: ${unknownToGit.join(' ',) || 'none'}; `
        + `registered but not subvolumes: ${
          notSubvolumes.join(' ',) || 'none'
        }. Give each an owner-approved disposition first.`,
    );
  }
  if (main.path === '') {
    throw new MoveStopError(`the inventory has no row for the main checkout at ${options.inventoryMain}`,);
  }
  log(`inventory agrees: ${String(nested.length,)} nested subvolumes`,);
  return [
    {
      path: options.mainPath,
      uuid: main.uuid,
    },
    ...nested.filter(function isCovered(row,) {
      return limited.includes(row.path,);
    },),
  ];
}
