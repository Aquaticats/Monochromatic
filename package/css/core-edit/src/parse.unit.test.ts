import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type CssAtRule,
  type CssEditState,
  type CssNode,
  CssParseError,
  type CssRule,
  isCssAtRule,
  isCssDeclaration,
  isCssRule,
  parseCss,
  type StringCss,
  stringifyCss,
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
 Full adversarial corpus: unknown at-rules with mixed bodies, relaxed and
 `&` nesting, braces and semicolons inside strings and url(), escaped
 selectors, custom-property block values, CDO/CDC, and comments.
 */
const ADVERSARIAL_CORPUS = `/* license comment */
<!-- -->
@charset "utf-8";
@import url('pkg/a.css') layer(base);
@mixin --card {
  padding: 1rem;
  .inner { color: red; }
  &:hover { color: blue; }
  @apply --flex;
}
.btn { @apply --card; background: var(--primary); span { color: green; } }
@media (width > 40rem) { .btn { gap: 1px; } }
.x { background: url("a}b;c.png"); content: "}{;*/"; --raw: { nested: token }; }
.esc\\{aped { color: red; }
.y[data-x="}"]::before { content: "\\"q\\""; }`;

/**
 Non-trivia children of a node list.
 
 @param nodes - Node list to filter.
 @returns Structural nodes only.
 */
function structural(nodes: readonly CssNode[],): readonly CssNode[] {
  return nodes.filter(function keepStructural(node,) {
    return node.kind !== 'trivia';
  },);
}

/**
 Kinds of the non-trivia children of a node list, in order.
 
 @param nodes - Node list to summarize.
 @returns Kind strings of structural children.
 */
function structuralKinds(nodes: readonly CssNode[],): readonly string[] {
  return structural(nodes,).map(function kindOf(node,) {
    return node.kind;
  },);
}

/**
 First qualified rule in a parsed document.
 
 @param state - Parsed edit state.
 @returns First rule node.
 */
function firstRule(state: CssEditState,) {
  return nonNullishOrThrow(
    state.root.children.find(
      function pickRule(node,): node is CssRule {
        return isCssRule(node,);
      },
    ),
  );
}

/**
 First at-rule with a given name in a parsed document.
 
 @param state - Parsed edit state.
 @param name - At-rule name to find.
 @returns Matching at-rule node.
 */
function atRuleNamed(
  state: CssEditState,
  name: string,
) {
  return nonNullishOrThrow(
    state.root.children
      .filter(
      function keepAtRules(node,): node is CssAtRule {
        return isCssAtRule(node,);
      },
      )
      .find(function named(node,) {
        return node.name === name;
      },),
  );
}

//endregion Test helpers

await describe({
  name: '',
  children: [
    describe({
      name: parseCss.name,
      children: [
        //region Round-trip fidelity

        it({
          name: 'round-trips the adversarial corpus byte-exactly',
          fn: async () => {
            const state = parse(ADVERSARIAL_CORPUS,);
            expect(stringifyCss({ state, },),).toBe(ADVERSARIAL_CORPUS,);
          },
        },),

        it({
          name: 'round-trips an empty document',
          fn: async () => {
            expect(stringifyCss({ state: parse('',), },),).toBe('',);
          },
        },),

        it({
          name: 'round-trips a trivia-only document',
          fn: async () => {
            const source = '/* only */\n\n';
            expect(stringifyCss({ state: parse(source,), },),).toBe(source,);
          },
        },),

        //endregion Round-trip fidelity

        //region Structure

        it({
          name: 'parses unknown at-rule bodies into declarations, nested rules, and at-rules',
          fn: async () => {
            const mixin = atRuleNamed(
              parse(ADVERSARIAL_CORPUS,),
              'mixin',
            );
            const block = nonNullishOrThrow(mixin.block,);
            expect(structuralKinds(block.children,),).toEqual(
              ['declaration', 'rule', 'rule', 'atRule',],
            );
          },
        },),

        it({
          name: 'classifies relaxed nesting without & as a rule',
          fn: async () => {
            const rule = firstRule(parse('.a { color: blue; span { color: green; } }',),);
            expect(structuralKinds(rule.block.children,),).toEqual(
              ['declaration', 'rule',],
            );
          },
        },),

        it({
          name: 'reclassifies ident-colon runs with block values as rules (span:hover trap)',
          fn: async () => {
            const rule = firstRule(parse('.a { span:hover { color: red; } top: 0; }',),);
            expect(structuralKinds(rule.block.children,),).toEqual(
              ['rule', 'declaration',],
            );
          },
        },),

        it({
          name: 'keeps block values inside custom-property declarations',
          fn: async () => {
            const rule = firstRule(parse('.a { --raw: { nested: token }; top: 0; }',),);
            expect(structuralKinds(rule.block.children,),).toEqual(
              ['declaration', 'declaration',],
            );
          },
        },),

        it({
          name: 'parses a declaration whose colon follows a comment',
          fn: async () => {
            const rule = firstRule(parse('.a { color /* note */ : red; }',),);
            expect(structuralKinds(rule.block.children,),).toEqual(['declaration',],);
          },
        },),

        it({
          name: 'ends a declaration at the closing brace when the semicolon is omitted',
          fn: async () => {
            const source = '.a { color: red }';
            const state = parse(source,);
            expect(stringifyCss({ state, },),).toBe(source,);
            expect(structuralKinds(firstRule(state,).block.children,),).toEqual(
              ['declaration',],
            );
          },
        },),

        it({
          name: 'ends a top-level declaration run at end of input',
          fn: async () => {
            const state = parse('color: red',);
            expect(structuralKinds(state.root.children,),).toEqual(['declaration',],);
          },
        },),

        it({
          name: 'unescapes at-rule names into the name field',
          fn: async () => {
            const source = String.raw`@\6d ixin --a { top: 0; }`;
            const state = parse(source,);
            expect(atRuleNamed(
              state,
              'mixin',
            ).name,).toBe('mixin',);
            expect(stringifyCss({ state, },),).toBe(source,);
          },
        },),

        it({
          name: 'parses statement at-rules terminated by semicolon, block end, and end of input',
          fn: async () => {
            const state = parse('.a { @apply --x }\n@apply --y',);
            const inner = structural(firstRule(state,).block.children,)
              .filter(
              function keepAtRules(node,): node is CssAtRule {
                return isCssAtRule(node,);
              },
              );
            expect(inner[0]?.name,).toBe('apply',);
            expect(inner[0]?.semicolonToken,).toBeUndefined();
            const topApply = atRuleNamed(
              state,
              'apply',
            );
            expect(topApply.block,).toBeUndefined();
          },
        },),

        it({
          name: 'keeps parenthesized semicolons and braces inside at-rule preludes',
          fn: async () => {
            const source = '@supports (font: ";{") { .a { top: 0; } }';
            const state = parse(source,);
            expect(stringifyCss({ state, },),).toBe(source,);
            expect(atRuleNamed(
              state,
              'supports',
            ).block,).toBeDefined();
          },
        },),

        it({
          name: 'parses consecutive declarations into distinct nodes',
          fn: async () => {
            const rule = firstRule(parse('.a { top: 0; left: 1px; }',),);
            expect(
              structural(rule.block.children,).filter(
                function keepDeclarations(node,) {
                  return isCssDeclaration(node,);
                },
              ),
            ).toHaveLength(2,);
          },
        },),

        //endregion Structure

        //region Strict errors

        it({
          name: 'throws on an unclosed block',
          fn: async () => {
            expect(function parseUnclosed() {
              parse('.a { color: red;',);
            },).toThrow('closing brace',);
          },
        },),

        it({
          name: 'throws on a stray closing brace at top level',
          fn: async () => {
            expect(function parseStray() {
              parse('}',);
            },).toThrow('stray closing brace',);
          },
        },),

        it({
          name: 'throws on a rule prelude that never reaches a block',
          fn: async () => {
            expect(function parseBlockless() {
              parse('.selector-only',);
            },).toThrow('without a block',);
          },
        },),

        it({
          name: 'throws on a semicolon inside a rule prelude',
          fn: async () => {
            expect(function parseSemicolonPrelude() {
              parse('.a; { top: 0; }',);
            },).toThrow('semicolon before any block',);
          },
        },),

        it({
          name: 'throws on a rule prelude cut off by its enclosing block',
          fn: async () => {
            expect(function parseCutoff() {
              parse('.a { .b }',);
            },).toThrow('without a block of its own',);
          },
        },),

        it({
          name: 'throws on an unbalanced closing token in a content run',
          fn: async () => {
            expect(function parseUnbalanced() {
              parse('.a { color: )red; }',);
            },).toThrow('unbalanced closing token',);
          },
        },),

        it({
          name: 'throws on an unbalanced closing token in an at-rule prelude',
          fn: async () => {
            expect(function parseUnbalancedPrelude() {
              parse('@media )screen { }',);
            },).toThrow('at-rule prelude',);
          },
        },),

        it({
          name: 'throws a positioned CssParseError on tokenizer failures',
          fn: async () => {
            let caught: unknown;
            try {
              parse('.a { content: "unterminated\n; }',);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(CssParseError,);
            expect((caught as CssParseError).message,).toContain('tokenizer error',);
            expect((caught as CssParseError).offset,).toBeGreaterThanOrEqual(0,);
          },
        },),

        //endregion Strict errors
      ],
    },),
  ],
},);
