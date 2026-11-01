/**
 Export-surface text classification for `prefer-describe-function-ref-name`.
 
 Single linear passes replace four per-lookup compiled `RegExp` patterns
 (`export function <name>`,
 `export class <name>`,
 `export const <name>`,
 and `export { <name> } from '...'`).
 The inspected name is compared as a literal identifier token and never
 reaches a pattern compiler, so identifier characters that carry regex
 meaning (`$`) classify like any other name instead of acting as anchors.
 Identifier-token edges count every non-ASCII non-whitespace character as
 part of the surrounding token, which the ASCII-only `\b` boundary misread
 in both directions: a name ending beside a non-ASCII letter never matched,
 and a name followed by `$` matched inside a longer identifier.
 The re-export scan resumes past each closing brace it has ruled out, so
 many unterminated `export {` openers cost O(n) time and O(1) stack where
 the pattern form rescanned the brace-free remainder per opener and was
 quadratic.
 
 @module
 */

//region Keywords and character classes

/**
 Literal token every scanned export shape starts with.
 */
const EXPORT_KEYWORD = 'export';

/**
 Optional prefix the function-declaration head accepts before the
 `function` keyword.
 */
const ASYNC_KEYWORD = 'async';

/**
 Keyword introducing every callable declaration shape; also the discriminant
 the declaration scan branches on.
 */
const FUNCTION_KEYWORD = 'function';

/**
 Keyword introducing the re-export source clause after the closing brace.
 */
const FROM_KEYWORD = 'from';

/**
 Characters treated as whitespace, exactly the set the replaced patterns'
 `\s` matched: the ASCII control spaces, space, no-break space, Ogham space
 mark, the U+2000 to U+200A typographic spaces, line and paragraph
 separators, narrow no-break space, medium mathematical space, ideographic
 space, and the byte-order mark.
 */
const WHITESPACE_CHARACTERS = [
  '\t\n\v\f\r ',
  '\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A',
  '\u2028\u2029\u202F\u205F\u3000\uFEFF',
].join('',);

/**
 Checks whether one character is whitespace, so the scanner reads whitespace
 runs exactly as the replaced patterns' `\s` did.
 
 The empty string reports false, which keeps the past-the-end index a walk
 reaches from counting as whitespace and running off the text.
 
 @param character - one character, or the empty string where an index falls
 outside the text
 
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
 Checks whether one character continues a JavaScript identifier, so a name
 occurrence only counts when it is a whole token.
 
 The set is ASCII letters, ASCII digits, `_`, `$`, plus every non-ASCII
 non-whitespace character as an over-approximation of Unicode ID_Continue:
 whitespace can never sit inside an identifier, so the `\s` set is exempted
 exactly, while any other non-ASCII neighbour keeps the token open. That is
 the conservative reading for a heuristic classifier because identifiers in
 real sources are nearly always ASCII and a non-ASCII punctuation neighbour
 only downgrades the classification to the caller's name-shape fallback.
 The empty string reports false, so both text edges close a token, the
 behaviour the scans rely on.
 
 @param character - one character, or the empty string where an index falls
 outside the text
 
 @returns whether the character can sit inside an identifier beside a name
 
 @example
 ```ts
 isIdentifierPartCharacter('$'); // true
 isIdentifierPartCharacter('\u00a0'); // false; no-break space is whitespace
 ```
 */
function isIdentifierPartCharacter(character: string,): boolean {
  return (character.length === 1)
    && (((character >= 'a') && (character <= 'z'))
      || ((character >= 'A') && (character <= 'Z'))
      || ((character >= '0') && (character <= '9'))
      || (character === '_')
      || (character === '$')
      || ((character > '\u007F') && (!isWhitespaceCharacter(character,))));
}

/**
 Reports whether a binding name has the all-caps snake-case shape the rule
 reads as a constant import, replacing the anchored `^[A-Z][A-Z0-9_]*$`
 pattern with one linear walk over characters.
 
 @param name - binding name to classify; empty names are not constant-shaped
 
 @returns whether the first character is `A`-`Z` and every character is
 `A`-`Z`, `0`-`9`, or `_`
 
 @example
 ```ts
 isAllCapsSnakeName('FEED_PAGE_SIZE'); // true
 isAllCapsSnakeName('FeedPageSize'); // false
 ```
 */
export function isAllCapsSnakeName(name: string,): boolean {
  /**
   Whether every walked character satisfied the all-caps snake-case grammar;
   starts false for the empty name, which the pattern's leading `[A-Z]` rejected.
   */
  let conforming = name !== '';
  for (let index = 0; (index < name.length) && conforming; index += 1) {
    /**
     Character at the walk position.
     */
    const character = name.charAt(index,);
    /**
     Whether the character is an ASCII uppercase letter, the only shape the first character may take.
     */
    const isUpperLetter = (character >= 'A') && (character <= 'Z');
    conforming = (index === 0)
      ? isUpperLetter
      : (isUpperLetter
        || ((character >= '0') && (character <= '9'))
        || (character === '_'));
  }
  return conforming;
}

//endregion Keywords and character classes

//region Runs and tokens

/**
 Finds where the whitespace run starting at one index ends, reading
 whitespace exactly as the replaced patterns' `\s` did via
 {@link isWhitespaceCharacter}.
 
 @param content - source text under scan
 
 @param start - index the run begins at, which may already sit past the end
 
 @returns index one past the last whitespace character, so `start` itself
 when the run is empty
 
 @example
 ```ts
 whitespaceRunEnd({ content: 'a  b', start: 1 }); // 3
 ```
 */
function whitespaceRunEnd({
  content,
  start,
}: Readonly<{
  content: string;
  start: number;
}>,): number {
  /**
   Index the run walk has reached.
   */
  let cursor = start;
  while (isWhitespaceCharacter(content.charAt(cursor,),))
    cursor += 1;
  return cursor;
}

/**
 Reports whether one index starts the inspected name as a whole identifier
 token: the name's characters sit at the index and an identifier-part
 character does not follow them.
 
 Only the trailing edge is checked because every caller positions the index
 directly after a whitespace run, and whitespace is never identifier-part,
 so the leading edge already closes the token.
 
 @param content - source text under scan
 
 @param name - inspected name, compared as literal characters
 
 @param position - index the name must start at
 
 @returns whether a whole-token name occurrence sits at the position
 
 @example
 ```ts
 hasNameTokenAt({ content: 'foo$bar', name: 'foo', position: 0 }); // false
 ```
 */
function hasNameTokenAt({
  content,
  name,
  position,
}: Readonly<{
  content: string;
  name: string;
  position: number;
}>,): boolean {
  if (!content.startsWith(
    name,
    position,
  ))
    return false;
  /**
   Character directly after the name occurrence, or the empty string past the text end.
   */
  const following = content.charAt(position + name.length,);
  return !isIdentifierPartCharacter(following,);
}

/**
 Reports whether a range of source text holds the inspected name as a whole
 identifier token, walking maximal identifier-part runs so every character
 is visited once.
 
 The characters one outside each range edge must not be identifier-part;
 callers pass brace-delimited segments, where `{` and `}` satisfy that
 precondition. A run longer than the name rejects the occurrence inside it,
 which is the token reading of the boundary the replaced patterns asked
 `\b` for.
 
 @param content - source text under scan
 
 @param name - inspected name, compared as literal characters
 
 @param from - first index of the range, inclusive
 
 @param to - last index of the range, exclusive
 
 @returns whether some maximal identifier-part run inside the range equals
 the name exactly
 
 @example
 ```ts
 hasIdentifierTokenInRange({
   content: '{ x as target }',
   name: 'target',
   from: 1,
   to: 14,
 }); // true
 ```
 */
function hasIdentifierTokenInRange({
  content,
  name,
  from,
  to,
}: Readonly<{
  content: string;
  name: string;
  from: number;
  to: number;
}>,): boolean {
  /**
   Whether some identifier-part run inside the range equals the name exactly.
   */
  let found = false;
  /**
   Index the run walk has reached inside the range.
   */
  let cursor = from;
  while ((!found) && (cursor < to)) {
    if (!isIdentifierPartCharacter(content.charAt(cursor,),)) {
      cursor += 1;
      continue;
    }
    /**
     Index the identifier-part run beginning at the cursor starts.
     */
    const runStart = cursor;
    while ((cursor < to) && isIdentifierPartCharacter(content.charAt(cursor,),))
      cursor += 1;
    found = ((cursor - runStart) === name.length)
      && content.startsWith(
        name,
        runStart,
      );
  }
  return found;
}

//endregion Runs and tokens

//region Declaration scan

/**
 Declaration keyword a line-start `export` scan looks for after the
 `export` token.
 */
type DeclarationKeyword = 'function' | 'class' | 'const';

/**
 Finds where the declared name must start after one line-start `export`
 keyword head, reading the head exactly as the replaced patterns did:
 `export`, a non-empty whitespace run, the keyword-specific middle, and a
 non-empty whitespace run before the name.
 
 For `function` the middle accepts the optional `async` prefix (which must
 be followed by its own non-empty whitespace run) and the optional
 generator star between whitespace runs, where the run directly before the
 name must be non-empty; `function*name` therefore fails exactly as the
 pattern `\s*\*?\s+` failed it. The alternatives are mutually exclusive by
 their first character, so no backtracking path exists that a
 single left-to-right walk would miss.
 
 @param content - source text under scan
 
 @param keyword - declaration keyword the head must carry
 
 @param afterExport - index one past the `export` literal
 
 @returns index the declared name must start at, or `-1` when the head does
 not match at this `export`
 
 @example
 ```ts
 declarationNamePosition({
   content: 'export function* target() {}',
   keyword: 'function',
   afterExport: 6,
 }); // 17
 ```
 */
function declarationNamePosition({
  content,
  keyword,
  afterExport,
}: Readonly<{
  content: string;
  keyword: DeclarationKeyword;
  afterExport: number;
}>,): number {
  /**
   Index one past the whitespace run that separates `export` from the keyword.
   */
  const keywordStart = whitespaceRunEnd({
    content,
    start: afterExport,
  },);
  if (keywordStart === afterExport)
    return -1;
  if (keyword !== FUNCTION_KEYWORD) {
    if (!content.startsWith(
      keyword,
      keywordStart,
    ))
      return -1;
    /**
     Index one past the `class` or `const` literal.
     */
    const afterKeyword = keywordStart + keyword.length;
    /**
     Index one past the whitespace run that separates the keyword from the name.
     */
    const nameStart = whitespaceRunEnd({
      content,
      start: afterKeyword,
    },);
    if (nameStart === afterKeyword)
      return -1;
    return nameStart;
  }
  /**
   Index the `function` literal must start at, past the optional `async` prefix.
   */
  let functionStart = keywordStart;
  if (content.startsWith(
    ASYNC_KEYWORD,
    keywordStart,
  )) {
    /**
     Index one past the `async` literal.
     */
    const afterAsyncLiteral = keywordStart + ASYNC_KEYWORD.length;
    /**
     Index one past the whitespace run separating `async` from `function`.
     */
    const functionCandidate = whitespaceRunEnd({
      content,
      start: afterAsyncLiteral,
    },);
    if (functionCandidate === afterAsyncLiteral)
      return -1;
    functionStart = functionCandidate;
  }
  if (!content.startsWith(
    FUNCTION_KEYWORD,
    functionStart,
  ))
    return -1;
  /**
   Index one past the `function` literal.
   */
  const afterFunctionLiteral = functionStart + FUNCTION_KEYWORD.length;
  /**
   Index one past the whitespace run after the `function` literal.
   */
  const afterFunction = whitespaceRunEnd({
    content,
    start: afterFunctionLiteral,
  },);
  if (content.charAt(afterFunction,) === '*') {
    /**
     Index one past the generator star.
     */
    const afterStar = afterFunction + 1;
    /**
     Index one past the whitespace run after the generator star.
     */
    const nameCandidate = whitespaceRunEnd({
      content,
      start: afterStar,
    },);
    if (nameCandidate === afterStar)
      return -1;
    return nameCandidate;
  }
  if (afterFunction === afterFunctionLiteral)
    return -1;
  return afterFunction;
}

/**
 Reports whether source text contains one line-start declaration of the
 inspected name in the given keyword shape, replacing the
 `(?:^|\n)export\s+...\s+<name>\b` patterns.
 
 Every `export` occurrence is a candidate; the occurrence only qualifies
 at the start of the text or directly after a line feed, exactly the
 positions `(?:^|\n)` allowed. Each candidate costs one bounded head walk,
 so the scan is one linear pass over the text.
 
 @param content - source text under scan
 
 @param keyword - declaration keyword the export head must carry
 
 @param name - inspected name, compared as a literal identifier token
 
 @returns whether some line-start `export` declares the name in this shape
 
 @example
 ```ts
 containsDeclarationExport({
   content: 'export async function target() {}',
   keyword: 'function',
   name: 'target',
 }); // true
 ```
 */
function containsDeclarationExport({
  content,
  keyword,
  name,
}: Readonly<{
  content: string;
  keyword: DeclarationKeyword;
  name: string;
}>,): boolean {
  /**
   Whether some line-start `export` declares the name in this shape.
   */
  let found = false;
  /**
   Index the search for the next `export` occurrence resumes at.
   */
  let cursor = 0;
  while ((!found) && (cursor < content.length)) {
    /**
     Index of the next `export` occurrence, or `-1` when the text holds none.
     */
    const exportIndex = content.indexOf(
      EXPORT_KEYWORD,
      cursor,
    );
    if (exportIndex === (-1))
      break;
    cursor = exportIndex + 1;
    /**
     Character before the occurrence; the line-feed side of the replaced `(?:^|\n)` anchor.
     */
    const preceding = content.charAt(exportIndex - 1,);
    if ((exportIndex !== 0) && (preceding !== '\n'))
      continue;
    /**
     Index the declared name must start at, or `-1` when this `export` carries another head.
     */
    const namePosition = declarationNamePosition({
      content,
      keyword,
      afterExport: exportIndex + EXPORT_KEYWORD.length,
    },);
    if (namePosition === (-1))
      continue;
    found = hasNameTokenAt({
      content,
      name,
      position: namePosition,
    },);
  }
  return found;
}

//endregion Declaration scan

//region Re-export scan

/**
 Reports whether one index starts the `from` clause a re-export needs after
 its closing brace: a whitespace run, the `from` literal, a whitespace run,
 and a single or double quote.
 
 @param content - source text under scan
 
 @param start - index one past the closing brace
 
 @returns whether the from-clause and quote follow at the index
 
 @example
 ```ts
 matchesFromQuote({ content: "} from './m'", start: 1 }); // true
 ```
 */
function matchesFromQuote({
  content,
  start,
}: Readonly<{
  content: string;
  start: number;
}>,): boolean {
  /**
   Index the `from` literal must start at.
   */
  const fromStart = whitespaceRunEnd({
    content,
    start,
  },);
  if (!content.startsWith(
    FROM_KEYWORD,
    fromStart,
  ))
    return false;
  /**
   Index the opening quote must sit at.
   */
  const quotePosition = whitespaceRunEnd({
    content,
    start: fromStart + FROM_KEYWORD.length,
  },);
  /**
   Character at the quote position, or the empty string past the text end.
   */
  const quote = content.charAt(quotePosition,);
  return (quote === "'") || (quote === '"');
}

/**
 Reports whether source text re-exports the inspected name through a
 brace list with a `from` clause, replacing the
 `export\s*\{[^}]*\b<name>\b[^}]*\}\s*from\s*['"]` pattern with a scan
 that stays linear on adversarial input.
 
 Unlike the declaration shapes, the replaced pattern carried no line-start
 anchor, so every `export` occurrence is a candidate wherever it sits.
 Linearity comes from two resume rules the pattern engine could not apply:
 when no closing brace exists anywhere ahead, no later opener can match
 either and the scan stops; when a closing brace was found, every opener
 before it shares that same first closing brace, the same from-clause
 verdict, and a segment containing the current one, so a failed attempt
 resumes past the brace instead of rescanning the remainder per opener.
 
 @param content - source text under scan
 
 @param name - inspected name, compared as a literal identifier token
 
 @returns whether some `export { ... } from '<quote>'` list holds the name
 as a whole token
 
 @example
 ```ts
 containsReexportOfName({
   content: "export { helper as target } from './m'",
   name: 'target',
 }); // true
 ```
 */
function containsReexportOfName({
  content,
  name,
}: Readonly<{
  content: string;
  name: string;
}>,): boolean {
  /**
   Whether some `export { ... } from '<quote>'` list holds the name as a whole token.
   */
  let found = false;
  /**
   Index the search for the next `export` occurrence resumes at.
   */
  let cursor = 0;
  while ((!found) && (cursor < content.length)) {
    /**
     Index of the next `export` occurrence, or `-1` when the text holds none.
     */
    const exportIndex = content.indexOf(
      EXPORT_KEYWORD,
      cursor,
    );
    if (exportIndex === (-1))
      break;
    /**
     Index the opening brace must sit at, past the whitespace run after `export`.
     */
    const braceIndex = whitespaceRunEnd({
      content,
      start: exportIndex + EXPORT_KEYWORD.length,
    },);
    if (content.charAt(braceIndex,) !== '{') {
      cursor = exportIndex + 1;
      continue;
    }
    /**
     Index of the first closing brace after the opener, or `-1` when none exists ahead.
     */
    const closeIndex = content.indexOf(
      '}',
      braceIndex + 1,
    );
    if (closeIndex === (-1))
      break;
    found = hasIdentifierTokenInRange({
      content,
      name,
      from: braceIndex + 1,
      to: closeIndex,
    },)
      && matchesFromQuote({
        content,
        start: closeIndex + 1,
      },);
    cursor = closeIndex + 1;
  }
  return found;
}

//endregion Re-export scan

//region Classification entry

/**
 Classification tag telling the caller how the inspected name is exported:
 a callable declaration, a plain const, a re-export from another file, or
 nothing the scan recognised.
 */
export type ExportedNameKind = 'callable' | 'const' | 'reexport' | 'unknown';

/**
 Classifies how source text exports one name, replacing four per-lookup
 `RegExp` constructions with literal-token scans in the same precedence:
 function declarations first, then class declarations, then const
 declarations, then brace-list re-exports, and `'unknown'` when no shape
 matched.
 
 The precedence mirrors the order the replaced patterns were tested in, so
 a text holding several shapes classifies exactly as before: the first
 shape with any match anywhere in the text wins, not the first match by
 position.
 
 @param content - source text of the imported file; never empty, since
 callers short-circuit unreadable files before classifying
 
 @param name - inspected name, compared as literal characters so no
 identifier ever reaches a pattern compiler
 
 @returns tag describing what kind of binding the source exposes
 
 @example
 ```ts
 classifyNameInSourceText({
   content: 'export function* target() {}',
   name: 'target',
 }); // 'callable'
 ```
 */
export function classifyNameInSourceText({
  content,
  name,
}: Readonly<{
  content: string;
  name: string;
}>,): ExportedNameKind {
  if (containsDeclarationExport({
    content,
    keyword: FUNCTION_KEYWORD,
    name,
  },))
    return 'callable';
  if (containsDeclarationExport({
    content,
    keyword: 'class',
    name,
  },))
    return 'callable';
  if (containsDeclarationExport({
    content,
    keyword: 'const',
    name,
  },))
    return 'const';
  if (containsReexportOfName({
    content,
    name,
  },))
    return 'reexport';
  return 'unknown';
}

//endregion Classification entry
