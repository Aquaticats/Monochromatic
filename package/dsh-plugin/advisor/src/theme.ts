/**
 Plain-text theme implementation of the shared Advisor theme port. @module
 */

import type {
  AdvisorTheme,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

//region Public API

/**
 Identity theme: every shared render passes through unstyled.

 DSH tool and command results carry text `ContentBlock`s only, so the theme
 port has no styled channel; the port-supplied `fg` and `bold` signatures are
 kept and their styling inputs ignored.

 @example
 ```ts
 const summary = renderAdvisorSummary({ result, theme: dshAdvisorTheme });
 ```
 */
export const dshAdvisorTheme: AdvisorTheme = {
  /**
   Render colored terminal text as its plain content.

   @param color - host color token requested by shared rendering

   @param text - text the shared renderer colored

   @returns text unchanged
   */
  fg: function fg(
    color: string,
    text: string,
  ): string {
    void color;
    return text;
  },
  /**
   Render emphasized terminal text as its plain content.

   @param text - text the shared renderer emphasized

   @returns text unchanged
   */
  bold: function bold(text: string): string {
    return text;
  },
};

//endregion Public API
