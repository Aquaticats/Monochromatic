import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Builds the status section with metrics, processing, success, and error indicators.
 *
 * @returns HTML string for the status `<section>`
 *
 * @example
 * ```ts
 * const statusHtml = buildStatusSection();
 * ```
 */
export function buildStatusSection(): string {
  return h({
    tag: 'section',
    class: 'status',
    children: [
      h({
        tag: 'p',
        class: 'metrics',
        attrs: { hidden: '', },
        html: `${
          h({
            tag: 'span',
            class: 'number numTotalSearches',
            text: '0',
          },)
        } searches total`,
      },),
      h({
        tag: 'p',
        class: 'processing',
        attrs: { hidden: '', },
        text: 'Searching',
      },),
      h({
        tag: 'p',
        class: 'success',
        attrs: { hidden: '', },
        html: [
          h({
            tag: 'span',
            class: 'number',
          },),
          ' results in ',
          h({
            tag: 'span',
            class: 'number',
          },),
          ' costing ',
          h({
            tag: 'span',
            class: 'number costDollars',
          },),
          ' USD',
        ]
          .join('',),
      },),
      h({
        tag: 'p',
        class: 'fail',
        attrs: { hidden: '', },
        html: `Error: ${
          h({
            tag: 'span',
            class: 'message',
          },)
        }`,
      },),
    ],
  },);
}

/**
 * Builds the search form inside a `<search>` landmark element.
 *
 * @returns HTML string for the search form
 *
 * @example
 * ```ts
 * const formHtml = buildSearchForm();
 * ```
 */
export function buildSearchForm(): string {
  return h({
    tag: 'search',
    children: [
      h({
        tag: 'form',
        class: 'searchForm',
        children: [
          h({
            tag: 'label',
            attrs: {
              for: 'search',
              'data-sr-only': '',
            },
            text: 'Search with Exa',
          },),
          h({
            tag: 'input',
            attrs: {
              type: 'text',
              placeholder: 'Search with Exa',
              autocomplete: 'off',
              autofocus: '',
              id: 'search',
            },
          },),
          h({
            tag: 'button',
            class: 'search-button',
            attrs: { type: 'submit', },
            text: 'Search',
          },),
        ],
      },),
    ],
  },);
}
