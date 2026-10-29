import type { CompletionUsage, } from '@monochromatic-dev/llm-llm-type/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Decision contract
// Request and reply shapes of a typed-decision exchange: OpenRouter's
// `/api/alpha/decisions` (its OpenAPI document, schemas `DecisionsRequest`
// and `DecisionsResponse`, read 2026-09-18), which fronts TypeSafe's System
// One models. A decision model takes a state and named questions and answers
// each with a typed value and probabilities, never with prose, so nothing
// here shares a shape with the chat contract: a decision seat cannot take a
// chat completion and a chat seat cannot take a question.

/**
 One question asking for one option out of a named set.

 @example
 ```ts
 const question: DecisionChoiceQuestion = { type: 'choice', instructions: 'Which cat wrote this?', criteria: { mittens: 'Mittens', tabby: 'Tabby', }, };
 ```
 */
export type DecisionChoiceQuestion = {
  /**
   Discriminator the endpoint expects.
   */
  readonly type: 'choice';

  /**
   What to decide, in prose.
   */
  readonly instructions: string;

  /**
   Options by key, each described.
   */
  readonly criteria: Readonly<Record<string, string>>;
};

/**
 One yes-or-no question, answered as the probability of yes.

 @example
 ```ts
 const question: DecisionNoulQuestion = { type: 'noul', instructions: 'Is the cat asleep?', };
 ```
 */
export type DecisionNoulQuestion = {
  /**
   Discriminator the endpoint expects.
   */
  readonly type: 'noul';

  /**
   What to decide, in prose.
   */
  readonly instructions: string;

  /**
   What each answer means, when the caller names it.
   */
  readonly criteria?: {
    readonly true: string;
    readonly false: string;
  };
};

/**
 One rating against ordered levels, answered as a score over them.

 @example
 ```ts
 const question: DecisionScoreQuestion = { type: 'score', instructions: 'How sleepy is the cat?', criteria: ['awake', 'drowsy', 'asleep',], };
 ```
 */
export type DecisionScoreQuestion = {
  /**
   Discriminator the endpoint expects.
   */
  readonly type: 'score';

  /**
   What to rate, in prose.
   */
  readonly instructions: string;

  /**
   Levels from lowest to highest.
   */
  readonly criteria: readonly string[];
};

/**
 Any question a decision model answers.
 */
export type DecisionQuestion =
  | DecisionChoiceQuestion
    | DecisionNoulQuestion
    | DecisionScoreQuestion;

/**
 What the questions are asked about: a plain string, or a JSON object or
 array of related context.
 */
export type DecisionState =
  | string
    | Readonly<Record<string, unknown>>
    | readonly unknown[];

/**
 Answer to a choice question.

 @example
 ```ts
 const answer: DecisionChoiceAnswer = { type: 'choice', choice: 'mittens', probabilities: { mittens: 0.9, tabby: 0.1, }, confidence: 0.8, };
 ```
 */
export type DecisionChoiceAnswer = {
  /**
   Discriminator as the endpoint sends it.
   */
  readonly type: 'choice';

  /**
   Key of the chosen option.
   */
  readonly choice: string;

  /**
   Probability per option, when reported.
   */
  readonly probabilities?: Readonly<Record<string, number>>;

  /**
   How sure the model is of the distribution, when reported.
   */
  readonly confidence?: number;
};

/**
 Answer to a yes-or-no question.

 @example
 ```ts
 const answer: DecisionNoulAnswer = { type: 'noul', noul: 0.95, };
 ```
 */
export type DecisionNoulAnswer = {
  /**
   Discriminator as the endpoint sends it.
   */
  readonly type: 'noul';

  /**
   Probability that the answer is yes.
   */
  readonly noul: number;
};

/**
 Answer to a score question.

 @example
 ```ts
 const answer: DecisionScoreAnswer = { type: 'score', score: 1.4, };
 ```
 */
export type DecisionScoreAnswer = {
  /**
   Discriminator as the endpoint sends it.
   */
  readonly type: 'score';

  /**
   Expected level, fractional between levels.
   */
  readonly score: number;

  /**
   Probability per level index, when reported.
   */
  readonly probabilities?: Readonly<Record<string, number>>;

  /**
   How sure the model is of the distribution, when reported.
   */
  readonly confidence?: number;

  /**
   Level names by index, when reported.
   */
  readonly legend?: Readonly<Record<string, string>>;
};

/**
 Any answer a decision model gives.
 */
export type DecisionAnswer =
  | DecisionChoiceAnswer
    | DecisionNoulAnswer
    | DecisionScoreAnswer;

/**
 One typed-decision exchange request.

 @example
 ```ts
 const request: DecisionRequest = { modelId: 'typesafe/jev-1.13', state: 'The cat is asleep.', questions: { asleep: { type: 'noul', instructions: 'Is the cat asleep?', }, }, signal: AbortSignal.timeout(30_000,), };
 ```
 */
export type DecisionRequest = {
  /**
   Decision seat receiving the exchange.
   */
  readonly modelId: RosterModelId;

  /**
   What the questions are about.
   */
  readonly state: DecisionState;

  /**
   Questions by name, answered together against the state.
   */
  readonly questions: Readonly<Record<string, DecisionQuestion>>;

  /**
   Abort signal honored for the whole exchange.
   */
  readonly signal: AbortSignal;

  /**
   Deadline bounding the exchange, when the caller sets one.
   */
  readonly exchangeTimeoutMs?: number;
};

/**
 What one typed-decision exchange answered.

 @example
 ```ts
 const reply: DecisionReply = { answers: { asleep: { type: 'noul', noul: 0.95, }, }, model: 'typesafe/jev-1.13', };
 ```
 */
export type DecisionReply = {
  /**
   Answers by question name.
   */
  readonly answers: Readonly<Record<string, DecisionAnswer>>;

  /**
   Model that answered, as the endpoint names it.
   */
  readonly model: string;

  /**
   Token usage when the endpoint reported it, output tokens included though
   the model writes no prose.
   */
  readonly usage?: CompletionUsage;

  /**
   USD the endpoint reported for this call, when it said.
   */
  readonly costUsd?: number;
};

/**
 What a client of the decisions endpoint can do.

 @example
 ```ts
 const reply = await decider.decide(request,);
 ```
 */
export type Decider = {
  /**
   Typed-decision exchange.
   */
  readonly decide: (request: ForeignBorrowed<DecisionRequest>,) => Promise<DecisionReply>;
};

/**
 Raised when a decisions reply body is not the shape the endpoint documents.

 @example
 ```ts
 throw new DecisionReplyShapeError({ detail: 'answers missing', },);
 ```
 */
export class DecisionReplyShapeError extends Error {
  /**
   Declares this message safe to forward: it names which part of the documented shape was missing and quotes nothing.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal naming the missing part.

   @param detail - which documented field was absent or mistyped

   @param cause - what the parse threw, carried for the chain and kept out of
   the message, which names nothing the reply held; absent for a refusal
   nothing threw before

   @example
   ```ts
   new DecisionReplyShapeError({ detail: 'answers missing', },);
   ```
   */
  public constructor(
    {
      detail,
      cause,
    }: {
      readonly detail: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      `decisions reply is not the documented shape: ${detail}`,
      // Conditional spread keeps cause absent when none was supplied, rather
      // than an own cause that is undefined.
      ...(cause === undefined
        ? []
        : [{ cause, },]),
    );
    this.name = 'DecisionReplyShapeError';
  }
}

/**
 Guards one answer as the endpoint documents it.

 @param value - candidate from unvalidated JSON

 @returns Whether value is a choice, noul or score answer

 @example
 ```ts
 isDecisionAnswer({ type: 'noul', noul: 0.5, },);
 ```
 */
export function isDecisionAnswer(value: unknown,): value is DecisionAnswer {
  if (!isJsonRecord(value,))
    return false;
  if (value.type === 'choice')
    return (typeof value.choice) === 'string';
  if (value.type === 'noul')
    return (typeof value.noul) === 'number';
  if (value.type === 'score')
    return (typeof value.score) === 'number';
  return false;
}

/**
 Reads a reply body into the typed reply.

 @param bodyText - response body as the transport handed it back

 @returns Answers, model and usage as the endpoint reported them

 @throws {@link DecisionReplyShapeError} when the body is not JSON or lacks
 the documented fields

 @example
 ```ts
 const reply = readDecisionReplyBody({ bodyText, },);
 ```
 */
export function readDecisionReplyBody({ bodyText, }: { readonly bodyText: string; },): DecisionReply {
  /**
   Body as JSON, or the parse failure.
   */
  const parsed: unknown = (function parse(): unknown {
    try {
      return JSON.parse(bodyText,);
    }
    catch (error) {
      throw new DecisionReplyShapeError({
        detail: 'body is not JSON',
        cause: error,
      },);
    }
  })();
  if (!isJsonRecord(parsed,))
    throw new DecisionReplyShapeError({ detail: 'body is not an object', },);

  /**
   Answers block as sent.
   */
  const { answers, } = parsed;
  if (!isJsonRecord(answers,))
    throw new DecisionReplyShapeError({ detail: (answers === undefined) ? 'answers missing' : 'answers is not an object', },);

  /**
   Each answer checked against the documented shapes.
   */
  const read: Readonly<Record<string, DecisionAnswer>> = Object.fromEntries(Object.entries(answers,)
    .map(function checked([
      name,
      answer,
    ],): readonly [
      string,
      DecisionAnswer,
    ] {
      if (!isDecisionAnswer(answer,))
        throw new DecisionReplyShapeError({ detail: `answer ${name} is not a choice, noul or score`, },);
      return [
        name,
        answer,
      ];
    },),);

  /**
   Model that answered.
   */
  const { model, } = parsed;
  if ((typeof model) !== 'string')
    throw new DecisionReplyShapeError({ detail: (model === undefined) ? 'model missing' : 'model is not a string', },);

  /**
   Usage block, absent when the endpoint sent none.
   */
  const { usage, } = parsed;
  if (!isJsonRecord(usage,))
    return {
      answers: read,
      model,
    };

  /**
   Token counts and cost as sent, under this package's names.
   */
  const {
    input_tokens: inputTokens,
    output_tokens: outputTokens,
    cost,
  } = usage;
  return {
    answers: read,
    model,
    ...(((typeof inputTokens) === 'number') && ((typeof outputTokens) === 'number')
      ? {
        usage: {
          prompt_tokens: inputTokens,
          completion_tokens: outputTokens,
          total_tokens: inputTokens + outputTokens,
        },
      }
      : {}),
    ...(((typeof cost) === 'number') ? { costUsd: cost, } : {}),
  };
}

//endregion Decision contract
