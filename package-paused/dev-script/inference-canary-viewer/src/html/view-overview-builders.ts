/**
 * Scatter point and legend builders for the overview section.
 *
 * Converts all-model entries into combined scatter points and generates
 * the color legend mapping vendor colors to model labels.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { ScatterPoint, } from '../chart/scatter.ts';
import { vendorColor, } from '../data/model-colors.ts';
import {
  iconDot,
  vendorIconEntry,
} from '../data/model-icons.ts';

import {
  hasMultipleProbes,
  type ViewerEntry,
} from '../data/viewer-types.ts';

import type { ModelSummary, } from './view-overview-types.ts';

/**
 * Builds scatter points for all models' overall scores, ordered by timestamp.
 *
 * @param entries - all history entries
 *
 * @returns scatter points array
 *
 * @example
 * ```ts
 * const points = buildAllModelPoints(entries);
 * // [{ runId: 'Sonnet 4.6-2026-03-06...', index: 0, score: 0.85, ... }]
 * ```
 */
export function buildAllModelPoints(
  entries: readonly ViewerEntry[],
): readonly ScatterPoint[] {
  return entries
    .filter(function hasScore(entry,): boolean {
      return (entry.overallScore
        > 0) && hasMultipleProbes(entry,);
    },)
    .map(function toPoint(
      entry,
      index,
    ): ScatterPoint {
      /**
       * Vendor-derived accent color for this point's button.
       */
      const color = vendorColor(entry.model,);
      /**
       * Stable id linking the point button to its detail overlay.
       */
      const runId = `${entry.label}-${entry.timestamp}`;
      /**
       * Source row used by the chart's optional data-table caption.
       */
      const tableRow = {
        timestamp: entry.timestamp,
        model: entry.label,
        probe: 'overall',
        score: entry.overallScore,
        failed: entry.failed,
      };
      return {
        runId,
        index,
        timestamp: entry.timestamp,
        score: entry.overallScore,
        color,
        ...vendorIconEntry(entry.model,),
        title: `${entry.label} ${
          entry.timestamp
            .slice(
            0,
            10,
          )
        }: ${entry.overallScore
          .toFixed(2,)}`,
        failed: entry.failed,
        tableRow,
      };
    },);
}

/**
 * Renders a color legend for the overview chart.
 *
 * @param summaries - model summaries
 *
 * @returns HTML legend string
 *
 * @example
 * ```ts
 * const html = buildOverviewLegend(summaries);
 * // '<div class="chart-legend">...<\/div>'
 * ```
 */
export function buildOverviewLegend(summaries: readonly ModelSummary[],): string {
  /**
   * Joined legend item markup feeding the legend container body.
   */
  const items = summaries
    .map(function buildLegendItem(summary,) {
      /**
       * Vendor-derived accent color used by the inline dot icon.
       */
      const color = vendorColor(summary.model,);
      return h({
        tag: 'span',
        class: 'item',
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
      },);
    },)
    .join('\n',);

  return h({
    tag: 'div',
    class: 'chart-legend',
    html: items,
  },);
}
