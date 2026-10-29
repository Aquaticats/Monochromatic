import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type CorpusPin,
  isMissingCorpusObject,
  readCorpusBytes,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  photoPath,
  photoReferences,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Entry pictures
// GATHERING ONE ENTRY'S PICTURES OFF THE PINNED CORPUS, which is the only step
// of reading a picture that knows where a corpus lives.
//
// THE LANE KNOWS NOTHING ABOUT DISKS and must not. `readDocumentPictures` takes
// bytes and returns readings; this turns an entry id and a slicing into those
// bytes. Splitting them is what keeps the lane a function of its arguments and
// testable without a corpus checkout.
//
// AT THE PIN, not from a working tree, exactly as the page text is read. A
// picture read from a different commit than its page would be a different
// picture, and the mismatch would show up as a reading nobody could explain.
//
// AN UNREADABLE PICTURE IS SKIPPED RATHER THAN FATAL. Measured over the pinned
// corpus, 380 references resolve to 380 files present and none missing, so this
// path is not expected to run; if it ever does, one absent asset must not cost
// an entry that has 50 other slices to settle. The gap becomes a finding on the
// slices that show it, because `slicePictures` reports a picture with no reading
// as unread.

/**
 Reads every picture one entry's slices name, at the pinned commit.

 @param pin - corpus clone and commit

 @param entryId - person entry whose photos directory holds these assets

 @param slices - prepared slice pairs, whose source sides name pictures

 @param l - entry logger

 @returns Bytes per asset name, omitting any that could not be read

 @example
 ```ts
 const assets = await gatherEntryPictures({ pin, entryId, slices, l, },);
 ```
 */
export async function gatherEntryPictures(
  {
    pin,
    entryId,
    slices,
    l,
  }: {
    readonly pin: CorpusPin;
    readonly entryId: string;
    readonly slices: readonly ChunkPair[];
    readonly l: Logger;
  },
): Promise<ReadonlyMap<string, Uint8Array>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const gl = tagged({
    tag: gatherEntryPictures.name,
    l,
  },);

  /**
   Every picture any slice names on its source side, each once.
   */
  const named = new Set<string>();
  for (const slice of slices) {
    for (const reference of photoReferences({
      text: slice.source
        .text,
    },))
      named.add(reference.assetName,);
  }

  /**
   Bytes gathered so far.
   */
  const gathered = new Map<string, Uint8Array>();
  if (named.size === 0)
    return gathered;

  // Every read, run together: these are local git invocations rather than
  // model calls, so nothing here contends for a per-model slot.
  await allInInputOrder({
    members: [...named].map(async function gather(assetName,): Promise<void> {
      try {
        gathered.set(
          assetName,
          await readCorpusBytes({
            pin,
            relPath: photoPath({
              entryId,
              assetName,
            },),
          },),
        );
      }
      catch (error) {
        // NAMED RATHER THAN SWALLOWED, and not rethrown: a picture the corpus
        // does not carry is a fact about the corpus, and the slices showing it
        // report it as unread.
        if (!isMissingCorpusObject(error,))
          throw error;
        gl.warn(`${entryId}/${assetName}: not in the corpus at this pin (${error.message})`,);
      }
    },),
  },);

  gl.info(`gathered ${String(gathered.size,)} of ${String(named.size,)} ${
    wordForCount({
      count: named.size,
      one: 'picture',
      many: 'pictures',
    },)
  } for ${entryId}`,);
  return gathered;
}

//endregion Entry pictures
