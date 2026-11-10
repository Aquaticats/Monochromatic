/**
 Tests for the autofill controller's request handling: a rejected suggestion request must reach the person through the shared API wrapper, and an accepted one must still fill the empty metadata fields.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Duration the controller waits after the last keystroke, read from the module that owns it.
 */
const { AUTOFILL_DEBOUNCE_MS, } = await import('./task-detail-types.ts');

//region Fakes

/**
 Toast element the page appended, recorded so a test can read its message.
 */
type RecordedToast = {
  readonly tagName: string;
  readonly attributes: ReadonlyMap<string, string>;
};

/**
 Constructor stand-in for the DOM base class the toast component extends.
 */
function FakeHTMLElement(): void {
  // No instance is ever built: the stand-in only lets the component module define its class.
}

/**
 Build the element stand-in the toast component manipulates.

 @param tagName - name the page created it under

 @returns element stand-in recording the attributes the toast component sets
 */
function createFakeElement(tagName: string,) {
  const attributes = new Map<string, string>();
  return {
    tagName,
    attributes,
    setAttribute: function setAttribute(name: string, value: string,): void {
      attributes.set(name, value,);
    },
    remove: function remove(): void {
      // The page stand-in holds no children between toasts.
    },
  };
}

/**
 Install the browser globals the shared API wrapper needs before it is imported.

 WHY: the wrapper imports the toast component, whose module extends `HTMLElement` and registers a custom element while it loads.

 @returns appended toasts and a disposal restoring the previous globals
 */
function installFakePage() {
  const originalDocument = Reflect.get(globalThis, 'document',);
  const originalHTMLElement = Reflect.get(globalThis, 'HTMLElement',);
  const originalCustomElements = Reflect.get(globalThis, 'customElements',);
  const toasts: RecordedToast[] = [];

  Reflect.set(globalThis, 'HTMLElement', FakeHTMLElement,);
  Reflect.set(globalThis, 'customElements', {
    define: function define(): void {
      // Registration is what the module does at load; the page stand-in holds no registry.
    },
  },);
  Reflect.set(globalThis, 'document', {
    querySelector: function querySelector(): undefined {
      return undefined;
    },
    createElement: function createElement(tagName: string,) {
      return createFakeElement(tagName,);
    },
    body: {
      append: function append(child: RecordedToast,): void {
        toasts.push(child,);
      },
    },
  },);

  return {
    toasts,
    [Symbol.dispose](): void {
      Reflect.set(globalThis, 'document', originalDocument,);
      Reflect.set(globalThis, 'HTMLElement', originalHTMLElement,);
      Reflect.set(globalThis, 'customElements', originalCustomElements,);
    },
  };
}

/**
 One request the controller sent, read the way the person's failure report needs it.
 */
type RecordedRequest = {
  readonly path: string;
  readonly method: string;
  readonly hasAbortSignal: boolean;
  readonly contentType: string;
};

/**
 Read the path a fetch stand-in was called with.

 @param input - request target the caller passed

 @returns the target as text
 */
function requestPath(input: string | URL | Request,): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

/**
 Install a fetch stand-in that records each request and answers with one response.

 @param respond - response every request receives

 @returns recorded requests and a disposal restoring the previous fetch
 */
function installFetch(respond: () => Response,) {
  const originalFetch = globalThis.fetch;
  const requests: RecordedRequest[] = [];

  globalThis.fetch = async function fakeFetch(input, init,) {
    const headers = new Headers(init?.headers,);
    requests.push({
      path: requestPath(input,),
      method: init?.method ?? 'GET',
      hasAbortSignal: init?.signal instanceof AbortSignal,
      contentType: headers.get('content-type',) ?? '',
    },);
    return respond();
  };

  return {
    requests,
    [Symbol.dispose](): void {
      globalThis.fetch = originalFetch;
    },
  };
}

//endregion Fakes

await describe({
  name: 'createAutofillController',
  // Both tests replace the same page and fetch globals, so they run one at a time.
  concurrency: 1,
  children: [
    it({
      name: 'reports a rejected suggestion request instead of dropping it',
      fn: async function runRejectedRequest() {
        using page = installFakePage();
        using fetchStandIn = installFetch(function rateLimited(): Response {
          return Response.json(
            { error: 'AI suggestions are rate limited', },
            { status: 429, },
          );
        },);
        const { createAutofillController, } = await import('./task-detail-autofill.ts');
        const updates: unknown[] = [];
        const controller = createAutofillController({
          getState: function getState() {
            return {
              tags: [],
              locations: [],
              priority: 'low',
              complexity: 'low',
            };
          },
          setState: function setState(update,) {
            updates.push(update,);
          },
          updateDisplay: function updateDisplay(): void {
            // The component rerenders; the controller's state is what this test reads.
          },
        },);

        controller.request('Write the quarterly report',);
        await new Promise(function settle(resolve,): void {
          setTimeout(resolve, AUTOFILL_DEBOUNCE_MS + 250,);
        },);

        expect(fetchStandIn.requests.map(function toPath(request,) {
          return request.path;
        },),).toEqual(['/api/ai/autofill',],);
        expect(fetchStandIn.requests[0]?.hasAbortSignal,).toBe(true,);
        expect(page.toasts.map(function toMessage(toast,) {
          return toast.attributes.get('message',);
        },),).toEqual(['AI suggestions are rate limited',],);
        expect(updates,).toEqual([],);
        expect(controller.loading,).toBe(false,);
      },
    },),
    it({
      name: 'fills empty metadata fields from an accepted suggestion',
      fn: async function runAcceptedRequest() {
        using page = installFakePage();
        using fetchStandIn = installFetch(function suggestions(): Response {
          return Response.json({
            tags: ['finance',],
            locations: ['office',],
          },);
        },);
        const { createAutofillController, } = await import('./task-detail-autofill.ts');
        const updates: unknown[] = [];
        const controller = createAutofillController({
          getState: function getState() {
            return {
              tags: [],
              locations: [],
              priority: 'low',
              complexity: 'low',
            };
          },
          setState: function setState(update,) {
            updates.push(update,);
          },
          updateDisplay: function updateDisplay(): void {
            // The component rerenders; the controller's state is what this test reads.
          },
        },);

        controller.request('Write the quarterly report',);
        await new Promise(function settle(resolve,): void {
          setTimeout(resolve, AUTOFILL_DEBOUNCE_MS + 250,);
        },);

        expect(updates,).toEqual([{
          tags: ['finance',],
          locations: ['office',],
        },],);
        expect(fetchStandIn.requests[0]?.method,).toBe('POST',);
        expect(fetchStandIn.requests[0]?.contentType,).toBe('application/json',);
        expect(page.toasts,).toEqual([],);
      },
    },),
  ],
},);
