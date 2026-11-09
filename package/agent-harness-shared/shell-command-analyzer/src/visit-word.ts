/**
 Word visitors for `unbash` shell command analysis.
 
 Every word part the parser can produce is answered here.
 A part that carries a `parts` array is descended whatever its type is,
 because that array is the only place the shell grammar puts a nested word;
 the leaf types below are the ones whose grammar has no child list at all.
 Any other part is named as unread instead of returning an empty result.
 
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
  visitResultForUnreadNodeKind,
  type VisitResult,
} from './internal-types.ts';
import { visitExpansion, } from './nested.ts';
import type { ShellCommandContext, } from './types.ts';
import { parameterExpansionResult, } from './work-items.ts';

//region Fixtures

/**
 Word part types whose grammar holds no child list, so no command can nest in them.

 `Literal`, `SingleQuoted`, and `SimpleExpansion` carry only text,
 `AnsiCQuoted` decodes backslash escapes without any expansion,
 and `Glob` appears here for the `star` and `question` kinds,
 whose `parts` field is typed `undefined`;
 a `bracket` glob carries a child list and is therefore never a leaf.
 */
const COMMAND_FREE_WORD_PART_TYPES: ReadonlySet<string> = new Set([
  'Literal',
  'SingleQuoted',
  'SimpleExpansion',
  'AnsiCQuoted',
],);

//endregion Fixtures

/**
 Read the child list a word part carries, when the parser gives it one.

 @param part - word part or quoted child to inspect

 @returns whether the part's grammar holds a child list, narrowing to it

 @example
 ```ts
 if (carriesWordParts(part,)) descend(part.parts);
 ```
 */
function carriesWordParts(
  part: ForeignBorrowed<UnbashWordPart | UnbashDoubleQuotedChild>,
): part is ForeignBorrowed<UnbashWordPart> & {
  readonly parts: readonly ForeignBorrowed<UnbashWordPart | UnbashDoubleQuotedChild>[]
} {
  return ('parts' in part) && Array.isArray(part.parts,);
}

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
  if (part.type === 'ParameterExpansion')
    return parameterExpansionResult({
      part,
      context,
    },);
  if ((part.type === 'CommandExpansion') || (part.type === 'ProcessSubstitution')) {
    return visitExpansion({
      expansion: part,
      context,
    },);
  }
  if (part.type === 'ArithmeticExpansion') {
    if (part.expression === undefined)
      return EMPTY_VISIT_RESULT;
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'arithmetic',
        expression: part.expression,
        context,
      },],
    };
  }
  /**
   A part whose grammar holds a child list covers `DoubleQuoted`, `LocaleString`,
   `BraceExpansion`, `ExtendedGlob`, and a `bracket` glob.
   */
  if (carriesWordParts(part,)) {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'parts',
        parts: part.parts,
        context,
      },],
    };
  }
  if ((part.type === 'Glob') || COMMAND_FREE_WORD_PART_TYPES.has(part.type,))
    return EMPTY_VISIT_RESULT;
  return visitResultForUnreadNodeKind(part.type,);
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
    notUnderstoodNodeKinds: results.flatMap(function resultNotUnderstoodNodeKinds(
      result,
    ): VisitResult['notUnderstoodNodeKinds'] {
      return result.notUnderstoodNodeKinds;
    },),
  };
}

export {
  visitParts,
  visitWord,
};
