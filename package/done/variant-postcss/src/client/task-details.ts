/**
 Client entry script for the Task Detail page.
 
 Same hydration pattern as inbox.ts: injectCSS -\> readPageData -\> build DOM into #app.
 The server renders its own HTML shell (not via renderPage) without `\<top-nav\>`,
 because the `\<task-detail\>` component provides its own back-button header.
 */
import styles from '../../dist/css/styles.css' with { type: 'text', };
import type { Task, } from '../lib/types.ts';
import { api, } from './lib/api.ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
import type { TaskDetail, } from './component/task-detail.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: registers the task-detail custom element
import './component/task-detail.ts';

/**
 Minimal task info shown in the blocker picker dropdown.
 */
type BlockerCandidate = {
  /**
   UUID of the candidate blocker task.
   */
  id: string;
  /**
   Title of the candidate blocker task.
   */
  title: string;
};

/**
 Summary of a task that blocks the current task (shown as a chip/badge).
 */
type BlockerSummary = {
  /**
   UUID of the blocking task.
   */
  id: string;
  /**
   Title of the blocking task.
   */
  title: string;
  /**
   Current status of the blocking task.
   */
  status: string;
};

/**
 Shape of the JSON blob embedded in the task detail page by the server.
 */
type TaskDetailsPageData = {
  /**
   Task being viewed/edited.
   */
  task: Task;
  /**
   Available tasks that could be selected as blockers.
   */
  blockerCandidates: BlockerCandidate[];
  /**
   Current blockers for this task.
   */
  blockerSummaries: BlockerSummary[];
};

injectCSS(styles,);

/**
 Deserialized page data from the server-rendered JSON blob.
 */
const pageData = readPageData<TaskDetailsPageData>();
/**
 Task being viewed, extracted from the page data blob.
 */
const { task, } = pageData;

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

/**
 Task detail web component instance for displaying/editing the task.
 */
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- custom element registered as "task-detail" returns TaskDetail
const detail = document.createElement('task-detail',) as TaskDetail;
detail.configure({
  task,
  blockerSummaries: pageData.blockerSummaries,
},);

detail.addEventListener(
  'action',
  function onAction(event,) {
    void (async function onActionAsync(): Promise<void> {
      try {
        if (!(event instanceof CustomEvent))
          throw new TypeError("Expected CustomEvent for 'action' listener",);
        /* oxlint-disable typescript/no-unsafe-type-assertion -- CustomEvent detail shape matches the action payload */
        /**
         Destructured action payload from the `action` custom event detail.
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
        /* oxlint-enable typescript/no-unsafe-type-assertion */

        if (action === 'close')
          globalThis.location
            .href = '/';
        else if (action === 'save') {
          /**
           Current metadata snapshot from the detail component for the PUT body.
           */
          const metadata = detail.getMetadata();
          /**
           PUT body merged from form inputs and metadata.
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
          await api({
            path: `/api/tasks/${task.id}`,
            options: {
              method: 'PUT',
              body: JSON.stringify(payload,),
            },
          },);
          globalThis.location
            .reload();
        }
        else if (action === 'start') {
          await api({
            path: `/api/tasks/${task.id}/start`,
            options: { method: 'POST', },
          },);
          globalThis.location
            .reload();
        }
        else if (action === 'stop') {
          await api({
            path: `/api/tasks/${task.id}/stop`,
            options: { method: 'POST', },
          },);
          globalThis.location
            .reload();
        }
        else if (action === 'complete') {
          await api({
            path: `/api/tasks/${task.id}/complete`,
            options: { method: 'POST', },
          },);
          globalThis.location
            .href = '/';
        }
        else if (action === 'delete') {
          await api({
            path: `/api/tasks/${task.id}`,
            options: { method: 'DELETE', },
          },);
          globalThis.location
            .href = '/';
        }
      }
      catch (error: unknown) {
        console.error(
          'task detail action handler failed',
          error,
        );
      }
    })();
  },
);

app.append(detail,);
