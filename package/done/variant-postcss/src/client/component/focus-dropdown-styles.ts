/**
 Shadow DOM styles for the `\<focus-dropdown\>` component.
 */
import { css, } from '../css.ts';

/**
 Z-index for the dropdown menu overlay.
 */
const MENU_Z_INDEX = 10;

/**
 Shadow DOM styles for `\<focus-dropdown\>` -- trigger button, popover menu (stacked at {@link MENU_Z_INDEX}), and option items.
 */
export const FOCUS_DROPDOWN_STYLES: string = css(`
  :host {
    display: block;
    inline-size: 100%;
    position: relative;
  }
  .trigger {
    @apply --button-outlined;
    inline-size: 100%;
    text-align: start;

    &:focus-visible {
      outline-width: 0.125rem;
      outline-style: solid;
      outline-color: var(--fg);
      outline-offset: 0.125rem;
    }
  }
  .text {
    flex: 1;
    text-align: start;
  }
  .divider {
    inline-size: calc(1 / 16 * 1rem);
    block-size: 100%;
    background-color: var(--fg-weaker);
  }
  .menu {
    position: absolute;
    inset-block-start: 100%;
    inset-inline-start: 0;
    inline-size: 100%;
    border-width: calc(1 / 16 * 1rem);
    border-style: solid;
    border-color: var(--fg);
    background-color: var(--bg);
    padding-block: 0.25rem;
    padding-inline: 0;
    margin-block: 0;
    margin-inline: 0;
    list-style: none;
    z-index: ${String(MENU_Z_INDEX,)};

    &:not(:popover-open) { display: none; }
  }
  .option {
    padding-block: 0.5rem;
    padding-inline: 0.5rem;
    cursor: pointer;

    &:hover {
      background-color: var(--hover-bg);
    }
  }
`,);
