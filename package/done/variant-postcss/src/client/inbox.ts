/**
 Client entry script for the Inbox page.
 
 Loaded by the browser as `\<script type="module" src="/dist/client/inbox.js"\>`.
 
 Hydration flow:
 1. `injectCSS()` inserts the compiled global stylesheet into `\<head\>`
 2. `readPageData()` deserializes the `\<script id="page-data"\>` JSON blob
 3. The script builds DOM elements via `h()` and appends them to `\<main id="app"\>`
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import styles from '../../dist/css/styles.css' with { type: 'text', };
import type {
  BlockedTasksByBlocker,
  Task,
} from '../lib/types.ts';
import { inboxStyles, } from './inbox-styles.ts';
import { buildSuggestedSection, } from './inbox-suggested.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import { createTaskCard, } from './lib/task-card.ts';
import { createNewTaskDialog, } from './new-task-dialog.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/top-nav.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/section-heading.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/toggle-switch.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/focus-dropdown.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/fab-button.ts';

/**
 Shape of the JSON blob embedded in the inbox page by the server.
 */
type InboxPageData = {
  /**
   Tasks suggested based on context.
   */
  suggestedTasks: Task[];
  /**
   All inbox tasks.
   */
  allTasks: Task[];
  /**
   Blocked tasks grouped by their blocker task ID.
   */
  blockedTasksByBlocker: BlockedTasksByBlocker;
};

injectCSS(styles,);
injectCSS(inboxStyles,);

/**
 Deserialized page data from the server-rendered JSON blob.
 */
const pageData = readPageData<InboxPageData>();

/**
 Root app container element where client-rendered content is appended.
 */
const appElement = document.querySelector<HTMLElement>('#app',);
if (!(appElement instanceof HTMLElement))
  throw new Error('Missing app element',);

/**
 Typed reference to the app container.
 */
const app = appElement;

/**
 Navigates to the task detail page for the given task.
 
 @param taskId - UUID of the task to open
 */
function openTask(taskId: string,): void {
  globalThis.location
    .href = `/tasks/${taskId}`;
}

/**
 Sends a complete-task API call and reloads the page on success.
 
 @param taskId - UUID of the task to complete
 
 @example
 ```ts
 await completeTask('uuid-123');
 ```
 */
async function completeTask(taskId: string,): Promise<void> {
  await api({
    path: `/api/tasks/${taskId}/complete`,
    options: { method: 'POST', },
  },);
  globalThis.location
    .reload();
}

/**
 Builds a task list with optional blocked-child nesting.
 
 @param tasks - Tasks to display
 
 @param blockedTasksByBlocker - Map of blocker ID to blocked task links
 
 @returns Unordered list element containing task cards
 
 @example
 ```ts
 const list = buildTaskList({ tasks: pageData.allTasks, blockedTasksByBlocker: pageData.blockedTasksByBlocker });
 app.append(list);
 ```
 */
function buildTaskList(
  {
    tasks,
    blockedTasksByBlocker,
  }: {
    readonly tasks: readonly Task[];
    readonly blockedTasksByBlocker: BlockedTasksByBlocker;
  },
): HTMLUListElement {
  /**
   Top-level list element; child tasks are appended as nested sub-lists.
   */
  const list = h({
    tag: 'ul',
    class: 'task-list',
  },);
  for (const task of tasks) {
    list.append(
      createTaskCard({
        task,
        options: {
          onOpen: openTask,
          onToggleComplete: completeTask,
        },
      },),
    );
    /**
     Blocked-by children for the current task, defaulted to empty when none exist.
     */
    const childLinks = blockedTasksByBlocker[task.id]
      ?? [];
    if (childLinks.length
      > 0) {
      list.append(h({
        tag: 'div',
        class: 'task-children',
        children: [h({
          tag: 'ul',
          class: 'task-list',
          children: childLinks.map(function createBlockedCard(childLink,) {
            return createTaskCard({
              task: childLink.task,
              options: {
                showBlockedBadge: true,
                onOpen: openTask,
                onToggleComplete: completeTask,
              },
            },);
          },),
        },),],
      },),);
    }
  }
  return list;
}

app.append(buildSuggestedSection({
  suggestedTasks: pageData.suggestedTasks,
  blockedTasksByBlocker: pageData.blockedTasksByBlocker,
  buildTaskList,
},),);

app.append(h({
  tag: 'div',
  class: 'divider',
},),);

/**
 Collapsible section heading for all tasks.
 */
const allSection = h({
  tag: 'section-heading',
  attrs: {
    icon: '\u221E',
    label: 'All',
  },
},);
allSection.append(h({
  tag: 'div',
  style: {
    display: 'flex',
    flexDirection: 'column',
    gap: 'var(--gap)',
  },
  children: [
    pageData.allTasks
      .length
      === 0
      ? h({
        tag: 'p',
        class: 'empty',
        text: 'No tasks yet.',
      },)
      : buildTaskList({
        tasks: pageData.allTasks,
        blockedTasksByBlocker: pageData.blockedTasksByBlocker,
      },),
  ],
},),);
app.append(allSection,);

/**
 New-task panel and FAB button created by the dialog module.
 */
const {
  panel: newTaskPanel,
  fab: newTaskFab,
} = createNewTaskDialog();
document.body
  .append(newTaskPanel,);
document.body
  .append(newTaskFab,);
