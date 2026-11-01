import {
  noteRefusedMask,
  type ProseMaskReading,
  proseMaskReading,
  textNamed,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Ellipsis restoration
// Puts back the ellipsis form the document uses, the way the quote rules put
// back its quote style.
//
// Found on the yulianNyanner page of 2026-09-07: one U+2026 against eleven
// three-dot ellipses on an archive that writes three dots throughout, from a
// consolidation wording that kept the source's `……`. Measured at pin a41fc607:
// 28 archives write three dots only, 14 write U+2026 only, 11 write both, and
// 39 write neither, so the corpus has no one convention and this rule reads
// each document's own. Where a document shows both forms the rule says
// nothing, since it cannot tell which the editors meant.

/**
 Horizontal ellipsis, U+2026.
 */
const UNICODE_ELLIPSIS = '\u{2026}';

/**
 Three-dot ellipsis.
 */
const THREE_DOTS = '...';

/**
 Which ellipsis form a text shows.

 @param text - text to read

 @returns `'dots'`, `'unicode'`, `'both'` or `'none'`

 @example
 ```ts
 ellipsisFormOf({ text: 'Well... no.', },);
 ```
 */
function ellipsisFormOf(
  { text, }: { readonly text: string; },
): 'dots' | 'unicode' | 'both' | 'none' {
  /**
   Whether three dots appear.
   */
  const dots = text.includes(THREE_DOTS,);

  /**
   Whether U+2026 appears.
   */
  const unicode = text.includes(UNICODE_ELLIPSIS,);
  if (dots && unicode)
    return 'both';
  if (dots)
    return 'dots';
  if (unicode)
    return 'unicode';
  return 'none';
}

/**
 The prose reading of a replacement: the one a caller handed on, else the reading of the text, logged when the grammar refuses it.

 @param replacement - text a stage wrote

 @param given - the reading a caller made already

 @returns The reading of the replacement

 @example
 ```ts
 const { mask, } = readingOf({ replacement: 'Well... no.', },);
 ```
 */
function readingOf(
  {
    replacement,
    given,
  }: {
    readonly replacement: string;
    readonly given?: ProseMaskReading;
  },
): ProseMaskReading {
  if (given !== undefined)
    return given;
  /**
   The reading of the text.
   */
  const reading = proseMaskReading({ text: replacement, },);
  if (reading.refused) {
    noteRefusedMask({
      reader: 'restoreEllipsis',
      name: textNamed({ text: replacement, },),
    },);
  }
  return reading;
}

/**
 Restores the ellipsis form the replaced text and the wider document use.

 Converts only when the replaced region and the document together show one
 form and never the other. A run of U+2026, which Chinese writes doubled,
 becomes one three-dot ellipsis; a run of exactly three dots becomes one
 U+2026. Only the text the page parse shows as prose is touched
 (`typography-prose-mask.ts`): code of any kind, an address, an expression, a tag and its attributes are never.

 @param replacement - text a stage wrote

 @param replaced - text it replaces

 @param convention - wider text whose form the replacement should match

 @param reading - the replacement's prose mask, which a caller that read it already hands on (the same units, whatever marks it
 curled since); read from the replacement where absent, and a refusal of the grammar is then logged

 @returns Replacement in the document's ellipsis form

 @example
 ```ts
 restoreEllipsis({ replacement: 'Well\u{2026} no.', replaced: 'Well... yes.', convention: documentText, },);
 ```
 */
export function restoreEllipsis(
  {
    replacement,
    replaced,
    convention,
    reading: given,
  }: {
    readonly replacement: string;
    readonly replaced: string;
    readonly convention: string;
    readonly reading?: ProseMaskReading;
  },
): string {
  /**
   Form the replaced region and the document show together.
   */
  const form = ellipsisFormOf({ text: `${replaced}\n${convention}`, },);
  if ((form === 'both') || (form === 'none'))
    return replacement;

  /**
   Which units of the replacement are prose.
   */
  const { mask, } = readingOf({
    replacement,
    ...((given === undefined) ? {} : { given, }),
  },);

  return (function scan(): string {
    /**
     Pieces emitted so far.
     */
    const rebuilt: string[] = [];
    for (let index = 0; index < replacement.length; index += 1) {
      /**
       Character under the cursor.
       */
      const character = replacement.charAt(index,);
      if (mask[index] !== true) {
        rebuilt.push(character,);
        continue;
      }
      if ((form === 'dots') && (character === UNICODE_ELLIPSIS)) {
        // A run of U+2026 collapses to one three-dot ellipsis; skip the rest
        // of the run.
        if (replacement.charAt(index - 1,) !== UNICODE_ELLIPSIS)
          rebuilt.push(THREE_DOTS,);
        continue;
      }
      if ((form === 'unicode') && (character === '.')) {
        /**
         Whether a run of exactly three dots starts here.
         */
        const exactlyThree = replacement.startsWith(
          THREE_DOTS,
          index,
        )
          && (replacement.charAt(index - 1,) !== '.')
          && (replacement.charAt(index + THREE_DOTS.length,) !== '.');
        if (exactlyThree) {
          rebuilt.push(UNICODE_ELLIPSIS,);
          index += THREE_DOTS.length - 1;
          continue;
        }
      }
      rebuilt.push(character,);
    }
    return rebuilt.join('',);
  })();
}

//endregion Ellipsis restoration
