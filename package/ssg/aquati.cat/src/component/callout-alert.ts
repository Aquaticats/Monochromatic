/**
 Callout-alert component for MDX.
 
 Renders a `<blockquote>` carrying the alert's `data-type`, so generated markup
 names the alert kind the callout was authored as.
 
 @example
 ```mdx
 <callout-alert data-type="note">Important information here.</callout-alert>
 ```
 */
import {
  cssRem,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  i18n,
  type Locale,
} from '../i18n/index.ts';
import type { Label, } from '../i18n/labels-types.ts';
import { icon, } from '../lib/icon/icon.ts';
import {
  jsx,
  type SafeHtml,
} from '../lib/jsx-to-html.ts';
import { GAP_SMALL, } from '../style/constants.ts';

/**
 Valid alert type identifiers.
 */
export type AlertType = 'caution' | 'important' | 'note' | 'tip' | 'warning';

/**
 Props an alert callout renders from.
 */
export type CalloutAlertProps = {
  readonly children: unknown;
  readonly lang: Locale;
  readonly 'data-type': AlertType;
};

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
 Label key per alert type, resolved through the active locale's table.
 */
const ALERT_LABEL_KEYS: Record<AlertType, Label> = {
  caution: 'alertCaution',
  important: 'alertImportant',
  note: 'alertNote',
  tip: 'alertTip',
  warning: 'alertWarning',
};

/**
 Renders an alert callout as a semantic blockquote.
 
 The indicator label comes from the active locale's table, so the callout reads
 in the language of the page that embeds it rather than in English everywhere.
 
 @param props - component props with `lang` (locale the page renders in), `data-type` (alert kind: `note`, `tip`, `important`, `warning`, or `caution`), and `children` (alert content)
 
 @returns rendered alert HTML
 
 @example
 ```ts
 CalloutAlert({ lang: 'ca', 'data-type': 'note', children: 'Atenció.' });
 ```
 */
export function CalloutAlert(props: CalloutAlertProps,): SafeHtml {
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
          {
            children: i18n.label(
              props.lang,
              ALERT_LABEL_KEYS[type],
            ),
          },
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
 Structural styles for the alert component.
 
 Scopes its rule under the `<alert-indicator>` wrapper emitted by
 {@link CalloutAlert}, so unrelated `<blockquote>` elements in MDX content are
 untouched. Colour is left to the page's own text and link tokens; the callout's
 kind is carried by the indicator icon and its localized label.
 
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
