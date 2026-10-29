/**
 Tests for `createObservable`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createObservable,
  type Observable,
} from './create-observable.ts';

await describe({
  name: createObservable.name,
  children: [
    it({
      name: 'getValue returns the initial value, then the updated value',
      fn: async () => {
        const obs = createObservable({
          initialValue: 1,
          onChange: function onChange(): void {},
        },);
        expect(obs.getValue(),).toBe(1,);
        obs.setValue(2,);
        expect(obs.getValue(),).toBe(2,);
      },
    },),

    it({
      name: 'setValue invokes onChange with the new value, then the previous value',
      fn: async ({ sinon, },) => {
        const onChange = sinon.spy(function onChange(
          _newValue: string,
          _oldValue: string,
        ): void {},);
        const obs = createObservable({
          initialValue: 'a',
          onChange,
        },);
        obs.setValue('b',);
        expect(onChange,).toHaveBeenCalledWith('b', 'a',);
      },
    },),

    it({
      name: 'state is already updated when onChange runs',
      fn: async ({ sinon, },) => {
        /** Mutable slot so the handler reads the observable assigned after construction, avoiding use-before-define. */
        const holder: { observable?: Observable<number>; } = {};
        const onChange = sinon.spy(function onChange(newValue: number,): void {
          expect(holder.observable?.getValue(),).toBe(newValue,);
        },);
        holder.observable = createObservable({
          initialValue: 0,
          onChange,
        },);
        holder.observable.setValue(5,);
        expect(onChange,).toHaveBeenCalledTimes(1,);
      },
    },),

    it({
      name: 'setValue propagates a thrown error from onChange synchronously',
      fn: async () => {
        const obs = createObservable({
          initialValue: 0,
          onChange: function onChange(): void {
            throw new Error('handler failed',);
          },
        },);
        expect(function trigger(): void {
          obs.setValue(1,);
        },).toThrow('handler failed',);
      },
    },),
  ],
},);
