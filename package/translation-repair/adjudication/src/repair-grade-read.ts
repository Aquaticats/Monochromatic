import {
  opensWithVerdict,
  trimLeadingDelimiters,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Repair grade reading
// Reading a human's filled-in REPAIR sheet back into verdicts.
//
// Separate from the detection reader because the sheets differ in shape as well
// as in question. Detection puts its answer on the item heading; repair puts it
// on a bullet under one, so a grade has to be associated with the heading above
// it rather than read off the same line.
//
// FENCES ARE TRACKED, and that is the whole reason this cannot be a line
// filter. Repair sheets quote corpus prose and model output inside fenced
// blocks, and `repair-sheet.ts` fences them precisely because such text may
// contain a literal `- repair grade: [Y]` or a `### 99.` heading. The sheet is
// safe to LOOK at because the fence is chosen against the content; the sheet is
// only safe to PARSE if the parser honors that fence. A reader that did not
// would let a quoted line fabricate a human verdict, which is worse than
// dropping one: an invented grade cannot be noticed downstream.

/**
 Marker beginning a sheet item.
 */
const ITEM_PREFIX = '### ';

/**
 Header line declaring the seed a sheet was drawn under.
 */
const SEED_PREFIX = 'Draw seed: ';

/**
 Header line declaring the corpus commit a sheet was produced against.
 */
const CORPUS_PREFIX = 'Corpus pin: ';

/**
 Header line carrying the fingerprint of the draw a sheet came from.
 */
const DIGEST_PREFIX = 'Draw digest: ';

/**
 What a sheet says about the draw it belongs to.

 @example
 ```ts
 const identity: SheetIdentity = { seed: 'round-three', corpusSha: 'a41fc60', };
 ```
 */
export type SheetIdentity = {
  /**
   Seed the sheet declares, empty when the header carried none.
   */
  readonly seed: string;

  /**
   Corpus commit the sheet declares, empty when the header carried none.
   */
  readonly corpusSha: string;

  /**
   Draw fingerprint the sheet declares, empty when the header carried none.

   Empty is the ordinary reading for every sheet drawn before the binding
   existed, including round three's, which is why an absent digest is scored
   under the weaker check rather than refused.
   */
  readonly drawDigest: string;
};

/**
 Reads the draw a sheet says it belongs to.

 Only the header is read, and only up to the first item, so a line quoted
 inside an item cannot introduce a second identity. Both sheets and the
 manifest are written in one breath by one draw, so these agreeing is what
 makes joining them by POSITION sound. Equal item counts alone do not: two
 unrelated draws of the same size match on count and mislabel every verdict.

 @param text - sheet contents as the grader left them

 @returns Declared seed and corpus commit, each empty when absent

 @example
 ```ts
 const identity = readSheetIdentity({ text, },);
 ```
 */
export function readSheetIdentity(
  { text, }: { readonly text: string; },
): SheetIdentity {
  /**
   Values found so far in the header.
   */
  const found = {
    seed: '',
    corpusSha: '',
    drawDigest: '',
  };
  for (const line of text.split('\n',)) {
    if (line.startsWith(ITEM_PREFIX,))
      break;
    if (line.startsWith(SEED_PREFIX,))
      found.seed = line.slice(SEED_PREFIX.length,)
        .trim();
    else if (line.startsWith(CORPUS_PREFIX,))
      found.corpusSha = line.slice(CORPUS_PREFIX.length,)
        .trim();
    else if (line.startsWith(DIGEST_PREFIX,))
      found.drawDigest = line.slice(DIGEST_PREFIX.length,)
        .trim();
  }
  return found;
}

/**
 Marker introducing the grade on a repair-sheet item.
 */
const GRADE_MARKER = '- repair grade:';

/**
 Legend that follows every grade, and therefore bounds the grader's answer.
 */
const LEGEND_MARKER = '(Y = ';

/**
 Fence character opening and closing a quoted block.
 */
const FENCE_CHARACTER = '`';

/**
 Shortest run that can delimit a fenced block.
 */
const FENCE_MIN = 3;

/**
 What a grader said about one repair.

 @example
 ```ts
 const verdict: RepairVerdict = 'fixes';
 ```
 */
export type RepairVerdict =
  | 'fixes'
    | 'does-not-fix'
    | 'unscored';

/**
 One item as the repair sheet carries it after grading.

 @example
 ```ts
 const item: GradedRepairItem = { index: 1, verdict: 'fixes', note: '', };
 ```
 */
export type GradedRepairItem = {
  /**
   Sheet position, matching the detection sheet exactly.
   */
  readonly index: number;

  /**
   Verdict the grader left, or `unscored` where they left none.
   */
  readonly verdict: RepairVerdict;

  /**
   Rationale following the verdict, empty when none was written.
   */
  readonly note: string;
};

/**
 Leading run of fence characters on one line.

 @param line - sheet line

 @returns Run length at the start of the line, zero when it starts otherwise

 @example
 ```ts
 const run = leadingFenceRun({ line: '````text', },);
 ```
 */
function leadingFenceRun({ line, }: { readonly line: string; },): number {
  for (let index = 0; index < line.length; index += 1)
    if (line.charAt(index,) !== FENCE_CHARACTER)
      return index;
  return line.length;
}

/**
 Reads the grader's answer out of one grade line, with the legend and any
 enclosing brackets removed.

 @param line - line carrying the grade marker

 @returns Answer as the grader left it

 @example
 ```ts
 const answer = extractAnswer({ line: '- repair grade: [Y]  (Y = ...)', },);
 ```
 */
function extractAnswer({ line, }: { readonly line: string; },): string {
  /**
   Text following the grade marker.
   */
  const afterMarker = line.slice(line.indexOf(GRADE_MARKER,) + GRADE_MARKER.length,);

  /**
   Where the printed legend begins, when it survived grading.
   */
  const legendAt = afterMarker.indexOf(LEGEND_MARKER,);

  /**
   Answer with the legend removed.
   */
  const withoutLegend = (legendAt === (-1)
    ? afterMarker
    : afterMarker.slice(
      0,
      legendAt,
    ))
    .trim();
  if (withoutLegend.startsWith('[',)) {
    /**
     Where the grader's bracket closes, when they kept it.
     */
    const closeAt = withoutLegend.indexOf(']',);
    return (closeAt === (-1)
      ? withoutLegend.slice(1,)
      : withoutLegend.slice(
        1,
        closeAt,
      ))
      .trim();
  }
  return withoutLegend;
}

/**
 Classifies one grader answer into a verdict and its remaining prose.

 @param answer - grader's answer, unbracketed

 @returns Verdict and the note that followed it

 @example
 ```ts
 const read = readAnswer({ answer: 'N, it drops the second clause', },);
 ```
 */
function readAnswer({ answer, }: { readonly answer: string; },): {
  readonly verdict: RepairVerdict;
  readonly note: string;
} {
  for (const [
    letter,
    verdict,
  ] of [
    [
      'Y',
      'fixes',
    ],
    [
      'N',
      'does-not-fix',
    ],
  ] as const)
    if (opensWithVerdict({
      answer,
      letter,
    },))
      return {
        verdict,
        note: trimLeadingDelimiters({ text: answer.slice(letter.length,), },),
      };

  // An untouched `[ ]` box, and answers naming no verdict, are deliberately
  // not verdicts: a coerced grade is worse evidence than an absent one.
  return {
    verdict: 'unscored',
    note: answer,
  };
}

/**
 Reads every graded repair off a filled repair sheet.

 Items carrying no grade box at all, which the sheet emits for repairs that
 never reached the reader, appear as `unscored`. They belong to coverage
 rather than to repair quality, and the caller keeps them out of the
 denominator by reading the verdict rather than by their absence.

 @param text - sheet as the grader left it

 @returns Items in sheet order, one per heading

 @example
 ```ts
 const items = parseGradedRepairSheet({ text: await readFile(path, 'utf8',), },);
 ```
 */
export function parseGradedRepairSheet(
  { text, }: { readonly text: string; },
): readonly GradedRepairItem[] {
  /**
   Items closed so far, plus the open fence and item being read.
   */
  const state = {
    items: [] as GradedRepairItem[],
    openFence: 0,
    started: false,
  };
  for (const line of text.split('\n',)) {
    /**
     Fence run opening this line, when it has one.
     */
    const run = leadingFenceRun({ line, },);
    if (state.openFence > 0) {
      // Inside a quoted block: only a fence at least as long as the opening one
      // closes it, and nothing here is ever read as sheet structure.
      if (run >= state.openFence)
        state.openFence = 0;
      continue;
    }
    if (run >= FENCE_MIN) {
      state.openFence = run;
      continue;
    }
    if (line.startsWith(ITEM_PREFIX,)) {
      state.started = true;
      state.items
        .push({
          index: state.items
            .length
            + 1,
          verdict: 'unscored',
          note: '',
        },);
      continue;
    }
    if (!state.started)
      continue;
    if (!line.startsWith(GRADE_MARKER,))
      continue;

    /**
     Verdict and note read off this grade line.
     */
    const read = readAnswer({ answer: extractAnswer({ line, },), },);
    state.items[state.items
      .length
      - 1] = {
        index: state.items
          .length,
        verdict: read.verdict,
        note: read.note,
      };
  }
  return state.items;
}

//endregion Repair grade reading
