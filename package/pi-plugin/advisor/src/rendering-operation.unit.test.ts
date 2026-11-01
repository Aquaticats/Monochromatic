/**
 Delegation tests for Advisor result rendering onto shared operation summaries.
 
 @module
 */

import type { Theme, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { renderAdvisorResult, } from '../dist/final/node/index.mjs';

//region Fixtures

/** Theme boundary returns text verbatim so content assertions ignore ANSI styling. */
const theme = { fg: (_color: string, text: string): string => text, } as unknown as Theme;

//endregion Fixtures

await describe({ name: '', children: [
  it({ name: 'delegates result rendering to the shared operation summary', fn: async (): Promise<void> => {
    const details = { operation: {
      attempts: [{ model: 'p/a', attempt: 1, state: 'cancelled' }, { model: 'q/b', attempt: 1, state: 'succeeded' }],
      reviews: [{ model: 'q/b' }], usageIncomplete: true, end: 'collection',
    } };
    const component = renderAdvisorResult({ result: { content: [{ type: 'text', text: 'Review first line', },], details, }, expanded: true, theme, },);
    expect(component.render(80,).join('\n',),).toContain('2 attempts',);
  }, },),
], },);
