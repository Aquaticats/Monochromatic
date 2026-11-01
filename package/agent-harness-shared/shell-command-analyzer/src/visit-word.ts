/**
 Word visitors for `unbash` shell command analysis.
 
 @module
 */

import type {
  DoubleQuotedChild as UnbashDoubleQuotedChild,
  Word as UnbashWord,
  WordPart as UnbashWordPart,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  EMPTY_VISIT_RESULT,
  type VisitResult,
} from './internal-types.ts';
import { visitExpansion, } from './nested.ts';
import type { ShellCommandContext, } from './types.ts';
import { parameterWordItems, } from './work-items.ts';

/**
 Build child work from one word.
 
 @param word - word whose parts should be visited
 
 @param context - execution context inherited by nested expansions
 
 @returns work items for word parts
 
 @example
 ```ts
 visitWord({ word, context });
 ```
 */
function visitWord(
  {
    word,
    context,
  }: {
    readonly word: ForeignBorrowed<UnbashWord>;
    readonly context: ShellCommandContext;
  },
): VisitResult {
  return {
    ...EMPTY_VISIT_RESULT,
    workItems: [{
      kind: 'parts',
      parts: word.parts ?? [],
      context,
    },],
  };
}

/**
 Build child work from one word part.
 
 @param part - word part to inspect
 
 @param context - execution context inherited by nested expansions
 
 @returns child work and nested parse diagnostics
 
 @example
 ```ts
 visitPart({ part, context });
 ```
 */
function visitPart(
  {
    part,
    context,
  }: {
    readonly part: ForeignBorrowed<UnbashWordPart | UnbashDoubleQuotedChild>;
    readonly context: ShellCommandContext;
  },
): VisitResult {
  if ((part.type === 'DoubleQuoted') || (part.type === 'LocaleString')) {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'parts',
        parts: part.parts,
        context,
      },],
    };
  }
  if (part.type === 'ParameterExpansion') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: parameterWordItems({
        part,
        context,
      },),
    };
  }
  if ((part.type === 'CommandExpansion') || (part.type === 'ProcessSubstitution')) {
    return visitExpansion({
      expansion: part,
      context,
    },);
  }
  if ((part.type === 'ArithmeticExpansion') && (part.expression !== undefined)) {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'arithmetic',
        expression: part.expression,
        context,
      },],
    };
  }
  return EMPTY_VISIT_RESULT;
}

/**
 Build child work from word parts.
 
 @param parts - parts to inspect
 
 @param context - execution context inherited by nested expansions
 
 @returns child work and nested parse diagnostics
 
 @example
 ```ts
 visitParts({ parts, context });
 ```
 */
function visitParts(
  {
    parts,
    context,
  }: {
    readonly parts: readonly ForeignBorrowed<
      UnbashWordPart | UnbashDoubleQuotedChild
      >[];
    readonly context: ShellCommandContext;
  },
): VisitResult {
  /**
   Visit result for each word part.
   */
  const results = parts.map(function visitWordPart(
    part: ForeignBorrowed<UnbashWordPart | UnbashDoubleQuotedChild>,
  ): VisitResult {
    return visitPart({
      part,
      context,
    },);
  },);

  return {
    ...EMPTY_VISIT_RESULT,
    workItems: results.flatMap(function resultWorkItems(result,): VisitResult['workItems'] {
      return result.workItems;
    },),
    flags: {
      isPipeline: false,
      hasBackground: false,
      hasCommandSubstitution: results.some(function resultHasCommandSubstitution(result,): boolean {
        return result.flags
          .hasCommandSubstitution;
      },),
      hasProcessSubstitution: results.some(function resultHasProcessSubstitution(result,): boolean {
        return result.flags
          .hasProcessSubstitution;
      },),
    },
    parseErrors: results.flatMap(function resultParseErrors(result,): VisitResult['parseErrors'] {
      return result.parseErrors;
    },),
  };
}

export {
  visitParts,
  visitWord,
};
