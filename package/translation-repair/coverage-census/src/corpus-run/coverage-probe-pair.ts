import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  type CorpusPin,
  isMissingCorpusObject,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Coverage probe pair
// Both sides of one entry, or the fact that it has only one.

/**
 Both sides of one entry, or the fact that it has only one.

 @example
 ```ts
 const texts: PairRead = { kind: 'missing', };
 ```
 */
export type PairRead = {
  /**
   Both files were there.
   */
  readonly kind: 'read';

  /**
   Original document text.
   */
  readonly source: string;

  /**
   Translation document text.
   */
  readonly target: string;
} | {
  /**
   One side is absent, which is an incomplete entry rather than a fault.
   */
  readonly kind: 'missing';
};

/**
 Reads both sides of one entry at the pin.

 A PAGE ABSENT AT A COMMIT THE CLONE HOLDS IS SKIPPED AND LOGGED rather than
 thrown, since an entry with only one side is an ordinary state of this
 corpus. Every other failure propagates: until 2026-10-06 every failed read
 was skipped, so a clone git could not open or a commit the clone lacks read
 as every entry having one side.

 @param pin - corpus clone and commit the reads resolve against

 @param entryId - entry to read

 @param log - logger the skip is written to

 @returns Both texts, or `missing` when either page is absent at the pin

 @throws {@link CorpusReadError} for any read failure other than a page
 absent at a commit the clone holds

 @example
 ```ts
 const texts = await readPair({ pin, entryId: 'Mittens', log, },);
 ```
 */
export async function readPair(
  {
    pin,
    entryId,
    log,
  }: {
    readonly pin: CorpusPin;
    readonly entryId: string;
    readonly log: Logger;
  },
): Promise<PairRead> {
  try {
    return {
      kind: 'read',
      source: await readCorpusFile({
        pin,
        relPath: `people/${entryId}/page.md`,
      },),
      target: await readCorpusFile({
        pin,
        relPath: `people/${entryId}/page.en.md`,
      },),
    };
  }
  catch (error) {
    if (!isMissingCorpusObject(error,))
      throw error;
    log.info(`${entryId}: skipped, ${String(error,)}`,);
    return { kind: 'missing', };
  }
}

//endregion Coverage probe pair
