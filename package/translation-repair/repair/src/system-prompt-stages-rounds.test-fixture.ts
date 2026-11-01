/**
 The translate slate over wording that cannot ship, once for each round of a pair, handed a subject, for the guard that
 holds every stage's system prompt to the same text on every call. Only the decline text differs between the three, and
 each is its own entry so the guard reads the sentence of every round (re-check 5, the rendered-sheets gap). The other
 selection slates are in `system-prompt-stages-slates.test-fixture.ts`.

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  buildCandidateSelectMessages,
  SETTLES_NOTHING_AND_ASKS_AGAIN,
  SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN,
  SHIPS_WHAT_TWO_JUDGES_NAMED,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  TRANSLATE_SELECTION_TASK,
  translateSelectionCriteria,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { translatedEvidence, } from './system-prompt-stages-slates.test-fixture.ts';
import type {
  PromptMessages,
  PromptStage,
  PromptSubject,
} from './system-prompt-subject.test-fixture.ts';

//region Round stages

/**
 The slate the translate lane asks over wording that cannot ship, for one round of a pair.

 @param subject - invented subject

 @param declineConsequence - what the round tells its judges a decline does

 @returns Slate's messages

 @example
 ```ts
 const messages = withheldWordingSlateOf({ subject: CAT_SUBJECT, declineConsequence: SETTLES_NOTHING_AND_ASKS_AGAIN, },);
 ```
 */
function withheldWordingSlateOf(
  {
    subject,
    declineConsequence,
  }: {
    readonly subject: PromptSubject;
    readonly declineConsequence: string;
  },
): PromptMessages {
  return buildCandidateSelectMessages({
    task: TRANSLATE_SELECTION_TASK,
    criteria: translateSelectionCriteria({
      lineStructured: false,
      syntax: 'front-matter',
    },),
    evidence: translatedEvidence({
      subject,
      sourceText: subject.source,
    },),
    rendered: [
      subject.archive,
      subject.repair,
    ],
    declineConsequence,
    sourceText: subject.source,
  },);
}

/**
 The three rounds of a pair over wording that cannot ship: the challenge that ships past a decline, the first ask, and
 the challenge that defers.
 */
export const ROUND_STAGES: readonly PromptStage[] = [
  {
    name: 'translate front matter slate',
    render: function renderTranslateFrontMatterSlate({ subject, },) {
      return withheldWordingSlateOf({
        subject,
        declineConsequence: SHIPS_WHAT_TWO_JUDGES_NAMED,
      },);
    },
  },
  {
    name: 'translate first ask slate',
    render: function renderTranslateFirstAskSlate({ subject, },) {
      return withheldWordingSlateOf({
        subject,
        declineConsequence: SETTLES_NOTHING_AND_ASKS_AGAIN,
      },);
    },
  },
  {
    name: 'translate deferring challenge slate',
    render: function renderTranslateDeferringChallengeSlate({ subject, },) {
      return withheldWordingSlateOf({
        subject,
        declineConsequence: SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN,
      },);
    },
  },
];

//endregion Round stages
