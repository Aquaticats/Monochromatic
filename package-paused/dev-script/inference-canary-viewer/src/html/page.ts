/**
 * Top-level HTML document shell.
 *
 * Produces a complete `<!DOCTYPE html>` page linking to the external stylesheet.
 * All content is static; viewable with JavaScript disabled.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Wraps body content in a full HTML document.
 *
 * @param body - inner body HTML
 *
 * @param title - page title
 *
 * @returns complete HTML document string
 *
 * @example
 * ```ts
 * const html = renderPage({ body: '<p>Hello<\/p>', title: 'Dashboard' });
 * // '<!DOCTYPE html>\n<html lang="en">...<\/html>'
 * ```
 */
export function renderPage({
  body,
  title,
}: {
  readonly body: string;
  readonly title: string;
},): string {
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
                content: 'light dark',
              },
            },),
            h({
              tag: 'title',
              text: title,
            },),
            h({
              tag: 'style',
              html: 'html { background-color: light-dark(#fafafa, #1a1a1a) }',
            },),
            h({
              tag: 'link',
              attrs: {
                rel: 'stylesheet',
                href: 'style.css',
              },
            },),
            h({
              tag: 'script',
              attrs: {
                type: 'module',
                src: 'client/index.js',
              },
            },),
          ],
        },),
        h({
          tag: 'body',
          children: [
            h({
              tag: 'header',
              class: 'site-header',
              children: [h({
                tag: 'h1',
                text: title,
              },),],
            },),
            h({
              tag: 'main',
              html: body,
            },),
          ],
        },),
      ],
    },)
  }`;
}
