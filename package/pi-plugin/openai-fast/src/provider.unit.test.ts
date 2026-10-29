/**
 Internal routing identity and live-original lookup boundaries.
 
 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  FastModelError,
  FAST_PROVIDER,
  isPriorityTarget,
  PRIORITY_TARGET_PREFIX,
  priorityTarget,
  resolvePriorityBase,
} from '../dist/final/node/index.mjs';
import { fixtureModel, } from './host-fixture-model.ts';

//region Identity: targets clone capabilities, never original request-auth headers.

await describe({ name: '', children: [
  describe({ name: priorityTarget.name, children: [
    it({ name: 'clones capabilities into the fast namespace without inheriting headers or mutating the original', fn: async function verifyTargetClone() {
      /** Original native metadata retains its auth headers and input limits. */
      const base = fixtureModel({ headers: { 'x-model': 'fixture', }, inputLimits: { maxRequestBytes: 123_456, }, },);
      /** Internal target must be distinct while preserving non-auth capabilities. */
      const target = priorityTarget(base,);
      expect(target,).not.toBe(base,);
      expect(target,).toMatchObject({ provider: FAST_PROVIDER, id: `${PRIORITY_TARGET_PREFIX}${base.id}`,
        api: base.api, input: base.input, contextWindow: base.contextWindow, maxTokens: base.maxTokens, inputLimits: base.inputLimits, },);
      expect(target,).not.toHaveProperty('headers',);
      expect(target.cost,).toBe(base.cost,);
      expect(base.id,).toBe('gpt-host-fixture',);
      expect(base.headers,).toEqual({ 'x-model': 'fixture', },);
      expect(isPriorityTarget(target,),).toBe(true,);
      expect(isPriorityTarget(base,),).toBe(false,);
      expect(isPriorityTarget({ id: `not-${PRIORITY_TARGET_PREFIX}model`, },),).toBe(false,);
      // The external test harness requires asynchronous completion of synchronous assertions.
      await Promise.resolve();
    }, },),
  ], },),
  describe({ name: resolvePriorityBase.name, children: [
    it({ name: 'rejects an ordinary input before consulting the original-model lookup', fn: async function verifyOrdinaryRejection() {
      /** Invalid ordinary IDs must not invoke the supplied upstream lookup. */
      const lookupState = { lookedUp: false, };
      expect(function resolveOrdinaryInput() {
        return resolvePriorityBase({ model: fixtureModel(), lookup: function lookup() {
          lookupState.lookedUp = true;
          return fixtureModel();
        }, },);
      },).toThrow(FastModelError,);
      expect(lookupState.lookedUp,).toBe(false,);
      await Promise.resolve();
    }, },),
    it({ name: 'returns the current original rather than stale cloned target metadata', fn: async function verifyLiveOriginal() {
      /** Target is built before the original model metadata changes. */
      const target = priorityTarget(fixtureModel(),);
      /** Current native model must replace stale capabilities at dispatch time. */
      const current = fixtureModel({ name: 'Refreshed model', contextWindow: 456_789, },);
      /** Captured IDs prove the internal prefix is stripped exactly once. */
      const ids: string[] = [];
      expect(resolvePriorityBase({ model: target, lookup: function lookup(id) {
        ids.push(id,);
        return current;
      }, },),).toBe(current,);
      expect(ids,).toEqual([current.id,],);
      await Promise.resolve();
    }, },),
    ...['missing', 'unsupported',].map(function originalFailureScenario(reason) {
      return it({ name: `rejects ${reason} original with actionable model error`, fn: async function verifyOriginalFailure() {
        /** Original identity must appear in the diagnostic without a substitute. */
        const base = fixtureModel();
        /** Supply actual absence or a differently configured native API. */
        function lookup() {
          return reason === 'missing' ? undefined : { ...base, api: 'openai-responses' as const, };
        }
        expect(function resolveFailure() {
          return resolvePriorityBase({ model: priorityTarget(base,), lookup, },);
        },).toThrow(FastModelError,);
        expect(function resolveFailureDiagnostic() {
          return resolvePriorityBase({ model: priorityTarget(base,), lookup, },);
        },).toThrow(base.id,);
        await Promise.resolve();
      }, },);
    },),
  ], },),
], },);

//endregion Identity
