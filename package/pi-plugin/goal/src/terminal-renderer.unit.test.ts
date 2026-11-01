/**
 Built-artifact tests for human-only goal outcome rendering.
 
 @module
 */

import type {
  EntryRenderer,
  ExtensionAPI,
  Theme,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  registerGoalTerminalRenderer,
  type GoalCompletionDiagnostic,
} from '../dist/final/node/index.mjs';

/**
 Capture registered entry renderers by custom type.
 
 @returns fake API and captured renderers
 */
function rendererHarness(): {
  readonly api: ExtensionAPI;
  readonly renderers: ReadonlyMap<string, EntryRenderer>;
} {
  /** Mutable renderer registry. */
  const renderers = new Map<string, EntryRenderer>();
  /** Focused registration API. */
  const api = {
    registerEntryRenderer(
      customType: string,
      renderer: EntryRenderer,
    ) {
      renderers.set(customType, renderer,);
    },
  } as unknown as ExtensionAPI;
  return { api, renderers, };
}

/**
 Build color-neutral theme fixture.
 
 @returns theme preserving plain text
 */
function plainTheme(): Theme {
  return {
    fg(_color: string, text: string,) {
      return text;
    },
    bold(text: string,) {
      return text;
    },
  } as unknown as Theme;
}

await describe({
  name: registerGoalTerminalRenderer.name,
  children: [
    it({
      name: 'renders compact completion and expanded private audit only for human',
      fn: async () => {
        const harness = rendererHarness();
        registerGoalTerminalRenderer(harness.api,);
        const renderer = harness.renderers.get('goal:completion',);
        if (renderer === undefined)
          throw new Error('completion renderer was not registered',);
        /** Durable completion audit fixture. */
        const data: GoalCompletionDiagnostic = {
          runId: 'run-1',
          generationId: 'generation-1',
          approvalSource: 'model',
          reviewerIdentity: 'review/model',
          reviewerRationale: 'Every requirement is supported.',
          attemptedReviewerIdentities: ['review/model',],
          transcriptTruncated: false,
          completedAt: '2026-08-26T00:00:00.000Z',
        };
        const compact = renderer(
          { type: 'custom', customType: 'goal:completion', data, } as never,
          { expanded: false, } as never,
          plainTheme(),
        );
        const expanded = renderer(
          { type: 'custom', customType: 'goal:completion', data, } as never,
          { expanded: true, } as never,
          plainTheme(),
        );
        if ((compact === undefined) || (expanded === undefined))
          throw new Error('completion renderer returned no component',);
        const compactText = compact.render(120,).join('\n',).trim();
        expect(compactText,).toBe('Goal complete',);
        expect(compactText,).not.toContain('Reviewer:',);
        const expandedText = expanded.render(120,).join('\n',);
        expect(expandedText,).toContain('Goal complete',);
        expect(expandedText,).toContain('Reviewer: review/model',);
        expect(expandedText,).toContain('Every requirement is supported.',);
      },
    },),
  ],
},);
