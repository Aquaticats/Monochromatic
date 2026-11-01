/**
 Page-scoped CSS for the Search page.
 
 Separated from search.ts to keep the entry script focused on
 hydration logic rather than style declarations.
 */

/**
 Large border-radius value for pill-shaped tag chips.
 */
const PILL_BORDER_RADIUS_REM = 62.5;

/**
 Search-specific styles for hints, tag chips, and responsive layout.
 */
export const searchStyles: string = `
.search-hint {
  color: var(--fg-weaker);
  font-size: 1rem;
  line-height: 1.5;
}

.tag-chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--min-gap);
}

.tag-chip {
  display: flex;
  align-items: center;
  justify-content: center;
  border-width: calc(1 / 16 * 1rem);
  border-style: solid;
  border-color: var(--fg);
  border-radius: ${String(PILL_BORDER_RADIUS_REM,)}rem;
  padding-block: 0.5rem;
  padding-inline: 0.5rem;
  gap: 0.25rem;
  white-space: nowrap;
  font-size: 1rem;
  line-height: 1.5;
  cursor: pointer;
  background-color: transparent;
  font: inherit;

  &:hover {
    background-color: var(--hover-bg);
  }
}

@media (min-width: 48rem) {
  .search-hint {
    font-size: 1.5rem;
  }
}
`;
