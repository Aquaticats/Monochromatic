/**
 Client entry script for the Inbox page.
 
 Loaded by the browser as `<script type="module" src="/dist/client/inbox.js">`.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  buildTaskList,
  type InboxPageData,
} from './inbox-builders.ts';
import { inboxStyles, } from './inbox-styles.ts';
import { buildSuggestedSection, } from './inbox-suggested.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import { createNewTaskDialog, } from './new-task-dialog.ts';
import { globalStyles, } from './styles.ts';
// Side-effect imports: register custom elements so the browser recognizes them in the DOM
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

injectCSS(globalStyles,);
injectCSS(inboxStyles,);

/**
 Deserialized page data containing suggested and all inbox tasks.
 */
const pageData = readPageData<InboxPageData>();

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

/**
 Navigates to the task detail page for the given task.
 
 @param taskId - ID of task to open
 */
function openTask(taskId: string,): void {
  globalThis.location
    .href = `/tasks/${taskId}`;
}

/**
 Sends a complete-task API call via {@link api} and reloads the page on success.
 
 @param taskId - ID of task to complete
 */
async function completeTask(taskId: string,): Promise<void> {
  await api({
    path: `/api/tasks/${taskId}/complete`,
    options: { method: 'POST', },
  },);
  globalThis.location
    .reload();
}

//region Suggested section

app.append(
  buildSuggestedSection({
    pageData,
    onOpen: openTask,
    onComplete: completeTask,
  },),
);

//endregion Suggested section

app.append(h({
  tag: 'div',
  class: 'divider',
},),);

//region All section

/**
 Collapsible section heading for the "All" tasks block.
 */
const allSection = h({
  tag: 'section-heading',
  attrs: {
    icon: '\u221E',
    label: 'All',
  },
},);

/**
 Content container for the all tasks section.
 */
const allContent = h({
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
        onOpen: openTask,
        onToggleComplete: completeTask,
      },),
  ],
},);

allSection.append(allContent,);
app.append(allSection,);

//endregion All section

//region New-task dialog (FAB opens a modal <dialog> with task-detail in create mode)

/**
 New-task dialog panel and trigger FAB button.
 */
const {
  panel: newTaskPanel,
  fab: newTaskFab,
} = createNewTaskDialog();
document.body
  .append(newTaskPanel,);
document.body
  .append(newTaskFab,);

//endregion New-task dialog
