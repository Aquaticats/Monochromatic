/**
 Detection of a choice left with the user in prose.

 An agent that writes "Say so if you want it" or "Tell me if you'd rather" has a
 decision for the user but routes it through text the user may not read. The
 question tool is the channel that reaches the user, so this detector blocks the
 stop and says to use it. The trailing-question detector cannot cover this wording
 because it needs a literal question mark.

 Unlike the wording detectors that run only on the first stop of a blocked chain,
 this one also runs inside a chain, once per phrase: an offer the agent already
 heard about is not raised again, so the agent can answer a block and finish.

 @module
 */

import {
  containsAnyOfWordBounded,
  PHRASE_NOT_FOUND,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import type { TranscriptRecord, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  FEEDBACK_PREFIX,
  isHumanTurnRecord,
  parseRecord,
  UNPARSABLE,
} from './continuation-depth.ts';
import { normaliseApostrophes, } from './uncertainty-phrases.ts';
import { NO_MATCH, } from './uncertainty.ts';

//region Phrases

/**
 Lowercase phrases of an offer that waits for the user to answer in prose.

 Measured on past stops, these read as offers almost every time ("Say so if you want it
 posted", "Tell me if you want me to unmount it"). Broader wording such as "your call"
 and a bare "if you want it" often explains a boundary instead, so it is left out.
 */
const PARKED_DECISION_PHRASES: readonly string[] = [
  'say so if',
  'say if you',
  'tell me if you',
  "if you'd rather",
  'if you would rather',
];

/**
 Opening of every reminder this detector issues, followed by the matched phrase in parentheses.

 The once-per-phrase rule finds an earlier reminder in the transcript by this text.
 */
const PARKED_DECISION_REMINDER_PREFIX = 'Your response leaves a choice with the user in prose' as const;

//endregion Phrases

//region Detection

/**
 Result of scanning text for a choice left with the user.
 */
type ParkedDecisionMatch = {
  /**
   First matched phrase.
   */
  phrase: string;
};

/**
 Scans prose for the first phrase that leaves a choice with the user.

 @param prose - text with code blocks and quotes already stripped

 @returns the matched phrase, or `NO_MATCH` when the prose offers nothing

 @example
 ```ts
 findParkedDecision('Say so if you want it posted.'); // { phrase: 'say so if' }
 findParkedDecision('Posted.'); // NO_MATCH
 ```
 */
function findParkedDecision(prose: string,): ParkedDecisionMatch | typeof NO_MATCH {
  /**
   First phrase hit, or `PHRASE_NOT_FOUND`; curly apostrophes are folded so one entry matches both.
   */
  const hit = containsAnyOfWordBounded({
    haystack: normaliseApostrophes(prose,),
    phrases: PARKED_DECISION_PHRASES,
  },);
  if (hit === PHRASE_NOT_FOUND)
    return NO_MATCH;
  return { phrase: hit.phrase, };
}

/**
 Builds the block reason for a matched phrase.

 @param phrase - phrase that matched

 @returns reminder sentences, the first of which opens with {@link PARKED_DECISION_REMINDER_PREFIX}

 @example
 ```ts
 parkedDecisionReminder('say so if')[0]; // 'Your response leaves a choice with the user in prose ("say so if").'
 ```
 */
function parkedDecisionReminder(phrase: string,): readonly string[] {
  return [
    `${PARKED_DECISION_REMINDER_PREFIX} ("${phrase}").`,
    'Use the AskUserQuestion tool to ask the user instead of leaving the choice in prose:',
    'prose may go unread, and the question tool reaches the user.',
    'Put the options and your recommendation in the question, then continue.',
  ];
}

/**
 Reports whether a record is a feedback record that carried this phrase's reminder.

 @param record - parsed transcript record

 @param phrase - phrase whose reminder is sought

 @returns whether the record is this hook's reminder for the phrase

 @example
 ```ts
 carriesReminder({ record, phrase: 'say so if' });
 ```
 */
function carriesReminder(
  {
    record,
    phrase,
  }: {
    readonly record: TranscriptRecord;
    readonly phrase: string;
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
    && content.includes(`${PARKED_DECISION_REMINDER_PREFIX} ("${phrase}")`,);
}

/**
 Reports whether this phrase's reminder was already issued since the last human turn.

 Scans newest first and stops at the first genuine human turn. Records are parsed
 rather than matched as substrings, so a record that merely quotes the reminder (a
 document read back, a command output) does not count as one the agent received.

 @param phrase - phrase about to be raised

 @param transcriptLines - transcript JSONL lines, oldest first

 @returns whether the agent was already told about this phrase in the current turn

 @example
 ```ts
 alreadyRaised({ phrase: 'say so if', transcriptLines: [] }); // false
 ```
 */
function alreadyRaised(
  {
    phrase,
    transcriptLines,
  }: {
    readonly phrase: string;
    readonly transcriptLines: readonly string[];
  },
): boolean {
  for (let index = transcriptLines.length - 1; index >= 0; index--) {
    /**
     Raw transcript line under inspection.
     */
    const line = transcriptLines[index] ?? '';

    // Only a reminder or a human turn can change the answer, so other lines are never parsed.
    if ((line === '')
      || ((!line.includes(PARKED_DECISION_REMINDER_PREFIX,))
        && (!line.includes('"kind":"human"',)))) {
      continue;
    }
    /**
     Parsed record for a line that might be a reminder or a human turn.
     */
    const record = parseRecord(line,);
    if (record === UNPARSABLE)
      continue;
    if (carriesReminder({
      record,
      phrase,
    },)) {
      return true;
    }
    // A human turn closes the window; anything before it belongs to an earlier turn.
    if (isHumanTurnRecord(record,))
      return false;
  }
  return false;
}

//endregion Detection

export type { ParkedDecisionMatch, };

export {
  alreadyRaised,
  findParkedDecision,
  PARKED_DECISION_PHRASES,
  PARKED_DECISION_REMINDER_PREFIX,
  parkedDecisionReminder,
};
