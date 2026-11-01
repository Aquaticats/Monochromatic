/**
 `<side-drawer>`: intrinsic sidebar navigation.
 
 Renders two copies of the nav: an inline sidebar visible when the
 component is narrow, and a popover panel opened by the hamburger menu
 when the component is full-width (stacked).
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  buildCloseButton,
  buildHeader,
  buildNav,
} from './side-drawer-helpers.ts';
import { SIDE_DRAWER_STYLES, } from './side-drawer-styles.ts';

/**
 `<side-drawer>` web component.
 
 Toggle the popover via the `open` attribute (set by the top-nav hamburger).
 */
class SideDrawer extends HTMLElement {
  /**
   Attributes that trigger `attributeChangedCallback`.
   */
  static observedAttributes = ['open',];

  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Reference to the popover panel element; absent before first render.
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
   Whether the popover panel is currently visible.
   
   @returns Current open state
   */
  get open(): boolean {
    return this.hasAttribute('open',);
  }

  /**
   Sets the open state by adding or removing the `open` attribute.
   
   @param value - Whether the drawer should be open
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
   Closes the drawer by removing the open attribute.
   */
  #closeDrawer(): void {
    this.open = false;
  }

  /**
   Renders content, sets up panel reference, and wires event listeners.
   */
  connectedCallback(): void {
    this.#render();
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- querySelector returns the `.panel` div rendered unconditionally above
    this.#panel = this.#shadow
      .querySelector<HTMLDivElement>('.panel',) as HTMLDivElement;
    /**
     Pre-bound closer so the inline listeners keep `this` after handoff.
     */
    const closeFn = this.#closeDrawer
      .bind(this,);
    /**
     Local alias used by the backdrop listener to compare against the click target.
     */
    const panel = this.#panel;

    this.#shadow
      .querySelector<HTMLElement>('.panel-close',)
      ?.addEventListener(
      'click',
      function handleClose(): void {
        closeFn();
      },
      );

    // Light-dismiss: close when clicking the backdrop area (outside the drawer)
    panel.addEventListener(
      'click',
      function handleBackdropClick(event: Event,): void {
        if (event.target
          === panel)
          closeFn();
      },
    );
  }

  /**
   Syncs the popover visibility when the `open` attribute changes.
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
   Renders the inline sidebar and popover panel into the shadow root.
   */
  #render(): void {
    /**
     Reused close button, tagged with `.panel-close` so the listener can find it.
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
          //region Inline sidebar (visible in sidebar mode)
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

          //region Popover panel (visible via hamburger in stacked mode)
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
