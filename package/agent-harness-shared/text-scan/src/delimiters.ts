/**
 Regex-free delimiter and line stripping helpers.
 
 @module
 */

//region Delimiter stripping

/**
 Raised when a delimiter literal is empty.
 
 An empty delimiter matches at every cursor position, so a scan between
 delimiters never advances past the opener and would consume memory until the
 process dies instead of returning. Only non-empty literals can bound a span.
 */
class EmptyDelimiterError extends Error {
  /**
   Build the diagnostic naming the parameter the caller left empty.
   
   @param parameter - name of the delimiter parameter carrying the empty literal
   
   @example
   ```ts
   new EmptyDelimiterError('openDelim');
   ```
   */
  constructor(parameter: 'openDelim' | 'closeDelim',) {
    super(`stripBetweenDelims needs a non-empty delimiter, but ${parameter} is empty`,);
    this.name = 'EmptyDelimiterError';
  }
}

/**
 Removes substrings bounded by matching open and close delimiters from `text`.
 
 Iterates left to right. For each open delimiter found, removes through the
 next close delimiter. Unmatched openers are left in place.
 When `disallowedInside` is provided, any disallowed character inside the
 delimiter span cancels stripping for that occurrence.
 
 @param text - input text
 
 @param openDelim - opening delimiter literal, which must not be empty
 
 @param closeDelim - closing delimiter literal, which must not be empty
 
 @param disallowedInside - optional character that aborts stripping when present inside candidate span
 
 @returns text with each matched delimiter span removed
 
 @throws {@link EmptyDelimiterError} when `openDelim` or `closeDelim` is empty,
 because an empty literal matches at the cursor and the scan could never advance
 
 @example
 ```ts
 stripBetweenDelims({ text: 'a `code` b', openDelim: '`', closeDelim: '`' });
 // => 'a  b'
 ```
 */
function stripBetweenDelims(
  {
    text,
    openDelim,
    closeDelim,
    disallowedInside,
  }: {
    readonly text: string;
    readonly openDelim: string;
    readonly closeDelim: string;
    readonly disallowedInside?: string;
  },
): string {
  if (openDelim === '')
    throw new EmptyDelimiterError('openDelim',);
  if (closeDelim === '')
    throw new EmptyDelimiterError('closeDelim',);

  /**
   Output segments, joined once so the accumulator is never recopied per span.
   */
  const parts: string[] = [];
  /**
   Cursor that advances monotonically past each emitted or stripped span.
   */
  let cursorIndex = 0;

  for (
    let openIndex = text.indexOf(
      openDelim,
      0,
    );
    openIndex !== (-1);
    openIndex = text.indexOf(
      openDelim,
      cursorIndex,
    )
  ) {
    /**
     Search start for matching close delimiter after the entire opener.
     */
    const closeSearchStart = openIndex + openDelim.length;
    /**
     Index of closing delimiter, or `-1` when opener is unmatched.
     */
    const closeIndex = text.indexOf(
      closeDelim,
      closeSearchStart,
    );
    if (closeIndex === (-1))
      break;

    /**
     Span between delimiters, exclusive of both delimiters.
     */
    const inside = text.slice(
      closeSearchStart,
      closeIndex,
    );
    if ((disallowedInside !== undefined) && inside.includes(disallowedInside,)) {
      parts.push(text.slice(
        cursorIndex,
        closeSearchStart,
      ),);
      cursorIndex = closeSearchStart;
    }
    else {
      parts.push(text.slice(
        cursorIndex,
        openIndex,
      ),);
      cursorIndex = closeIndex + closeDelim.length;
    }
  }

  parts.push(text.slice(cursorIndex,),);
  /**
   Joined output from retained parts.
   */
  const result = parts.join('',);
  return result;
}

/**
 Removes lines whose first non-whitespace character starts with `prefix`.
 
 Lines that do not match remain unchanged, including their trailing newlines.
 This matches stripping Markdown blockquote lines without regular expressions.
 
 @param text - input text containing one or more newline-separated lines
 
 @param prefix - prefix literal that flags a line for removal
 
 @returns text with matching lines removed
 
 @example
 ```ts
 stripLinesStartingWith({ text: 'a\n> quote\nb', prefix: '>' });
 // => 'a\nb'
 ```
 */
function stripLinesStartingWith(
  {
    text,
    prefix,
  }: {
    readonly text: string;
    readonly prefix: string;
  },
): string {
  return text
    .split('\n',)
    .filter(function keepLine(line,): boolean {
      return !line.trimStart()
        .startsWith(prefix,);
    },)
    .join('\n',);
}

//endregion Delimiter stripping

export {
  EmptyDelimiterError,
  stripBetweenDelims,
  stripLinesStartingWith,
};
