/**
 Button row styles for the `\<task-detail\>` component.
 
 Covers .btn-row, .btn-outline, and .btn-primary variants.
 Interpolated into the main task-detail styles.
 */

/**
 CSS for task-detail action buttons.
 */
export const TASK_DETAIL_BUTTON_STYLES = `
  .btn-row {
    @apply --flex-row;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-block-start: 1rem;
    &[data-hidden] { display: none; }
  }
  .btn-outline {
    @apply --button-outlined;
    &:focus-visible {
      outline-width: 0.125rem;
      outline-style: solid;
      outline-color: var(--fg);
      outline-offset: 0.125rem;
    }
  }
  .btn-primary {
    @apply --button-outlined;
    background-color: var(--fg);
    color: var(--bg);
    &:focus-visible {
      outline-width: 0.125rem;
      outline-style: solid;
      outline-color: var(--fg);
      outline-offset: 0.125rem;
    }
  }
`;
