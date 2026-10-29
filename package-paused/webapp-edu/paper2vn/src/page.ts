/**
 * HTML document skeleton for paper2vn.
 *
 * Holds an empty mount node plus an inlined sprite-pack JSON island.
 * The client SPA reads the JSON island on boot and renders all
 * screens client-side so the build stays single-file.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Renders the single-file HTML document.
 *
 * @param css - minified CSS stylesheet string
 *
 * @param js - bundled client-side JavaScript string
 *
 * @param spritePackManifest - sprite pack manifest serialized as JSON,
 *   embedded in a non-rendering script tag and fetched at runtime
 *
 * @returns complete HTML document string
 *
 * @example
 * ```ts
 * const html = renderPage({
 *   css: '.app { color: black; }',
 *   js: 'console.log("paper2vn ready");',
 *   spritePackManifest: '{"sprites":[]}',
 * });
 * // html === '<!DOCTYPE html>\n<html lang="en">...</html>'
 * ```
 */
export function renderPage(
  {
    css,
    js,
    spritePackManifest,
  }: Readonly<{
    css: string;
    js: string;
    spritePackManifest: string;
  }>,
): string {
  /**
   * Escape `</` as `<\/` to prevent premature script tag closure.
   * Valid JSON since `\/` is a legal escape for `/`.
   */
  const safeManifest = spritePackManifest.replaceAll(
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
                content: 'light dark',
              },
            },),
            h({
              tag: 'title',
              text: 'paper2vn',
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
              tag: 'main',
              attrs: { id: 'app', },
              text: '',
            },),
            h({
              tag: 'script',
              attrs: {
                id: 'sprite-pack',
                type: 'application/json',
              },
              html: safeManifest,
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
