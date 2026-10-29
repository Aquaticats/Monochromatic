import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { css, } from '../css.ts';

/**
 Z-index for toast positioning above page content.
 */
const TOAST_Z_INDEX = 1_000;

/**
 Shadow DOM styles for the `\<toast-message\>` component, positioned using {@link TOAST_Z_INDEX}.
 */
const STYLES = css(`
  :host {
    position: fixed;
    inset-block-end: 1rem;
    inset-inline-start: 50%;
    transform: translateX(-50%);
    z-index: ${String(TOAST_Z_INDEX,)};
  }
  .content {
    background-color: var(--red-bg);
    color: var(--bg-stronger);
    padding-block: 0.55rem;
    padding-inline: 0.85rem;
  }
`,);

/**
 Auto-dismiss duration in milliseconds.
 */
const DISMISS_MS = 3_000;

/**
 Sentinel for "no auto-dismiss timer is scheduled".
 */
const NO_TIMER: unique symbol = Symbol('toast auto dismiss timer not scheduled',);

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
   Handle for the auto-dismiss timer; {@link NO_TIMER} when not scheduled.
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
     Captured so the dismiss closure removes this component without a `this`-bound function.
     */
    const self = this;
    this.#timer = setTimeout(
      function dismiss(): void {
        self.remove();
      },
      DISMISS_MS,
    );
  }

  /**
   Cancels the auto-dismiss timer when the element is removed early.
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
     Text payload from the `message` attribute; empty string when the attribute is absent.
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
 showToast('Task created successfully');
 ```
 */
export function showToast(message: string,): void {
  document.querySelector<HTMLElement>('toast-message',)
    ?.remove();

  /**
   Fresh toast element; previous toast (if any) was removed above to avoid overlap.
   */
  const toast = document.createElement('toast-message',);
  toast.setAttribute(
    'message',
    message,
  );
  document.body
    .append(toast,);
}
