/**
 Test fixtures for shared model-selection unit tests.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ReadonlyModel, } from '../dist/final/node/index.mjs';

/**
 Cheap input token price.
 */
export const CHEAP_INPUT = 1;

/**
 Cheap output token price.
 */
export const CHEAP_OUTPUT = 2;

/**
 Mid input token price.
 */
export const MID_INPUT = 3;

/**
 Mid output token price.
 */
export const MID_OUTPUT = 4;

/**
 Expensive input token price.
 */
export const EXPENSIVE_INPUT = 5;

/**
 Expensive output token price.
 */
export const EXPENSIVE_OUTPUT = 8;

/**
 Fixture context budget.
 */
const CONTEXT_WINDOW = 128_000;

/**
 Fixture maximum output tokens.
 */
const MAX_TOKENS = 4_096;

/**
 Options for building fixture models.
 */
export type FixtureModelOptions = {
  /**
   Provider slug.
   */
  readonly provider: string;
  /**
   Model id.
   */
  readonly id: string;
  /**
   Display name.
   */
  readonly name?: string;
  /**
   Input-token price.
   */
  readonly inputCost?: number;
  /**
   Output-token price.
   */
  readonly outputCost?: number;
};

/**
 Build a complete readonly model fixture.
 
 @param provider - provider slug
 
 @param id - model id
 
 @param name - optional display name
 
 @param inputCost - optional input token price
 
 @param outputCost - optional output token price
 
 @returns readonly model fixture
 
 @example
 ```typescript
 fixtureModel({ provider: 'openai', id: 'gpt-5.5' });
 ```
 */
export function fixtureModel(
  {
    provider,
    id,
    name,
    inputCost = CHEAP_INPUT,
    outputCost = CHEAP_OUTPUT,
  }: FixtureModelOptions,
): ReadonlyModel {
  return {
    id,
    name: name ?? id,
    api: 'faux',
    provider,
    baseUrl: 'https://example.invalid',
    reasoning: false,
    input: ['text',],
    cost: {
      input: inputCost,
      output: outputCost,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: CONTEXT_WINDOW,
    maxTokens: MAX_TOKENS,
  };
}

/**
 Return provider/model slug for a fixture model.
 
 @param model - model fixture
 
 @returns provider/model slug
 
 @example
 ```typescript
 fixtureSlug(model);
 ```
 */
export function fixtureSlug(
  model: Pick<ReadonlyModel, 'provider' | 'id'>,
): string {
  return `${model.provider}/${model.id}`;
}

/**
 Capture a synchronous error from an action.
 
 @param action - action expected to throw
 
 @returns caught error value
 
 @mutates action - invokes supplied test callback and its captured state
 
 @example
 ```typescript
 const error = captureError(function fail() { throw new Error('x'); });
 ```
 */
export function captureError(
  action: ForeignBorrowed<() => unknown>,
): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected action to throw',);
}

/**
 Capture an async error from an action.
 
 @param action - async action expected to throw
 
 @returns caught error value
 
 @mutates action - invokes supplied async test callback and its captured state
 
 @example
 ```typescript
 const error = await captureAsyncError(async function fail() { throw new Error('x'); });
 ```
 */
export async function captureAsyncError(
  action: ForeignBorrowed<() => Promise<unknown>>,
): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected action to throw',);
}
