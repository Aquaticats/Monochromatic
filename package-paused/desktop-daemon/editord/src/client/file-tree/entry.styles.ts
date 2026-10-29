/**
 * Entry-level styles for the file tree: summaries, file entries,
 * toggle badges, marker arrows, and interaction states.
 *
 * Imported by `file-tree.styles.ts` and concatenated into the
 * file tree's shadow stylesheet.
 */

import {
  cssCommaList,
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Vertical padding for entry labels and summaries in `rem`: 0.125 = 1 / (2 * 2 * 2).
 */
const ENTRY_PADDING_BLOCK = 1 / (2 * 2
  * 2);

/**
 * Horizontal padding for entry labels and summaries in `rem`: 0.5 = 1 / 2.
 */
const ENTRY_PADDING_INLINE = 1 / 2;

/**
 * Entry-level styles for summaries, file entries, and interaction states.
 */
export const ENTRY_STYLES: string = [
  $({
    rule: 'tree-dir-entry',
    decls: { display: 'contents', },
  },),
  $({
    rule: 'summary',
    decls: {
      display: 'flex',
      'align-items': 'center',
      'padding-block': cssRem(ENTRY_PADDING_BLOCK,),
      'padding-inline': cssRem(ENTRY_PADDING_INLINE,),
      cursor: 'pointer',
      'white-space': 'nowrap',
      'list-style': 'none',
    },
  },),
  $({
    rule: 'summary::-webkit-details-marker',
    decls: { display: 'none', },
  },),
  $({
    rule: 'summary::marker',
    decls: { display: 'none', },
  },),
  $({
    rule: 'summary:hover, tree-file-entry:hover',
    decls: { 'background-color': cssVar('tree-hover-bg',), },
  },),
  $({
    rule: 'summary:focus-visible, tree-file-entry:focus-visible',
    decls: {
      'background-color': cssVar('tree-selected-bg',),
      'outline-style': 'dashed',
      'outline-width': cssRem(1 / (2 * 2
        * 2),),
      'outline-color': cssVar('fg',),
      'outline-offset': cssRem(-(1 / (2 * 2
        * 2)),),
    },
  },),
  $({
    rule: 'summary::after',
    decls: {
      content: `'\\25B8'`,
      'inline-size': cssRem(1,),
      'flex-shrink': cssNum(0,),
      'text-align': 'center',
    },
  },),
  $({
    rule: 'details[open] > summary::after',
    decls: { content: `'\\25BE'`, },
  },),
  $({
    rule: '.toggle',
    decls: {
      'inline-size': cssRem(1,),
      'flex-shrink': cssNum(0,),
      'text-align': 'center',
    },
  },),
  $({
    rule: 'tree-file-entry',
    decls: {
      display: 'flex',
      'align-items': 'center',
      'padding-block': cssRem(ENTRY_PADDING_BLOCK,),
      'padding-inline': cssRem(ENTRY_PADDING_INLINE,),
      cursor: 'pointer',
      'white-space': 'nowrap',
    },
  },),
  $({
    rule: 'tree-file-entry[data-recency] .toggle',
    decls: {
      'font-family': cssCommaList([
        "'Inter'",
        'sans-serif',
      ],),
      opacity: cssNum(1 / 2,),
    },
  },),
]
  .join('',);
