/**
 Tests for `createObservableCore`, the container both observable variants wrap.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createObservableCore,
  type ObservableContainer,
} from '../dist/final/node/index.mjs';

await describe({
  name: createObservableCore.name,
  children: [
    it({
      name: 'getValue returns the initial value',
      fn: async () => {
        const container = createObservableCore({
          initialValue: 1,
          notify: function notify(): void {},
        },);
        expect(container.getValue(),).toBe(1,);
      },
    },),

    it({
      name: 'setValue reports the new value, then the one it replaced',
      fn: async ({ sinon, },) => {
        const notify = sinon.spy(function notify(
          _newValue: string,
          _previous: string,
        ): void {},);
        const container = createObservableCore({
          initialValue: 'a',
          notify,
        },);
        container.setValue('b',);
        expect(notify,).toHaveBeenCalledWith('b', 'a',);
      },
    },),

    it({
      name: 'the container holds the new value when the notification runs',
      fn: async ({ sinon, },) => {
        /** Mutable slot so the notification reads the container assigned after construction. */
        const holder: { container?: ObservableContainer<number, void>; } = {};
        const notify = sinon.spy(function notify(newValue: number,): void {
          expect(holder.container?.getValue(),).toBe(newValue,);
        },);
        holder.container = createObservableCore({
          initialValue: 0,
          notify,
        },);
        holder.container.setValue(5,);
        expect(notify,).toHaveBeenCalledTimes(1,);
      },
    },),

    it({
      name: 'setValue returns what the notification returns',
      fn: async () => {
        const container = createObservableCore({
          initialValue: 0,
          notify: function notify(): string {
            return 'reported';
          },
        },);
        expect(container.setValue(1,),).toBe('reported',);
      },
    },),

    it({
      name: 'two containers keep separate values',
      fn: async () => {
        const first = createObservableCore({
          initialValue: 1,
          notify: function notify(): void {},
        },);
        const second = createObservableCore({
          initialValue: 2,
          notify: function notify(): void {},
        },);
        first.setValue(3,);
        expect(first.getValue(),).toBe(3,);
        expect(second.getValue(),).toBe(2,);
      },
    },),
  ],
},);
