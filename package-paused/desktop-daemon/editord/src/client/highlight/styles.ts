/**
 * CSS rules for syntax highlighting and diagnostic underlines.
 *
 * Uses the CSS Custom Highlight API `::highlight()` pseudo-elements
 * for both Lezer token groups and LSP diagnostic severity levels.
 */

import {
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 * CSS rules for syntax highlighting and diagnostic highlights.
 */
export const HIGHLIGHT_STYLES: string = [
  $({
    rule: '::highlight(hl-keyword)',
    decls: { color: cssVar('hl-keyword',), },
  },),
  $({
    rule: '::highlight(hl-string)',
    decls: { color: cssVar('hl-string',), },
  },),
  $({
    rule: '::highlight(hl-comment)',
    decls: { color: cssVar('hl-comment',), },
  },),
  $({
    rule: '::highlight(hl-number)',
    decls: { color: cssVar('hl-number',), },
  },),
  $({
    rule: '::highlight(hl-type)',
    decls: { color: cssVar('hl-type',), },
  },),
  $({
    rule: '::highlight(hl-function)',
    decls: { color: cssVar('hl-function',), },
  },),
  $({
    rule: '::highlight(hl-property)',
    decls: { color: cssVar('hl-property',), },
  },),
  $({
    rule: '::highlight(hl-heading)',
    decls: { color: cssVar('hl-heading',), },
  },),
  $({
    rule: '::highlight(hl-link)',
    decls: {
      color: cssVar('hl-link',),
      'text-decoration': 'underline',
    },
  },),
  $({
    rule: '::highlight(hl-emphasis)',
    decls: { color: cssVar('hl-emphasis',), },
  },),
  $({
    rule: '::highlight(diag-error)',
    decls: {
      'text-decoration-line': 'underline',
      'text-decoration-style': 'wavy',
      'text-decoration-color': cssVar('diag-error',),
      'text-decoration-skip-ink': 'none',
    },
  },),
  $({
    rule: '::highlight(diag-warning)',
    decls: {
      'text-decoration-line': 'underline',
      'text-decoration-style': 'wavy',
      'text-decoration-color': cssVar('diag-warning',),
      'text-decoration-skip-ink': 'none',
    },
  },),
  $({
    rule: '::highlight(diag-info)',
    decls: {
      'text-decoration-line': 'underline',
      'text-decoration-style': 'wavy',
      'text-decoration-color': cssVar('diag-info',),
      'text-decoration-skip-ink': 'none',
    },
  },),
  $({
    rule: '::highlight(diag-hint)',
    decls: {
      'text-decoration-line': 'underline',
      'text-decoration-style': 'wavy',
      'text-decoration-color': cssVar('diag-hint',),
      'text-decoration-skip-ink': 'none',
    },
  },),
]
  .join('',);
