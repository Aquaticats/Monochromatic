import {
  cssInt,
  cssPercent,
  cssRem,
  cssTranslateX,
  cssVar,
  hDom as h,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';

/**
 Z-index for toast positioning above page content.
 */
const TOAST_Z_INDEX = 1_000;

/**
 Center position percentage for inline-start.
 */
const CENTER_PERCENT = 50;

/**
 Negative center offset for translateX centering.
 */
const NEG_CENTER_PERCENT = -50;

/**
 Toast block padding in rem.
 */
const TOAST_PADDING_BLOCK = 0.55;

/**
 Toast inline padding in rem.
 */
const TOAST_PADDING_INLINE = 0.85;

/**
 Shadow DOM styles for the `\<toast-message\>` component.
 */
const STYLES = [
  css({
    rule: ':host',
    decls: {
      position: 'fixed',
      'inset-block-end': cssRem(1,),
      'inset-inline-start': cssPercent(CENTER_PERCENT,),
      transform: cssTranslateX(cssPercent(NEG_CENTER_PERCENT,),),
      'z-index': cssInt(TOAST_Z_INDEX,),
    },
  },),
  css({
    rule: '.content',
    decls: {
      'background-color': cssVar('red-bg',),
      color: cssVar('bg-stronger',),
      'padding-block': cssRem(TOAST_PADDING_BLOCK,),
      'padding-inline': cssRem(TOAST_PADDING_INLINE,),
    },
  },),
]
  .join('',);

/**
 Auto-dismiss duration in milliseconds.
 */
const DISMISS_MS = 3_000;

/**
 Sentinel for "no auto-dismiss timer is scheduled".
 */
const NO_TIMER: unique symbol = Symbol('toast auto-dismiss timer currently not scheduled',);

/**
 `\<toast-message\>`: ephemeral notification that auto-dismisses.
 Reads the `message` attribute for display text.
 */
class ToastMessage extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Handle for the auto-dismiss timer; `NO_TIMER` when not scheduled.
   */
  #timer: ReturnType<typeof setTimeout> | typeof NO_TIMER = NO_TIMER;

  /**
   Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   Renders content and schedules auto-removal after {@link DISMISS_MS}.
   */
  connectedCallback(): void {
    this.#render();
    /**
     Pre-bound `remove` so the timeout fires without losing `this`.
     */
    const removeFn = this.remove
      .bind(this,);
    this.#timer = setTimeout(
      function dismiss(): void {
        removeFn();
      },
      DISMISS_MS,
    );
  }

  /**
   Cancels the auto-dismiss timer when the element is removed early.
   
   @example
   ```ts
   toast.remove(); // triggers disconnectedCallback
   ```
   */
  disconnectedCallback(): void {
    if (this.#timer
      !== NO_TIMER) {
      clearTimeout(this.#timer,);
      this.#timer = NO_TIMER;
    }
  }

  /**
   Renders the toast content into the shadow root.
   */
  #render(): void {
    /**
     Resolved at render time so empty-attribute elements still produce a valid toast.
     */
    const message = this.getAttribute('message',)
      ?? '';
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      h({
        tag: 'div',
        class: 'content',
        text: message,
      },),
      );
  }
}

customElements.define(
  'toast-message',
  ToastMessage,
);

/**
 Shows a toast notification that auto-dismisses after 3 seconds.
 Removes any existing toast before showing the new one.
 
 @param message - Text to display in the toast
 
 @example
 ```ts
 showToast('Task saved successfully');
 ```
 */
export function showToast(message: string,): void {
  document.querySelector<HTMLElement>('toast-message',)
    ?.remove();

  /**
   Freshly constructed element so the new message replaces the previous one.
   */
  const toast = document.createElement('toast-message',);
  toast.setAttribute(
    'message',
    message,
  );
  document.body
    .append(toast,);
}
