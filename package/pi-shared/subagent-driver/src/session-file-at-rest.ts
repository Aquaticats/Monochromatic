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

import { replaceAFileWithItsMaskedText, } from './replacing-a-file-only-if-it-is-the-one-that-was-read.ts';
import type { TextMasking, } from './secret-value-masking.ts';

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

 @returns that it was rewritten, that there was nothing to do, or why the file still holds what Pi wrote

 @example
 ```ts
 const masked = await maskTheSessionAtRest({ sessionFile, mask, },);
 ```
 */
export async function maskTheSessionAtRest({
  sessionFile,
  mask,
}: {
  readonly sessionFile: string;
  readonly mask: TextMasking;
},): Promise<MaskedSessionAtRest> {
  /**
   What replacing the file found.
   */
  const replaced = await replaceAFileWithItsMaskedText({
    path: sessionFile,
    maskText: mask.maskText,
  },);
  if ('replaced' in replaced) {
    return { outcome: 'rewritten', };
  }
  if ('unchanged' in replaced) {
    return { outcome: 'nothing-to-do', };
  }
  return {
    outcome: 'unmasked',
    problem: `${replaced.problem}, so a value an entry of it holds still stands there`,
  };
}

//endregion Masking it
