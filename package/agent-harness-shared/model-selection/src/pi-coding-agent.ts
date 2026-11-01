/**
 Optional wrappers that import pi-coding-agent directly.
 
 @module
 */

import { estimateTokens, } from '@earendil-works/pi-coding-agent';

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
 Estimate Advisor-style request input tokens using pi's message token estimator.
 
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
  /**
   Synthetic user message matching secondary Advisor request shape.
   */
  const message: Parameters<typeof estimateTokens>[0] = {
    role: 'user',
    content: `${options.systemPrompt}\n\n${options.contextText}`,
    timestamp: 0,
  };
  return estimateTokens(message,);
}

//endregion Public API
