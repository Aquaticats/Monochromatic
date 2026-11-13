/**
 The loopback endpoint the passing-token-rate-limit proof runs pi against.

 It answers the catalog listing the extension asks for at session start,
 and answers chat completions in the order the proof needs:
 the first one of each run is the recorded `429` answer,
 and every later one is a reply.

 No request leaves the machine:
 the proof's throwaway `models.json` points the route at this endpoint.

 @module
 */

import {
  createServer,
  type IncomingMessage,
  type Server,
  type ServerResponse,
} from 'node:http';
import { once, } from 'node:events';

//region Constants

/**
 Status a chat completion the endpoint serves as a reply is answered with.
 */
const HTTP_OK = 200;

/**
 Status the recorded passing token-rate limit arrived under.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 The body fifteen review runs ended on 2026-11-13,
 with its request id replaced by a placeholder.
 */
export const RECORDED_PASSING_LIMIT_BODY: Readonly<Record<string, unknown>> = {
  message:
    'Allocated quota exceeded, please increase your quota limit. For details, see: https://www.alibabacloud.com/help/en/model-studio/error-code#token-limit',
  id: '00000000-0000-4000-8000-000000000000',
  type: 'insufficient_quota',
  code: 'insufficient_quota',
};

//endregion Constants

//region Types

/**
 One endpoint the proof talks to, and the counts it keeps.
 */
export type StandIn = {
  /**
   Port the endpoint listens on, which the throwaway `models.json` names.
   */
  readonly port: number;
  /**
   Server object the proof closes when it is done.
   */
  readonly server: Server;
  /**
   How many chat completions the endpoint has answered over the whole proof.
   
   @returns the count at the moment it is read
   */
  readonly chatRequests: () => number;
  /**
   Start counting a new run's chat completions,
   so each run meets the recorded answer first.
   */
  readonly beginRun: () => void;
};

/**
 What the endpoint answers, which the proof states once.
 */
type StandInAnswers = {
  /**
   Model id the listing serves.
   */
  readonly model: string;
  /**
   Creation time the listing reports for that model.
   */
  readonly listingCreatedSeconds: number;
  /**
   Reply text every chat completion after the run's first carries.
   */
  readonly replyText: string;
};

/**
 The counts the endpoint keeps while it answers,
 held in one object so the request handler and the proof share them.
 */
type StandInCounts = {
  /**
   Chat completions answered over the whole proof.
   */
  total: number;
  /**
   Chat completions answered since the last `beginRun`.
   */
  thisRun: number;
};

//endregion Types

//region Answers

/**
 Build the server-sent events one reply consists of.

 @param replyText - text the reply carries

 @returns the event body the endpoint writes

 @example
 ```typescript
 const body = replyEventsOf({ replyText: 'pong' });
 ```
 */
function replyEventsOf({
  replyText,
}: {
  readonly replyText: string;
},): string {
  /**
   Chunk lines the answer carries.
   */
  const chunkLines = [
    JSON.stringify({
      id: 'stand-in',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta: {
          role: 'assistant',
          content: replyText,
        },
        finish_reason: null,
      },],
    },),
    JSON.stringify({
      id: 'stand-in',
      object: 'chat.completion.chunk',
      choices: [{
        index: 0,
        delta: {},
        finish_reason: 'stop',
      },],
    },),
  ].map(function toChunkLine(payload,) {
    return `data: ${payload}\n\n`;
  },);
  return `${chunkLines.join('',)}data: [DONE]\n\n`;
}

//endregion Answers

//region The endpoint

/**
 Answer one request the endpoint receives:
 a listing for a read, and a chat completion for anything else,
 with the run's first completion carrying the recorded answer.

 @param request - request pi sent

 @param response - response the endpoint writes

 @param answers - what this endpoint serves

 @param counts - counts the endpoint keeps

 @example
 ```typescript
 createServer(function answer(request, response) {
   answerStandInRequest({ request, response, answers, counts });
 });
 ```
 */
function answerStandInRequest({
  request,
  response,
  answers,
  counts,
}: {
  readonly request: IncomingMessage;
  readonly response: ServerResponse;
  readonly answers: StandInAnswers;
  readonly counts: StandInCounts;
},): void {
  if (request.method === 'GET') {
    response.writeHead(
      HTTP_OK,
      { 'content-type': 'application/json', },
    );
    response.end(
      JSON.stringify({
        data: [{
          id: answers.model,
          created: answers.listingCreatedSeconds,
        },],
      },),
    );
    return;
  }
  counts.total += 1;
  counts.thisRun += 1;
  if (counts.thisRun === 1) {
    response.writeHead(
      HTTP_TOO_MANY_REQUESTS,
      { 'content-type': 'application/json', },
    );
    response.end(JSON.stringify(RECORDED_PASSING_LIMIT_BODY,),);
    return;
  }
  response.writeHead(
    HTTP_OK,
    { 'content-type': 'text/event-stream', },
  );
  response.end(replyEventsOf({ replyText: answers.replyText, },),);
}

/**
 Start the endpoint on a port the operating system chooses.

 @param answers - what this endpoint serves

 @returns the listening endpoint

 @example
 ```typescript
 const standIn = await startStandIn({ answers });
 ```
 */
export async function startStandIn({
  answers,
}: {
  readonly answers: StandInAnswers;
},): Promise<StandIn> {
  /**
   Counts the request handler and the proof share.
   */
  const counts: StandInCounts = {
    total: 0,
    thisRun: 0,
  };
  /**
   Endpoint itself.
   */
  const server = createServer(function answer(
    request,
    response,
  ) {
    answerStandInRequest({
      request,
      response,
      answers,
      counts,
    },);
  },);
  server.listen(
    0,
    '127.0.0.1',
  );
  await once(
    server,
    'listening',
  );
  /**
   Address the endpoint was given, which a listening server reports as a
   socket address.
   */
  const address = server.address();
  if ((address === null) || ((typeof address) === 'string'))
    throw new Error('the stand-in endpoint did not report the loopback address it listens on',);
  return {
    port: address.port,
    server,
    chatRequests: function chatRequests(): number {
      return counts.total;
    },
    beginRun: function beginRun(): void {
      counts.thisRun = 0;
    },
  };
}

//endregion The endpoint
