/**
 CSS rules for syntax highlighting via the CSS Custom Highlight API.
 
 Uses `::highlight(hl-<group>)` pseudo-elements styled by
 `--hl-<group>` custom properties defined in `tokens.ts`.
 Rules are generated from the shared `HIGHLIGHT_GROUPS` array
 so adding a new group automatically creates its CSS rule.
 */

import {
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { HIGHLIGHT_GROUPS, } from '../client/highlight-groups.ts';

/**
 Generates `::highlight()` CSS rules for all syntax highlight groups.
 
 @returns CSS string with `::highlight(hl-*)` rules
 
 @example
 ```ts
 const css = highlightStyles();
 // '::highlight(hl-keyword) { color: var(--hl-keyword); } ...'
 ```
 */
export function highlightStyles(): string {
  return HIGHLIGHT_GROUPS
    .map(function highlightRule(group,) {
      return $({
        rule: `::highlight(hl-${group})`,
        decls: group === 'link'
          ? {
            color: cssVar(`hl-${group}`,),
            'text-decoration-line': 'underline',
          }
          : { color: cssVar(`hl-${group}`,), },
      },);
    },)
    .join('',);
}
