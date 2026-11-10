/**
 Guards on the CSS quality rules in `renderStyles()`, which no linter enforces.

 The `css` skill (`\.agents/skills/css/SKILL.md`) requires `rem` sizing, logical
 properties, colors from custom properties only, and longhands instead of the
 `margin`, `padding`, `border`, `background` and `font` shorthands. This
 stylesheet is the template resumed demo work copies, so each rule is read out of
 the generated text here, and a violation names the rule and declaration it broke.

 Issue 1633 recorded the stylesheet breaking all four at once: `px` sizing,
 physical `top`/`left`/`right`/`bottom`, raw hex and `rgba()` colors, and
 multi-concept shorthands.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { renderStyles, } from './styles.ts';

//region Fixtures

/**
 Selector of the rule that declares the palette.
 */
const ROOT_SELECTOR = ':root';

/**
 Prefix every color token in the design-token system carries.
 */
const COLOR_TOKEN_PREFIX = '--color-';

/**
 Function reading a custom property.
 */
const VAR_FUNCTION = 'var(';

/**
 Length unit the skill bans outside device-pixel contexts.
 */
const FORBIDDEN_LENGTH_UNIT = 'px';

/**
 Physical property names the skill replaces with logical ones.
 */
const PHYSICAL_PROPERTIES: ReadonlySet<string> = new Set([
  'top',
  'right',
  'bottom',
  'left',
  'width',
  'height',
  'max-width',
  'max-height',
  'min-width',
  'min-height',
  'text-align-left',
  'text-align-right',
]);

/**
 Side suffixes a logical property replaces wherever a physical one carries them.
 */
const PHYSICAL_SIDES: readonly string[] = [
  'top',
  'right',
  'bottom',
  'left',
];

/**
 Property families whose physical side spelling is banned.
 */
const SIDED_FAMILIES: readonly string[] = [
  'margin',
  'padding',
  'border',
  'inset',
];

/**
 Shorthands combining unrelated axes or sub-properties.
 */
const FORBIDDEN_SHORTHANDS: ReadonlySet<string> = new Set([
  'margin',
  'padding',
  'border',
  'background',
  'font',
]);

/**
 Value opening a hardcoded color: a hex literal or a color function.
 */
const COLOR_LITERAL_PATTERN = /#[0-9a-f]{3,8}\b|(?:rgba?|hsla?|oklch|oklab|lab|lch|color)\(/iu;

/**
 Color keywords a hardcoded declaration uses instead of a token.
 */
const NAMED_COLORS: readonly string[] = [
  'aliceblue',
  'aqua',
  'black',
  'blue',
  'brown',
  'coral',
  'crimson',
  'cyan',
  'fuchsia',
  'gold',
  'gray',
  'green',
  'grey',
  'indigo',
  'ivory',
  'khaki',
  'lime',
  'magenta',
  'maroon',
  'navy',
  'olive',
  'orange',
  'orchid',
  'pink',
  'plum',
  'purple',
  'red',
  'salmon',
  'silver',
  'tan',
  'teal',
  'tomato',
  'turquoise',
  'violet',
  'wheat',
  'white',
  'yellow',
];

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
    if (!insideComment && css.startsWith('/*', index,)) {
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
 declarations(':root { --color-background: Canvas; }');
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
 varCalls('var(--color-background)');
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
 varName('--color-background');
 ```
 */
function varName(argument: string,): string {
  return argument.split(',',)[0]
    .trim();
}

/**
 Names a declaration in a failure message, so a reader finds the line.

 @param declaration - declaration as parsed

 @returns selector, property and rule family, joined for one line

 @example
 ```ts
 describeDeclaration({ selector: '#error div', property: 'border', value: '1px solid red' });
 ```
 */
function describeDeclaration(declaration: Declaration,): string {
  return `${declaration.selector} { ${declaration.property}: ${declaration.value} }`;
}

//endregion Helpers

//region Tests

/**
 Generated stylesheet text, read once for every assertion below.
 */
const css = stripComments(renderStyles(),);

/**
 Every declaration in the stylesheet, read once for every assertion below.
 */
const allDeclarations = declarations(renderStyles(),);

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

/**
 Declarations that carry a hardcoded color, ignoring the palette in `:root`.
 */
const colorLiteralDeclarations = allDeclarations.filter(function hasLiteral(declaration,) {
  return (declaration.selector !== ROOT_SELECTOR) || !declaration.property.startsWith('--',);
},)
  .filter(function hardcoded(declaration,) {
    if (COLOR_LITERAL_PATTERN.test(declaration.value,))
      return true;
    return NAMED_COLORS.some(function namesColor(name,) {
      return new RegExp(`(?:^|[^\\w-])${name}(?:$|[^\\w-])`, 'iu',).test(declaration.value,);
    },);
  },);

await describe({
  name: '',
  children: [
    describe({
      name: 'generated stylesheet',
      children: [
        it({
          name: 'declares the palette in a :root rule',
          fn: async () => {
            /**
             Color tokens defined at the document root.
             */
            const rootTokens = [...definedProperties.entries(),]
              .filter(function isColorToken(entry,) {
                return entry[1].includes(ROOT_SELECTOR,)
                  && entry[0].startsWith(COLOR_TOKEN_PREFIX,);
              },);
            expect(rootTokens.length,)
              .toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'sizes with rem instead of px',
          fn: async () => {
            /**
             Declarations holding a pixel length, with the text that holds it.
             */
            const pixelDeclarations = allDeclarations
              .filter(function hasPixelLength(declaration,) {
                return /(?:^|[^\w-])\d*\.?\d+px(?:$|[^\w-])/iu.test(declaration.value,);
              },)
              .map(function describe(declaration,) {
                return describeDeclaration(declaration,);
              },);
            expect(pixelDeclarations,)
              .toEqual([],);
          },
        },),
        it({
          name: 'positions and aligns with logical properties',
          fn: async () => {
            /**
             Declarations naming a physical side, with the text that names it.
             */
            const physicalDeclarations = allDeclarations
              .filter(function physical(declaration,) {
                if (PHYSICAL_PROPERTIES.has(declaration.property,))
                  return true;
                /**
                 Value naming `left` or `right`, which only `text-align` may carry.
                 */
                const namesSideValue = (declaration.property === 'text-align')
                  && ((declaration.value === 'left') || (declaration.value === 'right'));
                if (namesSideValue)
                  return true;
                return SIDED_FAMILIES.some(function family(prefix,) {
                  return PHYSICAL_SIDES.some(function side(suffix,) {
                    return declaration.property === `${prefix}-${suffix}`;
                  },);
                },);
              },)
              .map(function describe(declaration,) {
                return describeDeclaration(declaration,);
              },);
            expect(physicalDeclarations,)
              .toEqual([],);
          },
        },),
        it({
          name: 'keeps every color in the palette a custom property',
          fn: async () => {
            /**
             Declarations hardcoding a color outside the palette.
             */
            const hardcoded = colorLiteralDeclarations.map(function describe(declaration,) {
              return describeDeclaration(declaration,);
            },);
            expect(hardcoded,)
              .toEqual([],);
          },
        },),
        it({
          name: 'expands shorthands that combine unrelated axes or sub-properties',
          fn: async () => {
            /**
             Declarations written with a banned shorthand.
             */
            const shorthandDeclarations = allDeclarations
              .filter(function shorthand(declaration,) {
                return FORBIDDEN_SHORTHANDS.has(declaration.property,);
              },)
              .map(function describe(declaration,) {
                return describeDeclaration(declaration,);
              },);
            expect(shorthandDeclarations,)
              .toEqual([],);
          },
        },),
        it({
          name: 'reads at least one var() call, so the token checks have a subject',
          fn: async () => {
            expect(varDeclarations.length,)
              .toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'passes no fallback to var(), which would hardcode a second color',
          fn: async () => {
            /**
             Every var() call carrying a fallback, named by its declaration.
             */
            const withFallback = varDeclarations.flatMap(function fallbacks(declaration,) {
              return varCalls(declaration.value,)
                .filter(function hasComma(argument,) {
                  return argument.includes(',',);
                },)
                .map(function describeCall(argument,) {
                  return `${describeDeclaration(declaration,)} -> var(${argument})`;
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
             Referenced names with no defining declaration in this file.
             */
            const undefinedNames = varDeclarations.flatMap(function names(declaration,) {
              return varCalls(declaration.value,)
                .map(function nameOf(argument,) {
                  return varName(argument,);
                },);
            },)
              .filter(function undefinedHere(name,) {
                return !definedProperties.has(name,);
              },);
            expect(undefinedNames,)
              .toEqual([],);
          },
        },),
        it({
          name: 'carries no !important',
          fn: async () => {
            /**
             Declarations marked `!important`.
             */
            const importantDeclarations = allDeclarations
              .filter(function important(declaration,) {
                return declaration.value.includes('!important',);
              },)
              .map(function describe(declaration,) {
                return describeDeclaration(declaration,);
              },);
            expect(importantDeclarations,)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Tests
