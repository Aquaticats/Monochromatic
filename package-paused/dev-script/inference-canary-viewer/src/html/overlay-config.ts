/**
 * Collapsible config snapshot renderer for probe overlays.
 *
 * Renders a `<details>` element showing runner configuration parameters
 * from an enriched artifact's config snapshot.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { ConfigSnapshot, } from '../data/viewer-types.ts';

import { formatNumber, } from './overlay-meta.ts';

/**
 * Renders a collapsible config snapshot section.
 *
 * @param config - runner configuration snapshot
 *
 * @returns HTML `<details>` element
 *
 * @example
 * ```ts
 * const html = renderConfig(config);
 * // '<details class="collapsible-section">...<\/details>'
 * ```
 */
export function renderConfig(config: ConfigSnapshot,): string {
  return h({
    tag: 'details',
    class: 'collapsible-section',
    children: [
      h({
        tag: 'summary',
        text: 'Config',
      },),
      h({
        tag: 'dl',
        class: 'metadata-grid',
        children: [
          h({
            tag: 'dt',
            text: 'Verbosity',
          },),
          h({
            tag: 'dd',
            text: config.verbosity,
          },),
          h({
            tag: 'dt',
            text: 'Reasoning',
          },),
          h({
            tag: 'dd',
            text: String(config.reasoning,),
          },),
          h({
            tag: 'dt',
            text: 'Max tokens',
          },),
          h({
            tag: 'dd',
            text: formatNumber(config.maxTokens,),
          },),
          h({
            tag: 'dt',
            text: 'Consistency runs',
          },),
          h({
            tag: 'dd',
            text: String(config.consistencyRuns,),
          },),
        ],
      },),
    ],
  },);
}
