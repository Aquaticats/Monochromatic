/**
 * Overview section: all-models scatter chart and summary table.
 *
 * Shows a combined scatter chart with every model's overall score over time,
 * where each point is clickable to open the run detail overlay.
 * Below the chart, a summary table lists each model's latest status.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { SHAPE_LEGEND, } from '../chart/legend.ts';
import { renderScatterChart, } from '../chart/scatter.ts';
import { vendorColor, } from '../data/model-colors.ts';
import { iconDot, } from '../data/model-icons.ts';

import type { ViewerEntry, } from '../data/viewer-types.ts';

import {
  buildAllModelPoints,
  buildOverviewLegend,
} from './view-overview-builders.ts';
import {
  type ModelSummary,
  statusLevel,
} from './view-overview-types.ts';

export type { ModelSummary, } from './view-overview-types.ts';

/**
 * Renders the overview section with a scatter chart of all models and a summary table.
 *
 * @param summaries - per-model summaries sorted by model name
 *
 * @param entries - all history entries (for the combined chart)
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = renderOverview({ summaries, entries });
 * // '<div class="chart-legend">...<\/div>...<table class="overview-table">...<\/table>'
 * ```
 */
export function renderOverview({
  summaries,
  entries,
}: {
  readonly summaries: readonly ModelSummary[];
  readonly entries: readonly ViewerEntry[];
},): string {
  if (summaries.length
    === 0) {
    return h({
      tag: 'p',
      text: 'No history data available. Run the canary first.',
    },);
  }

  // Combined scatter chart: all models' overall scores
  /**
   * Scatter-plot data points feeding the combined overview chart.
   */
  const chartPoints = buildAllModelPoints(entries,);
  /**
   * Color/shape legend rendered above the chart.
   */
  const legend = buildOverviewLegend(summaries,);
  /**
   * Composed scatter chart markup for the all-models view.
   */
  const chart = renderScatterChart({
    points: chartPoints,
    caption: 'All models overall score',
    hideTable: true,
  },);

  // Summary table: status is shown inline rather than in its own column
  /**
   * Joined `<tr>` markup populating the summary table body.
   */
  const rows = summaries
    .map(function buildRow(summary,): string {
      /**
       * Vendor-derived accent color for the row's model icon.
       */
      const color = vendorColor(summary.model,);
      /**
       * Row-level health status driving the optional `data-level` styling attribute.
       */
      const statusClass = statusLevel(summary,);

      /**
       * Inline status badge appended to the score cell when applicable.
       */
      const inlineStatus = summary.failed
        ? ` ${
          h({
            tag: 'span',
            class: 'run-status',
            attrs: { 'data-level': 'failed', },
            text: '(timeout)',
          },)
        }`
        : (summary.degraded
          ? ` ${
            h({
              tag: 'span',
              class: 'run-status',
              attrs: { 'data-level': 'degraded', },
              text: '(degraded)',
            },)
          }`
          : '');

      return h({
        tag: 'tr',
        ...(statusClass !== 'healthy'
          ? {
            class: 'run-status',
            attrs: { 'data-level': statusClass, },
          }
          : {}),
        children: [
          h({
            tag: 'td',
            children: [
              iconDot({
                modelId: summary.model,
                color,
              },),
              ' ',
              h({
                tag: 'span',
                text: summary.label,
              },),
            ],
          },),
          h({
            tag: 'td',
            html: summary.latestScore
              .toFixed(2,)
              + inlineStatus,
          },),
          h({
            tag: 'td',
            text: summary.latestTimestamp
              .slice(
              0,
              10,
            ),
          },),
          h({
            tag: 'td',
            text: String(summary.runCount,),
          },),
          h({
            tag: 'td',
            text: summary.threshold
              .toFixed(2,),
          },),
        ],
      },);
    },)
    .join('\n',);

  return [
    legend,
    SHAPE_LEGEND,
    chart,
    h({
      tag: 'h3',
      text: 'Summary',
    },),
    h({
      tag: 'table',
      class: 'overview-table',
      children: [
        h({
          tag: 'thead',
          children: [
            h({
              tag: 'tr',
              children: [
                h({
                  tag: 'th',
                  attrs: { scope: 'col', },
                  text: 'Model',
                },),
                h({
                  tag: 'th',
                  attrs: { scope: 'col', },
                  text: 'Score',
                },),
                h({
                  tag: 'th',
                  attrs: { scope: 'col', },
                  text: 'Last run',
                },),
                h({
                  tag: 'th',
                  attrs: { scope: 'col', },
                  text: 'Runs',
                },),
                h({
                  tag: 'th',
                  attrs: { scope: 'col', },
                  text: 'Threshold',
                },),
              ],
            },),
          ],
        },),
        h({
          tag: 'tbody',
          html: rows,
        },),
      ],
    },),
  ]
    .join('\n',);
}
