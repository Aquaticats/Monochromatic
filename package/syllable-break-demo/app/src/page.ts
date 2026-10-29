/**
 HTML document structure for the syllable break demo.
 
 Uses h-html to produce a self-contained page with inlined CSS,
 JavaScript, and three comparison columns showing different
 word-breaking strategies.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Renders the input controls section: textarea for text entry
 and a slider for container width.
 
 @returns HTML string for the controls section
 */
function renderControls(): string {
  return h({
    tag: 'div',
    attrs: { class: 'controls', },
    children: [
      h({
        tag: 'label',
        attrs: { for: 'input-text', },
        text: 'Enter text to break:',
      },),
      h({
        tag: 'textarea',
        attrs: {
          id: 'input-text',
          rows: '3',
        },
      },),
      h({
        tag: 'div',
        attrs: { class: 'slider-row', },
        children: [
          h({
            tag: 'label',
            attrs: { for: 'width-slider', },
            text: 'Container width:',
          },),
          h({
            tag: 'input',
            attrs: {
              type: 'range',
              id: 'width-slider',
              min: '5',
              max: '40',
              value: '10',
            },
          },),
          h({
            tag: 'span',
            attrs: { id: 'width-label', },
            text: '10ch',
          },),
        ],
      },),
      h({
        tag: 'div',
        children: [
          h({
            tag: 'span',
            text: 'Break points (shown as dots): ',
          },),
          h({
            tag: 'span',
            attrs: {
              id: 'processed-text',
              class: 'processed-preview',
            },
          },),
        ],
      },),
    ],
  },);
}

/**
 Renders a single comparison column with a heading, note, and output box.
 
 @param heading - column title
 
 @param note - explanatory text shown below the heading
 
 @param outputId - DOM id for the output container
 
 @param extraClass - additional CSS class for the output box
 
 @returns HTML string for the column
 */
function renderColumn(
  {
    heading,
    note,
    outputId,
    extraClass,
  }: Readonly<{
    heading: string;
    note: string;
    outputId: string;
    extraClass: string;
  }>,
): string {
  return h({
    tag: 'div',
    attrs: { class: 'column', },
    children: [
      h({
        tag: 'h2',
        text: heading,
      },),
      h({
        tag: 'p',
        attrs: { class: 'note', },
        text: note,
      },),
      h({
        tag: 'div',
        attrs: {
          id: outputId,
          class: `output-box ${extraClass}`,
          lang: 'en',
        },
      },),
    ],
  },);
}

/**
 Renders the three side-by-side comparison columns, each via {@link renderColumn}.
 
 @returns HTML string for the columns section
 */
function renderColumns(): string {
  return h({
    tag: 'div',
    attrs: { class: 'columns', },
    children: [
      renderColumn({
        heading: 'JS zero-width space',
        note: 'hyphen library + ZWS at syllable boundaries',
        outputId: 'output-zws',
        extraClass: 'zws',
      },),
      renderColumn({
        heading: 'CSS hyphens: auto + hyphenate-character: ""',
        note: 'Browser dictionary only, no visible hyphen',
        outputId: 'output-hyphens-auto',
        extraClass: 'hyphens-auto',
      },),
      renderColumn({
        heading: 'Plain overflow-wrap: normal',
        note: 'No word breaking at all, text overflows',
        outputId: 'output-plain',
        extraClass: 'plain',
      },),
    ],
  },);
}

/**
 Renders the complete HTML document with all content inlined, combining
 {@link renderControls} and {@link renderColumns}.
 
 @param css - CSS stylesheet string
 
 @param js - client-side JavaScript string
 
 @returns complete HTML document string
 
 @example
 ```ts
 const html = renderPage({ css: 'body {}', js: 'console.log("ok")' });
 ```
 */
export function renderPage(
  {
    css,
    js,
  }: Readonly<{
    css: string;
    js: string;
  }>,
): string {
  return `<!DOCTYPE html>\n${
    h({
      tag: 'html',
      attrs: { lang: 'en', },
      children: [
        h({
          tag: 'head',
          children: [
            h({
              tag: 'meta',
              attrs: { charset: 'utf8', },
            },),
            h({
              tag: 'meta',
              attrs: {
                name: 'viewport',
                content: 'width=device-width, initial-scale=1',
              },
            },),
            h({
              tag: 'title',
              text: 'Syllable break demo',
            },),
            h({
              tag: 'style',
              html: css,
            },),
          ],
        },),
        h({
          tag: 'body',
          children: [
            h({
              tag: 'h1',
              text: 'Syllable-aware word breaking without visible hyphens',
            },),
            h({
              tag: 'p',
              attrs: { class: 'description', },
              text:
                'Compares three approaches: JS-inserted zero-width spaces at syllable boundaries (TeX patterns), CSS hyphens: auto with hyphenate-character: "", and plain text with no breaking.',
            },),
            renderControls(),
            renderColumns(),
            h({
              tag: 'script',
              attrs: { type: 'module', },
              html: js,
            },),
          ],
        },),
      ],
    },)
  }`;
}
