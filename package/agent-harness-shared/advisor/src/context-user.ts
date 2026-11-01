/**
 User-message helpers for Advisor context metadata.
 
 @module
 */

import type { UserMessage, } from '@earendil-works/pi-ai';
import type { ReadonlyDeep, } from 'type-fest';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { LATEST_USER_EXCERPT_CHARS, } from './constants.ts';
import type { AdvisorSessionEntry, } from './ports.ts';

//region Public API

/**
 Sentinel returned by {@link latestUserPromptExcerpt} when a branch carries no
 user message. A `unique symbol`; callers narrow with `=== NO_USER_PROMPT`.
 */
export const NO_USER_PROMPT: unique symbol = Symbol('advisor/no-user-prompt',);

/**
 Extract latest user prompt excerpt from a branch.
 
 @param branch - session branch entries
 
 @returns latest user prompt excerpt, or {@link NO_USER_PROMPT} when none
 
 @example
 ```typescript
 latestUserPromptExcerpt(branch);
 ```
 */
export function latestUserPromptExcerpt(
  branch: readonly ForeignBorrowed<AdvisorSessionEntry>[],
): string | typeof NO_USER_PROMPT {
  for (let index = branch.length - 1; index >= 0; index -= 1) {
    /**
     Branch entry at current reverse cursor.
     */
    const entry = branch[index];
    if ((entry === undefined) || (!isUserMessageEntry(entry,)))
      continue;

    /**
     Plain text extracted from latest user message content.
     */
    const text = userMessageText(entry.message
      .content,);
    return text.length
      <= LATEST_USER_EXCERPT_CHARS
      ? text
      : `${
        text.slice(
          0,
          LATEST_USER_EXCERPT_CHARS,
        )
      }…`;
  }
  return NO_USER_PROMPT;
}

//endregion Public API

//region Internal helpers

/**
 Convert user message content into plain text.
 
 @param content - user message content
 
 @returns plain text from text blocks
 */
function userMessageText(
  content: ReadonlyDeep<UserMessage['content']>,
): string {
  if ((typeof content) === 'string')
    return content;
  /**
   Text block values collected in source order.
   */
  const textParts: string[] = [];
  for (const block of content) {
    if (block.type
      === 'text')
      textParts.push(block.text,);
  }
  return textParts.join('\n',);
}

/**
 Detect user message session entries.
 
 @param entry - session entry to inspect
 
 @returns whether entry contains a user message
 */
function isUserMessageEntry(
  entry: ForeignBorrowed<AdvisorSessionEntry>,
): entry is ForeignBorrowed<AdvisorSessionEntry & {
  readonly type: 'message';
  readonly message: UserMessage;
}> {
  return (entry.type
    === 'message') && (entry.message
      .role
      === 'user');
}

//endregion Internal helpers
