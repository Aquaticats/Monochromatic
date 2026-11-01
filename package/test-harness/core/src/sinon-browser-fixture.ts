/**
 Browser consumer probes import the built neutral harness, not its source implementation. @module
 */
import {
  describe,
  expect,
  formatFailure,
  it,
  type TestContext,
} from '../dist/final/neutral/index.mjs';

/**
 Serialized acceptance evidence returned across the Playwright page boundary.
 */
export type BrowserSandboxEvidence = {
  /**
   Target was restored after every browser attempt.
   */
  readonly restored: boolean;
  /**
   Distinct contexts observed across repeats.
   */
  readonly contextCount: number;
  /**
   Node observer must remain absent in this realm.
   */
  readonly nodeObserverInstalled: boolean;
  /**
   Identifies the synthetic browser environment exercised.
   */
  readonly processShim: boolean;
};

/**
 Exercises ordinary browser Sinon behavior and completed-attempt guards without Node globals.

 @param processShim - whether a non-Node process shim should be present
 
 @returns measured browser state after all attempts finish
 
 @example
 ```ts
 await runBrowserSandboxProbe({ processShim: false });
 ```
 */
export async function runBrowserSandboxProbe({ processShim, }: {
  readonly processShim: boolean;
},): Promise<BrowserSandboxEvidence> {
  if (processShim)
    Object.defineProperty(
      globalThis,
      'process',
      {
        configurable: true,
        value: { env: {}, },
      },
    );
  else
    expect(Reflect.has(
      globalThis,
      'process',
    ),)
      .toBe(false,);
  /**
   Each page owns an independent target and browser storage context.
   */
  const target = {
    method(input: string,): string {
      return `original:${input}`;
    },
  };
  /**
   Ordinary browser restoration must retain the exact original method descriptor.
   */
  const original = Object.getOwnPropertyDescriptor(
    target,
    'method',
  );
  /**
   Factories are retained past successful, failing, and repeated attempts.
   */
  const late: (() => unknown)[] = [];
  /**
   Repeats must not reuse their sandbox context in the browser fallback either.
   */
  const contexts = new Set<TestContext>();
  await describe({
    name: 'browser sandbox fallback',
    concurrency: 1,
    children: [
      it({
        name: 'ordinary method stub',
        fn: function ordinaryStub({ sinon, }: TestContext,): Promise<void> {
          sinon.stub(
            target,
            'method',
          )
            .returns('stubbed',);
          expect(target.method('input',),)
            .toBe('stubbed',);
          /**
           The browser fallback installs a data property, not a contextual accessor.
           */
          const descriptor = Object.getOwnPropertyDescriptor(
            target,
            'method',
          );
          if (descriptor === undefined)
            throw new Error('Browser method descriptor disappeared',);
          expect(Object.hasOwn(
            descriptor,
            'get',
          ),)
            .toBe(false,);
          late.push(function lateStub(): unknown {
            return sinon.stub(
              target,
              'method',
            );
          },);
          return Promise.resolve();
        },
      },),
      it({
        name: 'ordinary method spy',
        fn: function ordinarySpy({ sinon, }: TestContext,): Promise<void> {
          /**
           The previous attempt has fully restored before this ordinary replacement.
           */
          const spy = sinon.spy(
            target,
            'method',
          );
          expect(target.method('spy',),)
            .toBe('original:spy',);
          expect(spy.callCount,)
            .toBe(1,);
          return Promise.resolve();
        },
      },),
      it({
        name: 'ordinary accessor configuration',
        fn: function ordinaryAccessor({ sinon, }: TestContext,): Promise<void> {
          /**
           Non-method descriptor mutation remains a Sinon operation.
           */
          const data = { value: 'original', };
          /**
           Deferred setters must retain the same lifecycle guard in browsers.
           */
          const stub = sinon.stub(
            data,
            'value',
          )
            .value('replacement',);
          expect(data.value,)
            .toBe('replacement',);
          late.push(function lateValue(): unknown {
            return stub.value('late',);
          },);
          return Promise.resolve();
        },
      },),
      it({
        name: 'failed body restores',
        fails: true,
        fn: function failingBody({ sinon, }: TestContext,): Promise<void> {
          sinon.stub(
            target,
            'method',
          );
          return Promise.reject(new Error('Expected browser fixture failure',),);
        },
      },),
      it({
        name: 'fresh repeated contexts',
        repeats: 1,
        fn: function repeatedBody(context: TestContext,): Promise<void> {
          contexts.add(context,);
          context.sinon
            .stub(
            target,
            'method',
            );
          late.push(function lateFake(): unknown {
            return context.sinon
              .fake
              .returns('late',);
          },);
          return Promise.resolve();
        },
      },),
    ],
  },);
  for (const invoke of late)
    expect(invoke,)
      .toThrow('completed',);
  expect(Object.getOwnPropertyDescriptor(
    target,
    'method',
  ),)
    .toEqual(original,);
  expect(target.method('restored',),)
    .toBe('original:restored',);
  /**
   Error formatting must not load workspace filesystem discovery in a browser.
   */
  const diagnostic = await formatFailure({
    summary: 'browser diagnostic',
    value: new Error('browser detail',),
  },);
  expect(diagnostic,)
    .toContain('browser detail',);
  return {
    restored: target.method('restored',) === 'original:restored',
    contextCount: contexts.size,
    nodeObserverInstalled: Reflect.has(
      globalThis,
      Symbol.for('@monochromatic-dev/test-harness-core/async-failure-runtime/v1',),
    ),
    processShim,
  };
}
