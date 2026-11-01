/**
 Advisor request message formatting helpers.
 
 @module
 */

//region Types

/**
 Options for building Advisor user-message text.
 */
export type BuildAdvisorUserMessageTextOptions = {
  /**
   Focused question supplied by the primary agent.
   */
  readonly question?: string;
  /**
   Serialized conversation context.
   */
  readonly contextText: string;
};

//endregion Types

//region Public API

/**
 Build user-message text sent to the Advisor model.
 
 @param options - focused question and serialized context
 
 @returns formatted Advisor user-message text
 
 @example
 ```typescript
 buildAdvisorUserMessageText({ question: 'What did I miss?', contextText: '...' });
 ```
 */
export function buildAdvisorUserMessageText(
  options: BuildAdvisorUserMessageTextOptions,
): string {
  /**
   Trimmed question text, if supplied.
   */
  const normalizedQuestion = options.question
    ?.trim();
  /**
   Serialized conversation block common to every Advisor request.
   */
  const serializedConversationBlock = `## Serialized conversation\n\n${options.contextText}`;

  if ((normalizedQuestion === undefined) || (normalizedQuestion === ''))
    return serializedConversationBlock;

  return `## Focus question\n\n${normalizedQuestion}\n\n${serializedConversationBlock}`;
}

//endregion Public API
