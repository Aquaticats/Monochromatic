import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';
import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { HYPER_MESSAGES_URL, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type BudgetView,
  createHyperClient,
  createOpenRouterClient,
  createRoutingClient,
  EveryProviderDryError,
  OPENROUTER_CHAT_URL,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID,
  SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/** Both serving protocols carry the same invented answer. */
const ANSWER = '{"animal":"cat","count":7}';
/** Complete Hyper tool stream, consumed by the actual compiled adapter. */
const HYPER_BODY = `${[
  { type: 'message_start', message: { usage: { input_tokens: 20, output_tokens: 1 } } },
  { type: 'content_block_start', index: 0, content_block: { type: 'tool_use', name: 'model_route', input: {} } },
  { type: 'content_block_delta', index: 0, delta: { type: 'input_json_delta', partial_json: ANSWER } },
  { type: 'content_block_stop', index: 0 },
  { type: 'message_delta', delta: { stop_reason: 'tool_use' }, usage: { output_tokens: 12 } },
  { type: 'message_stop' },
].map(event => `data: ${JSON.stringify(event)}`).join('\n\n')}\n\n`;
/** Complete OpenRouter stream includes reported cost to distinguish it from an estimate. */
const OPENROUTER_BODY = `data: ${JSON.stringify({ choices: [{ delta: { content: ANSWER }, finish_reason: 'stop' }],
  usage: { prompt_tokens: 20, completion_tokens: 12, cost: 0.00001 } })}\n\ndata: [DONE]\n\n`;
const FORMAT = { type: 'json_schema' as const, json_schema: { name: 'model_route', strict: true,
  schema: { type: 'object', additionalProperties: false,
    properties: { animal: { type: 'string' }, count: { type: 'integer' } }, required: ['animal', 'count'] } } };

/** Routes through real clients but disposable transports and budgets. */
function routingFixture(dry: BudgetView) {
  const exchanges: TransportExchange[] = [];
  const hyper = createHyperClient({ apiKey: 'fixture-only', retryPolicy: { limit: 0, baseMs: 1 },
    transport: async exchange => {
      exchanges.push(exchange);
      return { status: 200, bodyText: HYPER_BODY };
    } });
  const openrouter = createOpenRouterClient({ apiKey: 'fixture-only', retryPolicy: { limit: 0, baseMs: 1 },
    transport: async exchange => {
      exchanges.push(exchange);
      return { status: 200, bodyText: OPENROUTER_BODY };
    } });
  const unsupported = { chatText: async () => {
    throw new Error('Unserved provider must not receive this model');
  } };
  const client = createRoutingClient({ callers: { neuralwatt: unsupported, synthetic: unsupported, bedrock: unsupported, hyper, openrouter },
    budgets: { read: async () => dry,
      markRefused: async () => { throw new Error('No transport refusal is scripted'); },
      holds: () => ({ neuralwatt: 0, synthetic: 0, bedrock: 0, deepseek: 0, xiaomi: 0, anthropic: 0, hyper: 0, openrouter: 0 }) } });
  return { client, exchanges };
}

/** Predicate used by the real router's structured-response reader. */
function valid(value: unknown): value is { animal: 'cat'; count: 7 } {
  return ((typeof value) === 'object') && (value !== null) && ('animal' in value) && (value.animal === 'cat')
    && ('count' in value) && (value.count === 7);
}

await describe({
  name: createRoutingClient.name,
  children: [
    ...[
      { name: 'prefers Hyper when both approved routes are wet', hyper: false, openrouter: false,
        url: HYPER_MESSAGES_URL, model: SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID },
      { name: 'uses the OpenRouter spelling when Hyper is dry', hyper: true, openrouter: false,
        url: OPENROUTER_CHAT_URL, model: SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID },
    ].map(row => it({
      name: row.name,
      fn: async () => {
        const { client, exchanges } = routingFixture({ neuralwatt: true, synthetic: false, bedrock: false, deepseek: true, xiaomi: true, anthropic: true,
          hyper: row.hyper, openrouter: row.openrouter });
        const result = await client.chatJson({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: [{ role: 'user', content: 'Return animal cat and count 7.' }], responseFormat: FORMAT,
          signal: new AbortController().signal, validate: valid });
        expect(result.kind).toBe('ok');
        expect(exchanges).toHaveLength(1);
        expect(exchanges[0]?.url).toBe(row.url);
        const body: unknown = JSON.parse(exchanges[0]?.bodyJson ?? '{}');
        expect(body).toMatchObject({ model: row.model, max_tokens: 13_082, stream: true });
        for (const forbidden of ['thinking', 'budget_tokens', 'reasoning_effort'])
          expect(body).not.toHaveProperty(forbidden);
      },
    })),
    ...[
      { name: 'refuses when only unrelated providers are wet', dry: { neuralwatt: true, synthetic: false, bedrock: false, deepseek: true, xiaomi: true, anthropic: true, hyper: true, openrouter: true }, error: NoProviderForModelError },
      { name: 'reports the all-provider-dry boundary separately', dry: { neuralwatt: true, synthetic: true, bedrock: true, deepseek: true, xiaomi: true, anthropic: true, hyper: true, openrouter: true }, error: EveryProviderDryError },
    ].map(row => it({
      name: row.name,
      fn: async () => {
        const { client, exchanges } = routingFixture(row.dry);
        let caught: unknown;
        try {
          await client.chatText({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, messages: [{ role: 'user', content: 'Cat.' }],
            signal: new AbortController().signal });
        } catch (error) { caught = error; }
        expect(caught).toBeInstanceOf(row.error);
        expect(exchanges).toHaveLength(0);
      },
    })),
  ],
});
