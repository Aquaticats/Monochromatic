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
} from '../../lib/db/tasks-queries.ts';
import { serializePageData, } from './layout.ts';

/**
 Renders the search page with FTS results from {@link searchTasks} when a
 query is present, plus quick-pick tags from {@link listAllTags}, serialized
 for the client via {@link serializePageData}.
 
 @param url - Request URL (the `q` search param contains the query)
 
 @returns HTML response for the search page
 
 @example
 ```ts
 const response = await searchPage(new URL('http://localhost/search?q=groceries'));
 ```
 */
export async function searchPage(url: URL,): Promise<Response> {
  /**
   URL search-param query text; empty string when omitted.
   */
  const query = url.searchParams
    .get('q',)
    ?? '';
  /**
   FTS results for the query; empty when the query is blank.
   */
  const results = await searchTasks(query,);
  /**
   All known tags surfaced as quick-pick chips on the empty-query screen.
   */
  const availableTags = await listAllTags();

  /**
   Bundled payload serialised into the embedded `#page-data` script.
   */
  const pageData = {
    query,
    results,
    availableTags,
  };

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
