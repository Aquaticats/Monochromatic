/**
 Readers of the sheets the consistency pass shows its seats, for the scripted
 seats of the tests: a sheet is a run of blocks, each opened by a line that
 repeats one fence of equal signs around its label. Cat-themed invention; no
 corpus content appears here.

 @module
 */

//region Sheet reading

/**
 The fences that stand around the label of a block's opening line.
 */
const FENCES_AROUND_LABEL = 2;

/**
 What a seat's sheet is made of.

 @example
 ```ts
 const block: SheetBlock = { label: 'CANDIDATE 1', text: 'The cat naps.', };
 ```
 */
type SheetBlock = {
  /**
   The label between the two fences of the block's opening line.
   */
  readonly label: string;

  /**
   The lines under the opening line, up to the next block's.
   */
  readonly text: string;
};

/**
 The label of a line that opens a block, none for any other line.

 @param line - one line of a sheet

 @returns The label as a list of zero or one

 @example
 ```ts
 const [label,] = openingLabelOf({ line: '===== CANDIDATE 1 =====', },); // 'CANDIDATE 1'
 ```
 */
function openingLabelOf({ line, }: { readonly line: string; },): readonly string[] {
  /**
   The words of the line.
   */
  const words = line.split(' ',);
  /**
   The fence the line opens with.
   */
  const [fence,] = words;
  if ((fence === undefined) || (fence.length === 0)
    || (fence.replaceAll(
      '=',
      '',
    )
      .length
      > 0))
    return [];
  /**
   The label between the fences, absent unless the line ends with the fence it opens with.
   */
  const between = words.slice(
    1,
    -1,
  );
  if ((words.length <= FENCES_AROUND_LABEL) || (words.at(-1,) !== fence))
    return [];
  return [between.join(' ',),];
}

/**
 The blocks of a sheet, in the order it shows them.

 @param sheet - the sheet a seat was shown

 @returns Each block with its label and the text under it

 @example
 ```ts
 const blocks = blocksOf({ sheet, },);
 ```
 */
function blocksOf({ sheet, }: { readonly sheet: string; },): readonly SheetBlock[] {
  /**
   The lines of the sheet with the label each opens, if it opens one.
   */
  const lines = sheet.split('\n',);
  /**
   The label each line opens, as a list of zero or one.
   */
  const opened = lines.map(function labelled(line,): readonly string[] {
    return openingLabelOf({ line, },);
  },);
  return opened.flatMap(function blockOpenedBy(
    opens,
    at,
  ): readonly SheetBlock[] {
    /**
     The label of the block this line opens, none when it opens none.
     */
    const [label,] = opens;
    if (label === undefined)
      return [];
    /**
     Position of the next opening line, none when the block runs to the end.
     */
    const next = opened.findIndex(function opensAfter(
      later,
      position,
    ): boolean {
      return (position > at) && (later.length > 0);
    },);
    return [
      {
        label,
        text: lines
          .slice(
          at + 1,
          (next === (-1)) ? lines.length : next,
          )
          .join('\n',),
      },
    ];
  },);
}

/**
 The text under one fenced label of a sheet.

 @param sheet - the sheet a seat was shown

 @param label - the label of the block, such as `THE WORDS TO REPLACE`

 @returns The block's text, empty when the sheet has no such block

 @example
 ```ts
 const words = sheetBlockOf({ sheet, label: 'THE WORDS TO REPLACE', },);
 ```
 */
export function sheetBlockOf({
  sheet,
  label,
}: {
  readonly sheet: string;
  readonly label: string
},): string {
  return blocksOf({ sheet, },)
    .find(function isNamed(block,): boolean {
      return block.label === label;
    },)
    ?.text
    ?? '';
}

/**
 The current English paragraph a writer's sheet shows.

 @param sheet - the writer sheet a seat was shown

 @returns The paragraph, empty when the sheet shows none

 @example
 ```ts
 const paragraph = currentParagraphOf({ sheet, },);
 ```
 */
export function currentParagraphOf({ sheet, }: { readonly sheet: string; },): string {
  return sheetBlockOf({
    sheet,
    label: 'CURRENT ENGLISH PARAGRAPH',
  },);
}

/**
 The candidates of a choice sheet with their texts and whether each is the
 paragraph as the page has it now.

 @param sheet - the choice sheet a seat was shown

 @returns The candidates in the order the sheet shows them

 @example
 ```ts
 const candidates = candidateTextsOf({ sheet, },); // [{ label: 'CANDIDATE 1', current: true, text: '...' }]
 ```
 */
export function candidateTextsOf(
  { sheet, }: { readonly sheet: string; },
): readonly {
  readonly label: string;
  readonly current: boolean;
  readonly text: string
}[] {
  return blocksOf({ sheet, },)
    .filter(function isCandidate({ label, },): boolean {
      return label.startsWith('CANDIDATE ',);
    },)
    .map(function candidateOf({
      label,
      text,
    },): {
      readonly label: string;
      readonly current: boolean;
      readonly text: string
    } {
      return {
        label: label.split(' ',)
          .slice(
          0,
          2,
          )
          .join(' ',),
        current: label.endsWith('(THE PARAGRAPH AS THE PAGE HAS IT NOW)',),
        text,
      };
    },);
}

/**
 The labels of the places a run-off sheet shows, in the order it shows them.

 @param sheet - the run-off sheet a seat was shown

 @returns The finalists' labels

 @example
 ```ts
 const finalists = finalistsOf({ sheet, },); // ['OCCURRENCE 3', 'OCCURRENCE 4']
 ```
 */
export function finalistsOf({ sheet, }: { readonly sheet: string; },): readonly string[] {
  return blocksOf({ sheet, },)
    .map(function wordsOf({ label, },): readonly string[] {
      return label.split(' ',);
    },)
    .filter(function isPlaceOriginal([kind, number, tail, ...rest],): boolean {
      return (kind === 'OCCURRENCE') && (tail === 'ORIGINAL')
        && (rest.length === 0)
        && (number !== undefined)
        && (number.length > 0)
        && (Number.isInteger(Number(number,),));
    },)
    .map(function labelOf(words,): string {
      return words.slice(
        0,
        2,
      )
        .join(' ',);
    },);
}

/**
 The first label a run-off sheet's reply shape names, which is the first tied place.

 @param sheet - the run-off sheet a seat was shown

 @returns The label

 @example
 ```ts
 const label = firstFinalistOf({ sheet, },); // 'OCCURRENCE 3'
 ```
 */
export function firstFinalistOf({ sheet, }: { readonly sheet: string; },): string {
  return finalistsOf({ sheet, },)[0] ?? 'OCCURRENCE 1';
}

/**
 Which of the pass's questions a sheet asks.

 @param sheet - the sheet a seat was shown

 @returns `writer` for a sheet asking for a paragraph, `choice` for a sheet asking to choose among candidate
 paragraphs, `run-off` for a sheet asking to settle a tie among places, and `ballot` for the first round

 @example
 ```ts
 const kind = sheetKindOf({ sheet, },); // 'ballot'
 ```
 */
export function sheetKindOf({ sheet, }: { readonly sheet: string; },): 'ballot' | 'writer' | 'choice' | 'run-off' {
  // The choice sheet shows the words to replace as the writer's sheet does, so its own markers are read first: the
  // current paragraph's header, which a run-off among candidates leaves out when the current paragraph did not tie, and
  // the sentence listing the candidates' labels, which every choice sheet ends with.
  if (
    sheet.includes(' (THE PARAGRAPH AS THE PAGE HAS IT NOW) ',)
      || sheet.includes('The labels you may write are exactly: CANDIDATE ',)
  )
    return 'choice';
  if (sheet.includes('THE WORDS TO REPLACE',))
    return 'writer';
  if (sheet.includes('WORDS THAT RENDER THE REPEATED WORDING',))
    return 'run-off';
  return 'ballot';
}

/**
 Whether a sheet asks about the refrain rather than the repeated sentence.

 @param sheet - the sheet a judge was shown

 @returns Whether it shows a frame

 @example
 ```ts
 const framed = isRefrainSheet({ sheet, },);
 ```
 */
export function isRefrainSheet({ sheet, }: { readonly sheet: string; },): boolean {
  return sheet.includes('FRAME BEGINS WITH',) && (sheetKindOf({ sheet, },) === 'ballot');
}

//endregion Sheet reading
