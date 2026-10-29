/**
 MDX multi-correct quiz question component.
 
 Renders a single question with one or more correct answers, using
 CSS-only state reveal. Each option is an independent checkbox; the
 user can toggle any combination. When an option is checked, its
 explanation appears and the label turns green if the option is
 correct or red if it is wrong. No JavaScript; all state lives in
 the `:checked` attribute of the native checkbox inputs.
 
 Use this variant when the lesson is "pick every option that applies"
 rather than "pick the single best answer." Use {@link QuestionRadio}
 for single-correct questions.
 
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
 One answer choice in a {@link QuestionCheckbox}.
 */
export type QuestionOption = {
  /**
   Visible text on the checkbox's label; full MDX/JSX supported.
   */
  readonly label: SafeHtml | string;

  /**
   Per-option explanation shown when this option is checked; full MDX/JSX supported.
   */
  readonly explanation: SafeHtml | string;

  /**
   Marks a correct answer; at least one option per question must set this.
   */
  readonly correct?: boolean;
};

/**
 Props for {@link QuestionCheckbox}.
 */
type QuestionProps = {
  /**
   Scenario prose shown above the options; full MDX/JSX supported.
   */
  readonly scenario: SafeHtml | string;

  /**
   Answer choices; must contain at least two entries with at least one marked correct.
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
 Extracts HTML string from a `SafeHtml` wrapper, or returns the string as-is.
 
 @param value - rendered JSX fragment or plain string
 
 @returns underlying string for hashing
 */
function toHtmlString(value: SafeHtml | string,): string {
  return (typeof value) === 'string' ? value : value.html;
}

/**
 Derives a deterministic group ID from a question's content.
 
 Hashes scenario, every option label, and each option's correct flag.
 Explanations are intentionally excluded so they can be revised without
 invalidating the ID.
 
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
 Minimum count of options that must be marked `correct: true`.
 */
const MIN_CORRECT_COUNT = 1;

/**
 Rejects malformed question props with a descriptive error.
 
 @param props - question props to validate
 
 @throws if option count is below {@link MIN_OPTIONS} or correct count is below {@link MIN_CORRECT_COUNT}
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
   Number of options marked correct; checked against MIN_CORRECT_COUNT for validity.
   */
  const correctCount = props
    .options
    .filter(function isCorrect(o,) {
      return o.correct
        === true;
    },)
    .length;
  if (correctCount < MIN_CORRECT_COUNT) {
    throw new Error(
      `Question needs at least ${MIN_CORRECT_COUNT} correct option; got ${correctCount}.`,
    );
  }
}

//endregion Validation

//region Render

/**
 Renders one option as a checkbox input + label + hidden explanation.
 
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
    type: 'checkbox',
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
 Renders a multi-correct quiz question with CSS-only reveal behavior.
 
 @param props - question props
 
 @returns rendered `<question-checkbox>` element
 
 @throws if option count or correct-flag count is invalid
 
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
export function QuestionCheckbox(props: QuestionProps,): SafeHtml {
  validate(props,);
  /**
   Derived stable id grouping all option inputs of this question.
   */
  const qId = deriveQuestionId(props,);
  return jsx(
    'question-checkbox',
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
 Emphasis font-weight for the checked and revealed labels.
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
 Structural and state-reveal styles for the multi-correct question component.
 
 Scoped entirely under `question-checkbox`. Uses `:has()` and adjacent-sibling
 selectors to show and color explanations based on checkbox `:checked` state
 without any JavaScript. Matches {@link QuestionRadio} visually so the two
 variants feel consistent in a mixed quiz.
 
 @returns CSS string for the multi-correct component
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return $({
    rule: 'question-checkbox',
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
