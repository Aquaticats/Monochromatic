/**
 Global stylesheet assembled from token, layout, and panel modules.
 
 Imported by page entry scripts and passed to `injectCSS()`.
 */
import {
  layoutShell,
  resets,
} from './styles-layout.ts';
import { newTaskPanel, } from './styles-panel.ts';
import {
  darkMode,
  primitiveTokens,
  semanticTokens,
} from './styles-tokens.ts';
import { utilities, } from './styles-utilities.ts';

/**
 Complete global stylesheet string.
 */
export const globalStyles: string = [
  primitiveTokens,
  semanticTokens,
  darkMode,
  resets,
  layoutShell,
  utilities,
  newTaskPanel,
]
  .join('',);
