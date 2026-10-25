import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Tally error text
// FAILURE TEXT FOR A STDOUT LINE. A class that declared its message quote-free
// says it, anything else is named and not quoted, since stdout is read,
// grepped, and pasted; capped after that so one runaway message
// cannot swallow the line a reader counts by.
//
// A CORPUS READ'S KIND AND REMEDY ARE NEVER CUT. The cap applies to what
// precedes them, the path and revision asked for, and the closing follows
// whole: it is one of four fixed sentences, so it cannot run away. Until
// 2026-10-06 a long entry id or picture name pushed the remedy, and for a
// longer one the kind, past the cap, and the line told the reader neither
// what was wrong nor what to do.
//
// A CUT OPENING ENDS ON A MARK (`CUT_MARK`), inside the cap, so a path cut
// short never reads as a whole one followed by advice to check it.

/**
 Most UTF-16 units of an error message's opening kept in a `TALLY` or
 `CLEANUP` line, ending on a whole character (`wholeOpening`); a corpus
 read's kind and remedy follow it uncut (`cappedFailureText`), so such a
 line carries up to that closing's length more.
 */
export const TALLY_ERROR_CAP = 200;

/**
 What ends an opening that was cut before a corpus read's kind and remedy,
 so the reader sees the path asked for was longer than the line shows.
 */
const CUT_MARK = '…';

/**
 Renders a caught value for a stdout line, named or quoted per its class, its
 opening capped and a corpus read's kind and remedy kept whole after it, a
 cut opening of a corpus read ending on `CUT_MARK`.

 @param error - what was caught, of unknown type by construction

 @param units - most UTF-16 units of the message's opening the line keeps,
 a cut opening's mark included, which the printing line's own cap sets

 @returns Text safe to print on a line a reader greps

 @throws Error marked unreachable when a corpus read's message does not end
 on the kind and remedy its class built it with

 @example
 ```ts
 const text = cappedFailureText({ error, units: TALLY_ERROR_CAP, },);
 ```
 */
export function cappedFailureText(
  {
    error,
    units,
  }: {
    readonly error: unknown;
    readonly units: number;
  },
): string {
  /**
   The caught value as a line may carry it, before any cut.
   */
  const text = refusalText({ error, },);
  if (!(error instanceof CorpusReadError)) {
    return wholeOpening({
      text,
      units,
    },);
  }
  /**
   The kind and remedy with the space that parts them from the path asked for.
   */
  const closing = ` ${error.kindAndRemedy}`;
  if (!text.endsWith(closing,)) {
    throw new Error(
      'unreachable: a corpus read refusal\'s message does not end on its kind and remedy, which its '
        + 'constructor writes as the message\'s closing',
    );
  }
  /**
   The path and revision asked for, before the closing.
   */
  const opening = text.slice(
    0,
    text.length - closing.length,
  );
  if (opening.length <= units)
    return text;

  // Cut one unit short of the cap so the mark fits inside it.
  return `${
    wholeOpening({
      text: opening,
      units: units - CUT_MARK.length,
    },)
  }${CUT_MARK}${closing}`;
}

/**
 Renders a caught value for a stdout line, named or quoted per its class and
 capped, a corpus read's kind and remedy whole.

 @param error - what was caught

 @returns Text safe to print on a line a reader greps

 @example
 ```ts
 console.log(`TALLY ${id} status=ERROR ${lifecycleStamp({ attempt, },)} error=${tallyErrorText({ error, },)}`,);
 ```
 */
export function tallyErrorText({ error, }: { readonly error: unknown; },): string {
  return cappedFailureText({
    error,
    units: TALLY_ERROR_CAP,
  },);
}

//endregion Tally error text
