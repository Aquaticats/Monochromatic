/**
 Unit tests for Advisor rendering summaries.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AdvisorDetails,
  type AdvisorTheme,
  firstAdvisoryLine,
  renderAdvisorSummary,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** Theme boundary returns text verbatim so content assertions ignore ANSI styling. */
const theme = {
  fg: (_color: string, text: string): string => text,
  bold: (text: string): string => text,
} as unknown as AdvisorTheme;

/** Minimal Advisor details carrying one duration in milliseconds. */
function detailsWithDuration(durationMs: number): AdvisorDetails {
  return {
    selectedSlug: 'faux/reviewer',
    provider: 'faux',
    scopeSource: 'available',
    scopedSlugs: ['faux/reviewer'],
    durationMs,
    contextBudgetChars: 100,
    contextChars: 12,
    estimatedInputTokens: 10,
    truncated: false,
    stopReason: 'stop',
  };
}

//endregion Fixtures

await describe({
  name: firstAdvisoryLine.name,
  children: [
    it({
      name: 'skips markdown headings before body text',
      fn: async function testSkipsMarkdownHeadingsBeforeBodyText() {
        expect(firstAdvisoryLine(`
## Advisor review

### Flawed assumptions

1. **Assumption** Check the active model identity.
`,),).toBe('1. **Assumption** Check the active model identity.',);
      },
    },),
    it({
      name: 'keeps markdown heading when it is the only text',
      fn: async function testKeepsOnlyMarkdownHeading() {
        expect(firstAdvisoryLine('## Advisor review',),).toBe('## Advisor review',);
      },
    },),
  ],
},);

await describe({
  name: renderAdvisorSummary.name,
  children: [
    it({
      name: 'rounds sub-ten-millisecond durations to one decimal',
      fn: async function testSubTenMillisecondDuration() {
        const rendered = renderAdvisorSummary({
          text: 'Looks good.',
          details: detailsWithDuration(0.34,),
          expanded: false,
          theme,
        },);
        expect(rendered,).toContain('0.3ms',);
      },
    },),
    it({
      name: 'drops decimals above ten milliseconds below one second',
      fn: async function testMillisecondDuration() {
        const rendered = renderAdvisorSummary({
          text: 'Looks good.',
          details: detailsWithDuration(51.23,),
          expanded: false,
          theme,
        },);
        expect(rendered,).toContain('51ms',);
      },
    },),
    it({
      name: 'shows one decimal in seconds at one second and above',
      fn: async function testSecondDuration() {
        const rendered = renderAdvisorSummary({
          text: 'Looks good.',
          details: detailsWithDuration(1_234,),
          expanded: false,
          theme,
        },);
        expect(rendered,).toContain('1.2s',);
      },
    },),
  ],
},);
