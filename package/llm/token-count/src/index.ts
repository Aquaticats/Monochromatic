/**
 Token counting for Claude models using the Anthropic API.
 
 Wraps the Anthropic `messages.countTokens` endpoint into a minimal library
 with file-reading conveniences and an optique-based CLI.
 
 @example
 ```ts
 import { countTokens } from '@monochromatic-dev/llm-token-count';
 
 const result = await countTokens({ content: 'Hello, world!' });
 console.log(result.inputTokens);
 ```
 
 @example
 ```ts
 import { countFileTokens } from '@monochromatic-dev/llm-token-count';
 
 const result = await countFileTokens({ filePath: './CLAUDE.md' });
 console.log(`${result.filePath}: ${result.inputTokens} tokens`);
 ```
 
 @packageDocumentation
 */

export {
  countFileTokens,
  countTokens,
  DEFAULT_MODEL,
} from './client.ts';
export type {
  CountTokensConfig,
  FileTokenCountResult,
  TokenCountResult,
} from './types.ts';
