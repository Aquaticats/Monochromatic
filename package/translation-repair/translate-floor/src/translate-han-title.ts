import {
  bracketedTitles,
  carriesAsciiLetter,
  LINE_END,
  LINK_MIDDLE,
  LINK_OPEN,
  TITLE_CLOSE,
  TITLE_OPEN,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { withoutHtmlComments, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Han title floor
// CLASS NINETY-EIGHT (XingZ629, 2026-09-23). The house rule calls a work the
// ORIGINAL names by its official English title where one exists and by a
// translation of the title where none does, the same everywhere on the
// page. The second song credit of the XingZ60 tail (—— 雨狸【妄想症Paranoia】
// 《零重祈愿》) shipped with the song title in Han on XingZ629 where XingZ627
// and XingZ628 had written it in English, the web lookup's English on the
// sheet and a slate judge calling the Han "consistent"; XingZ619 and
// XingZ628 had shipped the Bird in a Cage link text in Han the same way. A
// title the ORIGINAL brackets in 《》 that carries Han and no Latin letter,
// standing in the candidate as written with no English beside it in
// parentheses, on a page that never wrote it, is refused HERE before any
// judge reads it. A title that carries Latin letters (【妄想症Paranoia】's
// album), a title kept in parentheses after its English, and a title the
// page itself keeps are left to the judges; comments are cut on both sides.
// A title standing as a link's whole text is never a gloss (class one
// hundred forty-five).

/**
 Opening parenthesis of a gloss.
 */
const GLOSS_OPEN = '(';

/**
 Closing parenthesis of a gloss.
 */
const GLOSS_CLOSE = ')';

/**
 Whether a parenthesis opened at an offset closes on the same line with a
 Latin letter inside.

 @param text - candidate text

 @param open - offset of the opening parenthesis

 @returns True for a gloss in Latin letters

 @example
 ```ts
 closesAsGloss({ text: '猫 (cat)', open: 2, },); // true
 ```
 */
function closesAsGloss(
  {
    text,
    open,
  }: {
    readonly text: string;
    readonly open: number;
  },
): boolean {
  for (let at = open + 1; at < text.length; at += 1) {
    /**
     Character under the scan.
     */
    const character = text.charAt(at,);
    if ((character === LINE_END) || (character === GLOSS_OPEN))
      return false;
    if (character === GLOSS_CLOSE)
      return carriesAsciiLetter({
        text,
        from: open + 1,
        to: at,
      },);
  }
  return false;
}

/**
 Offset of the first character past the spaces from an offset on.

 @param text - candidate text

 @param from - offset the spaces may start at

 @returns Offset of the first non-space character, or the text's end

 @example
 ```ts
 pastSpaces({ text: 'a  b', from: 1, },); // 3
 ```
 */
function pastSpaces(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let at = from; at < text.length; at += 1) {
    if (text.charAt(at,) !== ' ')
      return at;
  }
  return text.length;
}

/**
 Whether an occurrence of a title is followed, past an optional closing
 bracket and spaces, by a gloss in parentheses.

 @param text - candidate text

 @param end - offset just past the occurrence

 @returns True when an English gloss follows

 @example
 ```ts
 glossFollows({ text: '《猫》 (Cat)', end: 2, },); // true
 ```
 */
function glossFollows(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): boolean {
  /**
   Offset past the closing bracket, if one stands there.
   */
  const pastBracket = (text.charAt(end,) === TITLE_CLOSE) ? end + 1 : end;
  /**
   Offset of the character after the spaces.
   */
  const open = pastSpaces({
    text,
    from: pastBracket,
  },);
  if (text.charAt(open,) !== GLOSS_OPEN)
    return false;
  return closesAsGloss({
    text,
    open,
  },);
}

/**
 Offset of the parenthesis that opens on the occurrence's line before it
 with no closing one between, -1 for none.

 @param text - candidate text

 @param start - offset of the occurrence

 @returns Offset of the enclosing opening parenthesis, or -1

 @example
 ```ts
 enclosingOpen({ text: 'Cat (猫)', start: 5, },); // 4
 ```
 */
function enclosingOpen(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): number {
  for (let at = start - 1; at >= 0; at -= 1) {
    /**
     Character under the backward scan.
     */
    const character = text.charAt(at,);
    if ((character === LINE_END) || (character === GLOSS_CLOSE))
      return -1;
    if (character === GLOSS_OPEN)
      return at;
  }
  return -1;
}

/**
 Whether an occurrence of a title stands inside parentheses on its line, the
 shape of an English title followed by the Han in parentheses.

 @param text - candidate text

 @param start - offset of the occurrence

 @returns True when parentheses enclose the occurrence on one line

 @example
 ```ts
 insideParentheses({ text: 'Cat (猫)', start: 5, },); // true
 ```
 */
function insideParentheses(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): boolean {
  if (enclosingOpen({
    text,
    start,
  },) === (-1))
    return false;
  /**
   Where the enclosing parenthesis closes, -1 for never.
   */
  const close = text.indexOf(
    GLOSS_CLOSE,
    start,
  );
  if (close === (-1))
    return false;
  return !text.slice(
    start,
    close,
  )
    .includes(LINE_END,);
}

/**
 Whether an occurrence of a title is a Markdown link's whole text, which the
 reader sees as the work's name whatever stands around the link.

 CLASS ONE HUNDRED FORTY-FIVE (XingZ6013, 2026-09-26): the consolidated
 candidate wrote “Gilded Cage” ([笼中之鸟](url)), the Han the link's text in
 parentheses after a web-lookup English title, and read as a gloss it passed.

 @param text - candidate text

 @param title - Han title as the original brackets it

 @param start - offset of the occurrence

 @returns True when the occurrence opens and closes a link's text

 @example
 ```ts
 isLinkText({ text: 'Cat ([猫](https://example.test))', title: '猫', start: 6, },); // true
 ```
 */
function isLinkText(
  {
    text,
    title,
    start,
  }: {
    readonly text: string;
    readonly title: string;
    readonly start: number;
  },
): boolean {
  if (text.charAt(start - 1,) !== LINK_OPEN)
    return false;
  return text.startsWith(
    LINK_MIDDLE,
    start + title.length,
  );
}

/**
 Whether an occurrence of a title stands bare: a link's whole text, or with
 no gloss after it and no parentheses around it.

 @param text - candidate text

 @param title - Han title as the original brackets it

 @param start - offset of the occurrence

 @returns True when the occurrence stands bare

 @example
 ```ts
 standsBare({ text: 'Her song was 《猫猫摇篮曲》.', title: '猫猫摇篮曲', start: 14, },); // true
 ```
 */
function standsBare(
  {
    text,
    title,
    start,
  }: {
    readonly text: string;
    readonly title: string;
    readonly start: number;
  },
): boolean {
  if (isLinkText({
    text,
    title,
    start,
  },))
    return true;
  if (glossFollows({
    text,
    end: start + title.length,
  },))
    return false;
  return !insideParentheses({
    text,
    start,
  },);
}

/**
 Whether a candidate carries a title in Han with no English beside it.

 @param text - candidate text with its comments cut

 @param title - Han title as the original brackets it

 @returns True when some occurrence stands bare

 @example
 ```ts
 carriesBare({ text: 'Her song was 《猫猫摇篮曲》.', title: '猫猫摇篮曲', },); // true
 ```
 */
function carriesBare(
  {
    text,
    title,
  }: {
    readonly text: string;
    readonly title: string;
  },
): boolean {
  for (
    let start = text.indexOf(title,);
    start !== (-1);
    start = text.indexOf(
      title,
      start + title.length,
    )
  ) {
    if (standsBare({
      text,
      title,
      start,
    },))
      return true;
  }
  return false;
}

/**
 Offsets of the occurrences of a title this floor accepts: those not
 standing bare.

 @param text - candidate text

 @param title - Han title as the original brackets it

 @returns Offsets in order

 @example
 ```ts
 acceptedStarts({ text: '《猫》 (Cat)', title: '猫', },); // [1]
 ```
 */
function acceptedStarts(
  {
    text,
    title,
  }: {
    readonly text: string;
    readonly title: string;
  },
): readonly number[] {
  /**
   Accepted offsets found so far.
   */
  const starts: number[] = [];
  for (
    let start = text.indexOf(title,);
    start !== (-1);
    start = text.indexOf(
      title,
      start + title.length,
    )
  ) {
    if (!standsBare({
      text,
      title,
      start,
    },))
      starts.push(start,);
  }
  return starts;
}

/**
 A text with a title cut out at some offsets.

 @param text - candidate text

 @param title - title to cut

 @param starts - offsets of the occurrences to cut, in order

 @returns The text without those occurrences

 @example
 ```ts
 cutAt({ text: '《猫》 (Cat)', title: '猫', starts: [1,], },); // '《》 (Cat)'
 ```
 */
function cutAt(
  {
    text,
    title,
    starts,
  }: {
    readonly text: string;
    readonly title: string;
    readonly starts: readonly number[];
  },
): string {
  /**
   Where each kept piece begins: the text's start, then just past each cut.
   */
  const pieceStarts = [
    0,
    ...starts.map(function pastCut(start,): number {
      return start + title.length;
    },),
  ];
  return pieceStarts
    .map(function piece(
      from,
      index,
    ): string {
      return text.slice(
        from,
        starts[index] ?? text.length,
      );
    },)
    .join('',);
}

/**
 A candidate with every occurrence of the original's Han-carrying titles
 that this floor's shape accepts (glossed, or inside parentheses after the
 English) cut out, so another floor reading the candidate for Han words does
 not refuse a word inside a title kept the way this floor allows (ledger
 F-11: 《高考猫》 (The Exam Cat) refused for 高考). A bare occurrence is left
 for the floors that read it.

 TITLES CARRYING LATIN LETTERS ARE CUT TOO (ledger F-3), though this floor's
 findings read only Han-only titles: a glossed 《喵萌DX》 (Meowmeow DX) passes
 the Han residue floor as a glossed Han-only title does, and a bare one is
 left for that floor to refuse, where before it fell between this floor and
 the Latin title floor, each naming the other.

 @param sourceText - original passage

 @param candidateText - rendering, comments already cut

 @returns The rendering without its accepted title occurrences

 @example
 ```ts
 withoutGlossedTitles({ sourceText: '《高考猫》', candidateText: '《高考猫》 (The Exam Cat)', },);
 // => '《》 (The Exam Cat)'
 ```
 */
export function withoutGlossedTitles(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): string {
  return bracketedTitles({
    text: withoutHtmlComments({ text: sourceText, },),
    form: 'carrying-han',
  },)
    .reduce(
    function cutGlossed(
      text,
      title,
    ): string {
      return cutAt({
        text,
        title,
        starts: acceptedStarts({
          text,
          title,
        },),
      },);
    },
    candidateText,
    );
}

/**
 Findings against a candidate that leaves a work's title the original
 brackets in 《》 in Han, one per title; empty where every such title is
 rendered, glossed, or kept by the page the candidate would replace.

 @param sourceText - original passage

 @param candidateText - rendering under the floor

 @param pageText - text the rendering would replace, empty where the page
 has none

 @returns Findings, empty where the candidate passes

 @example
 ```ts
 hanTitleFindings({
   sourceText: '她最爱的歌是《猫猫摇篮曲》。',
   candidateText: 'Her favourite song was 《猫猫摇篮曲》.',
   pageText: '',
 },); // one finding
 ```
 */
export function hanTitleFindings(
  {
    sourceText,
    candidateText,
    pageText = '',
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly pageText?: string;
  },
): readonly string[] {
  /**
   Candidate with its comments cut.
   */
  const candidate = withoutHtmlComments({ text: candidateText, },);
  return bracketedTitles({
    text: withoutHtmlComments({ text: sourceText, },),
    form: 'han-only',
  },)
    .filter(function leftBare(title,): boolean {
      if (pageText.includes(title,))
        return false;
      return carriesBare({
        text: candidate,
        title,
      },);
    },)
    .map(function toFinding(title,): string {
      return `Your translation leaves the title ${TITLE_OPEN}${title}${TITLE_CLOSE} in Han: a work the ORIGINAL names is called by its English title on the page, the official English title where one exists and a translation of the title where none does, and the Han never stands alone as the name.`;
    },);
}

//endregion Han title floor
