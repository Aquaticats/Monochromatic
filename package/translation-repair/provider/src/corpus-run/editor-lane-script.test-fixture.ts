/**
 A model client that lets one invented slice go through the whole repair lane
 and ship an edit, with no network: every critic raises the one claim, every
 panelist and every checker agrees with it, every editor writes the same edit,
 and nobody finds damage in it. Each stage answers by the schema it asks for,
 whichever model it is asked on behalf of.

 @module
 */

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '../../dist/final/node/index.mjs';

/**
 Source sentence the claim quotes, which the slice of `benchSliceOf` carries.
 */
const SOURCE_QUOTE = '小猫在窗台上打盹。';

/**
 Archive sentence the claim quotes and the edit replaces.
 */
const TARGET_QUOTE = 'The kitten dozes on the windowsill.';

/**
 What a damage prober says of a region it found nothing wrong in.
 */
const NO_DEFECT_FOUND = {
  verdict: 'no-introduced-defect-found',
  category: '',
  severity: '',
  evidence: '',
  omittedText: '',
  verdictReason: '',
};

/**
 Text every editor writes in its place.
 */
const EDITED_SENTENCE = 'The kitten naps on the windowsill.';

/**
 Counts how many items of a sheet a stage was handed.

 @param request - exchange as the stage sent it

 @param marker - text each item of the user sheet begins with

 @returns How many items the sheet carries

 @example
 ```ts
 const claims = itemsOnSheet({ request, marker: '\nCLAIM ', },);
 ```
 */
function itemsOnSheet(
  {
    request,
    marker,
  }: {
    readonly request: ChatJsonRequest<unknown>;
    readonly marker: string;
  },
): number {
  /**
   User sheet of this exchange.
   */
  const last = request
    .messages
    .at(-1,);

  /**
   Text of that sheet, empty when the exchange carried none.
   */
  const sheet = (last === undefined) ? '' : messageText({ message: last, },);

  /**
   The sheet cut at every item's start, one piece more than there are items.
   */
  const pieces = sheet.split(marker,);

  return pieces.length - 1;
}

/**
 Builds the reply a stage gets, from the schema name and the sheet it was sent.

 @param stage - schema name the stage asked for

 @param request - exchange as the stage sent it

 @returns Reply in the stage's wire shape

 @throws Error naming the stage when the lane asks for one this client does not answer

 @example
 ```ts
 const reply = replyFor({ stage: 'critic_report', request, },);
 ```
 */
function replyFor(
  {
    stage,
    request,
  }: {
    readonly stage: string;
    readonly request: ChatJsonRequest<unknown>;
  },
): unknown {
  if (stage === 'critic_report') {
    return {
      issues: [
        {
          category: 'accuracy/mistranslation',
          severity: 'major',
          summary: 'Napping is rendered as dozing.',
          sourceQuote: SOURCE_QUOTE,
          targetQuote: TARGET_QUOTE,
        },
      ],
    };
  }
  if (stage === 'panel_ballot') {
    /**
     Claims the panel was handed.
     */
    const claims = itemsOnSheet({
      request,
      marker: '\nCLAIM ',
    },);
    return {
      verdicts: Array.from(
        { length: claims, },
        function supported(
          _unused,
          index,
        ) {
          return {
            claim: index + 1,
            reason: 'The quotes show it.',
            vote: 'supported',
          };
        },
      ),
    };
  }
  if (stage === 'editor_report') {
    return {
      edits: [
        {
          region: 1,
          newText: EDITED_SENTENCE,
        },
      ],
    };
  }
  if (stage === 'resolution_report') {
    /**
     Issues the checkers were handed.
     */
    const issues = itemsOnSheet({
      request,
      marker: '\nISSUE ',
    },);
    return {
      checks: Array.from(
        { length: issues, },
        function fixed(
          _unused,
          index,
        ) {
          return {
            issue: index + 1,
            verdict: 'fixed',
          };
        },
      ),
    };
  }
  if (stage === 'introduced_defect_report') {
    /**
     Regions the damage probers were handed.
     */
    const regions = itemsOnSheet({
      request,
      marker: ' REGION ',
    },);
    return {
      checks: Array.from(
        { length: regions, },
        function undamaged(
          _unused,
          index,
        ) {
          return {
            ...NO_DEFECT_FOUND,
            region: index + 1,
          };
        },
      ),
    };
  }
  throw new Error(`the shipping lane script holds no answer for the ${stage} stage`,);
}

/**
 Refuses a text call, which no stage of the accuracy lane makes.

 @throws Error on any invocation

 @example
 ```ts
 const chatText = unscriptedText;
 ```
 */
function unscriptedText(): never {
  throw new Error('chatText is not scripted',);
}

/**
 Refuses a quota read, which no stage of the accuracy lane makes.

 @throws Error on any invocation

 @example
 ```ts
 const quotas = unscriptedQuotas;
 ```
 */
function unscriptedQuotas(): never {
  throw new Error('quotas is not scripted',);
}

/**
 Answers one structured stage as the script says.

 @param request - exchange as the stage sent it

 @returns The scripted reply, which the stage's own guard has accepted

 @example
 ```ts
 const outcome = await scriptedChatJson(request,);
 ```
 */
function scriptedChatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
  /**
   Reply this stage is scripted to give.
   */
  const reply = replyFor({
    stage: request.responseFormat
      ?.json_schema
      .name
      ?? '',
    request,
  },);
  if (!request.validate(reply,))
    return Promise.reject(new Error('the scripted reply failed the stage\'s own guard',),);
  return Promise.resolve({
    kind: 'ok',
    value: reply,
    rawText: JSON.stringify(reply,),
  },);
}

/**
 Builds the client whose every stage lets the slice ship the one edit.

 @returns Client answering the five stages of the accuracy lane and refusing every other call

 @example
 ```ts
 const rounds = await runEditorLane({ slice, client: shippingLaneClient(), },);
 ```
 */
export function shippingLaneClient(): ModelClient {
  return {
    chatText: unscriptedText,
    chatJson: scriptedChatJson,
    quotas: unscriptedQuotas,
  };
}
