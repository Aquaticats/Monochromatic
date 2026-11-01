/**
 Client entry script for the In-Progress page.
 
 Same hydration pattern as inbox.ts: injectCSS -\> readPageData -\> build DOM into #app.
 Additionally runs a 1-second interval to live-update tracked-time chip text.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import styles from '../../dist/css/styles.css' with { type: 'text', };
import type { Task, } from '../lib/types.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import {
  createTaskCard,
  formatRunningTrackedTime,
} from './lib/task-card.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/top-nav.ts';

/**
 Shape of the JSON blob embedded in the in-progress page by the server.
 */
type InProgressPageData = {
  /**
   Active in-progress tasks with running timers.
   */
  tasks: Task[];
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
 Stops a running timer on a task, then reloads to reflect the new state.
 
 @param taskId - UUID of the task whose timer to stop
 */
async function stopTimer(taskId: string,): Promise<void> {
  await api({
    path: `/api/tasks/${taskId}/stop`,
    options: { method: 'POST', },
  },);
  globalThis.location
    .reload();
}

injectCSS(styles,);

/**
 Deserialized page data from the server-rendered JSON blob.
 */
const pageData = readPageData<InProgressPageData>();

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

if (pageData.tasks
  .length
  === 0) {
  app.append(h({
    tag: 'p',
    class: 'empty',
    text: 'No active timers.',
  },),);
}

/**
 Task card list for in-progress tasks.
 */
const list = h({
  tag: 'ul',
  class: 'task-list',
},);

for (const task of pageData.tasks) {
  list.append(
    createTaskCard({
      task,
      options: {
        onOpen: openTask,
        onToggleComplete: stopTimer,
      },
    },),
  );
}

if (pageData.tasks
  .length
  > 0)
  app.append(list,);

/**
 Timer update interval in milliseconds.
 */
const TIMER_UPDATE_MS = 1_000;

// Live timer updates: correlate each card with its task by DOM order
setInterval(
  function updateTimers() {
    /**
     Current set of task-card elements; recomputed each tick in case the list mutates.
     */
    const cards = list.querySelectorAll<HTMLElement>('task-card',);
    cards.forEach(function updateCard(
      card,
      cardIndex,
    ) {
      /**
       Task data aligned to the card by DOM order; `undefined` triggers an early return.
       */
      const task = pageData.tasks[cardIndex];
      if (task === undefined)
        return;
      /* oxlint-disable typescript/no-unsafe-type-assertion -- custom element has getChipElement method */
      /**
       Tracked-time chip on the task-card; narrowed via `instanceof` below since the card may not have rendered chips.
       */
      const chipEl: unknown = (card as unknown as {
        getChipElement?: (prefix: string,) => unknown;
      })
        .getChipElement?.('tracked:',);
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      if (chipEl instanceof HTMLSpanElement)
        chipEl.textContent = `tracked: ${formatRunningTrackedTime(task,)}`;
    },);
  },
  TIMER_UPDATE_MS,
);
