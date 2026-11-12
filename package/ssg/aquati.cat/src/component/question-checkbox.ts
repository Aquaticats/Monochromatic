/**
 MDX multi-correct quiz question component.
 
 Renders a single question whose options are independent checkboxes, so the
 user can toggle any combination. When an option is checked its explanation
 appears and the label turns green if the option is right or red if it is
 wrong. No JavaScript; all state lives in the `:checked` attribute of the
 native checkbox inputs.
 
 The shared renderer, id derivation, validation, and stylesheet live in
 `question-group.ts`. This module names the kind. Use this variant when the
 lesson is "pick every option that applies" rather than "pick the single best
 answer"; use {@link QuestionRadio} for single-correct questions.
 
 @example
 ```mdx
 <question-checkbox
   scenario={<>A shuffle toggle in a music player.</>}
   options={[
     { label: <>Button</>, correct: true, explanation: <>…</> },
     { label: <>Something else</>, correct: true, explanation: <>…</> },
     { label: <>Link</>, explanation: <>…</> },
   ]}
 />
 ```
 */
import type { SafeHtml, } from '../lib/jsx-to-html.ts';
import {
  type QuestionGroupProps,
  questionGroupStyles,
  renderQuestionGroup,
} from './question-group.ts';

/**
 Renders a multi-correct quiz question with CSS-only reveal behavior.
 
 The `fieldset` names itself after a derived scenario paragraph, so a screen
 reader announces the scenario as the name of the option group.
 
 @param props - question content
 
 @returns rendered `<question-checkbox>` element
 
 @throws if the option count is below two or no option is marked correct
 
 @example
 ```ts
 QuestionCheckbox({
   scenario: 'Pick every option that applies.',
   options: [
     { label: 'A', correct: true, explanation: 'yes' },
     { label: 'B', correct: true, explanation: 'also yes' },
     { label: 'C', explanation: 'no' },
   ],
 });
 ```
 */
export function QuestionCheckbox(props: QuestionGroupProps,): SafeHtml {
  return renderQuestionGroup({
    ...props,
    input: 'checkbox',
  },);
}

/**
 State-reveal styles for the multi-correct question, scoped under
 `question-checkbox`.
 
 @returns CSS string for the multi-correct component
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return questionGroupStyles('checkbox',);
}
