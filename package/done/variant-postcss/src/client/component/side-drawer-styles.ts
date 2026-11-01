/**
 Shadow DOM styles for the `\<side-drawer\>` component.
 
 Covers two display modes: an inline sidebar (visible beside main content)
 and a popover panel (hamburger-triggered overlay for stacked/narrow layouts).
 */
import { css, } from '../css.ts';
import { SIDE_DRAWER_PANEL_STYLES, } from './side-drawer-panel-styles.ts';

/**
 Viewport breakpoint matching the body flex-wrap threshold.
 */
const DESKTOP_BREAKPOINT = '48rem';

/**
 Shadow DOM styles for `\<side-drawer\>` -- inline sidebar, shared nav, and panel.
 */
export const SIDE_DRAWER_STYLES: string = css(`
  :host { display: block; }
  .wrapper { block-size: 100%; }

  .divider {
    block-size: calc(1 / 16 * 1rem);
    background-color: var(--bg-weaker);
    inline-size: 100%;
  }

  nav {
    @apply --flex-column;
    gap: var(--min-gap);
    flex: 1;
    padding-block-start: var(--min-gap);
  }

  a {
    @apply --flex-row;
    gap: var(--min-gap);
    min-block-size: 3rem;
    padding-block: 0;
    padding-inline: var(--min-gap);
    color: var(--fg);
    text-decoration: none;
    font-size: 1.25rem;
    font-weight: 400;
    &:hover { background-color: var(--hover-bg); }
    &:focus-visible {
      outline-width: 0.125rem;
      outline-style: solid;
      outline-color: var(--fg);
      outline-offset: -0.125rem;
    }
  }

  .header {
    @apply --flex-row;
    justify-content: space-between;
    padding-block-start: var(--min-gap);
    padding-block-end: var(--min-padding);
    padding-inline-start: var(--min-gap);
    padding-inline-end: var(--min-padding);
    min-block-size: 4rem;
  }

  .close {
    @apply --appearance-none;
    @apply --flex-center;
    @apply --min-touch-target;
    &:focus-visible {
      outline-width: 0.125rem;
      outline-style: solid;
      outline-color: var(--fg);
      outline-offset: -0.125rem;
    }
    & svg { inline-size: 2rem; block-size: 2rem; }
  }

  .sidebar {
    @apply --flex-column;
    block-size: 100%;
    border-inline-end-width: calc(1 / 16 * 1rem);
    border-inline-end-style: solid;
    border-inline-end-color: var(--bg-weaker);
    & .header {
      padding-block: var(--min-padding);
      padding-inline-start: var(--min-gap);
      padding-inline-end: var(--min-padding);
    }
    & .close { display: none; }
  }

  .sidebar { display: none; }

  @media (min-width: ${DESKTOP_BREAKPOINT}) {
    .sidebar { @apply --flex-column; }
  }

  ${SIDE_DRAWER_PANEL_STYLES}

  @apply --shadow-dom-globals;
`,);
