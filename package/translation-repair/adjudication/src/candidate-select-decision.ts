import {
  CANDIDATE_NONE,
  type CandidateBallotAsSent,
  judgedSelection,
  KEEPS_TRUSTED_TEXT,
  type SelectEvidence,
} from './candidate-select-wire.ts';
import {
  communityRenderingsBlock,
  type RenderingCandidate,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { DecisionAnswer, } from '@monochromatic-dev/translation-repair-provider/ts';
import { JUDGE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { StageDecision, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Candidate selection as a typed question
// THE SELECT BALLOT PHRASED FOR A DECISION SEAT: the same task, criteria,
// evidence and anonymized candidates the chat sheet carries
// (`buildCandidateSelectMessages`), as a state object, and one choice
// question over the candidate numbers plus the decline. A decision seat
// answers with a number and a distribution and never a reason, so the
// ballot's `assessment` carries the distribution, which is the audit trail a
// typed judge can give.
//
// THE HOUSE RULES TRAVEL IN THE STATE (the whole-package audit, 2026-09-27).
// The state once carried the criteria alone, so the seat read "every
// proposition of the ORIGINAL is rendered" with no word of reader
// protection, the tense order or the precedence line, and never heard of a
// community departure the chat seat was told of.

/**
 One evidence block as the state carries it.

 @example
 ```ts
 const block: EvidenceBlock = { label: 'ORIGINAL (Chinese)', text: '猫在窗台上打盹。', };
 ```
 */
type EvidenceBlock = {
  /**
   Heading naming what the text is.
   */
  readonly label: string;

  /**
   Material itself.
   */
  readonly text: string;
};

/**
 Label the ballot's reason opens with, so a reader can tell a typed ballot
 from a written one.
 */
export const TYPED_BALLOT_REASON = 'typed decision';

/**
 What a distribution or a confidence reads as when the answer carries none.
 */
const UNREPORTED = 'unreported';

/**
 What the reading hands the stage guard when the answers carry no choice for
 the ballot question: a value no guard admits, so the seat's voice is lost
 rather than read as a decline.
 */
export const NO_TYPED_ANSWER: unique symbol = Symbol('decision seat replied without choosing a candidate number (select stage, 2026-09-18)',);

/**
 Reads a decision seat's answers into a select ballot.

 @param answers - answers by question name

 @returns Ballot whose reason carries the distribution, or the marker no
 guard admits when the seat chose no candidate

 @example
 ```ts
 readTypedBallot({ best: { type: 'choice', choice: 1, }, },);
 ```
 */
function readTypedBallot(
  answers: Readonly<Record<string, DecisionAnswer>>,
): CandidateBallotAsSent | typeof NO_TYPED_ANSWER {
  /**
   Answer to the one question, when it is a choice.
   */
  const { best, } = answers;
  if ((best === undefined) || (best.type !== 'choice'))
    return NO_TYPED_ANSWER;
  // AN ABSENT DISTRIBUTION READS AS UNREPORTED, like an absent confidence: an
  // empty object said the seat reported a distribution with nothing in it,
  // which no answer said (ledger T8).
  return {
    best: best.choice,
    assessment: `${TYPED_BALLOT_REASON}: probabilities ${
      (best.probabilities === undefined) ? UNREPORTED : JSON.stringify(best.probabilities,)
    }, confidence ${(best.confidence === undefined) ? UNREPORTED : String(best.confidence,)}`,
  };
}

/**
 Builds the select stage's question for a decision seat.

 @param task - what the candidates are attempting, in one sentence

 @param criteria - ordered decision rules, most important first

 @param evidence - source and baseline material judges compare against

 @param rendered - candidate texts in caller-fixed order

 @param declineConsequence - what the caller does when every judge declines

 @param sourceText - original the candidates render, which names the
 community renderings a candidate lacks; omitted where the caller has none

 @returns State, one choice question and the reading into a ballot

 @example
 ```ts
 const decision = selectDecision({ task, criteria, evidence, rendered, },);
 ```
 */
export function selectDecision(
  {
    task,
    criteria,
    evidence: held,
    rendered: heldRendered,
    declineConsequence = KEEPS_TRUSTED_TEXT,
    sourceText,
  }: {
    readonly task: string;
    readonly criteria: readonly string[];
    readonly evidence: readonly SelectEvidence[];
    readonly rendered: readonly string[];
    readonly declineConsequence?: string;
    readonly sourceText?: string;
  },
): StageDecision {
  /**
   Evidence and candidates as the seat reads them, without the notes between
   translators, as the chat sheet cuts them.
   */
  const {
    evidence,
    rendered,
  } = judgedSelection({
    evidence: held,
    rendered: heldRendered,
    ...((sourceText === undefined) ? {} : { sourceText, }),
  },);
  /**
   Community renderings a candidate lacks, as the chat sheet names them;
   empty where the caller passed no original or none departs.
   */
  const communityRenderings = (sourceText === undefined)
    ? []
    : communityRenderingsBlock({
      sourceText,
      candidates: rendered.map(function toCandidate(
        text,
        index,
      ): RenderingCandidate {
        return {
          label: `CANDIDATE ${String(index + 1,)}`,
          text,
        };
      },),
    },);
  /**
   Options: the decline, then one per candidate by its one-based number.
   */
  const options: Readonly<Record<string, string>> = Object.fromEntries([
    [
      String(CANDIDATE_NONE,),
      'no candidate is acceptable',
    ],
    ...rendered.map(function option(
      _text,
      index,
    ): readonly [
      string,
      string,
    ] {
      return [
        String(index + 1,),
        `candidate ${String(index + 1,)} is the best rendering`,
      ];
    },),
  ],);
  return {
    state: {
      task,
      policy: JUDGE_POLICY_BLOCK,
      ...((communityRenderings.length === 0) ? {} : { communityRenderings: communityRenderings.join('\n',), }),
      criteria: Object.fromEntries(criteria.map(function rule(
        text,
        index,
      ): readonly [
        string,
        string,
      ] {
        return [
          String(index + 1,),
          text,
        ];
      },),),
      evidence: evidence.map(function block(entry,): EvidenceBlock {
        return {
          label: entry.label,
          text: entry.text,
        };
      },),
      candidates: Object.fromEntries(rendered.map(function candidate(
        text,
        index,
      ): readonly [
        string,
        string,
      ] {
        return [
          String(index + 1,),
          text,
        ];
      },),),
    },
    questions: {
      best: {
        type: 'choice',
        instructions: `${task} Decide by the numbered rules in state.criteria, earlier ones outranking later ones, `
          + 'under the house rules in state.policy, which outrank every criterion where the two disagree. '
          + 'Where state.communityRenderings is present, weigh it as evidence, not a verdict. '
          + 'You do not know which system produced which candidate. Judge only the texts in state.candidates '
          + `against state.evidence. Choose ${String(CANDIDATE_NONE,)} when NO candidate is acceptable; `
          + `${declineConsequence}.`,
        criteria: options,
      },
    },
    read: readTypedBallot,
  };
}

//endregion Candidate selection as a typed question
