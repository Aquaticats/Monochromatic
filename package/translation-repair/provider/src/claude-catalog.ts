import {
  type AnthropicCard,
  type AnthropicServedId,
  type RosterModelId,
  servedRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Claude catalog
// WHAT THE OWNER'S CLAUDE KEY SERVES FOR THIS PIPELINE, how it is spelled and where it is called: Sonnet 5.5 and
// Haiku 5.5, each with no effort and at three efforts, eight rows read off the cards (the grant is quoted in
// `claude-card-sides.ts`). The route speaks the Messages protocol of `https://api.anthropic.com/v1/messages` with an
// `x-api-key` header and the `anthropic-version` header, facts read from the API reference on 2026-10-22.
//
// THE KEY IS NOT MODEL-SCOPED, which is why the lock is in the types and the client and not in the account:
// `claude-client.ts` throws for any roster model that is not one of these rows before a request leaves, and the
// wire model of every row is one of two literals (`roster-id.ts`).
//
// THE FILES OF THIS ROUTE ARE NAMED `claude-`, NOT `anthropic-`: the `anthropic-` files of this package are the
// Messages protocol Hyper speaks too, which this route reuses for its message and content conversion.

/**
 Base address every Claude route hangs off.
 */
export const CLAUDE_BASE_URL = 'https://api.anthropic.com/v1';

/**
 Header carrying the key.
 */
export const CLAUDE_AUTH_HEADER = 'x-api-key';

/**
 Header naming the API version, and its value (the API reference's current one, read 2026-10-22).
 */
export const CLAUDE_VERSION_HEADER = 'anthropic-version';

/**
 Value of the version header.
 */
export const CLAUDE_API_VERSION = '2023-06-01';

/**
 One Claude card's verified facts the router and the request builder read.

 @example
 ```ts
 const info: ClaudeModelInfo = CLAUDE_MODELS['claude-haiku-5-5'];
 ```
 */
export type ClaudeModelInfo = AnthropicCard & {
  /**
   The roster seat this spelling reaches.
   */
  readonly sharedWith: RosterModelId;
};

/**
 Every Claude card, read off the cards.

 @example
 ```ts
 const info = CLAUDE_MODELS['claude-sonnet-5-5-effort-low'];
 ```
 */
export const CLAUDE_MODELS: Readonly<Record<AnthropicServedId, ClaudeModelInfo>> = servedRecord({
  provider: 'anthropic',
  toRow: function claudeRow(card,): ClaudeModelInfo {
    return {
      ...card.anthropic,
      sharedWith: card.id,
    };
  },
},);

/**
 Messages URL.

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to POST the body to

 @example
 ```ts
 claudeMessagesUrlFor({ baseUrl: CLAUDE_BASE_URL, },);
 // => 'https://api.anthropic.com/v1/messages'
 ```
 */
export function claudeMessagesUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/messages`;
}

//endregion Claude catalog
