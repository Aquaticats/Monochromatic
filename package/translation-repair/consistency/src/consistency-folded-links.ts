import type { OffsetFoldedText, } from './consistency-fold-offsets.ts';
import type {
  LabelledLink,
  MarkupRange,
} from './consistency-markup-ranges.ts';
import { rememberedViewOf, } from './consistency-prose-text.ts';

//region Consistency folded links
// THE LINKS OF AN ENGLISH THAT THE FOLD READS AROUND, at the offsets of the fold (n166 Z9, n168, n169). The word reading of the fold
// (`consistency-word-joints.ts`, `consistency-closing-marks.ts`) asks what stands beside a stand of some words and what stands before
// an apostrophe. A link shows a reader its label and nothing else (`consistency-link-label.ts`), so what stands beside a link is read
// as the label would stand there: a link whose label shows nothing (`[](u)`, `![](u)`) neither ends a word nor starts one, and the text
// on each side of it meets as if it were not there; a link whose label shows closing marks and nothing else (`[\]](u)`, `[”](u)`)
// closes a run of closing marks as those marks do; after a link whose label shows marks and no letter, a mark that starts a word. Which
// stretch is a link, and what its label shows, is the parse's to say (`rememberedViewOf`), read once for an English and not again for
// the next place that asks, and carried here from the offsets of the English to the offsets of the fold.
//
// A LINK WITH A LABEL IS READ AS ITS LABEL (n173): the opening square bracket and the resource of a link whose label shows letters,
// digits or marks (`[kit](u)`, `[-](u "t")`, `[kit][r]`) show a reader nothing, so each is a stretch the word reading steps over
// (`hiddenStarts`, `hiddenEnds`) and the label between them is read as the text it is. A link with no label or a label of closing
// marks keeps the reading it has (`emptyStarts`, `closingSpans`); an image is not listed, since an image is read whole.
//
// A LINK WHOSE LABEL ENDS IN AN OPEN MARK CLOSES NOTHING FOR AN APOSTROPHE after it. The bracket that closes `[kit!](u)`, `[Kit.](u)` or
// `[kit ](u)` is a closing bracket in plain text only when the label ends in one (`[kit)](u)`): a reader sees `kit!'s`, and `!` closes
// nothing. `openLabelEnds` names the last unit of such a link for the backward reading of an apostrophe (`consistency-word-joints.ts`)
// and `openLabelStretches` its two hidden stretches for the forward one (`consistency-link-twin.ts`). A label that ends in a letter or a
// digit keeps its reading, since the letter joins the word to the apostrophe whatever the link writes after it.
//
// THE PARSE IS ASKED ONLY WHERE A CLOSING SQUARE BRACKET STANDS IN THE FOLD, since every link has one and the fold asks nothing of a
// link otherwise. An English the page grammar refuses has no parse, so no link is read in it. A reference with an empty label is a
// link only where a definition for it stands in the English or in the definitions of the page that the caller hands over.

/**
 The links the fold reads around, at the offsets of the fold.
 */
export type FoldedLinks = {
  /**
   For the last unit of each link whose label shows nothing, the offset of its first unit.
   */
  readonly emptyStarts: ReadonlyMap<number, number>;

  /**
   For the first unit of each link whose label shows nothing, the offset after its last unit.
   */
  readonly emptyEnds: ReadonlyMap<number, number>;

  /**
   For the first unit of each link whose label shows closing round or square brackets and closing curly double quotation marks and
   nothing else, the offset after its last unit.
   */
  readonly closingSpans: ReadonlyMap<number, number>;

  /**
   The offsets of the last unit of each link whose label shows marks and spaces and no letter or digit, and no closing mark last.
   */
  readonly markOnlyEnds: ReadonlySet<number>;

  /**
   For the last unit of each stretch of a link with a label that shows a reader nothing (its opening square bracket, its closing
   bracket with the destination, the title or the reference), the offset of the stretch's first unit (n173).
   */
  readonly hiddenStarts: ReadonlyMap<number, number>;

  /**
   For the first unit of each stretch of a link with a label that shows a reader nothing, the offset after its last unit (n173).
   */
  readonly hiddenEnds: ReadonlyMap<number, number>;

  /**
   The offsets of the last unit of each link with a label that ends in an open mark: a label whose last character is neither a
   closing round or square bracket or closing curly double quotation mark nor a letter or a digit (`[kit!](u)`, `[Kit.](u)`,
   `[kit ](u)`). The `)` or `]` that closes such a link is no closing bracket to the apostrophe after it, as the label would
   be none in plain text (`kit!'s`).
   */
  readonly openLabelEnds: ReadonlySet<number>;

  /**
   For the first unit of each of the two stretches (the opening square bracket and the resource) of a link with a label that ends
   in an open mark, the offset after its last unit. The reading of closing marks writes such a link as its label, so that the
   text it reads is the text a reader sees.
   */
  readonly openLabelStretches: ReadonlyMap<number, number>;
};

/**
 What an English with no link read holds: no link.
 */
export const NO_FOLDED_LINKS: FoldedLinks = {
  emptyStarts: new Map(),
  emptyEnds: new Map(),
  closingSpans: new Map(),
  markOnlyEnds: new Set(),
  hiddenStarts: new Map(),
  hiddenEnds: new Map(),
  openLabelEnds: new Set(),
  openLabelStretches: new Map(),
};

/**
 One stretch that shows a reader nothing, by the folded units of its first and last characters.
 */
type HiddenSpan = {
  /**
   The folded unit of the first character.
   */
  readonly first: number;

  /**
   The folded unit of the last character.
   */
  readonly last: number;
};

/**
 One entry of a map from a folded unit to a folded unit or the offset after one.
 */
type UnitEntry = readonly [
  number,
  number,
];

/**
 The links of an English at the offsets of its fold.

 @param english - the English as it was given, which the parse reads the links from

 @param folded - the English in the fold, with the offsets of the English each folded unit stands for

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone: a reference with an empty label
 is a link only where a definition of its label stands in the English or here

 @returns The links whose label shows nothing, closing marks only, or marks, none where no closing square bracket stands in the fold
 or the grammar refuses the English

 @example
 ```ts
 const { emptyStarts, } = foldedLinksOf({ english: 'a [](u)\'kit', folded: foldWithOffsets({ text: 'a [](u)\'kit', },), },); // [[6, 2]]
 ```
 */
export function foldedLinksOf(
  {
    english,
    folded,
    definitions = '',
  }: {
    readonly english: string;
    readonly folded: OffsetFoldedText;
    readonly definitions?: string;
  },
): FoldedLinks {
  /**
   The English in the fold, and where in the English each folded unit starts.
   */
  const {
    folded: text,
    starts,
  } = folded;
  if (!text.includes(']',))
    return NO_FOLDED_LINKS;
  /**
   What the parse makes of the English.
   */
  const {
    emptyLinks,
    closingLinks,
    markOnlyLinks,
    labelledLinks,
  } = rememberedViewOf({
    english,
    definitions,
  },);
  /**
   The stretches of each kind of link the fold reads around.
   */
  const kinds = [
    emptyLinks,
    closingLinks,
    markOnlyLinks,
  ];
  if (kinds.every(function holdsNone(stretches,): boolean {
    return stretches.length === 0;
  },) && (labelledLinks.length === 0))
    return NO_FOLDED_LINKS;
  /**
   The first folded unit that stands for each offset of the English, mutated only inside this function.
   */
  const unitAt = new Map<number, number>();
  starts.forEach(function isFirstOf(
    start,
    unit,
  ): void {
    if (!unitAt.has(start,)) {
      unitAt.set(
        start,
        unit,
      );
    }
  },);
  /**
   The empty links by their last unit, mutated only inside this function.
   */
  const emptyStarts = new Map<number, number>();
  /**
   The empty links by their first unit, mutated only inside this function.
   */
  const emptyEnds = new Map<number, number>();
  for (const link of emptyLinks) {
    /**
     The folded unit of the first character of the link.
     */
    const first = unitAt.get(link.start,);
    /**
     The folded unit of the last character of the link.
     */
    const last = unitAt.get(link.end - 1,);
    // A link whose first or last character the fold holds no unit for is left out, and so is one whose first unit stands after its last.
    if ((first === undefined) || (last === undefined))
      continue;
    if (first > last)
      continue;
    emptyStarts.set(
      last,
      first,
    );
    emptyEnds.set(
      first,
      last + 1,
    );
  }
  /**
   The links of closing marks by their first unit, mutated only inside this function.
   */
  const closingSpans = new Map<number, number>();
  for (const link of closingLinks) {
    /**
     The folded unit of the first character of the link.
     */
    const first = unitAt.get(link.start,);
    /**
     The folded unit of the last character of the link.
     */
    const last = unitAt.get(link.end - 1,);
    if ((first === undefined) || (last === undefined))
      continue;
    if (first > last)
      continue;
    closingSpans.set(
      first,
      last + 1,
    );
  }
  /**
   The last units of the links of marks, mutated only inside this function.
   */
  const markOnlyEnds = new Set<number>();
  for (const link of markOnlyLinks) {
    /**
     The folded unit of the last character of the link.
     */
    const last = unitAt.get(link.end - 1,);
    if (last !== undefined)
      markOnlyEnds.add(last,);
  }
  /**
   The folded units of the stretches of some links with a label.

   @param links - links with a label, each with the two stretches that show a reader nothing

   @returns The stretches the fold holds units for, in the order of the links
   */
  function hiddenSpansOf({ links, }: { readonly links: readonly LabelledLink[]; },): readonly HiddenSpan[] {
    return links
      .flatMap(function stretchesOfLink({
        opener,
        resource,
      },): readonly MarkupRange[] {
        return [
          opener,
          resource,
        ];
      },)
      .flatMap(function unitsOfStretch(stretch,): readonly HiddenSpan[] {
        /**
         The folded unit of the first character of the stretch.
         */
        const first = unitAt.get(stretch.start,);
        /**
         The folded unit of the last character of the stretch.
         */
        const last = unitAt.get(stretch.end - 1,);
        if ((first === undefined) || (last === undefined))
          return [];
        return (first > last)
          ? []
          : [
            {
              first,
              last,
            },
          ];
      },);
  }
  /**
   The stretches of the links with a label that show a reader nothing, by the folded units of their first and last characters. One
   the fold holds no unit for, or whose first unit stands after its last, is left out. A map made from the pairs is never written
   over a key it has not read.
   */
  const hiddenSpans = hiddenSpansOf({ links: labelledLinks, },);
  /**
   The stretches of the links whose label ends in an open mark.
   */
  const openSpans = hiddenSpansOf({
    links: labelledLinks.filter(function endsOpen({ endsOpen: open, },): boolean {
      return open;
    },),
  },);
  /**
   The last units of the links whose label ends in an open mark, mutated only inside this function.
   */
  const openLabelEnds = new Set<number>();
  for (
    const {
      resource,
      endsOpen: open,
    } of labelledLinks
  ) {
    if (!open)
      continue;
    /**
     The offset of the last character of the link.
     */
    const lastCharacter = resource.end - 1;
    /**
     The folded unit of the last character of the link.
     */
    const last = unitAt.get(lastCharacter,);
    if (last !== undefined)
      openLabelEnds.add(last,);
  }
  return {
    emptyStarts,
    emptyEnds,
    closingSpans,
    markOnlyEnds,
    hiddenStarts: new Map(hiddenSpans.map(function byLast({
      first,
      last,
    },): UnitEntry {
      return [
        last,
        first,
      ];
    },),),
    hiddenEnds: new Map(hiddenSpans.map(function byFirst({
      first,
      last,
    },): UnitEntry {
      return [
        first,
        last + 1,
      ];
    },),),
    openLabelEnds,
    openLabelStretches: new Map(openSpans.map(function byFirst({
      first,
      last,
    },): UnitEntry {
      return [
        first,
        last + 1,
      ];
    },),),
  };
}

//endregion Consistency folded links
