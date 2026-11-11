/**
 Transform hook implementation that rewrites import attributes
 into query-parameter-tagged specifiers using AST parsing.
 
 Uses rolldown's built-in `parseSync` (backed by oxc) to parse source code
 and walk the AST, replacing fragile regex with proper syntax analysis.
 
 @module
 */

import {
  type ESTree,
  parseSync,
  Visitor,
} from 'rolldown/utils';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  extractTypeFromOptions,
  getStringLiteralValue,
  NO_ATTR_TYPE,
  NON_STRING_NODE,
  staticDeclAttrType,
} from './ast-extract.ts';
import { ATTR_QUERY_KEY, } from './patterns.ts';
import {
  collectStaticReplacements,
  findCommaOutsideComments,
  type Replacement,
} from './transform-helpers.ts';

//region Sentinel and fast-path gate

/**
 Sentinel returned by {@link transformImportAttributes} when the source has
 no import-attribute clause to rewrite. A `unique symbol`; the plugin's
 `transform` hook converts it to `null` (Rolldown's "no change" signal) at
 the boundary.
 */
export const NO_TRANSFORM: unique symbol = Symbol('import-attributes/source-has-no-attribute-clause-to-rewrite',);

/**
 Keywords that introduce an attribute clause: `with`, and the deprecated
 `assert` spelling rolldown and the elision helper both still accept.
 */
const ATTRIBUTE_CLAUSE_KEYWORDS: readonly string[] = [
  'with',
  'assert',
];

/**
 Reports whether `character` can continue a JavaScript identifier, which
 decides whether a keyword occurrence is a word of its own.
 
 ASCII letters, digits, `_` and `$` are the cases that matter here: every real
 clause has whitespace, a brace, a comment or a quote next to its keyword, and
 none of those continues an identifier. A non-ASCII identifier character beside
 a keyword only costs one parse the visitor answers with no rewrite.
 
 @param character - one character, or the empty string past either end
 
 @returns whether the character can continue an identifier
 */
function isIdentifierCharacter(character: string,): boolean {
  /**
   Lowercased character, so the letter test covers both cases.
   */
  const lowered = character.toLowerCase();
  return (character === '$') || (character === '_')
    || ((lowered >= 'a') && (lowered <= 'z'))
    || ((lowered >= '0') && (lowered <= '9'));
}

/**
 Reports whether one keyword occurs in `code` as a whole word.
 
 One linear forward scan: `indexOf` jumps between occurrences and each is
 decided by the two characters around it, so no regex backtracking and no
 recursion touches module source of any length.
 
 @param code - module source to search
 
 @param keyword - keyword to look for
 
 @returns whether a whole-word occurrence exists
 */
function mentionsKeyword({
  code,
  keyword,
}: {
  readonly code: string;
  readonly keyword: string;
},): boolean {
  return (function scan(): boolean {
    /**
     Cursor advanced past each rejected occurrence, so the whole pass is linear
     with one stack frame however many occurrences the module holds.
     */
    let from = 0;
    for (;;) {
      /**
       Offset of the next occurrence at or after the cursor; -1 ends the scan.
       */
      const at = code.indexOf(
        keyword,
        from,
      );
      if (at === (-1))
        return false;
      /**
       Character before the occurrence, empty when it starts the source.
       */
      const before = code.charAt(at - 1,);
      /**
       Character after the occurrence, empty when it ends the source.
       */
      const after = code.charAt(
        at + keyword.length,
      );
      if ((!isIdentifierCharacter(before,)) && (!isIdentifierCharacter(after,)))
        return true;
      from = at + 1;
    }
  })();
}

//endregion Sentinel and fast-path gate

//region Declaration replacements

/**
 Records the replacements for one declaration whose source and attribute clause
 the transform rewrites.
 
 @param code - because every replacement text is a slice of the module source
 
 @param source - because the specifier literal is the span that changes
 
 @param attributes - because the attribute clause decides the query value
 
 @param replacements - because the visitor accumulates spans for the reducer
 */
function recordStaticReplacements({
  code,
  source,
  attributes,
  replacements,
}: Readonly<{
  code: string;
  source: ESTree.StringLiteral;
  attributes: readonly ESTree.ImportAttribute[];
  replacements: Replacement[];
}>,): void {
  /**
   Attribute type on this declaration's clause; gates whether to emit a replacement.
   */
  const attrType = staticDeclAttrType(attributes,);
  if (attrType === NO_ATTR_TYPE)
    return;
  replacements.push(...collectStaticReplacements({
    source,
    attributes,
    attrType,
    code,
  },),);
}

//endregion Declaration replacements

//region Transform entry

/**
 Rewrites import/export statements that use `with { type: '...' }` attributes.
 Parses the source with oxc, walks the AST to find import attributes via
 {@link extractTypeFromAttributes}/{@link extractTypeFromOptions}, and
 applies precise span-based replacements collected by
 {@link collectStaticReplacements}.
 
 @param code - source code to transform
 
 @param id - module ID (filename) for the parser
 
 @returns transformed code object, or {@link NO_TRANSFORM} if no attributes were found
 
 @example
 ```ts
 const result = transformImportAttributes(
   "import x from './file.sql' with { type: 'text' }",
   'main.ts',
 );
 // result.code: "import x from './file.sql?attr=text'"
 ```
 */
export function transformImportAttributes({
  code,
  id,
}: {
  readonly code: string;
  readonly id: string;
},): { code: string; } | typeof NO_TRANSFORM {
  if (!ATTRIBUTE_CLAUSE_KEYWORDS.some(function byKeyword(keyword,) {
    return mentionsKeyword({
      code,
      keyword,
    },);
  },))
    return NO_TRANSFORM;

  /**
   Parsed AST root walked to collect attribute replacements.
   */
  const result = parseSync(
    id,
    code,
  );
  /**
   Accumulator of span replacements that the reducer below applies to the source code.
   */
  const replacements: Replacement[] = [];

  //region AST visitor

  /**
   AST visitor that records replacements for each kind of attribute-bearing declaration.
   */
  const visitor = new Visitor({
    ImportDeclaration(node: ForeignBorrowed<ESTree.ImportDeclaration>,): void {
      recordStaticReplacements({
        code,
        source: node.source,
        attributes: node.attributes,
        replacements,
      },);
    },

    ExportNamedDeclaration(node: ForeignBorrowed<ESTree.ExportNamedDeclaration>,): void {
      /**
       Declaration without a source names no module to rewrite.
       */
      if (node.source
        === null)
        return;
      recordStaticReplacements({
        code,
        source: node.source,
        attributes: node.attributes,
        replacements,
      },);
    },

    ExportAllDeclaration(node: ForeignBorrowed<ESTree.ExportAllDeclaration>,): void {
      recordStaticReplacements({
        code,
        source: node.source,
        attributes: node.attributes,
        replacements,
      },);
    },

    ImportExpression(node: ForeignBorrowed<ESTree.ImportExpression>,): void {
      if (node.options
        === null)
        return;
      /**
       Attribute type extracted from the dynamic-import options object.
       */
      const attrType = extractTypeFromOptions(node.options,);
      if (attrType === NO_ATTR_TYPE)
        return;
      /**
       Specifier is skipped when it is not a string literal: a computed source has no bytes to rewrite safely.
       */
      if (getStringLiteralValue(node.source,) === NON_STRING_NODE)
        return;

      /**
       Text of the specifier literal spanning both quotes, escape sequences exactly as the source spelled them.
       */
      const rawLiteral = code.slice(
        node.source
          .start,
        node.source
          .end,
      );
      replacements.push({
        start: node.source
          .start,
        end: node.source
          .end,
        text: `${rawLiteral.slice(
          0,
          -1,
        )}?${ATTR_QUERY_KEY}=${attrType}${rawLiteral.slice(-1,)}`,
      },);

      /**
       Absolute offset of the comma separating the specifier from its options,
       found outside comments; -1 means the options follow without one.
       */
      const commaPos = findCommaOutsideComments({
        code,
        fromPos: node.source
          .end,
        toPos: node.options
          .start,
      },);
      replacements.push({
        start: commaPos
          === (-1)
          ? node.options
            .start
          : commaPos,
        end: node.options
          .end,
        text: '',
      },);
    },
  },);

  //endregion AST visitor

  visitor.visit(result.program,);

  //region Replacement application

  if (replacements.length
    === 0)
    return NO_TRANSFORM;

  // Apply replacements in reverse order to preserve byte offsets
  replacements.sort(function byStartDesc(
    a,
    b,
  ): number {
    return b.start
      - a
        .start;
  },);

  /**
   Final source after every replacement has been applied in descending start order.
   */
  const transformed = replacements.reduce(
    function applyReplacement(
      acc,
      r,
    ): string {
      return acc.slice(
        0,
        r.start,
      )
        + r
          .text
        + acc
          .slice(r.end,);
    },
    code,
  );

  //endregion Replacement application

  return { code: transformed, };
}

//endregion Transform entry
