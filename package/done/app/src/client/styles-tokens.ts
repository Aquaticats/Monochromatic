/**
 Primitive and semantic color tokens plus dark-mode overrides.
 */
import { cssRem, } from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';

/**
 Minimum touch target size in rem.
 */
const MIN_TARGET = 3;

/**
 Minimum spacing padding in rem (1/2).
 */
const MIN_PADDING = 1 / 2;

//region Primitive color tokens (raw color values that never change between modes)

/**
 Raw color values that never change between light and dark modes.
 */
export const primitiveTokens: string = css({
  rule: ':root',
  decls: {
    '--gray-dark': '#111',
    '--gray-dark-50': 'rgb(17 17 17 / 0.5)',
    '--gray-dark-darker': '#000',
    '--gray-dark-lighter': '#444',
    '--gray-light': '#eee',
    '--gray-light-50': 'rgb(238 238 238 / 0.5)',
    '--gray-light-lighter': '#fff',
    '--gray-light-darker': '#bbb',
    '--gray-medium': '#888',
    '--red-dark': '#904649',
    '--red-light': '#faa',
    '--purple-dark': '#8d3d8c',
    '--purple-light': '#faf',
    '--yellow-dark': '#626000',
    '--yellow-light': '#ffa',
    '--cyan-dark': '#006769',
    '--cyan-light': '#aff',
    '--blue-dark': '#5754a1',
    '--blue-light': '#aaf',
    '--green-dark': '#106b1c',
    '--green-light': '#afa',
    '--orange-dark': '#90511e',
    '--orange-light': '#fca',
  },
},);

//endregion Primitive color tokens

//region Semantic color tokens (aliases that flip between light and dark modes)

/**
 Semantic aliases that map to primitives, plus spacing/sizing tokens.
 */
export const semanticTokens: string = css({
  rule: ':root',
  decls: {
    '--fg': 'var(--gray-dark)',
    '--fg-50': 'var(--gray-dark-50)',
    '--fg-stronger': 'var(--gray-dark-darker)',
    '--fg-weaker': 'var(--gray-dark-lighter)',
    '--bg': 'var(--gray-light)',
    '--bg-50': 'var(--gray-light-50)',
    '--bg-stronger': 'var(--gray-light-lighter)',
    '--bg-weaker': 'var(--gray-light-darker)',
    '--medium': 'var(--medium)',
    '--red-fg': 'var(--red-dark)',
    '--red-bg': 'var(--red-light)',
    '--purple-fg': 'var(--purple-dark)',
    '--purple-bg': 'var(--purple-light)',
    '--yellow-fg': 'var(--yellow-dark)',
    '--yellow-bg': 'var(--yellow-light)',
    '--cyan-fg': 'var(--cyan-dark)',
    '--cyan-bg': 'var(--cyan-light)',
    '--blue-fg': 'var(--blue-dark)',
    '--blue-bg': 'var(--blue-light)',
    '--green-fg': 'var(--green-dark)',
    '--green-bg': 'var(--green-light)',
    '--orange-fg': 'var(--orange-dark)',
    '--orange-bg': 'var(--orange-light)',
    '--overlay-bg': 'rgb(0 0 0 / 0.3)',
    '--hover-bg': 'rgb(0 0 0 / 0.05)',
    '--min-target': cssRem(MIN_TARGET,),
    '--min-gap': cssRem(1,),
    '--min-padding': cssRem(MIN_PADDING,),
    '--gap': cssRem(2,),
  },
},);

//endregion Semantic color tokens

//region Dark mode (re-exported from styles-tokens-dark.ts)

export { darkMode, } from './styles-tokens-dark.ts';

//endregion Dark mode
