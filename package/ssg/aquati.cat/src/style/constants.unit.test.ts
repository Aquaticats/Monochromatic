/**
 Tests for the shared CSS-side constants and the component styles that use them.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { css as calloutStyles, } from '../component/callout-alert.ts';
import { css as langSwitcherStyles, } from '../component/lang-switcher.ts';
import { css as questionCheckboxStyles, } from '../component/question-checkbox.ts';
import { css as questionRadioStyles, } from '../component/question-radio.ts';
import { css as siteHeaderStyles, } from '../component/site-header.ts';
import { css as siteSearchStyles, } from '../component/site-search.ts';
import { FONT_WEIGHT_EMPHASIS, } from './constants.ts';

/**
 Marker prefixing every emitted `font-weight` declaration.
 */
const FONT_WEIGHT_MARKER = 'font-weight:';

/**
 Extracts every `font-weight` value from a CSS string without a regex, so the
 scan stays linear in the stylesheet length.
 
 A declaration ends at the next `;` or, for the last declaration in a rule, at
 the closing `}`.
 
 @param styles - generated CSS to scan
 
 @returns one trimmed value per `font-weight` declaration, in source order
 
 @example
 ```ts
 fontWeightValuesIn('.a{font-weight:600}');
 // ['600']
 ```
 */
function fontWeightValuesIn(styles: string,): string[] {
  return styles
    .split(FONT_WEIGHT_MARKER,)
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

/**
 Component stylesheets that emphasise text, paired with their emitting module name.
 */
const STYLESHEETS = [
  {
    name: 'callout-alert',
    styles: calloutStyles(),
  },
  {
    name: 'lang-switcher',
    styles: langSwitcherStyles(),
  },
  {
    name: 'question-checkbox',
    styles: questionCheckboxStyles(),
  },
  {
    name: 'question-radio',
    styles: questionRadioStyles(),
  },
  {
    name: 'site-header',
    styles: siteHeaderStyles(),
  },
  {
    name: 'site-search',
    styles: siteSearchStyles(),
  },
] as const;

await describe({
  name: 'FONT_WEIGHT_EMPHASIS',
  children: [
    it({
      name: 'declares the emphasis weight once for every component to share',
      fn: async function declaresOneSharedWeight(): Promise<void> {
        expect(FONT_WEIGHT_EMPHASIS,).toBe(600,);
      },
    },),
    ...STYLESHEETS.map(function stylesheetCase({ name, styles, },) {
      return it({
        name: `emits only the shared emphasis weight in ${name} styles`,
        fn: async function emitsOnlySharedWeight(): Promise<void> {
          /**
           Shared weight in the string form the CSS generators emit.
           */
          const expected = String(FONT_WEIGHT_EMPHASIS,);

          for (const weight of fontWeightValuesIn(styles,))
            expect(weight,).toBe(expected,);
        },
      },);
    },),
    it({
      name: 'is actually emphasised somewhere',
      fn: async function isActuallyEmitted(): Promise<void> {
        /**
         Number of `font-weight` declarations across every component stylesheet,
         which proves the scans above are not vacuous.
         */
        const emitted = STYLESHEETS.flatMap(function weights({ styles, },) {
          return fontWeightValuesIn(styles,);
        },);

        expect(emitted.length,).toBeGreaterThan(0,);
        expect(emitted,).toContain(String(FONT_WEIGHT_EMPHASIS,),);
      },
    },),
  ],
},);
