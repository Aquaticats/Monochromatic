/**
 Runtime error-detail tests for Advisor tool rendering.
 
 @module
 */

import type {
  AgentToolResult,
  Theme,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AdvisorDetails,
  renderAdvisorResult,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** Render width large enough to keep one-line fixtures intact. */
const RENDER_WIDTH = 200;

/** Raw thrown-tool error shown by Pi. */
const ERROR_TEXT = 'advisor: call timed out after 120000ms for faux/reviewer on attempt 1';

/** Minimal theme methods used by Advisor renderer. */
const theme = {
  fg(_color: string, text: string,) {
    return text;
  },
  bold(text: string,) {
    return text;
  },
} as unknown as Theme;

/** Valid Advisor success details fixture. */
const validDetails: AdvisorDetails = {
  selectedSlug: 'faux/reviewer',
  provider: 'faux',
  scopeSource: 'available',
  scopedSlugs: ['faux/reviewer',],
  durationMs: 100,
  contextBudgetChars: 1_000,
  contextChars: 100,
  estimatedInputTokens: 25,
  truncated: false,
  stopReason: 'stop',
};

/**
 Render Advisor result text with optional runtime details.
 
 @param details - untrusted details supplied by Pi
 
 @param content - result text to render
 
 @returns rendered text lines joined for assertion
 */
function renderText(
  {
    details,
    content = ERROR_TEXT,
  }: {
    readonly details?: unknown;
    readonly content?: string;
  },
): string {
  /**
   Runtime text content passed to Advisor renderer.
   */
  const resultContent: AgentToolResult<unknown>['content'] = [{
    type: 'text',
    text: content,
  },];
  /** Runtime tool result passed to Advisor renderer. */
  const result = {
    content: resultContent,
    ...(details === undefined ? {} : { details, }),
  };
  return renderAdvisorResult({
    result,
    expanded: false,
    theme,
  },)
    .render(RENDER_WIDTH,)
    .join('\n',);
}

//endregion Fixtures

await describe({
  name: renderAdvisorResult.name,
  children: [
    ...[
      {
        name: 'missing details',
      },
      {
        name: 'empty details',
        details: {},
      },
      {
        name: 'partial details',
        details: {
          selectedSlug: 'faux/reviewer',
        },
      },
    ].map(function mapMalformedDetails(fixture,) {
      return it({
        name: `renders raw error for ${fixture.name}`,
        fn: async function testMalformedDetailsFallback() {
          /**
           Runtime details option, absent for missing-details fixture.
           */
          const renderOptions = 'details' in fixture
            ? { details: fixture.details, }
            : {};
          const text = renderText(renderOptions,);

          expect(text,).toContain(ERROR_TEXT,);
          expect(text,).not.toContain('undefined',);
          expect(text,).not.toContain('NaN',);
        },
      },);
    },),
    it({
      name: 'retains metadata summary for valid details',
      fn: async function testValidDetailsSummary() {
        const text = renderText({
          details: validDetails,
          content: 'advisor answer',
        },);

        expect(text,).toContain('faux/reviewer',);
        expect(text,).toContain('100/1000 chars',);
        expect(text,).toContain('advisor answer',);
      },
    },),
  ],
},);
