/**
 Whether this hook already raised a given reminder in the current turn.

 Inside a blocked chain a wording check runs once per trigger, so the agent can answer a block and
 finish: the second stop that repeats a trigger it was already told about goes through. The answer
 comes from the transcript, where Claude Code records each block as a user record opening with the
 hook's feedback prefix.

 @module
 */

import type { TranscriptRecord, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  FEEDBACK_PREFIX,
  isHumanTurnRecord,
  parseRecord,
  UNPARSABLE,
} from './continuation-depth.ts';

/**
 Reports whether a record is a feedback record carrying a reminder's opening sentence.

 A tool result or an assistant message that merely quotes the sentence is not a block the agent received,
 so only a user record with string content opening with the feedback prefix counts.

 @param record - parsed transcript record

 @param lead - opening sentence of the reminder

 @returns whether the record is this hook's reminder with that opening sentence

 @example
 ```ts
 carriesLead({ record, lead: 'Your response contains uncertain language ("probably").' });
 ```
 */
function carriesLead(
  {
    record,
    lead,
  }: {
    readonly record: TranscriptRecord;
    readonly lead: string;
  },
): boolean {
  /**
   Record content, only a string for the feedback records this looks for.
   */
  const content = record
    .message
    ?.content;

  return (record.type === 'user')
    && (record.toolUseResult === undefined)
    && ((typeof content) === 'string')
    && content.startsWith(FEEDBACK_PREFIX,)
    && content.includes(lead,);
}

/**
 Reports whether a reminder with this opening sentence was already issued since the last human turn.

 Scans newest first and stops at the first genuine human turn. Records are parsed rather than matched
 as substrings, for the reason given in `continuation-depth.ts`.

 @param lead - opening sentence of the reminder about to be raised

 @param transcriptLines - transcript JSONL lines, oldest first

 @returns whether the agent was already told about this trigger in the current turn

 @example
 ```ts
 alreadyRaised({ lead: 'Your response contains uncertain language ("probably").', transcriptLines: [] }); // false
 ```
 */
function alreadyRaised(
  {
    lead,
    transcriptLines,
  }: {
    readonly lead: string;
    readonly transcriptLines: readonly string[];
  },
): boolean {
  for (let index = transcriptLines.length - 1; index >= 0; index--) {
    /**
     Raw transcript line under inspection.
     */
    const line = transcriptLines[index] ?? '';

    // Only a feedback record or a human turn can change the answer, so other lines are never parsed.
    if ((line === '')
      || ((!line.includes(FEEDBACK_PREFIX,))
        && (!line.includes('"kind":"human"',)))) {
      continue;
    }
    /**
     Parsed record for a line that might be a reminder or a human turn.
     */
    const record = parseRecord(line,);
    if (record === UNPARSABLE)
      continue;
    if (carriesLead({
      record,
      lead,
    },)) {
      return true;
    }
    // A human turn closes the window; anything before it belongs to an earlier turn.
    if (isHumanTurnRecord(record,))
      return false;
  }
  return false;
}

export { alreadyRaised, };
