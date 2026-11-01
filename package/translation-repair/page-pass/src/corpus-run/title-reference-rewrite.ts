import {
  isSmallLetter,
  straightenQuotes,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  codePointAt,
  wordStarts,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { LocatedTitle, } from './title-reference-scope.ts';

//region Title reference rewrite
// A REFERENCE'S RENDERING REWRITTEN TO THE HEADING'S, span by span: a quoted,
// bracketed or linked span is the title alone; a glossed run keeps the
// small-letter words leading into its title and is reported where it has
// more words than the heading's rendering; a rendering apart from the
// heading's only in apostrophe style is already the heading's.

/**
 Punctuation a quoted span may end with, kept outside the rewrite.
 */
const TRAILING_MARKS: ReadonlySet<string> = new Set([
  ',',
  '.',
  '!',
  '?',
],);

/**
 Separator between the words of a run.
 */
const WORD_SEPARATOR = ' ';

/**
 Why a glossed run's title is left as it stands: it has more words than the
 heading's rendering.
 */
const RUN_LONGER = 'the glossed run reads as more than the title';

/**
 Why a glossed run's title is left as it stands: no word of the run can
 start a title.
 */
const RUN_UNCAPITALIZED = 'every word of the glossed run starts with a small letter, so where its title starts cannot be read';

/**
 One section heading as the page renders it.
 */
export type RenderedHeading = {
  /**
   Slices whose text carries the heading, several where headings share the
   title and render it alike.
   */
  readonly sliceIndexes: readonly number[];

  /**
   Title as the original writes it.
   */
  readonly title: string;

  /**
   Title as the page renders it, any Han gloss stripped.
   */
  readonly rendering: string;
};

/**
 One rendering rewritten: the span of the page text it replaces, what
 replaces it, and the title before and after, for the findings.
 */
type TitleRewrite = {
  readonly kind: 'rewritten';
  readonly start: number;
  readonly end: number;
  readonly replacement: string;
  readonly before: string;
  readonly after: string;
};

/**
 Outcome of a rewrite: the span rewritten, already the heading's, or a run
 that cannot be read as the title alone, with why.
 */
type RewriteOutcome = {
  readonly kind: 'same';
} | {
  readonly kind: 'ambiguous';
  readonly reason: string;
} | TitleRewrite;

/**
 Count of words in a run.

 @param text - run of words

 @returns Words separated by spaces

 @example
 ```ts
 wordCount({ text: 'Cat in a Cage', },); // 4
 ```
 */
function wordCount({ text, }: { readonly text: string; },): number {
  /**
   Words of the run.
   */
  const words = text.split(WORD_SEPARATOR,)
    .filter(function nonEmpty(word,): boolean {
      return word !== '';
    },);
  return words.length;
}

/**
 Whether a text ends with a run of words, the run starting at a word edge.

 @param text - text read

 @param tail - words it may end with

 @returns Whether the text ends with the tail and no word runs into it

 @example
 ```ts
 endsWithWords({ text: 'from the Cat Murmurs', tail: 'Cat Murmurs', },); // true
 endsWithWords({ text: 'Wildcat Murmurs', tail: 'cat Murmurs', },); // false
 ```
 */
function endsWithWords(
  {
    text,
    tail,
  }: {
    readonly text: string;
    readonly tail: string;
  },
): boolean {
  /**
   Where the tail would start if the text ends with it.
   */
  const tailAt = text.length - tail.length;

  /**
   Offsets where the tail stands as words.
   */
  const starts = wordStarts({
    text,
    needle: tail,
    end: 'word',
  },);
  return starts.includes(tailAt,);
}

/**
 Offset of a glossed run's first word that does not open with a small
 letter, where its title starts; the run's length where every word does.

 A TITLE OPENS WITH A CAPITAL (ledger B58). The pages write titles in
 English title case, which capitalizes a title's first word, so small-letter
 words ahead of it ("from", "sung in") lead into the title and stay on the
 page. Small is read by general category (`isSmallLetter`), since a word
 leading in may open with an accented letter; a word opening with a capital,
 a digit or anything else that is no small letter opens the title.

 @param run - glossed run, trimmed

 @returns Offset where the title starts

 @example
 ```ts
 leadInEnd({ run: 'sung in Cat Talk', },); // 8
 ```
 */
function leadInEnd({ run, }: { readonly run: string; },): number {
  for (let at = 0; at < run.length; at += 1) {
    /**
     Whether a word starts here: no space, at the run's start or after one.
     */
    const startsWord = (run.charAt(at,) !== WORD_SEPARATOR) && ((at === 0) || (run.charAt(at - 1,) === WORD_SEPARATOR));
    if (startsWord && (!isSmallLetter({ character: codePointAt({
      text: run,
      at,
    },), },)))
      return at;
  }
  return run.length;
}

/**
 Rewrite of one located rendering to the heading's: the span it replaces
 and with what, a glossed run's lead-in words kept outside the span; `same`
 where it already is the heading's, and `ambiguous` with why where it cannot
 be read as the title alone.

 @param text - page text of the slice

 @param located - where the rendering stands

 @param heading - heading the reference points at

 @returns Span rewritten with what changed, `same` or `ambiguous`

 @example
 ```ts
 const outcome = rewriteLocated({ text, located, heading, },);
 ```
 */
function rewriteLocated(
  {
    text,
    located,
    heading,
  }: {
    readonly text: string;
    readonly located: LocatedTitle;
    readonly heading: RenderedHeading;
  },
): RewriteOutcome {
  /**
   Rendering as the slice wrote it.
   */
  const current = text.slice(
    located.start,
    located.end,
  );
  /**
   Trailing punctuation a quoted span keeps.
   */
  const trailing = TRAILING_MARKS.has(current.slice(-1,),) ? current.slice(-1,) : '';
  /**
   Rendering without the trailing punctuation.
   */
  const core = current.slice(
    0,
    current.length - trailing.length,
  );
  /**
   Whether a glossed run ends with the heading's rendering as whole words.

   ONLY A GLOSSED RUN (ledger B23). The run before a Han gloss cannot be told
   from the words ahead of it ("from the Afternoon Cat Murmurs"), so it reads
   as the heading's when it ends with it; a quoted, bracketed or linked span
   is the title alone, and one that only ends with the heading's rendering
   ("Evening Cat Murmurs") is another rendering. The end is read at a word
   edge, so "Wildcat Murmurs" does not end in "cat Murmurs".
   */
  const glossEndsWithHeading = (located.kind === 'gloss') && endsWithWords({
    text: straightenQuotes({ text: core, },),
    tail: straightenQuotes({ text: heading.rendering, },),
  },);
  // THE TYPOGRAPHY FOLD (ledger B24): a reference apart from the heading only
  // in apostrophe style is the heading's, which the typography restoration
  // makes one; rewriting it would only swap one quote style for the other.
  if ((straightenQuotes({ text: core, },) === straightenQuotes({ text: heading.rendering, },)) || glossEndsWithHeading)
    return { kind: 'same', };
  /**
   Length of the words leading into the title: a glossed run's small-letter
   words ahead of it, none for a quoted, bracketed or linked span, which is
   the title alone.
   */
  const leadIn = (located.kind === 'gloss') ? leadInEnd({ run: core, },) : 0;
  /**
   Rendering of the title alone.
   */
  const title = core.slice(leadIn,);
  if ((located.kind === 'gloss') && (title === '')) {
    return {
      kind: 'ambiguous',
      reason: RUN_UNCAPITALIZED,
    };
  }
  // NO MORE WORDS THAN THE HEADING'S (ledger B58). A capitalized word leading
  // in ("From" opening a line) cannot be told from the title's first word, and
  // the two words of slack the run once had rewrote such words away; a run
  // longer than the heading's rendering is reported instead.
  if ((located.kind === 'gloss') && (wordCount({ text: title, },) > wordCount({ text: heading.rendering, },))) {
    return {
      kind: 'ambiguous',
      reason: RUN_LONGER,
    };
  }
  return {
    kind: 'rewritten',
    start: located.start + leadIn,
    end: located.end,
    replacement: `${heading.rendering}${trailing}`,
    before: title,
    after: heading.rendering,
  };
}

/**
 Page text with rewritten spans replaced, copying each untouched stretch
 once.

 @param text - page text of the slice

 @param rewrites - spans to replace, in page order and not overlapping

 @returns Text after the rewrites

 @example
 ```ts
 const rewritten = withRewrites({ text: 'See “Cat Talk”.', rewrites, },);
 ```
 */
function withRewrites(
  {
    text,
    rewrites,
  }: {
    readonly text: string;
    readonly rewrites: readonly TitleRewrite[];
  },
): string {
  /**
   Next character of the text not yet copied.
   */
  const cursor = { at: 0, };
  /**
   Pieces of the text after the rewrites.
   */
  const pieces: string[] = [];
  for (const rewrite of rewrites) {
    pieces.push(
      text.slice(
        cursor.at,
        rewrite.start,
      ),
      rewrite.replacement,
    );
    cursor.at = rewrite.end;
  }
  pieces.push(text.slice(cursor.at,),);
  return pieces.join('',);
}

/**
 Page text with every located rendering of one reference rewritten to the
 heading's, and one finding per rendering rewritten or left ambiguous, in
 page order.

 @param text - page text of the slice

 @param renderings - where the slice renders the title, in page order

 @param heading - heading the reference points at

 @param sliceIndex - slice the renderings stand in, for the findings

 @param whose - heading's title and rendering as the findings name them

 @returns Text after the rewrites, and the findings

 @throws {@link UnhandledMemberInvariantError} when a member of the rewrite outcome union has no branch here, which the compiler rules out, raised from the findings callback

 @example
 ```ts
 const rewrite = rewriteRenderings({ text, renderings, heading, sliceIndex: 3, whose, },);
 ```
 */
export function rewriteRenderings(
  {
    text,
    renderings,
    heading,
    sliceIndex,
    whose,
  }: {
    readonly text: string;
    readonly renderings: readonly LocatedTitle[];
    readonly heading: RenderedHeading;
    readonly sliceIndex: number;
    readonly whose: string;
  },
): {
  readonly text: string;
  readonly findings: readonly string[];
} {
  /**
   Rewrite of each rendering, each read against the same page text, since
   no two renderings overlap.
   */
  const outcomes = renderings.map(function rewriteOne(located,): RewriteOutcome {
    return rewriteLocated({
      text,
      located,
      heading,
    },);
  },);
  return {
    text: withRewrites({
      text,
      rewrites: outcomes.filter(function isRewrite(outcome,): outcome is TitleRewrite {
        return outcome.kind === 'rewritten';
      },),
    },),
    findings: outcomes.flatMap(function findingOf(outcome,): readonly string[] {
      if (outcome.kind === 'same')
        return [];
      if (outcome.kind === 'ambiguous')
        return [`title-reference-ambiguous (slice ${String(sliceIndex,)}: ${whose}, but ${outcome.reason})`,];
      if (outcome.kind === 'rewritten')
        return [`title-reference-unified (slice ${String(sliceIndex,)}: "${outcome.before}" to "${outcome.after}"; ${whose})`,];
      throw new UnhandledMemberInvariantError({
        what: 'title reference rewrite outcome',
        member: outcome,
      },);
    },),
  };
}

//endregion Title reference rewrite
