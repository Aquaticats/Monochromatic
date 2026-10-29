/**
 * Threshold line overlay for scatter plots.
 *
 * Renders a dashed horizontal line at the computed degradation threshold
 * (mean - 2*stddev), with a label showing the threshold value.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Renders a threshold line as a positioned `<div>` inside a chart container.
 *
 * Charts without a degradation threshold pass no argument, so absence is
 * modelled by an omitted optional rather than a `0`/empty-string sentinel.
 *
 * @param threshold - threshold value (0-1) and its label, omitted to draw no line
 *
 * @returns HTML string for the threshold line, or empty string when absent or non-positive
 *
 * @example
 * ```ts
 * renderThresholdLine({ value: 0.75, label: 'threshold: 0.75', });
 * // '<div class="chart-threshold" style="bottom:75%">...<\/div>'
 * renderThresholdLine(); // ''
 * ```
 */
export function renderThresholdLine(threshold?: {
  readonly value: number;
  readonly label: string;
},): string {
  if (threshold === undefined)
    return '';
  if (threshold.value <= 0)
    return '';
  /**
   * Percentage multiplier
   */
  const PERCENT = 100;
  /**
   * Inline-style positioning expressed as percentage from the chart floor.
   */
  const bottom = threshold.value * PERCENT;
  return h({
    tag: 'div',
    class: 'chart-threshold',
    style: { bottom: `${String(bottom,)}%`, },
    attrs: { title: threshold.label, },
    children: [
      h({
        tag: 'span',
        class: 'label',
        text: threshold.label,
      },),
    ],
  },);
}
