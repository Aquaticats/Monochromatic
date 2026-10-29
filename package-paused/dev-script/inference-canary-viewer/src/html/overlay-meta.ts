/**
 * Enriched metadata rendering helpers for probe detail overlays.
 *
 * Renders timing, usage, and badges as compact HTML sections
 * within popover overlays.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import type {
  ProbeDetail,
  StreamTiming,
  StreamUsage,
} from '../data/viewer-types.ts';

/**
 * Formats milliseconds as a human-readable duration.
 *
 * @param ms - milliseconds
 *
 * @returns "1.2s" for \>= 1000ms, "123ms" otherwise
 *
 * @example
 * ```ts
 * formatMs(1234); // "1.2s"
 * formatMs(50); // "50ms"
 * ```
 */
function formatMs(ms: number,): string {
  if (ms >= MS_PER_SECOND)
    return `${(ms / MS_PER_SECOND).toFixed(1,)}s`;
  return `${Math.round(ms,)}ms`;
}

/**
 * Formats a number with locale-appropriate thousands separators.
 *
 * @param num - number to format
 *
 * @returns formatted string
 *
 * @example
 * ```ts
 * formatNumber(12345); // "12,345"
 * ```
 */
export function formatNumber(num: number,): string {
  return num.toLocaleString('en-US',);
}

/**
 * Renders status badges for partial/error/finish-reason states.
 *
 * @param detail - probe detail
 *
 * @returns HTML string, empty when no badges apply
 *
 * @example
 * ```ts
 * renderBadges({ partial: true, error: 'timeout', finishReason: 'length' });
 * // '<div class="detail-popover-badges">...'
 * ```
 */
export function renderBadges(detail: ProbeDetail,): string {
  /**
   * Accumulator for badge spans whose corresponding state flag is set.
   */
  const badges: string[] = [];

  if (detail.partial
    === true) {
    badges.push(
      h({
        tag: 'span',
        class: 'run-indicator',
        attrs: { 'data-severity': 'warning', },
        text: 'partial',
      },),
    );
  }
  if ((detail.error
    !== undefined) && (detail.error
      !== '')) {
    badges.push(
      h({
        tag: 'span',
        class: 'run-indicator',
        attrs: { 'data-severity': 'error', },
        text: detail.error,
      },),
    );
  }
  if ((detail.finishReason
    !== undefined) && (detail.finishReason
      !== 'stop')) {
    badges.push(
      h({
        tag: 'span',
        class: 'run-indicator',
        attrs: { 'data-severity': 'neutral', },
        text: detail.finishReason,
      },),
    );
  }

  if (badges.length
    === 0)
    return '';
  return h({
    tag: 'div',
    class: 'detail-popover-badges',
    children: badges,
  },);
}

/**
 * Renders a compact metadata grid for one pass (initial or fix).
 * Shows timing and token usage as a collapsed `<details>` element.
 *
 * @param label - section label ("Initial pass" or "Fix pass")
 *
 * @param timing - streaming timing data
 *
 * @param usage - token usage data
 *
 * @param finishReason - why generation stopped
 *
 * @returns HTML string, empty when no data is available
 *
 * @example
 * ```ts
 * renderPassMeta({ label: 'Initial pass', timing, usage, finishReason: 'stop' });
 * // '<details class="collapsible-section"><summary>Initial pass</summary><dl ...>...'
 * ```
 */
export function renderPassMeta({
  label,
  timing,
  usage,
  finishReason,
}: {
  readonly label: string;
  readonly timing?: StreamTiming;
  readonly usage?: StreamUsage;
  readonly finishReason?: string;
},): string {
  /**
   * Accumulator for `<dt>`/`<dd>` pairs added per available metric.
   */
  const items: string[] = [];

  if (timing !== undefined) {
    items.push(
      h({
        tag: 'dt',
        text: 'TTFC',
      },),
      h({
        tag: 'dd',
        text: formatMs(timing.timeToFirstChunkMs,),
      },),
      h({
        tag: 'dt',
        text: 'Total time',
      },),
      h({
        tag: 'dd',
        text: formatMs(timing.totalMs,),
      },),
      h({
        tag: 'dt',
        text: 'Chunks',
      },),
      h({
        tag: 'dd',
        text: formatNumber(timing.chunkCount,),
      },),
    );
  }

  if (usage !== undefined) {
    items.push(
      h({
        tag: 'dt',
        text: 'Prompt tokens',
      },),
      h({
        tag: 'dd',
        text: formatNumber(usage.promptTokens,),
      },),
      h({
        tag: 'dt',
        text: 'Completion tokens',
      },),
      h({
        tag: 'dd',
        text: formatNumber(usage.completionTokens,),
      },),
    );
    if (usage.reasoningTokens
      !== undefined) {
      items.push(
        h({
          tag: 'dt',
          text: 'Reasoning tokens',
        },),
        h({
          tag: 'dd',
          text: formatNumber(usage.reasoningTokens,),
        },),
      );
    }
    items.push(
      h({
        tag: 'dt',
        text: 'Total tokens',
      },),
      h({
        tag: 'dd',
        text: formatNumber(usage.totalTokens,),
      },),
    );
  }

  if (finishReason !== undefined) {
    items.push(
      h({
        tag: 'dt',
        text: 'Finish reason',
      },),
      h({
        tag: 'dd',
        text: finishReason,
      },),
    );
  }

  if (items.length
    === 0)
    return '';

  return h({
    tag: 'details',
    class: 'collapsible-section',
    children: [
      h({
        tag: 'summary',
        text: label,
      },),
      h({
        tag: 'dl',
        class: 'metadata-grid',
        children: items,
      },),
    ],
  },);
}
