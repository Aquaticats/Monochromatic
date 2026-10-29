/**
 Shadow DOM styles for the `\<task-card\>` component.
 */
import { css, } from '../css.ts';

/**
 Shadow DOM styles for `\<task-card\>` -- card layout, checkbox, title, and metadata chips.
 */
export const TASK_CARD_STYLES: string = css(`
  :host {
    @apply --flex-column;
    gap: var(--min-gap);
    background-color: var(--bg);
    overflow: hidden;
    cursor: pointer;
  }
  .row {
    @apply --flex-row;
    gap: var(--min-gap);
    align-items: flex-start;
  }
  .checkbox {
    @apply --appearance-none;
    @apply --flex-center;
    inline-size: 2rem;
    block-size: 2rem;
  }
  .checkbox:focus-visible {
    outline-width: 0.125rem;
    outline-style: solid;
    outline-color: var(--fg);
    outline-offset: 0.125rem;
  }
  .checkbox-box {
    inline-size: 1.75rem;
    block-size: 1.75rem;
    border-width: 0.25rem;
    border-style: solid;
    border-color: var(--fg);
  }
  .title {
    font-size: 1.25rem;
    font-weight: 400;
    line-height: normal;
    flex: 1;
    min-inline-size: 0;
  }
  .chips {
    @apply --scroll-row;
  }
  .chips::-webkit-scrollbar { display: none; }
  .chip {
    @apply --flex-row;
    @apply --whitespace-nowrap;
    gap: 0.25rem;
    font-size: 1rem;
    line-height: 1.5;
  }
  .chip.blocked {
    border-color: var(--red-fg);
    color: var(--red-fg);
  }
`,);
