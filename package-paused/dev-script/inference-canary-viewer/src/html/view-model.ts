/**
 * Per-model view: scatter charts showing probe scores over time.
 *
 * Each model gets a nested `<details>` element containing one scatter chart
 * per probe, plus an aggregate chart of overall score.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { renderScatterChart, } from '../chart/scatter.ts';
import { vendorColor, } from '../data/model-colors.ts';
import { iconDot, } from '../data/model-icons.ts';

import type { ViewerEntry, } from '../data/viewer-types.ts';

import {
  buildOverallPoints,
  buildProbePoints,
} from './view-model-points.ts';

/**
 * Renders the by-model view: one `<details>` per model, each containing scatter charts.
 *
 * @param entries - all viewer entries
 *
 * @param thresholds - map from model label to computed threshold
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = renderByModel({ entries, thresholds });
 * // '<details class="model-section">...<\/details>...'
 * ```
 */
export function renderByModel({
  entries,
  thresholds,
}: {
  readonly entries: readonly ViewerEntry[];
  readonly thresholds: ReadonlyMap<string, number>;
},): string {
  /**
   * Unique model labels in the order they first appear
   */
  const labels = [...new Set(entries.map(function getLabel(entry,): string {
    return entry.label;
  },),),];

  if (labels.length
    === 0) {
    return h({
      tag: 'p',
      text: 'No model data available.',
    },);
  }

  /**
   * Hide Model and Probe columns since we're within a per-model overall context
   */
  const tableDisplay = {
    showModel: false,
    showProbe: false,
  };

  return labels
    .map(function renderModelSection(label,): string {
      /**
       * Entries narrowed to the current model label.
       */
      const modelEntries = entries.filter(function matchLabel(entry,): boolean {
        return entry.label
          === label;
      },);
      /**
       * First entry for this label; always present since `label` came from these entries.
       */
      const [firstEntry,] = modelEntries;
      if (firstEntry === undefined)
        throw new Error(`no entries for model label: ${label}`,);
      /**
       * OpenRouter model ID from the first entry, used for vendor color/icon
       */
      const openrouterId = firstEntry.model;
      /**
       * Vendor-derived accent color reused across this model's charts.
       */
      const color = vendorColor(openrouterId,);
      /**
       * Degradation threshold for this model; absent when none was computed.
       */
      const thresholdValue = thresholds.get(label,);

      // Overall score chart
      /**
       * Scatter points feeding the overall-score chart for this model.
       */
      const overallPoints = buildOverallPoints({
        entries: modelEntries,
        label,
        openrouterId,
        color,
      },);
      /**
       * Rendered overall-score chart markup for this model section.
       */
      const overallChart = renderScatterChart({
        points: overallPoints,
        ...(thresholdValue !== undefined
          ? {
            threshold: {
              value: thresholdValue,
              label: `threshold: ${thresholdValue.toFixed(2,)}`,
            },
          }
          : {}),
        caption: `${label} overall score`,
        tableDisplay,
      },);

      // Per-probe charts
      /**
       * Distinct probe slugs scored by this model across its runs.
       */
      const probeNames = [
        ...new Set(modelEntries.flatMap(function probeKeys(entry,): string[] {
          return Object.keys(entry.probeScores,);
        },),),
      ];
      /**
       * Joined per-probe scatter chart markup for this model's breakdown.
       */
      const probeCharts = probeNames
        .map(function renderProbeChart(probe,): string {
          /**
           * Scatter points feeding this probe's per-model chart.
           */
          const probePoints = buildProbePoints({
            entries: modelEntries,
            label,
            openrouterId,
            probe,
            color,
          },);
          return h({
            tag: 'details',
            class: 'probe-section',
            children: [
              h({
                tag: 'summary',
                text: probe,
              },),
              renderScatterChart({
                points: probePoints,
                caption: `${label} - ${probe}`,
                tableDisplay: {
                  showModel: false,
                  showProbe: false,
                },
              },),
            ],
          },);
        },)
        .join('\n',);

      return h({
        tag: 'details',
        class: 'model-section',
        children: [
          h({
            tag: 'summary',
            children: [
              iconDot({
                modelId: openrouterId,
                color,
              },),
              ' ',
              h({
                tag: 'span',
                text: label,
              },),
            ],
          },),
          h({
            tag: 'div',
            class: 'pane',
            children: [
              overallChart,
              h({
                tag: 'h3',
                text: 'Per-probe breakdown',
              },),
              probeCharts,
            ],
          },),
        ],
      },);
    },)
    .join('\n',);
}
