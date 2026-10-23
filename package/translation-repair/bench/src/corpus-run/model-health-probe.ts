import type { Logger, } from '@monochromatic-dev/module-logger/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Model health probe
// Asks ONE model for one trivial structured answer, and logs what came back.
//
// Built because `schema-mismatch, voice lost` is where three different faults
// arrive wearing one label: output truncated inside a thinking block, content
// that is not JSON at all, and JSON the caller's guard rejected. The client
// distinguishes them and says which at DEBUG level, which a corpus run does not
// record, so a run log shows a model failing and never says why.
//
// One model went from zero mismatches to 507 in four passes on unchanged code,
// across every role it holds. That is a property of the model or of how it is
// being asked, not of any stage, and the cheapest way to tell those apart is to
// ask it something a working model cannot get wrong.
//
// Fixtures are cat-themed invention. No corpus text takes part, and this writes
// nothing.

/**
 Question every model is asked.

 Deliberately trivial. The point is not to test capability: any model that can
 hold a role in this pipeline can answer it, so a failure here is about the
 response FORMAT rather than about the task.
 */
const HEALTH_PROMPT =
  'Reply with JSON matching the schema: how many cats are named in this '
    + 'sentence, and what is the first one called? "Mittens and Tabby sat on the '
    + 'windowsill."';

/**
 How much of a raw reply the log line carries, in UTF-16 units, ending on a
 whole character (`wholeOpening`).

 Enough to show a prefix, a fence, or an apology sitting in front of the JSON,
 which is what this probe was built to catch, and short enough that six models
 fit in one readable screen.
 */
const RAW_REPLY_PREVIEW_CHARS = 300;

/**
 Schema the reply must satisfy.
 */
const HEALTH_RESPONSE_FORMAT = {
  type: 'json_schema',
  json_schema: {
    name: 'cat_count',
    strict: true,
    schema: {
      type: 'object',
      additionalProperties: false,
      required: [
        'count',
        'first',
      ],
      properties: {
        count: { type: 'number', },
        first: { type: 'string', },
      },
    },
  },
} as const;

/**
 Accepts a reply carrying both fields, whatever their values.

 @param value - parsed reply

 @returns Whether the reply has the shape asked for

 @example
 ```ts
 const ok = isHealthReply(JSON.parse(text,),);
 ```
 */
function isHealthReply(value: unknown,): value is {
  readonly count: number;
  readonly first: string;
} {
  return isJsonRecord(value,)
    && ('count' in value)
    && ('first' in value);
}

/**
 Asks one model the health question and logs what returned.

 @param client - client the question goes through

 @param modelId - model asked

 @param timeoutMs - exchange deadline the client arms; a Claude model runs under the larger deadline its card needs

 @param l - logger the outcome lines are written to

 @throws Whatever the client threw, which the caller reports as an unreachable
 model rather than as an unhealthy reply

 @example
 ```ts
 await askModelHealth({ client, modelId, timeoutMs: 360_000, l, },);
 ```
 */
export async function askModelHealth(
  {
    client,
    modelId,
    timeoutMs,
    l,
  }: {
    readonly client: ModelClient;
    readonly modelId: RosterModelId;
    readonly timeoutMs: number;
    readonly l: Logger;
  },
): Promise<void> {
  /**
   What this model returned, or the fault that stopped it.
   */
  const outcome = await client.chatJson({
    modelId,
    messages: [
      {
        role: 'user',
        content: HEALTH_PROMPT,
      },
    ],
    responseFormat: HEALTH_RESPONSE_FORMAT,
    validate: isHealthReply,
    exchangeTimeoutMs: timeoutMs,
    signal: new AbortController().signal,
  },);

  l.info(
    `${modelId}: ${outcome.kind}${
      ('detail' in outcome) ? ` -- ${outcome.detail}` : ''
    }`,
  );
  if (('rawText' in outcome) && ((typeof outcome.rawText) === 'string'))
    l.info(
      `${modelId}: raw reply opening (at most ${
        String(RAW_REPLY_PREVIEW_CHARS,)
      } UTF-16 units): ${
        JSON.stringify(wholeOpening({
          text: outcome.rawText,
          units: RAW_REPLY_PREVIEW_CHARS,
        },),)
      }`,
    );
}

//endregion Model health probe
