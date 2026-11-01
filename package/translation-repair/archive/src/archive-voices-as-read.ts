import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ArchiveBlockReviewWire, } from './archive-block-review-wire.ts';

//region Archive voices as read
// Split out of `archive-block-review-stage.ts` when the composite candidate
// (ledger B712) took the stage over its line budget.

/**
 Replies as the review reads them.

 A replacement that shows a reader nothing is the removal the wire spells as
 an empty replacement, and it is read as that removal once, here, so the shape
 floor, the slate's rendering, the selectors' evidence and the page splice all
 read what was decided rather than each asking an empty-string comparison of
 the raw reply (ledger B40; "Two layers reading one refusal" in
 `package/translation-repair/doc/mistake-prevention.md`). Asked that way, a
 revision of one zero-width space passed the shape floor as a paragraph,
 reached the selectors as text and shipped, and a revision of spaces was
 withheld as one of no blocks.

 @param voices - replies heard

 @returns The same replies, a replacement that shows nothing spelled empty

 @example
 ```ts
 const voices = voicesAsRead({ voices: gather.voices, },);
 ```
 */
export function voicesAsRead(
  { voices, }: { readonly voices: readonly HeardVoice<ArchiveBlockReviewWire>[]; },
): readonly HeardVoice<ArchiveBlockReviewWire>[] {
  return voices
    .map(function asRead(voice,): HeardVoice<ArchiveBlockReviewWire> {
      if (!rendersAsNothing({
        text: voice.value
          .replacementText,
      },))
        return voice;
      return {
        ...voice,
        value: {
          ...voice.value,
          replacementText: '',
        },
      };
    },);
}

//endregion Archive voices as read
