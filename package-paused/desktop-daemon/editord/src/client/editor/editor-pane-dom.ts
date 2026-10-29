/**
 * DOM helper functions for `EditorPane`.
 *
 * Extracted from the class to keep `editor-pane.ts` under the max-lines limit.
 * Handles editor element creation, text content serialization, and scrolling.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

//region Editor element creation

/**
 * Creates the contenteditable editor div with a paste-to-plain-text handler.
 *
 * @returns editor div element ready to be inserted into the shadow DOM
 *
 * @example
 * ```ts
 * const result = createEditorElement();
 * ```
 */
export function createEditorElement(): HTMLDivElement {
  /**
   * Container element with paste-listener wired below.
   */
  const editor = h({
    tag: 'div',
    class: 'editor',
    attrs: {
      contenteditable: 'true',
      spellcheck: 'false',
    },
  },);
  editor.addEventListener(
    'paste',
    function handlePaste(event,) {
      event.preventDefault();
      // oxlint-disable-next-line typescript-eslint/no-deprecated -- execCommand is the only way to insert text while preserving the browser's native undo stack
      document.execCommand(
        'insertText',
        false,
        event.clipboardData
          ?.getData('text/plain',)
          ?? '',
      );
    },
  );
  return editor;
}

//endregion Editor element creation

//region Text content

/**
 * Replaces the editor's children with one div per line.
 *
 * @param editor - contenteditable container element
 *
 * @param text - full file content to render
 *
 * @example
 * ```ts
 * setTextContent({ editor: editor, text: 'const x = 42;', });
 * ```
 */
export function setTextContent({
  editor,
  text,
}: {
  readonly editor: HTMLDivElement;
  readonly text: string;
},): void {
  editor.replaceChildren(
    ...text.split('\n',)
      .map(function createLineDiv(line,) {
      return h({
        tag: 'div',
        text: line === '' ? '\n' : line,
      },);
    },),
  );
}

/**
 * Reads the full text content from the editor's line divs.
 *
 * @param editor - contenteditable container element
 *
 * @returns joined text content with newline separators
 *
 * @example
 * ```ts
 * const result = getTextContent({ editor: editor, });
 * ```
 */
export function getTextContent({ editor, }: { readonly editor: HTMLDivElement; },): string {
  return [...editor.children,]
    .map(function readLine(child,) {
      /**
       * Defensive default keeps empty divs producing the empty string rather than null.
       */
      const t = child.textContent
        ?? '';
      return t === '\n' ? '' : t;
    },)
    .join('\n',);
}

/**
 * Reads the text of a single line by its 0-based index.
 *
 * @param editor - contenteditable container element
 *
 * @param line - 0-based line index
 *
 * @returns text content of the line, or null when out of range
 *
 * @example
 * ```ts
 * const text = getLineText({ editor: editor, line: 0, });
 * ```
 */
export function getLineText({
  editor,
  line,
}: {
  readonly editor: HTMLDivElement;
  readonly line: number;
},): string | null {
  /**
   * Out-of-range line index returns null rather than throwing.
   */
  const child = editor.children[line];
  if (child === undefined)
    return null;
  /**
   * Defensive default keeps empty divs producing the empty string rather than null.
   */
  const t = child.textContent
    ?? '';
  return t === '\n' ? '' : t;
}

//endregion Text content

//region Scrolling

/**
 * Scrolls a 1-based line number into the center of the viewport.
 *
 * @param editor - contenteditable container element
 *
 * @param line - 1-based line number to scroll to
 *
 * @example
 * ```ts
 * scrollLineIntoView({ editor: editor, line: 10, });
 * ```
 */
export function scrollLineIntoView({
  editor,
  line,
}: {
  readonly editor: HTMLDivElement;
  readonly line: number;
},): void {
  /**
   * Clamped lookup so out-of-range line numbers fall back to first/last instead of throwing.
   */
  const child = editor.children[
    Math.max(
      0,
      Math.min(
        line - 1,
        editor.children
          .length
          - 1,
      ),
    )
  ];
  if (child !== undefined)
    child.scrollIntoView({ block: 'center', },);
}

//endregion Scrolling
