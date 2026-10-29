import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  closeValueOfOpenBracket,
  computeIndentExpectations,
  type ExpectationToken,
  type IndentExpectationRoute,
  type LineIndentExpectation,
} from '../../dist/final/node/index.mjs';

//region Helpers
// Structural stub builders and assertion shorthands; the model accepts plain
// objects, so no parser is needed to exercise every route.

/**
 Stub tree node: structural fields plus visitor-key-named child slots.
 
 Child slots repeat the stub shape so mis-typed children surface while type
 checking instead of silently failing the runtime shape guard.
 */
type StubNode = {
  readonly type: string;
  readonly start: number;
  readonly end: number;
  readonly [key: string]: string | number | undefined | StubNode | readonly StubNode[];
};

/** Parameters for {@link expectationsOf}. */
type ExpectationsOfParams = {
  readonly sourceText: string;
  readonly root: StubNode;
  readonly tokens: readonly ExpectationToken[];
  readonly visitorKeys: Readonly<Record<string, readonly string[]>>;
};

/**
 Runs the indentation model over one stub tree.
 
 @param params - source text, stub root tree, tokens, and visitor keys
 
 @returns one expectation entry per checked line
 */
function expectationsOf(params: ExpectationsOfParams,): readonly LineIndentExpectation[] {
  return computeIndentExpectations(params);
}

/** Parameters for {@link offsetOf}. */
type OffsetOfParams = {
  readonly sourceText: string;
  readonly marker: string;
  /**
   Optional offset the occurrence must strictly follow, so chained lookups
   walk forward instead of re-finding the previous marker.
   */
  readonly after?: number;
};

/**
 Offset of the first marker occurrence in `sourceText`.
 
 @param params - source text, marker, and optional offset the occurrence must follow
 
 @returns offset of the marker occurrence strictly after `after`, or the first one
 
 @throws when the marker does not occur
 */
function offsetOf({
  sourceText,
  marker,
  after,
}: OffsetOfParams,): number {
  const offset = sourceText.indexOf(
    marker,
    (after === undefined)
      ? 0
      : (after + 1),
  );
  if (offset === (-1)) {
    throw new Error(`marker not in source: ${marker}`,);
  }
  return offset;
}

/** Parameters for {@link tokenAt}. */
type TokenAtParams = {
  readonly type: string;
  readonly value: string;
  readonly start: number;
};

/**
 Builds a keyword, identifier, or punctuator token stub at one offset.
 
 @param params - structural type tag, token text, and source offset
 
 @returns token stub spanning `value`
 */
function tokenAt({
  type,
  value,
  start,
}: TokenAtParams,): ExpectationToken {
  return {
    type,
    value,
    start,
    end: start + value.length,
  };
}

/**
 Collects every line-leading and bracket token in `sourceText`.
 
 Keywords and identifiers arrive as word runs, punctuators as single
 characters; the model reads only token roles and line-leading offsets, so
 the scan need not reproduce full lexing.
 
 @param sourceText - text to scan
 
 @returns tokens sorted by offset
 */
function tokensOf(sourceText: string,): readonly ExpectationToken[] {
  const keywords = new Set([
    'if',
    'else',
    'try',
    'catch',
    'finally',
    'do',
    'while',
    'return',
    'function',
    'const',
    'class',
  ],);
  const punctuators = new Set([
    '(',
    ')',
    '[',
    ']',
    '{',
    '}',
    ';',
    ',',
    ':',
    '.',
    '=',
    '+',
    '*',
  ],);
  const tokens: ExpectationToken[] = [];
  let cursor = 0;
  while (cursor < sourceText.length) {
    const character = sourceText.charAt(cursor,);
    if (/\s/u.test(character,)) {
      cursor += 1;
      continue;
    }
    if (punctuators.has(character,)) {
      tokens.push(tokenAt({
        type: 'Punctuator',
        value: character,
        start: cursor,
      },),);
      cursor += 1;
      continue;
    }
    if (/[\p{Letter}\p{Number}]/u.test(character,)) {
      let end = cursor;
      while (
        (end < sourceText.length)
          && /[\p{Letter}\p{Number}]/u.test(sourceText.charAt(end,),)
      ) {
        end += 1;
      }
      const word = sourceText.slice(
        cursor,
        end,
      );
      tokens.push(tokenAt({
        type: keywords.has(word,) ? 'Keyword' : 'Identifier',
        value: word,
        start: cursor,
      },),);
      cursor = end;
      continue;
    }
    cursor += 1;
  }
  return tokens;
}

/** Parameters for {@link expectationAt}. */
type ExpectationAtParams = {
  readonly expectations: readonly LineIndentExpectation[];
  readonly lineIndex: number;
};

/**
 Expectation for one line index.
 
 @param params - expectations and target line index
 
 @returns the line's expectation entry
 
 @throws when the line has no entry
 */
function expectationAt({
  expectations,
  lineIndex,
}: ExpectationAtParams,): LineIndentExpectation {
  const found = expectations.find(
    function isLineExpectation(expectation,): boolean {
      return expectation.lineIndex === lineIndex;
    },
  );
  if (found === undefined) {
    throw new Error(`no expectation for line ${String(lineIndex,)}`,);
  }
  return found;
}

/** Parameters for {@link expectWidthAt}. */
type ExpectWidthAtParams = ExpectationAtParams & {
  readonly expectedWidth: number;
  readonly route: IndentExpectationRoute;
};

/**
 Asserts the model width and route assigned to one line.
 
 @param params - expectations, line index, and the width plus route to match
 */
function expectWidthAt({
  expectations,
  lineIndex,
  expectedWidth,
  route,
}: ExpectWidthAtParams,): void {
  const expectation = expectationAt({
    expectations,
    lineIndex,
  },);
  expect(expectation.expectedWidth,).toBe(expectedWidth,);
  expect(expectation.route,).toBe(route,);
}

/**
 Asserts the model produced no entry for one line (an exempt line).
 
 @param params - expectations and line index to check
 */
function expectExemptAt({
  expectations,
  lineIndex,
}: ExpectationAtParams,): void {
  const found = expectations.some(
    function isLineExpectation(expectation,): boolean {
      return expectation.lineIndex === lineIndex;
    },
  );
  expect(found,).toBe(false,);
}

/** Parameters for {@link blockWithStatement}. */
type BlockWithStatementParams = {
  readonly blockStart: number;
  readonly blockEnd: number;
  readonly statementStart: number;
};

/**
 Builds a block stub carrying one expression statement.
 
 Statement bodies appear in every suffix-keyword case, so one builder keeps
 their stub trees readable.
 
 @param params - block span offsets and the statement's start offset
 
 @returns block stub with its statement child
 */
function blockWithStatement({
  blockStart,
  blockEnd,
  statementStart,
}: BlockWithStatementParams,): StubNode {
  return {
    type: 'BlockStatement',
    start: blockStart,
    end: blockEnd,
    body: {
      type: 'ExpressionStatement',
      start: statementStart,
      end: statementStart + 4,
    },
  };
}
//endregion Helpers

//region Hanging signature
// Continuation lines, ancestor anchors, and owner-line closer bases.

await describe({
  name: 'computeIndentExpectations: hanging function signature',
  children: [
    it({
      name: 'anchors parameters and returns at the signature line',
      fn: async () => {
        const sourceText = 'function f(\n  a: number,\n): number {\n  return a;\n}';
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'FunctionDeclaration',
              start: 0,
              end: sourceText.length,
              body: {
                type: 'BlockStatement',
                start: offsetOf({
                  sourceText,
                  marker: '{',
                },),
                end: sourceText.length,
                body: {
                  type: 'ReturnStatement',
                  start: offsetOf({
                    sourceText,
                    marker: 'return',
                  },),
                  end: offsetOf({
                    sourceText,
                    marker: 'return',
                  },) + 10,
                },
              },
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            FunctionDeclaration: ['body'],
            BlockStatement: ['body'],
            ReturnStatement: [],
          },
        },);

        expect(expectations,).toHaveLength(5,);
        expectWidthAt({
          expectations,
          lineIndex: 0,
          expectedWidth: 0,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 2,
          expectedWidth: 0,
          route: 'closer',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 3,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 4,
          expectedWidth: 0,
          route: 'closer',
        },);
      },
    },),
  ],
},);

//endregion Hanging signature

//region Nested bare blocks
// Body-in-body remap disabled, so inner blocks keep their own base.

await describe({
  name: 'computeIndentExpectations: nested bare block',
  children: [
    it({
      name: 'anchors inner block content at the inner block line',
      fn: async () => {
        const sourceText = 'function nested(): void {\n  {\n    nested();\n  }\n}';
        const outerStart = offsetOf({
          sourceText,
          marker: '{',
        },);
        const innerStart = offsetOf({
          sourceText,
          marker: '{',
          after: outerStart,
        },);
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'FunctionDeclaration',
              start: 0,
              end: sourceText.length,
              body: {
                type: 'BlockStatement',
                start: outerStart,
                end: sourceText.length,
                body: {
                  type: 'BlockStatement',
                  start: innerStart,
                  end: offsetOf({
                    sourceText,
                    marker: '}',
                  },) + 1,
                  body: {
                    type: 'ExpressionStatement',
                    start: offsetOf({
                      sourceText,
                      marker: 'nested();',
                    },),
                    end: offsetOf({
                      sourceText,
                      marker: 'nested();',
                    },) + 9,
                  },
                },
              },
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            FunctionDeclaration: ['body'],
            BlockStatement: ['body'],
            ExpressionStatement: [],
          },
        },);

        expect(expectations,).toHaveLength(5,);
        expectWidthAt({
          expectations,
          lineIndex: 0,
          expectedWidth: 0,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 2,
          expectedWidth: 4,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 3,
          expectedWidth: 2,
          route: 'closer',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 4,
          expectedWidth: 0,
          route: 'closer',
        },);
      },
    },),
  ],
},);

//endregion Nested bare blocks

//region Class body
// Wrapper collapse for class bodies and decl-line closer bases.

await describe({
  name: 'computeIndentExpectations: class body wrapper',
  children: [
    it({
      name: 'collapses the class body onto the declaration and bases the closer there',
      fn: async () => {
        const sourceText = 'class C {\n  m(): number {\n    return 1;\n  }\n}';
        const classStart = offsetOf({
          sourceText,
          marker: '{',
        },);
        const methodStart = offsetOf({
          sourceText,
          marker: '{',
          after: classStart,
        },);
        const methodClose = offsetOf({
          sourceText,
          marker: '}',
        },);
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'ClassDeclaration',
              start: 0,
              end: sourceText.length,
              body: {
                type: 'ClassBody',
                start: classStart,
                end: sourceText.length,
                body: {
                  type: 'MethodDefinition',
                  start: offsetOf({
                    sourceText,
                    marker: 'm(): number',
                  },),
                  end: sourceText.length,
                  body: {
                    type: 'BlockStatement',
                    start: methodStart,
                    end: methodClose + 1,
                    body: {
                      type: 'ReturnStatement',
                      start: offsetOf({
                        sourceText,
                        marker: 'return',
                      },),
                      end: offsetOf({
                        sourceText,
                        marker: 'return',
                      },) + 10,
                    },
                  },
                },
              },
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            ClassDeclaration: ['body'],
            ClassBody: ['body'],
            MethodDefinition: ['body'],
            BlockStatement: ['body'],
            ReturnStatement: [],
          },
        },);

        expect(expectations,).toHaveLength(5,);
        expectWidthAt({
          expectations,
          lineIndex: 0,
          expectedWidth: 0,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 2,
          expectedWidth: 4,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 3,
          expectedWidth: 2,
          route: 'closer',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 4,
          expectedWidth: 0,
          route: 'closer',
        },);
      },
    },),
  ],
},);

//endregion Class body

//region Suffix keywords
// `else`, `catch`, `finally`, and do-while `while` align with their statement
// lines; same-text identifiers take the ancestor route instead.

await describe({
  name: 'computeIndentExpectations: suffix keywords',
  children: [
    it({
      name: 'aligns suffix keywords with their statement lines',
      fn: async () => {
        const sourceText = [
          'if (a) {',
          '  b();',
          '}',
          'else {',
          '  c();',
          '}',
          'try {',
          '  d();',
          '}',
          'catch (e) {',
          '  f();',
          '}',
          'finally {',
          '  g();',
          '}',
          'do {',
          '  h();',
          '}',
          'while (x);',
        ].join('\n',);
        const closeBrace1 = offsetOf({
          sourceText,
          marker: '}',
        },);
        const closeBrace2 = offsetOf({
          sourceText,
          marker: '}',
          after: closeBrace1,
        },);
        const closeBrace3 = offsetOf({
          sourceText,
          marker: '}',
          after: closeBrace2,
        },);
        const closeBrace4 = offsetOf({
          sourceText,
          marker: '}',
          after: closeBrace3,
        },);
        const closeBrace5 = offsetOf({
          sourceText,
          marker: '}',
          after: closeBrace4,
        },);
        const closeBrace6 = offsetOf({
          sourceText,
          marker: '}',
          after: closeBrace5,
        },);
        const openBrace1 = offsetOf({
          sourceText,
          marker: '{',
        },);
        const openBrace2 = offsetOf({
          sourceText,
          marker: '{',
          after: openBrace1,
        },);
        const openBrace3 = offsetOf({
          sourceText,
          marker: '{',
          after: openBrace2,
        },);
        const openBrace4 = offsetOf({
          sourceText,
          marker: '{',
          after: openBrace3,
        },);
        const openBrace5 = offsetOf({
          sourceText,
          marker: '{',
          after: openBrace4,
        },);
        const openBrace6 = offsetOf({
          sourceText,
          marker: '{',
          after: openBrace5,
        },);

        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: [
              {
                type: 'IfStatement',
                start: 0,
                end: closeBrace2 + 1,
                consequent: blockWithStatement({
                  blockStart: openBrace1,
                  blockEnd: closeBrace1 + 1,
                  statementStart: offsetOf({
                    sourceText,
                    marker: 'b();',
                  },),
                }),
                alternate: blockWithStatement({
                  blockStart: openBrace2,
                  blockEnd: closeBrace2 + 1,
                  statementStart: offsetOf({
                    sourceText,
                    marker: 'c();',
                  },),
                }),
              },
              {
                type: 'TryStatement',
                start: offsetOf({
                  sourceText,
                  marker: 'try',
                },),
                end: closeBrace5 + 1,
                block: blockWithStatement({
                  blockStart: openBrace3,
                  blockEnd: closeBrace3 + 1,
                  statementStart: offsetOf({
                    sourceText,
                    marker: 'd();',
                  },),
                }),
                handler: {
                  type: 'CatchClause',
                  start: offsetOf({
                    sourceText,
                    marker: 'catch',
                  },),
                  end: closeBrace4 + 1,
                  body: blockWithStatement({
                    blockStart: openBrace4,
                    blockEnd: closeBrace4 + 1,
                    statementStart: offsetOf({
                      sourceText,
                      marker: 'f();',
                    },),
                  }),
                },
                finalizer: blockWithStatement({
                  blockStart: openBrace5,
                  blockEnd: closeBrace5 + 1,
                  statementStart: offsetOf({
                    sourceText,
                    marker: 'g();',
                  },),
                }),
              },
              {
                type: 'DoWhileStatement',
                start: offsetOf({
                  sourceText,
                  marker: 'do',
                },),
                end: sourceText.length,
                body: blockWithStatement({
                  blockStart: openBrace6,
                  blockEnd: closeBrace6 + 1,
                  statementStart: offsetOf({
                    sourceText,
                    marker: 'h();',
                  },),
                }),
              },
            ],
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            IfStatement: ['consequent', 'alternate'],
            BlockStatement: ['body'],
            TryStatement: ['block', 'handler', 'finalizer'],
            CatchClause: ['body'],
            DoWhileStatement: ['body'],
            ExpressionStatement: [],
          },
        },);

        expect(expectations,).toHaveLength(19,);
        expectWidthAt({
          expectations,
          lineIndex: 3,
          expectedWidth: 0,
          route: 'suffix-keyword',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 4,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 9,
          expectedWidth: 0,
          route: 'suffix-keyword',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 12,
          expectedWidth: 0,
          route: 'suffix-keyword',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 18,
          expectedWidth: 0,
          route: 'suffix-keyword',
        },);
      },
    },),
    it({
      name: 'keeps an else-named property on the ancestor route',
      fn: async () => {
        const sourceText = 'const o = {\n  else: 1,\n};';
        const propertyStart = offsetOf({
          sourceText,
          marker: 'else',
        },);
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'VariableDeclaration',
              start: 0,
              end: sourceText.length,
              init: {
                type: 'ObjectExpression',
                start: offsetOf({
                  sourceText,
                  marker: '{',
                },),
                end: offsetOf({
                  sourceText,
                  marker: '}',
                },) + 1,
                properties: {
                  type: 'Property',
                  start: propertyStart,
                  end: propertyStart + 8,
                },
              },
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            VariableDeclaration: ['init'],
            ObjectExpression: ['properties'],
            Property: [],
          },
        },);

        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 2,
          route: 'ancestor',
        },);
      },
    },),
  ],
},);

//endregion Suffix keywords

//region Closer bases and exemptions
// Unmatched closers self-match, innermost frames win, non-brace owners keep
// the opener line, and blank, comment-led, and template lines stay exempt.

await describe({
  name: 'computeIndentExpectations: closer bases and exemptions',
  children: [
    it({
      name: 'self-matches an unmatched closer and exempts non-code lines',
      fn: async () => {
        const sourceText = [
          'const a = 1;',
          '',
          '    /* c */ const b = 2;',
          'const t = `',
          '      template content',
          '`;',
          '   )',
        ].join('\n',);
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'VariableDeclaration',
              start: offsetOf({
                sourceText,
                marker: 'const t',
              },),
              end: offsetOf({
                sourceText,
                marker: '`;',
              },) + 2,
              init: {
                type: 'TemplateLiteral',
                start: offsetOf({
                  sourceText,
                  marker: 'const t = `',
                },) + 10,
                end: offsetOf({
                  sourceText,
                  marker: '`;',
                },) + 1,
              },
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            VariableDeclaration: ['init'],
            TemplateLiteral: [],
          },
        },);

        expect(expectations,).toHaveLength(3,);
        expectWidthAt({
          expectations,
          lineIndex: 0,
          expectedWidth: 0,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 3,
          expectedWidth: 0,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 6,
          expectedWidth: 3,
          route: 'unmatched',
        },);
        expectExemptAt({
          expectations,
          lineIndex: 1,
        },);
        expectExemptAt({
          expectations,
          lineIndex: 2,
        },);
        expectExemptAt({
          expectations,
          lineIndex: 4,
        },);
        expectExemptAt({
          expectations,
          lineIndex: 5,
        },);
      },
    },),
    it({
      name: 'matches a closer to its own bracket below unrelated open frames',
      fn: async () => {
        const sourceText = 'z =\n{\n  v(2,\n};';
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'Weird',
              start: 0,
              end: sourceText.length,
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            Weird: [],
          },
        },);

        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 3,
          expectedWidth: 2,
          route: 'closer',
        },);
      },
    },),
    it({
      name: 'bases a bracket closer on its opener line',
      fn: async () => {
        const sourceText = 'const t = [\n  1,\n];';
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: sourceText.length,
            body: {
              type: 'VariableDeclaration',
              start: 0,
              end: sourceText.length,
              init: {
                type: 'ArrayExpression',
                start: offsetOf({
                  sourceText,
                  marker: '[',
                },),
                end: offsetOf({
                  sourceText,
                  marker: ']',
                },) + 1,
              },
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            VariableDeclaration: ['init'],
            ArrayExpression: [],
          },
        },);

        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 2,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 2,
          expectedWidth: 0,
          route: 'closer',
        },);
      },
    },),
    it({
      name: 'anchors tokens outside every record at source root',
      fn: async () => {
        const sourceText = 'q\nr';
        const expectations = expectationsOf({
          sourceText,
          root: {
            type: 'Program',
            start: 0,
            end: 1,
            body: {
              type: 'Weird',
              start: 2,
              end: 3,
            },
          },
          tokens: tokensOf(sourceText,),
          visitorKeys: {
            Program: ['body'],
            Weird: [],
          },
        },);

        expect(expectations,).toHaveLength(2,);
        expectWidthAt({
          expectations,
          lineIndex: 0,
          expectedWidth: 0,
          route: 'ancestor',
        },);
        expectWidthAt({
          expectations,
          lineIndex: 1,
          expectedWidth: 0,
          route: 'ancestor',
        },);
      },
    },),
  ],
},);

//endregion Closer bases and exemptions

//region Bracket text
// Close-bracket mapping including its unreachable-value throw.

await describe({
  name: 'closeValueOfOpenBracket',
  children: [
    it({
      name: 'maps every open bracket to its close bracket',
      fn: async () => {
        expect(closeValueOfOpenBracket('(',),).toBe(')',);
        expect(closeValueOfOpenBracket('[',),).toBe(']',);
        expect(closeValueOfOpenBracket('{',),).toBe('}',);
      },
    },),
    it({
      name: 'throws on values that are not open brackets',
      fn: async () => {
        /** Captured failure; the throw is the contract for non-bracket input. */
        const caught = (function catchNonBracket(): unknown {
          try {
            closeValueOfOpenBracket('=',);
            return undefined;
          }
          catch (error: unknown) {
            return error;
          }
        })();
        expect(Error.isError(caught,),).toBe(true,);
        expect(String(caught,),).toContain('Not an open bracket',);
      },
    },),
  ],
},);

//endregion Bracket text
