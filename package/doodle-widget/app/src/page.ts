/**
 HTML document structure for the doodle widget.
 
 Uses h-html to produce a self-contained page with inlined CSS,
 JavaScript, and embedded page background data.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { renderAttribution, } from './page-attribution.ts';
import { renderToolbar, } from './page-toolbar.ts';

/**
 Renders the canvas container with drawing surface and overlay layers.
 
 The drawing canvas renders beneath the SVG overlay. The overlay
 uses `mix-blend-mode: multiply` so white SVG fills become
 transparent (user strokes show through) while black outlines
 remain opaque on top.
 
 The SVG overlay starts empty; the client populates it from the
 embedded page backgrounds JSON on initialization.
 
 @returns canvas container HTML string
 */
function renderCanvasContainer(): string {
  return h({
    tag: 'div',
    attrs: { id: 'canvas-container', },
    children: [
      h({
        tag: 'div',
        attrs: { id: 'page', },
        children: [
          h({
            tag: 'div',
            attrs: { id: 'zoom-layer', },
            children: [
              h({
                tag: 'canvas',
                attrs: { id: 'draw-canvas', },
              },),
              h({
                tag: 'div',
                attrs: { id: 'svg-overlay', },
              },),
              h({
                tag: 'div',
                attrs: { id: 'text-layer', },
              },),
            ],
          },),
        ],
      },),
    ],
  },);
}

/**
 Renders the complete HTML document with all content inlined.
 
 @param css - CSS stylesheet string
 
 @param js - client-side JavaScript string
 
 @param svgBackgrounds - processed SVG background strings, one per page
 
 @param sourceUrl - browsable URL to the package source code on GitHub
 
 @returns complete HTML document string
 
 @mutates svgBackgrounds - `JSON.stringify` may invoke array accessors or proxy traps.
 
 @example
 ```ts
 const html = renderPage({ css: 'body {}', js: 'console.log("ok")', svgBackgrounds: [], sourceUrl: 'https://github.com/...' });
 ```
 */
export function renderPage(
  {
    css,
    js,
    svgBackgrounds,
    sourceUrl,
  }: {
    readonly css: string;
    readonly js: string;
    svgBackgrounds: readonly string[];
    readonly sourceUrl: string;
  },
): string {
  /**
   Escape `</` as `<\/` to prevent premature script tag closure.
   `\/` is a valid JSON escape for `/`.
   */
  const backgroundsJson = JSON.stringify(svgBackgrounds,)
    .replaceAll(
    '</',
    String.raw`<\/`,
    );

  return `<!DOCTYPE html>\n${
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
                content: 'width=device-width, initial-scale=1',
              },
            },),
            h({
              tag: 'meta',
              attrs: {
                name: 'color-scheme',
                content: 'light',
              },
            },),
            h({
              tag: 'title',
              text: 'Doodle Widget',
            },),
            h({
              tag: 'style',
              html: css,
            },),
          ],
        },),
        h({
          tag: 'body',
          children: [
            h({
              tag: 'div',
              attrs: { id: 'app', },
              children: [
                renderToolbar(svgBackgrounds.length,),
                renderCanvasContainer(),
                renderAttribution(sourceUrl,),
              ],
            },),
            h({
              tag: 'div',
              attrs: {
                id: 'zoom-toast',
                popover: 'manual',
              },
              text:
                'Click to zoom in \u00B7 Shift+click or long-press to zoom out \u00B7 Drag to pan',
            },),
            h({
              tag: 'script',
              attrs: {
                id: 'page-backgrounds',
                type: 'application/json',
              },
              html: backgroundsJson,
            },),
            h({
              tag: 'script',
              attrs: { type: 'module', },
              html: js,
            },),
          ],
        },),
      ],
    },)
  }`;
}
