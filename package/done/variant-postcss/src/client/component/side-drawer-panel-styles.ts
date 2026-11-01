/**
 Popover panel styles for the `\<side-drawer\>` component.
 
 Covers the fixed overlay, slide-in animation, scrim fade-in,
 and the drawer container. Interpolated into the main styles.
 */

/**
 Z-index for the popover panel overlay.
 */
const PANEL_Z_INDEX = 100;

/**
 CSS for the popover panel portion of `\<side-drawer\>`, layered at {@link PANEL_Z_INDEX}.
 */
export const SIDE_DRAWER_PANEL_STYLES: string = `
  .panel {
    position: fixed;
    inset: 0;
    margin: 0;
    padding-block: 0;
    padding-inline: 0;
    border-style: none;
    inline-size: 100%;
    max-inline-size: 100%;
    block-size: 100%;
    max-block-size: 100%;
    z-index: ${String(PANEL_Z_INDEX,)};
    display: flex;
    background-color: transparent;
    overflow: visible;
  }

  .panel:not(:popover-open) { display: none; }

  @keyframes drawer-slide-in {
    from { transform: translateX(-100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  @keyframes scrim-fade-in {
    from { background-color: transparent; }
    to { background-color: var(--overlay-bg); }
  }

  .panel:popover-open {
    animation-name: scrim-fade-in;
    animation-duration: 200ms;
    animation-timing-function: ease-out;
    animation-fill-mode: both;
  }

  .panel:popover-open > .panel-drawer {
    animation-name: drawer-slide-in;
    animation-duration: 250ms;
    animation-timing-function: cubic-bezier(0, 0, 0.2, 1);
    animation-fill-mode: both;
  }

  .panel-drawer {
    background-color: var(--bg);
    inline-size: 20rem;
    max-inline-size: 85vw;
    block-size: 100%;
    @apply --flex-column;
  }
`;
