/**
 The file operations of a numbered claim series, as `package/git-policy/cli/src/native/lock/single_owner_claim.rs` does them: a
 record is written whole into a private file and then published with one exclusive `link`, so a claim file is never seen
 half written and the kernel lets exactly one command create a given generation.
 */
import { constants } from 'node:fs';
import {
  link,
  open,
  rename,
  unlink,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  claimPath,
  type ClaimRecord,
  decodeClaimRecord,
  encodeClaimRecord,
  notAClaim,
} from './claim-record.ts';
import { CargoMutantsToolingError } from './error.ts';
import { isMissing } from './fs-guards.ts';

/**
 The most bytes of a claim file that are read (`single_owner_claim.rs:207`).
 */
const claimReadLimit = 4_096;

/**
 The mode of a claim file: readable and writable by its owner only.
 */
const claimMode = 0o600;

/**
 Stands for a claim file that was removed between the listing and the read.
 */
export const claimGone: unique symbol = Symbol('claim-file.removed');

/**
 Tell whether a caught value is a system error with a code.

 @param error - caught value

 @param code - the code, such as `EEXIST`

 @returns true when the value is an error carrying exactly that code

 @example
 ```ts
 hasCode({ error: new Error('x'), code: 'EEXIST' });
 ```
 */
function hasCode({
  error,
  code,
}: {
  readonly error: unknown;
  readonly code: string;
}): boolean {
  return Error.isError(error) && ('code' in error)
    && (error.code === code);
}

/**
 Make the directory's new entry durable (`single_owner_claim.rs:177`).

 @param directory - the directory

 @example
 ```ts
 await syncDirectory('/d');
 ```
 */
export async function syncDirectory(directory: string): Promise<void> {
  /**
   The directory, opened for the flush.
   */
  await using handle = await open(
    directory,
    'r',
  );
  await handle.sync();
}

/**
 Write a record into a new private file beside the claims, flushed (`single_owner_claim.rs:163`).

 @param directory - the directory of the claim files

 @param record - what the file says

 @returns the path of the private file, whose name is outside every series

 @example
 ```ts
 await writePrivate({ directory: '/d', record: { kind: 'released' } });
 ```
 */
export async function writePrivate({
  directory,
  record,
}: {
  readonly directory: string;
  readonly record: ClaimRecord;
}): Promise<string> {
  /**
   A name no other process or call uses.
   */
  const path = join(
    directory,
    `.owner-claim.tmp.${String(process.pid)}.${crypto.randomUUID()}`,
  );
  try {
    /**
     The new file, which must not exist yet.
     */
    await using handle = await open(
      path,
      'wx',
      claimMode,
    );
    await handle.writeFile(encodeClaimRecord(record));
    await handle.sync();
  }
  catch (error) {
    await unlink(path);
    throw error;
  }
  return path;
}

/**
 Publish a record as one generation without replacing anything (`single_owner_claim.rs:182`): the file is linked under its
 name, which fails when the generation exists.

 @param directory - the directory of the claim files

 @param target - the path of the generation

 @param record - what the file says

 @returns true when the generation was created, false when it already existed

 @throws Error when the file cannot be made or linked for any other reason

 @example
 ```ts
 await publishNew({ directory: '/d', target: '/d/item.owner.2', record: { kind: 'released' } });
 ```
 */
export async function publishNew({
  directory,
  target,
  record,
}: {
  readonly directory: string;
  readonly target: string;
  readonly record: ClaimRecord;
}): Promise<boolean> {
  /**
   The whole record, private until it is linked.
   */
  const privatePath = await writePrivate({
    directory,
    record,
  });
  try {
    await link(
      privatePath,
      target,
    );
  }
  catch (error) {
    await unlink(privatePath);
    if (hasCode({
      error,
      code: 'EEXIST',
    }))
      return false;
    throw error;
  }
  await unlink(privatePath);
  await syncDirectory(directory);
  return true;
}

/**
 Rewrite a claim file, atomically, as a released claim (`single_owner_claim.rs:308`).

 @param directory - the directory of the claim files

 @param path - the claim file

 @example
 ```ts
 await rewriteAsReleased({ directory: '/d', path: '/d/item.owner.1' });
 ```
 */
export async function rewriteAsReleased({
  directory,
  path,
}: {
  readonly directory: string;
  readonly path: string;
}): Promise<void> {
  /**
   The released record, private until it replaces the claim.
   */
  const privatePath = await writePrivate({
    directory,
    record: { kind: 'released' },
  });
  try {
    await rename(
      privatePath,
      path,
    );
  }
  catch (error) {
    await unlink(privatePath);
    throw error;
  }
  await syncDirectory(directory);
}

/**
 Read the first bytes of a file that is not followed through a link.

 @param path - the file

 @returns its text, empty for anything that is not a plain file (which then reads as no claim)

 @throws Error when the file cannot be opened or read

 @example
 ```ts
 await readSmallText('/d/item.owner.1');
 ```
 */
async function readSmallText(path: string): Promise<string> {
  /**
   The file, never followed through a link and never waited for: a named pipe with no writer would block an ordinary open.
   */
  await using handle = await open(
    path,
    constants.O_RDONLY | constants.O_NONBLOCK
      | constants.O_NOFOLLOW,
  );
  // Only a plain file is a claim (`worktree_convert_journal.rs`, `read_plain_file`); anything else cannot be read as one.
  if (!(await handle.stat()).isFile())
    return '';
  /**
   What the first bytes hold.
   */
  const buffer = Buffer.alloc(claimReadLimit);
  /**
   How many bytes were read.
   */
  const { bytesRead } = await handle.read(
    buffer,
    0,
    claimReadLimit,
    0,
  );
  return buffer.subarray(
    0,
    bytesRead,
  )
    .toString('utf8');
}

/**
 Read a claim file's text.

 @param path - the claim file

 @returns its text, or {@link claimGone} when the file is gone

 @throws CargoMutantsToolingError when the file cannot be read, because whether another command works on the item cannot
 then be told

 @example
 ```ts
 await readTextOrGone('/d/item.owner.1');
 ```
 */
async function readTextOrGone(path: string): Promise<string | typeof claimGone> {
  try {
    return await readSmallText(path);
  }
  catch (error) {
    if (isMissing(error))
      return claimGone;
    throw new CargoMutantsToolingError(`The claim file ${path} cannot be read (${String(error)}), so it cannot tell whether another command is working here; nothing was changed.`);
  }
}

/**
 Read a claim file's record (`single_owner_claim.rs:191`).

 @param path - the claim file

 @returns the record, or {@link claimGone} when the file is gone

 @throws CargoMutantsToolingError when the file is not a claim of this command, or cannot be read, because whether another
 command works on the item cannot then be told

 @example
 ```ts
 await readClaimRecord('/d/item.owner.1');
 ```
 */
export async function readClaimRecord(path: string): Promise<ClaimRecord | typeof claimGone> {
  /**
   The file's text, absent when it is gone.
   */
  const text = await readTextOrGone(path);
  if (text === claimGone)
    return claimGone;
  /**
   What the text says.
   */
  const record = decodeClaimRecord(text);
  if (record === notAClaim)
    throw new CargoMutantsToolingError(`${path} is not a claim file this command wrote, so it cannot tell whether another command is working here; nothing was changed. Look at the file: if no command of this package is running on this item, remove it and run the command again.`);
  return record;
}

/**
 Remove the older generations once the newer one is published (`single_owner_claim.rs:276`).

 @param directory - the directory of the claim files

 @param prefix - the claim series' name

 @param older - the numbers to remove

 @example
 ```ts
 await removeGenerations({ directory: '/d', prefix: 'item.owner', older: [1] });
 ```
 */
export async function removeGenerations({
  directory,
  prefix,
  older,
}: {
  readonly directory: string;
  readonly prefix: string;
  readonly older: readonly number[];
}): Promise<void> {
  await Promise.all(older.map(async function removeOne(generation) {
    try {
      await unlink(claimPath({
        directory,
        prefix,
        generation,
      }));
    }
    catch (error) {
      if (!isMissing(error))
        console.error(`An older claim file cannot be removed: ${String(error)}`);
    }
  }));
}
