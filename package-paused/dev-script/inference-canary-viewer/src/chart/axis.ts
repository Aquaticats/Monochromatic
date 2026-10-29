/**
 * Axis tick generation and label placement for scatter plots.
 *
 * Y axis uses fixed score ticks (0, 0.25, 0.5, 0.75, 1.0).
 * X axis uses timestamps, adapting label granularity to the data range:
 * multiple days show dates, same-day data shows times.
 *
 * Exceeds 100 lines: Y and X axis renderers share the same spatial
 * coordinate conventions and formatter selection logic.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  HALF,
  QUARTER,
  THREE_QUARTERS,
} from '@monochromatic-dev/const-core/ts';

/**
 * Fixed Y axis tick values for score plots (0 to 1)
 */
export const Y_TICKS: readonly number[] = [
  0,
  QUARTER,
  HALF,
  THREE_QUARTERS,
  1,
];

/**
 * Generates HTML for Y axis tick labels positioned absolutely within a chart container.
 * Each tick is a `<span>` positioned at the corresponding bottom percentage.
 *
 * @returns HTML string for Y axis ticks
 *
 * @example
 * ```ts
 * const html = renderYAxis();
 * // '<span class="tick" style="bottom:0%">0.00<\/span>\n...'
 * ```
 */
export function renderYAxis(): string {
  /**
   * Percentage multiplier to convert 0-1 score to 0-100% CSS bottom offset
   */
  const PERCENT = 100;
  return Y_TICKS
    .map(function renderTick(tick,) {
      /**
       * CSS bottom offset (as percent) corresponding to this tick value.
       */
      const bottom = tick * PERCENT;
      return h({
        tag: 'span',
        class: 'tick',
        style: { bottom: `${String(bottom,)}%`, },
        text: tick.toFixed(2,),
      },);
    },)
    .join('\n',);
}

/**
 * Generates HTML for X axis tick labels using timestamps.
 * Shows date labels at evenly spaced intervals to avoid overcrowding.
 * Adapts granularity: shows HH:MM when all points fall on the same day,
 * MM-DD for same-year multi-day ranges, or YYYY-MM-DD across years.
 *
 * @param timestamps - ordered array of ISO timestamp strings corresponding to data points
 *
 * @returns HTML string for X axis ticks
 *
 * @example
 * ```ts
 * const html = renderXAxis(['2026-03-01T00:00:00Z', '2026-03-02T00:00:00Z']);
 * // '<span class="tick" style="left:0.00%">03-01<\/span>\n...'
 * ```
 */
export function renderXAxis(timestamps: readonly string[],): string {
  if (timestamps.length
    === 0)
    return '';

  /**
   * Maximum number of X axis labels to show before skipping
   */
  const MAX_LABELS = 12;
  /**
   * Index stride between rendered ticks so the axis stays uncluttered.
   */
  const step = Math.max(
    1,
    Math.ceil(timestamps.length
      / MAX_LABELS,),
  );

  /**
   * Date/time formatter chosen by the data's span.
   */
  const formatter = chooseFormatter(timestamps,);

  /**
   * Percentage multiplier
   */
  const PERCENT = 100;
  /**
   * Center position when a single data point exists
   */
  const CENTER_PERCENT = HALF * PERCENT;
  /**
   * Total point count, used to spread ticks across the inline axis.
   */
  const total = timestamps.length;

  /**
   * Every stride-th timestamp paired with its original position; values come from the callback, so no index lookup is needed.
   */
  const picked = timestamps
    .map(function locate(
      timestamp,
      position,
    ): {
      readonly timestamp: string;
      readonly position: number;
    } {
      return {
        timestamp,
        position,
      };
    },)
    .filter(function onStride({ position, },): boolean {
      return (position % step)
        === 0;
    },);

  /**
   * Rendered tick spans; consecutive duplicate labels are suppressed against the previous picked tick.
   */
  const ticks = picked.map(function renderTick(
    {
      timestamp,
      position,
    },
    idx,
    all,
  ): string {
    /**
     * Horizontal position percentage for this tick along the inline axis.
     */
    const left = total === 1
      ? CENTER_PERCENT
      : (position / (total - 1)) * PERCENT;
    /**
     * Formatted label for this tick.
     */
    const label = formatter(timestamp,);
    /**
     * Immediately preceding picked tick, or undefined for the first one.
     */
    const previous = idx === 0 ? undefined : all[idx - 1];
    /**
     * Suppress consecutive duplicate labels so the axis stays readable
     */
    const displayLabel = (previous !== undefined) && (formatter(previous.timestamp,)
      === label) ? '' : label;
    return h({
      tag: 'span',
      class: 'tick',
      style: { left: `${left.toFixed(2,)}%`, },
      attrs: { title: timestamp, },
      text: displayLabel,
    },);
  },);

  return ticks.join('\n',);
}

/**
 * Chooses a label formatter based on the time span of the data.
 * Same-day data gets HH:MM labels; multi-day gets date labels.
 *
 * @param timestamps - all timestamps in the chart
 *
 * @returns formatter function mapping ISO timestamp to display label
 */
function chooseFormatter(timestamps: readonly string[],): (ts: string,) => string {
  /**
   * Distinct YYYY-MM-DD prefixes across the supplied timestamps.
   */
  const uniqueDates = new Set(timestamps.map(function extractDate(ts,) {
    return ts.slice(
      0,
      10,
    );
  },),);
  if (uniqueDates.size
    <= 1)
    return formatTime;
  return formatDate;
}

/**
 * Formats an ISO timestamp as HH:MM for same-day axis labels.
 *
 * @param timestamp - ISO 8601 timestamp string
 *
 * @returns time string like "14:30"
 *
 * @example
 * ```ts
 * formatTime('2026-03-06T14:30:00.000Z'); // "14:30"
 * ```
 */
function formatTime(timestamp: string,): string {
  /**
   * ISO format: YYYY-MM-DDTHH:MM:SS, time starts at index 11
   */
  const TIME_START = 11;
  /**
   * End index for the HH:MM slice extracted from the ISO timestamp.
   */
  const TIME_END = 16;
  if (timestamp.length
    < TIME_END)
    return timestamp;
  return timestamp.slice(
    TIME_START,
    TIME_END,
  );
}

/**
 * Formats an ISO timestamp into a short date label for axis display.
 * Shows month-day for dates within the current year, or year-month-day otherwise.
 *
 * @param timestamp - ISO 8601 timestamp string
 *
 * @returns short date string like "03-01" or "2025-12-15"
 *
 * @example
 * ```ts
 * formatDate('2026-03-01T01:12:43.219Z'); // "03-01"
 * ```
 */
function formatDate(timestamp: string,): string {
  if (timestamp.length
    < 10)
    return timestamp;
  /**
   * Extract YYYY-MM-DD from the ISO string
   */
  const datePart = timestamp.slice(
    0,
    10,
  );
  /**
   * Local current year used to detect the same-year shortening case.
   */
  const currentYear = new Date().getFullYear()
    .toString();
  if (datePart.startsWith(currentYear,)) {
    // Same year: show MM-DD only; skip "YYYY-" prefix
    /**
     * Length of the leading "YYYY-" prefix that gets trimmed for same-year labels.
     */
    const YEAR_PREFIX_LENGTH = 5;
    return datePart.slice(YEAR_PREFIX_LENGTH,);
  }
  return datePart;
}
