import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ConsistencyOccurrenceView, } from './consistency-sheet.ts';
import type { ConsistencyEnglish, } from './consistency-wire.ts';

//region Consistency shown notes
// THE CONSISTENCY JUDGES ARE SHOWN THE ENGLISH WITHOUT THE NOTES THE ARCHIVE'S
// TRANSLATORS LEFT EACH OTHER (`withoutTranslatorsNotes`), as every other judging
// sheet is: no reader sees one, so a note may be neither the difference between
// two renderings nor a word of the one chosen. Only what is SHOWN is cut. The
// judges' quotes are located in the stored English by the code that counts them
// (`consistency-count.ts`), so a quote of words that stand together on the sheet
// is found as before, and one across a place a note was cut from finds nothing.
// The original's own comments stay in view: a comment the slice's original holds
// is kept wherever the English repeats it.

/**
 The original text whose comments the English of these places may repeat as its
 own: the whole original of each place's slice where the pass hands it over,
 otherwise the paragraph the sheet shows for it.

 @param views - places the sheet shows

 @returns Their originals, joined

 @example
 ```ts
 const original = originalOfViews({ views, },);
 ```
 */
export function originalOfViews({ views, }: { readonly views: readonly ConsistencyOccurrenceView[]; },): string {
  return views
    .map(function originalOf(view,): string {
      return view.sliceSource ?? view.paragraph;
    },)
    .join('\n',);
}

/**
 English as the judges read it, without the notes between translators.

 @param english - the place's English as it stands

 @param original - original whose comments the English may repeat as its own

 @returns The same kind of English, its text cut where it carries text

 @example
 ```ts
 const shown = englishWithoutNotes({ english: { kind: 'absent', }, original: '', },);
 ```
 */
export function englishWithoutNotes(
  {
    english,
    original,
  }: {
    readonly english: ConsistencyEnglish;
    readonly original: string;
  },
): ConsistencyEnglish {
  return (english.kind === 'absent')
    ? english
    : {
      kind: 'present',
      text: withoutTranslatorsNotes({
        text: english.text,
        original,
      },),
    };
}

/**
 Places as the judges read them: each place's English without the notes
 between translators, everything else as it stands.

 @param views - places the sheet shows

 @returns The same places in the same order, each with its English cut

 @example
 ```ts
 const shown = viewsWithoutNotes({ views, },);
 ```
 */
export function viewsWithoutNotes(
  { views, }: { readonly views: readonly ConsistencyOccurrenceView[]; },
): readonly ConsistencyOccurrenceView[] {
  /**
   Originals of every place together, which the English of any place may repeat.
   */
  const original = originalOfViews({ views, },);
  return views.map(function shownView(view,): ConsistencyOccurrenceView {
    return {
      ...view,
      english: englishWithoutNotes({
        english: view.english,
        original,
      },),
    };
  },);
}

//endregion Consistency shown notes
