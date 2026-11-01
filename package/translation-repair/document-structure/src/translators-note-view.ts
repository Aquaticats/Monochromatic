import {
  type Cut,
  noteCutsOf,
} from './translators-note.ts';

//region Translators' notes, as shown
// A QUOTE COPIED OUT OF THE SHOWN TEXT MUST BE FOUND IN THE STORED ONE. The judges
// are shown a translation without the notes the archive's translators left each
// other (`withoutTranslatorsNotes`), and a quote they copy across the place a note
// was cut from stands in no stored text: the note, and the space or line it took
// with it, stand between the words. The quote is found in the text as shown
// instead, and this file carries the place it was found back to the stored text.
//
// A SPAN LEAVES THE NOTE OUT. A quote across a cut comes back as the stored
// stretches of words on each side of the note, one for each run the quote reaches
// into, and never as one span that covers the note. Covering it would put the
// note on the panel's evidence line, inside the region an editor replaces (whose
// replacement holds no note unless the editor copies it, and the page restores a
// lost note after the slice's last block, not where it stood), and into the words
// the consistency pass copies between places. Stretches that leave the note out
// follow the rule that already splits a quote across two blocks
// (`typography/src/locate-quote.ts`): what lies between is simply not covered.
//
// WHITESPACE AT A CUT IS NOT THE QUOTE'S. A stretch that begins where a cut ended,
// or ends where one began, gives up the line breaks and spaces on that edge: the
// cut took the line break or the space of its own accord (a note alone on its line
// takes the line with it), and the quote's words never began or ended there. A
// region that kept the break could lose it: a replacement of `\nThen it naps.`
// written without the break would run the line into the note's line.

/**
 One run of the shown text that stands unbroken in the stored text.

 @example
 ```ts
 const kept: KeptStretch = { shownStart: 22, storedStart: 36, length: 28, };
 ```
 */
export type KeptStretch = {
  /**
   Offset of the run in the shown text.
   */
  readonly shownStart: number;

  /**
   Offset of the same run in the stored text.
   */
  readonly storedStart: number;

  /**
   UTF-16 units in the run, the same in both texts.
   */
  readonly length: number;
};

/**
 A text as a judge is shown it, with what carries a place in it back to the
 stored text.

 @example
 ```ts
 const shown: ShownTranslation = shownTranslation({ text: 'The cat slept. <!-- 大家看一下 -->', },);
 ```
 */
export type ShownTranslation = {
  /**
   The text without the notes between translators, as `withoutTranslatorsNotes`
   returns it.
   */
  readonly text: string;

  /**
   The runs of the shown text that stand unbroken in the stored text, in order.
   */
  readonly kept: readonly KeptStretch[];

  /**
   Length of the stored text, which tells whether a note follows the last run.
   */
  readonly storedLength: number;
};

/**
 A stretch of the stored text.

 @example
 ```ts
 const stretch: StoredStretch = { from: 0, to: 22, };
 ```
 */
export type StoredStretch = {
  /**
   First offset of the stretch.
   */
  readonly from: number;

  /**
   Exclusive offset where it ends.
   */
  readonly to: number;
};

/**
 The runs of a text between its cuts.

 @param text - the stored text

 @param cuts - the stretches taken out, in order and apart from each other

 @returns One stretch of stored offsets for each gap between cuts, the empty
 ones left out

 @example
 ```ts
 const runs = storedRunsBetween({ text: 'a<!--x-->b', cuts: [{ from: 1, to: 9, },], },); // 0 to 1, 9 to 10
 ```
 */
function storedRunsBetween(
  {
    text,
    cuts,
  }: {
    readonly text: string;
    readonly cuts: readonly Cut[];
  },
): readonly StoredStretch[] {
  return [
    {
      from: 0,
      to: cuts[0]
        ?.from
        ?? text.length,
    },
    ...cuts.map(function runAfter(
      cut,
      index,
    ): StoredStretch {
      return {
        from: cut.to,
        to: cuts[index + 1]
          ?.from
          ?? text.length,
      };
    },),
  ].filter(function isNotEmpty(run,): boolean {
    return run.to > run.from;
  },);
}

/**
 A text as a judge is shown it, with the map back to the stored text.

 @param text - the stored passage

 @param original - the original passage, whose own comments stay in view: the
 same one the sheet was cut against, or the map describes another text

 @returns The shown text, identical to `withoutTranslatorsNotes` for the same
 arguments, and the runs of it that stand unbroken in the stored text

 @example
 ```ts
 const shown = shownTranslation({ text: 'The cat slept. <!-- 大家看一下 -->', original: '', },);
 // shown.text is 'The cat slept.'
 ```
 */
export function shownTranslation(
  {
    text,
    original = '',
  }: {
    readonly text: string;
    readonly original?: string;
  },
): ShownTranslation {
  /**
   The stored runs between the cuts.
   */
  const runs = storedRunsBetween({
    text,
    cuts: noteCutsOf({
      text,
      original,
    },),
  },);
  return {
    text: runs
      .map(function wordsOf(run,): string {
        return text.slice(
          run.from,
          run.to,
        );
      },)
      .join('',),
    kept: runs.map(function placed(
      run,
      index,
    ): KeptStretch {
      return {
        // The runs before this one, laid end to end, are where it starts in the shown text.
        shownStart: runs
          .slice(
          0,
          index,
          )
          .reduce(
          function laidEnd(
            total,
            before,
          ): number {
            return total + (before.to - before.from);
          },
          0,
          ),
        storedStart: run.from,
        length: run.to - run.from,
      };
    },),
    storedLength: text.length,
  };
}

/**
 Whether the shown character at an offset is a space, a tab or a line break.

 @param shown - the shown text

 @param at - offset in it

 @returns Whether the character there is one of the four

 @example
 ```ts
 const spaced = isBreakOrSpace({ shown, at: 3, },);
 ```
 */
function isBreakOrSpace(
  {
    shown,
    at,
  }: {
    readonly shown: ShownTranslation;
    readonly at: number;
  },
): boolean {
  /**
   The text being read, whose end no offset reaches.
   */
  const { text, } = shown;
  return (at < text.length) && ' \t\n\r'.includes(text.charAt(at,),);
}

/**
 First offset of a stretch of the shown text that is not a space, a tab or a line break.

 @param shown - the shown text

 @param from - first offset of the stretch

 @param to - exclusive offset where it ends

 @returns The offset, `to` when the stretch is nothing but breaks and spaces

 @example
 ```ts
 const at = offsetPastBreaks({ shown, from: 14, to: 22, },);
 ```
 */
function offsetPastBreaks(
  {
    shown,
    from,
    to,
  }: {
    readonly shown: ShownTranslation;
    readonly from: number;
    readonly to: number;
  },
): number {
  /**
   Offset being read, moved past each break or space.
   */
  let at = from;
  while ((at < to) && isBreakOrSpace({
    shown,
    at,
  },))
    at += 1;
  return at;
}

/**
 Exclusive offset past the last character of a stretch of the shown text that is not a space, a tab or a line break.

 @param shown - the shown text

 @param from - first offset of the stretch

 @param to - exclusive offset where it ends

 @returns The offset, `from` when the stretch is nothing but breaks and spaces

 @example
 ```ts
 const end = offsetBeforeBreaks({ shown, from: 0, to: 15, },);
 ```
 */
function offsetBeforeBreaks(
  {
    shown,
    from,
    to,
  }: {
    readonly shown: ShownTranslation;
    readonly from: number;
    readonly to: number;
  },
): number {
  /**
   Exclusive end being read, moved back past each break or space.
   */
  let end = to;
  while ((end > from) && isBreakOrSpace({
    shown,
    at: end - 1,
  },))
    end -= 1;
  return end;
}

/**
 The reach of a span inside one run, without the whitespace a cut left on the
 edge it touches.

 @param shown - the shown text with its runs

 @param run - the run

 @param start - first offset of the span in the shown text

 @param end - exclusive offset where it ends

 @returns The stored stretch, none where the span misses the run or what it
 reaches of it is whitespace beside a cut

 @example
 ```ts
 const stretches = stretchInRun({ shown, run: shown.kept[0], start: 0, end: 5, },);
 ```
 */
function stretchInRun(
  {
    shown,
    run,
    start,
    end,
  }: {
    readonly shown: ShownTranslation;
    readonly run: KeptStretch;
    readonly start: number;
    readonly end: number;
  },
): readonly StoredStretch[] {
  /**
   Where the span reaches into the run, in the shown text.
   */
  const reachFrom = Math.max(
    start,
    run.shownStart,
  );
  /**
   Where the reach ends, in the shown text.
   */
  const reachTo = Math.min(
    end,
    run.shownStart + run.length,
  );
  if (reachTo <= reachFrom)
    return [];
  // A CUT STANDS BEFORE THE RUN unless it starts the stored text, and after it unless it ends the stored text.
  /**
   The reach without the whitespace beside a cut that stands before the run, when it begins at the run's start.
   */
  const from = ((run.storedStart > 0) && (reachFrom === run.shownStart))
    ? offsetPastBreaks({
      shown,
      from: reachFrom,
      to: reachTo,
    },)
    : reachFrom;
  /**
   The reach without the whitespace beside a cut that stands after the run, when it ends at the run's end.
   */
  const to = (((run.storedStart + run.length) < shown.storedLength) && (reachTo === (run.shownStart + run.length)))
    ? offsetBeforeBreaks({
      shown,
      from,
      to: reachTo,
    },)
    : reachTo;
  if (to <= from)
    return [];
  return [
    {
      from: run.storedStart + (from - run.shownStart),
      to: run.storedStart + (to - run.shownStart),
    },
  ];
}

/**
 Where a span of the shown text stands in the stored text.

 @param shown - the shown text with its runs

 @param start - first offset of the span in the shown text

 @param end - exclusive offset where it ends

 @returns One stretch of the stored text for each run the span reaches into, in
 order, none of them holding a note and none beginning or ending in the
 whitespace a cut left beside it; empty when the span is nothing but that
 whitespace

 @example
 ```ts
 const stretches = storedStretchesOf({ shown, start: 0, end: 22, },);
 ```
 */
export function storedStretchesOf(
  {
    shown,
    start,
    end,
  }: {
    readonly shown: ShownTranslation;
    readonly start: number;
    readonly end: number;
  },
): readonly StoredStretch[] {
  return shown.kept
    .flatMap(function inRun(run,): readonly StoredStretch[] {
      return stretchInRun({
        shown,
        run,
        start,
        end,
      },);
    },);
}

//endregion Translators' notes, as shown
