/**
 Guards on the color-token rules in `styles.css`, which no linter enforces.

 The `css` skill (`\.agents/skills/css/SKILL.md`) requires every color to come
 from a custom property and forbids a `var()` fallback except for a
 user-configurable value, because a fallback hardcodes a second color and hides a
 missing token instead of surfacing it. This stylesheet declares no
 user-configurable properties, so the fallback ban applies to all of it, and
 every custom property a declaration references has to be defined in the same
 file, since the package imports no stylesheet that could supply one.

 Issue 1628 recorded `background-color: var(--color-bg, #fff)` here: a
 hardcoded fallback for a theme token that the file never defined.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 This stylesheet, the only stylesheet the package ships.
 */
const STYLESHEET_PATH = join(
  import.meta.dirname,
  'styles.css',
);

/**
 Prefix every color token in the design-token system carries.
 */
const COLOR_TOKEN_PREFIX = '--color-';

/**
 Selector whose rule may define a color token at the document root.
 */
const ROOT_SELECTOR = ':root';

/**
 Function reading a custom property.
 */
const VAR_FUNCTION = 'var(';

/**
 One `property: value` pair, with the selector of the rule holding it.
 */
type Declaration = {
  /**
   Selector of the rule the declaration sits in.
   */
  readonly selector: string;
  /**
   Property name as written, including the leading `--` of a custom property.
   */
  readonly property: string;
  /**
   Property value with surrounding whitespace removed.
   */
  readonly value: string;
};

//endregion Fixtures

//region Helpers

/**
 Removes comment content so prose about colors is never read as a declaration.

 @param css - stylesheet text as written

 @returns the same text with every comment replaced by nothing

 @example
 ```ts
 stripComments('*\/ no *\/ body { color: red; }');
 ```
 */
function stripComments(css: string,): string {
  /**
   Text accumulated outside comments.
   */
  let text = '';
  /**
   Whether a block comment is open.
   */
  let insideComment = false;
  for (let index = 0; index < css.length; index += 1) {
    if ((!insideComment) && css.startsWith('/*', index,)) {
      insideComment = true;
      index += 1;
      continue;
    }
    if (insideComment) {
      if (css.startsWith('*/', index,)) {
        insideComment = false;
        index += 1;
      }
      continue;
    }
    text += css[index];
  }
  return text;
}

/**
 Splits a stylesheet into its declarations, keeping the selector each one sits under.

 Nesting-free stylesheets only, which is what this package ships: a `{` opens a
 rule, `;` ends a declaration, and `}` returns to selector text.

 @param css - stylesheet text as written

 @returns every declaration in source order

 @example
 ```ts
 declarations(':root { --color-bg: Canvas; }');
 ```
 */
function declarations(css: string,): readonly Declaration[] {
  /**
   Declarations collected so far.
   */
  const found: Declaration[] = [];
  /**
   Selector text accumulated for the rule being read.
   */
  let selector = '';
  /**
   Property text accumulated for the declaration being read.
   */
  let property = '';
  /**
   Value text accumulated for the declaration being read.
   */
  let value = '';
  /**
   Part of the stylesheet the scan is inside.
   */
  let part: 'selector' | 'property' | 'value' = 'selector';
  for (const character of stripComments(css,)) {
    if (part === 'selector') {
      if (character === '{') {
        selector = selector.trim();
        property = '';
        value = '';
        part = 'property';
        continue;
      }
      selector += character;
      continue;
    }
    if (part === 'property') {
      if (character === ':') {
        property = property.trim();
        part = 'value';
        continue;
      }
      if (character === '}') {
        selector = '';
        part = 'selector';
        continue;
      }
      property += character;
      continue;
    }
    if ((character === ';') || (character === '}')) {
      if (property.trim().length > 0) {
        found.push({
          selector,
          property: property.trim(),
          value: value.trim(),
        },);
      }
      property = '';
      value = '';
      if (character === '}') {
        selector = '';
        part = 'selector';
      }
      else {
        part = 'property';
      }
      continue;
    }
    value += character;
  }
  return found;
}

/**
 Reads the arguments written inside every `var()` call of one value.

 @param value - declaration value as written

 @returns text between `var(` and its closing parenthesis, one per call

 @example
 ```ts
 varCalls('var(--color-bg, #fff)');
 ```
 */
function varCalls(value: string,): readonly string[] {
  /**
   Argument text collected so far.
   */
  const calls: string[] = [];
  for (let from = value.indexOf(VAR_FUNCTION,); from !== (-1); from = value.indexOf(VAR_FUNCTION, from + 1,)) {
    /**
     Depth of nested parentheses, starting inside the `var(` call.
     */
    let depth = 1;
    /**
     Column the argument text ends at, or the value length when unclosed.
     */
    let end = from + VAR_FUNCTION.length;
    for (; (end < value.length) && (depth > 0); end += 1) {
      if (value[end] === '(')
        depth += 1;
      else if (value[end] === ')') {
        depth -= 1;
      }
    }
    calls.push(value.slice(from + VAR_FUNCTION.length, (depth === 0) ? end - 1 : value.length,),);
  }
  return calls;
}

/**
 Reads the custom property name inside one `var()` argument list.

 @param argument - text between `var(` and its closing parenthesis

 @returns property name, empty when the argument opens with nothing nameable

 @example
 ```ts
 varName('--color-bg, #fff');
 ```
 */
function varName(argument: string,): string {
  /**
   Column the fallback separator starts at, or -1 when the call passes no fallback.
   */
  const commaIndex = argument.indexOf(',',);
  return ((commaIndex === (-1)) ? argument : argument.slice(0, commaIndex,)).trim();
}

//endregion Helpers

//region Tests

/**
 Stylesheet text, read once for every assertion below.
 */
const css = await readFile(STYLESHEET_PATH, 'utf8',);

/**
 Every declaration in the stylesheet, read once for every assertion below.
 */
const allDeclarations = declarations(css,);

/**
 Custom properties the stylesheet defines, each with the selectors of its rules.
 */
const definedProperties = new Map<string, readonly string[]>();

for (const declaration of allDeclarations) {
  if (!declaration.property.startsWith('--',))
    continue;
  definedProperties.set(
    declaration.property,
    [...(definedProperties.get(declaration.property,) ?? []), declaration.selector,],
  );
}

/**
 Declarations that reach a custom property through `var()`.
 */
const varDeclarations = allDeclarations.filter(function usesVar(declaration,) {
  return declaration.value.includes(VAR_FUNCTION,);
},);

await describe({
  name: '',
  children: [
    describe({
      name: 'stylesheet color tokens',
      children: [
        it({
          name: 'reads at least one var() call, so the checks below have a subject',
          fn: async () => {
            expect(varDeclarations.length,)
              .toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'passes no fallback to var(), which would hardcode a second color',
          fn: async () => {
            /**
             Referenced property and full argument list of every var() call with a fallback.
             */
            const withFallback = varDeclarations.flatMap(function fallbacks(declaration,) {
              return varCalls(declaration.value,)
                .filter(function hasComma(argument,) {
                  return argument.includes(',',);
                },)
                .map(function describeCall(argument,) {
                  return `${declaration.selector} { ${declaration.property} } -> var(${argument})`;
                },);
            },);
            expect(withFallback,)
              .toEqual([],);
          },
        },),
        it({
          name: 'references only custom properties the stylesheet defines',
          fn: async () => {
            /**
             Referenced property name of every var() call, in source order.
             */
            const referenced = varDeclarations.flatMap(function names(declaration,) {
              return varCalls(declaration.value,)
                .map(function nameOf(argument,) {
                  return varName(argument,);
                },);
            },);
            /**
             Referenced names with no defining declaration in this file.
             */
            const undefinedNames = referenced.filter(function undefinedHere(name,) {
              return !definedProperties.has(name,);
            },);
            expect(undefinedNames,)
              .toEqual([],);
          },
        },),
        it({
          name: 'defines every color token at the document root',
          fn: async () => {
            /**
             Color tokens defined somewhere other than a `:root` rule, with their selectors.
             */
            const misplaced = [...definedProperties.entries(),]
              .filter(function isColorToken(entry,) {
                return entry[0].startsWith(COLOR_TOKEN_PREFIX,);
              },)
              .flatMap(function selectors(entry,) {
                return entry[1]
                  .filter(function notRoot(selector,) {
                    return selector !== ROOT_SELECTOR;
                  },)
                  .map(function describeSelector(selector,) {
                    return `${entry[0]} in ${selector}`;
                  },);
              },);
            expect(misplaced,)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
