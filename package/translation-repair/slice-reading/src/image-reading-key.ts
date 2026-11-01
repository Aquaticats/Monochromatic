import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  IMAGE_READING_PERSPECTIVES,
  READING_INSTRUCTION,
} from './image-reading-stage.ts';
import { CORROBORATION_TRIGRAM_SHARE, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Image reading key
// WHAT MAKES TWO RUNS' PICTURE READINGS THE SAME READING, for cache purposes.
//
// A READING IS NOT DETERMINISTIC and that is the whole reason this exists. Ask
// the same model the same question about the same picture twice and the wording
// differs, so a slice cache key carrying raw reading text would change on every
// run, every picture-bearing slice would miss, and resuming a document would
// re-buy all of its translation work. The reading is stored under a key derived
// from its INPUTS, and the slice key names that key rather than the text.
//
// WHAT AN INPUT IS HERE. The picture's bytes, who was asked, what they were
// asked, and the bar their answers had to clear. Change any of those and a
// stored reading answers a different question; change none and it answers this
// one however differently it is worded.

/**
 Cross-run key for one picture's paired reading.

 @param bytes - picture as read from disk

 @param readerModelIds - vision sub-roster asked about it, in roster order

 @returns Stable hash naming this reading across runs

 @example
 ```ts
 const key = imageReadingKey({ bytes, readerModelIds, },);
 ```
 */
export function imageReadingKey(
  {
    bytes,
    readerModelIds,
  }: {
    readonly bytes: Uint8Array;
    readonly readerModelIds: readonly RosterModelId[];
  },
): string {
  /**
   Picture itself, as content a hash can take.

   BASE64 RATHER THAN A DECODE, since the bytes are not text and any decoding
   would map distinct pictures onto one replacement character.
   */
  const picture = Buffer.from(bytes,)
    .toString('base64',);

  return hashContent({
    content: JSON.stringify([
      'image-reading',
      picture,
      readerModelIds,
      READING_INSTRUCTION,
      ...IMAGE_READING_PERSPECTIVES,
      CORROBORATION_TRIGRAM_SHARE,
    ],),
  },);
}

//endregion Image reading key
