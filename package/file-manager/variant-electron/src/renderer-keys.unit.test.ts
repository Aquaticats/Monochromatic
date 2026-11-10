/**
 Unit tests for the strip keyboard dispatch.
 
 Cover both outcomes of `handleStripKey`: a key the lookup names is prevented
 and its action runs, and a key the lookup does not name is ignored. Tests
 import from built `dist/app` so they verify the artifact the Electron
 renderer consumes, not sibling source.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { handleStripKey, } from '../dist/app/renderer-keys.js';

/**
 Builds a keydown event stand-in reporting one key name.
 
 Node has no DOM, so the dispatch contract is exercised with the only two
 event members the handler reads: the key name and `preventDefault`.
 
 @param key - Key name the event reports.
 
 @returns Event stand-in plus a reader for its prevented flag.
 */
function fakeKeydown({ key, }: { readonly key: string; },): {
  readonly event: KeyboardEvent;
  readonly wasPrevented: () => boolean;
} {
  /**
   Whether the dispatched handler called `preventDefault` on this event.
   */
  let prevented = false;

  return {
    event: {
      key,
      preventDefault(): void {
        prevented = true;
      },
    } as unknown as KeyboardEvent,
    wasPrevented(): boolean {
      return prevented;
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: handleStripKey.name,
      children: [
        it({
          name: 'prevents a named key and runs its action',
          fn: async (): Promise<void> => {
            /**
             Count of action runs the dispatch triggered.
             */
            let runs = 0;
            const { event, wasPrevented, } = fakeKeydown({ key: 'Backspace', },);

            handleStripKey({
              actions: {
                Backspace(): void {
                  runs += 1;
                },
              },
              event,
            },);

            expect(runs,).toBe(1,);
            expect(wasPrevented(),).toBe(true,);
          },
        },),
        it({
          name: 'ignores an unnamed key without preventing it',
          fn: async (): Promise<void> => {
            /**
             Count of action runs the dispatch triggered.
             */
            let runs = 0;
            const { event, wasPrevented, } = fakeKeydown({ key: 'Tab', },);

            handleStripKey({
              actions: {
                Backspace(): void {
                  runs += 1;
                },
              },
              event,
            },);

            expect(runs,).toBe(0,);
            expect(wasPrevented(),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
