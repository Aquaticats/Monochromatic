/**
 Dark mode overrides for design tokens.
 
 Swaps foreground/background primitive color references
 when the user prefers a dark color scheme.
 */
import { $ as css, } from './css.ts';

/**
 Dark mode overrides: swaps foreground/background primitives.
 */
export const darkMode: string = css({
  at: 'media',
  params: '(prefers-color-scheme: dark)',
  children: [
    css({
      rule: ':root',
      decls: {
        '--fg': 'var(--gray-light)',
        '--fg-50': 'var(--gray-light-50)',
        '--fg-stronger': 'var(--gray-light-lighter)',
        '--fg-weaker': 'var(--gray-light-darker)',
        '--bg': 'var(--gray-dark)',
        '--bg-50': 'var(--gray-dark-50)',
        '--bg-stronger': 'var(--gray-dark-darker)',
        '--bg-weaker': 'var(--gray-dark-lighter)',
        '--red-fg': 'var(--red-light)',
        '--red-bg': 'var(--red-dark)',
        '--purple-fg': 'var(--purple-light)',
        '--purple-bg': 'var(--purple-dark)',
        '--yellow-fg': 'var(--yellow-light)',
        '--yellow-bg': 'var(--yellow-dark)',
        '--cyan-fg': 'var(--cyan-light)',
        '--cyan-bg': 'var(--cyan-dark)',
        '--blue-fg': 'var(--blue-light)',
        '--blue-bg': 'var(--blue-dark)',
        '--green-fg': 'var(--green-light)',
        '--green-bg': 'var(--green-dark)',
        '--orange-fg': 'var(--orange-light)',
        '--orange-bg': 'var(--orange-dark)',
        '--overlay-bg': 'rgb(255 255 255 / 0.15)',
        '--hover-bg': 'rgb(255 255 255 / 0.08)',
      },
    },),
  ],
},);
