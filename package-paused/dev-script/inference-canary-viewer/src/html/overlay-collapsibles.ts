/**
 * Collapsible detail sections for probe overlays.
 *
 * Renders reasoning traces, responses, fix prompts, and config
 * as `<details>` elements that can be expanded to inspect run internals.
 */
import { micromark, } from 'micromark';

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { ProbeDetail, } from '../data/viewer-types.ts';

import { renderConfig, } from './overlay-config.ts';
import { formatNumber, } from './overlay-meta.ts';

/**
 * Renders collapsible detail sections for reasoning, fix prompt, and config.
 * Sections with no data are omitted entirely.
 *
 * @param detail - probe detail
 *
 * @returns HTML string with `<details>` elements
 *
 * @example
 * ```ts
 * renderCollapsibles(detail);
 * // '<details class="collapsible-section">...'
 * ```
 */
export function renderCollapsibles(detail: ProbeDetail,): string {
  /**
   * Accumulates only the `<details>` blocks whose source data is present.
   */
  const sections: string[] = [];

  if ((detail.reasoning
    !== undefined) && (detail.reasoning
      !== '')) {
    sections.push(h({
      tag: 'details',
      class: 'collapsible-section',
      children: [
        h({
          tag: 'summary',
          text: `Thinking (${
            formatNumber(detail
              .reasoning
              .length,)
          } chars)`,
        },),
        h({
          tag: 'div',
          class: 'rendered-markdown',
          html: micromark(detail
            .reasoning,),
        },),
      ],
    },),);
  }

  if ((detail.initialResponse
    !== undefined) && (detail.initialResponse
      !== '')) {
    sections.push(h({
      tag: 'details',
      class: 'collapsible-section',
      children: [
        h({
          tag: 'summary',
          text: `Response (${
            formatNumber(detail
              .initialResponse
              .length,)
          } chars)`,
        },),
        h({
          tag: 'div',
          class: 'rendered-markdown',
          html: micromark(detail
            .initialResponse,),
        },),
      ],
    },),);
  }

  if ((detail.fixReasoning
    !== undefined) && (detail.fixReasoning
      !== '')) {
    sections.push(h({
      tag: 'details',
      class: 'collapsible-section',
      children: [
        h({
          tag: 'summary',
          text: `Fix thinking (${
            formatNumber(detail
              .fixReasoning
              .length,)
          } chars)`,
        },),
        h({
          tag: 'div',
          class: 'rendered-markdown',
          html: micromark(detail
            .fixReasoning,),
        },),
      ],
    },),);
  }

  if ((detail.fixResponse
    !== undefined) && (detail.fixResponse
      !== '')) {
    sections.push(h({
      tag: 'details',
      class: 'collapsible-section',
      children: [
        h({
          tag: 'summary',
          text: `Fix response (${
            formatNumber(detail
              .fixResponse
              .length,)
          } chars)`,
        },),
        h({
          tag: 'div',
          class: 'rendered-markdown',
          html: micromark(detail
            .fixResponse,),
        },),
      ],
    },),);
  }

  if ((detail.fixPrompt
    !== undefined) && (detail.fixPrompt
      !== '')) {
    sections.push(h({
      tag: 'details',
      class: 'collapsible-section',
      children: [
        h({
          tag: 'summary',
          text: 'Fix prompt',
        },),
        h({
          tag: 'div',
          class: 'rendered-markdown',
          html: micromark(detail
            .fixPrompt,),
        },),
      ],
    },),);
  }

  if (detail.config
    !== undefined)
    sections.push(renderConfig(detail.config,),);

  return sections.join('\n',);
}
