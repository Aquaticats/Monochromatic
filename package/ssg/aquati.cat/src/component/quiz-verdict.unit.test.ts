/**
 Tests that quiz verdict colours come from the site's colour tokens.
 
 A quiz marks an answer right or wrong with colour alone, so the two hues have
 to follow the theme: a hue picked for the light lavender surface loses its
 contrast on the near-black dark one.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  darkModeTokenStyles,
  questionCheckboxStyles,
  questionRadioStyles,
  tokenStyles,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Token naming the hue a correct answer is marked with.
 */
const OK_TOKEN = '--color-ok';

/**
 Token naming the hue a wrong answer is marked with.
 */
const ERR_TOKEN = '--color-err';

/**
 Colour words a stylesheet could spell instead of referencing a token.
 */
const COLOUR_WORDS = [
  'green',
  'red',
] as const;

/**
 Colour-literal markers a stylesheet could spell instead of referencing a token.
 */
const COLOUR_LITERALS = [
  '#',
  'hsl(',
  'rgb(',
] as const;

/**
 Derivation the verdict token documentation promises.
 */
const DERIVATION = 'color-mix(';

/**
 Question stylesheets paired with the component each one styles.
 */
const QUESTION_STYLESHEETS = [
  {
    name: 'question-radio',
    styles: questionRadioStyles(),
  },
  {
    name: 'question-checkbox',
    styles: questionCheckboxStyles(),
  },
] as const;

//endregion Fixtures

//region Helpers

/**
 Every value a generated CSS block declares for one custom property.
 
 A block declares the same property more than once when a media query or the
 theme toggle overrides it, so the list, not the first hit, is what a theme
 comparison needs.
 
 @param styles - CSS text that may declare the property
 
 @param property - custom property name, including its leading `--`
 
 @returns declared values in source order, empty when the block omits the property
 
 @example
 ```ts
 declaredValuesIn(':root{--color-ok:green;}', '--color-ok');
 // ['green']
 ```
 */
function declaredValuesIn(
  styles: string,
  property: string,
): string[] {
  /**
   Delimiter ending every declaration of the property.
   */
  const marker = `${property}:`;

  return styles
    .split(marker,)
    .slice(1,)
    .map(function valueBeforeEnd(part,) {
      /**
       Offsets of the delimiters that can end a declaration; `-1` means absent.
       */
      const ends = [
        part.indexOf(';',),
        part.indexOf('}',),
      ]
        .filter(function isPresent(index,) {
          return index !== (-1);
        },);
      /**
       Offset where this declaration's value stops, or the whole remainder when neither delimiter follows.
       */
      const stop = ends.length > 0 ? Math.min(...ends,) : part.length;
      return part.slice(
        0,
        stop,
      )
        .trim();
    },);
}

//endregion Helpers

await describe({
  name: 'quiz verdict colours',
  children: [
    ...QUESTION_STYLESHEETS.map(function verdictTokens({ name, styles, },) {
      return it({
        name: `takes both verdict colours from the theme tokens in ${name} styles`,
        fn: async function takesVerdictColoursFromTokens(): Promise<void> {
          expect(styles,).toContain(`var(${OK_TOKEN})`,);
          expect(styles,).toContain(`var(${ERR_TOKEN})`,);
        },
      },);
    },),
    ...QUESTION_STYLESHEETS.map(function noRawColour({ name, styles, },) {
      return it({
        name: `spells no raw verdict colour in ${name} styles`,
        fn: async function spellsNoRawColour(): Promise<void> {
          for (const word of COLOUR_WORDS)
            expect(styles,).not.toContain(word,);
          for (const literal of COLOUR_LITERALS)
            expect(styles,).not.toContain(literal,);
        },
      },);
    },),
    it({
      name: 'declares each verdict colour once per theme',
      fn: async function declaresOneVerdictColourPerTheme(): Promise<void> {
        for (const token of [OK_TOKEN, ERR_TOKEN,]) {
          expect(declaredValuesIn(
            tokenStyles(),
            token,
          ),).toHaveLength(1,);
          expect(declaredValuesIn(
            darkModeTokenStyles(),
            token,
          ),).toHaveLength(1,);
        }
      },
    },),
    it({
      name: 'derives each verdict colour from the theme foreground',
      fn: async function derivesVerdictColourFromForeground(): Promise<void> {
        for (const token of [OK_TOKEN, ERR_TOKEN,]) {
          for (const [value,] of [
            declaredValuesIn(
              tokenStyles(),
              token,
            ),
            declaredValuesIn(
              darkModeTokenStyles(),
              token,
            ),
          ])
            expect(value,).toContain(DERIVATION,);
        }
      },
    },),
    it({
      name: 'gives the light and dark theme different verdict colours',
      fn: async function differsPerTheme(): Promise<void> {
        for (const token of [OK_TOKEN, ERR_TOKEN,]) {
          /**
           Value the light theme declares for the token.
           */
          const light = declaredValuesIn(
            tokenStyles(),
            token,
          )[0];
          /**
           Value the dark theme declares for the token.
           */
          const dark = declaredValuesIn(
            darkModeTokenStyles(),
            token,
          )[0];

          expect(light,).not.toBe(dark,);
        }
      },
    },),
  ],
},);
