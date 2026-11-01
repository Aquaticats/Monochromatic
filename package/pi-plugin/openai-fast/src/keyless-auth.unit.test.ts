/**
 Source-dependent adapter readiness without credential resolution or another login.

 @module
 */
import type { AuthContext, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { createKeylessAuth, } from '../dist/final/node/index.mjs';
import { caughtFailure, } from './host-fixture-model.ts';

//region Fresh availability decisions preserve native cancellation and failures.

await describe({ name: '', children: [
  describe({ name: createKeylessAuth.name, children: [
    ...[false, true,].map(function readinessCase(configured,) {
      return it({ name: `declares readiness only when source availability is ${String(configured,)}`, fn: async function verifyReadiness(ctx) {
        /** Source callback receives cancellation but never adapter or original credentials. */
        const source = ctx.sinon.spy(async function sourceReady(signal: ForeignBorrowed<AbortSignal>) {
          signal.throwIfAborted();
          return await Promise.resolve(configured,);
        },);
        /** Built descriptor must not offer a separate authentication flow. */
        const auth = createKeylessAuth(source,);
        /** Unused context rejects any attempt to read environment values or files here. */
        const context = {
          env: function env(): ReturnType<AuthContext['env']> { throw new Error('Adapter must not resolve environment credentials.',); },
          fileExists: function fileExists(): Promise<boolean> { throw new Error('Adapter must not inspect credential files.',); },
        };
        /** Native availability supplies its own cancellation authority. */
        const {signal} = new AbortController();
        /** Stored adapter credentials must not be copied into original-provider requests. */
        const input = { ctx: context, signal, credential: { type: 'api_key' as const, key: 'unused-adapter-fixture', }, };
        expect(await auth.apiKey?.check?.(input,),).toEqual(configured
          ? { type: 'api_key', source: 'routes-to-original-provider', } : undefined,);
        expect(source,).toHaveBeenCalledExactlyOnceWith(signal,);
        expect(await auth.apiKey?.resolve(input,),).toEqual({ auth: {}, source: 'routes-to-original-provider', },);
        expect(source,).toHaveBeenCalledTimes(1,);
        expect(auth.apiKey,).not.toHaveProperty('login',);
        expect(auth.oauth,).toBeUndefined();
      }, },);
    },),
    ...['cancelled', 'failed',].map(function failureCase(reason,) {
      return it({ name: `preserves native ${reason} availability without inventing readiness`, fn: async function verifyFailure(ctx) {
        /** Exact error identity distinguishes cancellation and source failure from false readiness. */
        const failure = new Error(`Synthetic availability ${reason}.`,);
        /** Cancellation must prevent source work; a source failure must propagate unchanged. */
        const source = ctx.sinon.spy(function sourceReady(): Promise<boolean> {
          throw failure;
        },);
        /** Descriptor delegates readiness rather than swallowing source errors. */
        const auth = createKeylessAuth(source,);
        /** Cancellation is owned by this test and cannot affect another attempt. */
        const controller = new AbortController();
        if (reason === 'cancelled')
          controller.abort(failure,);
        /** Native auth context is unused by the adapter. */
        const context = {
          env: function env(): ReturnType<AuthContext['env']> { return Promise.resolve(undefined,); },
          fileExists: function fileExists(): Promise<boolean> { return Promise.resolve(false,); },
        };
        /** The native callback must reject instead of returning a misleading configured state. */
        const caught = await caughtFailure(async function checkReadiness() {
          return await auth.apiKey?.check?.({ ctx: context, signal: controller.signal, },);
        },);
        expect(caught,).toBe(failure,);
        expect(source,).toHaveBeenCalledTimes(reason === 'cancelled' ? 0 : 1,);
      }, },);
    },),
  ], },),
], },);

//endregion Fresh availability decisions.
