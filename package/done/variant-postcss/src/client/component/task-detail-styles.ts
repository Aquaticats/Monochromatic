/**
 Shadow DOM styles for the `\<task-detail\>` component.
 */
import { css, } from '../css.ts';
import { TASK_DETAIL_BUTTON_STYLES, } from './task-detail-button-styles.ts';

/**
 Shadow DOM styles for `\<task-detail\>`.
 */
export const TASK_DETAIL_STYLES: string = css(`
  :host {
    @apply --flex-column;
    gap: 1rem;
    padding-block: 1rem;
    padding-inline: 1rem;
  }
  .header { @apply --flex-row; justify-content: space-between; }
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
    & svg { inline-size: 2rem; block-size: 2rem; stroke: var(--fg); stroke-width: 4; }
  }
  .heading { font-size: 1.5rem; font-weight: 400; }
  .title-input {
    font-size: 1.5rem;
    font-weight: 400;
    border-style: none;
    border-block-end-width: calc(1 / 16 * 1rem);
    border-block-end-style: solid;
    border-block-end-color: var(--fg);
    background-color: transparent;
    inline-size: 100%;
    padding-block: 0.25rem;
    padding-inline: 0;
    outline: none;
    font-family: inherit;
    color: var(--fg);
  }
  .desc-input {
    border-width: calc(1 / 16 * 1rem);
    border-style: solid;
    border-color: var(--fg);
    padding-block: 0.5rem;
    padding-inline: 0.5rem;
    min-block-size: 4.5rem;
    resize: vertical;
    font: inherit;
    color: var(--fg);
    background-color: transparent;
  }
  .actions { display: flex; gap: 1rem; }
  .pills { @apply --scroll-row; flex-wrap: wrap; }
  .pill {
    @apply --flex-center;
    @apply --whitespace-nowrap;
    border-width: calc(1 / 16 * 1rem);
    border-style: solid;
    border-color: var(--fg);
    @apply --border-radius-full;
    padding-block: 0.5rem;
    padding-inline: 0.5rem;
    gap: 0.25rem;
    font-size: 1rem;
    line-height: 1.5;
    &[data-autofilled] { border-color: var(--red-fg); color: var(--red-fg); }
    &[data-loading] { opacity: 0.5; }
  }

  ${TASK_DETAIL_BUTTON_STYLES}

  @apply --shadow-dom-globals;
`,);
