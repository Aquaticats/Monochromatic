/**
 Native HTTP header precedence, canonical history, and tool continuation.
 
 @module
 */
import { Type, } from '@earendil-works/pi-ai';
import type { AgentSession, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { CODEX_API, CODEX_PROVIDER, FAST_PROVIDER, registerOpenAIFast, } from '../dist/final/node/index.mjs';
import { fixtureHttp, nativeResponse, } from './host-fixture-http.ts';
import { fixtureContext, } from './host-fixture-model.ts';
import { HOST_TOKEN, fixtureProvider, } from './host-fixture-provider.ts';
import { fixtureHome, fixtureHost, requireCompanion, } from './host-fixture-session.ts';
import { fixtureRuntime, } from './host-fixture-runtime.ts';

//region Native HTTP: original headers and history survive live registry dispatch.

await describe({ name: registerOpenAIFast.name, children: [
  it({ name: 'positive control recomposes configured provider wrappers without the fast extension', fn: async function verifyNativeWrapperIdentity() {
    /** Control has disposable configuration and no fast registrations. */
    await using home = await fixtureHome();
    /** Registered native identity remains stable across host-owned composition. */
    const source = fixtureProvider();
    /** Native runtime alone applies provider/model configuration. */
    const { runtime, effective, } = await fixtureRuntime({ home, source, expired: false, configured: true, },);
    await runtime.refresh({ allowNetwork: false, },);
    expect(runtime.getProvider(CODEX_PROVIDER,),).not.toBe(effective,);
    expect(runtime.getRegisteredNativeProvider(CODEX_PROVIDER,),).toBe(source.provider,);
    expect(runtime.getRegisteredNativeProvider(FAST_PROVIDER,),).toBeUndefined();
  }, },),
  it({ name: 'leaves normal HTTP unchanged and requests priority with original ID, high/xhigh, and configured headers', fn: async function verifyNativeHttp() {
    /** Native requests use only disposable model configuration and sessions. */
    await using home = await fixtureHome();
    /** Finite HTTP budget exposes any unexpected additional or fallback dispatch. */
    const http = fixtureHttp({ responses: [nativeResponse, nativeResponse, nativeResponse, nativeResponse,], },);
    /** Actual native runtime applies configured provider and model headers. */
    using host = await fixtureHost({ home, source: http.source, configured: true, },);
    /** Settings control detects unrelated default or selector-policy mutation. */
    const before = host.settings.getSettings();
    await host.session.prompt('Ordinary native request.',);
    /** User-visible entry must be genuinely virtual with configured capabilities. */
    const companion = requireCompanion({ runtime: host.runtime, id: host.base.id, },);
    expect(companion,).toMatchObject({ api: 'pi-virtual', name: 'Configured fixture Fast', contextWindow: 333_333, maxTokens: 16_384, },);
    await host.session.setModel(companion,);
    host.session.setThinkingLevel('high',);
    await host.session.prompt('Fast native request with high thinking.',);
    host.session.setThinkingLevel('xhigh',);
    await host.session.prompt('Fast native request with xhigh thinking.',);
    expect(host.session.messages.filter(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
      return message.role === 'assistant';
    },).map(function nativeError(message) {
      return message.errorMessage;
    },),).toEqual([undefined, undefined, undefined,],);
    expect(http.requests,).toHaveLength(3,);
    expect(http.requests[0]?.payload,).not.toHaveProperty('service_tier',);
    expect(http.requests[0]?.payload,).toHaveProperty('model', host.base.id,);
    expect(http.requests[0]?.headers.get('x-shared',),).toBe('model-value',);
    for (const [index, effort,] of [[1, 'high',], [2, 'xhigh',],] as const) {
      /** Final native HTTP input includes original OAuth and configuration merging. */
      const request = http.requests[index];
      expect(request?.payload,).toMatchObject({ model: host.base.id, service_tier: 'priority', },);
      expect(request?.payload,).toHaveProperty('reasoning.effort', effort,);
      expect(request?.headers.get('authorization',),).toBe(`Bearer ${HOST_TOKEN}`,);
      expect(request?.headers.get('x-fixture-oauth',),).toBe('resolved',);
      expect(request?.headers.get('x-config-provider',),).toBe('configured-provider',);
      expect(request?.headers.get('x-config-model',),).toBe('original-model-only',);
      expect(request?.headers.get('X-SHARED',),).toBe('model-value',);
      expect(http.source.state.calls[index]?.model,).toMatchObject({ id: host.base.id, contextWindow: 333_333, maxTokens: 16_384, },);
    }
    /** Actual host history must record original native identity for every reply. */
    const assistants = host.session.messages.filter(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
      return message.role === 'assistant';
    },);
    expect(assistants,).toHaveLength(3,);
    for (const assistant of assistants)
      expect(assistant,).toMatchObject({ model: host.base.id, provider: CODEX_PROVIDER, api: CODEX_API, stopReason: 'stop', },);
    expect(assistants.map(function thinkingLevel(message) {
      return message.thinkingLevel;
    },),).toEqual(['high', 'high', 'xhigh',],);
    expect(host.session.getLastAssistantText(),).toBe('Native fixture answer.',);
    expect(host.session.model,).toMatchObject({ api: 'pi-virtual', provider: FAST_PROVIDER, id: host.base.id, },);
    expect(host.settings.getSettings(),).toEqual(before,);
    expect(host.ctx.scopedModels,).toHaveLength(1,);
    /** Explicit caller headers must override the original configured model headers. */
    const explicit = await host.runtime.completeSimple(companion, fixtureContext(),
      { reasoning: 'high', headers: { 'x-SHARED': 'explicit-caller-value', }, },);
    expect(explicit.stopReason,).toBe('stop',);
    expect(http.requests,).toHaveLength(4,);
    expect(http.requests[3]?.headers.get('X-Shared',),).toBe('explicit-caller-value',);
    expect(http.requests[3]?.headers.get('x-config-model',),).toBe('original-model-only',);
    expect(http.requests[3]?.payload,).toMatchObject({ model: host.base.id, service_tier: 'priority', },);
    expect(host.runtime.getRegisteredNativeProvider(CODEX_PROVIDER,),).toBe(http.source.provider,);
    expect(host.runtime.getProvider(CODEX_PROVIDER,),).toMatchObject({ id: CODEX_PROVIDER, name: host.original.name, baseUrl: host.original.baseUrl, },);
  }, },),
  it({ name: 'routes tool followups through the same virtual selection and priority transport', fn: async function verifyToolContinuation() {
    /** Tool registration and prompts use only independently disposable host storage. */
    await using home = await fixtureHome();
    /** Native parser receives a real tool-use packet followed by a text completion. */
    const http = fixtureHttp({ responses: [function toolResponse() {
      return nativeResponse({ tool: true, },);
    }, nativeResponse,], },);
    /** Actual host performs the tool loop through the registered virtual selection. */
    using host = await fixtureHost({ home, source: http.source, },);
    /** Execution count belongs only to this synthetic no-side-effect tool. */
    const toolState = { executed: 0, };
    host.pi.registerTool({ name: 'fixture_tool', label: 'Fixture tool', description: 'Return a synthetic value without side effects.',
      parameters: Type.Object({},), execute: function execute() {
        toolState.executed += 1;
        return Promise.resolve({ content: [{ type: 'text' as const, text: 'Synthetic tool output.', },], details: {}, },);
      }, },);
    host.pi.setActiveTools(['fixture_tool',],);
    await host.session.setModel(requireCompanion({ runtime: host.runtime, id: host.base.id, },),);
    await host.session.prompt('Use the synthetic tool.',);
    expect(host.session.messages.filter(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
      return message.role === 'assistant';
    },).map(function nativeError(message) {
      return message.errorMessage;
    },),).toEqual([undefined, undefined,],);
    expect(toolState.executed,).toBe(1,);
    expect(http.requests,).toHaveLength(2,);
    for (const request of http.requests)
      expect(request.payload,).toMatchObject({ model: host.base.id, service_tier: 'priority', },);
    expect(http.requests[1]?.payload,).toHaveProperty('input',);
    expect(http.requests[1]?.payload,).toSatisfy(function includesNativeToolOutput(value) {
      return ((typeof value) === 'object') && (value !== null) && ('input' in value)
        && Array.isArray(value.input,) && value.input.some(function toolOutput(item: unknown) {
          return ((typeof item) === 'object') && (item !== null) && ('type' in item) && ('output' in item)
            && (item.type === 'function_call_output') && (item.output === 'Synthetic tool output.');
        },);
    },);
    expect(host.session.messages.filter(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
      return message.role === 'assistant';
    },).map(function stopReason(message) {
      return message.stopReason;
    },),).toEqual(['toolUse', 'stop',],);
    expect(host.session.model?.api,).toBe('pi-virtual',);
    expect(host.session.getLastAssistantText(),).toBe('Native fixture answer.',);
  }, },),
], },);

//endregion Native HTTP
