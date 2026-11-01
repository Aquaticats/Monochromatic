/**
 Shared HTML shell used by most page handlers (inbox, in-progress, settings).
 
 The rendered HTML contains three mounting points that the client entry script relies on:
 - `<main id="app">`: empty container; the client script imperatively builds its DOM here
 - `<script id="page-data" type="application/json">`: server-serialized JSON that the client
   reads via `readPageData()` (see `src/client/lib/page-data.ts`)
 - `<script type="module" src="...">`: the per-page bundled client entry (e.g. `/dist/client/inbox.js`)
 
 Some pages (search, task-details) render their own HTML inline instead of calling `renderPage`,
 because they need a different shell structure (e.g. search-bar replaces top-nav).
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Options for rendering a full HTML page with the standard layout shell.
 */
type LayoutOptions = {
  readonly title: string;
  readonly heading: string;
  /**
   Path to the bundled client entry script, e.g. "/dist/client/inbox.js"
   */
  readonly entryScriptPath: string;
  /**
   Arbitrary data serialized as JSON into `<script id="page-data">` for client hydration
   */
  readonly pageData: unknown;
  readonly hideTopNav?: boolean;
};

/**
 Escapes `\<` to prevent `\</script\>` injection inside the JSON blob.
 Used by page handlers that render their own HTML shell (search, task-details)
 in addition to the shared {@link renderPage} layout.
 
 @param data - Arbitrary data to serialize as JSON
 
 @returns Escaped JSON string safe for embedding in HTML
 
 @mutates data - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 const json = serializePageData({ tasks: [] });
 ```
 */
export function serializePageData(data: unknown,): string {
  return JSON.stringify(data,)
    .replaceAll(
    '<',
    String.raw`\u003C`,
    );
}

/**
 Inline script that wires the top-nav hamburger menu to the side-drawer
 */
const MENU_OPEN_SCRIPT = `document.addEventListener('menu-open', function() {
      var drawer = document.getElementById('drawer');
      if (drawer) drawer.open = true;
    });`;

/**
 Renders a full HTML page with the standard layout shell, serializing
 `options.pageData` via {@link serializePageData}.
 
 @param options - Page title, heading, script path, and serialized data
 
 @returns HTML response with the rendered page
 
 @mutates options - `JSON.stringify` may invoke hooks on page data stored in options.
 
 @example
 ```ts
 const response = renderPage({ title: 'Inbox', heading: 'Inbox', entryScriptPath: '/dist/client/inbox.js', pageData: {} });
 ```
 */
export function renderPage(options: LayoutOptions,): Response {
  /**
   Optional nav element; empty string when the page hides the top bar.
   */
  const topNav = options.hideTopNav
    === true
    ? ''
    : h({
      tag: 'top-nav',
      attrs: { heading: options.heading, },
    },);

  /**
   Full HTML document string; returned wrapped in a Response below.
   */
  const html = `<!DOCTYPE html>
${
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
              text: options.title,
            },),
          ],
        },),
        h({
          tag: 'body',
          children: [
            h({
              tag: 'side-drawer',
              attrs: { id: 'drawer', },
            },),
            h({
              tag: 'div',
              class: 'page-wrapper',
              children: [
                topNav,
                h({
                  tag: 'main',
                  attrs: { id: 'app', },
                },),
              ],
            },),
            h({
              tag: 'script',
              attrs: {
                type: 'application/json',
                id: 'page-data',
              },
              html: serializePageData(options.pageData,),
            },),
            h({
              tag: 'script',
              attrs: {
                type: 'module',
                src: options.entryScriptPath,
              },
            },),
            h({
              tag: 'script',
              html: MENU_OPEN_SCRIPT,
            },),
          ],
        },),
      ],
    },)
  }`;

  return new Response(
    html,
    {
      headers: { 'Content-Type': 'text/html; charset=utf-8', },
    },
  );
}
