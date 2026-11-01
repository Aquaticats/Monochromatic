/**
 Builds the "Suggested" section DOM for the inbox page.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  buildTaskList,
  type InboxPageData,
} from './inbox-builders.ts';

/**
 Creates the suggested-tasks section with location/focus controls and a
 {@link buildTaskList} task list.
 
 @param pageData - Deserialized inbox page data
 
 @param onOpen - Callback to navigate to task detail
 
 @param onComplete - Callback to complete a task
 
 @returns Section heading element containing the suggested tasks UI
 
 @example
 ```ts
 const section = buildSuggestedSection({ pageData, onOpen: handleOpen, onComplete: handleComplete });
 app.prepend(section);
 ```
 */
export function buildSuggestedSection({
  pageData,
  onOpen,
  onComplete,
}: {
  readonly pageData: InboxPageData;
  readonly onOpen: (taskId: string,) => void;
  readonly onComplete: (taskId: string,) => Promise<void>;
},): HTMLElement {
  /**
   Collapsible section heading for the suggested tasks block.
   */
  const suggestedSection = h({
    tag: 'section-heading',
    attrs: {
      icon: '\u2728',
      label: 'Suggested',
    },
  },);

  /**
   Content container for the suggested tasks section.
   */
  const suggestedContent = h({
    tag: 'div',
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 'var(--gap)',
    },
    children: [
      h({
        tag: 'div',
        class: 'controls',
        children: [
          h({
            tag: 'div',
            class: 'control-group',
            children: [
              h({
                tag: 'h3',
                class: 'subsection-heading',
                text: 'My location',
              },),
              h({
                tag: 'p',
                class: 'subsection-desc',
                text: 'Suggesting tasks can be done near the location.',
              },),
              h({
                tag: 'div',
                class: 'location-options',
                children: [
                  h({
                    tag: 'button',
                    class: 'autodetect-toggle',
                    children: [
                      h({
                        tag: 'span',
                        text: 'autodetect',
                      },),
                      h({
                        tag: 'toggle-switch',
                        attrs: { on: '', },
                      },),
                    ],
                  },),
                ],
              },),
            ],
          },),
          h({
            tag: 'div',
            class: 'control-group',
            children: [
              h({
                tag: 'h3',
                class: 'subsection-heading',
                text: 'My focus',
              },),
              h({
                tag: 'p',
                class: 'subsection-desc',
                text: 'Additional instructions on which tasks to suggest.',
              },),
              h({
                tag: 'focus-dropdown',
                attrs: { value: 'Adulting tasks first', },
              },),
            ],
          },),
        ],
      },),
      pageData.suggestedTasks
        .length
        === 0
        ? h({
          tag: 'p',
          class: 'empty',
          text: 'No tasks yet.',
        },)
        : buildTaskList({
          tasks: pageData.suggestedTasks,
          blockedTasksByBlocker: pageData.blockedTasksByBlocker,
          onOpen,
          onToggleComplete: onComplete,
        },),
    ],
  },);

  suggestedSection.append(suggestedContent,);
  return suggestedSection;
}
