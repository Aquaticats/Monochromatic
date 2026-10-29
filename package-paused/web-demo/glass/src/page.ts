/**
 * HTML document structure for the glass corridor demo.
 *
 * Uses h-html to produce a self-contained page with inlined CSS and the
 * pre-bundled client script.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Renders the HUD chrome: brand, score, hint, and backend badge.
 *
 * @returns HTML string for the HUD elements
 */
function renderHud(): string {
  return [
    h({
      tag: 'div',
      attrs: {
        id: 'brand',
        class: 'hud',
      },
      children: [
        h({
          tag: 'b',
          text: 'Glass corridor',
        },),
        h({
          tag: 'span',
          text: 'Walk forward, throw, crack, shatter',
        },),
      ],
    },),
    h({
      tag: 'div',
      attrs: {
        id: 'score',
        class: 'hud',
      },
      children: [
        h({
          tag: 'strong',
          attrs: { id: 'score-value', },
          text: '0',
        },),
        h({
          tag: 'span',
          text: 'panes shattered',
        },),
      ],
    },),
    h({
      tag: 'div',
      attrs: {
        id: 'hint',
        class: 'hud',
      },
      text: 'Click or tap to throw',
    },),
    h({
      tag: 'div',
      attrs: {
        id: 'backend-box',
        class: 'hud',
      },
      children: [
        h({
          tag: 'span',
          attrs: { id: 'backend', },
          text: 'Starting',
        },),
      ],
    },),
  ]
    .join('',);
}

/**
 * Renders the error overlay shown when the renderer cannot start.
 *
 * @returns HTML string for the hidden error overlay
 */
function renderErrorOverlay(): string {
  return h({
    tag: 'div',
    attrs: {
      id: 'error',
      hidden: '',
    },
    children: [
      h({
        tag: 'div',
        children: [
          h({
            tag: 'h1',
            text: 'This browser cannot start the scene',
          },),
          h({
            tag: 'p',
            attrs: { id: 'error-copy', },
          },),
        ],
      },),
    ],
  },);
}

/**
 * Renders the complete HTML document with all content inlined.
 *
 * @param css - CSS stylesheet string
 *
 * @param js - client-side JavaScript string
 *
 * @returns complete HTML document string
 *
 * @example
 * ```ts
 * const html = renderPage({ css: 'body {}', js: 'console.log("ok")' });
 * ```
 */
export function renderPage(
  {
    css,
    js,
  }: Readonly<{
    css: string;
    js: string;
  }>,
): string {
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
              tag: 'title',
              text: 'Glass corridor demo',
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
              tag: 'canvas',
              attrs: {
                id: 'stage',
                'aria-label': 'Click or tap to throw a ball at the glass',
              },
            },),
            renderHud(),
            renderErrorOverlay(),
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
