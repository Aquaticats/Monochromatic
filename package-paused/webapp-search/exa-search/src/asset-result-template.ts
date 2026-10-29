import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Builds the search result template element rendered once and cloned by client JS.
 *
 * @returns HTML string for a single result `<li>` with all sub-elements
 *
 * @example
 * ```ts
 * const html = buildResultTemplate();
 * document.querySelector('.results')!.innerHTML = html;
 * ```
 */
export function buildResultTemplate(): string {
  return h({
    tag: 'li',
    class: 'result',
    attrs: { hidden: '', },
    children: [
      h({
        tag: 'div',
        class: 'result__header',
        children: [
          h({
            tag: 'img',
            class: 'result__favicon',
            attrs: {
              src: '',
              alt: '',
            },
          },),
          h({
            tag: 'h2',
            class: 'result__title',
            children: [
              h({
                tag: 'a',
                class: 'result__link',
                attrs: {
                  href: '#',
                  target: '_blank',
                  rel: 'noopener noreferrer',
                },
              },),
            ],
          },),
          h({
            tag: 'address',
            class: 'result__author',
            children: [
              h({
                tag: 'time',
                class: 'result__publishedDate',
              },),
            ],
          },),
        ],
      },),
      h({
        tag: 'div',
        class: 'result__body',
        children: [
          h({
            tag: 'div',
            class: 'result__texturalContent',
            children: [
              h({
                tag: 'details',
                children: [
                  h({
                    tag: 'summary',
                    children: [
                      h({
                        tag: 'p',
                        class: 'result__summary',
                      },),
                    ],
                  },),
                  h({
                    tag: 'p',
                    class: 'result__text',
                  },),
                ],
              },),
              h({
                tag: 'ul',
                class: 'result__highlights',
                children: [
                  h({
                    tag: 'li',
                    class: 'result__highlight',
                  },),
                ],
              },),
            ],
          },),
          h({
            tag: 'img',
            class: 'result__image',
            attrs: {
              src: '',
              alt: '',
            },
          },),
        ],
      },),
    ],
  },);
}
