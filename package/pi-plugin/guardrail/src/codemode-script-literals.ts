/**
 Reads the string and template literals one JavaScript script spells.
 
 Code mode runs a script a model wrote, and a guard that reads its text needs
 the literals rather than the source: a command the script builds at run time is
 a template literal or a joined string, while a comment or an identifier that
 spells the same words is not.
 
 This reader is a scanner, not a JavaScript parser:
 it knows comments, escapes, quoted literals, and template substitutions,
 and it treats a slash as arithmetic rather than as the start of a regular
 expression, which the guard's README names as a limit.
 
 @module
 */

//region Constants

/**
 Quote characters that open a literal whose text is copied.
 */
const QUOTED_DELIMITERS: ReadonlySet<string> = new Set([
  '"',
  '\'',
]);

/**
 Character that opens a template literal.
 */
const TEMPLATE_DELIMITER = '`';

/**
 Character that makes the next one literal inside a literal.
 */
const BACKSLASH = '\\';

/**
 Character that introduces a template substitution.
 */
const DOLLAR_SIGN = '$';

/**
 Character that opens a template substitution.
 */
const SUBSTITUTION_OPEN = '{';

/**
 Character that closes a template substitution.
 */
const SUBSTITUTION_CLOSE = '}';

/**
 Character that starts a line comment.
 */
const SLASH = '/';

/**
 Character that marks an arithmetic or concatenation operator.
 */
const PLUS = '+';

/**
 Character that starts a block comment.
 */
const ASTERISK = '*';

/**
 Character that ends a line comment.
 */
const LINE_BREAK = '\n';

/**
 Text a template substitution stands in for, which keeps word boundaries.
 */
const SUBSTITUTION_STAND_IN = ' ';

/**
 Sentinel returned when a scan finds no closing delimiter.
 */
const UNTERMINATED = -1;

/**
 Program word, or empty text when no word stands where a command can begin.

 An empty program word cannot be confused with a real one, so this reader needs
 no separate absence value.
 */
const NO_PROGRAM_WORD = '';

//endregion Constants

//region Types

/**
 One literal a script spells.
 */
type ScriptLiteral = {
  /**
   Whether the literal is a template, which may hold substitutions.
   */
  readonly isTemplate: boolean;
  /**
   Text the literal copies, with each substitution replaced by a space.
   */
  readonly text: string;
  /**
   Whether a template holds a substitution, so the script computes part of its text.
   */
  readonly holdsSubstitution: boolean;
  /**
   Whether the literal stands beside a `+`, so the script joins its text to another value.
   */
  readonly joinedWithPlus: boolean;
};

/**
 End of one template literal or substitution.
 */
type ScanEnd = {
  /**
   Text read so far, empty for a substitution scan.
   */
  readonly text: string;
  /**
   Whether the scan met a substitution.
   */
  readonly holdsSubstitution: boolean;
  /**
   Offset just past the closing delimiter, or the source length when none closes.
   */
  readonly end: number;
};

//endregion Types

//region Scanners

/**
 Finds the offset just past a quoted literal.
 
 @param source - script text being read
 
 @param from - offset just after the opening quote
 
 @returns offset just past the closing quote, or {@link UNTERMINATED} when none closes
 
 @example
 ```ts
 quotedLiteralEnd({ source: `'rm -rf'`, from: 1, }); // 8
 ```
 */
function quotedLiteralEnd({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): number {
  for (let offset = from; offset < source.length; offset += 1) {
    /**
     Character at the inspected offset.
     */
    const character = source[offset];
    if (character === BACKSLASH) {
      offset += 1;
      continue;
    }
    if (QUOTED_DELIMITERS.has(character ?? '',))
      return offset + 1;
  }
  return UNTERMINATED;
}

/**
 Finds the offset where a block comment ends.
 
 @param source - script text being read
 
 @param from - offset just after the opening characters
 
 @returns offset just past the closing characters, or the source length when none closes
 
 @example
 ```ts
 blockCommentEnd({ source: '/* text *\/', from: 2, }); // 10
 ```
 */
function blockCommentEnd({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): number {
  /**
   Offset of the characters closing the comment, or -1 when none close it.
   */
  const closing = source.indexOf(
    '*/',
    from,
  );
  return closing === UNTERMINATED
    ? source.length
    : closing + 2;
}

/**
 Finds the offset where a line comment ends.
 
 @param source - script text being read
 
 @param from - offset just after the opening characters
 
 @returns offset of the line break that ends the comment, or the source length when none ends it
 
 @example
 ```ts
 lineCommentEnd({ source: '// text\ntext();', from: 2, }); // 7
 ```
 */
function lineCommentEnd({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): number {
  /**
   Offset of the line break ending the comment, or -1 when the comment is the last line.
   */
  const lineBreak = source.indexOf(
    LINE_BREAK,
    from,
  );
  return lineBreak === UNTERMINATED
    ? source.length
    : lineBreak;
}

/**
 Finds the offset where a quoted literal ends, for callers that only skip it.
 
 @param source - script text being read
 
 @param from - offset just after the opening quote
 
 @returns offset just past the closing quote, or the source length when none closes
 
 @example
 ```ts
 skipQuotedLiteral({ source: `'a' + 'b'`, from: 1, }); // 3
 ```
 */
function skipQuotedLiteral({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): number {
  /**
   Offset just past the closing quote, or -1 when none closes.
   */
  const end = quotedLiteralEnd({
    source,
    from,
  },);
  return end === UNTERMINATED
    ? source.length
    : end;
}

/**
 Finds the offset where a template literal ends, for callers that only skip it.
 
 @param source - script text being read
 
 @param from - offset just after the opening backtick
 
 @returns offset just past the closing backtick, or the source length when none closes
 
 @example
 ```ts
 skipTemplateLiteral({ source: '`a${b}c`', from: 1, }); // 8
 ```
 */
function skipTemplateLiteral({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): number {
  return scanTemplate({
    source,
    from,
  },)
    .end;
}

/**
 Finds the offset just past the substitution that starts at one offset.
 
 @param source - script text being read
 
 @param from - offset just after `${`
 
 @returns offset just past the closing brace, or the source length when none closes
 
 @example
 ```ts
 substitutionEnd({ source: '${a} ', from: 2, }); // 3
 ```
 */
function substitutionEnd({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): number {
  return (function readsSubstitution(): number {
    /**
     Braces opened inside the substitution that are still to close.
     */
    let depth = 1;
    for (let offset = from; offset < source.length; offset += 1) {
      /**
       Character at the inspected offset.
       */
      const character = source[offset];
      if (QUOTED_DELIMITERS.has(character ?? '',)) {
        offset = skipQuotedLiteral({
          source,
          from: offset + 1,
        },) - 1;
        continue;
      }
      if (character === TEMPLATE_DELIMITER) {
        offset = skipTemplateLiteral({
          source,
          from: offset + 1,
        },) - 1;
        continue;
      }
      if ((character === SLASH) && (source[offset + 1] === SLASH)) {
        offset = lineCommentEnd({
          source,
          from: offset + 2,
        },) - 1;
        continue;
      }
      if ((character === SLASH) && (source[offset + 1] === ASTERISK)) {
        offset = blockCommentEnd({
          source,
          from: offset + 2,
        },) - 1;
        continue;
      }
      if (character === SUBSTITUTION_OPEN) {
        depth += 1;
        continue;
      }
      if (character !== SUBSTITUTION_CLOSE)
        continue;
      depth -= 1;
      if (depth === 0)
        return offset + 1;
    }
    return source.length;
  })();
}

/**
 Reads one template literal.
 
 @param source - script text being read
 
 @param from - offset just after the opening backtick
 
 @returns the text it copies, whether it holds a substitution, and where it ends
 
 @example
 ```ts
 scanTemplate({ source: '`rm -rf ${d}`', from: 1, }).text; // 'rm -rf  '
 ```
 */
function scanTemplate({
  source,
  from,
}: {
  readonly source: string;
  readonly from: number;
},): ScanEnd {
  return (function readsTemplate(): ScanEnd {
    /**
     Text the literal copies, with each substitution replaced by a space.
     */
    const text: string[] = [];
    /**
     Whether the template met a substitution, so the script computes part of it.
     */
    let holdsSubstitution = false;
    for (let offset = from; offset < source.length; offset += 1) {
      /**
       Character at the inspected offset.
       */
      const character = source[offset];
      if (character === BACKSLASH) {
        offset += 1;
        continue;
      }
      if (character === TEMPLATE_DELIMITER) {
        return {
          text: text.join('',),
          holdsSubstitution,
          end: offset + 1,
        };
      }
      if ((character === DOLLAR_SIGN) && (source[offset + 1] === SUBSTITUTION_OPEN)) {
        holdsSubstitution = true;
        offset = substitutionEnd({
          source,
          from: offset + 2,
        },) - 1;
        text.push(SUBSTITUTION_STAND_IN,);
        continue;
      }
      text.push(character ?? '',);
    }
    return {
      text: text.join('',),
      holdsSubstitution,
      end: source.length,
    };
  })();
}

//endregion Scanners

//region Literal reads

/**
 Reports whether the character at one offset is part of the source rather than spacing.
 
 @param character - character to inspect
 
 @returns whether the character is neither whitespace nor a line break
 
 @example
 ```ts
 isSignificantCharacter('+'); // true
 isSignificantCharacter(' '); // false
 ```
 */
function isSignificantCharacter(character: string,): boolean {
  return character.trim() !== '';
}

/**
 Reads the operator standing just before or just after a literal.
 
 @param source - script text being read
 
 @param from - offset to start the search at
 
 @param step - direction to search in
 
 @returns the first significant character that way, or undefined when the source ends first
 
 @example
 ```ts
 operatorBeside({ source: `'a' + b`, from: 4, step: 1, }); // '+'
 ```
 */
function operatorBeside({
  source,
  from,
  step,
}: {
  readonly source: string;
  readonly from: number;
  readonly step: number;
},): string {
  return (function readsCharacters(): string {
    for (let offset = from; (offset >= 0) && (offset < source.length); offset += step) {
      /**
       Character at the inspected offset.
       */
      const character = source[offset] ?? '';
      if (isSignificantCharacter(character,))
        return character;
    }
    return '';
  })();
}

/**
 Reads every string and template literal a script spells.
 
 @param source - script text a model wrote
 
 @returns the literals in reading order
 
 @example
 ```ts
 scriptLiterals(`const c = \`rm -rf \${d}\`;`); // [{ isTemplate: true, text: 'rm -rf  ', holdsSubstitution: true, joinedWithPlus: false }]
 ```
 */
function scriptLiterals(source: string,): readonly ScriptLiteral[] {
  /**
   Literals read so far.
   */
  const literals: ScriptLiteral[] = [];
  for (let offset = 0; offset < source.length; offset += 1) {
    /**
     Character at the inspected offset.
     */
    const character = source[offset];
    if ((character === SLASH) && (source[offset + 1] === SLASH)) {
      offset = lineCommentEnd({
        source,
        from: offset + 2,
      },);
      continue;
    }
    if ((character === SLASH) && (source[offset + 1] === ASTERISK)) {
      offset = blockCommentEnd({
        source,
        from: offset + 2,
      },) - 1;
      continue;
    }
    if (QUOTED_DELIMITERS.has(character ?? '',)) {
      /**
       Offset just past the literal's closing quote, or -1 when none closes.
       */
      const end = quotedLiteralEnd({
        source,
        from: offset + 1,
      },);
      /**
       Offset where the literal stops, which a missing quote puts at the source end.
       */
      const literalEnd = end === UNTERMINATED
        ? source.length
        : end;
      literals.push({
        isTemplate: false,
        text: source.slice(
          offset + 1,
          Math.max(
            literalEnd - 1,
            offset + 1,
          ),
        ),
        holdsSubstitution: false,
        joinedWithPlus: standsBesidePlus({
          source,
          start: offset,
          end: literalEnd,
        },),
      },);
      offset = literalEnd - 1;
      continue;
    }
    if (character !== TEMPLATE_DELIMITER)
      continue;
    /**
     The template literal that starts at this backtick.
     */
    const template = scanTemplate({
      source,
      from: offset + 1,
    },);
    literals.push({
      isTemplate: true,
      text: template.text,
      holdsSubstitution: template.holdsSubstitution,
      joinedWithPlus: standsBesidePlus({
        source,
        start: offset,
        end: template.end,
      },),
    },);
    offset = template.end - 1;
  }
  return literals;
}

/**
 Reports whether a literal stands beside a `+` operator.
 
 @param source - script text being read
 
 @param start - offset of the literal's opening delimiter
 
 @param end - offset just past the literal's closing delimiter
 
 @returns whether a `+` sits directly before or after the literal
 
 @example
 ```ts
 standsBesidePlus({ source: `'rm' + d`, start: 0, end: 4, }); // true
 ```
 */
function standsBesidePlus({
  source,
  start,
  end,
}: {
  readonly source: string;
  readonly start: number;
  readonly end: number;
},): boolean {
  return (operatorBeside({
    source,
    from: start - 1,
    step: -1,
  },) === PLUS)
    || (operatorBeside({
      source,
      from: end,
      step: 1,
    },) === PLUS);
}

//endregion Literal reads

export {
  blockCommentEnd,
  isSignificantCharacter,
  lineCommentEnd,
  quotedLiteralEnd,
  scanTemplate,
  scriptLiterals,
  standsBesidePlus,
  substitutionEnd,
};
export type { ScriptLiteral, };
