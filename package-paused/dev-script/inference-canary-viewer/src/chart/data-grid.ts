/**
 * Compact card grid renderer for chart data.
 *
 * Used when both Model and Probe columns are hidden, leaving only
 * timestamp and score per row. Cards display timestamp on top, score below,
 * matching the probe grid layout in run detail overlays.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { TableRow, } from './data-table.ts';

/**
 * Renders a compact card grid for timestamp + score data.
 *
 * @param rows - data rows to render
 *
 * @param caption - accessible caption
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = renderDataGrid({ rows, caption: 'Claude overall score', });
 * // '<div class="score-card-grid" role="list" aria-label="...">...<\/div>'
 * ```
 */
export function renderDataGrid({
  rows,
  caption,
}: {
  readonly rows: readonly TableRow[];
  readonly caption: string;
},): string {
  /**
   * Show fix scores when at least one row has pass-2 data
   */
  const hasFixScores = rows.some(function hasPass2(row,) {
    return row.pass2Score
      !== undefined;
  },);

  /**
   * Rendered card markup strings, one per row, joined into the grid container's innerHTML.
   */
  const cards = rows
    .map(function renderCard(row,) {
      /**
       * Score rendered to 2 decimal places for display alongside the timestamp.
       */
      const score = row.score
        .toFixed(2,);

      /**
       * Timestamp line with optional "(timeout)" suffix
       */
      const timestampChildren: string[] = [h({
        tag: 'span',
        text: row.timestamp,
      },),];
      if (row.failed) {
        timestampChildren.push(
          ' ',
          h({
            tag: 'span',
            class: 'run-status',
            attrs: { 'data-level': 'failed', },
            text: '(timeout)',
          },),
        );
      }

      /**
       * Score line with optional fix suffix
       */
      const scoreChildren: string[] = [h({
        tag: 'strong',
        text: score,
      },),];
      if (hasFixScores) {
        if (row.pass2Score
          !== undefined)
          scoreChildren.push(` (fix: ${row.pass2Score
            .toFixed(2,)})`,);
        else if (row.failed) {
          scoreChildren.push(
            ' ',
            h({
              tag: 'span',
              class: 'score-warning',
              text: '(fix: not run)',
            },),
          );
        }
        else {
          scoreChildren.push(
            ' ',
            h({
              tag: 'span',
              class: 'score-warning',
              text: '(fix: no data)',
            },),
          );
        }
      }

      /**
       * Card element tag: `button` when a run-detail overlay can be opened, plain `div` otherwise.
       */
      const tag = row.runId
        !== undefined ? 'button' : 'div';
      /**
       * Mutable attribute bag populated below with overlay-target and failed-state markers as needed.
       */
      const attrs: Record<string, string> = { role: 'listitem', };
      if (row.runId
        !== undefined)
        attrs.popovertarget = `run-${row.runId}`;
      if (row.failed)
        attrs['data-failed'] = '';

      return h({
        tag,
        class: 'score-card',
        attrs,
        children: [
          h({
            tag: 'span',
            children: timestampChildren,
          },),
          h({
            tag: 'span',
            children: scoreChildren,
          },),
        ],
      },);
    },)
    .join('\n',);

  return h({
    tag: 'div',
    class: 'score-card-grid',
    attrs: {
      role: 'list',
      'aria-label': caption,
    },
    html: cards,
  },);
}
