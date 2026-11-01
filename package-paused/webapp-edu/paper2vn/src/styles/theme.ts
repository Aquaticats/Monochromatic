/**
 * Theme custom properties for paper2vn.
 *
 * Defines the design-token CSS variables on `:root` plus the dark-mode
 * overrides. All other style modules consume these via `cssVar(...)`.
 */
import {
  cssCommaList,
  cssCompounded,
  cssNum,
  cssOklch,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Theme custom property declarations.
 *
 * @returns ordered array of compiled rule strings
 *
 * @example
 * ```ts
 * const css = themeRules().join('\n');
 * ```
 */
export function themeRules(): string[] {
  return [
    $({
      rule: ':root',
      decls: {
        '--bg': cssOklch({
          l: 0.97,
          c: 0.01,
          h: 50,
        },),
        '--bg-elevated': cssOklch({
          l: 1,
          c: 0,
          h: 0,
        },),
        '--fg': cssOklch({
          l: 0.2,
          c: 0.02,
          h: 280,
        },),
        '--fg-muted': cssOklch({
          l: 0.45,
          c: 0.02,
          h: 280,
        },),
        '--accent': cssOklch({
          l: 0.6,
          c: 0.18,
          h: 350,
        },),
        '--accent-fg': cssOklch({
          l: 0.99,
          c: 0.01,
          h: 350,
        },),
        '--border': cssOklch({
          l: 0.85,
          c: 0.02,
          h: 50,
        },),
        '--shadow': cssOklch({
          l: 0.2,
          c: 0.02,
          h: 280,
          a: 0.16,
        },),
        '--bg-board': cssOklch({
          l: 0.32,
          c: 0.04,
          h: 165,
        },),
        '--font-scale': cssNum(1,),
        'color-scheme': cssCompounded([
          'light',
          'dark',
        ],),
        'font-family': cssCommaList([
          'system-ui',
          '-apple-system',
          '"Segoe UI"',
          'Roboto',
          'sans-serif',
        ],),
      },
    },),
    $({
      rule: '@media (prefers-color-scheme: dark)',
      children: [
        $({
          rule: ':root',
          decls: {
            '--bg': cssOklch({
              l: 0.16,
              c: 0.01,
              h: 280,
            },),
            '--bg-elevated': cssOklch({
              l: 0.22,
              c: 0.02,
              h: 280,
            },),
            '--fg': cssOklch({
              l: 0.95,
              c: 0.01,
              h: 50,
            },),
            '--fg-muted': cssOklch({
              l: 0.65,
              c: 0.02,
              h: 50,
            },),
            '--border': cssOklch({
              l: 0.32,
              c: 0.02,
              h: 280,
            },),
            '--shadow': cssOklch({
              l: 0,
              c: 0,
              h: 0,
              a: 0.45,
            },),
          },
        },),
      ],
    },),
  ];
}
