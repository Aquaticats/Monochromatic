/**
 Native host failure, overflow, cancellation, and no-fallback controls.
 
 @module
 */
import type { AgentSession, SessionBeforeCompactEvent, ProviderStreamEvent, } from '@earendil-works/pi-coding-agent';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { CODEX_API, CODEX_PROVIDER, registerOpenAIFast, } from '../dist/final/node/index.mjs';
import { verifyHostAbort, } from './host-fixture-abort.ts';
import { fixtureHttp, nativeFailureResponse, } from './host-fixture-http.ts';
import { fixtureAssistant, } from './host-fixture-model.ts';
import { fixtureProvider, } from './host-fixture-provider.ts';
import { fixtureHome, fixtureHost, requireCompanion, } from './host-fixture-session.ts';

//region Terminal failures: native failure or abort never retries under ordinary tier.

await describe({ name: registerOpenAIFast.name, children: [
  ...['upstream_fixture_error', 'context_length_exceeded',].map(function nativeFailureScenario(code) {
    return it({ name: `surfaces native HTTP ${code} without fallback`, fn: async function verifyNativeFailure() {
      /** Native failure handling runs entirely inside disposable host state. */
      await using home = await fixtureHome();
      /** Single failure packet budget detects every extra or fallback request. */
      const http = fixtureHttp({ responses: [function failureResponse() {
        return nativeFailureResponse({ code, message: 'Synthetic native failure.', },);
      },], },);
      /** Real SDK parser, history, and virtual routing handle the native failure. */
      using host = await fixtureHost({ home, source: http.source, },);
      /** Raw event observation preserves provider error codes separately from native message formatting. */
      const failures: unknown[] = [];
      host.pi.on('provider_stream_event', function observeFailure(event: ForeignBorrowed<ProviderStreamEvent>) {
        if (((typeof event.data) === 'object') && (event.data !== null) && ('type' in event.data)
          && (event.data.type === 'response.failed'))
          failures.push(event.data,);
      },);
      await host.session.setModel(requireCompanion({ runtime: host.runtime, id: host.base.id, },),);
      await host.session.prompt('Trigger native failure packet.',);
      /** Actual canonical history entry retains native error and original identity. */
      const assistant = host.session.messages.findLast(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
        return message.role === 'assistant';
      },);
      expect(assistant,).toMatchObject({ stopReason: 'error', model: host.base.id, provider: CODEX_PROVIDER, api: CODEX_API, },);
      expect(assistant,).toHaveProperty('errorMessage',);
      expect(assistant?.errorMessage,).toBe('Synthetic native failure.',);
      expect(failures,).toHaveLength(1,);
      expect(failures[0],).toHaveProperty('type', 'response.failed',);
      expect(failures[0],).toHaveProperty('response.error.code', code,);
      expect(failures[0],).toHaveProperty('response.error.message', 'Synthetic native failure.',);
      expect(http.requests,).toHaveLength(1,);
      expect(http.requests[0]?.payload,).toHaveProperty('service_tier', 'priority',);
    }, },);
  },),
  ...[false, true,].map(function overflowScenario(fast) {
    return it({ name: fast
      ? 'recognizes original-response overflow under a virtual fast selection with a different target provider'
      : 'positive control recognizes native ordinary-response overflow', fn: async function verifyNativeOverflow() {
        /** Ordinary and virtual controls both use independently disposable state. */
        await using home = await fixtureHome();
        /** Synthetic native response makes original context overflow observable. */
        const source = fixtureProvider();
        source.state.reply = function overflowReply(call) {
          return fixtureAssistant({ model: call.model, overrides: { usage: {
            input: call.model.contextWindow + 100, output: 2, cacheRead: 0, cacheWrite: 0,
            totalTokens: call.model.contextWindow + 102,
            cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0, }, }, }, },);
        };
        /** Actual host compaction logic consumes canonical native history. */
        using host = await fixtureHost({ home, source, },);
        /** Native compaction reasons distinguish overflow from unrelated triggers. */
        const reasons: string[] = [];
        host.pi.on('session_before_compact', function beforeCompact(event: ForeignBorrowed<SessionBeforeCompactEvent>) {
          reasons.push(event.reason,);
          return { cancel: true, };
        },);
        host.settings.applyOverrides({ compaction: { enabled: true, reserveTokens: 256, keepRecentTokens: 0, }, },);
        if (fast)
          await host.session.setModel(requireCompanion({ runtime: host.runtime, id: host.base.id, },),);
        await host.session.prompt('Synthetic usage exceeds original native context.',);
        expect(reasons,).toEqual(['overflow',],);
        expect(source.state.calls,).toHaveLength(1,);
        expect(host.session.messages.findLast(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
          return message.role === 'assistant';
        },),).toMatchObject({ model: host.base.id, provider: CODEX_PROVIDER, },);
      }, },);
  },),
  ...['upstream failure', 'maximum context length exceeded',].map(function providerFailureScenario(message) {
    return it({ name: `surfaces ${message} without ordinary fallback or extra dispatch`, fn: async function verifyProviderFailure() {
      /** Provider failure prompts and history use only disposable native host state. */
      await using home = await fixtureHome();
      /** Distinct native failure response must be surfaced without substitution. */
      const source = fixtureProvider();
      source.state.reply = function failureReply(call) {
        return fixtureAssistant({ model: call.model, overrides: { stopReason: 'error', errorMessage: message, }, },);
      };
      /** Actual host retains virtual selection after canonical original-provider failure. */
      using host = await fixtureHost({ home, source, },);
      await host.session.setModel(requireCompanion({ runtime: host.runtime, id: host.base.id, },),);
      await host.session.prompt('Trigger synthetic provider failure.',);
      /** Native history entry retains exact error text and original transport identity. */
      const assistant = host.session.messages.findLast(function assistantMessage(entry: ForeignBorrowed<AgentSession['messages'][number]>) {
        return entry.role === 'assistant';
      },);
      expect(assistant,).toMatchObject({ stopReason: 'error', errorMessage: message, model: host.base.id, provider: CODEX_PROVIDER, api: CODEX_API, },);
      expect(source.state.calls,).toHaveLength(1,);
      expect(source.state.calls[0]?.options,).toHaveProperty('serviceTier', 'priority',);
      expect(host.session.model?.api,).toBe('pi-virtual',);
    }, },);
  },),
  it({ name: 'surfaces actual host cancellation without fallback and retains the virtual selection', fn: verifyHostAbort, timeout: 15_000, },),
], },);

//endregion Terminal failures
