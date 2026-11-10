/**
 Provider-specific forced structured-tool selection.
 
 @module
 */

//region Types -- provider tool-choice request

/**
 Options for provider tool-choice projection.
 
 @example
 ```ts
 toolChoiceForApi({ api: 'anthropic-messages', toolName: 'submit_review' });
 ```
 */
type ToolChoiceForApiOptions = {
  /**
   Pi AI API identifier.
   */
  readonly api: string;
  /**
   Exact forced tool name.
   */
  readonly toolName: string;
};

//endregion Types -- provider tool-choice request

//region Public API -- forced tool choice per Pi AI API

/**
 Return provider-specific forced tool choice.
 
 @param api - Pi AI API identifier
 
 @param toolName - exact forced tool name
 
 @returns provider tool-choice value
 
 @example
 ```ts
 toolChoiceForApi({ api: 'openai-responses', toolName: 'submit_review' });
 ```
 */
function toolChoiceForApi(
  {
    api,
    toolName,
  }: ToolChoiceForApiOptions,
): Readonly<Record<string, string>> | string {
  if (api === 'anthropic-messages') {
    return {
      type: 'tool',
      name: toolName,
    };
  }
  if ((api === 'openai-completions')
    || (api === 'openai-responses')
    || (api === 'azure-openai-responses')
    || (api === 'openai-codex-responses')) {
    return 'required';
  }
  return 'any';
}

//endregion Public API -- forced tool choice per Pi AI API

export { toolChoiceForApi, };
export type { ToolChoiceForApiOptions, };
