/**
 OpenAI's policy stop, built through the installed Responses adapter over an in-memory stream.
 @module
 */

import type {
  Api,
  AssistantMessageEventStream,
  Model,
  SimpleStreamOptions,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import { streamSimple as streamResponses, } from '@earendil-works/pi-ai/api/openai-responses';
import { MEMBER_KEY, } from './stand-in-pi-fixture.ts';

/**
 OpenAI's policy stop, verbatim as two recorded runs on this machine report it:
 `pi-runs/prewarm-of-a-new-worktree-acts-only-on-the-entry-it-checked-1/summary.json`
 and `pi-codemode-abort-crash-troubleshooting-2026-11-18/run-1/summary.json`,
 both under `${HOME}/temp/agent`, field `modelError`.
 */
export const POLICY_STOP_TEXT: string = 'This content was flagged for possible cybersecurity risk. If this seems wrong, '
  + 'try rephrasing your request. If you’re doing authorized security work that requires more cyber permissive '
  + 'safeguards, apply for Daybreak access via https://platform.openai.com/settings/organization/status-and-access '
  + 'before retrying.';

/**
 Serialize provider events as the OpenAI SDK's server-sent-event body reads them.

 @param events - Events the adapter consumes in order

 @returns response body with one `event` and one `data` line per event
 */
function serverSentEvents(events: readonly {
  readonly type: string;
}[],): string {
  return events.map(function toFrame(event,) {
    return `event: ${event.type}\ndata: ${JSON.stringify(event,)}\n\n`;
  },)
    .join('',);
}

/**
 Build OpenAI's policy stop through the installed Responses adapter, over a stream
 that answered HTTP 200 as the recorded runs observed.

 @param model - physical model whose adapter reads the body

 @param context - transcript the adapter is asked about

 @param options - request options the pool forwarded to the member

 @param exposedText - text the member streams before the stop, or empty to stop before any output

 @returns physical stream that fails with the policy stop

 @example
 ```ts
 policyStopStream({ model: PHYSICAL_MODELS[0], context: TRANSCRIPT, exposedText: 'partial' });
 ```
 */
export function policyStopStream({
  model,
  context,
  options,
  exposedText = '',
}: {
  readonly model: Model<Api>;
  readonly context: TranscriptContext;
  readonly options?: SimpleStreamOptions;
  readonly exposedText?: string;
},): AssistantMessageEventStream {
  /**
   Events the SDK reads: a created response, any exposed text, then the stop.
   */
  const events = [
    {
      type: 'response.created',
      response: {
        id: 'resp_policy',
        status: 'in_progress',
      },
    },
    ...(exposedText === '' ? [] : [
      {
        type: 'response.output_item.added',
        output_index: 0,
        item: {
          type: 'message',
          id: 'msg_policy',
          role: 'assistant',
          status: 'in_progress',
          content: [],
        },
      },
      {
        type: 'response.output_text.delta',
        output_index: 0,
        content_index: 0,
        item_id: 'msg_policy',
        delta: exposedText,
      },
    ]),
    {
      type: 'error',
      code: 'invalid_request_error',
      message: POLICY_STOP_TEXT,
    },
  ];
  /**
   One in-memory body; the adapter reaches no network.
   */
  const body = serverSentEvents(events,);
  return streamResponses(
    {
      ...model,
      api: 'openai-responses',
    },
    context,
    {
      ...options,
      apiKey: MEMBER_KEY,
      maxRetries: 0,
      fetch() {
        return Promise.resolve(new Response(
          body,
          {
            status: 200,
            headers: { 'content-type': 'text/event-stream', },
          },
        ),);
      },
    },
  );
}
