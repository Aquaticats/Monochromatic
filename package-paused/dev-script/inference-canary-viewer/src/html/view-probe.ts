/**
 * Per-probe view: compares all models on the same probe.
 *
 * Each probe gets a `<details>` section containing a combined scatter chart
 * where each model's points use its vendor color, plus a per-model breakdown
 * with individual charts.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { renderScatterChart, } from '../chart/scatter.ts';
import { vendorColor, } from '../data/model-colors.ts';
import { iconDot, } from '../data/model-icons.ts';

import type { ViewerEntry, } from '../data/viewer-types.ts';

import {
  buildCrossModelPoints,
  buildSingleModelPoints,
} from './view-probe-builders.ts';
import { buildProbeLegend, } from './view-probe-legend.ts';

/**
 * Renders the by-probe view: one `<details>` per probe, with all models overlaid
 * and a per-model breakdown nested inside.
 *
 * @param entries - all history entries
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = renderByProbe({ entries });
 * // '<details class="probe-section">...<\/details>...'
 * ```
 */
export function renderByProbe({ entries, }: {
  readonly entries: readonly ViewerEntry[];
},): string {
  /**
   * All unique probe names across all entries
   */
  const probeNames = [...new Set(entries.flatMap(function probeKeys(entry,): string[] {
    return Object.keys(entry.probeScores,);
  },),),];

  if (probeNames.length
    === 0) {
    return h({
      tag: 'p',
      text: 'No probe data available.',
    },);
  }

  return probeNames
    .map(function renderProbeSection(probe,): string {
      /**
       * Scatter points spanning every model for the current probe.
       */
      const points = buildCrossModelPoints({
        entries,
        probe,
      },);
      /**
       * Color/icon legend rendered above the cross-model chart.
       */
      const legend = buildProbeLegend(entries,);

      // Per-model breakdown within this probe
      /**
       * Unique model labels that recorded a score for the current probe.
       */
      const labels = [...new Set(entries
        .filter(function hasProbe(entry,): boolean {
          return probe in entry
            .probeScores;
        },)
        .map(function getLabel(entry,): string {
          return entry.label;
        },),),];
      /**
       * Joined per-model `<details>` sections beneath the cross-model chart.
       */
      const modelBreakdown = labels
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
           * OpenRouter model ID from the first entry for vendor icon/color
           */
          const openrouterId = firstEntry.model;
          /**
           * Vendor-derived accent color reused across this model's points.
           */
          const color = vendorColor(openrouterId,);
          /**
           * Scatter points for this single model within the current probe.
           */
          const modelPoints = buildSingleModelPoints({
            entries,
            probe,
            label,
            openrouterId,
            color,
          },);
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
              renderScatterChart({
                points: modelPoints,
                caption: `${probe} - ${label}`,
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
        class: 'probe-section',
        children: [
          h({
            tag: 'summary',
            text: probe,
          },),
          h({
            tag: 'div',
            class: 'pane',
            children: [
              legend,
              renderScatterChart({
                points,
                caption: `${probe} - all models`,
                tableDisplay: { showProbe: false, },
              },),
              h({
                tag: 'h3',
                text: 'Per-model breakdown',
              },),
              modelBreakdown,
            ],
          },),
        ],
      },);
    },)
    .join('\n',);
}
