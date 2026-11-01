/**
 Tests for quiz question component ID derivation.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { fingerprint32, } from 'farmhashjs';

import type { SafeHtml, } from '../lib/jsx-to-html.ts';

import {
  QuestionCheckbox,
  css as checkboxCss,
  type QuestionOption as CheckboxOption,
} from './question-checkbox.ts';
import {
  QuestionRadio,
  css as radioCss,
  type QuestionOption as RadioOption,
} from './question-radio.ts';

//region Hash fixtures

/**
 Hex digit count for a padded 32-bit FarmHash fingerprint.
 */
const HEX_DIGITS_32 = 8;

/**
 Hex radix for Number.prototype.toString.
 */
const HEX_RADIX = 16;

/**
 Field boundary separator used by the question ID hash input.
 */
const HASH_FIELD_SEPARATOR = '\u0000';

/**
 Minimal option shape included in question ID hashing.
 */
type HashableQuestionOption = {
  /**
   Rendered option label included in the hash input.
   */
  readonly label: SafeHtml | string;

  /**
   Correctness flag included in the hash input.
   */
  readonly correct?: boolean;
};

/**
 Minimal question shape included in question ID hashing.
 */
type HashableQuestion = {
  /**
   Rendered scenario included in the hash input.
   */
  readonly scenario: SafeHtml | string;

  /**
   Options whose labels and correctness flags are included in the hash input.
   */
  readonly options: readonly HashableQuestionOption[];
};

/**
 Radio question scenario fixture.
 */
const RADIO_SCENARIO = 'Pick the accessible control.';

/**
 Radio question option fixture.
 */
const RADIO_OPTIONS = [
  {
    label: 'Link',
    explanation: 'Navigates rather than submits.',
  },
  {
    label: 'Button',
    correct: true,
    explanation: 'Triggers the action in place.',
  },
] as const satisfies readonly RadioOption[];

/**
 Checkbox question scenario fixture using already-rendered HTML.
 */
const CHECKBOX_SCENARIO = {
  html: '<em>Pick every semantic control.</em>',
} as const satisfies SafeHtml;

/**
 Checkbox question option fixture using a rendered label.
 */
const CHECKBOX_OPTIONS = [
  {
    label: {
      html: '<strong>Button</strong>',
    },
    correct: true,
    explanation: 'Buttons trigger actions.',
  },
  {
    label: 'Checkbox',
    correct: true,
    explanation: 'Checkboxes support multi-select answers.',
  },
  {
    label: 'Div',
    explanation: 'A div has no built-in form semantics.',
  },
] as const satisfies readonly CheckboxOption[];

/**
 Same checkbox question answers with rewritten explanations.
 */
const CHECKBOX_OPTIONS_WITH_REWRITTEN_EXPLANATIONS = [
  {
    label: {
      html: '<strong>Button</strong>',
    },
    correct: true,
    explanation: 'Rewritten button explanation.',
  },
  {
    label: 'Checkbox',
    correct: true,
    explanation: 'Rewritten checkbox explanation.',
  },
  {
    label: 'Div',
    explanation: 'Rewritten div explanation.',
  },
] as const satisfies readonly CheckboxOption[];

//endregion Hash fixtures

//region Invalid fixtures

/**
 Invalid radio question with too few options.
 */
const TOO_FEW_RADIO_OPTIONS = [
  {
    label: 'Only option',
    correct: true,
    explanation: 'Invalid because there is no alternative.',
  },
] as const satisfies readonly RadioOption[];

/**
 Invalid radio question with no correct option.
 */
const NO_CORRECT_RADIO_OPTIONS = [
  {
    label: 'Link',
    explanation: 'Not the answer.',
  },
  {
    label: 'Div',
    explanation: 'Also not the answer.',
  },
] as const satisfies readonly RadioOption[];

/**
 Invalid checkbox question with too few options.
 */
const TOO_FEW_CHECKBOX_OPTIONS = [
  {
    label: 'Only option',
    correct: true,
    explanation: 'Invalid because there is no alternative.',
  },
] as const satisfies readonly CheckboxOption[];

/**
 Invalid checkbox question with no correct option.
 */
const NO_CORRECT_CHECKBOX_OPTIONS = [
  {
    label: 'Link',
    explanation: 'Not correct.',
  },
  {
    label: 'Div',
    explanation: 'Also not correct.',
  },
] as const satisfies readonly CheckboxOption[];

//endregion Invalid fixtures

//region Helpers

/**
 Extracts rendered HTML string from a SafeHtml wrapper, or returns plain text unchanged.
 
 @param value - rendered fixture or plain text.
 
 @returns String used as hash input.
 
 @example
 ```ts
 toHtmlString('plain');
 ```
 */
function toHtmlString(value: SafeHtml | string,): string {
  return (typeof value) === 'string' ? value : value.html;
}

/**
 Computes expected FarmHash-backed question ID for a fixture.
 
 @param question - question fixture to hash.
 
 @returns Padded hexadecimal question ID.
 
 @example
 ```ts
 expectedQuestionId({ scenario: 'Pick one', options: RADIO_OPTIONS });
 ```
 */
function expectedQuestionId(question: HashableQuestion,): string {
  /**
   Hash input fragments in the same order as production code.
   */
  const parts = [
    toHtmlString(question.scenario,),
    ...question.options.flatMap(function optionHashFields(option,): readonly string[] {
      return [
        toHtmlString(option.label,),
        option.correct === true ? '1' : '0',
      ];
    },),
  ];

  return fingerprint32(parts.join(HASH_FIELD_SEPARATOR,),)
    .toString(HEX_RADIX,)
    .padStart(
    HEX_DIGITS_32,
    '0',
    );
}

/**
 Captures a synchronous component validation error.
 
 @param render - render call expected to throw.
 
 @returns Captured error.
 
 @throws If the render call does not throw an Error instance.
 
 @example
 ```ts
 captureRenderError(() => QuestionRadio({ scenario: 'x', options: [] }));
 ```
 */
function captureRenderError(render: () => SafeHtml,): Error {
  /**
   Value thrown by the render call, or the returned HTML if no error occurred.
   */
  const caught = (function catchRenderError(): unknown {
    try {
      return render();
    }
    catch (error) {
      return error;
    }
  })();

  if (Error.isError(caught,))
    return caught;

  throw new Error('Expected component render to throw an Error instance.',);
}

//endregion Helpers

await describe({
  name: 'quiz question components',
  children: [
    it({
      name: 'formats radio input group IDs from FarmHash32',
      fn: async function formatsRadioInputGroupIdsFromFarmHash32(): Promise<void> {
        /**
         Expected padded hex ID derived from farmhashjs fingerprint32.
         */
        const expectedId = expectedQuestionId({
          scenario: RADIO_SCENARIO,
          options: RADIO_OPTIONS,
        },);
        /**
         Rendered radio question HTML.
         */
        const { html, } = QuestionRadio({
          scenario: RADIO_SCENARIO,
          options: RADIO_OPTIONS,
        },);

        expect(html,).toContain(`name="q-${expectedId}"`,);
        expect(html,).toContain(`id="q-${expectedId}-0"`,);
      },
    },),
    it({
      name: 'formats checkbox input group IDs from FarmHash32',
      fn: async function formatsCheckboxInputGroupIdsFromFarmHash32(): Promise<void> {
        /**
         Expected padded hex ID derived from farmhashjs fingerprint32.
         */
        const expectedId = expectedQuestionId({
          scenario: CHECKBOX_SCENARIO,
          options: CHECKBOX_OPTIONS,
        },);
        /**
         Rendered checkbox question HTML.
         */
        const { html, } = QuestionCheckbox({
          scenario: CHECKBOX_SCENARIO,
          options: CHECKBOX_OPTIONS,
        },);

        expect(html,).toContain(`name="q-${expectedId}"`,);
        expect(html,).toContain(`id="q-${expectedId}-0"`,);
      },
    },),
    it({
      name: 'keeps checkbox IDs stable when explanations change',
      fn: async function keepsCheckboxIdsStableWhenExplanationsChange(): Promise<void> {
        /**
         Expected ID from fields that should affect grouping.
         */
        const expectedId = expectedQuestionId({
          scenario: CHECKBOX_SCENARIO,
          options: CHECKBOX_OPTIONS,
        },);
        /**
         Original checkbox question HTML.
         */
        const { html: originalHtml, } = QuestionCheckbox({
          scenario: CHECKBOX_SCENARIO,
          options: CHECKBOX_OPTIONS,
        },);
        /**
         Checkbox question HTML after explanation-only edits.
         */
        const { html: rewrittenHtml, } = QuestionCheckbox({
          scenario: CHECKBOX_SCENARIO,
          options: CHECKBOX_OPTIONS_WITH_REWRITTEN_EXPLANATIONS,
        },);

        expect(originalHtml,).toContain(`name="q-${expectedId}"`,);
        expect(rewrittenHtml,).toContain(`name="q-${expectedId}"`,);
      },
    },),
    it({
      name: 'rejects invalid radio questions',
      fn: async function rejectsInvalidRadioQuestions(): Promise<void> {
        /**
         Error for a radio question with too few options.
         */
        const tooFewError = captureRenderError(function renderTooFewRadioOptions(): SafeHtml {
          return QuestionRadio({
            scenario: RADIO_SCENARIO,
            options: TOO_FEW_RADIO_OPTIONS,
          },);
        },);
        /**
         Error for a radio question without exactly one correct option.
         */
        const noCorrectError = captureRenderError(function renderNoCorrectRadioOptions(): SafeHtml {
          return QuestionRadio({
            scenario: RADIO_SCENARIO,
            options: NO_CORRECT_RADIO_OPTIONS,
          },);
        },);

        expect(tooFewError.message,).toContain('at least 2 options',);
        expect(noCorrectError.message,).toContain('exactly 1 correct option',);
      },
    },),
    it({
      name: 'rejects invalid checkbox questions',
      fn: async function rejectsInvalidCheckboxQuestions(): Promise<void> {
        /**
         Error for a checkbox question with too few options.
         */
        const tooFewError = captureRenderError(function renderTooFewCheckboxOptions(): SafeHtml {
          return QuestionCheckbox({
            scenario: CHECKBOX_SCENARIO,
            options: TOO_FEW_CHECKBOX_OPTIONS,
          },);
        },);
        /**
         Error for a checkbox question without any correct options.
         */
        const noCorrectError = captureRenderError(function renderNoCorrectCheckboxOptions(): SafeHtml {
          return QuestionCheckbox({
            scenario: CHECKBOX_SCENARIO,
            options: NO_CORRECT_CHECKBOX_OPTIONS,
          },);
        },);

        expect(tooFewError.message,).toContain('at least 2 options',);
        expect(noCorrectError.message,).toContain('at least 1 correct option',);
      },
    },),
    it({
      name: 'keeps component CSS scoped to each custom element',
      fn: async function keepsComponentCssScopedToEachCustomElement(): Promise<void> {
        /**
         Radio component CSS.
         */
        const radioStyles = radioCss();
        /**
         Checkbox component CSS.
         */
        const checkboxStyles = checkboxCss();

        expect(radioStyles,).toContain('question-radio',);
        expect(checkboxStyles,).toContain('question-checkbox',);
      },
    },),
  ],
},);
