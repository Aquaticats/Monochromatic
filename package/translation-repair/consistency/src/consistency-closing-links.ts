import { closingJointsInText, } from './consistency-closing-marks.ts';
import {
  type FoldedLinks,
  NO_FOLDED_LINKS,
} from './consistency-folded-links.ts';
import {
  linkTwinOf,
  originalOffsetsOf,
} from './consistency-link-twin.ts';

//region Consistency closing links
// THE CLOSING JOINTS OF AN ENGLISH WITH LINKS IN IT, as a reader sees it (n169). The reading of closing marks before an apostrophe
// (`closingJointsInText`, `consistency-closing-marks.ts`) runs over a folded English one unit at a time. A link shows a reader its
// label and nothing of its brackets, destination or title, so where the parse holds a link that shows nothing or only closing marks
// the reading is made over the text a reader sees (`consistency-link-twin.ts`) and its answer carried back to the offsets of the
// English: `kit[](u)'kit` is read as `kit'kit` and `kit[\]](u)'kit` as `kit]'kit`. A link whose label ends in an open mark, neither a
// closing mark nor a letter or a digit, is read as its label too: `[kit!](u)'s` as `kit!'s`, whose apostrophe no closing bracket
// precedes, where `[kit)](u)'s` keeps the closing bracket its label ends in.

/**
 The offsets of a folded English at which a run of closing marks ends in an apostrophe that a letter or a digit follows, as a
 reader sees the English: a link whose label shows nothing (`kit[](u)'kit`) is not there, and a link whose label shows closing marks
 and nothing else (`kit[\]](u)'kit`, `kit[”](u)'kit`) closes as those marks do, so the offsets are those of the English with each
 link written as its label shows (`kit'kit`, `kit]'kit`), carried back to the offsets of the English, and a stand that ends either
 side of a link that shows nothing ends at the same place. Where no link is given the reading is that of the text alone: one pass
 from the end of the text to its start, a constant number of reads for each unit; where links are given one more pass writes the
 text a reader sees.

 @param folded - the English in the fold

 @param originals - the character the English held before the fold at each UTF-16 unit of it

 @param closing - the offsets in the fold of the units that are the closing delimiter of an emphasis, strong or strikethrough
 stretch by the parse (`consistency-prose-text.ts`); an emphasis character at no such offset closes nothing

 @param links - the links the parse holds, at the offsets of the fold (`foldedLinksOf`, `consistency-folded-links.ts`), none where
 the caller read no link

 @returns The offsets at which a stand may end to lie inside such a word

 @example
 ```ts
 const text = 'Fill the "cat"\'s bowl.';
 const inside = closingJointsOf({ folded: text, originals: Array.from(text,), closing: new Set(), },).has(14,); // true
 ```
 */
export function closingJointsOf(
  {
    folded,
    originals,
    closing,
    links = NO_FOLDED_LINKS,
  }: {
    readonly folded: string;
    readonly originals: readonly string[];
    readonly closing: ReadonlySet<number>;
    readonly links?: FoldedLinks;
  },
): ReadonlySet<number> {
  /**
   The links of the English that show nothing or closing marks, and the stretches of those with a label that ends in an open
   mark, by their first unit.
   */
  const {
    emptyEnds,
    closingSpans,
    openLabelStretches,
  } = links;
  /**
   Whether no link of the English shows a reader anything but its label.
   */
  const noneRewritten = (emptyEnds.size === 0)
    && (closingSpans.size === 0)
    && (openLabelStretches.size === 0);
  if (noneRewritten) {
    return closingJointsInText({
      folded,
      originals,
      closing,
    },);
  }
  /**
   The English as a reader sees it, with the offsets that carry an answer back.
   */
  const twin = linkTwinOf({
    folded,
    originals,
    closing,
    links,
  },);
  return originalOffsetsOf({
    twin,
    found: closingJointsInText({
      folded: twin.folded,
      originals: twin.originals,
      closing: twin.closing,
    },),
  },);
}

//endregion Consistency closing links
