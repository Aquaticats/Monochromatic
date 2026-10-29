/**
 * Renders individual scatter point HTML elements.
 *
 * Each data point becomes a positioned `<button>` element.
 * Pass-1 points are filled circles; pass-2 points are hollow circles
 * overlaid at the same X position.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { ScatterPoint, } from './scatter.ts';

/**
 * Percentage multiplier
 */
const PERCENT = 100;

/**
 * Center position percentage for single-point charts
 */
const CENTER_PERCENT = 50;

/**
 * Renders all scatter point button elements as a single HTML string.
 *
 * Positions each point absolutely within the chart plot area using
 * percentage-based `left` and `bottom` CSS values.
 *
 * @param points - data points to render as positioned buttons
 *
 * @returns concatenated HTML string of all point elements
 *
 * @example
 * ```ts
 * const html = renderPointElements(points);
 * // '<button class="chart-point" ...>...</button>\n<button ...'
 * ```
 */
export function renderPointElements(points: readonly ScatterPoint[],): string {
  /**
   * Denominator for horizontal spacing across the plot area.
   */
  const totalRuns = points.length;

  return points
    .map(function renderPoint(point,) {
      /**
       * Horizontal position percentage along the plot's inline axis.
       */
      const left = totalRuns === 1
        ? CENTER_PERCENT
        : (point.index
          / (totalRuns - 1)) * PERCENT;
      /**
       * Vertical position percentage from the plot's block-end edge.
       */
      const bottom = point.score
        * PERCENT;

      /**
       * Whether the point should render a vendor icon glyph instead of a plain dot.
       */
      const hasIcon = (point.icon
        !== undefined)
        && (!point.failed);
      /**
       * Embedded SVG markup for the optional icon glyph.
       */
      const iconHtml = hasIcon ? point.icon : '';

      /**
       * Primary (pass-1) scatter point button markup.
       */
      const pass1 = h({
        tag: 'button',
        class: 'chart-point',
        style: {
          left: `${left.toFixed(2,)}%`,
          bottom: `${bottom.toFixed(2,)}%`,
          '--point-color': point.color,
        },
        attrs: {
          popovertarget: `run-${point.runId}`,
          title: point.title,
          'aria-label': point.title,
          ...(point.failed ? { 'data-status': 'failed', } : {}),
          ...(hasIcon ? { 'data-shape': 'icon', } : {}),
        },
        html: iconHtml,
      },);

      if (point.pass2Score
        === undefined)
        return pass1;

      /**
       * Pass-2 vertical position percentage shown above the primary point.
       */
      const pass2Bottom = point.pass2Score
        * PERCENT;
      /**
       * Whether the pass-2 overlay should render a vendor icon glyph.
       */
      const pass2HasIcon = point.icon
        !== undefined;
      /**
       * Embedded SVG markup for the pass-2 overlay glyph.
       */
      const pass2IconHtml = pass2HasIcon ? point.icon : '';
      /**
       * Overlaid (pass-2) scatter point button markup.
       */
      const pass2 = h({
        tag: 'button',
        class: 'chart-point',
        style: {
          left: `${left.toFixed(2,)}%`,
          bottom: `${pass2Bottom.toFixed(2,)}%`,
          '--point-color': point.color,
        },
        attrs: {
          popovertarget: `run-${point.runId}`,
          title: `fix: ${point.pass2Score
            .toFixed(2,)}`,
          'aria-label': `fix score ${point.pass2Score
            .toFixed(2,)}`,
          'data-pass': 'fix',
          ...(pass2HasIcon ? { 'data-shape': 'icon', } : {}),
        },
        html: pass2IconHtml,
      },);

      return `${pass1}\n${pass2}`;
    },)
    .join('\n',);
}
