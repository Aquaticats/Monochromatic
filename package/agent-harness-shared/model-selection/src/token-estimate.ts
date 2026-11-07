/**
 Text-only token estimates shared by Advisor host adapters.

 @module
 */

//region Estimate policy

/**
 UTF-16 code units per estimated token, preserving the text-only estimate
 measured against Pi 1.5.0. This heuristic is not a model tokenizer.

 @example
 ```ts
 const contextChars = availableTokens * TOKEN_ESTIMATE_CHARS_PER_TOKEN;
 ```
 */
export const TOKEN_ESTIMATE_CHARS_PER_TOKEN = 4;

/**
 Text separator included between the system prompt and serialized context.
 */
const ADVISOR_CONTEXT_SEPARATOR = '\n\n';

//endregion Estimate policy

//region Types

/**
 Options for estimating Advisor-style request input tokens.
 */
export type EstimateAdvisorTokensOptions = {
  /**
   Advisor model system prompt.
   */
  readonly systemPrompt: string;
  /**
   Serialized conversation context.
   */
  readonly contextText: string;
};

//endregion Types

//region Public API

/**
 Estimate Advisor text without importing a host SDK or allocating a combined message.

 Counts JavaScript string code units, including the prompt/context separator,
 and rounds up once for the entire request. Empty input still includes the
 separator. The estimate deliberately retains the previous text-only policy.

 @param options - system prompt and serialized conversation

 @returns estimated token count

 @example
 ```typescript
 estimateAdvisorInputTokens({ systemPrompt, contextText });
 ```
 */
export function estimateAdvisorInputTokens(
  options: EstimateAdvisorTokensOptions,
): number {
  return Math.ceil(
    (options.systemPrompt
      .length
      + ADVISOR_CONTEXT_SEPARATOR.length
      + options.contextText
        .length)
      / TOKEN_ESTIMATE_CHARS_PER_TOKEN,
  );
}

//endregion Public API
