/**
 `<toggle-switch>`: boolean toggle with animated thumb.
 Reflects state via the `on` attribute and dispatches a `change` event on toggle.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { TOGGLE_SWITCH_STYLES, } from './toggle-switch-styles.ts';

/**
 `<toggle-switch>` web component.
 */
class ToggleSwitch extends HTMLElement {
  /**
   Attributes that trigger `attributeChangedCallback`.
   */
  static observedAttributes = ['on',];

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
   Whether the toggle is currently in the "on" position.
   
   @returns Current toggle state
   */
  get on(): boolean {
    return this.hasAttribute('on',);
  }

  /**
   Sets the toggle state by adding or removing the `on` attribute.
   
   @param value - Whether the toggle should be on
   */
  set on(value: boolean,) {
    if (value) {
      this.setAttribute(
        'on',
        '',
      );
    }
    else {
      this.removeAttribute('on',);
    }
  }

  /**
   Renders initial content and wires up the click listener.
   */
  connectedCallback(): void {
    this.#render();
    this.addEventListener(
      'click',
      this.#handleClick,
    );
  }

  /**
   Removes the click listener when the element is disconnected.
   */
  disconnectedCallback(): void {
    this.removeEventListener(
      'click',
      this.#handleClick,
    );
  }

  /**
   Re-renders when the `on` attribute changes.
   */
  attributeChangedCallback(): void {
    this.#render();
  }

  /**
   Bound click handler that toggles state and dispatches a `change` event.
   */
  readonly #handleClick = this.#onHandleClick
    .bind(this,);

  /**
   Toggles state and dispatches a change event.
   */
  #onHandleClick(): void {
    this.on = !this.on;
    this.dispatchEvent(
      new CustomEvent(
        'change',
        {
          detail: { on: this.on, },
          bubbles: true,
        },
      ),
    );
  }

  /**
   Renders the track and thumb elements into the shadow root.
   */
  #render(): void {
    /**
     Captured once so the value stays stable across both child branches below.
     */
    const isOn = this.on;
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: TOGGLE_SWITCH_STYLES,
      },),
      h({
        tag: 'div',
        class: 'track',
        children: [
          h({
            tag: 'span',
            class: `thumb ${isOn ? 'on' : 'off'}`,
            text: isOn ? '\u2713' : '\u2717',
          },),
        ],
      },),
      );
  }
}

customElements.define(
  'toggle-switch',
  ToggleSwitch,
);
