import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { wait, } from '@monochromatic-dev/async-time-core/ts';

//region Scripted review client
// THE SCRIPTED CLIENT EVERY ARCHIVE BLOCK REVIEW CASE DRIVES THE STAGE WITH,
// kept in one place so the cases split across files (the stage's own, the
// straggler windows', the composite's) share one client instead of each
// holding a copy of it.

/**
 Reply selector for one scripted request.
 */
type ReplyFor = (input: {
  readonly schema: string;
  readonly prompt: string;
  readonly modelId: string;
}) => unknown;

/**
 How long a held seat's reply waits after its request, in milliseconds. It is
 long enough that the stage hears every seat answering at once, as every other
 scripted seat does, tallies quorum and arms its straggler window before the
 held one lands, and short enough that a case whose window waits for it (the
 hang stop) ends well within the case. The scripted peers answer at once, but
 the stage's own work between their answers and the tally (logging, the
 tally's turns of the event loop) is real time: at 25 ms the rotation sweep's
 load once let the held seat land before quorum, so the readers' window had no
 straggler to cut.
 */
const HELD_REPLY_MS = 1_000;

/**
 Reads no seat as cut before its content.

 @returns Always false
 */
function noneCut(): boolean {
  return false;
}

/**
 Reads no seat as refused by the router.

 @returns Always false
 */
function noneRefused(): boolean {
  return false;
}

/**
 Reads no seat as held back.

 @returns Always false
 */
function noneHeld(): boolean {
  return false;
}

/**
 Refuses the text exchange, which the archive block review never asks.
 */
function refuseChatText(): Promise<never> {
  return Promise.reject(new Error('chatText not used',),);
}

/**
 Refuses the quota read, which the archive block review never asks.
 */
function refuseQuotas(): Promise<never> {
  return Promise.reject(new Error('quotas not used',),);
}

/**
 Creates schema-aware direct client and captures exact prompts.

 @param replyFor - reply selector

 @param prompts - prompt capture sink

 @param payloads - optional model-plus-prompt identity sink

 @param unreadableFor - seats whose reply to a prompt the completion cap cut
 before its content, the shape `chat-json-outcome.ts` reads off
 `finish_reason=length`; the prompt lets a case cut one review's replies and
 not another's

 @param refusedFor - seats the router refuses because no provider serving them
 is wet, the refusal `stage-call.ts` counts out of reach

 @param heldFor - seats whose reply to a schema comes after every seat that
 answers at once, so a round with a window of its own hears it only when the
 window is long enough

 @returns Scripted client

 @example
 ```ts
 const client = scriptedClient({ replyFor, prompts, },);
 ```
 */
export function scriptedClient(
  {
    replyFor,
    prompts,
    payloads,
    unreadableFor = noneCut,
    refusedFor = noneRefused,
    heldFor = noneHeld,
  }: {
    readonly replyFor: ReplyFor;
    readonly prompts: string[];
    readonly payloads?: string[];
    readonly unreadableFor?: (request: {
      readonly modelId: string;
      readonly prompt: string;
    }) => boolean;
    readonly refusedFor?: (modelId: string) => boolean;
    readonly heldFor?: (request: {
      readonly modelId: string;
      readonly schema: string;
    }) => boolean;
  },
): ModelClient {
  return {
    chatText: refuseChatText,
    chatJson: async function chatJson<ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Structured schema naming requested responsibility.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name
        ?? '';
      /**
       Complete prompt used for responsibility and anonymity assertions.
       */
      const prompt = JSON.stringify(request.messages,);
      prompts.push(prompt,);
      payloads?.push(`${request.modelId}\u0000${prompt}`,);
      if (refusedFor(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }
      if (heldFor({
        modelId: request.modelId,
        schema,
      },))
        await wait(HELD_REPLY_MS,);
      if (unreadableFor({
        modelId: request.modelId,
        prompt,
      },)) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          reason: 'truncated-completion',
          detail: 'provider reported a truncating completion (model stopped with finish_reason=length)',
        };
      }
      /**
       Scripted value for current role.
       */
      const value = replyFor({
        schema,
        prompt,
        modelId: request.modelId,
      },);
      if (!request.validate(value,))
        throw new Error(`scripted ${schema} reply failed validator`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: refuseQuotas,
  };
}

//endregion Scripted review client
