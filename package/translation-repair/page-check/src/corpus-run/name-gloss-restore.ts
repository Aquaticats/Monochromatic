import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordStarts, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  pageTextOf,
  rowsChangedBy,
  type SliceReplacement,
  slicesInOrder,
} from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Name gloss restore
// THE ONE HUNDRED AND FIFTH CLASS (CuspariaKLSY9, 2026-09-23). The archive
// carries, on the line after the nickname sentence, a translator's gloss of
// the declared name: "“Ling Shui Yu Yu Zi” means fish in clear water." The
// Chinese is silent about it, so it is the page's own apparatus, and the
// eighty-fifth class put one clause on every writing and judging sheet saying
// it is kept. The judges still dropped it: on CuspariaKLSY9 the contest chose
// the gloss-less translate text 4 to 1, one ballot calling the gloss
// unsupported, and the slate endorsed that standing 4 of 5; CuspariaKLSY5 had
// lost it the same way. A line whose shape says what it is, a quoted name the
// archive itself uses followed by "means", is restored here, after the
// judges, where the page is in view: inserted after the shipped line that
// carries the name, unless the shipped text glosses the name in its own way.

/**
 Quote marks a gloss line may open and close with.
 */
const QUOTES: ReadonlySet<string> = new Set([
  '“',
  '”',
  '"',
],);

/**
 Words that follow the quoted name on a gloss line.
 */
const GLOSS_VERB = ' means ';

/**
 What `indexOf` answers for a text it never finds.
 */
const NOT_FOUND = -1;

/**
 One gloss line of the archive: the name it glosses and the line itself.
 */
type NameGloss = {
  readonly name: string;
  readonly line: string;
};

/**
 Reads one archive line as a gloss of a name, where it opens with a quoted
 name followed by "means".

 @param line - archive line, untrimmed

 @returns Gloss, or nothing when the line has another shape

 @example
 ```ts
 readGlossLine({ line: '“Mittens” means a small cat.', },);
 ```
 */
function readGlossLine(
  { line, }: { readonly line: string; },
): readonly NameGloss[] {
  /**
   Line without its edges.
   */
  const trimmed = line.trim();
  if ((trimmed.length === 0) || (!QUOTES.has(trimmed.charAt(0,),)))
    return [];
  /**
   Where the name's closing quote stands: the nearest quote mark past the
   opening one, each mark a single code unit.
   */
  const close = Math.min(...[...QUOTES,].map(function closingAt(quote,): number {
    /**
     Where this mark first stands past the opening quote.
     */
    const at = trimmed.indexOf(
      quote,
      1,
    );
    return (at === NOT_FOUND) ? Number.POSITIVE_INFINITY : at;
  },),);
  if ((!Number.isFinite(close,)) || (close < 2))
    return [];
  /**
   Name the quotes hold.
   */
  const name = trimmed.slice(
    1,
    close,
  );
  /**
   What follows the closing quote.
   */
  const rest = trimmed.slice(close + 1,);
  if (!rest.startsWith(GLOSS_VERB,))
    return [];
  return [{
    name,
    line: trimmed,
  },];
}

/**
 Gloss lines of names the archive text uses elsewhere too, so a quoted
 phrase glossed once and never used is not read as the page's apparatus.

 @param archiveText - archive side of one slice

 @returns Every gloss line of a name the text carries at least twice

 @example
 ```ts
 const glosses = nameGlosses({ archiveText: slice.target.text, },);
 ```
 */
function nameGlosses(
  { archiveText, }: { readonly archiveText: string; },
): readonly NameGloss[] {
  return archiveText
    .split('\n',)
    .flatMap(function toGloss(line,): readonly NameGloss[] {
      return readGlossLine({ line, },);
    },)
    .filter(function usedElsewhere(gloss,): boolean {
      /**
       Where the name stands as a word, the gloss line's own quoted use among
       them; one inside a longer word is no use of the name (ledger B23).
       */
      const uses = wordStarts({
        text: archiveText,
        needle: gloss.name,
        end: 'word',
      },);
      return uses.length >= 2;
    },);
}

/**
 Where the line holding one position ends: the next line break, or the
 text's end on its last line.

 @param text - text under scan

 @param at - position on the line

 @returns Offset the line's text stops at

 @example
 ```ts
 lineEndAfter({ text: 'a\nb', at: 0, },);
 ```
 */
function lineEndAfter(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  /**
   Next line break at or past the position.
   */
  const end = text.indexOf(
    '\n',
    at,
  );
  if (end === NOT_FOUND)
    return text.length;
  return end;
}

/**
 Whether the shipped text already glosses the name in its own way: the
 archive's line itself, "means" on the line carrying the name, or a
 parenthesis right after the name.

 @param text - shipped text

 @param gloss - archive gloss

 @param at - where the name first stands in the text

 @returns Whether a gloss is already there

 @example
 ```ts
 glossed({ text, gloss, at: wordStarts({ text, needle: gloss.name, end: 'word', },)[0] ?? -1, },);
 ```
 */
function glossed(
  {
    text,
    gloss,
    at,
  }: {
    readonly text: string;
    readonly gloss: NameGloss;
    readonly at: number;
  },
): boolean {
  if (text.includes(gloss.line,))
    return true;
  /**
   Bounds of the line carrying the name.
   */
  const lineStart = text.lastIndexOf(
    '\n',
    at,
  ) + 1;
  /**
   The line carrying the name.
   */
  const line = text.slice(
    lineStart,
    lineEndAfter({
      text,
      at,
    },),
  );
  if (line.includes(GLOSS_VERB,))
    return true;
  /**
   Name under gloss.
   */
  const { name, } = gloss;
  /**
   Where the name ends in the text.
   */
  const nameEnd = at + name.length;
  /**
   What follows the name.
   */
  const tail = text.slice(nameEnd,);
  /**
   That tail past a closing quote, when one stands first.
   */
  const unquoted = QUOTES.has(tail.charAt(0,),) ? tail.slice(1,) : tail;
  /**
   That tail past a comma too, when one stands first.
   */
  const after = unquoted.startsWith(',',) ? unquoted.slice(1,) : unquoted;
  return after.startsWith(' (',);
}

/**
 Restores the archive's gloss line of a name wherever the shipped text
 carries the name with no gloss of its own.

 @param slices - prepared pairs, whose archive text carries the gloss

 @param replacements - what the page would write per slice

 @returns Replacements with the gloss lines restored, the rewritten rows
 alone, and one finding per restored line

 @throws SliceNotOnPageError where a replacement names a slice the pairs lack

 @example
 ```ts
 const restored = restoreNameGlossLines({ slices, replacements, },);
 ```
 */
export function restoreNameGlossLines(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Slices in order.
   */
  const ordered = slicesInOrder({ slices, },);
  /**
   Archive text per slice, by index.
   */
  const archiveBySlice = new Map<number, string>(ordered.map(function toEntry(
    slice,
  ): readonly [
    number,
    string,
  ] {
    return [
      slice.target
        .sliceIndex,
      slice.target
        .text,
    ];
  },),);
  /**
   Findings, one per restored line.
   */
  const findings: string[] = [];
  /**
   Every replacement, the rewritten ones swapped in.
   */
  const rewritten = replacements.map(function restore(row,): SliceReplacement {
    /**
     Gloss lines the archive carries on this slice.
     */
    const glosses = nameGlosses({
      archiveText: pageTextOf({
        pageText: archiveBySlice,
        sliceIndex: row.sliceIndex,
      },),
    },);
    return glosses.reduce(
      function place(
        current,
        gloss,
      ): SliceReplacement {
      /**
       Where the name first stands as a word in the shipped text, so a
       longer word carrying its letters ("Pipit" for "Pip") neither draws
       the gloss nor places it (ledger B23).
       */
        const at = wordStarts({
          text: current.replacementText,
          needle: gloss.name,
          end: 'word',
        },)[0] ?? NOT_FOUND;
        if (at === NOT_FOUND)
          return current;
        if (glossed({
          text: current.replacementText,
          gloss,
          at,
        },))
          return current;
      /**
       Cut at the end of the line carrying the name.
       */
        const cut = lineEndAfter({
          text: current.replacementText,
          at,
        },);
      /**
       Shipped text up to the cut.
       */
        const head = current.replacementText
          .slice(
          0,
          cut,
          );
      /**
       Shipped text from the cut.
       */
        const rest = current.replacementText
          .slice(cut,);
        findings.push(`name-gloss-restored (slice ${String(row.sliceIndex,)}: "${gloss.line}")`,);
        return {
          sliceIndex: current.sliceIndex,
          replacementText: `${head}\n${gloss.line}${rest}`,
        };
      },
      row,
    );
  },);
  /**
   Rows this pass changed.
   */
  const restored = rowsChangedBy({
    before: replacements,
    after: rewritten,
  },);
  return {
    replacements: rewritten,
    restored,
    findings,
  };
}
//endregion Name gloss restore
