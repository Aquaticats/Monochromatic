/**
 Tests for the error classes and the condition helper.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CheckFailureError,
  expect as expectThat,
  SafetyStopError,
} from './errors.ts';

await describe({
  name: expectThat.name,
  children: [
    it({
      name: 'does nothing when the condition holds',
      fn: async ({ expect, }) => {
        expect(() => expectThat({ condition: true, message: 'unused', },),).not.toThrow();
      },
    },),
    it({
      name: 'throws a check failure with the message when the condition does not hold',
      fn: async ({ expect, }) => {
        expect(() => expectThat({ condition: false, message: 'the ring is off', },),).toThrow('the ring is off',);
        try {
          expectThat({ condition: false, message: 'x', },);
        }
        catch (error: unknown) {
          expect(error,).toBeInstanceOf(CheckFailureError,);
          expect(error,).not.toBeInstanceOf(SafetyStopError,);
        }
      },
    },),
  ],
},);

await describe({
  name: 'error classes',
  children: [
    it({
      name: 'name themselves so outcomes show their kind',
      fn: async ({ expect, }) => {
        expect(new CheckFailureError('a',).name,).toBe('CheckFailureError',);
        expect(new SafetyStopError('b',).name,).toBe('SafetyStopError',);
        expect(new SafetyStopError('b',),).toBeInstanceOf(Error,);
      },
    },),
  ],
},);
