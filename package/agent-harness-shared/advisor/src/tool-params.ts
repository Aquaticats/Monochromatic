/**
 Advisor tool parameter schema and argument normalization.
 
 @module
 */

import {
  type TObject,
  type TOptional,
  type TString,
  Type,
} from 'typebox';
import type { AdvisorToolParams, } from './types.ts';

//region Schema

/**
 TypeBox object builder aliased to satisfy constructor-style lint.
 */
const typeObject = Type.Object;

/**
 TypeBox optional builder aliased to satisfy constructor-style lint.
 */
const typeOptional = Type.Optional;

/**
 TypeBox string builder aliased to satisfy constructor-style lint.
 */
const typeString = Type.String;

/**
 TypeBox schema for Advisor tool parameters.
 */
export const AdvisorToolParametersSchema: TObject<{
  model: TOptional<TString>;
  question: TOptional<TString>;
}> = typeObject(
  {
    model: typeOptional(typeString({
      description:
        'Optional scoped model slug. Use provider/model for canonical slugs. Empty params select the highest expected-cost non-current scoped model when possible.',
    },),),
    question: typeOptional(typeString({
      description:
        'Optional focused question or review request for Advisor to answer using the conversation context.',
    },),),
  },
  { additionalProperties: false, },
);

//endregion Schema

//region Argument normalization

/**
 Normalize optional model argument.
 
 @param rawModel - raw model value
 
 @returns model param when non-blank
 */
function normalizeModelArgument(
  rawModel: unknown,
): Pick<AdvisorToolParams, 'model'> {
  if ((typeof rawModel) !== 'string')
    return {};
  if (rawModel.trim()
    === '')
    return {};
  return { model: rawModel, };
}

/**
 Normalize optional focused-question argument.
 
 @param rawQuestion - raw focused-question value
 
 @returns question param when non-blank
 */
function normalizeQuestionArgument(
  rawQuestion: unknown,
): Pick<AdvisorToolParams, 'question'> {
  if ((typeof rawQuestion) !== 'string')
    return {};
  /**
   Trimmed focused question text.
   */
  const question = rawQuestion.trim();
  if (question === '')
    return {};
  return { question, };
}

/**
 Normalize raw tool arguments before TypeBox validation.
 
 @param args - raw tool-call arguments from the model
 
 @returns normalized Advisor params
 
 @throws when non-empty args are malformed
 
 @example
 ```typescript
 prepareAdvisorArguments('anthropic/claude-sonnet');
 ```
 */
export function prepareAdvisorArguments(
  args: unknown,
): AdvisorToolParams {
  if ((args === undefined) || (args === null))
    return {};
  if ((typeof args) === 'string')
    return args.trim()
      === '' ? {} : { model: args, };
  if ((typeof args) !== 'object')
    throw new Error('advisor: arguments must be an object with optional model field',);

  /**
   Unknown fields not accepted by public tool contract.
   */
  const extraKeys = Object
    .keys(args,)
    .filter(function isExtraKey(key,) {
      return (key !== 'model') && (key !== 'question');
    },);
  if (extraKeys.length
    > 0)
    throw new Error(`advisor: unsupported argument fields: ${extraKeys.join(', ',)}`,);

  /**
   Raw model value read after object narrowing.
   */
  const rawModel = 'model' in args
    ? args.model
    : undefined;
  /**
   Raw question value read after object narrowing.
   */
  const rawQuestion = 'question' in args
    ? args.question
    : undefined;

  if ((rawModel !== undefined) && ((typeof rawModel) !== 'string'))
    throw new Error('advisor: model must be a string when provided',);
  if ((rawQuestion !== undefined) && ((typeof rawQuestion) !== 'string'))
    throw new Error('advisor: question must be a string when provided',);

  /**
   Model parameter carried through after existing string compatibility rules.
   */
  const normalizedModel = normalizeModelArgument(rawModel,);
  /**
   Trimmed focused question, omitted when blank.
   */
  const normalizedQuestion = normalizeQuestionArgument(rawQuestion,);

  return {
    ...normalizedModel,
    ...normalizedQuestion,
  };
}

//endregion Argument normalization
