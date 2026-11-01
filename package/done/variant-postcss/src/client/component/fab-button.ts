import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { css, } from '../css.ts';

/**
 Z-index for the floating action button above page content.
 */
const FAB_Z_INDEX = 50;

/**
 Shadow DOM styles for the `\<fab-button\>` component, positioned using {@link FAB_Z_INDEX}.
 */
const STYLES = css(`
  :host {
    position: fixed;
    inset-block-end: 1rem;
    inset-inline-end: 1rem;
    z-index: ${String(FAB_Z_INDEX,)};
  }
  button {
    @apply --flex-center;
    @apply --min-touch-target;
    inline-size: 4rem;
    block-size: 4rem;
    @apply --border-radius-full;
    background-color: var(--fg);
    border-width: 0.25rem;
    border-style: solid;
    border-color: var(--bg);
    color: var(--bg);
    font-size: 2rem;
    cursor: pointer;
    line-height: 1.2;
  }
  button:hover { opacity: 0.85; }
  button:focus-visible {
    outline-width: 0.125rem;
    outline-style: solid;
    outline-color: var(--fg);
    outline-offset: 0.125rem;
  }

`,);

/**
 `\<fab-button\>`: floating action button pinned to the bottom-right.
 Reads the `label` attribute for accessibility and renders a `\<slot\>` for custom content.
 */
class FabButton extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   Renders the button with aria-label and slot for content.
   */
  connectedCallback(): void {
    /**
     Accessibility label from the `label` attribute, with a fallback for missing attribute.
     */
    const label = this.getAttribute('label',)
      ?? 'Action';
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      h({
        tag: 'button',
        attrs: {
          part: 'button',
          'aria-label': label,
        },
        children: [h({
          tag: 'slot',
          text: '+',
        },),],
      },),
      );
  }
}

customElements.define(
  'fab-button',
  FabButton,
);
