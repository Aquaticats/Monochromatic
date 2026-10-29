import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createPublicationInterruptControl,
  type PublicationSignalHost,
} from '../dist/final/node/index.mjs';

await describe({
  name: createPublicationInterruptControl.name,
  children: [
    it({
      name: 'stops on first interrupt and forces on second',
      fn: async () => {
        /**
         Mutable fake signal host state.
         */
        const state: {
          listener?: () => void;
          forced: number;
          removed: number;
        } = {
          forced: 0,
          removed: 0,
        };
        /**
         Fake process signal host.
         */
        const host: PublicationSignalHost = {
          onInterrupt(listener,) {
            state.listener = listener;
          },
          offInterrupt() {
            state.removed += 1;
          },
          forceInterrupt() {
            state.forced += 1;
          },
        };

        {
          using control = createPublicationInterruptControl({ host, });
          if (state.listener === undefined) {
            throw new Error('interrupt listener was not installed',);
          }
          state.listener();
          expect(control.shouldStop(),).toBe(true,);
          state.listener();
          expect(state.forced,).toBe(1,);
        }
        expect(state.removed,).toBe(2,);
      },
    },),
  ],
},);
