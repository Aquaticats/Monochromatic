/**
 * Dark and light theme custom property definitions.
 *
 * Declares all CSS custom properties on `:root` (dark default)
 * and `:root[data-theme="light"]` (light override).
 * Built at compile time into `dist/client/global.css`.
 */

import { hCss as $, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Dark theme (default) custom property values on `:root`.
 */
const DARK = $({
  rule: ':root',
  decls: {
    '--fg': '#ccc',
    '--bg': '#000',
    '--gutter-fg': '#555',
    '--tree-hover-bg': '#1a1a1a',
    '--tree-selected-bg': '#2a2a2a',
    '--editor-padding': '0.5rem',
    '--hl-keyword': '#c678dd',
    '--hl-string': '#98c379',
    '--hl-comment': '#888888',
    '--hl-number': '#d19a66',
    '--hl-type': '#e5c07b',
    '--hl-function': '#61afef',
    '--hl-property': '#e06c75',
    '--hl-heading': '#e5c07b',
    '--hl-link': '#61afef',
    '--hl-emphasis': '#d19a66',
    '--search-match-bg': 'oklch(0.55 0.15 70 / 0.4)',
    '--diag-error': '#f44747',
    '--diag-warning': '#ff8800',
    '--diag-info': '#4fc1ff',
    '--diag-hint': '#666',
    '--hover-bg': '#1e1e1e',
    '--hover-border': '#454545',
    '--accent': '#61afef',
    '--inlay-fg': '#8b949e',
    '--inlay-bg': 'rgba(110, 118, 129, 0.15)',
    '--inlay-bg-error': 'rgba(244, 71, 71, 0.15)',
    '--inlay-bg-warning': 'rgba(255, 136, 0, 0.15)',
  },
},);

/**
 * Light theme override custom property values.
 */
const LIGHT = $({
  rule: ':root[data-theme="light"]',
  decls: {
    '--fg': '#444',
    '--bg': '#fff',
    '--gutter-fg': '#aaa',
    '--tree-hover-bg': '#f0f0f0',
    '--tree-selected-bg': '#e0e0e0',
    '--hl-keyword': '#a626a4',
    '--hl-string': '#50a14f',
    '--hl-comment': '#a0a1a7',
    '--hl-number': '#986801',
    '--hl-type': '#c18401',
    '--hl-function': '#4078f2',
    '--hl-property': '#e45649',
    '--hl-heading': '#c18401',
    '--hl-link': '#4078f2',
    '--hl-emphasis': '#986801',
    '--search-match-bg': 'oklch(0.8 0.15 80 / 0.35)',
    '--diag-error': '#d32f2f',
    '--diag-warning': '#f57c00',
    '--diag-info': '#1976d2',
    '--diag-hint': '#999',
    '--hover-bg': '#f5f5f5',
    '--hover-border': '#ddd',
    '--accent': '#4078f2',
    '--inlay-fg': '#6e7781',
    '--inlay-bg': 'rgba(110, 118, 129, 0.1)',
    '--inlay-bg-error': 'rgba(211, 47, 47, 0.1)',
    '--inlay-bg-warning': 'rgba(245, 124, 0, 0.1)',
  },
},);

/**
 * Combined dark and light theme custom property rules.
 */
export const STYLES: string = DARK + LIGHT;
