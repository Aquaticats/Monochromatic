/**
 Built-artifact tests for requests reaching real Pi AI provider modules.

 Scripted transports bypass provider dispatch,
 so these tests send real HTTP requests to a local capture server
 and inspect the wire body each provider module builds.
 Pi AI 0.87 made providers read prompt and tools only from transcript system messages;
 a raw `Context` then silently sent neither.

 Three dispatch branches cannot reach the wire here,
 because their provider modules load an optional peer package this repository
 does not install (`@google/genai` for google-generative-ai and google-vertex,
 `@aws-sdk/client-bedrock-runtime` for bedrock-converse-stream).

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
import { zstdDecompressSync, } from 'node:zlib';

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

/** API key every provider module accepts as an opaque credential. */
const FIXTURE_API_KEY = 'fixture-key';

/** Account identifier the codex module reads out of its credential. */
const CODEX_ACCOUNT_ID = 'fixture-account';

/** JWT claim path the codex module reads the account identifier from. */
const CODEX_CLAIM_PATH = 'https://api.openai.com/auth';

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

//endregion Fixtures

//region Capture server

/** One captured provider request. */
type CapturedRequest = {
  /** HTTP method provider module sent. */
  readonly method: string;
  /** Request path provider module built. */
  readonly path: string;
  /** Parsed JSON body, absent for a bodiless transport probe. */
  readonly body?: Readonly<Record<string, unknown>>;
};

/** Local server recording each JSON request body it receives. */
type CaptureServer = {
  /** Base URL reaching capture server. */
  readonly origin: string;
  /** Captured requests in arrival order. */
  readonly requests: readonly CapturedRequest[];
  /** Listening server closed after test. */
  readonly server: Server;
};

/**
 Decode one captured request body the way the sending provider encoded it.
 
 @param encoding - content-encoding header the provider module sent
 
 @param raw - complete raw body
 
 @returns decoded body text
 
 @example
 ```ts
 decodeBody({ encoding: 'zstd', raw: chunks });
 ```
 */
function decodeBody(
  {
    encoding,
    raw,
  }: {
    readonly encoding: string;
    readonly raw: Buffer;
  },
): string {
  return encoding === 'zstd'
    ? zstdDecompressSync(raw,).toString('utf8',)
    : raw.toString('utf8',);
}

/**
 Start local server that records decoded JSON bodies and rejects every request.

 @returns listening capture server

 @example
 ```ts
 const capture = await startCaptureServer();
 ```
 */
async function startCaptureServer(): Promise<CaptureServer> {
  /** Captured provider requests in arrival order. */
  const requests: CapturedRequest[] = [];
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
      /** Request path as sent. */
      const path = request.url ?? '';
      /** HTTP method as sent. */
      const method = request.method ?? '';
      /** Complete raw body. */
      const bodyBytes = Buffer.concat(chunks,);
      /** Provider request body text, inflated when the module compressed it. */
      const bodyText = decodeBody({
        encoding: request.headers['content-encoding'] === 'zstd' ? 'zstd' : 'identity',
        raw: bodyBytes,
      },);
      requests.push(bodyBytes.length === 0
        ? { method, path, }
        : {
          method,
          path,
          body: JSON.parse(bodyText,) as Readonly<Record<string, unknown>>,
        },);
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
    requests,
    server,
  };
}

//endregion Capture server

//region Wire cases

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
}): Model<Api> {
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
 Build the account-bearing credential the codex module requires.

 The module reads a ChatGPT account identifier out of its bearer token,
 so the fixture token carries that claim and a placeholder signature.

 @returns JWT-shaped fixture credential

 @example
 ```ts
 forgedCodexToken();
 ```
 */
function forgedCodexToken(): string {
  /**
   Base64url JSON segment.
   */
  const encode = (value: unknown,): string =>
    Buffer.from(JSON.stringify(value,),).toString('base64url',);
  /** Unsecured header the module never verifies. */
  const header = encode({ alg: 'none', typ: 'JWT', },);
  /** Payload carrying the account claim. */
  const payload = encode({ [CODEX_CLAIM_PATH]: { chatgpt_account_id: CODEX_ACCOUNT_ID, }, },);
  return `${header}.${payload}.fixture-signature`;
}

/**
 One provider module's expected wire request.

 @example
 ```ts
 const wireCase: ProviderWireCase = {
   api: 'openai-responses',
   basePath: '/v1',
   expectedToolChoice: 'required',
 };
 ```
 */
type ProviderWireCase = {
  /** Provider API under test. */
  readonly api: Api;
  /** API path prefix the provider module appends its endpoint to. */
  readonly basePath: string;
  /** Credential the provider module accepts. */
  readonly apiKey: string;
  /** Exact `tool_choice` the module must put on the wire. */
  readonly expectedToolChoice: unknown;
  /** Exact request path the module must build, which names the module that ran. */
  readonly expectedPath: string;
};

/** Provider APIs this environment can drive to the wire, with their expected selector. */
const PROVIDER_WIRE_CASES: readonly ProviderWireCase[] = [
  {
    api: 'anthropic-messages',
    basePath: '',
    apiKey: FIXTURE_API_KEY,
    expectedToolChoice: {
      type: 'tool',
      name: TOOL.name,
    },
    expectedPath: '/v1/messages',
  },
  {
    api: 'azure-openai-responses',
    basePath: '/openai',
    apiKey: FIXTURE_API_KEY,
    expectedToolChoice: 'required',
    expectedPath: '/openai/responses?api-version=v1',
  },
  {
    api: 'mistral-conversations',
    basePath: '',
    apiKey: FIXTURE_API_KEY,
    expectedToolChoice: 'any',
    expectedPath: '/v1/chat/completions',
  },
  {
    api: 'openai-codex-responses',
    basePath: '/v1',
    apiKey: forgedCodexToken(),
    expectedToolChoice: 'required',
    expectedPath: '/v1/codex/responses',
  },
  {
    api: 'openai-completions',
    basePath: '/completions',
    apiKey: FIXTURE_API_KEY,
    expectedToolChoice: 'required',
    expectedPath: '/completions/chat/completions',
  },
  {
    api: 'openai-responses',
    basePath: '/v1',
    apiKey: FIXTURE_API_KEY,
    expectedToolChoice: 'required',
    expectedPath: '/v1/responses',
  },
];

/**
 Send one forced-tool review through real provider module and return captured body.

 @param api - provider API under test

 @param basePath - API path prefix provider module appends endpoints to

 @param apiKey - credential the provider module accepts

 @returns single captured request body

 @example
 ```ts
 await captureForcedToolRequest({
   api: 'openai-responses',
   basePath: '/v1',
   apiKey: 'fixture-key',
 });
 ```
 */
async function captureForcedToolRequest({
  api,
  basePath,
  apiKey,
}: {
  readonly api: Api;
  readonly basePath: string;
  readonly apiKey: string;
}): Promise<CapturedRequest & { readonly body: Readonly<Record<string, unknown>>; }> {
  /** Local capture server for this request. */
  const capture = await startCaptureServer();
  try {
    await runStructuredToolRequest({
      model: captureModel({
        api,
        baseUrl: `${capture.origin}${basePath}`,
      },),
      auth: { apiKey, },
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
    // The capture server rejects every request on purpose, so a provider that
    // surfaces that rejection is expected; the recorded request is the assertion.
    expect(error,).toBeInstanceOf(Error,);
  }
  capture.server.close();
  /** Captured requests that carried a provider body. */
  const providerRequests = capture.requests.filter(function hasBody(request,): boolean {
    return request.body !== undefined;
  },);
  expect(providerRequests,).toHaveLength(1,);
  /** Single captured provider request. */
  const [providerRequest,] = providerRequests;
  if (providerRequest?.body === undefined)
    throw new Error('Expected one captured provider request body.',);
  return { ...providerRequest, body: providerRequest.body, };
}

//endregion Wire cases

await describe({
  name: runStructuredToolRequest.name,
  children: PROVIDER_WIRE_CASES.map(function wireCase(providerCase,) {
    return it({
      name: `sends system prompt, verdict tool, and forced selector through ${providerCase.api}`,
      fn: async () => {
        /** Captured provider request. */
        const { path, body, } = await captureForcedToolRequest({
          api: providerCase.api,
          basePath: providerCase.basePath,
          apiKey: providerCase.apiKey,
        },);
        expect(path,).toBe(providerCase.expectedPath,);
        expect(JSON.stringify(body,),).toContain(SYSTEM_PROMPT_MARKER,);
        expect(JSON.stringify(body.tools ?? [],),).toContain(TOOL.name,);
        expect(body.tool_choice,).toEqual(providerCase.expectedToolChoice,);
      },
    },);
  },),
},);
