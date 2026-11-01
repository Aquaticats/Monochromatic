/**
 Toolbar HTML rendering for the doodle widget.
 
 Produces the top toolbar with tool selection toggle, page navigation
 toggle, drawing settings, upload, export, and clear controls.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  renderPageToggle,
  renderToggleOption,
} from './page-toolbar-toggle.ts';

/**
 Renders the toolbar with tool selection, page navigation (via
 {@link renderPageToggle}), upload, and clear controls.
 
 @param pageCount - number of pages for the page toggle group
 
 @returns toolbar HTML string
 
 @example
 ```ts
 const html = renderToolbar(2);
 ```
 */
export function renderToolbar(pageCount: number,): string {
  return h({
    tag: 'div',
    class: 'toolbar',
    children: [
      h({
        tag: 'span',
        class: 'toolbar-title',
        text: 'Doodle',
      },),
      h({
        tag: 'div',
        class: 'toggle-group',
        attrs: { id: 'tool-toggle', },
        children: [
          renderToggleOption({
            id: 'tool-draw',
            name: 'tool',
            label: 'Draw',
            checked: true,
          },),
          renderToggleOption({
            id: 'tool-erase',
            name: 'tool',
            label: 'Erase',
            checked: false,
          },),
          renderToggleOption({
            id: 'tool-text',
            name: 'tool',
            label: 'Text',
            checked: false,
          },),
          renderToggleOption({
            id: 'tool-zoom',
            name: 'tool',
            label: 'Zoom',
            checked: false,
          },),
        ],
      },),
      h({
        tag: 'div',
        class: 'draw-settings',
        children: [
          h({
            tag: 'input',
            attrs: {
              type: 'color',
              id: 'color-picker',
              value: '#c24e2e',
            },
          },),
          h({
            tag: 'input',
            attrs: {
              type: 'range',
              id: 'size-slider',
              min: '1',
              max: '50',
              value: '10',
            },
          },),
        ],
      },),
      h({
        tag: 'div',
        class: 'undo-group',
        children: [
          h({
            tag: 'button',
            attrs: {
              id: 'undo-btn',
              type: 'button',
              disabled: '',
            },
            text: 'Undo',
          },),
          h({
            tag: 'button',
            attrs: {
              id: 'redo-btn',
              type: 'button',
              disabled: '',
            },
            text: 'Redo',
          },),
        ],
      },),
      renderPageToggle(pageCount,),
      h({
        tag: 'button',
        attrs: {
          id: 'upload-btn',
          type: 'button',
          // Upload background button hidden by default because it's not needed for the upcoming demo.
          hidden: '',
        },
        text: 'Upload background',
      },),
      h({
        tag: 'input',
        attrs: {
          type: 'file',
          id: 'upload-input',
          accept: '.svg,image/svg+xml',
          hidden: '',
        },
      },),
      h({
        tag: 'div',
        class: 'export-group',
        children: [
          h({
            tag: 'button',
            attrs: {
              id: 'export-btn',
              type: 'button',
            },
            text: 'Export',
          },),
          h({
            tag: 'select',
            attrs: { id: 'format-select', },
            children: [
              h({
                tag: 'option',
                attrs: {
                  value: 'pdf',
                  selected: '',
                },
                text: 'PDF',
              },),
              h({
                tag: 'option',
                attrs: { value: 'svg', },
                text: 'SVG',
              },),
              h({
                tag: 'option',
                attrs: { value: 'png', },
                text: 'PNG',
              },),
            ],
          },),
        ],
      },),
      h({
        tag: 'button',
        attrs: {
          id: 'clear-btn',
          type: 'button',
        },
        text: 'Clear',
      },),
    ],
  },);
}
