import { tokenize, } from '@csstools/css-tokenizer';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type CssEditState,
  type CssNode,
  CssParseError,
  isClosingToken,
  isCssAtRule,
  isCssDeclaration,
  isOpeningToken,
  isTriviaToken,
  parseCss,
  rawTextOfTokens,
  type StringCss,
  stringifyCss,
  stringifyNodes,
  tokenData,
  transformNodes,
  transformStylesheet,
} from '../dist/final/neutral/index.mjs';

//region Test helpers

/**
 Parses a plain string as CSS, asserting the brand at the test boundary.
 
 @param source - CSS text under test.
 @returns Parsed edit state.
 */
function parse(source: string,): CssEditState {
  return parseCss({ source: source as StringCss, },);
}

/**
 Visitor removing every at-rule with a given name via the empty-array form.
 
 @param name - At-rule name to remove.
 @returns Visitor for {@link transformNodes}.
 */
function removeAtRule(name: string,) {
  return function visit(node: CssNode,) {
    return (isCssAtRule(node,) && (node.name === name)) ? [] : node;
  };
}

/**
 First structural node parsed from a CSS fragment, for building replacements.
 
 @param source - Fragment to parse.
 @returns First non-trivia node.
 */
function nodeFrom(source: string,): CssNode {
  return nonNullishOrThrow(
    parse(source,).root.children.find(function keepStructural(node,) {
      return node.kind !== 'trivia';
    },),
  );
}

//endregion Test helpers

await describe({
  name: '',
  children: [
    describe({
      name: transformNodes.name,
      children: [
        //region Removal and trivia pruning

        it({
          name: 'removes a node and its leading whitespace when pruning',
          fn: async () => {
            const root = transformStylesheet({
              root: parse('.a { top: 0; }\n\n@mixin --x { top: 0; }\n.b { left: 0; }',).root,
              visit: removeAtRule('mixin',),
              pruneTriviaBeforeRemoved: true,
            },);
            expect(stringifyNodes({ nodes: root.children, },),).toBe(
              '.a { top: 0; }\n.b { left: 0; }',
            );
          },
        },),

        it({
          name: 'keeps comments in pruned leading trivia',
          fn: async () => {
            const root = transformStylesheet({
              root: parse('/* keep me */\n@mixin --x { top: 0; }\n.b { left: 0; }',).root,
              visit: removeAtRule('mixin',),
              pruneTriviaBeforeRemoved: true,
            },);
            expect(stringifyNodes({ nodes: root.children, },),).toBe(
              '/* keep me */\n.b { left: 0; }',
            );
          },
        },),

        it({
          name: 'keeps a comment-terminated trivia run whole when pruning',
          fn: async () => {
            const root = transformStylesheet({
              root: parse('/* keep */@mixin --x { top: 0; }.b { left: 0; }',).root,
              visit: removeAtRule('mixin',),
              pruneTriviaBeforeRemoved: true,
            },);
            expect(stringifyNodes({ nodes: root.children, },),).toBe(
              '/* keep */.b { left: 0; }',
            );
          },
        },),

        it({
          name: 'keeps leading trivia when not pruning',
          fn: async () => {
            const root = transformStylesheet({
              root: parse('.a { top: 0; }\n\n@mixin --x { top: 0; }',).root,
              visit: removeAtRule('mixin',),
            },);
            expect(stringifyNodes({ nodes: root.children, },),).toBe(
              '.a { top: 0; }\n\n',
            );
          },
        },),

        it({
          name: 'removes a node at list start without a preceding sibling',
          fn: async () => {
            const root = transformStylesheet({
              root: parse('@mixin --x { top: 0; }.b { left: 0; }',).root,
              visit: removeAtRule('mixin',),
              pruneTriviaBeforeRemoved: true,
            },);
            expect(stringifyNodes({ nodes: root.children, },),).toBe('.b { left: 0; }',);
          },
        },),

        it({
          name: 'skips pruning when the preceding sibling is not trivia',
          fn: async () => {
            const root = transformStylesheet({
              root: parse('.a { top: 0; }@mixin --x { top: 0; }',).root,
              visit: removeAtRule('mixin',),
              pruneTriviaBeforeRemoved: true,
            },);
            expect(stringifyNodes({ nodes: root.children, },),).toBe('.a { top: 0; }',);
          },
        },),

        //endregion Removal and trivia pruning

        //region Replacement and splicing

        it({
          name: 'replaces a node with a visitor-returned node',
          fn: async () => {
            const replacement = nodeFrom('left: 1px;',);
            const nodes = transformNodes({
              nodes: parse('.a { top: 0; }',).root.children,
              visit: function swapDeclarations(node,) {
                return isCssDeclaration(node,) ? replacement : node;
              },
            },);
            expect(stringifyNodes({ nodes, },),).toBe('.a { left: 1px; }',);
          },
        },),

        it({
          name: 'splices a multi-node array in place',
          fn: async () => {
            /**
             Replacement body including its interior trivia, so the splice
             carries its own separator whitespace.
             */
            const body = parse('top: 0; left: 1px;',).root.children;
            const nodes = transformNodes({
              nodes: parse('.a { @apply --card; }',).root.children,
              visit: function expandApply(node,) {
                return (isCssAtRule(node,) && (node.name === 'apply')) ? body : node;
              },
            },);
            expect(stringifyNodes({ nodes, },),).toBe('.a { top: 0; left: 1px; }',);
          },
        },),

        it({
          name: 'reaches nodes nested through at-rule and rule blocks bottom-up',
          fn: async () => {
            const nodes = transformNodes({
              nodes: parse('@media (width > 1px) { .a { @apply --x; } }',).root.children,
              visit: removeAtRule('apply',),
              pruneTriviaBeforeRemoved: true,
            },);
            expect(stringifyNodes({ nodes, },),).toBe('@media (width > 1px) { .a { } }',);
          },
        },),

        //endregion Replacement and splicing

        //region Structural sharing

        it({
          name: 'returns the same array and root when nothing changes',
          fn: async () => {
            const { root, } = parse('@import url(a.css);\n.a { top: 0; }',);
            const keptNodes = transformNodes({
              nodes: root.children,
              visit: function keepAll(node,) {
                return node;
              },
            },);
            expect(keptNodes,).toBe(root.children,);
            const keptRoot = transformStylesheet({
              root,
              visit: function keepAll(node,) {
                return node;
              },
            },);
            expect(keptRoot,).toBe(root,);
          },
        },),

        //endregion Structural sharing
      ],
    },),

    describe({
      name: stringifyCss.name,
      children: [
        it({
          name: 'renders statement at-rules without block or semicolon',
          fn: async () => {
            const source = '@apply --y';
            expect(stringifyCss({ state: parse(source,), },),).toBe(source,);
          },
        },),
      ],
    },),

    describe({
      name: CssParseError.name,
      children: [
        it({
          name: 'carries the failure offset in field and message',
          fn: async () => {
            const error = new CssParseError({
              message: 'boom',
              offset: 7,
            },);
            expect(error.name,).toBe('CssParseError',);
            expect(error.offset,).toBe(7,);
            expect(error.message,).toContain('offset 7',);
          },
        },),
      ],
    },),

    describe({
      name: 'token helpers',
      children: [
        it({
          name: 'classifies trivia, opening, and closing tokens',
          fn: async () => {
            const tokens = tokenize({ css: '/* c */ <!-- --> ({[ )}] .x', },);
            const trivia = tokens.filter(
              function keepTrivia(token,) {
                return isTriviaToken(token,);
              },
            );
            expect(trivia.length,).toBeGreaterThanOrEqual(4,);
            expect(
              tokens.filter(
                function keepOpening(token,) {
                  return isOpeningToken(token,);
                },
              ),
            ).toHaveLength(3,);
            expect(
              tokens.filter(
                function keepClosing(token,) {
                  return isClosingToken(token,);
                },
              ),
            ).toHaveLength(3,);
          },
        },),

        it({
          name: 'joins raw token text byte-exactly',
          fn: async () => {
            const css = '.x { content: "}{"; }';
            expect(rawTextOfTokens({ tokens: tokenize({ css, },), },),).toBe(css,);
          },
        },),

        it({
          name: 'reads parsed token data',
          fn: async () => {
            const tokens = tokenize({ css: String.raw`@\6d ixin`, },);
            const atKeyword = nonNullishOrThrow(tokens[0],);
            expect(tokenData(atKeyword,),).toEqual({ value: 'mixin', },);
          },
        },),
      ],
    },),
  ],
},);
