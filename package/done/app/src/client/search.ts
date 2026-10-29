/**
 Client entry script for the Search page.
 
 Same hydration pattern as inbox.ts: injectCSS -> readPageData -> build DOM into #app.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { SearchTask, } from '../lib/types.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import { createTaskCard, } from './lib/task-card.ts';
import { searchStyles, } from './search-styles.ts';
import { globalStyles, } from './styles.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/search-bar.ts';

/**
 Shape of the JSON blob embedded in the search page by the server.
 */
type SearchPageData = {
  /**
   User's search query string.
   */
  query: string;
  /**
   Task results matching the query.
   */
  results: SearchTask[];
  /**
   All known tags for the tag chip display.
   */
  availableTags: string[];
};

/**
 Navigates to the task detail page.
 
 @param taskId - ID of task to open
 */
function handleOpen(taskId: string,): void {
  globalThis.location
    .href = `/tasks/${taskId}`;
}

/**
 Navigates to the search results for the search-bar's query.
 
 @param event - `search` CustomEvent carrying the typed query
 */
function handleSearch(event: CustomEvent<{ query: string; }>,): void {
  /**
   Search-bar query text destructured for the URL builder below.
   */
  const { query, } = event.detail;
  globalThis.location
    .href = query.length
      === 0
      ? '/search'
      : `/search?q=${encodeURIComponent(query,)}`;
}

/**
 Completes a task via {@link api}, then reloads to drop it from the results.
 
 @param taskId - ID of task to complete
 */
async function handleComplete(taskId: string,): Promise<void> {
  await api({
    path: `/api/tasks/${taskId}/complete`,
    options: { method: 'POST', },
  },);
  globalThis.location
    .reload();
}

injectCSS(globalStyles,);
injectCSS(searchStyles,);

/**
 Deserialized page data containing search query, results, and available tags.
 */
const pageData = readPageData<SearchPageData>();

/**
 Raw DOM element for the `#app` container.
 */
const appElement = document.querySelector<HTMLElement>('#app',);
if (!(appElement instanceof HTMLElement))
  throw new Error('Missing app element',);

/**
 Validated `#app` container element.
 */
const app = appElement;

// Listen for search events from the search-bar component
document.querySelector<HTMLElement>('search-bar',)
  ?.addEventListener(
  'search',
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- CustomEvent handler must be cast to EventListener for addEventListener
  handleSearch as EventListener,
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
   Tags surfaced as quick-pick chips when the user has not typed anything yet.
   */
  const { availableTags, } = pageData;
  if (availableTags.length
    > 0) {
    app.append(
      h({
        tag: 'div',
        class: 'tag-chips',
        children: availableTags.map(function buildTagChip(tag,) {
          return h({
            tag: 'button',
            class: 'tag-chip',
            text: `# ${tag}`,
            on: {
              click: function handleTagClick() {
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
   Container appended below; populated by iterating `pageData.results`.
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
          onOpen: handleOpen,
          onToggleComplete: handleComplete,
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
