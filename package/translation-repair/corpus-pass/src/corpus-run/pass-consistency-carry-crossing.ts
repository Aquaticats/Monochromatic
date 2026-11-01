import type {
  CarryBlock,
  CarryEdgeWords,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { isLetterOrDigitOfAnyScript, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Pass consistency carry crossing
// THE WORDS OF A PLACE THAT RUNS OVER THE EDGE OF A BLOCK AND MUST STAY AS THEY ARE.
// A place whose words cross a paragraph boundary lies in no single block, so a writer
// that rewrites the paragraph it reaches into (`pass-consistency-carry-units.ts`)
// could change the stretch of the words that lies inside that paragraph while the
// record still says the place kept its words. Four kinds of place of the group must
// keep them: the chosen place, a place the majority keeps for its context, a place
// that already reads as the chosen place does, and a receiving place whose plan is
// `unavailable` (no edit addresses it, so it is recorded `kept-current`). The
// stretch of each that lies inside the paragraph must come out of the rewrite where it
// stood: at the start of the paragraph where the place crosses in, at its end where it
// crosses out. Words of other places that must stay are listed with the unit and
// counted anywhere in a paragraph (`keptWordsLost` in
// `pass-consistency-carry-candidates.ts`), which cannot say that a stretch stayed at an
// edge, so this module reads the edge.
//
// WHAT THIS DOES NOT LIST. A receiving place whose plan is `replace` is not listed:
// its words are replaced, and a block that overlaps them is not writable. The located
// words of a place of another group are not listed either: a block replacement that
// changes them is left out where all the groups' replacements meet
// (`pass-consistency-apply-located.ts`). That test skips the places of the replacement's
// own group, which is why the places of the own group are listed here.

/**
 The stretches of words of places that must stay as they are and cross an edge of a
 block, each with the edge of the block it must stay at.

 @param block - the block a writer would rewrite

 @param spans - where the words of the places that must stay stand in the slice's English, with the words

 @returns One entry for each edge of the block a span reaches over, in the order of the spans, the first edge first; none for a span that lies wholly inside or wholly outside the block

 @example
 ```ts
 const crossing = crossingWordsOf({ block, spans: [{ start: 4, core: 'put the kitten\n\nto sleep', },], },);
 ```
 */
export function crossingWordsOf(
  {
    block,
    spans,
  }: {
    readonly block: CarryBlock;
    readonly spans: readonly {
      readonly start: number;
      readonly core: string;
    }[];
  },
): readonly CarryEdgeWords[] {
  return spans.flatMap(function edgesOf({
    start,
    core,
  },): readonly CarryEdgeWords[] {
    /**
     Exclusive offset where the span ends.
     */
    const end = start + core.length;
    /**
     Whether the span comes in over the start of the block.
     */
    const crossesIn = (start < block.start) && (end > block.start);
    /**
     Whether the span goes out over the end of the block.
     */
    const crossesOut = (start < block.end) && (end > block.end);
    return [
      ...(crossesIn
        ? [
          {
            edge: 'start',
            words: block.text
              .slice(
              0,
              Math.min(
                end,
                block.end,
              ) - block.start,
              ),
          } as const,
        ]
        : []),
      ...(crossesOut
        ? [
          {
            edge: 'end',
            words: block.text
              .slice(Math.max(
                start,
                block.start,
              ) - block.start,),
          } as const,
        ]
        : []),
    ];
  },);
}

/**
 The part of a held receiving place that the stretches are read from.
 */
type CrossingHeld = {
  /**
   The receiving place.
   */
  readonly receiver: {
    /**
     The place, whose slice the words stand in.
     */
    readonly place: {
      readonly slice: {
        readonly sliceIndex: number;
      };
    };

    /**
     How the chosen words move into the place; only `unavailable` leaves its words with no edit and so to stay.
     */
    readonly plan: {
      readonly kind: string;
    };

    /**
     Where the place's words stand in its slice's English.
     */
    readonly rendering: {
      readonly start: number;
      readonly core: string;
    };
  };
};

/**
 The words of a place of the group that a carry must leave as they are, where they stand.
 */
type CrossingKept = {
  /**
   Slice whose English holds the words.
   */
  readonly sliceIndex: number;

  /**
   Offset in that English where the words start.
   */
  readonly start: number;

  /**
   The words.
   */
  readonly core: string;

  /**
   Present when the words are the located words of a place of another group, which the apply stage holds, and absent when
   they are those of a place of this group.
   */
  readonly ofOtherGroup?: true;
};

/**
 The stretches of a unit's block that belong to places of the group outside the unit
 whose words must stay as they are and run over an edge of the block: receiving places
 whose plan is `unavailable`, and the chosen place, the places the majority keeps for
 their context and the places that already read as the chosen place does.

 @param block - the block of the unit

 @param members - the unit's own receiving places

 @param held - every receiving place of the group

 @param kept - the words of the places that stay, of this group and, marked, of the other groups of the page

 @returns One entry for each edge of the block such a place reaches over, in the page order of the places

 @example
 ```ts
 const crossing = crossingOfUnit({ block, members, held, kept, },);
 ```
 */
export function crossingOfUnit(
  {
    block,
    members,
    held,
    kept,
  }: {
    readonly block: CarryBlock;
    readonly members: readonly CrossingHeld[];
    readonly held: readonly CrossingHeld[];
    readonly kept: readonly CrossingKept[];
  },
): readonly CarryEdgeWords[] {
  /**
   The slice the unit stands in.
   */
  const sliceIndex = members[0]
    ?.receiver
    .place
    .slice
    .sliceIndex;
  /**
   The words of the receiving places beside the unit that have no splice.
   */
  const withoutSplice = held
    .filter(function hasNoSpliceBesideTheUnit({ receiver, },): boolean {
      return (receiver.place
        .slice
        .sliceIndex
        === sliceIndex)
        && (receiver.plan
          .kind
          === 'unavailable')
        && (!members.some(function sameReceiver(member,): boolean {
          return member.receiver === receiver;
        },));
    },)
    .map(function spanOfReceiver({ receiver, },): {
      readonly start: number;
      readonly core: string;
    } {
      return {
        start: receiver.rendering
          .start,
        core: receiver.rendering
          .core,
      };
    },);
  /**
   The words of this group's other places in the slice that stay as they are.
   */
  const staying = kept
    .filter(function staysInTheSlice(
      {
        sliceIndex: wordsSlice,
        ofOtherGroup,
      },
    ): boolean {
      return (wordsSlice === sliceIndex) && (ofOtherGroup !== true);
    },)
    .map(function spanOfKept(
      {
        start,
        core,
      },
    ): {
      readonly start: number;
      readonly core: string;
    } {
      return {
        start,
        core,
      };
    },);
  /**
   Both lists together, in the order of the slice.
   */
  const spans = [
    ...withoutSplice,
    ...staying,
  ].toSorted(function inPageOrder(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
  return crossingWordsOf({
    block,
    spans,
  },);
}

/**
 The characters read as white space when words are compared: the space, the tab,
 the line breaks and the no-break space.
 */
const WHITE_SPACE: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
  '\u{00A0}',
  '\u{2028}',
  '\u{2029}',
],);

/**
 A text with each run of white space read as one space and none at its edges, so
 words a writer laid out again on other lines are the same words. One pass over
 the characters, never a scan for each.

 @param text - the words

 @returns The words in one line

 @example
 ```ts
 const line = oneLineOf({ text: 'put the kitten\n\nto sleep', },); // 'put the kitten to sleep'
 ```
 */
function oneLineOf({ text, }: { readonly text: string; },): string {
  /**
   The characters kept, with one space for each run of white space, mutated only inside this function.
   */
  const kept: string[] = [];
  /**
   Whether the character before was white space, so that a run gives one space and the start gives none.
   */
  const reading = { spaced: true, };
  for (const character of text) {
    if (WHITE_SPACE.has(character,)) {
      if (!reading.spaced)
        kept.push(' ',);
      reading.spaced = true;
    }
    else {
      kept.push(character,);
      reading.spaced = false;
    }
  }
  if (reading.spaced)
    kept.pop();
  return kept.join('',);
}

/**
 Whether a writer's paragraph still holds a stretch of a crossing place's words at
 the edge it stood at, character for character apart from white space, and not as
 the beginning or the end of a longer word.

 @param paragraph - the writer's paragraph as it would ship

 @param crossing - the stretch and the edge

 @returns Whether the paragraph starts (for a start edge) or ends (for an end edge) with the stretch

 @example
 ```ts
 const held = holdsAtEdge({ paragraph: 'to sleep. Fill the bowl.', crossing: { edge: 'start', words: 'to sleep', }, },);
 ```
 */
function holdsAtEdge(
  {
    paragraph,
    crossing,
  }: {
    readonly paragraph: string;
    readonly crossing: CarryEdgeWords;
  },
): boolean {
  /**
   The paragraph in one line.
   */
  const line = oneLineOf({ text: paragraph, },);
  /**
   The stretch in one line.
   */
  const words = oneLineOf({ text: crossing.words, },);
  if (crossing.edge === 'start') {
    return line.startsWith(words,)
      && (!(isLetterOrDigitOfAnyScript({ character: Array.from(words,)
        .at(-1,)
        ?? '', },)
        && isLetterOrDigitOfAnyScript({ character: Array.from(line.slice(words.length,),)[0] ?? '', },)));
  }
  return line.endsWith(words,)
    && (!(isLetterOrDigitOfAnyScript({ character: Array.from(words,)[0] ?? '', },)
      && isLetterOrDigitOfAnyScript({ character: Array.from(line.slice(
        0,
        line.length - words.length,
      ),)
        .at(-1,)
        ?? '', },)));
}

/**
 The stretches of crossing places' words that a writer's paragraph no longer holds
 at the edge of the block they stood at.

 @param paragraph - the writer's paragraph as it would ship

 @param crossing - the stretches and their edges

 @returns The entries the paragraph does not hold at their edge, in order

 @example
 ```ts
 const lost = crossingWordsLost({ paragraph, crossing: [{ edge: 'start', words: 'to sleep', },], },);
 ```
 */
export function crossingWordsLost(
  {
    paragraph,
    crossing,
  }: {
    readonly paragraph: string;
    readonly crossing: readonly CarryEdgeWords[];
  },
): readonly CarryEdgeWords[] {
  return crossing.filter(function isLost(entry,): boolean {
    return !holdsAtEdge({
      paragraph,
      crossing: entry,
    },);
  },);
}

//endregion Pass consistency carry crossing
