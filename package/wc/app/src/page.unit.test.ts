/**
 Tests for the complete HTML document assembly.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { FAVICON_SIZE, } from './favicon.ts';
import {
  FREQUENCY_COLUMN_LABELS,
  renderPage,
} from './page.ts';

/**
 Fixture base64 payload standing in for the favicon PNG bytes.
 */
const FAVICON_FIXTURE_BASE64 = 'iVBORw0KGgo=';

/**
 Fixture base64 payload standing in for the favicon SVG markup.
 */
const FAVICON_SVG_FIXTURE_BASE64 = 'PHN2Zy8+';

/**
 Renders the page from fixture CSS/JS/favicon inputs, the shared
 arrangement for every markup assertion below.
 
 @returns complete document rendered from fixtures
 */
function renderFixturePage(): string {
  return renderPage(
    {
      css: '',
      js: '',
      faviconSvgBase64: FAVICON_SVG_FIXTURE_BASE64,
      faviconPngBase64: FAVICON_FIXTURE_BASE64,
    },
  );
}

await describe({
  name: renderPage.name,
  children: [
    it({
      name: 'inlines the given CSS and JS into the document',
      fn: async function inlinesCssAndJs(): Promise<void> {
        /**
         Complete document rendered from fixture CSS/JS strings.
         */
        const html = renderPage(
          {
            css: 'body{color:red}',
            js: 'console.log(1)',
            faviconSvgBase64: FAVICON_SVG_FIXTURE_BASE64,
            faviconPngBase64: FAVICON_FIXTURE_BASE64,
          },
        );

        expect(html.startsWith('<!DOCTYPE html>',),).toBe(true,);
        expect(html,).toContain('<style>body{color:red}</style>',);
        expect(html,).toContain('<script type="module">console.log(1)</script>',);
      },
    },),
    it({
      name: 'declares the spec-conformant utf-8 charset and a meta description',
      fn: async function declaresCharsetAndDescription(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        expect(html,).toContain('<meta charset="utf-8">',);
        expect(html,).toContain('<meta name="description" content="',);
      },
    },),
    it({
      name: 'inlines the favicon as SVG and PNG data URI links, vector first',
      fn: async function inlinesFaviconDataUris(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        /**
         Vector icon link: `sizes="any"` so engines that support SVG
         icons prefer it.
         */
        const svgLink =
          `<link rel="icon" type="image/svg+xml" sizes="any" href="data:image/svg+xml;base64,${FAVICON_SVG_FIXTURE_BASE64}">`;

        /**
         Raster fallback link.
         */
        const pngLink =
          `<link rel="icon" type="image/png" sizes="${FAVICON_SIZE}x${FAVICON_SIZE}" href="data:image/png;base64,${FAVICON_FIXTURE_BASE64}">`;

        expect(html,).toContain(svgLink,);
        expect(html,).toContain(pngLink,);
        expect(html.indexOf(svgLink,),).toBeLessThan(html.indexOf(pngLink,),);
      },
    },),
    it({
      name: 'explains the zeroed counts inside a noscript block',
      fn: async function explainsNoscript(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        /**
         Index of the opening noscript tag.
         */
        const noscriptIndex = html.indexOf('<noscript>',);

        /**
         Index of the explanatory note, which must fall inside the
         noscript block.
         */
        const noteIndex = html.indexOf('class="noscript-note"',);

        /**
         Index of the closing noscript tag.
         */
        const noscriptCloseIndex = html.indexOf(
          '</noscript>',
          noscriptIndex,
        );

        expect(noscriptIndex,).toBeGreaterThan(-1,);
        expect(noteIndex,).toBeGreaterThan(noscriptIndex,);
        expect(noscriptCloseIndex,).toBeGreaterThan(noteIndex,);
      },
    },),
    it({
      name: 'renders a visually hidden frequency header row naming every exposed column',
      fn: async function rendersHiddenHeaderRow(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        expect(html,).toContain(
          '<div class="frequency-header visually-hidden" role="row">',
        );
        for (const label of FREQUENCY_COLUMN_LABELS) {
          expect(html,).toContain(
            `<span role="columnheader">${label}</span>`,
          );
        }
      },
    },),
    it({
      name: 'includes the input textarea and the frequency body rowgroup',
      fn: async function includesInputAndFrequencyBody(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        expect(html,).toContain('class="wc-input"',);
        expect(html,).toContain('class="frequency-body"',);
      },
    },),
    it({
      name: 'nests the textarea inside its label for an implicit association',
      fn: async function nestsTextareaInLabel(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();
        /**
         Index of the opening `<label>` tag.
         */
        const labelIndex = html.indexOf('<label class="input-label">',);
        /**
         Index of the nested textarea, which must fall inside the label
         (no `for`/`id` pair exists to associate them otherwise).
         */
        const textareaIndex = html.indexOf('<textarea class="wc-input"',);
        /**
         Index of the label's closing tag.
         */
        const labelCloseIndex = html.indexOf(
          '</label>',
          labelIndex,
        );

        expect(labelIndex,).toBeGreaterThan(-1,);
        expect(textareaIndex,).toBeGreaterThan(labelIndex,);
        expect(labelCloseIndex,).toBeGreaterThan(textareaIndex,);
      },
    },),
    it({
      name: 'never emits an id or an inline style attribute',
      fn: async function omitsIdsAndInlineStyles(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        expect(html,).not
          .toContain(' id="',);
        expect(html,).not
          .toContain(' style="',);
      },
    },),
    it({
      name: 'marks the frequency section up as an ARIA table, not a native table',
      fn: async function usesAriaTableRoles(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        expect(html,).toContain('role="table"',);
        expect(html,).toContain('role="rowgroup"',);
        expect(html,).not
          .toContain('<table',);
      },
    },),
    it({
      name: 'wraps content in a viewport-filling page scaffold with a masthead',
      fn: async function wrapsInPageScaffold(): Promise<void> {
        /**
         Complete document rendered from fixture inputs.
         */
        const html = renderFixturePage();

        expect(html,).toContain('class="page"',);
        expect(html,).toContain('class="masthead"',);
        expect(html,).toContain('<h1>wc</h1>',);
        expect(html,).toContain('<title>wc: text stats</title>',);
      },
    },),
  ],
},);
