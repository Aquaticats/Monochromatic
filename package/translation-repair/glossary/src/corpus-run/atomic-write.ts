import { randomUUID, } from 'node:crypto';
import {
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Atomic write
// Writing a file that a CONCURRENT READER may list at any moment.
//
// The accumulation runs for days into a directory every rate-producing reader
// globs. A direct write is visible while it is still partial, and a partial
// artifact is worse than an unreadable one here: the census classifies it as
// malformed, the pool keeps malformed files deliberately so the reader that
// reports them still sees them, and a later reader then parses a
// now-complete file and counts it without the generation checks it should have
// faced. The window is small and the consequence is a silently wrong
// denominator, which is this package's recurring failure.
//
// Rename within one directory is atomic on every filesystem this runs on, so a
// reader sees the file either absent or whole.
//
// A WRITE THAT FAILS TAKES ITS TEMPORARY FILE WITH IT. A write the disk
// refused part way, or a rename refused by a directory standing at the path,
// used to leave `<path>.<pid>.partial` beside the target for good. Every
// reader steps over that name (`directory-listing.ts` and each reader's suffix
// filter), and none relies on finding one, so nothing was misread; the files
// only piled up. A process killed mid-write still leaves one, which nothing
// running can remove.
//
// ONE TEMPORARY NAME PER CALL, NOT PER PROCESS. Two writes of one path at
// once in one process shared `<path>.<pid>.partial`: both opened the same
// file, the later rename found it gone and rejected, and the two writes'
// bytes could interleave in it. Measured on 2026-10-05 over 100 such pairs:
// 100 rejected one write, and 8 left text at the path that was neither write
// whole. With a name per call, removing a failed write's file can never take
// a sibling write's file either.

/**
 Logger root for the write.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Removes the temporary file a failed write may have left, so the write's own
 refusal is what its caller sees and nothing is left beside the target.

 A REMOVAL THAT FAILS TOO is said on a warning naming the file and the
 filesystem code, and the caller still sees the write's refusal: that is the
 failure the caller has to act on, and the file left over is one more
 `.partial` every reader steps over.

 @param partial - temporary file of the failed write, absent where the write
 failed before creating it

 @example
 ```ts
 await removePartial({ partial, },);
 ```
 */
async function removePartial(
  { partial, }: { readonly partial: string; },
): Promise<void> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: removePartial.name,
    l,
  },);
  try {
    await rm(
      partial,
      { force: true, },
    );
  } catch (error) {
    rl.warn(`${partial} is left after a failed write and could not be removed (${failureName({ error, },)})`,);
  }
}

/**
 Writes a file so no reader can observe it half-written.

 The temporary name carries the process id and a name only this call knows,
 so no two writes of the same path, by two passes or by two calls in one
 process, share a temporary file. It sits beside the target rather than in a
 system temporary directory, because rename is only atomic within a
 filesystem and the two can differ.

 @param path - final path the content should appear at

 @param text - complete file content

 @throws whatever the write or the rename raised, unchanged, once the
 temporary file is removed

 @example
 ```ts
 await writeFileAtomic({ path, text, },);
 ```
 */
export async function writeFileAtomic(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): Promise<void> {
  /**
   Path the content is built at before it takes its real name.
   */
  const partial = `${path}.${String(process.pid,)}.${randomUUID()}.partial`;

  try {
    await writeFile(
      partial,
      text,
    );
    await rename(
      partial,
      path,
    );
  } catch (error) {
    await removePartial({ partial, },);
    throw error;
  }
}

//endregion Atomic write
