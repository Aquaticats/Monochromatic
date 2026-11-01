/**
 Client entry script for the In-Progress page.
 
 Same hydration pattern as inbox.ts: injectCSS -> readPageData -> build DOM into #app.
 Additionally runs a 1-second interval to live-update tracked-time chip text.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../lib/types.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import {
  createTaskCard,
  formatRunningTrackedTime,
} from './lib/task-card.ts';
import { globalStyles, } from './styles.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/top-nav.ts';

/**
 Shape of the JSON blob embedded in the in-progress page by the server.
 */
type InProgressPageData = {
  tasks: Task[];
};

/**
 Timer tick interval in milliseconds.
 */
const TIMER_INTERVAL_MS = 1_000;

/**
 Navigates to the task detail page.
 
 @param taskId - ID of task to open
 */
function handleOpen(taskId: string,): void {
  globalThis.location
    .href = `/tasks/${taskId}`;
}

/**
 Stops a task's timer via {@link api}, then reloads to reflect the change.
 
 @param taskId - ID of task whose timer to stop
 */
async function handleStop(taskId: string,): Promise<void> {
  await api({
    path: `/api/tasks/${taskId}/stop`,
    options: { method: 'POST', },
  },);
  globalThis.location
    .reload();
}

injectCSS(globalStyles,);

/**
 Deserialized page data containing in-progress tasks.
 */
const pageData = readPageData<InProgressPageData>();

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
 UL container for in-progress task cards.
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
        onOpen: handleOpen,
        onToggleComplete: handleStop,
      },
    },),
  );
}

if (pageData.tasks
  .length
  > 0)
  app.append(list,);

// Live timer updates: correlate each card with its task by DOM order
setInterval(
  function updateTimers() {
    /**
     Fresh query each tick so newly appended cards participate in the update loop.
     */
    const cards = list.querySelectorAll<HTMLElement>('task-card',);
    cards.forEach(function updateCard(
      card,
      cardIndex,
    ) {
      /**
       Card-to-task correlation by DOM order; index may overshoot during reloads.
       */
      const task = pageData.tasks[cardIndex];
      if (task === undefined)
        return;
      /* oxlint-disable typescript/no-unsafe-type-assertion -- TaskCard has getChipElement but querySelectorAll returns generic HTMLElement */
      /**
       Optional chip lookup typed as `unknown`; narrowed by the `instanceof` check below.
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
  TIMER_INTERVAL_MS,
);
