/**
 * Side-by-side diff view renderer.
 *
 * Renders a two-column diff with syntax highlighting where the left column
 * shows the initial pass (removed lines highlighted) and the right column
 * shows the fix pass (added lines highlighted).
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { DiffLine, } from '../data/diff.ts';

/**
 * Renders a side-by-side diff view with syntax highlighting.
 *
 * Left column: initial pass (removed lines highlighted).
 * Right column: fix pass (added lines highlighted).
 * Unchanged lines appear in both columns.
 *
 * @param diffLines - computed diff lines
 *
 * @returns HTML string for the diff view
 *
 * @example
 * ```ts
 * const html = renderSideBySideDiff([{ type: 'removed', content: 'old' }, { type: 'added', content: 'new' }]);
 * // '<div class="diff-container">...<\/div>'
 * ```
 */
export function renderSideBySideDiff(
  diffLines: readonly DiffLine[],
): string {
  /**
   * Accumulator for the initial-pass column's line spans.
   */
  const leftLines: string[] = [];
  /**
   * Accumulator for the fix-pass column's line spans.
   */
  const rightLines: string[] = [];

  for (const line of diffLines) {
    if (line.type
      === 'removed') {
      leftLines.push(h({
        tag: 'span',
        class: 'diff-removed',
        text: line.content,
      },),);
      rightLines.push(h({
        tag: 'span',
        class: 'diff-spacer',
      },),);
    }
    else if (line.type
      === 'added') {
      leftLines.push(h({
        tag: 'span',
        class: 'diff-spacer',
      },),);
      rightLines.push(h({
        tag: 'span',
        class: 'diff-added',
        text: line.content,
      },),);
    }
    else {
      leftLines.push(h({
        tag: 'span',
        text: line.content,
      },),);
      rightLines.push(h({
        tag: 'span',
        text: line.content,
      },),);
    }
  }

  return h({
    tag: 'div',
    class: 'diff-container',
    children: [
      h({
        tag: 'div',
        class: 'diff-column',
        children: [
          h({
            tag: 'h3',
            text: 'Initial pass',
          },),
          h({
            tag: 'pre',
            class: 'glow diff-pre',
            children: [h({
              tag: 'code',
              html: leftLines.join('\n',),
            },),],
          },),
        ],
      },),
      h({
        tag: 'div',
        class: 'diff-column',
        children: [
          h({
            tag: 'h3',
            text: 'Fix pass',
          },),
          h({
            tag: 'pre',
            class: 'glow diff-pre',
            children: [h({
              tag: 'code',
              html: rightLines.join('\n',),
            },),],
          },),
        ],
      },),
    ],
  },);
}
