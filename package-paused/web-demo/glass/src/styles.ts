/**
 * Stylesheet for the glass corridor demo page: full-viewport canvas, a
 * minimal HUD in the corners, and an error overlay for browsers that
 * cannot start the renderer.
 *
 * Sizing is in `rem`, positions and alignment use logical properties, every
 * color comes from a custom property declared in `:root`, and shorthands stay
 * single-concept, as `.agents/skills/css/SKILL.md` requires. The palette is
 * dark-only by design: the corridor renders dark and the HUD is lit text over
 * it, so retuning the demo means changing the tokens, not the rules.
 */

/**
 * Renders the complete CSS for the demo page.
 *
 * @returns CSS stylesheet string
 *
 * @example
 * ```ts
 * const css = renderStyles();
 * ```
 */
export function renderStyles(): string {
  return `
:root {
  color-scheme: dark;

  --color-background: #04070e;
  --color-foreground: #e8f4fb;
  --color-foreground-strong: #ffffff;
  --color-foreground-muted: #8fb4c9;
  --color-accent: #8ee8c2;
  --color-shadow: #000000;
  --color-scrim: rgba(2, 5, 11, 0.92);
  --color-error-surface: #0d1725;
  --color-error-foreground: #a9bfd0;
  --color-error-border: rgba(255, 180, 180, 0.4);
}
* {
  box-sizing: border-box;
}
html,
body {
  block-size: 100%;
  margin-block: 0;
  margin-inline: 0;
  overflow: hidden;
}
html {
  font-size: 100%;
}
body {
  background-color: var(--color-background);
  color: var(--color-foreground);
  font-family: ui-sans-serif, system-ui, sans-serif;
  font-size: 0.875rem;
  font-weight: 500;
  line-height: 1.4;
}
#stage {
  position: fixed;
  inset: 0;
  inline-size: 100%;
  block-size: 100%;
  touch-action: none;
  cursor: crosshair;
}
.hud {
  position: fixed;
  z-index: 2;
  pointer-events: none;
  text-shadow: 0 0.125rem 0.875rem var(--color-shadow);
}
#brand {
  inset-block-start: 1.125rem;
  inset-inline-start: 1.25rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}
#brand b {
  display: block;
  font-size: 0.8125rem;
  color: var(--color-foreground-strong);
}
#brand span {
  display: block;
  margin-block-start: 0.25rem;
  font-size: 0.625rem;
  color: var(--color-foreground-muted);
}
#score {
  inset-block-start: 1.125rem;
  inset-inline-end: 1.375rem;
  text-align: end;
}
#score-value {
  display: block;
  font-size: 2.75rem;
  font-variant-numeric: tabular-nums;
  line-height: 0.9;
  letter-spacing: -0.04em;
  color: var(--color-foreground-strong);
}
#score span {
  display: block;
  margin-block-start: 0.375rem;
  font-size: 0.625rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color-foreground-muted);
}
#hint {
  inset-inline-start: 50%;
  inset-block-end: 12%;
  transform: translateX(-50%);
  font-size: 0.6875rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  transition: opacity 0.5s;
}
#hint.hidden {
  opacity: 0;
}
#backend-box {
  inset-inline-start: 1.25rem;
  inset-block-end: 1rem;
  font-size: 0.625rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--color-foreground-muted);
}
#backend {
  color: var(--color-accent);
}
#error {
  position: fixed;
  z-index: 8;
  inset: 0;
  display: grid;
  place-items: center;
  padding-block: 1.75rem;
  padding-inline: 1.75rem;
  background-color: var(--color-scrim);
}
#error[hidden] {
  display: none;
}
#error div {
  max-inline-size: 30rem;
  padding-block: 1.5rem;
  padding-inline: 1.5rem;
  border-width: 0.0625rem;
  border-style: solid;
  border-color: var(--color-error-border);
  border-radius: 0.875rem;
  background-color: var(--color-error-surface);
  text-align: center;
}
#error h1 {
  margin-block-start: 0;
  margin-block-end: 0.5rem;
  margin-inline: 0;
  font-size: 1.25rem;
}
#error p {
  margin-block: 0;
  margin-inline: 0;
  color: var(--color-error-foreground);
}
`;
}
