import type { ContainmentEdges, } from './consistency-contains-words.ts';
import { locateWords, } from './consistency-fold.ts';
import { carriesOrCutsMarkup, } from './consistency-markup.ts';
import type { CodePadding, } from './consistency-markup-ranges.ts';
import {
  agreedSlotWordsOf,
  type QuotesByLabel,
  quotesByLabelOf,
  type SlotAgreement,
} from './consistency-majority.ts';
import type { ConsistencyVoice, } from './consistency-count-types.ts';
import type { PageEnglish, } from './consistency-page-english.ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency slot in span
// A SLOT IS NO CONDITION OF A SPAN. A frame's slot words are agreed the way its span is, by a strict majority of all the usable
// ballots, each counting when the quote it gave the slot beside contains the located words at the edges of words (the place is
// located by containment, so a slot given beside a longer quote lies inside the located words just as well; a recount of an
// artifact written before that rule counts only a quote that is the words, `slotAgreement`), and they must stand once inside the
// span the place was located by, as the English holds them; a slot the ballots do not agree on, that shows a reader nothing, that stands nowhere or twice in the span, that is all of
// the span or that cuts markup is no slot, and the place is located without one (`consistency-count.ts`), which leaves the code
// splice unavailable for it and the writers to carry the slot.

/**
 The slot words a strict majority gave inside a located span, as the English
 holds them.

 @param voices - usable ballots

 @param label - the place's label

 @param words - the folded words the span was agreed by

 @param span - the English's own words of the span

 @param quotesByLabel - the quote entries of these ballots by label, which a caller that asks many places reads once for the
 group; read from the ballots where absent

 @param page - the English of the whole page the span belongs to and where the span starts in it, whose link and footnote
 definitions a reference inside the span resolves against (n173); the span alone is the page where absent

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`) in place of those read from `page`, for a
 reader of a recorded artifact, which stores the definitions a slot was read with; read from `page` where absent

 @param slotAgreement - which quotes a slot given beside counts for the words: any quote that contains them, or only a quote that
 is them, as a recount of an artifact written under the earlier rule reads it; containing quotes where absent

 @param edges - which joints of a longer word bar a quote from containing the words, under containing quotes: the locator's test of a
 joint, or none but the letters beside the match, as a recount of an artifact written before the locator's test counted reads it;
 the locator's where absent

 @param codePadding - what pads the content of inline code when the slot is located in the span: a space and a line ending as
 CommonMark reads it, or a literal space alone as a recount of an artifact written before line endings padded code reads it; a space
 and a line ending where absent

 @returns The slot as the span holds it and its offset in the span, as a list of
 zero or one; none when the ballots do not agree on a slot, it shows a reader
 nothing, it does not stand once inside the span, it is all of the span, or it
 cuts markup

 @example
 ```ts
 const [slot,] = slotInSpan({ voices, label: 'OCCURRENCE 1', words: 'So she was a girl, wasn\'t she', span: 'So she was a girl, wasn’t she', },);
 ```
 */
export function slotInSpan(
  {
    voices,
    label,
    words,
    span,
    quotesByLabel = quotesByLabelOf({ voices, },),
    page,
    definitions,
    slotAgreement = 'containing-quote',
    edges,
    codePadding,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly label: string;
    readonly words: string;
    readonly span: string;
    readonly quotesByLabel?: QuotesByLabel;
    readonly page?: PageEnglish;
    readonly definitions?: string;
    readonly slotAgreement?: SlotAgreement;
    readonly edges?: ContainmentEdges;
    readonly codePadding?: CodePadding;
  },
): readonly {
  readonly slot: string;
  readonly slotStart: number;
}[] {
  /**
   The slot words a strict majority gave beside the agreed words.
   */
  const [slotWords,] = agreedSlotWordsOf({
    voices,
    label,
    quotesByLabel,
    words,
    slotAgreement,
    ...((edges === undefined) ? {} : { edges, }),
  },);
  if ((slotWords === undefined) || rendersAsNothing({ text: slotWords, },))
    return [];
  /**
   Where the slot words stand in the span.
   */
  const inside = locateWords({
    haystack: span,
    words: slotWords,
    apostrophes: 'outside',
    ...((page === undefined) ? {} : { page, }),
    ...((definitions === undefined) ? {} : { definitions, }),
    ...((codePadding === undefined) ? {} : { codePadding, }),
  },);
  if (inside.kind !== 'once')
    return [];
  /**
   The slot as the span holds it.
   */
  const slot = span.slice(
    inside.start,
    inside.end,
  );
  if ((slot === span) || carriesOrCutsMarkup({ text: slot, },))
    return [];
  return [
    {
      slot,
      slotStart: inside.start,
    },
  ];
}

//endregion Consistency slot in span
