/**
 Curated CSS conformance corpus against css-edit's strict CST semantics,
 category-modeled on the css-parsing-tests suite (tabatkins): stylesheet
 structure, at-rules, declarations, nesting, strings and escapes, and
 error recovery boundaries. Valid cases parse to expected top-level shapes
 and round-trip byte-exactly; invalid cases throw `CssParseError`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  asCssSource,
  CssParseError,
  parseCss,
  stringifyCss,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  assert,
  constantFrom,
  property,
  tuple,
} from 'fast-check';

//region Case tables

/**
 One valid conformance case: source plus the expected non-trivia top-level
 node kinds in order.
 */
type ValidCase = {
  readonly label: string;
  readonly input: string;
  readonly topLevelKinds: readonly string[];
};

/**
 Valid corpus. Every case must parse and round-trip byte-exactly.
 */
const validCases: readonly ValidCase[] = [
  //region Stylesheet structure
  { label: 'empty stylesheet', input: '', topLevelKinds: [], },
  { label: 'whitespace only', input: ' \n\t\f ', topLevelKinds: [], },
  { label: 'comment only', input: '/* c */', topLevelKinds: [], },
  { label: 'unterminated-free CDO CDC guards', input: '<!-- .a { top: 0; } -->', topLevelKinds: ['rule',], },
  { label: 'rule with empty block', input: '.a {}', topLevelKinds: ['rule',], },
  { label: 'two rules', input: '.a{}.b{}', topLevelKinds: ['rule', 'rule',], },
  //endregion Stylesheet structure

  //region At-rules
  { label: 'charset-like statement', input: '@charset "utf-8";', topLevelKinds: ['atRule',], },
  { label: 'import with layer and media', input: "@import url('x.css') layer(a) screen;", topLevelKinds: ['atRule',], },
  { label: 'statement at-rule without semicolon at EOF', input: '@layer base', topLevelKinds: ['atRule',], },
  { label: 'block at-rule', input: '@media (width > 1px) { .a { top: 0; } }', topLevelKinds: ['atRule',], },
  { label: 'unknown at-rule with declaration body', input: '@mixin --m { top: 0; }', topLevelKinds: ['atRule',], },
  { label: 'nested at-rules', input: '@layer a { @layer b { .x { top: 0; } } }', topLevelKinds: ['atRule',], },
  { label: 'escaped at-keyword', input: String.raw`@\6d ixin --m { top: 0; }`, topLevelKinds: ['atRule',], },
  { label: 'at-rule prelude with nested parens', input: '@supports (not (display: flex)) { .a { top: 0; } }', topLevelKinds: ['atRule',], },
  //endregion At-rules

  //region Declarations
  { label: 'declaration without trailing semicolon', input: '.a { top: 0 }', topLevelKinds: ['rule',], },
  { label: 'important flag', input: '.a { top: 0 !important; }', topLevelKinds: ['rule',], },
  { label: 'custom property with block value', input: '.a { --x: { y: z }; }', topLevelKinds: ['rule',], },
  { label: 'custom property with empty value', input: '.a { --x: ; }', topLevelKinds: ['rule',], },
  { label: 'colon after comment', input: '.a { top /* c */ : 0; }', topLevelKinds: ['rule',], },
  { label: 'semicolon inside parens', input: '.a { background: url("a;b.png"); }', topLevelKinds: ['rule',], },
  //endregion Declarations

  //region Nesting
  { label: 'ampersand nesting', input: '.a { &:hover { top: 0; } }', topLevelKinds: ['rule',], },
  { label: 'relaxed nesting', input: '.a { span { top: 0; } }', topLevelKinds: ['rule',], },
  { label: 'ident-colon selector reclassified', input: '.a { span:hover { top: 0; } }', topLevelKinds: ['rule',], },
  { label: 'media nested in rule', input: '.a { @media (width > 1px) { top: 0; } }', topLevelKinds: ['rule',], },
  //endregion Nesting

  //region Strings and escapes
  { label: 'braces in string', input: '.a { content: "}{"; }', topLevelKinds: ['rule',], },
  { label: 'comment opener in string', input: '.a { content: "/*"; }', topLevelKinds: ['rule',], },
  { label: 'escaped quote in string', input: String.raw`.a { content: "\""; }`, topLevelKinds: ['rule',], },
  { label: 'escaped brace in selector', input: String.raw`.esc\{aped { top: 0; }`, topLevelKinds: ['rule',], },
  { label: 'attribute selector with brace string', input: '.a[data-x="}"] { top: 0; }', topLevelKinds: ['rule',], },
  { label: 'unicode escape in ident', input: String.raw`.a { \74 op: 0; }`, topLevelKinds: ['rule',], },
  //endregion Strings and escapes
];

/**
 One invalid conformance case: source plus the expected error fragment.
 */
type InvalidCase = {
  readonly label: string;
  readonly input: string;
  readonly messageFragment: string;
};

/**
 Invalid corpus. Every case must throw `CssParseError` mentioning the
 fragment.
 */
const invalidCases: readonly InvalidCase[] = [
  { label: 'unclosed block', input: '.a {', messageFragment: 'closing brace', },
  { label: 'stray top-level close', input: '} .a {}', messageFragment: 'stray closing brace', },
  { label: 'selector without block', input: '.only-selector', messageFragment: 'without a block', },
  { label: 'semicolon in selector prelude', input: '.a; {}', messageFragment: 'semicolon before any block', },
  { label: 'selector cut off by block end', input: '.a { .b }', messageFragment: 'without a block of its own', },
  { label: 'unbalanced close paren in value', input: '.a { top: )0; }', messageFragment: 'unbalanced closing token', },
  { label: 'unbalanced close in at-rule prelude', input: '@media )x {}', messageFragment: 'at-rule prelude', },
  { label: 'newline in string', input: '.a { content: "x\n"; }', messageFragment: 'tokenizer error', },
  { label: 'unterminated comment', input: '.a { top: 0; } /* never closed', messageFragment: 'tokenizer error', },
  { label: 'unterminated string at EOF', input: '.a { content: "open', messageFragment: 'tokenizer error', },
];

//endregion Case tables

//region Amplification arbitraries

/**
 Default fast-check runs for context amplification; `CSS_EDIT_FUZZ_RUNS`
 overrides for longer campaigns.
 */
const DEFAULT_AMPLIFICATION_RUNS = 150;

/**
 Parsed override from the environment, or `NaN` when unset or unparseable.
 */
const runsOverride = Number(process.env
  .CSS_EDIT_FUZZ_RUNS,);

/**
 Effective amplification run count.
 */
const amplificationRuns = Number.isFinite(runsOverride,)
  ? runsOverride
  : DEFAULT_AMPLIFICATION_RUNS;

/**
 Structural contexts every valid case must survive: identity, block
 at-rules, and a style-rule body (css-edit's unified block contents accept
 statement at-rules and declarations in any block).
 */
const contextArb = constantFrom(
  {
    prefix: '',
    suffix: '',
  },
  {
    prefix: '@media (width > 1px) {\n',
    suffix: '\n}',
  },
  {
    prefix: '@layer amp {\n',
    suffix: '\n}',
  },
  {
    prefix: '.amp-host {\n',
    suffix: '\n}',
  },
);

/**
 Leading and trailing trivia variations.
 */
const triviaArb = constantFrom(
  '',
  ' ',
  '\n',
  '\t\n',
  '/* amp */',
  '\n/* amp */\n',
);

/**
 One valid case drawn from the curated corpus.
 */
const validCaseArb = constantFrom(...validCases,);

//endregion Amplification arbitraries

await describe({
  name: '',
  children: [
    describe({
      name: 'valid corpus',
      children: validCases.map(function toValidTest(validCase,) {
        return it({
          name: `${validCase.label}: parses and round-trips byte-exactly`,
          fn: async () => {
            /**
             Parsed state of the case input.
             */
            const state = parseCss({ source: asCssSource(validCase.input,), },);
            expect(stringifyCss({ state, },),).toBe(validCase.input,);
            /**
             Non-trivia top-level kinds in source order.
             */
            const kinds = state.root.children
              .filter(function keepStructural(node,) {
                return node.kind !== 'trivia';
              },)
              .map(function kindOf(node,) {
                return node.kind;
              },);
            expect(kinds,).toEqual(validCase.topLevelKinds,);
          },
        },);
      },),
    },),

    describe({
      name: 'context amplification',
      children: [
        it({
          name: 'every valid case survives generated structural contexts byte-exactly',
          fn: async () => {
            assert(
              property(
                tuple(
                  validCaseArb,
                  contextArb,
                  triviaArb,
                  triviaArb,
                ),
                ([validCase, context, lead, trail,],) => {
                  /**
                   Case embedded in a generated context with surrounding trivia.
                   */
                  const amplified = `${lead}${context.prefix}${validCase.input}${context.suffix}${trail}`;
                  /**
                   Parsed state of the amplified document.
                   */
                  const state = parseCss({ source: asCssSource(amplified,), },);
                  expect(stringifyCss({ state, },),).toBe(amplified,);
                },
              ),
              { numRuns: amplificationRuns, },
            );
          },
        },),
      ],
    },),

    describe({
      name: 'invalid corpus',
      children: invalidCases.map(function toInvalidTest(invalidCase,) {
        return it({
          name: `${invalidCase.label}: throws CssParseError`,
          fn: async () => {
            let caught: unknown;
            try {
              parseCss({ source: asCssSource(invalidCase.input,), },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(CssParseError,);
            expect((caught as CssParseError).message,).toContain(invalidCase.messageFragment,);
          },
        },);
      },),
    },),
  ],
},);
