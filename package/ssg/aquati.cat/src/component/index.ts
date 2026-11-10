/**
 MDX component registry.
 
 Every name this module returns is automatically available in MDX content
 without an import statement. To add a new component, create a `.ts` file in
 this directory and add it to {@link createMdxComponents}.
 
 Authors write the capitalized identifier in MDX source
 (e.g. `<CalloutAlert>`); the component function renders a
 custom-element-compliant tag in the output HTML
 (e.g. `<callout-alert data-is>...</callout-alert>`). This split is
 required because MDX only consults the components map for
 capitalized tag names; lowercase hyphenated JSX tags compile to
 raw HTML element strings.
 
 The map is built per render rather than exported as a constant, because some
 components render localized chrome: a callout's indicator label is chosen from
 the label table of the locale the page is written in, so the same MDX body
 produces different HTML in a Catalan and a Chinese post.
 */
import type { Locale, } from '../i18n/index.ts';

import {
  CalloutAlert,
  type CalloutAlertProps,
} from './callout-alert.ts';
import { QuestionCheckbox, } from './question-checkbox.ts';
import { QuestionRadio, } from './question-radio.ts';
import { ShuffleChildren, } from './shuffle-children.ts';

/**
 Props an MDX callout supplies; the locale is bound by {@link createMdxComponents}.
 */
type MdxCalloutProps = Omit<CalloutAlertProps, 'lang'>;

/**
 Component map MDX resolves capitalized tag names against.
 */
export type MdxComponents = {
  readonly CalloutAlert: (props: MdxCalloutProps,) => ReturnType<typeof CalloutAlert>;
  readonly QuestionCheckbox: typeof QuestionCheckbox;
  readonly QuestionRadio: typeof QuestionRadio;
  readonly ShuffleChildren: typeof ShuffleChildren;
};

/**
 Builds the MDX component map for one locale.
 
 @param lang - locale the page being rendered is written in
 
 @returns component map whose entries close over {@link lang}
 
 @example
 ```ts
 const components = createMdxComponents('ca');
 components.CalloutAlert({ 'data-type': 'note', children: 'Atenció.' });
 ```
 */
export function createMdxComponents(lang: Locale,): MdxComponents {
  return {
    CalloutAlert: function calloutInLocale(props: MdxCalloutProps,) {
      return CalloutAlert({
        ...props,
        lang,
      },);
    },
    QuestionCheckbox,
    QuestionRadio,
    ShuffleChildren,
  };
}
