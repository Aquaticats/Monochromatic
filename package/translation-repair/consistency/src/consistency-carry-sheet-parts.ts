import {
  type ConsistencyMarks,
  type ConsistencyOccurrenceView,
  markedParagraph,
} from './consistency-sheet.ts';

//region Consistency carry sheet parts
// WHAT THE WRITER'S SHEET AND THE CHOICE SHEET BOTH SHOW, built once. The
// judges who choose among the texts a block might have are shown what the
// writers of those texts were shown (`mistake-prevention.md`, "What each sheet
// is shown"): the receiving places' ORIGINAL with the blocks around it, the
// chosen place's ORIGINAL, its English paragraph, the words that render the
// repeated wording there and, for a frame, the English slot, and the words of
// the group's other places that no candidate may change.

/**
 A receiving place as a carry sheet shows it.

 @example
 ```ts
 const place: CarryReceivingPlace = { view, span: 'the kitten naps', };
 ```
 */
export type CarryReceivingPlace = {
  /**
   The place as the consistency sheet shows it: its paragraph and where the
   repeated wording stands in it.
   */
  readonly view: ConsistencyOccurrenceView;

  /**
   The words of the place's English that render the repeated wording.
   */
  readonly span: string;
};

/**
 The chosen place as a carry sheet shows it.

 @example
 ```ts
 const chosen: CarryChosenPlace = { view, span: 'The cat naps', paragraph: 'The cat naps. It is warm.', };
 ```
 */
export type CarryChosenPlace = CarryReceivingPlace & {
  /**
   The English paragraph of the chosen place that holds those words.
   */
  readonly paragraph: string;

  /**
   For a frame whose chosen place has a located slot: the English words that
   render the slot there.
   */
  readonly slot?: string;
};

/**
 The label of one receiving place on a sheet: bare for the only one, numbered
 from one when a block holds several.

 @param name - what the sheet calls a receiving place, such as `THIS PLACE`

 @param position - zero-based position among the block's receiving places

 @param count - how many receiving places the block has

 @returns The label

 @example
 ```ts
 const label = receivingLabel({ name: 'THIS PLACE', position: 1, count: 2, },); // 'THIS PLACE 2'
 ```
 */
export function receivingLabel(
  {
    name,
    position,
    count,
  }: {
    readonly name: string;
    readonly position: number;
    readonly count: number;
  },
): string {
  return (count === 1) ? name : `${name} ${String(position + 1,)}`;
}

/**
 The fenced blocks that show one place's ORIGINAL to a model.

 @param view - the place

 @param name - what the blocks call the place, such as `THIS PLACE`

 @param fence - fence no shown text contains

 @param marks - marks the paragraph carries

 @returns The blocks of its ORIGINAL and its slot where it has one, each ending in a line break

 @example
 ```ts
 const blocks = originalBlocksOf({ view, name: 'THIS PLACE', fence, marks, },);
 ```
 */
export function originalBlocksOf(
  {
    view,
    name,
    fence,
    marks,
  }: {
    readonly view: ConsistencyOccurrenceView;
    readonly name: string;
    readonly fence: string;
    readonly marks: ConsistencyMarks;
  },
): string {
  /**
   The place's slot, for a frame.
   */
  const slot = (view.slotSource === undefined)
    ? ''
    : `${fence} ${name}: SLOT IN THE ORIGINAL ${fence}\n${view.slotSource}\n`;
  return `${fence} ${name}: ORIGINAL ${fence}\n${markedParagraph({
    view,
    marks,
  },)}\n${slot}`;
}

/**
 The fenced blocks that show the words each receiving place's English renders
 the repeated wording with, which the carry replaces. Both sheets of a block's
 carry show them, under the same labels: bare for the only receiving place,
 numbered from one when a block holds several.

 @param receiving - the receiving places of the block, in page order

 @param fence - fence no shown text contains

 @returns The blocks, each ending in a line break

 @example
 ```ts
 const blocks = replaceBlocksOf({ receiving, fence, },);
 ```
 */
export function replaceBlocksOf(
  {
    receiving,
    fence,
  }: {
    readonly receiving: readonly CarryReceivingPlace[];
    readonly fence: string;
  },
): string {
  return receiving
    .map(function replaceBlockOf(
      place,
      position,
    ): string {
      return `${fence} ${
        receivingLabel({
          name: 'THE WORDS TO REPLACE',
          position,
          count: receiving.length,
        },)
      } ${fence}\n${place.span}\n`;
    },)
    .join('',);
}

/**
 The fenced blocks of the original blocks before and after the one that holds
 the receiving places.

 @param before - the block before, where the slice has one

 @param after - the block after, where the slice has one

 @param fence - fence no shown text contains

 @returns The blocks, each ending in a line break, nothing for a block the slice lacks

 @example
 ```ts
 const around = aroundBlocksOf({ after: '第二段', fence, },);
 ```
 */
export function aroundBlocksOf(
  {
    before,
    after,
    fence,
  }: {
    readonly before?: string;
    readonly after?: string;
    readonly fence: string;
  },
): string {
  return [
    (before === undefined) ? '' : `${fence} ORIGINAL BEFORE THIS PLACE ${fence}\n${before}\n`,
    (after === undefined) ? '' : `${fence} ORIGINAL AFTER THIS PLACE ${fence}\n${after}\n`,
  ].join('',);
}

/**
 The fenced blocks that show the chosen place's English to a model: the words
 that render the repeated wording there, the paragraph that holds them and, for
 a frame with a located slot, the English slot.

 @param chosen - the chosen place

 @param fence - fence no shown text contains

 @returns The blocks, each ending in a line break

 @example
 ```ts
 const blocks = chosenEnglishBlocksOf({ chosen, fence, },);
 ```
 */
export function chosenEnglishBlocksOf(
  {
    chosen,
    fence,
  }: {
    readonly chosen: CarryChosenPlace;
    readonly fence: string;
  },
): string {
  /**
   The English slot, for a frame whose slot was located.
   */
  const slot = (chosen.slot === undefined)
    ? ''
    : `${fence} CHOSEN PLACE: ENGLISH SLOT ${fence}\n${chosen.slot}\n`;
  return `${fence} CHOSEN RENDERING ${fence}\n${chosen.span}\n${fence} CHOSEN PLACE: ENGLISH PARAGRAPH ${fence}\n${chosen.paragraph}\n${slot}`;
}

/**
 The stretch of a place's words that lies inside a paragraph although the place
 runs over an edge of it, and the edge the stretch must stay at.

 @example
 ```ts
 const crossing: CarryEdgeWords = { edge: 'start', words: 'to sleep', };
 ```
 */
export type CarryEdgeWords = {
  /**
   The edge of the paragraph the stretch stands at: `start` where the place comes in
   over the start of the paragraph, `end` where it goes out over the end.
   */
  readonly edge: 'start' | 'end';

  /**
   The stretch of the place's words inside the paragraph.
   */
  readonly words: string;
};

/**
 The line that says where a stretch of a crossing place's words must stay, in the
 list of words to keep.

 @param crossing - the stretch and its edge

 @returns The words followed by what is asked of them, in one entry of the list

 @example
 ```ts
 const entry = crossingEntryOf({ crossing: { edge: 'start', words: 'to sleep', }, },);
 ```
 */
function crossingEntryOf({ crossing, }: { readonly crossing: CarryEdgeWords; },): string {
  return (crossing.edge === 'start')
    ? `${crossing.words}\n(these are the FIRST words of the paragraph: the place they belong to runs on from the previous paragraph, so they stay the first words of your paragraph, exactly as they are)`
    : `${crossing.words}\n(these are the LAST words of the paragraph: the place they belong to runs on into the next paragraph, so they stay the last words of your paragraph, exactly as they are)`;
}

/**
 The fenced block naming words of other places that must stay as they are.

 @param keeping - the words, one entry for each place

 @param crossing - stretches of places that run over an edge of the paragraph, each of which must stay at that edge

 @param fence - fence no shown text contains

 @returns The block, nothing when there are no words, ending in a line break

 @example
 ```ts
 const block = keepingBlockOf({ keeping: ['The cat naps',], crossing: [], fence, },);
 ```
 */
export function keepingBlockOf(
  {
    keeping,
    crossing = [],
    fence,
  }: {
    readonly keeping: readonly string[];
    readonly crossing?: readonly CarryEdgeWords[];
    readonly fence: string;
  },
): string {
  /**
   The entries of the list: the words to keep wherever they stand, then the stretches that stay at an edge.
   */
  const entries = [
    ...keeping,
    ...crossing.map(function entryOf(stretch,): string {
      return crossingEntryOf({ crossing: stretch, },);
    },),
  ];
  return (entries.length === 0)
    ? ''
    : `${fence} WORDS OF OTHER PLACES TO KEEP EXACTLY ${fence}\n${entries.join('\n',)}\n`;
}

//endregion Consistency carry sheet parts
