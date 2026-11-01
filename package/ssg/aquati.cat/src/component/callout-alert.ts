/**
 Callout-alert component for MDX.
 
 Renders a `<blockquote>` with a `data-type` attribute for styling.
 
 @example
 ```mdx
 <callout-alert data-type="note">Important information here.</callout-alert>
 ```
 */
import {
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { icon, } from '../lib/icon/icon.ts';
import {
  jsx,
  type SafeHtml,
} from '../lib/jsx-to-html.ts';
import { GAP_SMALL, } from '../style/constants.ts';

/**
 Valid alert type identifiers.
 */
type AlertType = 'caution' | 'important' | 'note' | 'tip' | 'warning';

/**
 Material Symbols PUA codepoint per alert type (resolved at import time from ligature names).
 */
const ALERT_ICONS: Record<AlertType, string> = {
  caution: icon('report',),
  important: icon('priority_high',),
  note: icon('info',),
  tip: icon('lightbulb',),
  warning: icon('warning',),
};

/**
 Human-readable label per alert type.
 */
const ALERT_LABELS: Record<AlertType, string> = {
  caution: 'Caution',
  important: 'Important',
  note: 'Note',
  tip: 'Tip',
  warning: 'Warning',
};

/**
 Renders an alert callout as a semantic blockquote.
 
 @param props - component props with `data-type` (alert kind: `note`, `tip`, `important`, `warning`, or `caution`) and `children` (alert content)
 
 @returns rendered alert HTML
 
 @example
 ```ts
 CalloutAlert({ 'data-type': 'note', children: 'Heads up.' });
 ```
 */
export function CalloutAlert(props: {
  readonly children: unknown;
  readonly 'data-type': AlertType;
},): SafeHtml {
  /**
   Discriminator picking the right icon and label pair from the alert tables.
   */
  const type = props['data-type'];
  /**
   Icon plus label cluster rendered ahead of the alert body.
   */
  const indicator = jsx(
    'alert-indicator',
    {
      'data-is': true,
      children: [
        jsx(
          'span',
          {
            className: 'material-symbols-outlined',
            children: ALERT_ICONS[type],
          },
        ),
        jsx(
          'span',
          { children: ALERT_LABELS[type], },
        ),
      ],
    },
  );
  /**
   Slot wrapper for the user-supplied alert body.
   */
  const content = jsx(
    'alert-content',
    {
      'data-is': true,
      children: props.children,
    },
  );
  /**
   Semantic blockquote root keeping screen readers' alert quote context.
   */
  const blockquote = jsx(
    'blockquote',
    {
      'data-type': type,
      children: [
        indicator,
        content,
      ],
    },
  );
  return jsx(
    'callout-alert',
    {
      'data-is': true,
      children: [blockquote,],
    },
  );
}

//region CSS

/**
 Border thickness for the alert's leading edge, in rem.
 */
const ALERT_BORDER_REM = 0.25;

/**
 Emphasis font-weight for the indicator label.
 */
const INDICATOR_WEIGHT = 600;

/**
 Per-type accent color expression.
 
 Derives from existing link tokens via `color-mix()` so alerts flip with
 the site's dark-mode toggle without introducing new global tokens.
 
 @param token - CSS custom property name supplying the source hue
 
 @returns `color-mix()` CSS expression blending the token toward `color-fg`
 */
function accent(token: string,): string {
  return `color-mix(in oklch, ${cssVar(token,)} 65%, ${cssVar('color-fg',)})`;
}

/**
 Structural and per-type styles for the alert component.
 
 Scopes all rules under `blockquote[data-type]` and the two custom wrappers
 (`alert-indicator`, `alert-content`) emitted by {@link CalloutAlert}. Uses the
 attribute selector so unrelated `<blockquote>` elements in MDX content are
 untouched.
 
 @returns CSS string for the alert component
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'alert-indicator',
      decls: {
        display: 'flex',
        'align-items': 'center',
        gap: cssRem(GAP_SMALL,),
      },
    },),
  ]
    .join('\n',);
}

//endregion CSS
