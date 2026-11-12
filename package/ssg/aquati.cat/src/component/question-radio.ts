/**
 MDX single-correct quiz question component.
 
 Renders a single multiple-choice question whose options are radio inputs, so
 picking one answer retires the previous one. The explanation for the picked
 option appears, and picks are never locked; the user can change their answer
 at any time. No JavaScript; all state lives in the `:checked` attribute of the
 native radio inputs.
 
 The shared renderer, id derivation, validation, and stylesheet live in
 `question-group.ts`. This module names the kind. Use
 {@link QuestionCheckbox} for "pick every option that applies".
 
 @example
 ```mdx
 <question-radio
   scenario={<>A "Sign out" button in the top nav.</>}
   options={[
     { label: <>Link</>, explanation: <>Would be CSRF-vulnerable.</> },
     { label: <>Button</>, correct: true, explanation: <>Destructive action.</> },
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
 Renders a single-correct quiz question with CSS-only reveal behavior.
 
 The `fieldset` names itself after a derived scenario paragraph, so a screen
 reader announces the scenario as the name of the option group.
 
 @param props - question content
 
 @returns rendered `<question-radio>` element
 
 @throws if the option count is below two or the number of options marked correct is not exactly one
 
 @example
 ```ts
 QuestionRadio({
   scenario: 'Pick one.',
   options: [
     { label: 'A', explanation: 'because' },
     { label: 'B', correct: true, explanation: 'yes' },
   ],
 });
 ```
 */
export function QuestionRadio(props: QuestionGroupProps,): SafeHtml {
  return renderQuestionGroup({
    ...props,
    input: 'radio',
  },);
}

/**
 State-reveal styles for the single-correct question, scoped under
 `question-radio`.
 
 @returns CSS string for the quiz component
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return questionGroupStyles('radio',);
}
