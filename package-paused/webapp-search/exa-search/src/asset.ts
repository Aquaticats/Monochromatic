import { readFile, } from 'node:fs/promises';

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { buildHeader, } from './asset-header.ts';
import {
  buildSearchForm,
  buildStatusSection,
} from './asset-page-sections.ts';
import { buildResultTemplate, } from './asset-result-template.ts';
import clientCss from './index.css' with { type: 'text', };
import { l, } from './log.ts';

l.debug(`asset module loading`,);

/**
 * CSS source for the exa-search interface.
 * Imported at build time via static asset import.
 */
export const css: string = clientCss;

/**
 * Bundled client-side JavaScript for the exa-search interface.
 * Pre-built by tsdown via `mise run build:js:client` and read from disk at startup.
 */
export const js: string = await readFile(
  './dist/client/client.js',
  'utf8',
);

/**
 * Escaped JS source safe for embedding inside a `<script>` tag.
 */
const safeJs: string = js.replaceAll(
  '</script>',
  String.raw`<\/script>`,
);

//region HTML structure: Declarative page composition via h-html

/**
 * Complete self-contained HTML page with inlined CSS and JS.
 * Composed declaratively via h-html; served as the response for every GET / request.
 */
export const indexHtml: string = [
  '<!DOCTYPE html>',
  h({
    tag: 'html',
    attrs: { lang: 'en', },
    children: [
      h({
        tag: 'head',
        children: [
          h({
            tag: 'meta',
            attrs: { charset: 'utf8', },
          },),
          h({
            tag: 'meta',
            attrs: {
              name: 'viewport',
              content: 'width=device-width, initial-scale=1.0',
            },
          },),
          h({
            tag: 'title',
            text: 'Exa Search',
          },),
          h({
            tag: 'style',
            html: css,
          },),
          h({
            tag: 'script',
            attrs: { type: 'module', },
            html: safeJs,
          },),
        ],
      },),
      h({
        tag: 'body',
        children: [
          buildHeader(),
          h({
            tag: 'main',
            class: 'main',
            children: [
              buildSearchForm(),
              buildStatusSection(),
              h({
                tag: 'ol',
                class: 'results',
                attrs: { hidden: '', },
                children: [buildResultTemplate(),],
              },),
            ],
          },),
        ],
      },),
    ],
  },),
]
  .join('',);

//endregion HTML structure

l.debug(`asset module loaded, css ${css.length} chars, js ${js.length} chars`,);
