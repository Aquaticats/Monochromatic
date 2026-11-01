import {
  cssInt,
  cssRem,
  cssVar,
  hDom as h,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import {
  flexCenter,
  flexColumn,
  flexRow,
  minTouchTarget,
} from '../mixins.ts';

/**
 Host gap in rem (1/2).
 */
const HOST_GAP = 1 / 2;

/**
 Heading and toggle size in rem (1 1/4).
 */
const HEADING_SIZE = 1 + ((1 / 2) / 2);

/**
 Normal font weight.
 */
const FONT_WEIGHT_NORMAL = 400;

/**
 Compiled CSS string for `<section-heading>` Shadow DOM.
 */
const STYLES = [
  css({
    rule: ':host',
    decls: {
      ...flexColumn(),
      gap: cssRem(HOST_GAP,),
    },
  },),
  css({
    rule: '.heading',
    decls: {
      ...flexRow(),
      gap: cssRem(1,),
      'font-size': cssRem(HEADING_SIZE,),
      'font-weight': cssInt(FONT_WEIGHT_NORMAL,),
      cursor: 'pointer',
    },
  },),
  css({
    rule: '.icon',
    decls: {
      ...flexCenter(),
      ...minTouchTarget(),
      'font-size': cssRem(2,),
    },
  },),
  css({
    rule: '.toggle',
    decls: {
      'inline-size': cssRem(HEADING_SIZE,),
      'block-size': cssRem(HEADING_SIZE,),
    },
  },),
  css({
    rule: '.content',
    decls: {
      ...flexColumn(),
      gap: cssVar('gap',),
    },
  },),
]
  .join('',);

/**
 `<section-heading>`: collapsible section with icon, label, and toggle indicator.
 Dispatches a `toggle` event with `{ open }` when the heading is clicked.
 */
class SectionHeading extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Whether the section content is expanded.
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
   Whether the section content is currently visible.
   
   @returns Current open state
   */
  get open(): boolean {
    return this.#open;
  }

  /**
   Renders content and wires the heading click listener.
   */
  connectedCallback(): void {
    this.#render();
    this.#shadow
      .querySelector<HTMLElement>('.heading',)
      ?.addEventListener(
      'click',
      this.#toggle,
      );
  }

  /**
   Bound toggle handler that collapses/expands and dispatches a `toggle` event.
   */
  readonly #toggle = this.#onToggle
    .bind(this,);

  /**
   Toggles the open state and dispatches a `toggle` event.
   */
  #onToggle(): void {
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
   
   @example
   ```ts
   this.#updateToggle();
   ```
   */
  #updateToggle(): void {
    /**
     Shadow-DOM lookup; element may be missing if `#render` has not run yet.
     */
    const toggle = this.#shadow
      .querySelector<HTMLElement>('.toggle',);
    if (toggle instanceof HTMLElement)
      toggle.textContent = this.#open ? '\u25B2' : '\u25BC';
    /**
     Sibling content region whose visibility tracks the open flag.
     */
    const content = this.#shadow
      .querySelector<HTMLElement>('.content',);
    if (content !== null)
      content.style
        .display = this.#open ? 'flex' : 'none';
  }

  /**
   Renders the heading row and content slot into the shadow root.
   */
  #render(): void {
    /**
     Resolved at render time so heading still works when the icon attribute is omitted.
     */
    const icon = this.getAttribute('icon',)
      ?? '';
    /**
     Resolved at render time so heading still works when the label attribute is omitted.
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
