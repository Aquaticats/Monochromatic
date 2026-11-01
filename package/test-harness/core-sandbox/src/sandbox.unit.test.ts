/** Built sandbox factories preserve standalone disposal and injected attempt ownership. @module */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  createOwnedSandbox,
  createSinon,
  NO_SANDBOX_OWNER,
  SandboxCleanupError,
  SandboxOwnershipError,
  type SandboxOwner,
  type SandboxRuntime,
} from '@monochromatic-dev/test-harness-core-sandbox';

/** Ordinary runtime used without the runner's Node observation context. */
const ordinaryRuntime: SandboxRuntime = {
  contextual: false,
  current(): typeof NO_SANDBOX_OWNER { return NO_SANDBOX_OWNER; },
  isProxy(): boolean { return false; },
  run({ body, }): Promise<void> { return body(); },
};

await describe({
  name: '',
  children: [
    describe({
      name: createSinon.name,
      children: [
        it({
          name: 'restores the original method on synchronous scope exit',
          fn: async () => {
            const target = { method: (): string => 'original', };
            {
              using sandbox = createSinon();
              sandbox.stub(target, 'method',).returns('stubbed',);
              expect(target.method(),).toBe('stubbed',);
            }
            expect(target.method(),).toBe('original',);
          },
        },),
        it({
          name: 'accepts configuration and restores on asynchronous scope exit',
          fn: async () => {
            const target = { method: (): string => 'original', };
            {
              await using sandbox = createSinon({
                injectInto: null,
                properties: [],
                useFakeTimers: false,
                assertOptions: {},
              },);
              sandbox.stub(target, 'method',).returns('stubbed',);
              expect(target.method(),).toBe('stubbed',);
            }
            expect(target.method(),).toBe('original',);
          },
        },),
      ],
    },),
    describe({
      name: createOwnedSandbox.name,
      children: [
        it({
          name: 'uses injected ownership and closes retained capabilities before restoring',
          fn: async () => {
            const owner: SandboxOwner = { name: 'standalone attempt', l: tagged({ tag: 'sandbox fixture', },), phase: 'running', };
            const target = { method: (): string => 'original', };
            using sandbox = createOwnedSandbox({ owner, runtime: ordinaryRuntime, },);
            const stub = sandbox.sinon.stub(target, 'method',).returns('stubbed',);
            expect(target.method(),).toBe('stubbed',);
            sandbox.sinon.restore();
            expect(owner.phase,).toBe('running',);
            expect(target.method(),).toBe('original',);
            sandbox.sinon.stub(target, 'method',).returns('restubbed',);
            sandbox[Symbol.dispose]();
            expect(owner.phase,).toBe('completed',);
            expect(target.method(),).toBe('original',);
            expect(() => sandbox.sinon.stub(target, 'method',),).toThrow(SandboxOwnershipError,);
            expect(() => stub.value(() => 'late',),).toThrow(SandboxOwnershipError,);
            // Local fake configuration remains available without reattaching the restored method.
            stub.returns('detached',);
            expect(stub(),).toBe('detached',);
            sandbox[Symbol.dispose]();
            expect(target.method(),).toBe('original',);
          },
        },),
      ],
    },),
    it({
      name: 'exports distinct ownership and cleanup errors without changing their causes',
      fn: async () => {
        const ownership = new SandboxOwnershipError('completed attempt',);
        const cleanup = new SandboxCleanupError([ownership,], 'cleanup failed',);
        expect(ownership.name,).toBe('SandboxOwnershipError',);
        expect(cleanup.name,).toBe('SandboxCleanupError',);
        expect(cleanup,).toBeInstanceOf(AggregateError,);
        expect(cleanup.errors,).toEqual([ownership,],);
      },
    },),
  ],
},);
