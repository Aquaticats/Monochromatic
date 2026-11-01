/**
 * Top-level dashboard layout with three view sections.
 *
 * Uses `<details>` elements for view switching. CSS `:has()` on the parent
 * hides non-open siblings so only one view is visible at a time.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Wraps three view sections into the dashboard layout.
 *
 * @param overviewHtml - overview section content
 *
 * @param byModelHtml - per-model section content
 *
 * @param byProbeHtml - per-probe section content
 *
 * @param overlaysHtml - all run detail overlays (rendered outside the views)
 *
 * @returns dashboard HTML string
 *
 * @example
 * ```ts
 * const html = renderDashboard({ overviewHtml, byModelHtml, byProbeHtml, overlaysHtml });
 * // '<nav class="view-switcher">...<\/nav>\n...'
 * ```
 */
export function renderDashboard(
  {
    overviewHtml,
    byModelHtml,
    byProbeHtml,
    overlaysHtml,
  }: {
    readonly overviewHtml: string;
    readonly byModelHtml: string;
    readonly byProbeHtml: string;
    readonly overlaysHtml: string;
  },
): string {
  return `${
    h({
      tag: 'nav',
      class: 'view-switcher',
      children: [
        h({
          tag: 'details',
          class: 'view-section',
          attrs: { open: '', },
          children: [
            h({
              tag: 'summary',
              text: 'Overview',
            },),
            h({
              tag: 'div',
              class: 'pane',
              html: overviewHtml,
            },),
          ],
        },),
        h({
          tag: 'details',
          class: 'view-section',
          children: [
            h({
              tag: 'summary',
              text: 'By model',
            },),
            h({
              tag: 'div',
              class: 'pane',
              html: byModelHtml,
            },),
          ],
        },),
        h({
          tag: 'details',
          class: 'view-section',
          children: [
            h({
              tag: 'summary',
              text: 'By probe',
            },),
            h({
              tag: 'div',
              class: 'pane',
              html: byProbeHtml,
            },),
          ],
        },),
      ],
    },)
  }\n${overlaysHtml}`;
}
