/**
 fast-check arbitraries generating structurally valid CSS documents:
 declarations (custom properties included), nested rules (`&` and relaxed),
 statement and block at-rules, comments, adversarial strings, and varied
 whitespace. Used by every property file in this package.
 
 @module
 */

import { ASCII_LOWERCASE_LETTER_CHARS, } from '@monochromatic-dev/const-core/ts';
import {
  array,
  type Arbitrary,
  constantFrom,
  oneof,
  tuple,
} from 'fast-check';

//region Atoms

/**
 Lowercase identifier-safe characters for generated names.
 */
const IDENT_CHARS = ASCII_LOWERCASE_LETTER_CHARS;

/**
 Generated CSS identifier: a guaranteed leading letter plus a short tail.
 */
const identArb: Arbitrary<string> = tuple(
  constantFrom(...IDENT_CHARS,),
  array(
    constantFrom(
      ...IDENT_CHARS,
      '-',
      '0',
      '9',
    ),
    { maxLength: 6, },
  ),
)
  .map(function joinIdent([head, tail,],) {
    return `${head}${tail.join('',)}`;
  },);

/**
 Property name: an ordinary ident or a custom property.
 */
const propertyNameArb: Arbitrary<string> = oneof(
  identArb,
  identArb.map(function toCustomProperty(name,) {
    return `--${name}`;
  },),
);

/**
 Declaration value: keywords, dimensions, var() references, url() targets,
 and strings carrying structural characters that must stay inert.
 */
const valueArb: Arbitrary<string> = oneof(
  identArb,
  constantFrom(
    '1rem',
    '62.5%',
    '0',
    '1px solid gray',
    'var(--primary)',
    'var(--x, fallback)',
    'url("a}b;c.png")',
    "url('plain.css')",
    '"}{;"',
    "'*/{'",
    'calc(100% - 2rem)',
  ),
);

/**
 Inter-node whitespace and comment trivia.
 */
const triviaArb: Arbitrary<string> = constantFrom(
  ' ',
  '\n',
  '\n  ',
  '\t',
  '\n/* note */\n',
  ' /* c */ ',
);

/**
 Selector prelude: simple, compound, nesting (`&` and relaxed), and
 attribute selectors carrying structural characters in strings.
 */
const selectorArb: Arbitrary<string> = oneof(
  identArb.map(function toClassSelector(name,) {
    return `.${name}`;
  },),
  identArb.map(function toIdSelector(name,) {
    return `#${name}`;
  },),
  identArb,
  identArb.map(function toNestedPseudo(name,) {
    return `&:${name}`;
  },),
  identArb.map(function toDescendant(name,) {
    return `& > .${name}`;
  },),
  constantFrom(
    '[data-x="}"]',
    '*',
    'a:hover',
    '.x::before',
  ),
);

//endregion Atoms

//region Composites

/**
 One declaration, optionally missing its semicolon when generated last.
 */
const declarationArb: Arbitrary<string> = tuple(
  propertyNameArb,
  valueArb,
)
  .map(function joinDeclaration([name, value,],) {
    return `${name}: ${value};`;
  },);

/**
 Block items at a given remaining depth: declarations always; nested rules
 and at-rules while depth remains.
 
 @param depth - Remaining nesting depth.
 
 @returns Arbitrary for one block item.
 */
function blockItemArb(depth: number,): Arbitrary<string> {
  if (depth <= 0)
    return declarationArb;
  return oneof(
    {
      weight: 4,
      arbitrary: declarationArb,
    },
    {
      weight: 1,
      arbitrary: ruleArb(depth - 1,),
    },
    {
      weight: 1,
      arbitrary: atRuleArb(depth - 1,),
    },
  );
}

/**
 Braced block of items joined by trivia.
 
 @param depth - Remaining nesting depth.
 
 @returns Arbitrary for one braced block including braces.
 */
function blockArb(depth: number,): Arbitrary<string> {
  return tuple(
    array(
      blockItemArb(depth,),
      { maxLength: 4, },
    ),
    triviaArb,
  )
    .map(function joinBlock([items, trivia,],) {
      return `{${trivia}${items.join(trivia,)}${trivia}}`;
    },);
}

/**
 Qualified rule: selector prelude plus block.
 
 @param depth - Remaining nesting depth.
 
 @returns Arbitrary for one rule.
 */
function ruleArb(depth: number,): Arbitrary<string> {
  return tuple(
    selectorArb,
    blockArb(depth,),
  )
    .map(function joinRule([selector, block,],) {
      return `${selector} ${block}`;
    },);
}

/**
 At-rule: statement form or block form, known and unknown names mixed.
 
 @param depth - Remaining nesting depth.
 
 @returns Arbitrary for one at-rule.
 */
function atRuleArb(depth: number,): Arbitrary<string> {
  /**
   Statement at-rules with structural characters confined to strings.
   */
  const statementArb = constantFrom(
    '@layer base;',
    "@import url('x.css') layer(base);",
    '@apply --thing;',
    '@charset "utf-8";',
  );
  /**
   Block at-rule names covering known and custom cases.
   */
  const blockNameArb = constantFrom(
    '@media (width > 40rem)',
    '@supports (display: flex)',
    '@mixin --card',
    '@layer base',
  );
  return oneof(
    statementArb,
    tuple(
      blockNameArb,
      blockArb(depth,),
    )
      .map(function joinAtRule([name, block,],) {
        return `${name} ${block}`;
      },),
  );
}

//endregion Composites

//region Document

/**
 Maximum nesting depth for generated documents.
 */
const MAX_DEPTH = 2;

/**
 Whole CSS document: top-level rules, at-rules, and trivia runs, joined by
 varied whitespace. Structurally valid by construction, so parsing must
 succeed and stringification must reproduce it byte-exactly.
 
 @example
 ```ts
 assert(property(cssDocumentArb, (css) => { ... }), { numRuns: fuzzRuns });
 ```
 */
export const cssDocumentArb: Arbitrary<string> = tuple(
  array(
    oneof(
      ruleArb(MAX_DEPTH,),
      atRuleArb(MAX_DEPTH,),
    ),
    { maxLength: 5, },
  ),
  triviaArb,
)
  .map(function joinDocument([items, trivia,],) {
    return `${items.join(trivia,)}${trivia}`;
  },);

//endregion Document
