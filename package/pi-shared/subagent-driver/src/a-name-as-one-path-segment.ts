/**
 A name that becomes a path: an entry's name is one path segment, and every file the package writes for a name is inside the folder it
 belongs to.

 A name arrives from a queue file, which whoever can write that file decides. On 2026-11-16 the review of `main...35d724bdbe` showed an
 entry named `../outside` replacing a file beside the runs folder: `setAsideFileOf` joined the runs folder with the name and the
 set-aside write renamed over the result. Text that becomes a path obeys the grammar of paths, so the name is checked at the door,
 where an entry is read, and again at the join, where it becomes a path.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  resolve,
  sep,
} from 'node:path';

/**
 A name that is not one path segment, so no file or folder can be named after it.
 */
export class PathSegmentError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PathSegmentError';
}

/**
 Reads why a name is not one path segment.

 A segment is what a folder holds: anything but a separator, and not the folder itself, the folder above, or empty. A control character
 is refused too, because a newline or a null byte ends a path in every program that reads one as text. A leading dash is allowed: the
 package writes the name beside the runs, joined onto a folder, and never hands it to a program as a word of its own.

 @param name - the name a file or folder would be named after

 @returns the problem in words, or nothing when the name is one segment

 @example
 ```ts
 const { problem } = pathSegmentProblemOf({ name: '../outside' },);
 ```
 */
export function pathSegmentProblemOf({ name, }: { readonly name: string },): { readonly problem?: string } {
  if (name === '') {
    return { problem: 'it is no name at all', };
  }
  if (name.includes('/',)) {
    return { problem: `it holds a path separator, so it names more than one segment: ${JSON.stringify(name,)}`, };
  }
  if ((name === '.') || (name === '..')) {
    return { problem: `it is the folder itself or the folder above: ${JSON.stringify(name,)}`, };
  }
/**
 The first code a control character has, which is a space minus one.
 */
  const FIRST_CONTROL_CODE = 0x20;

/**
 The code of the delete character, which is a control character too.
 */
  const DELETE_CODE = 0x7F;

  for (const character of name) {
    /**
     The code of that character, which a control character sits below.
     */
    const code = character.codePointAt(0,) ?? 0;
    if ((code < FIRST_CONTROL_CODE) || (code === DELETE_CODE)) {
      return { problem: `it holds a control character, so a program reading it as text would end the path there: ${JSON.stringify(name,)}`, };
    }
  }
  return {};
}

/**
 Reads the path of a file named after one entry, inside the folder it belongs to.
 The name is checked before it is joined, and the join is checked again against the folder: a name that passes the first check and
 still lands outside fails here rather than being written.

 @param runsFolder - the folder the file belongs in

 @param name - the entry's name, which is one path segment

 @param suffix - what the file's name adds after the entry's name

 @returns the file's path, inside the folder

 @throws PathSegmentError when the name is not one segment, or the join would leave the folder

 @example
 ```ts
 const file = fileBesideTheRuns({ runsFolder: '/runs', name: 'a', suffix: '-set-aside.json', },);
 ```
 */
export function fileBesideTheRuns({
  runsFolder,
  name,
  suffix,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly suffix: string;
},): string {
  /**
   Why the name is not one segment, if it is not.
   */
  /**
   Why the name is not one segment, if it is not.
   */
  const { problem, } = pathSegmentProblemOf({ name, },);
  if (problem !== undefined) {
    throw new PathSegmentError(`${JSON.stringify(name,)} is not one path segment, so it cannot name a file beside the runs: ${problem}`,);
  }

  /**
   The folder the file belongs in, as an absolute path.
   */
  const inside = resolve(runsFolder,);
  /**
   The file's path, joined and then checked against that folder.
   */
  const file = resolve(
    inside,
    `${name}${suffix}`,
  );
  if (!file.startsWith(`${inside}${sep}`,)) {
    throw new PathSegmentError(`${JSON.stringify(name,)} joined to ${runsFolder} names ${file}, which is outside it`,);
  }
  return file;
}

/**
 Writes a file whole, and renames it into place, so no reader sees half of it and no link is written through.

 The text lands in a file of its own beside the target, whose name nothing can predict, and the rename replaces whatever stands at the
 target: a link there is replaced, never followed. A write straight to the target's own being-written name would be a path another
 process can plant a link at, which is how a record could be written beside the folder it belongs to.

 @param file - the target's path, inside the folder it belongs to

 @param text - what the file is to hold

 @throws Error when the text cannot be written, and the file it wrote is removed first

 @example
 ```ts
 await writeInOneStep({ file: '/runs/a-set-aside.json', text: '{}\n', },);
 ```
 */
export async function writeInOneStep({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string;
},): Promise<void> {
  /**
   The file the text is written in first, which is beside the target and named after nothing another process can guess.
   */
  const beside = `${file}${BEING_WRITTEN_SUFFIX}-${randomUUID()}`;
  try {
    await writeFile(
      beside,
      text,
      {
        encoding: 'utf8',
        flag: 'wx',
      },
    );
    await rename(
      beside,
      file,
    );
  }
  catch (error) {
    await rm(
      beside,
      { force: true, },
    );
    throw error;
  }
}

/**
 The end of the name a file is written under first, before it is renamed into place.
 */
const BEING_WRITTEN_SUFFIX = '.being-written';
