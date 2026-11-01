//region Transcription suspect
// Whether a slice's surplus translation could be read out of a PICTURE rather
// than moved from a neighbour.
//
// WHY THIS IS SEPARATE FROM THE SIZE SCREEN. `displacement-class.ts` reads
// character counts and nothing else, which is what makes it cheap and what makes
// it honest about its limits. This is a TEXT signal, and it answers a question
// size cannot: two of the relocation candidates the size screen produced were
// verified by hand as transcriptions, where the original embeds an image and the
// translation embeds the same image plus a reading of what it holds. The surplus
// is real, the neighbour's deficit is real, and they have nothing to do with
// each other.

/**
 Components that put content on a page without putting it in the text.

 BOTH ENTRIES THAT THIS CAUGHT WERE VERIFIED BY HAND. `wangzihao980/4` embeds a
 `PhotoScroll` of a note and its English adds a full transcription and
 translation of that note; `dogesir_/3` does the same for a self-description.
 In both, the neighbour's deficit is independent condensation that happens to
 sit next door.
 */
const MEDIA_MARKERS = [
  'PhotoScroll',
  '<img',
  '![',
  '<Banner',
  '<Video',
  '<Audio',
  '<Tweet',
] as const;

/**
 Whether a slice embeds the same media on both sides.

 A MARKER ON BOTH SIDES IS THE SIGNAL, not one on either. Media present only in
 the translation is content the original lacks entirely, and media present only
 in the original is something the translation dropped. It is the SAME component
 on both sides, with more prose beside it on one, that says the prose was read
 out of the picture.

 ONE SLICE MATCHING IS A SUSPICION, NOT A VERDICT. A translator may embed an
 image and independently move a passage across the same boundary. What this
 earns is a hand-check before the candidate is counted as relocation.

 @param sourceText - original text of this slice

 @param targetText - translated text of this slice

 @returns Whether a transcription could explain a surplus here

 @example
 ```ts
 const suspect = sharesMedia({ sourceText, targetText, },);
 ```
 */
export function sharesMedia(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): boolean {
  return MEDIA_MARKERS.some(function onBothSides(marker,) {
    if (!sourceText.includes(marker,))
      return false;
    return targetText.includes(marker,);
  },);
}

//endregion Transcription suspect
