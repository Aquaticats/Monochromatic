/**
 Shared implementation of the CSS-only quiz question components.
 
 A question renders a scenario, its answer options as native inputs, and one
 explanation per option that CSS reveals from the `:checked` state. The two
 kinds this package ships, single-correct and multi-correct, differ in three
 places only: the custom element they render, the `type` of their option
 inputs, and how many options an author may mark correct. Everything else, the
 content-derived group id, the option skeleton, the validation of the option
 count, and the stylesheet, lives here so the two kinds cannot drift apart.
 
 MDX never names this module. `question-radio.ts` and `question-checkbox.ts`
 name their kind and delegate here.
 
 @module
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
  FONT_WEIGHT_EMPHASIS,
  GAP,
  GAP_SMALL,
} from '../style/constants.ts';

//region Types

/**
 Native input element a question's options render as, which also names the
 custom element and decides how many options may be marked correct.
 */
export type QuestionInput = 'radio' | 'checkbox';

/**
 One answer choice in a question.
 */
export type QuestionOption = {
  /**
   Visible text on the option's label; full MDX/JSX supported.
   */
  readonly label: SafeHtml | string;

  /**
   Per-option explanation shown when this option is chosen; full MDX/JSX supported.
   */
  readonly explanation: SafeHtml | string;

  /**
   Marks a correct answer; how many options may set this depends on the question kind.
   */
  readonly correct?: boolean;
};

/**
 The content of a question, independent of the kind that renders it.
 */
export type QuestionGroupProps = {
  /**
   Scenario prose shown above the options; full MDX/JSX supported.
   */
  readonly scenario: SafeHtml | string;

  /**
   Answer choices; must contain at least two entries.
   */
  readonly options: readonly QuestionOption[];
};

/**
 One request to render a question: its content plus the kind rendering it.
 */
export type QuestionGroupRequest = QuestionGroupProps & {
  /**
   Kind of question to render.
   */
  readonly input: QuestionInput;
};

/**
 How a question kind constrains the number of options an author marks correct.
 */
type CorrectCountRule = {
  /**
   Phrase naming the accepted count, completed by the error message.
   */
  readonly description: string;

  /**
   Reports whether a question carrying this many correct options is valid.
   */
  readonly accepts: (correctCount: number,) => boolean;
};

//endregion Types

//region Question kinds

/**
 Correct-option count a single-correct question accepts.
 */
const SINGLE_CORRECT_OPTION = 1;

/**
 Correct-option count a multi-correct question requires at least.
 */
const FIRST_CORRECT_OPTION = 1;

/**
 Count rules each question kind applies, keyed by the input element it renders.
 */
const CORRECT_COUNT_RULES: Readonly<Record<QuestionInput, CorrectCountRule>> = {
  radio: {
    description: `exactly ${String(SINGLE_CORRECT_OPTION,)}`,
    accepts: function acceptsSingleCorrectOption(correctCount,) {
      return correctCount === SINGLE_CORRECT_OPTION;
    },
  },
  checkbox: {
    description: `at least ${String(FIRST_CORRECT_OPTION,)}`,
    accepts: function acceptsAnyCorrectOption(correctCount,) {
      return correctCount >= FIRST_CORRECT_OPTION;
    },
  },
};

/**
 Derives the custom element a question kind renders.
 
 The renderer and the stylesheet both scope to this name, so deriving it once
 keeps the two in step.

 @param input - kind of question being rendered

 @returns custom element name for that kind

 @example
 ```ts
 questionElementName('radio');
 // 'question-radio'
 ```
 */
function questionElementName(input: QuestionInput,): string {
  return `question-${input}`;
}

//endregion Question kinds

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
 Derives a deterministic question-group ID from a question's content.
 
 Hashes the scenario, every option label, and each option's correct flag. The
 kind is deliberately excluded, so the same content produces the same ID
 whichever kind renders it. Explanations are excluded too; they can be revised
 without invalidating the ID, which keeps radio group names stable across
 edits.
 
 @param props - question content
 
 @returns 8-digit hex ID suitable for `name="q-{id}"` input grouping
 */
function deriveQuestionId({ scenario, options, }: QuestionGroupProps,): string {
  /**
   Hash input fragments joined with NUL to keep field boundaries unambiguous.
   */
  const parts = [toHtmlString(scenario,),];
  for (const opt of options) {
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
 Rejects malformed question content with a descriptive error.
 
 @param input - kind of question being rendered, which fixes the correct-option rule
 
 @param options - answer choices to validate
 
 @throws if the option count is below {@link MIN_OPTIONS} or the correct-flag count violates the kind's rule
 */
function validateQuestionGroup(
  {
    input,
    options,
  }: {
    readonly input: QuestionInput;
    readonly options: readonly QuestionOption[];
  },
): void {
  if (options.length
    < MIN_OPTIONS) {
    throw new Error(
      `Question needs at least ${MIN_OPTIONS} options; got ${options.length}.`,
    );
  }
  /**
   Number of options marked correct, checked against the kind's count rule.
   */
  const correctCount = options
    .filter(function isCorrect(o,) {
      return o.correct
        === true;
    },)
    .length;
  /**
   Count rule the rendered kind applies.
   */
  const rule = CORRECT_COUNT_RULES[input];
  if (!rule.accepts(correctCount,)) {
    throw new Error(
      `Question needs ${rule.description} correct option; got ${correctCount}.`,
    );
  }
}

//endregion Validation

//region Render

/**
 Renders one option as an input + label + hidden explanation.
 
 @param input - kind of question being rendered, which fixes the input's `type`
 
 @param qId - parent question's derived ID (shared `name` across siblings)
 
 @param idx - zero-based option index (unique input `id` suffix)
 
 @param opt - option data
 
 @returns rendered option fragment
 */
function renderOption(
  {
    input,
    qId,
    idx,
    opt,
  }: {
    readonly input: QuestionInput;
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
    type: input,
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
 
 The outer `<p>` carries a derived `id` and the option `<fieldset>` points at
 it with `aria-labelledby`, so a screen reader announces the scenario as the
 name of the option group instead of leaving it unnamed.
 
 @param request - question content plus the kind rendering it
 
 @returns rendered question custom element
 
 @throws if the option count or correct-flag count is invalid for the kind
 
 @example
 ```ts
 renderQuestionGroup({
   input: 'radio',
   scenario: 'Pick one.',
   options: [
     { label: 'A', explanation: 'because' },
     { label: 'B', correct: true, explanation: 'yes' },
   ],
 });
 ```
 */
export function renderQuestionGroup(
  {
    input,
    scenario,
    options,
  }: QuestionGroupRequest,
): SafeHtml {
  validateQuestionGroup({
    input,
    options,
  },);
  /**
   Derived stable id grouping all option inputs of this question.
   */
  const qId = deriveQuestionId({
    scenario,
    options,
  },);
  /**
   Identifier the question's fieldset points at, so the scenario paragraph names the option group.
   */
  const scenarioId = `q-${qId}-scenario`;
  return jsx(
    questionElementName(input,),
    {
      'data-is': '',
      children: [
        jsx(
          'p',
          {
            className: 'scenario',
            id: scenarioId,
            children: scenario,
          },
        ),
        jsx(
          'fieldset',
          {
            'aria-labelledby': scenarioId,
            children: options
              .map(function mapOption(
                opt,
                idx,
              ) {
                return renderOption({
                  input,
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
 Slight font-size reduction for the per-option explanation text, in em.
 */
const EXPLANATION_FONT_SIZE_EM = 0.95;

/**
 Percent multiplier for the flex-basis that forces the explanation onto its own row.
 */
const FULL_ROW = 100;

/**
 Structural and state-reveal styles for a quiz question.
 
 Scoped entirely under the kind's own custom element. Uses `:has()` and
 adjacent-sibling selectors to show and color explanations based on the
 `:checked` state without any JavaScript. Correct and wrong hues derive from
 the local `--quiz-ok` / `--quiz-err` custom properties, so a quiz flips with
 the site's dark-mode toggle without adding new global tokens.
 
 @param input - kind of question whose element the stylesheet scopes to
 
 @returns CSS string for the quiz component
 
 @example
 ```ts
 const styles = questionGroupStyles('radio');
 ```
 */
export function questionGroupStyles(input: QuestionInput,): string {
  return $({
    rule: questionElementName(input,),
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
          'font-weight': FONT_WEIGHT_EMPHASIS,
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
          'font-weight': FONT_WEIGHT_EMPHASIS,
        },
      },),
      $({
        rule: 'input:checked:not([data-correct]) + label',
        decls: {
          color: cssVar('quiz-err',),
          'font-weight': FONT_WEIGHT_EMPHASIS,
        },
      },),
    ],
  },);
}

//endregion CSS
