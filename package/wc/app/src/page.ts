/**
 HTML document structure for the wc text-stats tool.
 
 Uses h-html to produce a self-contained page with inlined CSS and
 JavaScript: a viewport-filling flex column holding a masthead, an input
 textarea, six stat tiles (`./page-stats.ts`), and a Frequency section
 marked up as flex rows with ARIA table roles so per-row
 `content-visibility: auto` works (`content-visibility` is ignored on
 internal table boxes like `tr`).
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { FAVICON_SIZE, } from './favicon.ts';
import { renderStatsSection, } from './page-stats.ts';

export {
  STAT_TILES,
  type StatSub,
  type StatTile,
} from './page-stats.ts';

/**
 One-line tool description, shared by the masthead paragraph and the
 `meta name="description"` head tag so the two never drift apart.
 */
const PAGE_DESCRIPTION =
  'Byte, character, line, word, sentence, and paragraph statistics, plus word frequency, computed live in the browser as you type.';

/**
 Renders the input panel: a label wrapping the textarea the client
 script reads text from, an implicit label/control association that
 needs no `id`/`for` pair. The textarea flexes to fill the remaining
 viewport height and auto-grows with content via the client script.
 
 @returns HTML string for the input panel
 */
function renderInputPanel(): string {
  return h(
    {
      tag: 'section',
      attrs: { class: 'input-panel', },
      children: [
        h(
          {
            tag: 'label',
            class: 'input-label',
            children: [
              h(
                {
                  tag: 'span',
                  class: 'input-label-text',
                  text: 'Text to analyze',
                },
              ),
              h(
                {
                  tag: 'textarea',
                  class: 'wc-input',
                  attrs: { placeholder: 'Paste or type text here…', },
                },
              ),
            ],
          },
        ),
      ],
    },
  );
}

/**
 Column labels of the Frequency table's visually hidden header row, in
 cell order. The decorative bar column has no header: its cell is
 `aria-hidden` (see `./client/main.ts`), so assistive tech sees
 exactly these columns.
 */
export const FREQUENCY_COLUMN_LABELS: readonly string[] = [
  'Count',
  'Percent',
  'Word',
];

/**
 Renders the Frequency table's header row: visually hidden by design
 (the numbers and words are self-explanatory to sighted users), but
 kept in the accessibility tree so screen readers get column context,
 via the inclusively-hidden pattern (`.visually-hidden` in
 `./styles-layout.ts`).
 
 @returns HTML string for the header row
 */
function renderFrequencyHeaderRow(): string {
  return h(
    {
      tag: 'div',
      class: 'frequency-header visually-hidden',
      attrs: { role: 'row', },
      children: FREQUENCY_COLUMN_LABELS
        .map(function renderColumnHeader(label,): string {
          return h(
            {
              tag: 'span',
              attrs: { role: 'columnheader', },
              text: label,
            },
          );
        },),
    },
  );
}

/**
 Renders the Frequency section: a heading and an ARIA table (flex rows,
 not a native `<table>`, so per-row `content-visibility: auto` takes
 effect) holding a visually hidden header row
 ({@link renderFrequencyHeaderRow}) and the body rowgroup the client
 script fills with word-frequency rows.
 
 @returns HTML string for the Frequency section
 */
function renderFrequencySection(): string {
  return h(
    {
      tag: 'section',
      class: 'frequency-section',
      children: [
        h(
          {
            tag: 'h2',
            text: 'Frequency',
          },
        ),
        h(
          {
            tag: 'div',
            class: 'frequency',
            attrs: {
              role: 'table',
              'aria-label': 'Word frequency',
            },
            children: [
              renderFrequencyHeaderRow(),
              h(
                {
                  tag: 'div',
                  class: 'frequency-body',
                  attrs: { role: 'rowgroup', },
                },
              ),
            ],
          },
        ),
      ],
    },
  );
}

/**
 Renders the masthead: the tool name and a one-line description.
 
 @returns HTML string for the masthead
 */
function renderMasthead(): string {
  return h(
    {
      tag: 'header',
      class: 'masthead',
      children: [
        h(
          {
            tag: 'h1',
            text: 'wc',
          },
        ),
        h(
          {
            tag: 'p',
            attrs: { class: 'description', },
            text: PAGE_DESCRIPTION,
          },
        ),
        h(
          {
            // Every count on the page is computed by the inlined
            // script; without it the zeros would sit unexplained.
            tag: 'noscript',
            children: [
              h(
                {
                  tag: 'p',
                  attrs: { class: 'noscript-note', },
                  text:
                    'JavaScript is off, so every count stays at 0. The text box still accepts and scrolls text; enable scripting to compute stats.',
                },
              ),
            ],
          },
        ),
      ],
    },
  );
}

/**
 Renders the complete HTML document with all content inlined, combining
 {@link renderMasthead}, {@link renderInputPanel},
 {@link renderStatsSection} (`./page-stats.ts`), and
 {@link renderFrequencySection}.
 
 @param css - CSS stylesheet string
 
 @param js - client-side JavaScript string
 
 @param faviconSvgBase64 - base64-encoded SVG favicon markup, inlined
 as a data URI; engines that take vector icons pick it
 
 @param faviconPngBase64 - base64-encoded PNG favicon bytes, inlined
 as a data URI; raster fallback for engines without SVG icon support
 
 @returns complete HTML document string
 
 @example
 ```ts
 const html = renderPage({
   css: 'body {}',
   js: 'console.log("ok")',
   faviconSvgBase64: 'PHN2Zy…',
   faviconPngBase64: 'iVBORw0KGgo…',
 });
 ```
 */
export function renderPage(
  {
    css,
    js,
    faviconSvgBase64,
    faviconPngBase64,
  }: Readonly<{
    css: string;
    js: string;
    faviconSvgBase64: string;
    faviconPngBase64: string;
  }>,
): string {
  return `<!DOCTYPE html>\n${
    h(
      {
        tag: 'html',
        attrs: { lang: 'en', },
        children: [
          h(
            {
              tag: 'head',
              children: [
                h(
                  {
                    tag: 'meta',
                    // oxlint-disable-next-line unicorn/text-encoding-identifier-case -- Destination grammar wins: the HTML spec requires the literal `utf-8` for meta charset; `utf8` is only a legacy encoding label.
                    attrs: { charset: 'utf-8', },
                  },
                ),
                h(
                  {
                    tag: 'meta',
                    attrs: {
                      name: 'viewport',
                      content: 'width=device-width, initial-scale=1',
                    },
                  },
                ),
                h(
                  {
                    tag: 'meta',
                    attrs: {
                      name: 'description',
                      content: PAGE_DESCRIPTION,
                    },
                  },
                ),
                h(
                  {
                    tag: 'title',
                    text: 'wc: text stats',
                  },
                ),
                h(
                  {
                    // `sizes="any"` steers engines that support vector
                    // icons (Chromium prefers a sized raster otherwise);
                    // the PNG link below is the fallback for engines
                    // that don't (e.g. Safari).
                    tag: 'link',
                    attrs: {
                      rel: 'icon',
                      type: 'image/svg+xml',
                      sizes: 'any',
                      href: `data:image/svg+xml;base64,${faviconSvgBase64}`,
                    },
                  },
                ),
                h(
                  {
                    tag: 'link',
                    attrs: {
                      rel: 'icon',
                      type: 'image/png',
                      sizes: `${FAVICON_SIZE}x${FAVICON_SIZE}`,
                      href: `data:image/png;base64,${faviconPngBase64}`,
                    },
                  },
                ),
                h(
                  {
                    tag: 'style',
                    html: css,
                  },
                ),
              ],
            },
          ),
          h(
            {
              tag: 'body',
              children: [
                h(
                  {
                    tag: 'div',
                    attrs: { class: 'page', },
                    children: [
                      renderMasthead(),
                      h(
                        {
                          tag: 'div',
                          attrs: { class: 'layout', },
                          children: [
                            renderInputPanel(),
                            h(
                              {
                                tag: 'div',
                                attrs: { class: 'results-panel', },
                                children: [
                                  renderStatsSection(),
                                  renderFrequencySection(),
                                ],
                              },
                            ),
                          ],
                        },
                      ),
                    ],
                  },
                ),
                h(
                  {
                    tag: 'script',
                    attrs: { type: 'module', },
                    html: js,
                  },
                ),
              ],
            },
          ),
        ],
      },
    )
  }`;
}
