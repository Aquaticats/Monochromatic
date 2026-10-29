import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { css, } from '../css.ts';

/**
 Shadow DOM styles for the `\<section-heading\>` component.
 */
const STYLES = css(`
  :host {
    @apply --flex-column;
    gap: 0.5rem;
  }
  .heading {
    @apply --flex-row;
    gap: 1rem;
    font-size: 1.25rem;
    font-weight: 400;
    cursor: pointer;
  }
  .icon {
    @apply --flex-center;
    @apply --min-touch-target;
    font-size: 2rem;
  }
  .toggle {
    inline-size: 1.25rem;
    block-size: 1.25rem;
  }
  .content {
    @apply --flex-column;
    gap: var(--gap);
  }
`,);

/**
 `\<section-heading\>`: collapsible section with icon, label, and toggle indicator.
 Dispatches a `toggle` event with `\{ open \}` when the heading is clicked.
 */
class SectionHeading extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Whether the section content is currently expanded.
   */
  #open = true;

  /**
   Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   Whether the section is currently expanded.
   
   @returns True when the section content is visible
   */
  get open(): boolean {
    return this.#open;
  }

  /**
   Renders the heading and attaches the toggle click handler.
   */
  connectedCallback(): void {
    this.#render();
    this.#shadow
      .querySelector<HTMLElement>('.heading',)
      ?.addEventListener(
      'click',
      this.#toggle
        .bind(this,),
      );
  }

  /**
   Toggles the open state and dispatches a toggle event.
   
   Bound to this instance at the listener site because it is attached to the
   inner `.heading` element, whose `currentTarget` is not this component.
   */
  #toggle(): void {
    this.#open = !this.#open;
    this.#updateToggle();
    this.dispatchEvent(
      new CustomEvent(
        'toggle',
        {
          detail: { open: this.#open, },
          bubbles: true,
        },
      ),
    );
  }

  /**
   Updates the toggle indicator and content visibility.
   */
  #updateToggle(): void {
    /**
     Toggle indicator span; absent before first render, so the check below guards.
     */
    const toggle = this.#shadow
      .querySelector<HTMLElement>('.toggle',);
    if (toggle instanceof HTMLElement)
      toggle.textContent = this.#open ? '\u25B2' : '\u25BC';
    /**
     Content wrapper whose display style is flipped to show or hide the slotted content.
     */
    const content = this.#shadow
      .querySelector<HTMLElement>('.content',);
    if (content !== null) {
      (content as HTMLElement).style
        .display = this.#open ? 'flex' : 'none';
    }
  }

  /**
   Renders the heading, toggle indicator, and content slot into the shadow root.
   */
  #render(): void {
    /**
     Leading icon glyph from the `icon` attribute.
     */
    const icon = this.getAttribute('icon',)
      ?? '';
    /**
     Heading text from the `label` attribute, displayed next to the icon.
     */
    const label = this.getAttribute('label',)
      ?? '';

    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      h({
        tag: 'div',
        class: 'heading',
        children: [
          h({
            tag: 'span',
            class: 'icon',
            text: icon,
          },),
          h({
            tag: 'span',
            text: label,
          },),
          h({
            tag: 'span',
            class: 'toggle',
            text: this.#open ? '\u25B2' : '\u25BC',
          },),
        ],
      },),
      h({
        tag: 'div',
        class: 'content',
        children: [h({ tag: 'slot', },),],
      },),
      );
  }
}

customElements.define(
  'section-heading',
  SectionHeading,
);
