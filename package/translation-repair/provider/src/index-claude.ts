//region Claude public index
// The names the owner's Claude route gives its siblings and its own tests, each from the module that declares it;
// split out of `index.ts` for the reason `index-neuralwatt.ts` gives.

export {
  CLAUDE_API_VERSION,
  CLAUDE_AUTH_HEADER,
  CLAUDE_BASE_URL,
  CLAUDE_MODELS,
  CLAUDE_VERSION_HEADER,
  claudeMessagesUrlFor,
  type ClaudeModelInfo,
} from './claude-catalog.ts';
export {
  buildClaudeBody,
  ClaudePrefillError,
} from './claude-body.ts';
export {
  ClaudeSchemaKeywordError,
  claudeSchemaOf,
} from './claude-schema.ts';
export {
  type SchemaRefusal,
  structuralRefusalOf,
} from './claude-schema-refusals.ts';
export {
  CLAUDE_PACE_WINDOW_MS,
  CLAUDE_REQUESTS_PER_MINUTE,
  ClaudeModelNotServedError,
  createClaudeClient,
  type ClaudeClient,
} from './claude-client.ts';
export { claudeCostOf, } from './claude-cost.ts';
export {
  claudeSpendNow,
  type ClaudeSpend,
} from './claude-spend.ts';
export {
  claudeStreamFactsOf,
  type ClaudeStreamFacts,
  type ClaudeUsageSplit,
} from './claude-stream-facts.ts';
export {
  claudeIdFor,
  claudeShowsPictures,
  type ClaudeSpelling,
} from './claude-reach.ts';

//endregion Claude public index
