/**
 DSH stream chunk to pi-ai event mapping for the DSH Advisor stream
 consumption loop. @module
 */

import type {
  AssistantMessage,
  AssistantMessageEvent,
  TextContent,
  ThinkingContent,
  ToolCall,
} from '@earendil-works/pi-ai';
import type {
  StreamChunk,
} from '@deepseek-ai/dsh-llm';
import {
  mapDshAssistantContent,
} from './message-map.ts';

//region Chunk mapping

/**
 Publish the pi-ai event for one non-finish, non-usage DSH stream chunk.

 Assembled content grows at every block end through the shared mutable array.

 @param chunk - mapped non-terminal stream chunk

 @param content - assembled pi-ai content blocks mutated in completion order

 @param publish - event publisher driving the pi-ai stream surface

 @param partial - latest partial assistant message snapshot

 @example
 ```ts
 publishAdvisorChunkEvent({ chunk, content, publish, partial });
 ```
 */
export function publishAdvisorChunkEvent(
  {
    chunk,
    content,
    publish,
    partial,
  }: {
    readonly chunk: Exclude<StreamChunk, {
      type: 'finish';
    } | {
      type: 'usage';
    }>;
    readonly content: (TextContent | ThinkingContent | ToolCall)[];
    readonly publish: (event: AssistantMessageEvent) => void;
    readonly partial: AssistantMessage;
  },
): void {
  if (chunk.type
    === 'block-start') {
    if (chunk.blockType
      === 'text')
      publish({
        type: 'text_start',
        contentIndex: chunk.index,
        partial,
      },);
    else if (chunk.blockType
      === 'reasoning')
      publish({
        type: 'thinking_start',
        contentIndex: chunk.index,
        partial,
      },);
    else if (chunk.blockType
      === 'tool-call')
      publish({
        type: 'toolcall_start',
        contentIndex: chunk.index,
        partial,
      },);
    return;
  }
  if (chunk.type
    === 'text-delta') {
    publish({
      type: 'text_delta',
      contentIndex: chunk.index,
      delta: chunk.text,
      partial,
    },);
    return;
  }
  if (chunk.type
    === 'reasoning-delta') {
    publish({
      type: 'thinking_delta',
      contentIndex: chunk.index,
      delta: chunk.text,
      partial,
    },);
    return;
  }
  if (chunk.type
    === 'tool-call-delta') {
    publish({
      type: 'toolcall_delta',
      contentIndex: chunk.index,
      delta: chunk.argumentsDelta,
      partial,
    },);
    return;
  }
  /**
   pi-ai content mapped from the assembled DSH block, when it has a
   counterpart.
   */
  const mapped = mapDshAssistantContent([chunk.block,],);
  for (const block of mapped) {
    content.push(block,);
    if (block.type
      === 'text')
      publish({
        type: 'text_end',
        contentIndex: chunk.index,
        content: block.text,
        partial,
      },);
    else if (block.type
      === 'thinking')
      publish({
        type: 'thinking_end',
        contentIndex: chunk.index,
        content: block.thinking,
        partial,
      },);
    else
      publish({
        type: 'toolcall_end',
        contentIndex: chunk.index,
        toolCall: block,
        partial,
      },);
  }
}

//endregion Chunk mapping
