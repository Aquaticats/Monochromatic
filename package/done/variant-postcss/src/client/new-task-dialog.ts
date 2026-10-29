/**
 New-task panel module for the Inbox page.
 
 The FAB button transforms into a fixed panel containing `\<task-detail\>`
 in create mode. Uses the Popover API for top-layer stacking without a
 blocking backdrop, so the side-drawer remains visible and interactive.
 
 Exceeds 100 lines: the blank task template (18 fields), event listener
 with save/close branches, and panel open/close logic form a single cohesive
 unit; splitting further would scatter the lifecycle across files.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../lib/types.ts';
import { METADATA_UNSET, } from './component/task-detail-types.ts';
import type { TaskDetail, } from './component/task-detail.ts';
import { api, } from './lib/api.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: registers the task-detail custom element
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
  /* oxlint-disable typescript/no-unsafe-type-assertion -- custom element registered as "task-detail" returns TaskDetail */
  /**
   Detail web component captured so the action handler and open flow can configure it.
   */
  const detail = document.createElement('task-detail',) as TaskDetail;
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  /**
   Popover panel wrapping the detail component; toggled via `showPopover`/`hidePopover`.
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
   FAB element returned to the caller and toggled hidden when the panel is open.
   */
  const fab = h({
    tag: 'fab-button',
    attrs: { label: 'Add task', },
  },);

  /**
   Hides the panel popover and restores the FAB.
   */
  function closePanel(): void {
    panel.hidePopover();
    fab.hidden = false;
  }

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

          if (action === 'close') {
            closePanel();
            return;
          }

          if (action === 'save') {
            /**
             Title with leading/trailing whitespace stripped; empty trim aborts the save.
             */
            const trimmedTitle = title.trim();
            if (trimmedTitle.length
              === 0)
              return;

            /**
             Current metadata snapshot from the detail component, included in the POST body.
             */
            const metadata = detail.getMetadata();
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
                  priority: metadata.priority
                    === METADATA_UNSET ? undefined : metadata.priority,
                  complexity: metadata.complexity
                    === METADATA_UNSET ? undefined : metadata.complexity,
                },),
              },
            },);
            globalThis.location
              .reload();
          }
        }
        catch (error: unknown) {
          console.error(
            'new task action handler failed',
            error,
          );
        }
      })();
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
    requestAnimationFrame(function focusTitleInput() {
      panel.dataset
        .animating = '';
      /**
       Title input from inside the detail's shadow root; focused after the expand frame.
       */
      const titleInput = detail.shadowRoot
        ?.querySelector<HTMLInputElement>(
        '.title-input',
        );
      titleInput?.focus();
    },);
  }

  fab.addEventListener(
    'click',
    openPanel,
  );

  return {
    panel,
    fab,
  };
}
