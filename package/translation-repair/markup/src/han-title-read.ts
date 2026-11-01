import { closedMarkSpans, } from './closed-mark-spans.ts';
import {
  carriesHan,
  isHanOnly,
} from './han-only-text.ts';

//region Han title reading
// The titles an original brackets in 《》, read once for the floors that ask
// about them: the Han title floor's findings read the titles written in Han
// alone, and the cut other floors read a candidate through takes every title
// carrying Han (`translate-han-title.ts`, ledger F-3). Split out of the
// title floor when that file reached the line budget.

/**
 Opening title bracket.
 */
export const TITLE_OPEN = '《';

/**
 Closing title bracket.
 */
export const TITLE_CLOSE = '》';

/**
 Newline, which no title or gloss crosses.
 */
export const LINE_END = '\n';

/**
 Opening of a Markdown link's text.
 */
export const LINK_OPEN = '[';

/**
 Separator between a Markdown link's text and its destination.
 */
export const LINK_MIDDLE = '](';

/**
 Closing of a Markdown link's destination.
 */
const LINK_CLOSE = ')';

/**
 Title as it names the work: the link text where the brackets hold a
 Markdown link, the bracketed text itself otherwise.

 @param bracketed - text between 《 and 》

 @returns Text that names the work

 @example
 ```ts
 titleText({ bracketed: '[猫猫摇篮曲](https://example.test/song)', },); // '猫猫摇篮曲'
 ```
 */
export function titleText({ bracketed, }: { readonly bracketed: string; },): string {
  /**
   Whether the brackets hold a link's shape end to end.
   */
  const linkShaped = bracketed.startsWith(LINK_OPEN,) && bracketed.endsWith(LINK_CLOSE,);
  if (!linkShaped)
    return bracketed;
  /**
   Where the link text ends, -1 for no link.
   */
  const middle = bracketed.indexOf(LINK_MIDDLE,);
  if (middle === (-1))
    return bracketed;
  return bracketed.slice(
    LINK_OPEN.length,
    middle,
  );
}

/**
 Form a bracketed title's text must take for a reader to read it: Han alone
 for this floor's findings, or any Han at all for the cut other floors read
 through.
 */
export type TitleForm = 'han-only' | 'carrying-han';

/**
 Test each form applies to a title's text.
 */
const TITLE_FORM_TESTS: Readonly<Record<TitleForm, (input: { readonly text: string; },) => boolean>> = {
  'han-only': isHanOnly,
  'carrying-han': carriesHan,
};

/**
 Whether a bracketed title is one a reader takes: on one line, in the form
 asked for, and not read already.

 @param title - text between the brackets

 @param titles - titles read so far

 @param form - form the title must take

 @returns True for a fresh title of that form

 @example
 ```ts
 isFreshTitle({ title: '猫猫摇篮曲', titles: [], form: 'han-only', },); // true
 ```
 */
function isFreshTitle(
  {
    title,
    titles,
    form,
  }: {
    readonly title: string;
    readonly titles: readonly string[];
    readonly form: TitleForm;
  },
): boolean {
  if (title.includes(LINE_END,))
    return false;
  if (titles.includes(title,))
    return false;
  return TITLE_FORM_TESTS[form]({ text: title, },);
}

/**
 Every distinct title an original brackets in 《》 in the form asked for, in
 order of first appearance.

 @param text - original passage with its comments cut

 @param form - Han alone, or any Han at all

 @returns Titles without their brackets

 @example
 ```ts
 bracketedTitles({ text: '她最爱的歌是《猫猫摇篮曲》。', form: 'han-only', },); // ['猫猫摇篮曲']
 ```
 */
export function bracketedTitles(
  {
    text,
    form,
  }: {
    readonly text: string;
    readonly form: TitleForm;
  },
): readonly string[] {
  /**
   Titles read so far.
   */
  const titles: string[] = [];
  // A MARK THAT NEVER CLOSED brackets nothing (ledger B38).
  for (const span of closedMarkSpans({
    text,
    open: TITLE_OPEN,
    close: TITLE_CLOSE,
  },)) {
    /**
     Title between the brackets, the link text where they hold a link.
     */
    const title = titleText({ bracketed: text.slice(
      span.open + TITLE_OPEN.length,
      span.close,
    ), },);
    if (isFreshTitle({
      title,
      titles,
      form,
    },))
      titles.push(title,);
  }
  return titles;
}

//endregion Han title reading
