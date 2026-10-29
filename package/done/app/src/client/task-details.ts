/**
 Client entry script for the Task Detail page.
 
 Same hydration pattern as inbox.ts: injectCSS -> readPageData -> build DOM into #app.
 The server renders its own HTML shell (not via renderPage) without `<top-nav>`,
 because the `<task-detail>` component provides its own back-button header.
 */
import type { Task, } from '../lib/types.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import { globalStyles, } from './styles.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
import type { TaskDetail, } from './component/task-detail.ts';
// Side-effect import: registers the `<task-detail>` custom element
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/task-detail.ts';

/**
 Minimal task info shown in the blocker picker dropdown.
 */
type BlockerCandidate = {
  id: string;
  title: string;
};

/**
 Summary of a task that blocks the current task (shown as a chip/badge).
 */
type BlockerSummary = {
  id: string;
  title: string;
  status: string;
};

/**
 Shape of the JSON blob embedded in the task detail page by the server.
 */
type TaskDetailsPageData = {
  task: Task;
  blockerCandidates: BlockerCandidate[];
  blockerSummaries: BlockerSummary[];
};

injectCSS(globalStyles,);

/**
 Deserialized page data containing task, blocker candidates, and summaries.
 */
const pageData = readPageData<TaskDetailsPageData>();

/**
 Task record from the deserialized page data.
 */
const { task, } = pageData;

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
 Task detail web component configured with server-provided data.
 */
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- createElement returns HTMLElement but task-detail is registered as TaskDetail
const detail = document.createElement('task-detail',) as TaskDetail;
detail.configure({
  task,
  blockerSummaries: pageData.blockerSummaries,
},);

detail.addEventListener(
  'action',
  function handleAction(event,) {
    if (!(event instanceof CustomEvent))
      throw new TypeError("Expected CustomEvent for 'action' listener",);
    /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- event.detail shape is controlled by the task-detail component */
    /**
     Destructured action payload dispatched by the embedded `<task-detail>`.
     */
    const {
      action,
      title,
      description,
    } = event.detail as {
      action: string;
      title: string;
      description: string;
    };
    /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */

    if (action === 'close')
      globalThis.location
        .href = '/';
    else if (action === 'save') {
      /**
       Snapshot of the autofill/manual metadata fields captured before the request body is built.
       */
      const metadata = detail.getMetadata();
      /**
       Body forwarded to the PUT endpoint; merges metadata with the unchanged base task fields.
       */
      const payload = {
        title,
        description: description.length
          === 0 ? null : description,
        tags: metadata.tags,
        locations: metadata.locations,
        priority: metadata.priority,
        complexity: metadata.complexity,
        dueDate: task.dueDate,
        blockedBy: task.blockedBy,
      };
      void (async function saveTask(): Promise<void> {
        await api({
          path: `/api/tasks/${task.id}`,
          options: {
            method: 'PUT',
            body: JSON.stringify(payload,),
          },
        },);
        globalThis.location
          .reload();
      })();
    }
    else if (action === 'start') {
      void (async function startTask(): Promise<void> {
        await api({
          path: `/api/tasks/${task.id}/start`,
          options: { method: 'POST', },
        },);
        globalThis.location
          .reload();
      })();
    }
    else if (action === 'stop') {
      void (async function stopTask(): Promise<void> {
        await api({
          path: `/api/tasks/${task.id}/stop`,
          options: { method: 'POST', },
        },);
        globalThis.location
          .reload();
      })();
    }
    else if (action === 'complete') {
      void (async function completeTask(): Promise<void> {
        await api({
          path: `/api/tasks/${task.id}/complete`,
          options: { method: 'POST', },
        },);
        globalThis.location
          .href = '/';
      })();
    }
    else if (action === 'delete') {
      void (async function deleteTask(): Promise<void> {
        await api({
          path: `/api/tasks/${task.id}`,
          options: { method: 'DELETE', },
        },);
        globalThis.location
          .href = '/';
      })();
    }
  },
);

app.append(detail,);
