/**
 `\<side-drawer\>`: intrinsic sidebar navigation.
 
 Renders two copies of the nav: an inline sidebar visible when the
 component is narrow (~22 rem, meaning it sits beside the main content
 in the Every Layout sidebar pattern), and a popover panel opened by
 the hamburger menu when the component is full-width (stacked).
 
 Visibility is governed entirely by a CSS container query on the
 wrapper element; no viewport media queries, no JS resize observers.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  buildCloseButton,
  buildHeader,
  buildNav,
} from './side-drawer-nav.ts';
import { SIDE_DRAWER_STYLES, } from './side-drawer-styles.ts';

/**
 `\<side-drawer\>` web component with intrinsic layout switching.
 
 When the component is narrow (inside the Every Layout sidebar flex container),
 the inline sidebar is visible. When stacked (full viewport width), the inline
 sidebar hides and the hamburger popover is used instead.
 
 Toggle the popover via the `open` attribute (set by the top-nav hamburger).
 */
class SideDrawer extends HTMLElement {
  /**
   Attributes to observe for popover toggling.
   */
  static observedAttributes = ['open',];

  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Reference to the popover panel element; absent until first render.
   */
  #panel?: HTMLDivElement;

  /**
   Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   Whether the popover panel is currently open.
   
   @returns True when the `open` attribute is present
   */
  get open(): boolean {
    return this.hasAttribute('open',);
  }

  /**
   Sets or removes the `open` attribute to control popover visibility.
   
   @param value - New open state
   */
  set open(value: boolean,) {
    if (value) {
      this.setAttribute(
        'open',
        '',
      );
    }
    else {
      this.removeAttribute('open',);
    }
  }

  /**
   Renders content and attaches event handlers for closing the drawer.
   */
  connectedCallback(): void {
    this.#render();
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- querySelector returns the panel div we created
    this.#panel = this.#shadow
      .querySelector<HTMLDivElement>('.panel',) as HTMLDivElement;
    /**
     Captured so the close handlers reach this component without `this`-bound functions.
     */
    const self = this;

    this.#shadow
      .querySelector<HTMLElement>('.panel-close',)
      ?.addEventListener(
      'click',
      function closeDrawer(): void {
        self.open = false;
      },
      );

    // Light-dismiss: close when clicking the backdrop area (outside the drawer)
    this.#panel
      .addEventListener(
      'click',
      function lightDismiss(event: Event,): void {
        if (event.target
          === self
            .#panel)
          self.open = false;
      },
      );
  }

  /**
   Toggles popover visibility when the open attribute changes.
   */
  attributeChangedCallback(): void {
    if (this.#panel
      === undefined)
      return;

    if (this.open)
      this.#panel
        .showPopover();
    else
      this.#panel
        .hidePopover();
  }

  /**
   Renders both the inline sidebar and popover panel into the shadow root.
   */
  #render(): void {
    /**
     Popover-only close button; tagged with `panel-close` so the click handler can find it.
     */
    const panelClose = buildCloseButton('Close menu',);
    panelClose.classList
      .add('panel-close',);

    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: SIDE_DRAWER_STYLES,
      },),
      h({
        tag: 'div',
        class: 'wrapper',
        children: [
          //region Inline sidebar: visible in sidebar mode
          h({
            tag: 'aside',
            class: 'sidebar',
            children: [
              buildHeader(),
              h({
                tag: 'div',
                class: 'divider',
              },),
              buildNav(),
            ],
          },),
          //endregion Inline sidebar

          //region Popover panel: visible via hamburger in stacked mode
          h({
            tag: 'div',
            class: 'panel',
            attrs: { popover: 'manual', },
            children: [
              h({
                tag: 'aside',
                class: 'panel-drawer',
                children: [
                  buildHeader(panelClose,),
                  h({
                    tag: 'div',
                    class: 'divider',
                  },),
                  buildNav(),
                ],
              },),
            ],
          },),
          //endregion Popover panel
        ],
      },),
      );
  }
}

customElements.define(
  'side-drawer',
  SideDrawer,
);
