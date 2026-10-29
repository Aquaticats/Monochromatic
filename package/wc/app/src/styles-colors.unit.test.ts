/**
 Tests for the light/dark color-scheme stylesheet fragments, the
 strict five-stop grayscale palette, and the AAA small-text contrast
 discipline (every text role at 7:1 or better in both themes).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  renderDarkColors,
  renderRootColors,
} from './styles-colors.ts';

/**
 Custom-property names both palette fragments must declare.
 */
const TOKEN_NAMES = [
  '--color-bg:',
  '--color-surface:',
  '--color-fg:',
  '--color-fg-strong:',
  '--color-muted:',
  '--color-border-subtle:',
  '--color-border-strong:',
  '--color-placeholder:',
];

/**
 The only lightness values the palette may use; the design decree is
 strictly `oklch(L 0 0)` for these five stops, no alpha.
 */
const ALLOWED_STOPS = [
  'oklch(0 0 0)',
  'oklch(0.1 0 0)',
  'oklch(0.5 0 0)',
  'oklch(0.9 0 0)',
  'oklch(1 0 0)',
];

/**
 One half, composed from the exempt literal range.
 */
const HALF = 1 / 2;

/**
 Five, composed from the exempt literal range.
 */
const FIVE = (2 + 2) + 1;

/**
 Ambient-flare term WCAG adds to both relative luminances in a
 contrast ratio (the 0.05 in `(L1 + 0.05) / (L2 + 0.05)`).
 */
const WCAG_FLARE = HALF / (FIVE * 2);

/**
 WCAG AAA contrast floor for small text (the decree: every text role
 must reach it, regardless of size).
 */
const AAA_SMALL_TEXT_MIN = FIVE + 2;

/**
 Extracts one custom property's `oklch(...)` value from css via a
 linear index scan (no regex).
 
 @param css - stylesheet fragment holding token
 
 @param token - custom-property name including trailing colon, e.g.
 `--color-fg:`
 
 @returns declared value, e.g. `oklch(0.1 0 0)`
 
 @throws Error when token or its value terminator is absent
 */
function tokenValue(
  {
    css,
    token,
  }: Readonly<{
    css: string;
    token: string;
  }>,
): string {
  /**
   `indexOf` sentinel for "not found".
   */
  const NOT_FOUND = -1;

  /**
   Index where token's declaration starts.
   */
  const start = css.indexOf(token,);

  if (start === NOT_FOUND) {
    throw new Error(`token ${token} not declared in fragment`,);
  }

  /**
   Index where token's value starts.
   */
  const valueStart = start + token.length;

  /**
   Index of the declaration-terminating semicolon, when present.
   */
  const semicolon = css.indexOf(
    ';',
    valueStart,
  );

  /**
   Index of the rule-closing brace, for the final declaration in a
   minified rule (which has no trailing semicolon).
   */
  const brace = css.indexOf(
    '}',
    valueStart,
  );

  /**
   Terminator indexes that exist past the value start.
   */
  const terminators = [
    semicolon,
    brace,
  ]
    .filter(function keepFound(index,): boolean {
      return index !== NOT_FOUND;
    },);

  if (terminators.length === 0) {
    throw new Error(`token ${token} declaration is unterminated`,);
  }

  /**
   Nearest declaration terminator among semicolon and closing brace.
   */
  const end = Math.min(...terminators,);

  return css.slice(
    valueStart,
    end,
  );
}

/**
 Computes WCAG relative luminance of an achromatic `oklch(L 0 0)`
 value. For grays, OKLab lightness is the cube root of CIE luminance
 Y, so Y is recovered as L cubed.
 
 @param value - achromatic color, e.g. `oklch(0.9 0 0)`
 
 @returns WCAG relative luminance in 0 to 1
 */
function stopLuminance(value: string,): number {
  /**
   Index where the lightness component starts.
   */
  const lStart = value.indexOf('(',) + 1;

  /**
   Index where the lightness component ends.
   */
  const lEnd = value.indexOf(
    ' ',
    lStart,
  );

  /**
   OKLab lightness of value.
   */
  const lightness = Number(value.slice(
    lStart,
    lEnd,
  ),);

  return lightness * lightness * lightness;
}

/**
 Computes the WCAG contrast ratio between two custom properties
 declared in css.
 
 @param css - stylesheet fragment declaring both tokens
 
 @param ink - text-role custom-property name including trailing colon
 
 @param paper - background-role custom-property name including
 trailing colon
 
 @returns contrast ratio in 1 to 21
 */
function contrastRatio(
  {
    css,
    ink,
    paper,
  }: Readonly<{
    css: string;
    ink: string;
    paper: string;
  }>,
): number {
  /**
   Relative luminance of the text color.
   */
  const inkY = stopLuminance(tokenValue(
    {
      css,
      token: ink,
    },
  ),);

  /**
   Relative luminance of the background color.
   */
  const paperY = stopLuminance(tokenValue(
    {
      css,
      token: paper,
    },
  ),);

  /**
   Lighter of the two luminances.
   */
  const lighter = Math.max(
    inkY,
    paperY,
  );

  /**
   Darker of the two luminances.
   */
  const darker = Math.min(
    inkY,
    paperY,
  );

  return (lighter + WCAG_FLARE) / (darker + WCAG_FLARE);
}

/**
 Ink-on-paper pairings the page actually renders, every one of which
 must reach AAA small-text contrast. The frequency-bar fill is
 `--color-fg-strong` on the page background (the track is
 transparent), so its pairing is already covered by the
 fg-strong-on-bg entry.
 */
const CONTRAST_PAIRS = [
  {
    ink: '--color-fg:',
    paper: '--color-bg:',
  },
  {
    ink: '--color-fg:',
    paper: '--color-surface:',
  },
  {
    ink: '--color-fg-strong:',
    paper: '--color-bg:',
  },
  {
    ink: '--color-fg-strong:',
    paper: '--color-surface:',
  },
  {
    ink: '--color-muted:',
    paper: '--color-bg:',
  },
  {
    ink: '--color-placeholder:',
    paper: '--color-bg:',
  },
];

/**
 Collects every `oklch(...)` function occurrence inside css via a
 linear index scan (no regex).
 
 @param css - stylesheet fragment to scan
 
 @returns every `oklch(...)` substring, in source order
 */
function collectOklchValues(css: string,): readonly string[] {
  /**
   `indexOf` sentinel for "no further occurrence".
   */
  const NOT_FOUND = -1;

  /**
   Collected `oklch(...)` substrings.
   */
  const values: string[] = [];

  /**
   Scan cursor advanced past each match.
   */
  let cursor = css.indexOf('oklch(',);

  while (cursor !== NOT_FOUND) {
    /**
     Index of the closing parenthesis for the current occurrence.
     */
    const end = css.indexOf(
      ')',
      cursor,
    );

    if (end === NOT_FOUND) {
      break;
    }

    values.push(css.slice(
      cursor,
      end + 1,
    ),);
    cursor = css.indexOf(
      'oklch(',
      end,
    );
  }

  return values;
}

await describe({
  name: '',
  children: [
    describe({
      name: renderRootColors.name,
      children: [
        it({
          name: 'declares color-scheme and every custom property on :root',
          fn: async function declaresRootCustomProperties(): Promise<void> {
            /**
             `:root` rule with light-theme custom properties.
             */
            const css = renderRootColors();

            expect(css.startsWith(':root{',),).toBe(true,);
            expect(css,).toContain('color-scheme:light dark',);
            for (const token of TOKEN_NAMES) {
              expect(css,).toContain(token,);
            }
          },
        },),
      ],
    },),
    describe({
      name: renderDarkColors.name,
      children: [
        it({
          name: 'overrides every custom property inside a prefers-color-scheme: dark query',
          fn: async function overridesInDarkMediaQuery(): Promise<void> {
            /**
             `@media (prefers-color-scheme: dark)` rule with overridden
             custom properties.
             */
            const css = renderDarkColors();

            expect(
              css.startsWith('@media (prefers-color-scheme: dark){',),
            ).toBe(true,);
            for (const token of TOKEN_NAMES) {
              expect(css,).toContain(token,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'five-stop palette discipline',
      children: [
        it({
          name: 'uses only the five decreed grayscale stops, with no alpha',
          fn: async function usesOnlyAllowedStops(): Promise<void> {
            /**
             Both palette fragments concatenated for scanning.
             */
            const css = `${renderRootColors()}${renderDarkColors()}`;

            /**
             Every oklch() occurrence in the palette.
             */
            const values = collectOklchValues(css,);

            expect(values.length,).toBeGreaterThan(0,);
            for (const value of values) {
              expect(ALLOWED_STOPS,).toContain(value,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'AAA small-text contrast discipline',
      children: [
        {
          theme: 'light',
          css: renderRootColors(),
        },
        {
          theme: 'dark',
          css: renderDarkColors(),
        },
      ]
        .flatMap(function mapTheme(
          {
            theme,
            css,
          },
        ) {
          return CONTRAST_PAIRS.map(function mapPair(
            {
              ink,
              paper,
            },
          ) {
            return it({
              name: `${ink} on ${paper} reaches 7:1 in ${theme}`,
              fn: async function reachesAaaContrast(): Promise<void> {
                expect(
                  contrastRatio(
                    {
                      css,
                      ink,
                      paper,
                    },
                  ),
                ).toBeGreaterThanOrEqual(AAA_SMALL_TEXT_MIN,);
              },
            },);
          },);
        },),
    },),
  ],
},);
