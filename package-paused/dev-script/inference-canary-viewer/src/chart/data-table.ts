/**
 * Accessible backing `<table>` for each scatter chart.
 *
 * Provides the same data as the visual chart in a machine-readable,
 * screen-reader-navigable format.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { renderDataGrid, } from './data-grid.ts';

/**
 * Single data point for the table
 */
export type TableRow = {
  readonly timestamp: string;
  readonly model: string;
  readonly probe: string;
  readonly score: number;
  readonly pass2Score?: number;
  readonly failed: boolean;
  /**
   * Run ID for linking to the detail overlay (used in grid mode)
   */
  readonly runId?: string;
};

/**
 * Controls which columns are visible in the data table.
 * Columns default to visible when the option is omitted.
 */
export type TableDisplayOptions = {
  /**
   * Whether to show the Model column (hide when context is per-model)
   */
  readonly showModel?: boolean;
  /**
   * Whether to show the Probe column (hide when context is per-probe)
   */
  readonly showProbe?: boolean;
};

/**
 * Produces the fix-score `<td>` cell for a table row.
 *
 * Returns the rendered score when present, a "(not run)" label for failed runs
 * that never attempted the fix pass, or a "(data error)" label for missing data.
 * Returns an empty string when the table has no fix-score column at all.
 *
 * @param hasFixScores - whether the table includes a fix-score column
 *
 * @param row - data row to render
 *
 * @returns HTML string for the cell, or empty string when column is hidden
 *
 * @example
 * ```ts
 * fixScoreCell({ hasFixScores: true, row: { timestamp: '...', model: 'Sonnet 4.6', probe: 'csv', score: 0.8, pass2Score: 0.95, failed: false, }, });
 * // '<td>0.95<\/td>'
 * fixScoreCell({ hasFixScores: false, row, }); // ''
 * ```
 */
function fixScoreCell({
  hasFixScores,
  row,
}: {
  readonly hasFixScores: boolean;
  readonly row: TableRow;
},): string {
  if (!hasFixScores)
    return '';
  if (row.pass2Score
    !== undefined) {
    return h({
      tag: 'td',
      text: row.pass2Score
        .toFixed(2,),
    },);
  }
  if (row.failed) {
    return h({
      tag: 'td',
      children: [
        h({
          tag: 'span',
          class: 'missing-data-label',
          text: '(not run)',
        },),
      ],
    },);
  }
  return h({
    tag: 'td',
    children: [
      h({
        tag: 'span',
        class: 'missing-data-label',
        text: '(data error)',
      },),
    ],
  },);
}

/**
 * Renders a `<table>` element containing chart data.
 * Hidden visually but accessible to screen readers and present
 * when CSS fails to load.
 *
 * Status is shown inline: "(timeout)" is appended to the timestamp cell
 * for failed runs instead of occupying its own column.
 * Missing fix scores distinguish between failed runs ("not run": fix was
 * never attempted) and genuine data errors ("data error").
 *
 * @param rows - data rows to render
 *
 * @param caption - table caption describing the chart
 *
 * @param options - column visibility overrides
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = renderDataTable({
 *   rows,
 *   caption: 'Score history',
 *   options: { showModel: true, showProbe: false, },
 * });
 * // '<table class="chart-data-table">...<\/table>'
 * ```
 */
export function renderDataTable({
  rows,
  caption,
  options = {},
}: {
  readonly rows: readonly TableRow[];
  readonly caption: string;
  readonly options?: TableDisplayOptions;
},): string {
  /**
   * Resolved Model-column visibility; defaults to visible when option is omitted.
   */
  const showModel = options.showModel
    ?? true;
  /**
   * Resolved Probe-column visibility; defaults to visible when option is omitted.
   */
  const showProbe = options.showProbe
    ?? true;

  /**
   * When only timestamp + score remain, render as a compact grid instead of a table
   */
  if ((!showModel) && (!showProbe)) {
    return renderDataGrid({
      rows,
      caption,
    },);
  }

  /**
   * Only show the fix score column when at least one row has pass2 data
   */
  const hasFixScores = rows.some(function hasPass2(row,) {
    return row.pass2Score
      !== undefined;
  },);

  /**
   * Table header row built once and reused; column set depends on resolved visibility flags.
   */
  const headerRow = h({
    tag: 'tr',
    children: [
      h({
        tag: 'th',
        attrs: { scope: 'col', },
        text: 'Timestamp',
      },),
      ...(showModel
        ? [h({
          tag: 'th',
          attrs: { scope: 'col', },
          text: 'Model',
        },),]
        : []),
      ...(showProbe
        ? [h({
          tag: 'th',
          attrs: { scope: 'col', },
          text: 'Probe',
        },),]
        : []),
      h({
        tag: 'th',
        attrs: { scope: 'col', },
        text: 'Score',
      },),
      ...(hasFixScores
        ? [h({
          tag: 'th',
          attrs: { scope: 'col', },
          text: 'Fix score',
        },),]
        : []),
    ],
  },);

  /**
   * Rendered `<tr>` strings joined into the table body; each row mirrors the visibility flags from the header.
   */
  const bodyRows = rows
    .map(function renderRow(row,) {
      /**
       * Timestamp cell with inline "(timeout)" for failed runs
       */
      const timestampTd = row.failed
        ? h({
          tag: 'td',
          children: [
            h({
              tag: 'span',
              text: row.timestamp,
            },),
            ' ',
            h({
              tag: 'span',
              class: 'run-status',
              attrs: { 'data-level': 'failed', },
              text: '(timeout)',
            },),
          ],
        },)
        : h({
          tag: 'td',
          text: row.timestamp,
        },);

      /**
       * Fix score cell: present, "(not run)" for failed, or "(data error)"
       */
      const fixScoreTd = fixScoreCell({
        hasFixScores,
        row,
      },);

      return h({
        tag: 'tr',
        ...(row.failed
          ? {
            class: 'run-status',
            attrs: { 'data-level': 'failed', },
          }
          : {}),
        children: [
          timestampTd,
          ...(showModel
            ? [h({
              tag: 'td',
              text: row.model,
            },),]
            : []),
          ...(showProbe
            ? [h({
              tag: 'td',
              text: row.probe,
            },),]
            : []),
          h({
            tag: 'td',
            text: row.score
              .toFixed(2,),
          },),
          fixScoreTd,
        ]
          .filter(Boolean,),
      },);
    },)
    .join('\n',);

  return h({
    tag: 'table',
    class: 'chart-data-table',
    children: [
      h({
        tag: 'caption',
        text: caption,
      },),
      h({
        tag: 'thead',
        children: [headerRow,],
      },),
      h({
        tag: 'tbody',
        html: bodyRows,
      },),
    ],
  },);
}
