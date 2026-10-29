/**
 * Color legend builder for the per-probe view.
 *
 * Generates a legend mapping vendor colors and icons to model labels,
 * used alongside probe scatter charts.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { vendorColor, } from '../data/model-colors.ts';
import { iconDot, } from '../data/model-icons.ts';

import type { ViewerEntry, } from '../data/viewer-types.ts';

/**
 * Renders a color legend mapping model colors to labels.
 *
 * @param entries - history entries (to extract unique models)
 *
 * @returns HTML string for the legend
 *
 * @example
 * ```ts
 * const html = buildProbeLegend(entries);
 * // '<div class="chart-legend">...<\/div>'
 * ```
 */
export function buildProbeLegend(
  entries: readonly ViewerEntry[],
): string {
  /**
   * Deduplicate by label, keeping first occurrence for model ID
   */
  const seen = new Map<string, string>();
  for (const entry of entries) {
    if (!seen.has(entry.label,)) {
      seen.set(
        entry.label,
        entry.model,
      );
    }
  }

  /**
   * Rendered legend item markup joined into the legend container body.
   */
  const items = [...seen.entries(),]
    .map(function buildItem([label, openrouterId,],): string {
      /**
       * Vendor-derived accent color used by the inline dot icon.
       */
      const color = vendorColor(openrouterId,);
      return h({
        tag: 'span',
        class: 'item',
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
      },);
    },)
    .join('\n',);

  return h({
    tag: 'div',
    class: 'chart-legend',
    html: items,
  },);
}
