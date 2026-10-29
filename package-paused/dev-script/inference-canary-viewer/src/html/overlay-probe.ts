/**
 * Probe-specific overlay renderer for run detail popovers.
 *
 * Renders source code (with optional diff), enriched metadata,
 * and collapsible sections for reasoning, responses, and config.
 */
import { join, } from 'node:path';

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { computeDiff, } from '../data/diff.ts';
import type {
  ProbeDetail,
  ViewerEntry,
} from '../data/viewer-types.ts';

import { renderSideBySideDiff, } from './diff-view.ts';
import { renderCollapsibles, } from './overlay-collapsibles.ts';
import {
  renderBadges,
  renderPassMeta,
} from './overlay-meta.ts';

/**
 * Builds the source-code section of a probe overlay.
 *
 * Returns an empty string when the initial source is absent, a side-by-side
 * diff when both fix source and fix directory are present, or a single source
 * pane otherwise.
 *
 * @param detail - enriched probe detail
 *
 * @returns HTML string for the source section
 *
 * @example
 * ```ts
 * const html = await buildSourceSection(probeDetailWithDiff);
 * // '<details class="collapsible-section">...<\/details>'
 * ```
 */
async function buildSourceSection(detail: ProbeDetail,): Promise<string> {
  if (detail.initialSource
    === undefined)
    return '';
  if ((detail.fixSource
    !== undefined) && (detail.fixDir
      !== undefined)) {
    /**
     * Absolute path to the initial canary file; left side of the diff.
     */
    const initialFile = join(
      detail.initialDir,
      'canary.ts',
    );
    /**
     * Absolute path to the fixed canary file; right side of the diff.
     */
    const fixFile = join(
      detail.fixDir,
      'canary.ts',
    );
    /**
     * Computed line-level diff between initial and fix sources, fed to the side-by-side renderer.
     */
    const diffLines = await computeDiff({
      initialPath: initialFile,
      fixPath: fixFile,
    },);
    return h({
      tag: 'details',
      class: 'collapsible-section',
      children: [
        h({
          tag: 'summary',
          text: 'Source diff',
        },),
        renderSideBySideDiff(diffLines,),
      ],
    },);
  }
  return h({
    tag: 'details',
    class: 'collapsible-section',
    children: [
      h({
        tag: 'summary',
        text: 'Source',
      },),
      h({
        tag: 'pre',
        class: 'source-code',
        children: [
          h({
            tag: 'code',
            class: 'language-ts',
            text: detail.initialSource,
          },),
        ],
      },),
    ],
  },);
}

/**
 * Renders a probe-specific overlay with source code, diff, and enriched metadata.
 *
 * @param id - unique overlay ID
 *
 * @param entry - viewer entry
 *
 * @param probe - probe name
 *
 * @param detail - enriched probe detail (may be undefined for missing artifacts)
 *
 * @returns HTML string
 *
 * @example
 * ```ts
 * const html = await renderProbeOverlay({ id: 'Sonnet-csv-2026', entry, probe: 'csv-rfc4180', detail });
 * // '<div class="detail-popover" popover="auto" id="run-Sonnet-csv-2026">...<\/div>'
 * ```
 */
export async function renderProbeOverlay({
  id,
  entry,
  probe,
  detail,
}: {
  readonly id: string;
  readonly entry: ViewerEntry;
  readonly probe: string;
  readonly detail?: ProbeDetail;
},): Promise<string> {
  /**
   * Run label destructured from the viewer entry; used in the overlay title.
   */
  const { label, } = entry;
  /**
   * Initial-pass score for this probe; defaulted to zero so the title always renders a number.
   */
  const score = entry.probeScores[probe]
    ?? 0;
  /**
   * Fix-pass score for this probe; absent when no fix run produced one.
   */
  const pass2Score = entry.pass2Scores?.[probe];

  /**
   * Status badges (partial, error, non-stop finish reason); empty when no detail is available.
   */
  const badges = detail !== undefined ? renderBadges(detail,) : '';

  /**
   * Initial-pass metadata block (timing, usage, finish reason); omitted when detail is missing.
   */
  const initialMeta = detail !== undefined
    ? renderPassMeta({
      label: 'Initial pass',
      ...(detail.timing !== undefined ? { timing: detail.timing, } : {}),
      ...(detail.usage !== undefined ? { usage: detail.usage, } : {}),
      ...(detail.finishReason !== undefined ? { finishReason: detail.finishReason, } : {}),
    },)
    : '';
  /**
   * Fix-pass metadata block; rendered only when the run actually has fix-pass timing or usage.
   */
  const fixMeta = (detail !== undefined)
      && ((detail.fixTiming
        !== undefined) || (detail.fixUsage
          !== undefined))
    ? renderPassMeta({
      label: 'Fix pass',
      ...(detail.fixTiming !== undefined ? { timing: detail.fixTiming, } : {}),
      ...(detail.fixUsage !== undefined ? { usage: detail.fixUsage, } : {}),
      ...(detail.fixFinishReason !== undefined ? { finishReason: detail.fixFinishReason, } : {}),
    },)
    : '';

  /**
   * Source-code section content: diff, single source, or empty; placeholder shown when detail is missing.
   */
  const sourceSection = detail !== undefined
    ? await buildSourceSection(detail,)
    : h({
      tag: 'p',
      class: 'detail-popover-empty',
      text: 'Artifacts not available for this run.',
    },);

  /**
   * Collapsible detail sections (reasoning, fix prompt, config); empty when detail is missing.
   */
  const collapsibles = detail !== undefined ? renderCollapsibles(detail,) : '';

  /**
   * Optional `(fix: X.XX)` suffix on the title; only appears when a fix-pass score exists.
   */
  const pass2Suffix = pass2Score !== undefined ? ` (fix: ${pass2Score.toFixed(2,)})` : '';
  /**
   * Composed overlay heading combining label, probe, scores, timestamp, and failure marker.
   */
  const title = `${label} - ${probe} - ${
    score.toFixed(2,)
  }${pass2Suffix} - ${entry.timestamp}${entry.failed ? ' (FAILED)' : ''}`;

  return h({
    tag: 'div',
    class: 'detail-popover',
    attrs: {
      popover: 'auto',
      id: `run-${id}`,
      'data-layout': 'centered',
    },
    children: [
      h({
        tag: 'h2',
        class: 'detail-popover-title',
        text: title,
      },),
      badges,
      initialMeta,
      fixMeta,
      sourceSection,
      collapsibles,
    ],
  },);
}
