import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Builds the page header with logo and nav controls.
 *
 * @returns HTML string for the `<header>`
 *
 * @example
 * ```ts
 * const headerHtml = buildHeader();
 * ```
 */
export function buildHeader(): string {
  return h({
    tag: 'header',
    class: 'header',
    children: [
      h({
        tag: 'h1',
        class: 'logo',
        text: 'Unofficial Exa Search',
      },),
      h({
        tag: 'nav',
        class: 'headerNav',
        children: [
          h({
            tag: 'div',
            class: 'apiKey',
            children: [
              h({
                tag: 'button',
                class: 'changeApiKey',
                attrs: { type: 'button', },
                text: 'Change API Key',
              },),
              h({
                tag: 'dialog',
                class: 'setApiKey',
              },),
            ],
          },),
          h({
            tag: 'div',
            class: 'numResults',
            children: [
              h({
                tag: 'label',
                html: [
                  'request ',
                  h({
                    tag: 'input',
                    attrs: {
                      type: 'number',
                      name: 'numResults',
                      value: '10',
                      max: '100',
                      min: '1',
                    },
                  },),
                  ' results',
                ]
                  .join('',),
              },),
            ],
          },),
        ],
      },),
    ],
  },);
}
