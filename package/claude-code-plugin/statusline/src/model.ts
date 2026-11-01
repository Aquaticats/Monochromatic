/**
 Claude model display formatting.
 
 @module
 */

//region Defaults

/**
 Latest versions and default context sizes per model family.
 */
const MODEL_DEFAULTS: Record<
  string,
  {
    readonly latestVersion: string;
    readonly defaultContext: string;
  }
  > = {
    Opus: {
      latestVersion: '4.8',
      defaultContext: '1M',
    },
    Sonnet: {
      latestVersion: '4.6',
      defaultContext: '200K',
    },
    Haiku: {
      latestVersion: '4.5',
      defaultContext: '200K',
    },
  };

//endregion Defaults

//region Display name grammar

/**
 Separator between the family, version, and context pieces of a display name.
 */
const PIECE_SEPARATOR = ' ';

/**
 Character opening the context piece of a display name.
 */
const CONTEXT_OPENING = '(';

/**
 Text closing the context piece, which also ends the whole display name.
 */
const CONTEXT_CLOSING = ' context)';

/**
 Separator between a version's major and minor numbers.
 */
const VERSION_SEPARATOR = '.';

/**
 Characters treated as whitespace, exactly the set the replaced `\S` class excluded: the ASCII
 control spaces, space, no-break space, Ogham space mark, the U+2000 to U+200A typographic spaces,
 line and paragraph separators, narrow no-break space, medium mathematical space, ideographic space,
 and the byte-order mark.
 */
const WHITESPACE_CHARACTERS = [
  '\t\n\v\f\r ',
  '\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A',
  '\u2028\u2029\u202F\u205F\u3000\uFEFF',
].join('',);

/**
 Pieces of a display name that fits the family, version, and context grammar.
 
 An empty `version` or `context` means the display name omitted that piece. An empty `family` means
 the whole name failed to parse, since the grammar requires at least one letter there.
 */
type DisplayNamePieces = {
  /**
   Model family name, such as `Sonnet`.
   */
  readonly family: string;
  /**
   Version number in major.minor form, empty when the display name omitted it.
   */
  readonly version: string;
  /**
   Context size, empty when the display name omitted it.
   */
  readonly context: string;
};

/**
 Pieces reported for a display name that does not fit the grammar.
 */
const UNPARSED_DISPLAY_NAME: DisplayNamePieces = {
  family: '',
  version: '',
  context: '',
};

/**
 Checks whether one character is an ASCII letter, the only character a family name holds.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character sits in `a` through `z` or `A` through `Z`
 
 @example
 ```ts
 isAsciiLetter('S'); // true
 ```
 */
function isAsciiLetter(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z')) || ((character >= 'A') && (character <= 'Z'));
}

/**
 Checks whether one character is an ASCII decimal digit, the only character a version number holds.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character sits in `0` through `9`
 
 @example
 ```ts
 isAsciiDigit('4'); // true
 ```
 */
function isAsciiDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Checks whether one character is whitespace, so a context piece can be told where it ends.
 
 The empty string reports false, which keeps the past-the-end index a caller passes from counting as
 whitespace and swallowing the rest of the text.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether character is one of {@link WHITESPACE_CHARACTERS}
 
 @example
 ```ts
 isWhitespaceCharacter('\t'); // true
 ```
 */
function isWhitespaceCharacter(character: string,): boolean {
  return (character.length === 1) && WHITESPACE_CHARACTERS.includes(character,);
}

/**
 Finds where the ASCII letter run starting at one index ends.
 
 @param text - display name to scan
 
 @param start - index the run begins at, which may already sit past the end
 
 @returns index one past the last letter of the run, so `start` itself when there is none
 
 @example
 ```ts
 asciiLetterRunEnd({ text: 'Sonnet 4.6', start: 0 }); // 6
 ```
 */
function asciiLetterRunEnd({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>,): number {
  /**
   Index the scan has reached.
   */
  let cursor = start;
  while (isAsciiLetter(text.charAt(cursor,),))
    cursor += 1;
  return cursor;
}

/**
 Finds where the decimal digit run starting at one index ends.
 
 @param text - display name to scan
 
 @param start - index the run begins at, which may already sit past the end
 
 @returns index one past the last digit of the run, so `start` itself when there is none
 
 @example
 ```ts
 digitRunEnd({ text: '4.6', start: 0 }); // 1
 ```
 */
function digitRunEnd({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>,): number {
  /**
   Index the scan has reached.
   */
  let cursor = start;
  while (isAsciiDigit(text.charAt(cursor,),))
    cursor += 1;
  return cursor;
}

/**
 Finds where the whitespace-free run starting at one index ends.
 
 @param text - display name to scan
 
 @param start - index the run begins at, which may already sit past the end
 
 @returns index one past the last whitespace-free character, so `start` itself when there is none
 
 @example
 ```ts
 nonWhitespaceRunEnd({ text: '1M context)', start: 0 }); // 2
 ```
 */
function nonWhitespaceRunEnd({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>,): number {
  /**
   Index the scan has reached.
   */
  let cursor = start;
  while ((cursor < text.length) && (!isWhitespaceCharacter(text.charAt(cursor,),)))
    cursor += 1;
  return cursor;
}

/**
 Finds where a major.minor version starting at one index ends.
 
 Both numbers need at least one digit, so `4.` reports no version. A third number is not this
 function's concern: it reports the end of the `1.2` inside `1.2.3` and leaves the caller to reject
 the leftover `.3` as text the grammar does not allow.
 
 @param text - display name to scan
 
 @param start - index the version would begin at
 
 @returns index one past the last digit of the version, or `start` when the text there is not one
 
 @example
 ```ts
 versionPieceEnd({ text: '4.6 (1M context)', start: 0 }); // 3
 ```
 */
function versionPieceEnd({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>,): number {
  /**
   End of the major number.
   */
  const majorEnd = digitRunEnd({
    text,
    start,
  },);
  if ((majorEnd === start) || (text.charAt(majorEnd,) !== VERSION_SEPARATOR))
    return start;
  /**
   Index the minor number begins at.
   */
  const minorStart = majorEnd + 1;
  /**
   End of the minor number.
   */
  const minorEnd = digitRunEnd({
    text,
    start: minorStart,
  },);
  if (minorEnd === minorStart)
    return start;
  return minorEnd;
}

/**
 Parses a display name into its family, version, and context pieces.
 
 The version and context pieces are optional and both begin with a separator, so at most one of them
 can follow any given position; a single left-to-right pass is enough and nothing backtracks.
 
 @param raw - model display name from Claude Code
 
 @returns parsed pieces, or {@link UNPARSED_DISPLAY_NAME} when the name does not fit the grammar
 
 @example
 ```ts
 parseDisplayName('Sonnet 4.6 (1M context)');
 ```
 */
function parseDisplayName(raw: string,): DisplayNamePieces {
  /**
   End of the leading family name.
   */
  const familyEnd = asciiLetterRunEnd({
    text: raw,
    start: 0,
  },);
  if (familyEnd === 0)
    return UNPARSED_DISPLAY_NAME;
  /**
   Family name, kept for the assembled pieces.
   */
  const family = raw.slice(
    0,
    familyEnd,
  );
  /**
   Index the version piece would begin at, just past its separator.
   */
  const versionStart = familyEnd + 1;
  /**
   End of the version piece, or `versionStart` when the text there is not a version.
   */
  const versionEnd = (raw.charAt(familyEnd,) === PIECE_SEPARATOR)
    ? versionPieceEnd({
      text: raw,
      start: versionStart,
    },)
    : versionStart;
  /**
   Whether the display name carried a version piece.
   */
  const hasVersion = versionEnd > versionStart;
  /**
   Version piece, empty when absent.
   */
  const version = hasVersion
    ? raw.slice(
      versionStart,
      versionEnd,
    )
    : '';
  /**
   Index the context piece would begin at, after the family and any version.
   */
  const contextStart = hasVersion ? versionEnd : familyEnd;
  /**
   Whether a separator and an opening paren sit where the context piece needs them.
   */
  const hasContextOpening = (raw.charAt(contextStart,) === PIECE_SEPARATOR)
    && (raw.charAt(contextStart + 1,) === CONTEXT_OPENING);

  if (!hasContextOpening) {
    return (contextStart === raw.length)
      ? {
        family,
        version,
        context: '',
      }
      : UNPARSED_DISPLAY_NAME;
  }
  /**
   Index the context content begins at, just past the separator and opening paren.
   */
  const contentStart = contextStart + 2;
  /**
   Index the context content ends at, being the first whitespace after it.
   */
  const contentEnd = nonWhitespaceRunEnd({
    text: raw,
    start: contentStart,
  },);
  /**
   Whether at least one character of content is followed by the exact closing text at end of name.
   */
  const closesGrammar = (contentEnd > contentStart)
    && (raw.slice(contentEnd,) === CONTEXT_CLOSING);
  if (!closesGrammar)
    return UNPARSED_DISPLAY_NAME;

  return {
    family,
    version,
    context: raw.slice(
      contentStart,
      contentEnd,
    ),
  };
}

//endregion Display name grammar

//region Model display

/**
 Parses a Claude model display name and strips values matching defaults.
 
 @param raw - model display name from Claude Code
 
 @returns concise model display name
 
 @example
 ```ts
 formatModelDisplay('Sonnet 4.6 (1M context)');
 ```
 */
export function formatModelDisplay(raw: string,): string {
  /**
   Parsed display-name components.
   */
  const {
    family,
    version,
    context,
  } = parseDisplayName(raw,);
  if (family === '')
    return raw;
  /**
   Reference values for this family.
   */
  const defaults = MODEL_DEFAULTS[family];
  /**
   Accumulator for the trimmed display name.
   */
  let result = family;

  if ((version !== '') && (version !== defaults?.latestVersion))
    result += ` ${version}`;
  if ((context !== '') && (context !== defaults?.defaultContext))
    result += ` (${context})`;

  return result;
}

//endregion Model display
