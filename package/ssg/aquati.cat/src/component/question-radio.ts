/**
 MDX single-correct quiz question component.
 
 Renders a single multiple-choice question with CSS-only state reveal.
 Each option is a radio input. When the user picks any option, the
 explanation for that option appears. Picks are never locked; the
 user can change their answer at any time. No JavaScript; all state
 lives in the `:checked` attribute of the native radio inputs.
 
 Designed to be reusable across quizzes: variable option count, no
 hardcoded A to E letter scheme. Labels and explanations accept full
 MDX/JSX fragments.
 
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
import {
  cssEm,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { fingerprint32, } from 'farmhashjs';

import {
  jsx,
  type SafeHtml,
} from '../lib/jsx-to-html.ts';
import {
  GAP,
  GAP_SMALL,
} from '../style/constants.ts';

//region Types

/**
 One answer choice in a {@link QuestionRadio}.
 */
export type QuestionOption = {
  /**
   Visible text on the radio's label; full MDX/JSX supported.
   */
  readonly label: SafeHtml | string;

  /**
   Per-option explanation shown when this option is chosen; full MDX/JSX supported.
   */
  readonly explanation: SafeHtml | string;

  /**
   Marks the correct answer; exactly one option per question must set this.
   */
  readonly correct?: boolean;
};

/**
 Props for {@link QuestionRadio}.
 */
type QuestionProps = {
  /**
   Scenario prose shown above the options; full MDX/JSX supported.
   */
  readonly scenario: SafeHtml | string;

  /**
   Answer choices; must contain at least two entries with exactly one marked correct.
   */
  readonly options: readonly QuestionOption[];
};

//endregion Types

//region ID derivation

/**
 Hex digit count for a 32-bit hash (padded representation).
 */
const HEX_DIGITS_32 = 8;

/**
 Hex radix for Number.prototype.toString.
 */
const HEX_RADIX = 16;

/**
 Extracts the HTML string from a `SafeHtml` wrapper, or returns the string as-is.
 
 @param value - rendered JSX fragment or plain string
 
 @returns underlying string for hashing
 */
function toHtmlString(value: SafeHtml | string,): string {
  return (typeof value) === 'string' ? value : value.html;
}

/**
 Derives a deterministic radio-group ID from a question's content.
 
 Hashes the scenario, every option label, and each option's correct flag.
 Explanations are intentionally excluded; they can be revised without
 invalidating the ID, which keeps radio group names stable across edits.
 
 @param props - question props
 
 @returns 8-digit hex ID suitable for `name="q-{id}"` input grouping
 */
function deriveQuestionId(props: QuestionProps,): string {
  /**
   Hash input fragments joined with NUL to keep field boundaries unambiguous.
   */
  const parts = [toHtmlString(props.scenario,),];
  for (const opt of props.options) {
    parts.push(
      toHtmlString(opt.label,),
      opt.correct
        === true ? '1' : '0'
    );
  }
  return fingerprint32(
    parts.join('\u0000',),
  )
    .toString(HEX_RADIX,)
    .padStart(
    HEX_DIGITS_32,
    '0',
    );
}

//endregion ID derivation

//region Validation

/**
 Minimum option count for a meaningful multiple-choice question.
 */
const MIN_OPTIONS = 2;

/**
 Required count of options with `correct: true`.
 */
const REQUIRED_CORRECT_COUNT = 1;

/**
 Rejects malformed question props with a descriptive error.
 
 @param props - question props to validate
 
 @throws if option count is below {@link MIN_OPTIONS} or the correct-flag count is not {@link REQUIRED_CORRECT_COUNT}
 */
function validate(props: QuestionProps,): void {
  if (props.options
    .length
    < MIN_OPTIONS) {
    throw new Error(
      `Question needs at least ${MIN_OPTIONS} options; got ${props.options
        .length}.`,
    );
  }
  /**
   Number of options marked correct; must equal REQUIRED_CORRECT_COUNT for radio questions.
   */
  const correctCount = props
    .options
    .filter(function isCorrect(o,) {
      return o.correct
        === true;
    },)
    .length;
  if (correctCount !== REQUIRED_CORRECT_COUNT) {
    throw new Error(
      `Question needs exactly ${REQUIRED_CORRECT_COUNT} correct option; got ${correctCount}.`,
    );
  }
}

//endregion Validation

//region Render

/**
 Renders one option as a radio input + label + hidden explanation.
 
 @param qId - parent question's derived ID (shared `name` across siblings)
 
 @param idx - zero-based option index (unique input `id` suffix)
 
 @param opt - option data
 
 @returns rendered option fragment
 */
function renderOption(
  {
    qId,
    idx,
    opt,
  }: {
    readonly qId: string;
    readonly idx: number;
    readonly opt: QuestionOption;
  },
): SafeHtml {
  /**
   Unique input identifier; linked from the sibling label's htmlFor.
   */
  const inputId = `q-${qId}-${idx}`;
  /**
   Built incrementally so the `data-correct` attribute is set only when applicable.
   */
  const inputProps: Record<string, unknown> = {
    type: 'radio',
    name: `q-${qId}`,
    id: inputId,
  };
  if (opt.correct
    === true)
    inputProps['data-correct'] = '';
  return jsx(
    'div',
    {
      className: 'option',
      children: [
        jsx(
          'input',
          inputProps,
        ),
        jsx(
          'label',
          {
            htmlFor: inputId,
            children: opt.label,
          },
        ),
        jsx(
          'div',
          {
            className: 'explanation',
            children: opt.explanation,
          },
        ),
      ],
    },
  );
}

/**
 Renders a quiz question with CSS-only reveal behavior.
 
 @param props - question props
 
 @returns rendered `<question-radio>` element
 
 @throws if the option count or correct-flag count is invalid
 
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
export function QuestionRadio(props: QuestionProps,): SafeHtml {
  validate(props,);
  /**
   Derived stable id grouping all option inputs of this question.
   */
  const qId = deriveQuestionId(props,);
  return jsx(
    'question-radio',
    {
      'data-is': '',
      children: [
        jsx(
          'p',
          {
            className: 'scenario',
            children: props.scenario,
          },
        ),
        jsx(
          'fieldset',
          {
            children: props.options
              .map(function mapOption(
                opt,
                idx,
              ) {
                return renderOption({
                  qId,
                  idx,
                  opt,
                },);
              },),
          },
        ),
      ],
    },
  );
}

//endregion Render

//region CSS

/**
 Mix ratio for deriving ok/err colors from existing link tokens (percentage toward fg).
 */
const COLOR_MIX_RATIO = '65%';

/**
 Emphasis font-weight for the chosen and revealed labels.
 
 Literal kept out of {@link css} itself so a single point of change propagates
 to any future variant rule.
 */
const EMPHASIS_WEIGHT = 600;

/**
 Slight font-size reduction for the per-option explanation text, in em.
 */
const EXPLANATION_FONT_SIZE_EM = 0.95;

/**
 Percent multiplier for the flex-basis that forces the explanation onto its own row.
 */
const FULL_ROW = 100;

/**
 Structural and state-reveal styles for the quiz question component.
 
 Scoped entirely under `question-radio`. Uses `:has()` and adjacent-sibling
 selectors to show and color explanations based on radio `:checked` state
 without any JavaScript. Correct/wrong hues derive via `color-mix()` from
 existing link tokens, so the quiz flips with the site's dark-mode toggle
 without adding new global tokens.
 
 @returns CSS string for the quiz component
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return $({
    rule: 'question-radio',
    decls: {
      display: 'block',
      'margin-block': cssRem(GAP,),
      '--quiz-ok': `green`,
      '--quiz-err': `red`,
    },
    children: [
      $({
        rule: '.scenario',
        decls: {
          'margin-block-end': cssRem(GAP_SMALL,),
          'font-weight': EMPHASIS_WEIGHT,
        },
      },),
      $({
        rule: '.option',
        decls: {
          display: 'flex',
          'flex-wrap': 'wrap',
          'align-items': 'baseline',
          gap: cssRem(GAP_SMALL,),
          'margin-block': cssRem(GAP_SMALL,),
        },
        children: [
          $({
            rule: 'input',
            decls: { 'flex-shrink': 0, },
          },),
          $({
            rule: 'label',
            decls: { cursor: 'pointer', },
          },),
          $({
            rule: '.explanation',
            decls: {
              display: 'none',
              'flex-basis': cssPercent(FULL_ROW,),
              'margin-inline-start': cssRem(GAP,),
              color: cssVar('color-muted',),
              'font-size': cssEm(EXPLANATION_FONT_SIZE_EM,),
            },
          },),
          $({
            rule: '&:has(input:checked) .explanation',
            decls: { display: 'block', },
          },),
        ],
      },),
      $({
        rule: 'input:checked[data-correct] + label',
        decls: {
          color: cssVar('quiz-ok',),
          'font-weight': EMPHASIS_WEIGHT,
        },
      },),
      $({
        rule: 'input:checked:not([data-correct]) + label',
        decls: {
          color: cssVar('quiz-err',),
          'font-weight': EMPHASIS_WEIGHT,
        },
      },),
    ],
  },);
}

//endregion CSS
