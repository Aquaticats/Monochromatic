/**
 Shadow DOM styles for the `\<top-nav\>` component.
 */
import { css, } from '../css.ts';

/**
 Shadow DOM styles for `\<top-nav\>` -- hamburger, heading, search icon, responsive layout.
 */
export const TOP_NAV_STYLES: string = css(`
  :host {
    @apply --sticky-bar;
    justify-content: center;
  }
  h1 {
    flex: 1;
    text-align: center;
    font-size: 1.5rem;
    font-weight: 400;
    line-height: normal;
    margin-block: 0;
    margin-inline: 0;
  }
  .action {
    @apply --appearance-none;
    @apply --flex-center;
    @apply --min-touch-target;
    color: var(--fg);
    text-decoration: none;
  }
  .action:focus-visible {
    outline-width: 0.125rem;
    outline-style: solid;
    outline-color: var(--fg);
    outline-offset: -0.125rem;
  }
  .hamburger {
    inline-size: 2rem;
    block-size: 2rem;
    @apply --flex-column;
    justify-content: center;
    align-items: center;
    gap: 0.375rem;
  }
  .line {
    inline-size: 1.75rem;
    block-size: 0.25rem;
    background-color: var(--fg);
    display: block;
  }
  .search-icon {
    inline-size: 2rem;
    block-size: 2rem;
    position: relative;
  }
  .circle {
    position: absolute;
    inset-block-start: 0;
    inset-inline-start: 0;
    inline-size: 1.375rem;
    block-size: 1.375rem;
    border-width: 0.25rem;
    border-style: solid;
    border-color: var(--fg);
    @apply --border-radius-full;
  }
  .handle {
    position: absolute;
    inset-block-start: calc(19 / 16 * 1rem);
    inset-inline-start: calc(19 / 16 * 1rem);
    inline-size: 0.25rem;
    block-size: 0.875rem;
    background-color: var(--fg);
    transform: rotate(-45deg);
    transform-origin: top left;
  }
  @media (min-width: 48rem) {
    :host {
      justify-content: space-between;
      padding-inline-start: var(--min-gap);
      border-block-end-width: calc(1 / 16 * 1rem);
      border-block-end-style: solid;
      border-block-end-color: var(--bg-weaker);
    }
    .menu-toggle { display: none; }
    h1 { text-align: start; }
  }
`,);
