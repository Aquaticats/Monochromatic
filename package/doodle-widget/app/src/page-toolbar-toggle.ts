/**
 Toggle group rendering utilities for the doodle widget toolbar.
 
 Produces radio-based toggle groups used for tool selection
 and page navigation.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Renders a single radio toggle option inside a toggle group.
 
 @param id - unique element id for the radio input
 
 @param name - shared radio group name
 
 @param label - visible label text
 
 @param checked - whether this option is initially selected
 
 @param value - radio value attribute, defaults to id
 
 @returns label-wrapped radio input HTML string
 
 @example
 ```ts
 renderToggleOption({ id: 'tool-draw', name: 'tool', label: 'Draw', checked: true });
 ```
 */
export function renderToggleOption({
  id,
  name,
  label,
  checked,
  value,
}: {
  readonly id: string;
  readonly name: string;
  readonly label: string;
  readonly checked: boolean;
  readonly value?: string;
},): string {
  return h({
    tag: 'label',
    class: 'toggle-option',
    children: [
      h({
        tag: 'input',
        attrs: {
          type: 'radio',
          name,
          id,
          value: value ?? id,
          ...(checked ? { checked: '', } : {}),
        },
      },),
      h({
        tag: 'span',
        text: label,
      },),
    ],
  },);
}

/**
 Renders the page selector toggle group.
 
 @param pageCount - number of pages to generate radio buttons for
 
 @returns page toggle group HTML string
 
 @example
 ```ts
 const html = renderPageToggle(3);
 ```
 */
export function renderPageToggle(pageCount: number,): string {
  return h({
    tag: 'div',
    class: 'toggle-group',
    attrs: { id: 'page-toggle', },
    children: Array.from(
      { length: pageCount, },
      function renderPageOption(
        _: unknown,
        index: number,
      ): string {
        return renderToggleOption({
          id: `page-${String(index,)}`,
          name: 'page',
          label: String(index + 1,),
          checked: index === 0,
          value: String(index,),
        },);
      },
    ),
  },);
}
