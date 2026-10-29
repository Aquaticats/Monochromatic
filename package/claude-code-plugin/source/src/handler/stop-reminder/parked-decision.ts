/**
 Detection of a choice left with the user in prose.

 An agent that writes "Say so if you want it" or "Tell me if you'd rather" has a
 decision for the user but routes it through text the user may not read. The
 question tool is the channel that reaches the user, so this detector blocks the
 stop and says to use it. The trailing-question detector cannot cover this wording
 because it needs a literal question mark.

 @module
 */

import {
  containsAnyOfWordBounded,
  PHRASE_NOT_FOUND,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
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
 Opening sentence of the reminder for a matched phrase.

 @param phrase - phrase that matched

 @returns the sentence the reminder opens with

 @example
 ```ts
 parkedDecisionLead('say so if'); // 'Your response leaves a choice with the user in prose ("say so if").'
 ```
 */
function parkedDecisionLead(phrase: string,): string {
  return `${PARKED_DECISION_REMINDER_PREFIX} ("${phrase}").`;
}

/**
 Builds the block reason for a matched phrase.

 @param phrase - phrase that matched

 @returns reminder sentences, the first of which is {@link parkedDecisionLead}

 @example
 ```ts
 parkedDecisionReminder('say so if')[0]; // 'Your response leaves a choice with the user in prose ("say so if").'
 ```
 */
function parkedDecisionReminder(phrase: string,): readonly string[] {
  return [
    parkedDecisionLead(phrase,),
    'Use the AskUserQuestion tool to ask the user instead of leaving the choice in prose:',
    'prose may go unread, and the question tool reaches the user.',
    'Put the options and your recommendation in the question, then continue.',
  ];
}

//endregion Detection

export type { ParkedDecisionMatch, };

export {
  findParkedDecision,
  PARKED_DECISION_PHRASES,
  PARKED_DECISION_REMINDER_PREFIX,
  parkedDecisionLead,
  parkedDecisionReminder,
};
