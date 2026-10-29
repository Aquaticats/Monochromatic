/**
 Built-artifact tests for requests reaching real Pi AI provider modules.

 Scripted transports bypass provider dispatch,
 so these tests send real HTTP requests to a local capture server
 and inspect the wire body each provider module builds.
 Pi AI 0.87 made providers read prompt and tools only from transcript system messages;
 a raw `Context` then silently sent neither.

 @module
 */

import type {
  Api,
  Model,
  Tool,
} from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { once, } from 'node:events';
import {
  createServer,
  type IncomingMessage,
  type Server,
  type ServerResponse,
} from 'node:http';
import type { AddressInfo, } from 'node:net';

import { runStructuredToolRequest, } from '../dist/final/node/index.mjs';

//region Fixtures

/** Fixture reviewer context window. */
const CONTEXT_WINDOW = 128_000;

/** Fixture reviewer output capacity. */
const MAX_TOKENS = 16_384;

/** Complete request timeout for local capture round trips. */
const TEST_TIMEOUT_MS = 10_000;

/** HTTP status that makes providers stop after one captured request. */
const CAPTURE_REJECT_STATUS = 400;

/** Unique system-prompt text searched for in captured request bodies. */
const SYSTEM_PROMPT_MARKER = 'fixture reviewer system prompt marker';

/** Fixture structured verdict tool. */
const TOOL = {
  name: 'submit_fixture_review',
  description: 'Submit fixture review.',
  parameters: {
    type: 'object',
    properties: {
      approved: { type: 'boolean', },
    },
    required: ['approved',],
  },
} satisfies Tool;

/** Local server recording each JSON request body it receives. */
type CaptureServer = {
  /** Base URL reaching capture server. */
  readonly origin: string;
  /** Parsed request bodies in arrival order. */
  readonly bodies: readonly Readonly<Record<string, unknown>>[];
  /** Listening server closed after test. */
  readonly server: Server;
};

/**
 Start local server that records JSON bodies and rejects every request.

 @returns listening capture server

 @example
 ```ts
 const capture = await startCaptureServer();
 ```
 */
async function startCaptureServer(): Promise<CaptureServer> {
  /** Parsed request bodies in arrival order. */
  const bodies: Readonly<Record<string, unknown>>[] = [];
  /** Server rejecting each request after recording its body. */
  const server = createServer(function recordRequest(
    request: IncomingMessage,
    response: ServerResponse,
  ): void {
    /** Raw request body chunks. */
    const chunks: Buffer[] = [];
    request.on('data', function collectChunk(chunk: Buffer,): void {
      chunks.push(chunk,);
    },);
    request.on('end', function recordBody(): void {
      /** Complete request body text. */
      const bodyText = Buffer
        .concat(chunks,)
        .toString('utf8',);
      bodies.push(JSON.parse(bodyText,) as Record<string, unknown>,);
      response.writeHead(CAPTURE_REJECT_STATUS, { 'content-type': 'application/json', },);
      response.end('{"type":"error","error":{"type":"invalid_request_error","message":"capture only"}}',);
    },);
  },);
  server.listen(0, '127.0.0.1',);
  await once(server, 'listening',);
  /** Ephemeral port chosen by operating system. */
  const { port, } = server.address() as AddressInfo;
  return {
    origin: `http://127.0.0.1:${String(port,)}`,
    bodies,
    server,
  };
}

/**
 Build fixture model for one provider API against capture server.

 @param api - provider API under test

 @param baseUrl - capture server URL provider module calls

 @returns fixture model

 @example
 ```ts
 captureModel({ api: 'anthropic-messages', baseUrl: 'http://127.0.0.1:1' });
 ```
 */
function captureModel({
  api,
  baseUrl,
}: {
  readonly api: Api;
  readonly baseUrl: string;
},): Model<Api> {
  return {
    id: 'reviewer',
    name: 'Reviewer',
    api,
    provider: 'test',
    baseUrl,
    reasoning: false,
    input: ['text',],
    cost: {
      input: 1,
      output: 1,
      cacheRead: 0,
      cacheWrite: 0,
    },
    contextWindow: CONTEXT_WINDOW,
    maxTokens: MAX_TOKENS,
  };
}

/**
 Send one forced-tool review through real provider module and return captured body.

 @param api - provider API under test

 @param basePath - API path prefix provider module appends endpoints to

 @returns single captured request body

 @example
 ```ts
 await captureForcedToolRequest({ api: 'openai-responses', basePath: '/v1' });
 ```
 */
async function captureForcedToolRequest({
  api,
  basePath,
}: {
  readonly api: Api;
  readonly basePath: string;
},): Promise<Readonly<Record<string, unknown>>> {
  /** Local capture server for this request. */
  const capture = await startCaptureServer();
  try {
    await runStructuredToolRequest({
      model: captureModel({
        api,
        baseUrl: `${capture.origin}${basePath}`,
      },),
      auth: { apiKey: 'fixture-key', },
      prompt: {
        systemPrompt: SYSTEM_PROMPT_MARKER,
        userContent: 'Review fixture action.',
      },
      tool: TOOL,
      toolName: TOOL.name,
      signal: AbortSignal.timeout(TEST_TIMEOUT_MS,),
    },);
  }
  catch (error) {
    // Capture server rejects every request; only recorded body matters.
    expect(error,).toBeDefined();
  }
  capture.server.close();
  expect(capture.bodies.length,).toBe(1,);
  return capture.bodies[0] ?? {};
}

//endregion Fixtures

await describe({
  name: runStructuredToolRequest.name,
  children: [
    it({
      name: 'sends system prompt, verdict tool, and forced selector through anthropic-messages',
      fn: async () => {
        /** Captured Anthropic Messages request body. */
        const body = await captureForcedToolRequest({
          api: 'anthropic-messages',
          basePath: '',
        },);
        expect(JSON.stringify(body.system ?? null,),).toContain(SYSTEM_PROMPT_MARKER,);
        expect(JSON.stringify(body.tools ?? [],),).toContain(TOOL.name,);
        expect(body.tool_choice,).toEqual({
          type: 'tool',
          name: TOOL.name,
        },);
      },
    },),
    it({
      name: 'sends system prompt, verdict tool, and forced selector through openai-responses',
      fn: async () => {
        /** Captured OpenAI Responses request body. */
        const body = await captureForcedToolRequest({
          api: 'openai-responses',
          basePath: '/v1',
        },);
        expect(JSON.stringify(body,),).toContain(SYSTEM_PROMPT_MARKER,);
        expect(JSON.stringify(body.tools ?? [],),).toContain(TOOL.name,);
        expect(body.tool_choice,).toBe('required',);
      },
    },),
  ],
},);
