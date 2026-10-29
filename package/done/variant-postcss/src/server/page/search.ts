/**
 Search page handler.
 
 Unlike inbox/settings/in-progress, this page renders its own HTML inline
 instead of calling `renderPage()`, because it replaces the `<top-nav>` with
 a `<search-bar>` component at the top of the shell.
 
 Client entry: `/dist/client/search.js` (src/client/search.ts)
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  listAllTags,
  searchTasks,
} from '../../lib/db/tasks.ts';
import { serializePageData, } from './layout.ts';

/**
 Renders the search page with FTS results when a query is present.
 
 @param url - Request URL (the `q` search param contains the query)
 
 @returns HTML response for the search page
 
 @example
 ```ts
 const response = await searchPage(new URL('https://example.com/search?q=test'));
 ```
 */
export async function searchPage(url: URL,): Promise<Response> {
  /**
   Query string lifted from the `?q=` parameter; defaults to empty for the landing view.
   */
  const query = url.searchParams
    .get('q',)
    ?? '';
  /**
   Search results from the FTS (or LIKE fallback) query.
   */
  const results = await searchTasks(query,);
  /**
   All tags in the database; rendered as quick-filter chips on the landing view.
   */
  const availableTags = await listAllTags();

  /**
   Serialized into the embedded JSON `<script>` for the client entry to hydrate from.
   */
  const pageData = {
    query,
    results,
    availableTags,
  };

  /**
   Rendered HTML response body; built via `hHtml` and wrapped with `<!DOCTYPE html>`.
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
              text: 'Search - Done',
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
                h({
                  tag: 'search-bar',
                  attrs: { value: query, },
                },),
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
              html: serializePageData(pageData,),
            },),
            h({
              tag: 'script',
              attrs: {
                type: 'module',
                src: '/dist/client/search.js',
              },
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
