import {
  cssInt,
  cssRem,
  cssVar,
  hDom as h,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import {
  borderRadiusFull,
  flexCenter,
  focusOutline,
  minTouchTarget,
} from '../mixins.ts';

/**
 Z-index for FAB positioning.
 */
const Z_INDEX_FAB = 50;

/**
 FAB button size in rem.
 */
const FAB_SIZE = 4;

/**
 FAB border width in rem (1/4).
 */
const FAB_BORDER = 1 / 2
  / 2;

/**
 Compiled CSS string for `<fab-button>` Shadow DOM.
 */
const STYLES = [
  css({
    rule: ':host',
    decls: {
      position: 'fixed',
      'inset-block-end': cssRem(1,),
      'inset-inline-end': cssRem(1,),
      'z-index': cssInt(Z_INDEX_FAB,),
    },
  },),
  css({
    rule: 'button',
    decls: {
      ...flexCenter(),
      ...minTouchTarget(),
      'inline-size': cssRem(FAB_SIZE,),
      'block-size': cssRem(FAB_SIZE,),
      ...borderRadiusFull(),
      'background-color': cssVar('fg',),
      'border-width': cssRem(FAB_BORDER,),
      'border-style': 'solid',
      'border-color': cssVar('bg',),
      color: cssVar('bg',),
      'font-size': cssRem(2,),
      cursor: 'pointer',
      'line-height': 1.2,
    },
    children: [
      css({
        rule: '&:hover',
        decls: { opacity: 0.85, },
      },),
      css({
        rule: '&:focus-visible',
        decls: focusOutline(),
      },),
    ],
  },),
]
  .join('',);

/**
 `<fab-button>`: floating action button pinned to the bottom-right.
 Reads the `label` attribute for accessibility and renders a `<slot>` for custom content.
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
   Renders the action button with label and slot into the shadow root.
   */
  connectedCallback(): void {
    /**
     Resolved at attach time so missing attributes still render a usable button.
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
