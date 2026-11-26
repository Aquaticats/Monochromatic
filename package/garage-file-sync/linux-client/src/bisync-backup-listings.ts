/**
 Tells whether bisync kept the backup listings that let `--recover` resume an aborted run.

 When bisync aborts with "Must run --resync to recover", it moves its listings aside as
 `<name>.path1.lst-old` and `<name>.path2.lst-old`. With `--recover` and both backups
 present, the next run resumes from them and no `--resync` is needed
 (doc/troubleshooting/rclone-bisync-aborts-on-a-name-the-local-disk-refuses.md); only
 without them does syncing stay stopped. The alert's advice depends on which case it is.

 Bisync names both files after both paths of the pair (`bilib.SessionName` at rclone
 `v1.76.0`), so the listings of another local folder paired with `garage:files/Plain`
 belong to a different pair and say nothing about this home's.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Path1 of the pair, as the sync unit names it.
 */
const GARAGE_PAIR_PATH = 'garage:files/Plain';

/**
 Characters bisync writes as `_` in a session name.

 `bilib.CanonicalPath` at rclone `v1.76.0` replaces ASCII whitespace, `\`, `/`, `:`, `?`
 and `*`; that is Go's `\s`, which matches fewer characters than JavaScript's.
 */
const NON_CANONICAL_CHARACTERS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\f',
  '\r',
  '\\',
  '/',
  ':',
  '?',
  '*',
],);

/**
 Characters bisync drops from both ends of a path in a session name.
 */
const PATH_SEPARATORS: ReadonlySet<string> = new Set([
  '\\',
  '/',
],);

/**
 What the backup-listing check found.

 `problem` names why the folder could not be read; `exist` is then false, so the alert
 gives the safer advice.
 */
export type BackupListings = {
  readonly exist: boolean;
  readonly problem?: string;
};

/**
 The two backup-listing names of one home's pair.
 */
export type BackupListingNames = {
  readonly path1: string;
  readonly path2: string;
};

/**
 Lists a folder's names, or says why it could not.

 @param folder - folder to list

 @returns the names, or no names and the reason
 */
async function namesIn(folder: string,): Promise<{
  readonly names: readonly string[];
  readonly problem?: string;
}> {
  try {
    return { names: await readdir(folder,), };
  } catch (error) {
    return {
      names: [],
      problem: `cannot list ${folder}: ${String(error,)}`,
    };
  }
}

/**
 Converts a path to the form bisync puts in a session name.

 Bisync trims `\` and `/` from both ends and replaces every non-canonical character with
 `_` (`bilib.CanonicalPath` at rclone `v1.76.0`).

 @param path - one side's path, as rclone names it

 @returns the path as it appears in the session name
 */
function canonicalPath(path: string,): string {
  /**
   Characters of the path, one code point each.
   */
  const characters = Array.from(path,);
  /**
   Index of the first character that is not a separator, or -1 when every character is.
   */
  const first = characters.findIndex(function isContent(character,) {
    return !PATH_SEPARATORS.has(character,);
  },);
  if (first === (-1)) return '';
  /**
   Index of the last character that is not a separator.
   */
  const lastContent = characters.findLastIndex(function isContent(character,) {
    return !PATH_SEPARATORS.has(character,);
  },);
  /**
   Path without the separators bisync drops from both ends.
   */
  const trimmed = characters.slice(
    first,
    lastContent + 1,
  );
  return trimmed
    .map(function canonical(character,) {
      return NON_CANONICAL_CHARACTERS.has(character,) ? '_' : character;
    },)
    .join('',);
}

/**
 Removes the first `{...}` span of a path.

 Bisync strips the canonical `{hexstring}` suffix that config overrides used to add
 (`bilib.SessionName` and `bilib.StripHexString` at rclone `v1.76.0`).

 @param path - canonical path of one side

 @returns the path without its first `{...}` span
 */
function stripHexString(path: string,): string {
  /**
   Index of the first `{`, or -1 when there is none.
   */
  const open = path.indexOf('{',);
  /**
   Index of the first `}`, or -1 when there is none.
   */
  const close = path.indexOf('}',);
  if ((open === (-1)) || (close <= open)) return path;
  /**
   Text before the span.
   */
  const before = path.slice(
    0,
    open,
  );
  /**
   Text after the span.
   */
  const after = path.slice(close + 1,);
  return `${before}${after}`;
}

/**
 Derives the two backup-listing names bisync writes for this home's pair.

 Bisync's session name joins both paths of the pair with `..`: the Garage path
 `garage:files/Plain` and the local folder the sync unit passes, `%h/Plain`
 (`bilib.SessionName` at rclone `v1.76.0`). On this PC that gives
 `garage_files_Plain..home_user_Plain`, and both `.lst-old` files carry it.

 @param home - home folder whose `Plain` is the local side of the pair

 @returns the exact `.path1.lst-old` and `.path2.lst-old` names

 @example
 ```ts
 backupListingNames({ home: '/home/user', },);
 // { path1: 'garage_files_Plain..home_user_Plain.path1.lst-old', path2: '...path2.lst-old' }
 ```
 */
export function backupListingNames({ home, }: Readonly<{ home: string; }>,): BackupListingNames {
  /**
   Path1 of the pair, as it appears in the session name.
   */
  const garagePath = canonicalPath(GARAGE_PAIR_PATH,);
  /**
   Local folder of the pair, as the sync unit names it.
   */
  const localPath = join(
    home,
    'Plain',
  );
  /**
   Local side of the pair, as it appears in the session name.
   */
  const localSession = stripHexString(canonicalPath(localPath,),);
  /**
   Session name of this home's pair, without a listing suffix.
   */
  const session = `${garagePath}..${localSession}`;
  return {
    path1: `${session}.path1.lst-old`,
    path2: `${session}.path2.lst-old`,
  };
}

/**
 Looks for both backup listings of this home's pair in bisync's working folder.

 @param home - home folder whose `.cache/rclone/bisync` bisync uses

 @returns whether both `.lst-old` backups of the Garage pair exist

 @example
 ```ts
 const { exist } = await backupListingsExist({ home: '/home/example', },);
 ```
 */
export async function backupListingsExist({ home, }: Readonly<{ home: string; }>,): Promise<BackupListings> {
  /**
   bisync's working folder under the home folder.
   */
  const folder = join(
    home,
    '.cache',
    'rclone',
    'bisync',
  );
  /**
   Names in the folder, or the reason they could not be listed.
   */
  const listed = await namesIn(folder,);
  if (listed.problem !== undefined) {
    return {
      exist: false,
      problem: listed.problem,
    };
  }
  /**
   Names in the folder.
   */
  const { names } = listed;
  /**
   The two names this home's pair leaves behind.
   */
  const wanted = backupListingNames({ home, },);
  return {
    exist: names.includes(wanted.path1,)
      && names.includes(wanted.path2,),
  };
}
