/**
 Comment-driven mutation suppression.

 Syntax, mirroring the repo's oxlint disable-comment shape:

 ```ts
 // mutation-test-disable-next-line string, boolean -- reason
 // mutation-test-disable-next-line equality:>=, conditional:"false" -- reason
 // mutation-test-disable-file regex -- reason
 ```

 Family lists are optional (bare directives suppress every family).
 A family may carry one exact replacement after a colon, and then the
 directive hides only the mutants that replace the code with exactly that
 text; a bare family name keeps meaning every replacement of that family.
 Repeating a family with several replacements hides each of them, and a
 bare mention of the family wins over its restricted mentions.

 A replacement is written either bare, as a run of characters without
 whitespace, commas, or double quotes, or as a JSON string literal, which can
 hold any text (commas, colons, spaces, ` -- `, newlines, and the empty
 string as `""`). A bare replacement that is empty, or that contains a double
 quote, is rejected with a diagnostic instead of guessed at.

 Reasons after `--` are optional but land in the report so ignored
 mutants stay auditable.

 @example
 ```ts
 const rules = suppressionRules({ comments, table });
 suppressionReason({ rules, line: 4, operator: 'string', replacement: "''" });
 ```
 */

import { positionAt, } from './lines.ts';
import type { OperatorName, } from './types.ts';

/**
 Next-line directive prefix inside a comment's text.
 */
const NEXT_LINE_DIRECTIVE = 'mutation-test-disable-next-line';

/**
 File-level directive prefix inside a comment's text.
 */
const FILE_DIRECTIVE = 'mutation-test-disable-file';

/**
 Separator between the family list and the free-text reason.
 */
const REASON_SEPARATOR = '--';

/**
 Every valid operator family name, for directive validation.
 */
const FAMILY_NAMES: ReadonlySet<string> = new Set([
  'arithmetic',
  'equality',
  'logical',
  'conditional',
  'boolean',
  'string',
  'unary',
  'update',
  'array',
  'object',
  'optional-chaining',
  'block',
  'method',
  'arrow',
  'regex',
],);

/**
 Returns whether a directive token names a known operator family.

 @param name - Candidate family name from a directive.

 @returns Whether name is an operator family.

 @example
 ```ts
 isFamilyName('string');
 // true
 ```
 */
function isFamilyName(name: string,): name is OperatorName {
  return FAMILY_NAMES.has(name,);
}

/**
 One parsed suppression rule.

 `line` is the source line the rule suppresses (absent for file-wide
 rules); an empty `families` set means every family. `replacements` holds,
 for each family the directive restricted to exact replacement texts, the
 texts it hides; a family absent from that map is hidden for every
 replacement.
 */
export type SuppressionRule = {
  readonly line?: number;
  readonly families: ReadonlySet<OperatorName>;
  readonly replacements: ReadonlyMap<OperatorName, ReadonlySet<string>>;
  readonly reason: string;
};

/**
 Structural comment shape consumed from yuku-parser output.
 */
export type ParsedComment = {
  readonly value: string;
  readonly start: number;
  readonly end: number;
};

/**
 One family mention of a directive, optionally restricted to one replacement.
 */
type DirectiveEntry = {
  readonly family: OperatorName;
  readonly replacement?: string;
};

/**
 Position inside one directive tail.
 */
type TailPosition = {
  readonly text: string;
  readonly index: number;
};

/**
 One value read from a directive tail and the position after it.
 */
type TailRead = {
  readonly value: string;
  readonly index: number;
};

/**
 Returns whether one character is whitespace; the empty string past the end
 of the text is not.

 @param char - One character or the empty string.

 @returns Whether the character is whitespace.

 @example
 ```ts
 isSpace(' '); // true
 ```
 */
function isSpace(char: string,): boolean {
  return (char !== '') && (char.trim() === '');
}

/**
 Moves past whitespace.

 @param position - Where to start.

 @returns Index of the first character that is not whitespace.

 @example
 ```ts
 skipSpaces({ text: '  a', index: 0, }); // 2
 ```
 */
function skipSpaces(position: TailPosition,): number {
  /**
   Index of the first character that is not whitespace.
   */
  const end = position.text
    .split('',)
    .findIndex(function isContent(
      char,
      index,
    ): boolean {
      return (index >= position.index) && (!isSpace(char,));
    },);

  return end === (-1)
    ? position.text
      .length
    : end;
}

/**
 Returns whether the list part of the tail ended: the end of the text or the
 reason separator.

 @param position - Where to look.

 @returns Whether no further entry follows.

 @example
 ```ts
 atListEnd({ text: '-- why', index: 0, }); // true
 ```
 */
function atListEnd(position: TailPosition,): boolean {
  return (position.index
    >= position.text
      .length)
    || position.text
      .startsWith(
      REASON_SEPARATOR,
      position.index,
      );
}

/**
 Reads a family name, up to a colon, comma, whitespace, or the reason
 separator.

 @param position - Where the name starts.

 @returns The name, possibly empty, and the index after it.

 @example
 ```ts
 readFamilyName({ text: 'equality:>=', index: 0, }); // { value: 'equality', index: 8 }
 ```
 */
function readFamilyName(position: TailPosition,): TailRead {
  /**
   Index of the first character that ends the name.
   */
  const end = position.text
    .split('',)
    .findIndex(function endsName(
      char,
      index,
    ): boolean {
      return (index >= position.index)
        && (
        isSpace(char,)
          || (char === ',')
          || (char === ':')
          || position.text
            .startsWith(
            REASON_SEPARATOR,
            index,
            )
        );
    },);
  /**
   Index after the name, the end of the text when nothing ends it.
   */
  const next = end === (-1)
    ? position.text
      .length
    : end;

  return {
    value: position.text
      .slice(
      position.index,
      next,
      ),
    index: next,
  };
}

/**
 Reads a replacement written as a JSON string literal.

 @param position - Where the opening double quote is.

 @returns The decoded replacement and the index after the closing quote.

 @throws Error when the literal is not closed or is not valid JSON.

 @example
 ```ts
 readQuotedReplacement({ text: '"a, b"', index: 0, }); // { value: 'a, b', index: 6 }
 ```
 */
function readQuotedReplacement(position: TailPosition,): TailRead {
  /**
   Index of the closing quote, past the end when the literal is not closed.
   */
  const closing = findClosingQuote(position,);

  if (closing
    >= position.text
      .length)
    throw new Error(
      `unterminated quoted replacement in suppression comment: ${position.text
        .slice(position.index,)}`,
    );

  /**
   The literal with both quotes.
   */
  const literal = position.text
    .slice(
    position.index,
    closing + 1,
    );
  /**
   Decoded literal, whatever JSON made of it.
   */
  const decoded: unknown = decodeJson(literal,);

  if ((typeof decoded) !== 'string')
    throw new Error(
      `quoted replacement ${literal} in suppression comment is not a JSON string literal`,
    );

  return {
    value: decoded,
    index: closing + 1,
  };
}

/**
 Finds the double quote that closes a literal, skipping escaped characters.

 @param position - Where the opening double quote is.

 @returns Index of the closing quote, or the text length when none closes it.

 @example
 ```ts
 findClosingQuote({ text: '"a\\"b"', index: 0, }); // 5
 ```
 */
function findClosingQuote(position: TailPosition,): number {
  /**
   Index of the next character to inspect.
   */
  const scan = { index: position.index + 1, };

  while ((scan.index
    < position.text
      .length) && (position.text
        .charAt(scan.index,)
        !== '"'))
    // A backslash keeps the next character, a quote included, inside the literal.
    scan.index += position.text
      .charAt(scan.index,)
      === '\\'
      ? 2
      : 1;

  return scan.index;
}

/**
 Decodes a JSON literal, naming the literal when it is not valid.

 @param literal - JSON text.

 @returns The decoded value.

 @throws Error when the text is not valid JSON.

 @example
 ```ts
 decodeJson('"a"'); // 'a'
 ```
 */
function decodeJson(literal: string,): unknown {
  try {
    return JSON.parse(literal,) as unknown;
  }
  catch (error) {
    throw new Error(
      `quoted replacement ${literal} in suppression comment is not a JSON string literal`,
      { cause: error, },
    );
  }
}

/**
 Reads a replacement written bare, up to whitespace or a comma.

 @param position - Where the replacement starts.

 @returns The replacement and the index after it.

 @throws Error when the replacement is empty or contains a double quote.

 @example
 ```ts
 readBareReplacement({ text: '>= -- why', index: 0, }); // { value: '>=', index: 2 }
 ```
 */
function readBareReplacement(position: TailPosition,): TailRead {
  /**
   Index of the first character that ends the replacement.
   */
  const end = position.text
    .split('',)
    .findIndex(function endsReplacement(
      char,
      index,
    ): boolean {
      return (index >= position.index) && (isSpace(char,) || (char === ','));
    },);
  /**
   Index after the replacement, the end of the text when nothing ends it.
   */
  const next = end === (-1)
    ? position.text
      .length
    : end;
  /**
   The replacement as written.
   */
  const written = position.text
    .slice(
    position.index,
    next,
    );

  if (written === '')
    throw new Error(
      'empty replacement in suppression comment; write the empty string as "" and a replacement that has whitespace, commas, or a double quote as a JSON string literal',
    );

  if (written.includes('"',))
    throw new Error(
      `bare replacement ${written} in suppression comment contains a double quote; write it as a JSON string literal`,
    );

  return {
    value: written,
    index: next,
  };
}

/**
 Reads the replacement after a family's colon.

 @param position - Where the replacement starts.

 @returns The replacement and the index after it.

 @throws Error when the replacement cannot be read.

 @example
 ```ts
 readReplacement({ text: '"a, b"', index: 0, }); // { value: 'a, b', index: 6 }
 ```
 */
function readReplacement(position: TailPosition,): TailRead {
  return position.text
    .charAt(position.index,)
    === '"'
    ? readQuotedReplacement(position,)
    : readBareReplacement(position,);
}

/**
 Reads one family entry, with its replacement when a colon follows the name.

 @param position - Where the entry starts.

 @returns The entry and the index after it.

 @throws Error when the family is unknown or the replacement cannot be read.

 @example
 ```ts
 readEntry({ text: 'equality:>=', index: 0, }); // { entry: { family: 'equality', replacement: '>=' }, index: 11 }
 ```
 */
function readEntry(position: TailPosition,): {
  readonly entry: DirectiveEntry;
  readonly index: number;
} {
  /**
   Family name of the entry.
   */
  const name = readFamilyName(position,);

  if (!isFamilyName(name.value,))
    throw new Error(
      `unknown mutation family ${name.value} in suppression comment; known: ${[...FAMILY_NAMES,].join(', ',)}`,
    );

  if (position.text
    .charAt(name.index,)
    !== ':')
    return {
      entry: { family: name.value, },
      index: name.index,
    };

  /**
   Replacement after the colon.
   */
  const replacement = readReplacement({
    text: position.text,
    index: name.index + 1,
  },);

  return {
    entry: {
      family: name.value,
      replacement: replacement.value,
    },
    index: replacement.index,
  };
}

/**
 Parses one directive tail into family entries and reason.

 @param tail - Directive text after the directive keyword.

 @returns Entries (empty for every family) and reason.

 @throws Error when a family name is not a known operator family, a
 replacement cannot be read, or text other than a comma or the reason
 separator follows an entry.

 @example
 ```ts
 parseDirectiveTail('string, equality:>= -- flaky filler');
 ```
 */
function parseDirectiveTail(tail: string,): {
  readonly entries: readonly DirectiveEntry[];
  readonly reason: string;
} {
  /**
   Entries read so far.
   */
  const entries: DirectiveEntry[] = [];
  /**
   Where the next read starts.
   */
  const cursor = {
    index: skipSpaces({
      text: tail,
      index: 0,
    },),
  };

  while (!atListEnd({
    text: tail,
    index: cursor.index,
  },)) {
    /**
     The entry at the cursor.
     */
    const read = readEntry({
      text: tail,
      index: cursor.index,
    },);
    entries.push(read.entry,);
    cursor.index = skipSpaces({
      text: tail,
      index: read.index,
    },);

    if (tail.charAt(cursor.index,) === ',') {
      cursor.index = skipSpaces({
        text: tail,
        index: cursor.index + 1,
      },);

      if (atListEnd({
        text: tail,
        index: cursor.index,
      },))
        throw new Error('trailing comma without a family in suppression comment',);
    }
    else if (!atListEnd({
      text: tail,
      index: cursor.index,
    },))
      throw new Error(
        `unexpected text ${tail.slice(cursor.index,)} after a family in suppression comment; separate entries with commas and start the reason with ${REASON_SEPARATOR}`,
      );
  }

  return {
    entries,
    reason: tail.slice(cursor.index + REASON_SEPARATOR.length,)
      .trim(),
  };
}

/**
 Folds directive entries into the families a rule covers and the replacement
 restrictions of the families that have any.

 A family mentioned without a replacement anywhere in the directive is hidden
 for every replacement, whatever else the directive says about it.

 @param entries - Entries read from one directive.

 @returns Families and per-family replacement sets.

 @example
 ```ts
 foldEntries([{ family: 'equality', replacement: '>=' }]);
 ```
 */
function foldEntries(entries: readonly DirectiveEntry[],): Pick<SuppressionRule, 'families' | 'replacements'> {
  /**
   Families mentioned without a replacement.
   */
  const unrestricted = new Set<OperatorName>(
    entries.filter(function isBare(entry,): boolean {
      return entry.replacement === undefined;
    },)
      .map(function toFamily(entry,): OperatorName {
        return entry.family;
      },),
  );
  /**
   Replacement texts of the families that are only mentioned with one.
   */
  const replacements = new Map<OperatorName, Set<string>>();

  for (const entry of entries) {
    if ((entry.replacement === undefined) || unrestricted.has(entry.family,))
      continue;

    replacements.set(
      entry.family,
      new Set([
        ...(replacements.get(entry.family,) ?? []),
        entry.replacement,
      ],),
    );
  }

  return {
    families: new Set(entries.map(function toFamily(entry,): OperatorName {
      return entry.family;
    },),),
    replacements,
  };
}

/**
 Parses suppression rules out of a file's comments.

 @param options - Parsed comments and the file's line-start table.

 @returns Parsed rules, possibly empty.

 @throws Error when a directive names an unknown family or holds a
 replacement that cannot be read.

 @example
 ```ts
 suppressionRules({ comments: result.comments, table: lineStarts(source) });
 ```
 */
export function suppressionRules(options: {
  readonly comments: readonly ParsedComment[];
  readonly table: readonly number[];
},): readonly SuppressionRule[] {
  return options.comments
    .flatMap(function toRules(comment,): readonly SuppressionRule[] {
    /**
     Comment text without leading whitespace.
     */
      const text = comment.value
        .trim();

      if (text.startsWith(NEXT_LINE_DIRECTIVE,)) {
      /**
       Families and reason after the next-line keyword.
       */
        const tail = parseDirectiveTail(text.slice(NEXT_LINE_DIRECTIVE.length,),);
        return [{
          line: positionAt({
            table: options.table,
            offset: comment.start,
          },)
            .line
            + 1,
          ...foldEntries(tail.entries,),
          reason: tail.reason,
        },];
      }

      if (text.startsWith(FILE_DIRECTIVE,)) {
      /**
       Families and reason after the file-level keyword.
       */
        const tail = parseDirectiveTail(text.slice(FILE_DIRECTIVE.length,),);
        return [{
          ...foldEntries(tail.entries,),
          reason: tail.reason,
        },];
      }

      return [];
    },);
}

/**
 Finds every suppression rule covering one mutant.

 Returns matches instead of a nullable reason so absence stays a plain
 empty array at call sites.

 @param options - Parsed rules plus the mutant's line, family, and exact
 replacement text.

 @returns Matching rules, empty when the mutant is not suppressed.

 @example
 ```ts
 matchingSuppressions({ rules, line: 4, operator: 'string', replacement: "''" });
 ```
 */
export function matchingSuppressions(options: {
  readonly rules: readonly SuppressionRule[];
  readonly line: number;
  readonly operator: OperatorName;
  readonly replacement: string;
},): readonly SuppressionRule[] {
  return options.rules
    .filter(function matches(rule,): boolean {
      if ((rule.line !== undefined) && (rule.line !== options.line))
        return false;

      if (rule.families
        .size
        === 0)
        return true;

      if (!rule.families
        .has(options.operator,))
        return false;

      /**
       Replacements the rule is restricted to for this family, absent when
       it hides every replacement.
       */
      const restricted = rule.replacements
        .get(options.operator,);

      return (restricted === undefined)
        || restricted.has(options.replacement,);
    },);
}
