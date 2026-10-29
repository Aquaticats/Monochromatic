import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isJsonArray,
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { AttestedDetail, } from './reference-attest-match.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Reference attestation confirmation wire
// CLASS FORTY-THREE (Mio26, 2026-09-17). The open question ("list every
// detail...") is answered with an empty list by voices that never looked:
// on Mio26 four of the five heard voices returned `{"attested":[]}` in six to
// 2,416 completion tokens, one voice attested the sister with both quotes
// verifying, and the quorum of three dropped it. The same bench had attested
// it five of six on Mio25. So extraction proposes and confirmation disposes:
// every verified detail goes back to the bench as a numbered candidate with
// its two quotes, and a yes-or-no on a quote pair is a question a voice
// answers by reading rather than by scanning.

/**
 Reply to the confirmation question: the numbers of the candidates the
 voice confirms.

 @example
 ```ts
 const wire: ReferenceAttestConfirmWire = { confirmed: [1, 3,], };
 ```
 */
export type ReferenceAttestConfirmWire = {
  /**
   One-based candidate numbers, as the sheet numbers them.
   */
  readonly confirmed: readonly number[];
};

/**
 One voice's confirmation ballot.

 @example
 ```ts
 const ballot: ConfirmationBallot = { modelId, confirmed: [1,], };
 ```
 */
export type ConfirmationBallot = {
  /**
   Voice that answered.
   */
  readonly modelId: RosterModelId;

  /**
   Candidate numbers it confirmed.
   */
  readonly confirmed: readonly number[];
};

/**
 Builds the confirmation question over the verified candidates.

 @param sourceText - original document

 @param archiveText - archive rendering as inherited

 @param referenceContext - reference lines, one per page

 @param candidates - verified details, numbered from one in this order

 @returns Request messages

 @example
 ```ts
 const messages = buildReferenceAttestConfirmMessages({ sourceText, archiveText, referenceContext, candidates, },);
 ```
 */
export function buildReferenceAttestConfirmMessages(
  {
    sourceText,
    archiveText,
    referenceContext,
    candidates,
  }: {
    readonly sourceText: string;
    readonly archiveText: string;
    readonly referenceContext: string;
    readonly candidates: readonly AttestedDetail[];
  },
): readonly ChatMessage[] {
  /**
   Candidate lines, numbered from one.
   */
  const candidateLines = candidates.map(function toLine(
    candidate,
    index,
  ): string {
    return `CANDIDATE ${String(index + 1,)}\nARCHIVE RENDERING says: "${candidate.archiveQuote}"\nREFERENCE ${
      String(candidate.reference,)
    } says: "${candidate.referenceQuote}"`;
  },);
  // THE VOICE IS SHOWN THE ARCHIVE RENDERING WITHOUT THE NOTES THE ARCHIVE'S
  // TRANSLATORS LEFT EACH OTHER, as the voices whose candidates it weighs were
  // (`reference-attest-wire.ts`): no reader sees one. The candidates' quotes
  // were verified against the stored text and are shown as given, and the
  // original's own comments stay in view.
  /**
   The archive rendering as the voice reads it.
   */
  const archiveShown = withoutTranslatorsNotes({
    text: archiveText,
    original: sourceText,
  },);
  /**
   Fence absent from every enclosed value, the archive rendering as written
   and as shown.
   */
  const fence = selectFence({ texts: [
    sourceText,
    archiveText,
    archiveShown,
    referenceContext,
    ...candidateLines,
  ], },);
  return [
    {
      role: 'system',
      content: `You are shown an ORIGINAL, its ARCHIVE RENDERING (a human translation of the ORIGINAL), CITED REFERENCES (what the pages the ORIGINAL itself links say), and numbered CANDIDATES. Each candidate pairs a sentence of the ARCHIVE RENDERING with a passage of a cited reference that another reader says states the same detail.

For each candidate, read both quotes and decide whether the reference passage states the detail the archive sentence carries: the same fact about the same person or thing, whatever the wording. Confirm it when it does. Do not confirm a pair whose reference passage is about something else, or states less than the archive sentence claims.

Reply with JSON only: {"confirmed":[1,3]} naming the candidates you confirm by number, or {"confirmed":[]} when you confirm none.`,
    },
    {
      role: 'user',
      content: `${fence} ORIGINAL ${fence}\n${sourceText}\n${fence} ARCHIVE RENDERING ${fence}\n${archiveShown}\n${fence} CITED REFERENCES ${fence}\n${referenceContext}\n${fence} CANDIDATES ${fence}\n${
        candidateLines.join('\n\n',)
      }\n${fence}`,
    },
  ];
}

/**
 Guards confirmation JSON.

 @param value - parsed provider value

 @returns Whether the reply is a list of integers

 @example
 ```ts
 isReferenceAttestConfirmWire(JSON.parse(text,));
 ```
 */
export function isReferenceAttestConfirmWire(value: unknown,): value is ReferenceAttestConfirmWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.confirmed,))
    return false;
  return value.confirmed
    .every(function isCandidateNumber(entry,): boolean {
      return ((typeof entry) === 'number') && Number.isInteger(entry,);
    },);
}

/**
 Structured output constraint for confirmation replies.
 */
export const REFERENCE_ATTEST_CONFIRM_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'reference_attest_confirm',
    schema: {
      type: 'object',
      required: ['confirmed',],
      additionalProperties: false,
      properties: {
        confirmed: {
          type: 'array',
          items: { type: 'integer', },
        },
      },
    },
  },
};

/**
 Candidates enough distinct voices confirmed, with the confirmation counts
 in place of the extraction counts.

 @param candidates - verified details as numbered on the sheet

 @param ballots - one per voice heard

 @param needed - distinct confirming voices a candidate needs

 @returns Confirmed details in candidate order

 @example
 ```ts
 const details = confirmedDetails({ candidates, ballots, needed: 2, },);
 ```
 */
export function confirmedDetails(
  {
    candidates,
    ballots,
    needed,
  }: {
    readonly candidates: readonly AttestedDetail[];
    readonly ballots: readonly ConfirmationBallot[];
    readonly needed: number;
  },
): readonly AttestedDetail[] {
  return candidates
    .map(function withVotes(
      candidate,
      index,
    ): AttestedDetail {
      /**
       Number this candidate carries on the sheet.
       */
      const number = index + 1;
      /**
       Distinct voices naming it; a ballot naming it twice counts once.
       */
      const naming = ballots.filter(function names(ballot,): boolean {
        return ballot.confirmed
          .includes(number,);
      },);
      /**
       How many named it.
       */
      const voices = naming.length;
      return {
        ...candidate,
        voices,
        heard: ballots.length,
      };
    },)
    .filter(function enough(detail,): boolean {
      return detail.voices >= needed;
    },);
}

//endregion Reference attestation confirmation wire
