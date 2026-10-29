/**
 * HTML+CSS scatter plot renderer.
 *
 * Each data point is a positioned `<button>` element inside a `position: relative`
 * container. Points open run detail popovers via `popovertarget` attributes.
 * Pass-1 points are filled circles; pass-2 points are hollow circles overlaid
 * at the same X position.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import {
  renderXAxis,
  renderYAxis,
} from './axis.ts';
import {
  renderDataTable,
  type TableDisplayOptions,
  type TableRow,
} from './data-table.ts';
import { renderPointElements, } from './scatter-point.ts';
import { renderThresholdLine, } from './threshold-line.ts';

/**
 * Single data point for the scatter plot
 */
export type ScatterPoint = {
  /**
   * Unique identifier for the run (used in fragment link)
   */
  readonly runId: string;
  /**
   * 0-based index on the X axis
   */
  readonly index: number;
  /**
   * ISO timestamp for X axis label
   */
  readonly timestamp: string;
  /**
   * Score value (0-1) for Y axis
   */
  readonly score: number;
  /**
   * Optional pass-2 (fix) score overlaid at same X
   */
  readonly pass2Score?: number;
  /**
   * CSS color for this point
   */
  readonly color: string;
  /**
   * Tooltip text
   */
  readonly title: string;
  /**
   * Whether this run failed
   */
  readonly failed: boolean;
  /**
   * Raw SVG icon to render inside the chart point (when available)
   */
  readonly icon?: string;
  /**
   * Metadata for the backing table
   */
  readonly tableRow: TableRow;
};

/**
 * Renders a complete scatter chart: plot area, axes, threshold line, and backing table.
 *
 * @param points - data points to render
 *
 * @param threshold - degradation threshold value and label, omitted to draw no line
 *
 * @param caption - accessible caption for the chart and table
 *
 * @param hideTable - when true, omit the backing data table entirely
 *
 * @param tableDisplay - column visibility options forwarded to the data table
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = renderScatterChart({ points, threshold: { value: 0.75, label: 'threshold: 0.75' }, caption: 'Score' });
 * // '<figure class="chart-figure">...<\/figure>'
 * ```
 */
export function renderScatterChart({
  points,
  threshold,
  caption,
  hideTable,
  tableDisplay,
}: {
  readonly points: readonly ScatterPoint[];
  readonly threshold?: {
    readonly value: number;
    readonly label: string;
  };
  readonly caption: string;
  readonly hideTable?: boolean;
  readonly tableDisplay?: TableDisplayOptions;
},): string {
  if (points.length
    === 0) {
    return h({
      tag: 'p',
      class: 'chart-empty-state',
      text: 'No data available.',
    },);
  }

  /**
   * Positioned `<button>` markup for every point; rendered once and embedded inside the plot area.
   */
  const pointElements = renderPointElements(points,);

  /**
   * Ordered timestamps extracted for the X-axis label renderer.
   */
  const timestamps = points.map(function getTimestamp(point,) {
    return point.timestamp;
  },);

  /**
   * Backing data table HTML; suppressed entirely when the caller opts out via `hideTable`.
   */
  const tableHtml = hideTable === true
    ? ''
    : renderDataTable({
      rows: points.map(function getTableRow(point,) {
        return point.tableRow;
      },),
      caption,
      ...(tableDisplay !== undefined ? { options: tableDisplay, } : {}),
    },);

  return h({
    tag: 'figure',
    class: 'chart-figure',
    children: [
      h({
        tag: 'figcaption',
        text: caption,
      },),
      h({
        tag: 'div',
        class: 'chart-area',
        attrs: {
          role: 'img',
          'aria-label': caption,
        },
        children: [
          h({
            tag: 'div',
            class: 'chart-y-axis',
            html: renderYAxis(),
          },),
          h({
            tag: 'div',
            class: 'chart-plot',
            children: [
              renderThresholdLine(threshold,),
              pointElements,
            ],
          },),
          h({
            tag: 'div',
            class: 'chart-x-axis',
            html: renderXAxis(timestamps,),
          },),
        ],
      },),
      tableHtml,
    ],
  },);
}
