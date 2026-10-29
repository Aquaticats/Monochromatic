/**
 Tests for `createObservableAsync`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createObservableAsync,
  type ObservableAsync,
} from './create-observable-async.ts';

await describe({
  name: createObservableAsync.name,
  children: [
    it({
      name: 'getValue returns the initial value, then the updated value after await setValue',
      fn: async () => {
        const obs = await createObservableAsync({
          initialValue: 1,
          onChange: function onChange(): void {},
        },);
        expect(obs.getValue(),).toBe(1,);
        await obs.setValue(2,);
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
        const obs = await createObservableAsync({
          initialValue: 'a',
          onChange,
        },);
        await obs.setValue('b',);
        expect(onChange,).toHaveBeenCalledWith('b', 'a',);
      },
    },),

    it({
      name: 'state is already updated when onChange runs',
      fn: async ({ sinon, },) => {
        /** Mutable slot so the handler reads the observable assigned after construction, avoiding use-before-define. */
        const holder: { observable?: ObservableAsync<number>; } = {};
        const onChange = sinon.spy(function onChange(newValue: number,): void {
          expect(holder.observable?.getValue(),).toBe(newValue,);
        },);
        holder.observable = await createObservableAsync({
          initialValue: 0,
          onChange,
        },);
        await holder.observable.setValue(5,);
        expect(onChange,).toHaveBeenCalledTimes(1,);
      },
    },),

    it({
      name: 'await setValue waits for an async onChange to finish',
      fn: async () => {
        /** Effect flag the async handler flips only after its internal await, observed before and after awaiting setValue. */
        const effect: { done: boolean; } = { done: false, };
        const obs = await createObservableAsync({
          initialValue: 0,
          onChange: async function onChange(): Promise<void> {
            await Promise.resolve();
            effect.done = true;
          },
        },);
        /** Pending setValue promise captured before awaiting, to prove the effect is not yet visible. */
        const pending = obs.setValue(1,);
        expect(effect.done,).toBe(false,);
        await pending;
        expect(effect.done,).toBe(true,);
      },
    },),

    it({
      name: 'a rejecting async onChange causes await setValue to reject',
      fn: async () => {
        const obs = await createObservableAsync({
          initialValue: 0,
          onChange: async function onChange(): Promise<void> {
            throw new Error('async handler failed',);
          },
        },);
        await expect(obs.setValue(1,),).rejects.toThrow('async handler failed',);
      },
    },),
  ],
},);
