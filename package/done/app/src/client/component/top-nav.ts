/**
 `<top-nav>`: sticky navigation bar with hamburger menu, page heading, and search link.
 Dispatches a `menu-open` composed event when the hamburger is clicked.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { TOP_NAV_STYLES, } from './top-nav-styles.ts';

/**
 `<top-nav>` web component.
 
 Renders a sticky top bar with a hamburger toggle (for mobile), a heading,
 and a search link.
 */
class TopNav extends HTMLElement {
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
   Renders the hamburger, heading, and search link into the shadow root.
   */
  connectedCallback(): void {
    /**
     Resolved at attach time so a missing attribute still renders an empty title slot.
     */
    const heading = this.getAttribute('heading',)
      ?? '';
    /**
     Pre-bound dispatcher so the inner click handler keeps `this` after handoff.
     */
    const dispatchFn = this.dispatchEvent
      .bind(this,);

    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: TOP_NAV_STYLES,
      },),
      h({
        tag: 'button',
        class: 'action menu-toggle',
        attrs: { 'aria-label': 'Open menu', },
        children: [
          h({
            tag: 'span',
            class: 'hamburger',
            children: [
              h({
                tag: 'span',
                class: 'line',
              },),
              h({
                tag: 'span',
                class: 'line',
              },),
              h({
                tag: 'span',
                class: 'line',
              },),
            ],
          },),
        ],
        on: {
          click: function handleMenuOpen(): void {
            dispatchFn(
              new CustomEvent(
                'menu-open',
                {
                  bubbles: true,
                  composed: true,
                },
              ),
            );
          },
        },
      },),
      h({
        tag: 'h1',
        text: heading,
      },),
      h({
        tag: 'a',
        class: 'action',
        attrs: {
          href: '/search',
          'aria-label': 'Search',
        },
        children: [
          h({
            tag: 'span',
            class: 'search-icon',
            children: [
              h({
                tag: 'span',
                class: 'circle',
              },),
              h({
                tag: 'span',
                class: 'handle',
              },),
            ],
          },),
        ],
      },),
      );
  }
}

customElements.define(
  'top-nav',
  TopNav,
);
