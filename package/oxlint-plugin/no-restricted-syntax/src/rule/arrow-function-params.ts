import { isWhitespaceChar, } from '@monochromatic-dev/oxlint-plugin-shared/ts';

/**
 Minimal node shape required by {@link extractParamsText}.
 
 Subset of `ArrowFunctionExpression` covering only the properties
 needed for parameter text extraction.
 */
type ArrowParamsNode = {
  /**
   Whether the function is async.
   */
  readonly async: boolean;
  /**
   Generic type parameter list, if present.
   */
  readonly typeParameters?: unknown;
};

/**
 Thrown when {@link scanParenGroupText} reaches the end of the scanned text
 without closing a parenthesized parameter group.
 
 Reached only when the text scanner and the parser disagree, for example a
 nested template literal inside a parameter default value.
 */
class ArrowParamsUnbalancedError extends Error {}

/**
 Async keyword that may prefix arrow function source text.
 */
const ASYNC_KEYWORD = 'async';

//region Scanning

/**
 Finds the index after a leading async keyword and its following whitespace.
 
 @param fullText - source text of an async arrow function
 
 @returns first index after `async` whitespace, or zero when no prefix exists
 
 @example
 ```ts
 indexAfterAsyncPrefix({ fullText: 'async  (x) => x' }); // 7
 ```
 */
function indexAfterAsyncPrefix({ fullText, }: { readonly fullText: string; },): number {
  if (!fullText.startsWith(ASYNC_KEYWORD,))
    return 0;
  /**
   Cursor after the async keyword, advanced across following whitespace.
   */
  let index = ASYNC_KEYWORD.length;
  if (!isWhitespaceChar(fullText.charAt(index,),))
    return 0;
  while (isWhitespaceChar(fullText.charAt(index,),))
    index += 1;
  return index;
}

/**
 Finds the index at or after `index` that is outside whitespace and comments.
 
 @param text - source text holding the scan
 
 @param index - scan start position inside `text`
 
 @returns index of the first character that is neither whitespace nor comment,
 or `text.length` when only trivia remains
 
 @example
 ```ts
 indexAfterTrivia({ text: ' /* c *\/ (x)', index: 0 }); // 9
 ```
 */
function indexAfterTrivia(
  {
    text,
    index,
  }: {
    readonly text: string;
    readonly index: number;
  },
): number {
  /**
   Cursor advanced across trivia so the caller sees code, not comments.
   */
  let cursor = index;
  while (cursor < text.length) {
    /**
     Character under the cursor.
     */
    const char = text.charAt(cursor,);
    if (isWhitespaceChar(char,)) {
      cursor += 1;
      continue;
    }
    if ((char === '/')
      && (text.charAt(cursor + 1,) === '/')) {
      cursor += 2;
      while ((cursor < text.length)
        && (text.charAt(cursor,) !== '\n')
        && (text.charAt(cursor,) !== '\r'))
        cursor += 1;
      continue;
    }
    if ((char === '/')
      && (text.charAt(cursor + 1,) === '*')) {
      cursor += 2;
      while ((cursor < text.length)
        && !((text.charAt(cursor,) === '*')
          && (text.charAt(cursor + 1,) === '/')))
        cursor += 1;
      cursor += 2;
      continue;
    }
    return cursor;
  }
  return cursor;
}

/**
 Extracts the verbatim text of the first parenthesized group at or after
 `index`, skipping string literals and comments so delimiters inside them do
 not open or close the group.
 
 @param text - source text holding the scan
 
 @param index - scan start position inside `text`
 
 @returns verbatim text from the opening paren of the first group through its
 matching close paren, including trailing commas and comments
 
 @throws ArrowParamsUnbalancedError when no balanced group exists at or after
 `index`
 
 @example
 ```ts
 scanParenGroupText({ text: '(x /* ) *\/)', index: 0 }); // '(x /* ) *\/)'
 ```
 */
function scanParenGroupText(
  {
    text,
    index,
  }: {
    readonly text: string;
    readonly index: number;
  },
): string {
  /**
   Parenthesis nesting counter; the group ends when it returns to zero.
   */
  let depth = 0;
  /**
   Index of the opening paren of the group, or negative before one is seen.
   */
  let groupStart = -1;
  /**
   Active string-literal delimiter, or empty string outside any literal; an
   empty-string sentinel keeps the cursor type free of nullish unions.
   */
  let quote = '';
  /**
   Active comment kind, or empty string outside any comment.
   */
  let commentKind = '';
  /**
   Cursor advanced one lexical unit at a time over `text`.
   */
  let cursor = index;
  while (cursor < text.length) {
    /**
     Character under the cursor.
     */
    const char = text.charAt(cursor,);
    if (commentKind === 'line') {
      if ((char === '\n')
        || (char === '\r'))
        commentKind = '';
      cursor += 1;
      continue;
    }
    if (commentKind === 'block') {
      if ((char === '*')
        && (text.charAt(cursor + 1,) === '/')) {
        commentKind = '';
        cursor += 2;
        continue;
      }
      cursor += 1;
      continue;
    }
    if (quote !== '') {
      if (char === '\\') {
        cursor += 2;
        continue;
      }
      if (char === quote)
        quote = '';
      cursor += 1;
      continue;
    }
    if ((char === '\'')
      || (char === '"')
      || (char === '`')) {
      quote = char;
      cursor += 1;
      continue;
    }
    if ((char === '/')
      && (text.charAt(cursor + 1,) === '/')) {
      commentKind = 'line';
      cursor += 2;
      continue;
    }
    if ((char === '/')
      && (text.charAt(cursor + 1,) === '*')) {
      commentKind = 'block';
      cursor += 2;
      continue;
    }
    if (char === '(') {
      if (depth === 0)
        groupStart = cursor;
      depth += 1;
      cursor += 1;
      continue;
    }
    if (char === ')') {
      depth -= 1;
      if (depth === 0)
        return text.slice(
          groupStart,
          cursor + 1,
        );
      cursor += 1;
      continue;
    }
    cursor += 1;
  }
  throw new ArrowParamsUnbalancedError(
    'arrow parameter text has no balanced parenthesized group',
  );
}

//endregion Scanning

//region Extraction

/**
 Extracts the parameter list text (including parentheses) from an arrow
 function's source text.
 
 The arrow function source may start with `async ` (skipped via
 {@link indexAfterAsyncPrefix}), then optional type parameters `<T>`, then
 the parameter list. A parenthesized parameter list is returned verbatim from
 its opening paren to its matching close paren, preserving trailing commas and
 comments. A bare parameter (a single identifier without parentheses) is
 re-emitted as its identifier text wrapped in parentheses, because a function
 declaration needs a parenthesized list and the body text follows the arrow,
 not the parameter name.
 
 @param fullText - complete source text of the ArrowFunctionExpression node
 
 @param node - arrow function AST node, used for `async` flag and
 `typeParameters` presence
 
 @param paramTexts - source text of each parameter from the AST; rebuilds the
 list when the source uses a bare parameter, where scanning would otherwise
 swallow the `=>` arrow and body
 
 @returns parameter list text including surrounding parentheses
 
 @throws ArrowParamsUnbalancedError when zero parameters and no parenthesized
 group exist at or after the parameter position; unreachable for parsed arrow
 source except scanner disagreements
 
 @example
 ```ts
 const params = extractParamsText({
   fullText: 'async <T>(x: T): T => x',
   node: { async: true, typeParameters: { start: 6, end: 9 } },
   paramTexts: ['x: T'],
 });
 // params === '(x: T)'
 ```
 
 @example
 ```ts
 const params = extractParamsText({
   fullText: 'label => repliesOf(label)',
   node: { async: false },
   paramTexts: ['label'],
 });
 // params === '(label)'
 ```
 */
export function extractParamsText(
  {
    fullText,
    node,
    paramTexts,
  }: {
    readonly fullText: string;
    readonly node: ArrowParamsNode;
    readonly paramTexts: readonly string[];
  },
): string {
  /**
   Skip `async ` prefix if present.
   */
  let start = 0;
  if (node.async)
    start = indexAfterAsyncPrefix({ fullText, },);

  /**
   Skip type parameters `<...>` if present.
   Count angle bracket depth to handle nested generics.
   */
  if ((node.typeParameters
    !== null) && (node.typeParameters
      !== undefined)) {
    /**
     Source slice starting at the first non-`async` character; inspected for a leading `<`.
     */
    const tpText = fullText.slice(start,);
    if (tpText.startsWith('<',)) {
      /**
       Angle-bracket nesting counter so nested generics resolve before the params open.
       */
      let depth = 0;
      for (let loopIndex = 0; loopIndex < tpText
        .length; loopIndex++) {
        if (tpText[loopIndex]
          === '<')
          depth++;
        else if (tpText[loopIndex]
          === '>') {
          depth--;
          if (depth === 0) {
            start += loopIndex + 1;
            break;
          }
        }
      }
    }
  }

  /**
   Source from after the `async` prefix and type parameters, holding the
   parameter list.
   */
  const rest = fullText.slice(start,);
  /**
   Index of the parameter list itself, past whitespace and comments.
   */
  const paramsStart = indexAfterTrivia({
    text: rest,
    index: 0,
  },);
  if (rest.charAt(paramsStart,) === '(')
    return scanParenGroupText({
      text: rest,
      index: paramsStart,
    },);
  if (paramTexts.length > 0)
    return `(${paramTexts.join(', ',)})`;
  /**
   Zero parameters without a paren at the parameter position: `async() => x`
   keeps `async` glued to the paren, so the async skip never applied and the
   group still needs scanning.
   */
  return scanParenGroupText({
    text: rest,
    index: 0,
  },);
}

//endregion Extraction
