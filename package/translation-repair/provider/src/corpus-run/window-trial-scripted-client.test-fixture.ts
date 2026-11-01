/**
 Test-only client for the window trial: translators answer with a rendering of
 their own and judges cast one ballot per sheet, one for sheets showing the
 neighbouring original and one for the rest, so a case chooses which arms
 replace the archive. No call reaches a network.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '../../dist/final/node/index.mjs';

import { replyWith, } from '../scripted-reply-outcome.test-fixture.ts';

/**
 Schema name the producing half asks translators for.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Start of a heading line of the fixture pages' archive English.
 */
const HEADING_START = '## Section ';

/**
 Label the wide arm's sheets carry.
 */
const WINDOW_LABEL = 'SURROUNDING ORIGINAL';

/**
 Text chat, which the walk never asks for.

 @throws {@link Error} always

 @example
 ```ts
 const chatText = chatTextUnused;
 ```
 */
function chatTextUnused(): never {
  throw new Error('chatText unused',);
}

/**
 Quota reading, which the walk never asks for.

 @throws {@link Error} always

 @example
 ```ts
 const quotas = quotasUnused;
 ```
 */
function quotasUnused(): never {
  throw new Error('quotas unused',);
}

/**
 The sheet a request carries, its messages joined.

 @param request - request a model is asked

 @returns Every message's text, one per line

 @example
 ```ts
 const sheet = sheetOf({ request, },);
 ```
 */
function sheetOf<ValueT,>({ request, }: { readonly request: ChatJsonRequest<ValueT>; },): string {
  return request.messages
    .map(function toContent(message,): string {
      return messageText({ message, },);
    },)
    .join('\n',);
}

/**
 Name of the schema a request asks its answer in.

 @param request - request a model is asked

 @returns The schema's name, empty where the request names none

 @example
 ```ts
 const asksTranslation = schemaNameOf({ request, },) === 'translation_report';
 ```
 */
function schemaNameOf<ValueT,>({ request, }: { readonly request: ChatJsonRequest<ValueT>; },): string {
  /**
   Format the answer is asked in, when the request names one.
   */
  const { responseFormat, } = request;
  if (responseFormat === undefined)
    return '';
  /**
   Schema the answer is asked in.
   */
  const { json_schema: schema, } = responseFormat;
  return schema.name;
}

/**
 A scripted client with the calls it served.

 @example
 ```ts
 const rig: TrialRig = scriptedTrialClient({ narrowBallot: 0, wideBallot: 1, },);
 ```
 */
type TrialRig = {
  /**
   Client the walk calls through.
   */
  readonly client: ModelClient;

  /**
   Translator calls served, one per translator asked.
   */
  readonly served: { count: number; };

  /**
   Sheets the judges received, in order.
   */
  readonly judgeSheets: string[];
};

/**
 Builds a client whose judges vote as a case says.

 @param narrowBallot - candidate every judge votes for on a sheet without the
 neighbouring original, 0 for the archive's own wording

 @param wideBallot - candidate every judge votes for on a sheet with it

 @returns The client and what it served

 @example
 ```ts
 const { client, served, judgeSheets, } = scriptedTrialClient({ narrowBallot: 0, wideBallot: 1, },);
 ```
 */
export function scriptedTrialClient(
  {
    narrowBallot,
    wideBallot,
  }: {
    readonly narrowBallot: number;
    readonly wideBallot: number;
  },
): TrialRig {
  /**
   Translator calls served.
   */
  const served = { count: 0, };

  /**
   Sheets the judges received.
   */
  const judgeSheets: string[] = [];

  return {
    served,
    judgeSheets,
    client: {
      chatText: chatTextUnused,
      quotas: quotasUnused,
      chatJson: function chatJson<ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> {
        if (schemaNameOf({ request, },) === TRANSLATE_SCHEMA) {
          served.count += 1;

          /**
           Heading the page already carries, which the publication rule
           requires every rendering to keep in its own place.
           */
          const heading = sheetOf({ request, },)
            .split('\n',)
            .find(function isHeading(line,): boolean {
              return line.startsWith(HEADING_START,);
            },);
          return Promise.resolve(replyWith({
            report: {
              translation: `${(heading === undefined) ? '' : `${heading}\n\n`}A fresh rendering number ${String(served.count,)}.`,
            },
            request,
          },),);
        }

        /**
         Sheet this judge was shown.
         */
        const sheet = sheetOf({ request, },);
        judgeSheets.push(sheet,);
        return Promise.resolve(replyWith({
          report: {
            best: sheet.includes(WINDOW_LABEL,) ? wideBallot : narrowBallot,
            assessment: 'fixture',
          },
          request,
        },),);
      },
    },
  };
}
