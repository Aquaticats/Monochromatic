/**
 Tests for the renderer and stylesheet generator the single-correct and
 multi-correct quiz components delegate to.

 Both kinds come from one function, so the tests below hold every kind to the
 same shape and pin the two places they are allowed to differ: the custom
 element name and the `type` on each option input.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  questionGroupStyles,
  renderQuestionGroup,
  type QuestionGroupOption,
  type SafeHtml,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Scenario fixture naming the question its options answer.
 */
const SCENARIO = 'Pick the accessible control.';

/**
 Option fixture both kinds render: one wrong answer and one right answer.
 */
const OPTIONS = [
  {
    label: 'Link',
    explanation: 'Navigates rather than submits.',
  },
  {
    label: 'Button',
    correct: true,
    explanation: 'Triggers the action in place.',
  },
] as const satisfies readonly QuestionGroupOption[];

/**
 Option fixture marking both answers right, which only the multi-correct kind
 accepts.
 */
const TWO_CORRECT_OPTIONS = [
  {
    label: 'Button',
    correct: true,
    explanation: 'Buttons trigger actions.',
  },
  {
    label: 'Checkbox',
    correct: true,
    explanation: 'Checkboxes support multi-select answers.',
  },
] as const satisfies readonly QuestionGroupOption[];

/**
 Every input kind the shared renderer serves.
 */
const INPUT_KINDS = [
  'radio',
  'checkbox',
] as const;

//endregion Fixtures

//region Helpers

/**
 Reads the group name rendered question HTML puts on every option input.
 
 @param html - rendered question group
 
 @returns the shared input name, or the empty string when the HTML carries none
 
 @example
 ```ts
 inputGroupName('<input name="q-0123abcd">');
 // 'q-0123abcd'
 ```
 */
function inputGroupName(html: string,): string {
  /**
   First rendered option input's group name, matched once at its first occurrence.
   */
  const matched = /name="(q-[0-9a-f]+)"/u.exec(html,);
  return matched?.[1] ?? '';
}

/**
 Counts the occurrences of a marker inside rendered HTML.
 
 @param html - rendered question group
 
 @param marker - exact text to count
 
 @returns how many times the marker appears
 
 @example
 ```ts
 countOf('<input type="radio"><input type="radio">', 'type="radio"');
 // 2
 ```
 */
function countOf(html: string, marker: string,): number {
  return html.split(marker,).length - 1;
}

/**
 Captures the error a render call throws.
 
 @param render - render call expected to throw
 
 @returns the captured error
 
 @throws if the render call does not throw an `Error` instance
 
 @example
 ```ts
 captureRenderError(() => renderQuestionGroup({ input: 'radio', scenario: 'x', options: [] }));
 ```
 */
function captureRenderError(render: () => SafeHtml,): Error {
  /**
   Value thrown by the render call, or the returned HTML when it threw nothing.
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

  throw new Error('Expected the render call to throw an Error instance.',);
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: renderQuestionGroup.name,
      children: [
        //region Element and input kinds

        ...INPUT_KINDS.map(function mapInputKind(input,) {
          return it({
            name: `renders the ${input} custom element for its kind`,
            fn: async function rendersKindElement(): Promise<void> {
              /**
               Rendered question group of one input kind.
               */
              const { html, } = renderQuestionGroup({
                input,
                scenario: SCENARIO,
                options: OPTIONS,
              },);

              expect(html,).toContain(`<question-${input} data-is="">`,);
              expect(html,).toContain('</p><fieldset aria-labelledby="q-',);
            },
          },);
        },),
        ...INPUT_KINDS.map(function mapInputKind(input,) {
          return it({
            name: `marks every option input as ${input}`,
            fn: async function marksOptionInputs(): Promise<void> {
              /**
               Rendered question group of one input kind.
               */
              const { html, } = renderQuestionGroup({
                input,
                scenario: SCENARIO,
                options: OPTIONS,
              },);

              expect(countOf(
                html,
                `type="${input}"`,
              ),).toBe(OPTIONS.length,);
            },
          },);
        },),

        //endregion Element and input kinds

        //region Shared derivation

        it({
          name: 'derives one group name for both kinds',
          fn: async function derivesOneGroupName(): Promise<void> {
            /**
             Rendered single-correct question.
             */
            const radio = renderQuestionGroup({
              input: 'radio',
              scenario: SCENARIO,
              options: OPTIONS,
            },);
            /**
             Rendered multi-correct question over the same content.
             */
            const checkbox = renderQuestionGroup({
              input: 'checkbox',
              scenario: SCENARIO,
              options: OPTIONS,
            },);
            /**
             Group name the single-correct question assigned.
             */
            const radioGroup = inputGroupName(radio.html,);

            expect(radioGroup,).toMatch(/^q-[0-9a-f]{8}$/u,);
            expect(checkbox.html,).toContain(`name="${radioGroup}"`,);
          },
        },),
        it({
          name: 'marks only the right answers as correct',
          fn: async function marksRightAnswers(): Promise<void> {
            /**
             Rendered single-correct question.
             */
            const { html, } = renderQuestionGroup({
              input: 'radio',
              scenario: SCENARIO,
              options: OPTIONS,
            },);
            /**
             How many options the fixture marks correct.
             */
            const correctCount = OPTIONS
              .filter(function isCorrect(option,) {
                return option.correct === true;
              },)
              .length;

            expect(countOf(
              html,
              'data-correct=""',
            ),).toBe(correctCount,);
          },
        },),

        //endregion Shared derivation

        //region Validation

        it({
          name: 'rejects a single-correct question with two right answers',
          fn: async function rejectsTwoCorrectRadioOptions(): Promise<void> {
            /**
             Error for a question whose kind accepts one right answer but renders two.
             */
            const error = captureRenderError(function renderTwoCorrectRadioOptions(): SafeHtml {
              return renderQuestionGroup({
                input: 'radio',
                scenario: SCENARIO,
                options: TWO_CORRECT_OPTIONS,
              },);
            },);

            expect(error.message,).toContain('exactly 1 correct option',);
          },
        },),
        it({
          name: 'accepts a multi-correct question with two right answers',
          fn: async function acceptsTwoCorrectCheckboxOptions(): Promise<void> {
            /**
             Rendered multi-correct question whose options both count as right.
             */
            const { html, } = renderQuestionGroup({
              input: 'checkbox',
              scenario: SCENARIO,
              options: TWO_CORRECT_OPTIONS,
            },);

            expect(countOf(
              html,
              'data-correct=""',
            ),).toBe(TWO_CORRECT_OPTIONS.length,);
          },
        },),

        //endregion Validation
      ],
    },),
    describe({
      name: questionGroupStyles.name,
      children: [
        ...INPUT_KINDS.map(function mapInputKind(input,) {
          return it({
            name: `scopes the stylesheet to the ${input} element`,
            fn: async function scopesStylesToKind(): Promise<void> {
              /**
               Stylesheet generated for one input kind.
               */
              const styles = questionGroupStyles(input,);
              /**
               The other element the stylesheet must leave alone.
               */
              const otherKind = input === 'radio' ? 'checkbox' : 'radio';

              expect(styles,).toContain(`question-${input}{`,);
              expect(styles,).not.toContain(`question-${otherKind}`,);
            },
          },);
        },),
      ],
    },),
  ],
},);
