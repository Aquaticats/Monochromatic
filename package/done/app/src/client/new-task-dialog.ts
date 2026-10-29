/**
 New-task panel module for the Inbox page.
 
 The FAB button transforms into a fixed panel containing `<task-detail>`
 in create mode. Uses the Popover API for top-layer stacking without a
 blocking backdrop, so the side-drawer remains visible and interactive.
 
 Exceeds 100 lines: the blank task template (18 fields), event listener
 with save/close branches, and panel open/close logic form a single cohesive
 unit; splitting further would scatter the lifecycle across files.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../lib/types.ts';
import type { TaskDetail, } from './component/task-detail.ts';
import { api, } from './lib/api.ts';
// Side-effect import: registers the `<task-detail>` custom element
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/task-detail.ts';

/**
 Blank task template used when creating a new task; optional fields stay absent.
 */
const emptyTask: Task = {
  id: '',
  title: '',
  tags: [],
  locations: [],
  reminders: [],
  blockedBy: [],
  trackedTime: 0,
  status: 'inbox',
  source: 'local',
  createdAt: '',
  updatedAt: '',
};

/**
 Return value of {@link createNewTaskDialog}.
 */
type NewTaskDialog = {
  /**
   Fixed panel element to append to the document body.
   */
  panel: HTMLElement;
  /**
   FAB button element to append to the document body.
   */
  fab: HTMLElement;
};

/**
 Builds the new-task panel and its trigger FAB button.
 
 Clicking the FAB hides it and reveals a fixed panel in the same
 bottom-right region. Closing or saving collapses the panel and
 restores the FAB. The panel uses `popover="manual"` for top-layer
 stacking without a blocking backdrop.
 
 @returns panel and fab elements ready for DOM insertion
 
 @example
 ```ts
 const { panel, fab } = createNewTaskDialog();
 document.body.append(panel, fab);
 ```
 */
export function createNewTaskDialog(): NewTaskDialog {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- createElement returns HTMLElement but task-detail is registered as TaskDetail */
  /**
   Live instance of the custom element so its imperative API stays accessible.
   */
  const detail = document.createElement('task-detail',) as TaskDetail;
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  /**
   Popover host containing the detail panel; toggled by `openPanel`/`closePanel`.
   */
  const panel = h({
    tag: 'div',
    class: 'new-task-panel',
  },);
  panel.setAttribute(
    'popover',
    'manual',
  );
  panel.append(detail,);

  /**
   Hides the panel popover and restores the FAB.
   */
  function closePanel(): void {
    panel.hidePopover();
    fab.hidden = false;
  }

  detail.addEventListener(
    'action',
    function handleAction(event,) {
      if (!(event instanceof CustomEvent))
        throw new TypeError("Expected CustomEvent for 'action' listener",);
      /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- event.detail shape is controlled by the task-detail component */
      /**
       Destructured action payload dispatched by the inner `<task-detail>`.
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

      if (action === 'close') {
        closePanel();
        return;
      }

      if (action === 'save') {
        /**
         Trimmed title captured once; empty titles short-circuit the save below.
         */
        const trimmedTitle = title.trim();
        if (trimmedTitle.length
          === 0)
          return;

        /**
         Snapshot of the autofill/manual metadata, forwarded to the create endpoint.
         */
        const metadata = detail.getMetadata();
        void (async function saveTask(): Promise<void> {
          await api({
            path: '/api/tasks',
            options: {
              method: 'POST',
              body: JSON.stringify({
                title: trimmedTitle,
                description: description.length
                  === 0 ? null : description,
                tags: metadata.tags,
                locations: metadata.locations,
                priority: metadata.priority,
                complexity: metadata.complexity,
              },),
            },
          },);
          globalThis.location
            .reload();
        })();
      }
    },
  );

  /**
   Opens the panel with a fresh empty task, hiding the FAB and playing the expand animation.
   */
  function openPanel(): void {
    console.log(
      '[new-task-dialog] openPanel(), detail.configure is:',
      typeof detail.configure,
    );
    detail.configure({
      task: emptyTask,
      blockerSummaries: [],
      mode: 'create',
    },);
    fab.hidden = true;

    // Restart the expand animation by toggling the data attribute
    delete panel.dataset
      .animating;
    panel.showPopover();
    requestAnimationFrame(function animatePanel() {
      panel.dataset
        .animating = '';
      /**
       Looked up after the panel opens so the autofocus lands on the right input.
       */
      const titleInput =
        detail.shadowRoot
          ?.querySelector<HTMLInputElement>('.title-input',)
          ?? null;
      titleInput?.focus();
    },);
  }

  /**
   Floating action button referenced by `openPanel`/`closePanel` to toggle visibility.
   */
  const fab = h({
    tag: 'fab-button',
    attrs: { label: 'Add task', },
    on: { click: openPanel, },
  },);

  return {
    panel,
    fab,
  };
}
