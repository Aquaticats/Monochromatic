/**
 Page-scoped CSS for the Inbox page controls.
 
 Separated from inbox.ts to keep the entry script focused on
 hydration logic rather than style declarations.
 */

/**
 Inbox-specific styles for task children, controls, and location options.
 */
export const inboxStyles = `
.task-children {
  margin-inline-start: 1.5rem;
  border-inline-start-width: 0.125rem;
  border-inline-start-style: solid;
  border-inline-start-color: var(--bg-weaker);
  padding-inline-start: 0.75rem;
}

.controls {
  display: flex;
  flex-wrap: wrap;
  gap: var(--gap);
  align-items: flex-start;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: var(--min-padding);
  flex: 1 0 0;
  min-inline-size: 100%;
  overflow: hidden;
}

.subsection-heading {
  font-size: 1.25rem;
  font-weight: 400;
}

.subsection-desc {
  font-size: calc(15 / 16 * 1rem);
  line-height: 1.5;
  color: var(--fg-weaker);
}

.location-options {
  display: flex;
  gap: var(--min-gap);
  align-items: center;
  min-block-size: 3rem;
  flex-wrap: wrap;
}

.autodetect-toggle {
  display: flex;
  gap: var(--min-padding);
  align-items: center;
  cursor: pointer;
  background-color: transparent;
  border-style: none;
  font: inherit;
  color: var(--fg);
  padding-block: 0;
  padding-inline: 0;
}
`;
