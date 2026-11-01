/**
 Test-only archive review clients exercising real stage boundaries.

 @module
 */
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Source containing every fact the complete replacement preserves.
 */
export const ARCHIVE_TEST_SOURCE = '猫猫有柔软的毛，也很勇敢。';
/**
 Original with one unintended spelling error.
 */
export const ARCHIVE_TEST_BLOCK = '> Cat has soft fur and is couragous.';
/**
 Complete correction, not an excerpt.
 */
export const ARCHIVE_TEST_CORRECTION = '> Cat has soft fur and is courageous.';
/**
 Incomplete proposal the selector must see as a competing proposal.
 */
export const ARCHIVE_TEST_PARTIAL = '> Cat.';
/**
 Archive context already available to the review stage.
 */
export const ARCHIVE_TEST_PAGE: string = `Earlier archive context.\n\n${ARCHIVE_TEST_BLOCK}\n\nFollowing archive context.`;
/**
 Eleven seats require six schema-valid review voices.
 */
export const ARCHIVE_TEST_ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_OPENROUTER_ONLY,
];

/**
 Review shapes vary evidence, never the configured quorum.
 */
type ArchiveReviewFixtureMode = 'mixed' | 'retention-only' | 'unavailable' | 'all-anchored' | 'single' | 'echo';
/**
 Selector can deliberately choose original or lose quorum.
 */
type ArchiveSelectionFixtureMode = 'revision' | 'original' | 'unavailable';

/**
 Locates an actual numbered candidate rather than assuming proposal order.

 @param text - selector's user message

 @param wanted - exact candidate value expected on its slate

 @returns One-based candidate number

 @throws Error where no candidate on the slate is the wanted text

 @example
 ```ts
 const best = candidateNumber({ text: sheet, wanted: ARCHIVE_TEST_CORRECTION, },);
 ```
 */
export function candidateNumber({
  text,
  wanted,
}: {
  readonly text: string;
  readonly wanted: string
},): number {
  /**
   Controlled fixtures contain no candidate-header text in their prose.
   */
  const match = text.split('CANDIDATE ',)
    .slice(1,)
    .find(function containsValue(part,) {
      return part.includes(`\n${wanted}\n`,);
    },);
  if (match === undefined)
    throw new Error('Expected candidate was not offered to the selector',);
  return Number(match.slice(
    0,
    match.indexOf('\n',),
  ),);
}

/**
 Rejects text-only calls outside this fixture's protocol.

 @throws Error on any invocation
 */
function unexpectedText(): never {
  throw new Error('Unexpected text call',);
}

/**
 Prevents accidental access to live quota resources.

 @throws Error on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('Unexpected quota read',);
}

/**
 Builds schema-valid review replies and captures the real selection message.

 @param review - anchor and revision mixture

 @param selection - selector result independent of review replies

 @returns Scripted client and observable selector inputs

 @example
 ```ts
 const fixture = archiveSelectionFixture({ review: 'mixed' });
 ```
 */
export function archiveSelectionFixture(
  {
    review = 'mixed',
    selection = 'revision',
  }: {
    readonly review?: ArchiveReviewFixtureMode;
    readonly selection?: ArchiveSelectionFixtureMode;
  },
): {
  readonly client: ModelClient;
  readonly selections: readonly string[];
  readonly reviews: () => number
} {
  /**
   First-window review callbacks, with all seven schema-valid replies delivered.
   */
  const reviewRequests: string[] = [];
  /**
   One entry for every selector request, containing no system instructions.
   */
  const selections: string[] = [];
  /**
   Source-supported replies whose literal anchor is present.
   */
  const retained = {
    disposition: 'source-supported',
    sourceQuote: ARCHIVE_TEST_SOURCE,
    replacementText: '',
    assessment: 'Valid retention opinion.',
  };
  /**
   Invalid retention evidence remains in audit findings, not selector authority.
   */
  const unanchored = {
    ...retained,
    sourceQuote: 'This is not in the source.',
    assessment: 'Unanchored retention opinion.',
  };
  /**
   Both complete and incomplete schema-valid revisions require independent selection.
   */
  const corrected = {
    disposition: 'revise',
    sourceQuote: '',
    replacementText: ARCHIVE_TEST_CORRECTION,
    assessment: 'Correct the spelling without changing meaning.',
  };
  /**
   Incomplete value tests that independent selection receives the actual proposal.
   */
  const partial = {
    ...corrected,
    replacementText: ARCHIVE_TEST_PARTIAL,
    assessment: 'A competing partial proposal.',
  };
  /**
   Fixed review shapes make the guard independent of prompt-rotated model order.
   */
  const replies = review === 'retention-only'
    ? [
      retained,
      retained,
      unanchored,
      unanchored,
      unanchored,
      unanchored,
      unanchored,
    ]
    : review === 'all-anchored'
      ? [
        retained,
        retained,
        retained,
        retained,
        retained,
        corrected,
        partial,
      ]
      : review === 'single'
        ? [
          retained,
          unanchored,
          unanchored,
          unanchored,
          unanchored,
          unanchored,
          corrected,
        ]
        : [
          retained,
          retained,
          unanchored,
          unanchored,
          unanchored,
          corrected,
          partial,
        ];
  /**
   Synchronous fixture logic, adapted to the provider's asynchronous interface.

   @param request - actual production role and schema

   @returns Scripted schema-valid outcome

   @throws Error when the fixture's review bench is unavailable past its first seat
   */
  function respond<ValueT,>(request: ChatJsonRequest<ValueT>,): ChatJsonOutcome<ValueT> {
      /**
       Role is identified by the actual production schema, not call order.
       */
    const schema = request.responseFormat
      ?.json_schema
      .name;
    if (schema === 'archive_block_review') {
        /**
         First-window position, independent of the prompt-rotated model identity.
         */
      const index = reviewRequests.length;
      reviewRequests.push(request.modelId,);
        // A seat that is unavailable delivers nothing. Until 2026-09-16 this
        // was scripted as `schema-mismatch`, which is an answer nobody could
        // read, and class thirty-one reads that as the bench answering: Mio13
        // was interrupted `provider-unavailable` at 4 of 12 heard with every
        // provider wet because seven cap-cut replies were counted as silence.
      if ((review === 'unavailable') && (index > 0))
        throw new Error('Review quorum unavailable in fixture',);
        /**
         Echoes remain actual revision replies but do not change the block.
         */
      const value: unknown = review === 'unavailable' ? corrected : review === 'echo'
        ? {
          ...corrected,
          replacementText: ARCHIVE_TEST_BLOCK,
        }
        : replies[index];
      if (!request.validate(value,))
        throw new Error('Unexpected review call or invalid fixture reply',);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    }
    if (schema !== 'candidate_ballot')
      throw new Error(`Unexpected schema ${String(schema,)}`,);
      /**
       Actual candidate/evidence data used to identify the intended choice.
       */
    const text = request.messages
      .filter(function user(message,) {
        return message.role === 'user';
      },)
      .map(function content(message,) {
        return messageText({ message, },);
      },)
      .join('\n',);
    selections.push(text,);
    if (selection === 'unavailable')
      return {
        kind: 'schema-mismatch',
        rawText: '{}',
        detail: 'Selector quorum unavailable in fixture',
      };
      /**
       Decision binds to the candidate's content rather than assumed order.
       */
    const value: unknown = {
      best: candidateNumber({
        text,
        wanted: selection === 'original' ? ARCHIVE_TEST_BLOCK : ARCHIVE_TEST_CORRECTION,
      },),
      assessment: 'Fixture selects the exact intended block.',
    };
    if (!request.validate(value,))
      throw new Error('Invalid fixture selector reply',);
    return {
      kind: 'ok',
      value,
      rawText: JSON.stringify(value,),
    };
  }
  /**
   No external provider is reachable through this fixture.
   */
  const client: ModelClient = {
    /**
     Text-only calls are outside this scripted protocol.
     */
    chatText: unexpectedText,
    /**
     Fixtures cannot query live provider resources.
     */
    quotas: unexpectedQuotas,
    /**
     Adapts deterministic reply construction to the provider interface.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      return Promise.resolve(respond(request,),);
    },
  };
  return {
    client,
    selections,
    /**
     Read-only view of how many review calls the stage made.
     */
    reviews: function reviews(): number {
      return reviewRequests.length;
    },
  };
}
