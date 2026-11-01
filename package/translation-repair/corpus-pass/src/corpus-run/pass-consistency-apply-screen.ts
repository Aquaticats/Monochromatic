import {
  codePointAt,
  codePointBefore,
  continuesLatinWordOrCompatibilityForm,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type ConsistencyEdit,
  partsMarkFromWord,
  WRITTEN_APOSTROPHES,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  type GainedSides,
  madeIn,
  type MadeSlice,
  unprovenOnPage,
} from './pass-consistency-apply-gains.ts';
import {
  gainSliceOf,
  idsOf,
  keyOf,
  pageFrameOf,
  type ReadmitLedger,
} from './pass-consistency-apply-ledger.ts';

//region Pass consistency apply screen
// ONE READING OF THE PAGE SERVES MANY SPLICES (`pass-consistency-apply-readmit.ts`). A TRIAL is the question asked of one
// splice left out: with it made beside the splices that stand, do the words overlap none of theirs, do the floors pass its slice,
// and is every apostrophe carried in proven on the page that ships (`pass-consistency-apply-trial.ts`). A trial reads
// the whole page, so a page that left out as many splices as it has slices would read itself that many times
// over. A SCREEN is the cheaper question this file answers with one reading of the page for all the splices of a side, which
// blocks the splices no trial could put back.
//
// THE SCREEN READS ONLY A SPLICE WHOSE MARK THE PROOF READS AS THE APOSTROPHE OF ITS EDGE ON ITS OWN. The proof reads a mark as
// trailing when a Latin letter or a digit stands directly before it and a character that parts a mark from a word stands
// directly after it (white space, punctuation that is no single mark, the edge of the text: `partsMarkFromWord`,
// `consistency-mark-neighbours.ts`), and as leading in the mirror. The screen asks the same two questions of the splice (the
// letter inside its words, `readsAsEdgeApostrophe`, and the character beside the mark outside them, `besideOf`). A mark beside a
// symbol, a pictograph, a letter of another script or another single mark reads FREE in the proof, which makes it a possible
// partner of every later mark, and a mark that is no apostrophe (the opening curly single mark, the low, reversed and angle
// marks) is a partner of every apostrophe and is never proven itself. A page with such a mark made would block splices that a trial
// of their own proves, so a splice with such a mark is not made in the reading and goes to a trial.
//
// WHAT THE SCREEN THEN BLOCKS. Adding quotation marks to a page can only make a proof stricter, and the apostrophe a splice brings to
// the end of its words (a trailing mark) is no partner for any other trailing mark, as one it brings to the
// start is none for another leading one. So when a splice whose mark the proof reads that way touches no other replacement of
// its slice, its words hold that mark and no other quotation mark, and the words it takes out hold none, the page with ALL
// such splices of that side made proves its mark exactly where the page with it alone would, and a mark
// unproven there is unproven in every trial. That splice is blocked without a trial of its own. A mark proven
// there still gets its trial, since the splices that stand and the floors are still to be checked: THE SCREEN
// ONLY SPARES TRIALS, it never puts a splice back. Because a proof only gets stricter as marks are added, a
// splice blocked once stays blocked in every later pass.

/**
 The characters the screen counts as quotation marks: every character the proof of an apostrophe could read as a partner.
 */
const QUOTE_CHARACTERS: ReadonlySet<string> = new Set(Array.from('\'"`‘’‚‛“”„‟«»‹›ʼʻ′″',),);

/**
 How many quotation marks a text holds.

 @param text - the words

 @returns The count

 @example
 ```ts
 const count = quoteCountOf({ text: 'the parents’', },); // 1
 ```
 */
function quoteCountOf({ text, }: { readonly text: string; },): number {
  /**
   The characters of the text.
   */
  const points = Array.from(text,);
  /**
   The ones that are quotation marks.
   */
  const quotes = points.filter(function isQuote(point,): boolean {
    return QUOTE_CHARACTERS.has(point,);
  },);
  /**
   How many there are.
   */
  const { length, } = quotes;
  return length;
}

/**
 Whether a splice's words hold one quotation mark and no other. For a splice that gains an apostrophe at an edge, which the
 caller has checked, that one mark is the apostrophe at the edge: an edge apostrophe is one of the characters counted.

 @param edit - the splice

 @returns Whether exactly one character of its words is a quotation mark

 @example
 ```ts
 const holds = holdsOneQuoteMark({ edit, },);
 ```
 */
function holdsOneQuoteMark({ edit, }: { readonly edit: ConsistencyEdit; },): boolean {
  return quoteCountOf({ text: edit.text, },) === 1;
}

/**
 Whether another replacement of the slice overlaps or touches the words of a splice.

 @param edits - every replacement addressed to the slice

 @param edit - the splice

 @returns Whether some other replacement meets its words

 @example
 ```ts
 const meets = touchesAnother({ edits, edit, },);
 ```
 */
function touchesAnother(
  {
    edits,
    edit,
  }: {
    readonly edits: readonly ConsistencyEdit[];
    readonly edit: ConsistencyEdit;
  },
): boolean {
  return edits.some(function touchesIt(other,): boolean {
    return (other.id !== edit.id)
      && (other.start <= edit.end)
      && (other.end >= edit.start);
  },);
}

/**
 The character that stands beside a splice's words, outside them, when no other splice can change it.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param edit - the splice

 @param side - the edge: `end` for the character after the words, `start` for the one before

 @returns Whether the character is known to stay as it is, and the characters beside the mark, each a whole character: one,
 or none at the edge of the page; an empty gap between two slices leaves the next slice's own words beside the mark, which
 a splice of that slice could change, so it is not known to stay

 @example
 ```ts
 const { stays, characters, } = besideOf({ ledger, position: 0, edit, side: 'end', },);
 ```
 */
function besideOf(
  {
    ledger,
    position,
    edit,
    side,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly edit: ConsistencyEdit;
    readonly side: 'start' | 'end';
  },
): {
  readonly stays: boolean;
  readonly characters: readonly string[];
} {
  /**
   The slices and the text around them.
   */
  const {
    worked,
    between,
  } = ledger;
  /**
   The slice the splice is addressed to.
   */
  const slice = worked[position];
  if (slice === undefined) {
    return {
      stays: false,
      characters: [],
    };
  }
  /**
   The slice's wording.
   */
  const { currentText, } = slice;
  /**
   How many slices the page has.
   */
  const { length: count, } = worked;
  if ((side === 'end') && (edit.end < currentText.length)) {
    return {
      stays: true,
      characters: [
        codePointAt({
          text: currentText,
          at: edit.end,
        },),
      ],
    };
  }
  if ((side === 'start') && (edit.start > 0)) {
    return {
      stays: true,
      characters: [
        codePointBefore({
          text: currentText,
          at: edit.start,
        },),
      ],
    };
  }
  /**
   The place around the slices that lies beyond the slice on the side of the mark.
   */
  const gapPlace = (side === 'end') ? (position + 1) : position;
  /**
   The characters the page carries there.
   */
  const gap = Array.from(between[gapPlace] ?? '',);
  if (gap.length === 0) {
    return {
      stays: (side === 'end') ? (position === (count - 1)) : (position === 0),
      characters: [],
    };
  }
  return {
    stays: true,
    characters: (side === 'end')
      ? gap.slice(
        0,
        1,
      )
      : gap.slice(-1,),
  };
}

/**
 Whether the mark a splice brings to an edge is one the proof can read as that edge's own apostrophe, whatever the page holds
 around it: an apostrophe-like mark (the straight one, the closing curly one and the modifier letter apostrophe; the opening
 curly mark and the low, reversed and angle marks are partners of every apostrophe and are never proven themselves), with a Latin
 letter or a digit directly inside it, which is the one side of the proof's reading that the splice's own words decide.

 @param edit - the splice

 @param side - the edge it is read at

 @returns Whether the first or last whole character of its words is such a mark with a letter or a digit inside it

 @example
 ```ts
 const reads = readsAsEdgeApostrophe({ edit, side: 'end', },);
 ```
 */
function readsAsEdgeApostrophe(
  {
    edit,
    side,
  }: {
    readonly edit: ConsistencyEdit;
    readonly side: 'start' | 'end';
  },
): boolean {
  /**
   The whole characters of the splice's words.
   */
  const points = Array.from(edit.text,);
  /**
   The character at the edge.
   */
  const mark = (side === 'start') ? points[0] : points.at(-1,);
  /**
   The character directly inside it, none past the words.
   */
  const inside = (side === 'start') ? points[1] : points.at(-2,);
  if ((mark === undefined) || (!WRITTEN_APOSTROPHES.has(mark,)))
    return false;
  return continuesLatinWordOrCompatibilityForm({ character: inside ?? '', },);
}

/**
 Whether the screen can read the apostrophe a splice carries in without a trial of its own.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param edit - the splice

 @param side - the edge it is read at

 @returns Whether the proof reads this splice's mark as the trailing (or leading) apostrophe of its words on its own, so that
 the page with every such splice made proves it exactly where the page with it alone would: it gains a mark at that edge
 only, the mark is an apostrophe-like one with a Latin letter or a digit inside it, its words hold that mark and no other,
 the words it takes out hold none, no other replacement of its slice touches it, and the character beside the mark outside
 the words (the edge of the text counts) is one the proof parts a mark from (`partsMarkFromWord`) and no splice of another
 slice can change it; a mark beside a symbol, a pictograph, a letter of another script or another single mark reads free,
 is a possible partner of every other mark, and goes to a trial

 @example
 ```ts
 const reads = isScreenable({ ledger, position: 0, edit, side: 'end', },);
 ```
 */
function isScreenable(
  {
    ledger,
    position,
    edit,
    side,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly edit: ConsistencyEdit;
    readonly side: 'start' | 'end';
  },
): boolean {
  /**
   The slices and the sides each splice gains an apostrophe on.
   */
  const {
    worked,
    gained,
  } = ledger;
  /**
   The slice the splice is addressed to.
   */
  const slice = worked[position];
  if (slice === undefined)
    return false;
  /**
   The sides each splice of the slice gains an apostrophe on.
   */
  const gainedHere = gained[position];
  /**
   The sides of its words this splice gains an apostrophe on.
   */
  const sides = (gainedHere === undefined) ? [] : (gainedHere.get(edit.id,) ?? []);
  if (sides[0] !== side)
    return false;
  if (!holdsOneQuoteMark({ edit, },))
    return false;
  /**
   The slice's wording and the replacements addressed to it.
   */
  const {
    currentText,
    named,
  } = slice;
  /**
   The words the splice takes out.
   */
  const taken = currentText.slice(
    edit.start,
    edit.end,
  );
  if (quoteCountOf({ text: taken, },) !== 0)
    return false;
  /**
   Every replacement addressed to the slice.
   */
  const { edits, } = named;
  if (touchesAnother({
    edits,
    edit,
  },))
    return false;
  /**
   The characters beside the mark, outside the words, and whether they stay as they are.
   */
  const {
    stays,
    characters,
  } = besideOf({
    ledger,
    position,
    edit,
    side,
  },);
  if (!stays)
    return false;
  if (!readsAsEdgeApostrophe({
    edit,
    side,
  },))
    return false;
  return characters.every(function partsTheMark(character,): boolean {
    return partsMarkFromWord({ character, },);
  },);
}

/**
 Blocks the splices whose apostrophe no trial can prove, in one reading of the page.

 @param ledger - what the step knows of the page; its blocked splices and reasons are added to

 @param side - the edge the screen reads: `end` for the trailing marks, `start` for the leading ones

 @example
 ```ts
 screenSide({ ledger, side: 'end', },);
 ```
 */
export function screenSide(
  {
    ledger,
    side,
  }: {
    readonly ledger: ReadmitLedger;
    readonly side: 'start' | 'end';
  },
): void {
  /**
   What the step knows of the page.
   */
  const {
    worked,
    refused,
    dropped,
    blocked,
    reasonLog,
    gained,
    made: ledgerMade,
    toMake,
  } = ledger;
  /**
   The splices this side's screen reads, for each slice.
   */
  const screenable = worked.map(function screenableIn(
    _slice,
    position,
  ): readonly ConsistencyEdit[] {
    if (refused.has(position,))
      return [];
    /**
     The splices of the slice that are left out.
     */
    const left = dropped[position] ?? [];
    return left.filter(function isReadable(edit,): boolean {
      return (!blocked.has(keyOf({
        position,
        id: edit.id,
      },),)) && isScreenable({
        ledger,
        position,
        edit,
        side,
      },);
    },);
  },);
  /**
   Whether this side's screen reads no splice at all.
   */
  const none = screenable.every(function holdsNone(edits,): boolean {
    return edits.length === 0;
  },);
  if (none)
    return;
  /**
   The page with the splices that stand and every splice this screen reads made.
   */
  const made = worked.map(function madeWith(
    slice,
    position,
  ): MadeSlice {
    /**
     The splices of this slice the screen reads.
     */
    const reading = screenable[position] ?? [];
    if (reading.length === 0) {
      return ledgerMade[position] ?? {
        text: slice.currentText,
        edits: [],
      };
    }
    /**
     The names of the replacements that stand and of the splices read, which are all made.
     */
    const wanted = new Set([
      ...idsOf({ edits: toMake[position] ?? [], },),
      ...idsOf({ edits: reading, },),
    ],);
    /**
     The slice's replacements that were not left out for another group's words.
     */
    const { candidates, } = slice;
    return madeIn({
      slice: gainSliceOf({ worked: slice, },),
      remaining: candidates.filter(function isWanted({ id, },): boolean {
        return wanted.has(id,);
      },),
    },);
  },);
  /**
   For each slice, only the splices the screen reads are asked about.
   */
  const readGained = worked.map(function readOnly(
    _slice,
    position,
  ): ReadonlyMap<string, GainedSides> {
    /**
     The sides each splice of the slice gains an apostrophe on.
     */
    const sidesHere = gained[position] ?? new Map<string, GainedSides>();
    return new Map((screenable[position] ?? []).map(function sidesOf({ id, },): readonly [
      string,
      GainedSides,
    ] {
      return [
        id,
        sidesHere.get(id,) ?? [],
      ];
    },),);
  },);
  /**
   The splices whose mark is unproven on that page.
   */
  const unproven = unprovenOnPage({
    made,
    gained: readGained,
    ...pageFrameOf({ ledger, },),
  },);
  for (const [position, edits,] of unproven.entries()) {
    for (const edit of edits) {
      /**
       The splice's key.
       */
      const key = keyOf({
        position,
        id: edit.id,
      },);
      blocked.add(key,);
      reasonLog.push({
        key,
        reason: { kind: 'pairs', },
      },);
    }
  }
}

//endregion Pass consistency apply screen
