/**
 Comment-driven mutation suppression.
 
 Syntax, mirroring the repo's oxlint disable-comment shape:
 
 ```ts
 // mutation-test-disable-next-line string, boolean -- reason
 // mutation-test-disable-file regex -- reason
 ```
 
 Family lists are optional (bare directives suppress every family);
 reasons after `--` are optional but land in the report so ignored
 mutants stay auditable.
 
 @example
 ```ts
 const rules = suppressionRules({ comments, table });
 suppressionReason({ rules, line: 4, operator: 'string' });
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
 rules); an empty `families` set means every family.
 */
export type SuppressionRule = {
  readonly line?: number;
  readonly families: ReadonlySet<OperatorName>;
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
 Parses one directive tail into families and reason.
 
 @param tail - Directive text after the directive keyword.
 
 @returns Families (empty for all) and reason.
 
 @throws Error when a family name is not a known operator family.
 
 @example
 ```ts
 parseDirectiveTail('string, boolean -- flaky filler');
 ```
 */
function parseDirectiveTail(tail: string,): {
  readonly families: ReadonlySet<OperatorName>;
  readonly reason: string;
} {
  /**
   Reason separator position in the directive tail.
   */
  const separatorAt = tail.indexOf('--',);
  /**
   Family list text before the reason separator.
   */
  const familiesText = (separatorAt === (-1) ? tail : tail.slice(
    0,
    separatorAt,
  )).trim();
  /**
   Reason text after the separator.
   */
  const reason = separatorAt === (-1) ? '' : tail.slice(separatorAt + 2,)
    .trim();
  /**
   Declared family names, empty when the directive suppresses all.
   */
  const families = familiesText === ''
    ? []
    : familiesText.split(',',)
      .map(function trimName(name,): string {
        return name.trim();
      },);

  /**
   Family names validated against the known operator families.
   */
  const validated = families.map(function toFamily(family,): OperatorName {
    if (!isFamilyName(family,))
      throw new Error(
        `unknown mutation family ${family} in suppression comment; known: ${[...FAMILY_NAMES,].join(', ',)}`,
      );

    return family;
  },);

  return {
    families: new Set(validated,),
    reason,
  };
}

/**
 Parses suppression rules out of a file's comments.
 
 @param options - Parsed comments and the file's line-start table.
 
 @returns Parsed rules, possibly empty.
 
 @throws Error when a directive names an unknown family.
 
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
          families: tail.families,
          reason: tail.reason,
        },];
      }

      if (text.startsWith(FILE_DIRECTIVE,)) {
      /**
       Families and reason after the file-level keyword.
       */
        const tail = parseDirectiveTail(text.slice(FILE_DIRECTIVE.length,),);
        return [{
          families: tail.families,
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
 
 @param options - Parsed rules plus the mutant's line and family.
 
 @returns Matching rules, empty when the mutant is not suppressed.
 
 @example
 ```ts
 matchingSuppressions({ rules, line: 4, operator: 'string' });
 ```
 */
export function matchingSuppressions(options: {
  readonly rules: readonly SuppressionRule[];
  readonly line: number;
  readonly operator: OperatorName;
},): readonly SuppressionRule[] {
  return options.rules
    .filter(function matches(rule,): boolean {
      if ((rule.line !== undefined) && (rule.line !== options.line))
        return false;

      return (rule.families
        .size
        === 0)
        || rule.families
          .has(options.operator,);
    },);
}
