/**
 The session file a run leaves behind, masked once the run is over.

 Pi writes every message it keeps through its own hooks, which this package's extension masks before Pi keeps them, so this step
 should find nothing to do. It exists for what no hook reaches: an entry an extension wrote, a compaction or branch summary Pi
 appended itself, and a session a run resumed from before this masking existed. Masking the file at rest is the one boundary every
 entry of it passes, whatever wrote it.

 The masked text goes to a file of its own beside the session and is renamed over it, so the session is either wholly what Pi wrote
 or wholly masked, and a failure leaves it exactly as it was.

 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { textOfAnOrdinaryFile, } from './ordinary-file-opened-without-waiting.ts';
import type { TextMasking, } from './secret-value-masking.ts';

//region Constants

/**
 How many random bytes the file of its own is named with, which is what keeps two rewrites of one folder apart.
 */
const REPLACEMENT_NAME_BYTES = 8;

/**
 The mode a session file has, and the one its replacement keeps.
 */
const PRIVATE_FILE_MODE = 0o600;

//endregion Constants

//region Masking it

/**
 What masking the session at rest found.
 */
export type MaskedSessionAtRest = {
  /**
   Whether the file held a value and was rewritten.
   */
  readonly rewritten: boolean;

  /**
   Why it could not be masked, when it could not.
   */
  readonly problem?: string;
};

/**
 Masks every value of this machine's secret environment in the session a run leaves behind.

 @param sessionFile - the session file Pi wrote, as Pi reported it

 @param mask - the masking

 @returns whether the file was rewritten, and why it could not be when it could not

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
   The file's text, or nothing when there is no ordinary file at that path.
   */
  const read = await textOfAnOrdinaryFile({ path: sessionFile, },);
  if ('absent' in read) {
    return { rewritten: false, };
  }
  if ('cause' in read) {
    return {
      rewritten: false,
      problem: `${sessionFile} could not be read to be masked, so a value an entry of it holds may still stand in it: ${read.cause}`,
    };
  }
  /**
   The text with every value masked.
   */
  const masked = mask.maskText(read.text,);
  if (masked === read.text) {
    return { rewritten: false, };
  }
  /**
   The file of its own the masked text goes to first.
   */
  const replacement = `${sessionFile}.masked-${randomBytes(
    REPLACEMENT_NAME_BYTES,
  )
    .toString('hex',)}`;
  try {
    await writeFile(
      replacement,
      masked,
      {
        mode: 0o600,
        flag: 'wx',
      },
    );
    await rename(
      replacement,
      sessionFile,
    );
    return { rewritten: true, };
  }
  catch (error) {
    await rm(
      replacement,
      { force: true, },
    );
    return {
      rewritten: false,
      problem: `the session could not be rewritten with its values masked, so it keeps what Pi wrote: ${
        Error.isError(error,) ? error.name : 'a failure'
      }`,
    };
  }
}

//endregion Masking it
