import { locateLinks, } from './title-reference-link.ts';
import { locateMarked, } from './title-reference-marks.ts';
import {
  type LocatedTitle,
  referenceScope,
  type TitleLocation,
  type TitleLocations,
} from './title-reference-scope.ts';

export type {
  LocatedTitle,
  TitleLocation,
  TitleLocations,
} from './title-reference-scope.ts';

//region Title reference locate
// WHERE A RENDERING OF A BRACKETED TITLE STANDS IN A SLICE'S PAGE TEXT. The
// original brackets a section title as a link (`《[title](url)》`), in
// title brackets (`《title》`) or in corner brackets (`「title」篇`); the
// bench renders it as a link with the same destination, as the English
// with the Han in parentheses after it, in title brackets, or in quotes.
// Each shape is found by index scan. Each link of the title is the page's
// link at that link's place among the links to its destination
// (title-reference-link.ts); every gloss names the title, so each is read
// (ledger B59); and a slice that offers two bracketed or quoted spans, or
// links the destination a different number of times than the original, is
// reported ambiguous rather than guessed at.

/**
 Opening title bracket.
 */
const TITLE_OPEN = '《';

/**
 Closing title bracket.
 */
const TITLE_CLOSE = '》';

/**
 Opening parenthesis of a gloss.
 */
const GLOSS_OPEN = '(';

/**
 Closing parenthesis of a gloss.
 */
const GLOSS_CLOSE = ')';

/**
 Newline, which no span crosses.
 */
const LINE_END = '\n';

/**
 Quote pairs a title may stand in, opening then closing.
 */
const QUOTE_PAIRS: readonly (readonly [
  string,
  string,
])[] = [
  [
    '“',
    '”',
  ],
  [
    '"',
    '"',
  ],
];

/**
 Characters that end a gloss's English run when read leftward.
 */
const RUN_BOUNDARIES: ReadonlySet<string> = new Set([
  ',',
  ';',
  ':',
  '“',
  '”',
  '"',
  '‘',
  '’',
  '《',
  '》',
  '【',
  '】',
  '「',
  '」',
  '『',
  '』',
  '(',
  ')',
  '—',
  '–',
  LINE_END,
],);

/**
 Every place the page renders the title as English with the Han in
 parentheses after it: the English run before each parenthesis, in page
 order, a gloss with no run before it skipped.

 EVERY GLOSS (ledger B59). Each gloss names the title, so each run before
 one renders it; reading only the first left a second credit's rendering as
 it stood.

 @param pageText - page text of the slice

 @param title - Han title

 @returns Located runs, empty for none

 @example
 ```ts
 locateGlosses({ pageText: '—— Yumao, Caged Cat (笼中猫)', title: '笼中猫', },); // one run, 10 to 19
 ```
 */
function locateGlosses(
  {
    pageText,
    title,
  }: {
    readonly pageText: string;
    readonly title: string;
  },
): readonly LocatedTitle[] {
  /**
   Gloss after a rendering.
   */
  const gloss = `${GLOSS_OPEN}${title}${GLOSS_CLOSE}`;
  /**
   Runs found so far.
   */
  const runs: LocatedTitle[] = [];
  for (
    let open = pageText.indexOf(gloss,);
    open !== (-1);
    open = pageText.indexOf(
      gloss,
      open + gloss.length,
    )
  ) {
    /**
     Offset just past the run, before the spaces ahead of the parenthesis.
     */
    const end = trimmedEnd({
      pageText,
      from: open,
    },);
    /**
     Offset of the run's first character.
     */
    const start = runStart({
      pageText,
      end,
    },);
    if (start < end) {
      runs.push({
        kind: 'gloss',
        start,
        end,
      },);
    }
  }
  return runs;
}

/**
 Offset just past the last non-space character before an offset.

 @param pageText - page text of the slice

 @param from - offset the spaces end at

 @returns Offset just past the run

 @example
 ```ts
 trimmedEnd({ pageText: 'Cat  (猫)', from: 5, },); // 3
 ```
 */
function trimmedEnd(
  {
    pageText,
    from,
  }: {
    readonly pageText: string;
    readonly from: number;
  },
): number {
  for (let at = from; at > 0; at -= 1) {
    if (pageText.charAt(at - 1,) !== ' ')
      return at;
  }
  return 0;
}

/**
 Offset of the first character of the run that ends at an offset, read
 leftward to a boundary character or the line's start, spaces trimmed.

 @param pageText - page text of the slice

 @param end - offset just past the run

 @returns Offset of the run's first non-space character

 @example
 ```ts
 runStart({ pageText: '—— Yumao, Caged Cat (笼中猫)', end: 19, },); // 10
 ```
 */
function runStart(
  {
    pageText,
    end,
  }: {
    readonly pageText: string;
    readonly end: number;
  },
): number {
  /**
   Offset of the boundary the run stops at.
   */
  const boundary = boundaryBefore({
    pageText,
    end,
  },);
  for (let at = boundary; at < end; at += 1) {
    if (pageText.charAt(at,) !== ' ')
      return at;
  }
  return end;
}

/**
 Offset just past the nearest boundary character before an offset, zero
 for none.

 @param pageText - page text of the slice

 @param end - offset the leftward read starts at

 @returns Offset just past the boundary

 @example
 ```ts
 boundaryBefore({ pageText: 'a, b', end: 4, },); // 2
 ```
 */
function boundaryBefore(
  {
    pageText,
    end,
  }: {
    readonly pageText: string;
    readonly end: number;
  },
): number {
  for (let at = end; at > 0; at -= 1) {
    if (RUN_BOUNDARIES.has(pageText.charAt(at - 1,),))
      return at;
  }
  return 0;
}

/**
 Outcome of a search inside a scope, a located span moved by the scope's
 offset so it is reported against the whole text.

 @param location - location inside the scope

 @param by - offset of the scope's first character

 @returns The one rendering against the whole text, or ambiguous or none as
 found

 @example
 ```ts
 shifted({ location: { kind: 'quote', start: 1, end: 4, }, by: 10, },); // one rendering, 11 to 14
 ```
 */
function shifted(
  {
    location,
    by,
  }: {
    readonly location: TitleLocation;
    readonly by: number;
  },
): TitleLocations {
  if ((location.kind === 'none') || (location.kind === 'ambiguous'))
    return location;
  return {
    kind: 'located',
    renderings: [
      {
        kind: location.kind,
        start: location.start + by,
        end: location.end + by,
      },
    ],
  };
}

/**
 Where a slice's page text renders a title the original brackets: by the
 link's destination first, then by every Han gloss after the English, then
 by title brackets, then by quotes, the bracket and quote searches held to
 the page's definition line where the original references the title in a
 footnote definition.

 @param sourceText - original text of the slice, comments cut

 @param pageText - page text of the slice

 @param title - Han title as the original writes it

 @param rendering - heading's rendering the reference should carry

 @returns Located renderings in page order, ambiguous, or none

 @example
 ```ts
 locateTitleRendering({ sourceText: '《猫》', pageText: '《Cat》', title: '猫', rendering: 'Cat', },); // one bracket, 1 to 4
 ```
 */
export function locateTitleRendering(
  {
    sourceText,
    pageText,
    title,
    rendering,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly title: string;
    readonly rendering: string;
  },
): TitleLocations {
  /**
   Link texts, where the page links the destinations the original does.
   */
  const linked = locateLinks({
    sourceText,
    pageText,
    title,
  },);
  if (linked.kind !== 'none')
    return linked;
  /**
   English runs before the Han gloss, where the page glosses.
   */
  const glossed = locateGlosses({
    pageText,
    title,
  },);
  if (glossed.length > 0) {
    return {
      kind: 'located',
      renderings: glossed,
    };
  }
  /**
   Span the bracket and quote searches read.
   */
  const scope = referenceScope({
    sourceText,
    pageText,
    title,
  },);
  /**
   Page text inside the scope.
   */
  const scoped = pageText.slice(
    scope.start,
    scope.end,
  );
  /**
   Title-bracketed span, where the page brackets.
   */
  const bracketed = locateMarked({
    pageText: scoped,
    pairs: [
      [
        TITLE_OPEN,
        TITLE_CLOSE,
      ],
    ],
    kind: 'bracket',
    rendering,
  },);
  if (bracketed.kind !== 'none')
    return shifted({
      location: bracketed,
      by: scope.start,
    },);
  return shifted({
    location: locateMarked({
      pageText: scoped,
      pairs: QUOTE_PAIRS,
      kind: 'quote',
      rendering,
    },),
    by: scope.start,
  },);
}

//endregion Title reference locate
