import { readdir, } from 'node:fs/promises';

import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Directory listing
// Lists a directory's entries of the kind its reader takes, and names WHY a
// listing failed in the terms an operator acts on.
//
// EVERY LISTING TAKES ONE KIND (ledger B65). Each directory a run writes holds
// regular files or directories its own writer made, and nothing else: a
// symlink, a socket, a fifo or a device is never a record, and neither is a
// directory where the writer makes files. Readers that listed bare names and
// opened each took a directory named like a record as one (EISDIR out of the
// slice cache, ENOTDIR out of the published tree), read a record twice
// through a link, or read a write still in flight under its temporary name
// (`atomic-write.ts`). So a reader lists through `namesOfKind` and filters
// the names its writer gives records, and nothing else in the package calls
// `readdir` but the walkers `directory-listing-scan.unit.test.ts` names, each
// with why it must see every kind.
//
// LIFTED OUT OF TWO COPIES on 2026-08-25, following the same rule
// `error-name.ts` records: `verify-published.ts` returned an empty array and
// printed the absence, and `editor-standing-read.ts` carried its own
// `DirectoryReading` union for the identical job. A third caller was about to
// be written for the published-tree listing.
//
// NAMES THE FILESYSTEM REASON, NOT THE CLASS. Both copies reported
// `errorName`, which answers `Error` for every filesystem failure: a run
// directory that was never created and one whose permissions were changed read
// identically, while the remedies are "point at the right run" and "fix the
// mode". The `code` separates them and is a bounded token rather than a
// message, so it never quotes the path it failed on. A run directory path can
// name a person.

/**
 What listing one directory produced.

 ABSENCE IS A KIND rather than an empty list of names, because the two decide
 different things. A directory that holds nothing and a directory that is not
 there both yield nothing to work on, but only one of them says the caller
 was pointed somewhere real.

 TYPED BY THE NAMES A READER TAKES, so a listing of artifacts reads as
 artifact file names (`artifact-file-name.ts`) through the same union.

 @example
 ```ts
 const reading: DirectoryReading = { kind: 'read', names: [], };
 ```
 */
export type DirectoryReading<NameT extends string = string,> =
  | {
    readonly kind: 'read';

    /**
     The entries the reader takes, in whatever order the directory gave them.
     */
    readonly names: readonly NameT[];
  }
    | {
      readonly kind: 'unreadable';

    /**
     Filesystem reason, as a bounded token: `ENOENT`, `EACCES`, `ENOTDIR`.
     */
      readonly reason: string;
    };

/**
 Narrows a caught value to one carrying a filesystem error code.

 POSITIONAL BY NECESSITY, against the house preference for a destructured
 parameter: a type guard narrows the binding it names, and a parameter
 destructured out of an object narrows nothing the caller holds.

 @param error - caught value, of unknown type by construction

 @returns Whether a `code` string can be read off it

 @example
 ```ts
 if (carriesFilesystemCode(error,))
   console.log(error.code,);
 ```
 */
function carriesFilesystemCode(error: unknown,): error is { readonly code: string; } {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && ((typeof error.code) === 'string');
}

/**
 Names why an operation on a path failed, for a reader deciding what to do.

 @param error - caught value, of unknown type by construction

 @returns Filesystem code, falling back to the class where there is none

 @example
 ```ts
 console.log(filesystemReason({ error, },),);
 ```
 */
export function filesystemReason(
  { error, }: { readonly error: unknown; },
): string {
  if (carriesFilesystemCode(error,))
    return error.code;
  return errorName({ error, },);
}

/**
 Kind of directory entry a reader takes as one of its records: a regular file
 or a directory, never a link or a special file.

 @example
 ```ts
 const kind: EntryKind = 'file';
 ```
 */
export type EntryKind = 'file' | 'directory';

/**
 Lists the entries of one kind a directory holds, by name.

 A SYMLINK IS NEITHER KIND. The entry's own type is read rather than its
 target's, so a link to a record is not a second record and a link out of
 the directory is not followed.

 @param dir - directory to list

 @param kind - kind of entry the reader takes

 @returns Names of the entries of that kind, unsorted

 @throws What `readdir` raises, an absent directory included

 @example
 ```ts
 const names = await namesOfKind({ dir, kind: 'file', },);
 ```
 */
export async function namesOfKind(
  {
    dir,
    kind,
  }: {
    readonly dir: string;
    readonly kind: EntryKind;
  },
): Promise<readonly string[]> {
  return (await readdir(
    dir,
    { withFileTypes: true, },
  ))
    .filter(function isOfKind(entry,): boolean {
      return (kind === 'file') ? entry.isFile() : entry.isDirectory();
    },)
    .map(function toName(entry,): string {
      return entry.name;
    },);
}

/**
 Lists the entries of one kind a directory holds, reading an absent directory
 as holding none, for a directory its writer makes on first write.

 @param dir - directory to list

 @param kind - kind of entry the reader takes

 @returns Names of the entries of that kind, empty where the directory is not
 there

 @throws What `readdir` raises for any failure but an absent directory

 @example
 ```ts
 const names = await presentNamesOfKind({ dir, kind: 'file', },);
 ```
 */
export async function presentNamesOfKind(
  {
    dir,
    kind,
  }: {
    readonly dir: string;
    readonly kind: EntryKind;
  },
): Promise<readonly string[]> {
  try {
    return await namesOfKind({
      dir,
      kind,
    },);
  } catch (error) {
    // Only an absent directory is an answer; a permission fault read as empty
    // would say the writer never wrote.
    rethrowUnlessMissingPath({ error, },);
    return [];
  }
}

/**
 Runs one listing, reporting a directory it could not read rather than
 raising, for readers that choose between layouts or report a missing run as
 a finding.

 @param list - listing to run, which raises what `readdir` raises

 @returns Its names, or why the directory could not be listed

 @example
 ```ts
 const reading = await readingOf({ list: async () => await namesOfKind({ dir, kind: 'file', },), },);
 ```
 */
export async function readingOf<const NameT extends string,>(
  { list, }: { readonly list: () => Promise<readonly NameT[]>; },
): Promise<DirectoryReading<NameT>> {
  try {
    return {
      kind: 'read',
      names: await list(),
    };
  } catch (error) {
    return {
      kind: 'unreadable',
      reason: filesystemReason({ error, },),
    };
  }
}

/**
 Lists the entries of one kind a directory holds, reporting an absent one
 rather than raising.

 @param dir - directory to list

 @param kind - kind of entry the reader takes

 @returns Their names, or why the directory could not be listed

 @example
 ```ts
 const reading = await namesIn({ dir, kind: 'directory', },);
 ```
 */
export async function namesIn(
  {
    dir,
    kind,
  }: {
    readonly dir: string;
    readonly kind: EntryKind;
  },
): Promise<DirectoryReading> {
  return await readingOf({
    list: async function listed(): Promise<readonly string[]> {
      return await namesOfKind({
        dir,
        kind,
      },);
    },
  },);
}

//endregion Directory listing
