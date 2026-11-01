/**
 Client entry script for the Search page.
 
 Same hydration pattern as inbox.ts: injectCSS -\> readPageData -\> build DOM into #app.
 The search page's HTML shell (rendered inline by the server, not via renderPage)
 places a `\<search-bar\>` above `\<main id="app"\>` instead of a `\<top-nav\>`.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import styles from '../../dist/css/styles.css' with { type: 'text', };
import type { SearchTask, } from '../lib/types.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import { createTaskCard, } from './lib/task-card.ts';
import { searchStyles, } from './search-styles.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/search-bar.ts';

/**
 Shape of the JSON blob embedded in the search page by the server.
 */
type SearchPageData = {
  /**
   Current search query string.
   */
  query: string;
  /**
   Matching tasks from search.
   */
  results: SearchTask[];
  /**
   All unique tags for category browsing.
   */
  availableTags: string[];
};

/**
 Navigates to the task detail page.
 
 @param taskId - UUID of the task to open
 */
function openTask(taskId: string,): void {
  globalThis.location
    .href = `/tasks/${taskId}`;
}

/**
 Completes a task, then reloads to drop it from the results.
 
 @param taskId - UUID of the task to complete
 */
async function completeTask(taskId: string,): Promise<void> {
  await api({
    path: `/api/tasks/${taskId}/complete`,
    options: { method: 'POST', },
  },);
  globalThis.location
    .reload();
}

injectCSS(styles,);
injectCSS(searchStyles,);

/**
 Deserialized page data from the server-rendered JSON blob.
 */
const pageData = readPageData<SearchPageData>();

/**
 Root app container element.
 */
const appElement = document.querySelector<HTMLElement>('#app',);
if (!(appElement instanceof HTMLElement))
  throw new Error('Missing app element',);

/**
 Typed reference to the app container.
 */
const app = appElement;

// Listen for search events from the search-bar component
document
  .querySelector<HTMLElement>('search-bar',)
  ?.addEventListener(
  'search',
  function onSearch(event,) {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- CustomEvent detail contains query string */
    /**
     Search query lifted from the custom event detail.
     */
    const { query, } = (event as CustomEvent<{ query: string; }>).detail;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    globalThis.location
      .href = query.length
        === 0
        ? '/search'
        : `/search?q=${encodeURIComponent(query,)}`;
  } as EventListener,
  );

if (pageData.query
  .length
  === 0) {
  app.append(
    h({
      tag: 'p',
      class: 'search-hint',
      text: 'Type something...or select one of the categories.',
    },),
  );

  /**
   Tag chip list extracted from `pageData`; rendered only when non-empty.
   */
  const { availableTags, } = pageData;
  if (availableTags.length
    > 0) {
    app.append(
      h({
        tag: 'div',
        class: 'tag-chips',
        children: availableTags.map(function createTagChip(tag,) {
          return h({
            tag: 'button',
            class: 'tag-chip',
            text: `# ${tag}`,
            on: {
              click: function onTagClick(): void {
                globalThis.location
                  .href = `/search?q=${encodeURIComponent(`#${tag}`,)}`;
              },
            },
          },);
        },),
      },),
    );
  }
}
else {
  /**
   List element for search results.
   */
  const resultList = h({
    tag: 'ul',
    class: 'task-list',
  },);

  for (const result of pageData.results) {
    resultList.append(
      createTaskCard({
        task: result,
        options: {
          showBlockedBadge: result.isBlocked,
          onOpen: openTask,
          onToggleComplete: completeTask,
        },
      },),
    );
  }

  if (pageData.results
    .length
    === 0) {
    app.append(h({
      tag: 'p',
      class: 'empty',
      text: 'No matching tasks.',
    },),);
  }
  else {
    app.append(resultList,);
  }
}
