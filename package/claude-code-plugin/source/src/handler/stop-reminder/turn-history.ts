/**
 The assistant's earlier stops in the current turn, read from the transcript.

 On a repeated stop (a stop after the hook already blocked this turn) Luna is given
 what the agent said at the earlier stops, so it can tell a wait the agent already
 explained from a new one. Only text-only assistant records count as stops; a record
 that issued a tool call was the agent working, not stopping.

 @module
 */

import type { TranscriptRecord, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  isHumanTurnRecord,
  parseRecord,
  UNPARSABLE,
} from './continuation-depth.ts';

/**
 Characters from the end of the final message compared with the newest stop to recognise it.

 The end is compared instead of the whole text because the message handed to the hook and the transcript text can
 differ at the edges; an end of this length is specific to one message.
 */
const FINAL_MATCH_CHARS = 200;

/**
 Prose of an assistant record that stopped without a tool call, or the empty string.

 @param record - parsed transcript record

 @returns the record's text blocks joined by newlines, empty for any other record

 @example
 ```ts
 stopProse({ type: 'assistant', message: { content: [{ type: 'text', text: 'Done.' }] } }); // 'Done.'
 ```
 */
function stopProse(record: TranscriptRecord,): string {
  /**
   Record content, an array only for assistant turns.
   */
  const content = record
    .message
    ?.content;
  if (record.type !== 'assistant')
    return '';
  if (record.isSidechain === true)
    return '';
  if ((content === undefined) || ((typeof content) === 'string'))
    return '';
  if (content.some(function usesTool(block,) {
    return block.type === 'tool_use';
  },)) {
    return '';
  }
  return content
    .map(function textOf(block,) {
      return (block.type === 'text') ? (block.text ?? '') : '';
    },)
    .filter(Boolean,)
    .join('\n',)
    .trim();
}

/**
 Collects the assistant text of the earlier stops in the current turn.

 Scans newest first and stops at the first genuine human turn, so only this turn's
 stops count. Records are parsed rather than matched as substrings, for the reason
 given in `continuation-depth.ts`. The final message is usually the newest record
 already, and is dropped so the history never repeats the message being judged.

 @param transcriptLines - transcript JSONL lines, oldest first

 @param finalMessage - the final assistant message being judged

 @returns earlier stop texts, oldest first; empty when the turn has none or the transcript is unreadable

 @example
 ```ts
 earlierStopTexts({ transcriptLines: [], finalMessage: 'Done.' }); // []
 ```
 */
function earlierStopTexts(
  {
    transcriptLines,
    finalMessage,
  }: {
    readonly transcriptLines: readonly string[];
    readonly finalMessage: string;
  },
): readonly string[] {
  /**
   Stop texts met so far, newest first.
   */
  const newestFirst: string[] = [];

  for (let index = transcriptLines.length - 1; index >= 0; index--) {
    /**
     Raw transcript line under inspection.
     */
    const line = transcriptLines[index] ?? '';

    // Only an assistant record or a human turn can change the answer, so other lines are never parsed.
    if ((line === '')
      || ((!line.includes('"type":"assistant"',))
        && (!line.includes('"kind":"human"',)))) {
      continue;
    }
    /**
     Parsed record for a line that might be a stop or a human turn.
     */
    const record = parseRecord(line,);
    if (record === UNPARSABLE)
      continue;
    // A human turn closes the window; anything before it belongs to an earlier turn.
    if (isHumanTurnRecord(record,))
      break;
    /**
     Prose of this record when it was a stop.
     */
    const prose = stopProse(record,);
    if (prose !== '')
      newestFirst.push(prose,);
  }
  /**
   End of the message being judged, enough to recognise it among the stops without needing identical whitespace.
   */
  const finalEnd = finalMessage
    .trim()
    .slice(-FINAL_MATCH_CHARS,);
  /**
   Newest stop met, or the empty string when the turn has none.
   */
  const newest = newestFirst[0] ?? '';
  /**
   Whether the newest stop is the message being judged, which the history must not repeat.
   */
  const startsWithFinal = (finalEnd !== '') && newest.endsWith(finalEnd,);
  return (startsWithFinal ? newestFirst.slice(1,) : newestFirst).toReversed();
}

export { earlierStopTexts, };
