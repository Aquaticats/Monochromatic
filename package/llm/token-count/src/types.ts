/**
 Configuration for token counting requests.
 
 @example
 ```ts
 const config: CountTokensConfig = {
   model: 'claude-sonnet-4-6',
   apiKey: 'sk-ant-...',
 };
 ```
 */
export type CountTokensConfig = {
  /**
   Claude model for tokenization; defaults to `claude-sonnet-4-6`. Only selects the tokenizer; no inference is performed.
   */
  readonly model?: string;
  /**
   Anthropic API key; falls back to `TOKEN_COUNT_CLAUDE_API_KEY`, `CLAUDE_API_KEY`, then `ANTHROPIC_API_KEY` env vars
   */
  readonly apiKey?: string;
};

/**
 Result of a token counting operation.
 
 @example
 ```ts
 const result: TokenCountResult = { inputTokens: 4700, model: 'claude-sonnet-4-6' };
 ```
 */
export type TokenCountResult = {
  /**
   Number of input tokens in the content
   */
  readonly inputTokens: number;
  /**
   Model used for tokenization
   */
  readonly model: string;
};

/**
 Result of counting tokens in a file, extending {@link TokenCountResult}
 with the source file path.
 
 @example
 ```ts
 const result: FileTokenCountResult = {
   inputTokens: 4700,
   model: 'claude-sonnet-4-6',
   filePath: './CLAUDE.md',
 };
 ```
 */
export type FileTokenCountResult = TokenCountResult & {
  /**
   Path of the counted file, as provided to the function
   */
  readonly filePath: string;
};
