/**
 Tests for suppression directives that name an exact replacement per family,
 and for the grammar boundaries of that syntax.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  enumerateMutants,
  lineStarts,
  matchingSuppressions,
  suppressionRules,
} from '../../dist/final/node/index.mjs';

/**
 Source line mutated by the fixtures: one comparison with two equality
 replacements, `<` and `>`.
 */
const COMPARISON = 'export const isAfter = (a: number, b: number) => a >= b;';

/**
 Replacement texts of the equality mutants of one snippet, runnable and
 ignored.

 @param directive - Directive text placed on the line before the snippet.

 @returns Texts of runnable equality mutants and of ignored ones.

 @example
 ```ts
 equalityOutcome('mutation-test-disable-next-line equality:<');
 ```
 */
function equalityOutcome(directive: string,): {
  readonly runnable: readonly string[];
  readonly ignored: readonly string[];
} {
  const result = enumerateMutants({
    file: 'src/snippet.ts',
    source: `// ${directive}\n${COMPARISON}\n`,
  },);
  return {
    runnable: result.mutants
      .filter(function equality(mutant,): boolean {
        return mutant.operator === 'equality';
      },)
      .map(function toText(mutant,): string {
        return mutant.replacement;
      },)
      .toSorted(),
    ignored: result.ignored
      .filter(function equality(mutant,): boolean {
        return mutant.operator === 'equality';
      },)
      .map(function toText(mutant,): string {
        return mutant.replacement;
      },)
      .toSorted(),
  };
}

/**
 Parses one directive comment through the public rule parser.

 @param text - Comment text after the comment opener.

 @returns The parsed rules.

 @throws Error when the directive does not parse.

 @example
 ```ts
 parseDirective(' mutation-test-disable-next-line string:"a, b"');
 ```
 */
function parseDirective(text: string,): ReturnType<typeof suppressionRules> {
  return suppressionRules({
    comments: [
      {
        value: text,
        start: 0,
        end: text.length,
      },
    ],
    table: lineStarts(`//${text}\nx\n`,),
  },);
}

/**
 Whether the first rule of a directive hides one mutant of the second line.

 @param options - Directive text, mutant family and replacement.

 @returns Whether the rule covers the mutant.

 @example
 ```ts
 hides({ text: ' mutation-test-disable-next-line string:"a, b"', operator: 'string', replacement: 'a, b' });
 ```
 */
function hides(options: {
  readonly text: string;
  readonly operator: Parameters<typeof matchingSuppressions>[0]['operator'];
  readonly replacement: string;
}): boolean {
  return matchingSuppressions({
    rules: parseDirective(options.text,),
    line: 2,
    operator: options.operator,
    replacement: options.replacement,
  },).length > 0;
}

await describe({
  name: 'suppression by replacement',
  children: [
    describe({
      name: 'matching',
      children: [
        it({
          name: 'a bare family hides every replacement of the family',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality',);
            expect(outcome.runnable,).toEqual([],);
            expect(outcome.ignored,).toEqual([
              '<',
              '>',
            ],);
          },
        },),

        it({
          name: 'a family with a replacement hides only that replacement',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality:> -- proof',);
            expect(outcome.runnable,).toEqual(['<'],);
            expect(outcome.ignored,).toEqual(['>'],);
          },
        },),

        it({
          name: 'a replacement for a different family hides nothing here',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line conditional:true',);
            expect(outcome.ignored,).toEqual([],);
          },
        },),

        it({
          name: 'a replacement that no mutant has hides nothing',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality:<=',);
            expect(outcome.ignored,).toEqual([],);
          },
        },),

        it({
          name: 'matching is exact, not a prefix or a substring',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality:>=',);
            expect(outcome.ignored,).toEqual([],);
          },
        },),

        it({
          name: 'several replacements of one family hide each of them',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality:<, equality:>',);
            expect(outcome.runnable,).toEqual([],);
            expect(outcome.ignored,).toEqual([
              '<',
              '>',
            ],);
          },
        },),

        it({
          name: 'a bare mention wins over a restricted mention of the same family',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality:>, equality',);
            expect(outcome.runnable,).toEqual([],);
          },
        },),

        it({
          name: 'duplicate entries are harmless',
          fn: async () => {
            const outcome = equalityOutcome('mutation-test-disable-next-line equality:>, equality:>',);
            expect(outcome.runnable,).toEqual(['<'],);
            expect(outcome.ignored,).toEqual(['>'],);
          },
        },),

        it({
          name: 'plain and restricted families mix in one directive',
          fn: async () => {
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: `// mutation-test-disable-next-line equality:<, arithmetic\n${COMPARISON.replace('a >= b', 'a + b >= b',)}\n`,
            },);
            expect(result.ignored
              .map(function toFamily(mutant,): string {
                return `${mutant.operator}:${mutant.replacement}`;
              },)
              .toSorted(),).toEqual([
                'arithmetic:-',
                'equality:<',
              ],);
          },
        },),

        it({
          name: 'a file directive restricts by replacement too',
          fn: async () => {
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: `// mutation-test-disable-file equality:>\n${COMPARISON}\n`,
            },);
            expect(result.ignored
              .filter(function equality(mutant,): boolean {
                return mutant.operator === 'equality';
              },)
              .map(function toText(mutant,): string {
                return mutant.replacement;
              },),).toEqual(['>'],);
          },
        },),

        it({
          name: 'the ignored mutant keeps the reason of the directive',
          fn: async () => {
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: `// mutation-test-disable-next-line equality:> -- tie cannot happen\n${COMPARISON}\n`,
            },);
            expect(result.ignored[0]?.reason,).toBe('tie cannot happen',);
          },
        },),
      ],
    },),

    describe({
      name: 'grammar boundaries',
      children: [
        it({
          name: 'a quoted replacement can hold commas, colons, spaces and the reason separator',
          fn: async () => {
            const text = ' mutation-test-disable-next-line string:"a, b: c -- d" -- why';
            expect(hides({
              text,
              operator: 'string',
              replacement: 'a, b: c -- d',
            },),).toBe(true,);
            expect(hides({
              text,
              operator: 'string',
              replacement: 'a',
            },),).toBe(false,);
            expect(parseDirective(text,)[0]?.reason,).toBe('why',);
          },
        },),

        it({
          name: 'a quoted replacement can hold a newline and an escaped quote',
          fn: async () => {
            expect(hides({
              text: String.raw` mutation-test-disable-next-line method:"x.slice(\n  0,\n  \"y\",\n)"`,
              operator: 'method',
              replacement: 'x.slice(\n  0,\n  "y",\n)',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'a quoted empty string is the empty replacement',
          fn: async () => {
            const text = ' mutation-test-disable-next-line optional-chaining:""';
            expect(hides({
              text,
              operator: 'optional-chaining',
              replacement: '',
            },),).toBe(true,);
            expect(hides({
              text,
              operator: 'optional-chaining',
              replacement: '.',
            },),).toBe(false,);
          },
        },),

        it({
          name: 'a quoted replacement is the same as the bare one when it needs no quoting',
          fn: async () => {
            expect(hides({
              text: ' mutation-test-disable-next-line equality:">="',
              operator: 'equality',
              replacement: '>=',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'a bare replacement may be the reason separator text',
          fn: async () => {
            expect(hides({
              text: ' mutation-test-disable-next-line update:-- -- why',
              operator: 'update',
              replacement: '--',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'a bare replacement may hold a colon',
          fn: async () => {
            expect(hides({
              text: ' mutation-test-disable-next-line string:a:b',
              operator: 'string',
              replacement: 'a:b',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'a family colon with nothing after it is rejected with a pointer to the quoted empty string',
          fn: async () => {
            expect(function parseEmptyReplacement(): unknown {
              return parseDirective(' mutation-test-disable-next-line string:',);
            },).toThrow('empty replacement',);
            expect(function parseEmptyBeforeComma(): unknown {
              return parseDirective(' mutation-test-disable-next-line string:, boolean',);
            },).toThrow('empty replacement',);
            expect(function parseEmptyBeforeReason(): unknown {
              return parseDirective(' mutation-test-disable-next-line string: -- why',);
            },).toThrow('empty replacement',);
          },
        },),

        it({
          name: 'a bare replacement with a double quote is rejected',
          fn: async () => {
            expect(function parseQuoteInBare(): unknown {
              return parseDirective(' mutation-test-disable-next-line string:a"b',);
            },).toThrow('double quote',);
          },
        },),

        it({
          name: 'an unterminated quoted replacement is rejected',
          fn: async () => {
            expect(function parseUnterminated(): unknown {
              return parseDirective(' mutation-test-disable-next-line string:"abc',);
            },).toThrow('unterminated',);
            expect(function parseEscapedEnd(): unknown {
              return parseDirective(String.raw` mutation-test-disable-next-line string:"abc\"`,);
            },).toThrow('unterminated',);
          },
        },),

        it({
          name: 'a quoted replacement with an invalid JSON escape is rejected',
          fn: async () => {
            expect(function parseBadEscape(): unknown {
              return parseDirective(String.raw` mutation-test-disable-next-line string:"a\qb"`,);
            },).toThrow('JSON string literal',);
          },
        },),

        it({
          name: 'text after a replacement that is not a comma or the reason is rejected',
          fn: async () => {
            expect(function parseSpaceInBare(): unknown {
              return parseDirective(' mutation-test-disable-next-line string:a b',);
            },).toThrow('unexpected text',);
            expect(function parseTextAfterQuoted(): unknown {
              return parseDirective(' mutation-test-disable-next-line string:"a"b',);
            },).toThrow('unexpected text',);
          },
        },),

        it({
          name: 'an unknown family is rejected with or without a replacement',
          fn: async () => {
            expect(function parseUnknownBare(): unknown {
              return parseDirective(' mutation-test-disable-next-line bogus',);
            },).toThrow('unknown mutation family bogus',);
            expect(function parseUnknownWithReplacement(): unknown {
              return parseDirective(' mutation-test-disable-next-line bogus:true',);
            },).toThrow('unknown mutation family bogus',);
          },
        },),

        it({
          name: 'an empty family entry is rejected',
          fn: async () => {
            expect(function parseLeadingComma(): unknown {
              return parseDirective(' mutation-test-disable-next-line , string',);
            },).toThrow('unknown mutation family',);
            expect(function parseDoubleComma(): unknown {
              return parseDirective(' mutation-test-disable-next-line string,, boolean',);
            },).toThrow('unknown mutation family',);
            expect(function parseTrailingComma(): unknown {
              return parseDirective(' mutation-test-disable-next-line string,',);
            },).toThrow('trailing comma',);
          },
        },),

        it({
          name: 'a family name keeps its own hyphen and still ends at the reason separator',
          fn: async () => {
            expect(hides({
              text: ' mutation-test-disable-next-line optional-chaining:. -- why',
              operator: 'optional-chaining',
              replacement: '.',
            },),).toBe(true,);
            expect(hides({
              text: ' mutation-test-disable-next-line string-- why',
              operator: 'string',
              replacement: '',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'a bare directive with only a reason suppresses every family',
          fn: async () => {
            expect(hides({
              text: ' mutation-test-disable-next-line -- why',
              operator: 'regex',
              replacement: '/x/',
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
