/**
 The session file a run leaves behind, masked once the run is over, and a resumed session masked before Pi reads it.

 Pi writes every message it keeps through its own hooks, which this package's extension masks before Pi keeps them, so this step
 should find nothing to do for a new run. It exists for what no hook reaches: an entry an extension wrote, a compaction or branch
 summary Pi appended itself, and a session a run resumed from before this masking existed. Masking the file at rest is the one
 boundary every entry of it passes, whatever wrote it.

 The masked text goes to a file of its own beside the session and is renamed over it, and only while the path still names the file
 that was read: an entry another command appended in between is read and masked on the next turn, not dropped.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { textOfAnOrdinaryFile, } from './ordinary-file-opened-without-waiting.ts';
import {
  type FileWriterAnswer,
  replaceAFileWithItsMaskedText,
} from './replacing-a-file-only-if-it-is-the-one-that-was-read.ts';
import type { TextMasking, } from './secret-value-masking.ts';

//region Constants

/**
 The mode a session file has, and the one a copy of it keeps.
 */
const PRIVATE_FILE_MODE = 0o600;

//endregion Constants

//region What it found

/**
 What masking a session found: the three answers every place a value is kept has.
 */
export type MaskedSessionAtRest =
  | { readonly outcome: 'rewritten' }
  // The file was there and held no value, so nothing was written.
    | { readonly outcome: 'nothing-to-do' }
  // No ordinary file is at that path, which is what a session Pi never wrote is.
    | { readonly outcome: 'absent' }
  // The file still holds what Pi wrote, and this says why.
    | {
      readonly outcome: 'unmasked';
      readonly problem: string;
    };

//endregion What it found

//region Masking it

/**
 Masks every value of this machine's secret environment in a session file.

 @param sessionFile - the session file Pi wrote

 @param mask - the masking

 @param writer - what the caller knows about the process that writes the file, which must be gone

 @returns that it was rewritten, that there was nothing to do, or why the file still holds what Pi wrote

 @example
 ```ts
 const masked = await maskTheSessionAtRest({ sessionFile, mask, },);
 ```
 */
export async function maskTheSessionAtRest({
  sessionFile,
  mask,
  writer,
}: {
  readonly sessionFile: string;
  readonly mask: TextMasking;
  readonly writer: FileWriterAnswer;
},): Promise<MaskedSessionAtRest> {
  /**
   What replacing the file found.
   */
  const replaced = await replaceAFileWithItsMaskedText({
    path: sessionFile,
    maskText: mask.maskText,
    writer,
  },);
  if ('replaced' in replaced) {
    return { outcome: 'rewritten', };
  }
  if ('unchanged' in replaced) {
    return { outcome: 'nothing-to-do', };
  }
  return {
    outcome: 'unmasked',
    problem: `${('refused' in replaced) ? replaced.refused : replaced.problem}, so a value an entry of it holds still stands there`,
  };
}

//endregion Masking it

//region A session a run resumes

/**
 What masking a session a run resumes found.
 */
export type MaskedSessionToResume =
  | { readonly path: string }
    | { readonly absent: true }
    | {
      readonly problem: string;
    };

/**
 Masks a session a run resumes into a copy of its own, and says where the copy is.

 The file it copies is another run's, and another Pi may be writing it: replacing it would drop what that writer appends. So it is read
 and left exactly as it is, and the masked text goes to a file of its own that Pi resumes instead. What the original still holds is a
 stated limit of this masking, not a window this code pretends to close.

 @param sessionFile - the session the run was told to resume

 @param copyFile - where the masked copy is written, which must not exist yet

 @param mask - the masking

 @returns the copy's path, that there is no such session, or why the copy could not be written

 @example
 ```ts
 const copy = await maskASessionIntoACopy({ sessionFile, copyFile, mask, },);
 ```
 */
export async function maskASessionIntoACopy({
  sessionFile,
  copyFile,
  mask,
}: {
  readonly sessionFile: string;
  readonly copyFile: string;
  readonly mask: TextMasking;
},): Promise<MaskedSessionToResume> {
  /**
   The file's text, or why there is none.
   */
  const read = await textOfAnOrdinaryFile({ path: sessionFile, },);
  if ('absent' in read) {
    return { absent: true, };
  }
  if ('cause' in read) {
    return { problem: read.cause, };
  }
  try {
    await writeFile(
      copyFile,
      mask.maskText(read.text,),
      {
        mode: PRIVATE_FILE_MODE,
        flag: 'wx',
      },
    );
    return { path: copyFile, };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return {
      problem: `a copy of ${sessionFile} could not be written at ${copyFile}: ${error.name}`,
    };
  }
}

//endregion A session a run resumes
