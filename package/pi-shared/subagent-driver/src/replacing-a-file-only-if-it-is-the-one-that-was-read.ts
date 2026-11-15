/**
 Replacing a file, but only the file that was read: the masked text goes to a sibling first, and the path is swapped only when it
 still names the file whose contents were masked.

 A rename is atomic against a half-written file and helpless against a writer who appended in between: it would replace the newer file
 with the older snapshot. So the file is identified before it is read, the identity is asked for again before the swap, and a file that
 changed is read, masked and asked about again within a bound. A file that keeps changing is left exactly as it is, with the reason.

 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  constants,
  open,
  rename,
  rm,
  writeFile,
  type FileHandle,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';

//region Constants

/**
 How many random bytes the file of its own is named with, which keeps two rewrites of one folder apart.
 */
const REPLACEMENT_NAME_BYTES = 8;

/**
 The mode a replacement keeps, which is the mode every file this touches has.
 */
const PRIVATE_FILE_MODE = 0o600;

/**
 How many times a file that changed under the writer is read, masked and asked about again before it is left alone.
 */
const ATTEMPTS_ON_A_CHANGING_FILE = 3;

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

//endregion Constants

//region What identifies a file

/**
 What identifies the file that was read: the same inode, the same size and the same modification time.

 @param path - the path the file was read from

 @returns what identifies it, or that the path is not there, or could not be asked

 @example
 ```ts
 const identity = await identityOfFile({ path: '/run/session.jsonl', },);
 ```
 */
export async function identityOfFile({ path, }: { readonly path: string },): Promise<
  | { readonly identity: FileIdentity }
    | { readonly absent: true }
    | { readonly problem: string }
  > {
  /**
   The file, opened without following a link and without waiting on a pipe.
   */
  const opened = await openWithoutWaiting({ path, },);
  if ('missing' in opened) {
    return { absent: true, };
  }
  if ('failed' in opened) {
    return { problem: opened.failed, };
  }
  /**
   The opened file, which is closed when this call ends.
   */
  await using handle = opened.handle;
  /**
   What the descriptor says about the file.
   */
  const status = await handle.stat();
  if (!status.isFile()) {
    return { problem: `${path} is not an ordinary file` };
  }
  return {
    identity: {
      device: status.dev,
      inode: status.ino,
      size: status.size,
      modifiedMilliseconds: status.mtimeMs,
    },
  };
}

/**
 What says a path still names the file it named before.
 */
export type FileIdentity = {
  readonly device: number;
  readonly inode: number;
  readonly size: number;
  readonly modifiedMilliseconds: number;
};

/**
 Says whether two identities are the same file, unchanged.

 @param left - one identity

 @param right - the other

 @returns whether the path names the same file with the same contents

 @example
 ```ts
 const unchanged = sameFile({ left, right, },);
 ```
 */
export function sameFile({
  left,
  right,
}: {
  readonly left: FileIdentity;
  readonly right: FileIdentity;
},): boolean {
  return (left.device === right.device) && (left.inode === right.inode)
    && (left.size === right.size)
    && (left.modifiedMilliseconds === right.modifiedMilliseconds);
}

/**
 Opens a path without following a link and without waiting.

 @param path - the path

 @returns the handle, that the path is not there, or why it could not be opened
 */
async function openWithoutWaiting({ path, }: { readonly path: string },): Promise<
  | { readonly handle: FileHandle }
    | { readonly missing: true }
    | { readonly failed: string }
  > {
  try {
    return { handle: await open(
      path,
      constants.O_RDONLY | constants.O_NOFOLLOW
        | constants.O_NONBLOCK,
    ), };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return (errorCodeOf(error,) === MISSING_PATH_CODE)
      ? { missing: true, }
      : { failed: error.name, };
  }
}

//endregion What identifies a file

//region The replacement

/**
 What replacing a file found.
 */
export type ReplacedFile =
  | { readonly replaced: true }
    | { readonly unchanged: true }
    | {
      readonly problem: string;
    };

/**
 Replaces a file's contents with a masked text, only while the path still names the file that was read.

 @param path - the file

 @param maskText - the masking

 @returns whether it was replaced, was already clear, or was left alone and why

 @example
 ```ts
 const replaced = await replaceAFileWithItsMaskedText({ path, maskText, },);
 ```
 */
export async function replaceAFileWithItsMaskedText({
  path,
  maskText,
}: {
  readonly path: string;
  readonly maskText: (text: string) => string;
},): Promise<ReplacedFile> {
  /**
   How many times the file has been read, masked and asked about again.
   */
  const attempts: { taken: number } = { taken: 0, };
  /* oxlint-disable eslint/no-await-in-loop -- one attempt at a time: each turn reads the file again, masks it, and decides whether
     the path still names the file it read before the swap. See doc/lint/sequential-awaits.md. */
  while (attempts.taken < ATTEMPTS_ON_A_CHANGING_FILE) {
    attempts.taken += 1;
    /**
     The file's text, and what identifies it.
     */
    const read = await readAFileWithItsIdentity({ path, },);
    if ('absent' in read) {
      return { unchanged: true, };
    }
    if ('problem' in read) {
      return { problem: read.problem, };
    }
    /**
     The text with every value masked.
     */
    const masked = maskText(read.text,);
    if (masked === read.text) {
      return { unchanged: true, };
    }
    /**
     The file of its own the masked text goes to first.
     */
    const replacement = `${path}.masked-${randomBytes(
      REPLACEMENT_NAME_BYTES,
    )
      .toString('hex',)}`;
    try {
      await writeFile(
        replacement,
        masked,
        {
          mode: PRIVATE_FILE_MODE,
          flag: 'wx',
        },
      );
    }
    catch (error) {
      await rm(
        replacement,
        { force: true, },
      );
      return { problem: `a replacement could not be written beside ${path}: ${Error.isError(error,) ? error.name : 'a failure'}` };
    }
    /**
     The path as it stands now, just before the swap.
     */
    const now = await identityOfFile({ path, },);
    /**
     Whether the path still names the file whose contents were masked.
     */
    const stillTheSame = ('identity' in now) && sameFile({
      left: read.identity,
      right: now.identity,
    },);
    if (!stillTheSame) {
      await rm(
        replacement,
        { force: true, },
      );
      // The file changed under the writer, so its newer text is read and masked in the next turn of this loop.
      continue;
    }
    try {
      await rename(
        replacement,
        path,
      );
      return { replaced: true, };
    }
    catch (error) {
      await rm(
        replacement,
        { force: true, },
      );
      return { problem: `${path} could not be replaced with its masked text: ${Error.isError(error,) ? error.name : 'a failure'}` };
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return { problem: `${path} changed under the rewrite ${String(ATTEMPTS_ON_A_CHANGING_FILE,)} times, so it was left as it is` };
}

/**
 Reads a file and what identifies it, in one opening.

 @param path - the file

 @returns its text and identity, that it is not there, or why it could not be read

 @example
 ```ts
 const read = await readAFileWithItsIdentity({ path: '/run/session.jsonl', },);
 ```
 */
export async function readAFileWithItsIdentity({ path, }: { readonly path: string },): Promise<
  | {
    readonly text: string;
    readonly identity: FileIdentity
  }
    | { readonly absent: true }
    | { readonly problem: string }
  > {
  /**
   The file, opened without following a link and without waiting on a pipe.
   */
  const opened = await openWithoutWaiting({ path, },);
  if ('missing' in opened) {
    return { absent: true, };
  }
  if ('failed' in opened) {
    return { problem: opened.failed, };
  }
  /**
   The opened file, which is closed when this call ends.
   */
  await using handle = opened.handle;
  /**
   What the descriptor says about the file, before it is read.
   */
  const status = await handle.stat();
  if (!status.isFile()) {
    return { problem: `${path} is not an ordinary file` };
  }
  return {
    identity: {
      device: status.dev,
      inode: status.ino,
      size: status.size,
      modifiedMilliseconds: status.mtimeMs,
    },
    text: await handle.readFile({ encoding: 'utf8', },),
  };
}

//endregion The replacement
